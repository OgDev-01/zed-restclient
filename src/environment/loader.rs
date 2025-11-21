//! Environment file loader for REST Client
//!
//! This module handles loading environment configuration files from the workspace.
//! It searches for .http-client-env.json or http-client.env.json files starting
//! from the workspace root and traversing up to 3 parent directories.

use super::models::{Environment, Environments};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Errors that can occur during environment loading
#[derive(Debug, Clone, PartialEq)]
pub enum EnvError {
    /// Environment file was not found in workspace or parent directories
    FileNotFound,

    /// Failed to parse JSON content
    ParseError(String),

    /// Invalid format or structure in the environment file
    InvalidFormat(String),

    /// IO error occurred while reading file
    IoError(String),
}

impl std::fmt::Display for EnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvError::FileNotFound => write!(
                f,
                "Environment file not found (.http-client-env.json or http-client.env.json)"
            ),
            EnvError::ParseError(msg) => write!(f, "Failed to parse environment file: {}", msg),
            EnvError::InvalidFormat(msg) => write!(f, "Invalid environment format: {}", msg),
            EnvError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for EnvError {}

impl From<io::Error> for EnvError {
    fn from(err: io::Error) -> Self {
        EnvError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for EnvError {
    fn from(err: serde_json::Error) -> Self {
        EnvError::ParseError(err.to_string())
    }
}

/// Supported environment file names in order of preference
const ENV_FILE_NAMES: &[&str] = &[".http-client-env.json", "http-client.env.json"];

/// Maximum number of parent directories to search
const MAX_PARENT_SEARCH_DEPTH: usize = 3;

/// Loads environment configuration from workspace
///
/// Searches for environment files starting from the workspace path and
/// traversing up to 3 parent directories. Returns an empty Environments
/// struct if no file is found (graceful fallback).
///
/// # Arguments
///
/// * `workspace_path` - The root workspace directory to start searching from
///
/// # Returns
///
/// * `Ok(Environments)` - Loaded environments or empty if file not found
/// * `Err(EnvError)` - If file exists but parsing failed
pub fn load_environments(workspace_path: &Path) -> Result<Environments, EnvError> {
    // Search for environment file
    let env_file = match find_environment_file(workspace_path) {
        Some(path) => path,
        None => {
            // Gracefully return empty environments if file not found
            return Ok(Environments::new());
        }
    };

    // Read file content
    let content = fs::read_to_string(&env_file)?;

    // Parse JSON into raw structure
    let raw: serde_json::Value = serde_json::from_str(&content)?;

    // Validate and convert to Environments struct
    parse_environment_file(raw)
}

/// Finds the environment file by searching workspace and parent directories
fn find_environment_file(workspace_path: &Path) -> Option<PathBuf> {
    let mut current_path = workspace_path.to_path_buf();

    for _ in 0..=MAX_PARENT_SEARCH_DEPTH {
        // Try each supported filename
        for filename in ENV_FILE_NAMES {
            let candidate = current_path.join(filename);
            if candidate.exists() && candidate.is_file() {
                return Some(candidate);
            }
        }

        // Move to parent directory
        match current_path.parent() {
            Some(parent) => current_path = parent.to_path_buf(),
            None => break, // Reached filesystem root
        }
    }

    None
}

/// Parses the raw JSON into validated Environments structure
fn parse_environment_file(raw: serde_json::Value) -> Result<Environments, EnvError> {
    let obj = raw
        .as_object()
        .ok_or_else(|| EnvError::InvalidFormat("Root must be a JSON object".to_string()))?;

    let mut environments = HashMap::new();
    let mut shared = HashMap::new();
    let mut active = None;

    for (key, value) in obj.iter() {
        match key.as_str() {
            // Special key for shared variables
            "shared" | "$shared" => {
                shared = parse_variable_map(value, "shared")?;
            }

            // Special key for active environment (optional)
            "active" | "$active" => {
                active = value.as_str().map(|s| s.to_string()).or_else(|| {
                    // If not a string, ignore it
                    None
                });
            }

            // Everything else is treated as an environment
            env_name => {
                // Validate environment name is a valid identifier
                if !is_valid_identifier(env_name) {
                    return Err(EnvError::InvalidFormat(format!(
                        "Invalid environment name: '{}'. Names must be alphanumeric with underscores/hyphens",
                        env_name
                    )));
                }

                let variables = parse_variable_map(value, env_name)?;

                environments.insert(
                    env_name.to_string(),
                    Environment {
                        name: env_name.to_string(),
                        variables,
                    },
                );
            }
        }
    }

    // Validate active environment exists if specified
    if let Some(ref active_name) = active {
        if !environments.contains_key(active_name) {
            return Err(EnvError::InvalidFormat(format!(
                "Active environment '{}' does not exist",
                active_name
            )));
        }
    }

    Ok(Environments {
        environments,
        shared,
        active,
    })
}

/// Parses a JSON value into a variable map (HashMap<String, String>)
fn parse_variable_map(
    value: &serde_json::Value,
    context: &str,
) -> Result<HashMap<String, String>, EnvError> {
    let obj = value
        .as_object()
        .ok_or_else(|| EnvError::InvalidFormat(format!("'{}' must be a JSON object", context)))?;

    let mut map = HashMap::new();

    for (key, val) in obj.iter() {
        // Convert value to string
        let value_str = match val {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            _ => {
                return Err(EnvError::InvalidFormat(format!(
                    "Variable '{}' in '{}' has invalid type (must be string, number, or boolean)",
                    key, context
                )));
            }
        };

        map.insert(key.clone(), value_str);
    }

    Ok(map)
}

/// Validates that a string is a valid identifier for environment names
///
/// Valid identifiers must:
/// - Start with a letter or underscore
/// - Contain only letters, numbers, underscores, or hyphens
/// - Not be a reserved keyword (shared, active, or with $ prefix)
fn is_valid_identifier(name: &str) -> bool {
    // Reserved keywords
    if name == "shared" || name == "active" || name.starts_with('$') {
        return false;
    }

    if name.is_empty() {
        return false;
    }

    // First character must be letter or underscore
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Rest can be alphanumeric, underscore, or hyphen
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_env_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_load_environments_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_environments(temp_dir.path()).unwrap();

        // Should return empty environments gracefully
        assert!(result.is_empty());
        assert!(result.shared.is_empty());
        assert!(result.active.is_none());
    }

    #[test]
    fn test_load_environments_simple() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "dev": {
                "baseUrl": "http://localhost:3000",
                "apiKey": "dev-key-123"
            },
            "prod": {
                "baseUrl": "https://api.example.com",
                "apiKey": "prod-key-456"
            }
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let envs = load_environments(temp_dir.path()).unwrap();

        assert_eq!(envs.len(), 2);
        assert!(envs.has_environment("dev"));
        assert!(envs.has_environment("prod"));

        let dev = envs.get_environment("dev").unwrap();
        assert_eq!(dev.get("baseUrl").unwrap(), "http://localhost:3000");
        assert_eq!(dev.get("apiKey").unwrap(), "dev-key-123");
    }

    #[test]
    fn test_load_environments_with_shared() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "shared": {
                "contentType": "application/json",
                "version": "v1"
            },
            "dev": {
                "baseUrl": "http://localhost:3000"
            }
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let envs = load_environments(temp_dir.path()).unwrap();

        assert_eq!(envs.shared.len(), 2);
        assert_eq!(envs.shared.get("contentType").unwrap(), "application/json");
        assert_eq!(envs.shared.get("version").unwrap(), "v1");
    }

    #[test]
    fn test_load_environments_with_active() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "dev": {
                "baseUrl": "http://localhost:3000"
            },
            "prod": {
                "baseUrl": "https://api.example.com"
            },
            "active": "dev"
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let envs = load_environments(temp_dir.path()).unwrap();

        assert_eq!(envs.active.as_ref().unwrap(), "dev");
        assert_eq!(
            envs.get_active().unwrap().get("baseUrl").unwrap(),
            "http://localhost:3000"
        );
    }

    #[test]
    fn test_load_environments_alternative_filename() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "dev": {
                "url": "http://localhost"
            }
        }"#;

        create_temp_env_file(temp_dir.path(), "http-client.env.json", content);

        let envs = load_environments(temp_dir.path()).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.has_environment("dev"));
    }

    #[test]
    fn test_find_environment_file_in_parent() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("sub");
        fs::create_dir(&sub_dir).unwrap();

        let content = r#"{"dev": {"url": "http://localhost"}}"#;
        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        // Load from subdirectory should find parent file
        let envs = load_environments(&sub_dir).unwrap();

        assert_eq!(envs.len(), 1);
        assert!(envs.has_environment("dev"));
    }

    #[test]
    fn test_find_environment_file_max_depth() {
        let temp_dir = TempDir::new().unwrap();

        // Create deeply nested structure
        let mut current = temp_dir.path().to_path_buf();
        for i in 0..5 {
            current = current.join(format!("level{}", i));
            fs::create_dir(&current).unwrap();
        }

        // Place file at root
        let content = r#"{"dev": {"url": "http://localhost"}}"#;
        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        // Should find file within MAX_PARENT_SEARCH_DEPTH (3)
        let level3 = temp_dir.path().join("level0/level1/level2");
        let envs = load_environments(&level3).unwrap();
        assert_eq!(envs.len(), 1);

        // Should NOT find file beyond MAX_PARENT_SEARCH_DEPTH
        let level5 = temp_dir.path().join("level0/level1/level2/level3/level4");
        let envs = load_environments(&level5).unwrap();
        assert_eq!(envs.len(), 0); // Empty, file not found
    }

    #[test]
    fn test_parse_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let content = "not valid json {";

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let result = load_environments(temp_dir.path());
        assert!(matches!(result, Err(EnvError::ParseError(_))));
    }

    #[test]
    fn test_parse_invalid_format_not_object() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"["not", "an", "object"]"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let result = load_environments(temp_dir.path());
        assert!(matches!(result, Err(EnvError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "123-invalid": {
                "url": "http://localhost"
            }
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let result = load_environments(temp_dir.path());
        assert!(matches!(result, Err(EnvError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_invalid_active_environment() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "dev": {
                "url": "http://localhost"
            },
            "active": "nonexistent"
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let result = load_environments(temp_dir.path());
        assert!(matches!(result, Err(EnvError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_variable_types() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "dev": {
                "stringVar": "hello",
                "numberVar": 42,
                "boolVar": true,
                "nullVar": null
            }
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let envs = load_environments(temp_dir.path()).unwrap();
        let dev = envs.get_environment("dev").unwrap();

        assert_eq!(dev.get("stringVar").unwrap(), "hello");
        assert_eq!(dev.get("numberVar").unwrap(), "42");
        assert_eq!(dev.get("boolVar").unwrap(), "true");
        assert_eq!(dev.get("nullVar").unwrap(), "");
    }

    #[test]
    fn test_is_valid_identifier() {
        // Valid identifiers
        assert!(is_valid_identifier("dev"));
        assert!(is_valid_identifier("Dev"));
        assert!(is_valid_identifier("_dev"));
        assert!(is_valid_identifier("dev_123"));
        assert!(is_valid_identifier("dev-staging"));
        assert!(is_valid_identifier("my_env_123"));

        // Invalid identifiers
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123dev")); // Starts with number
        assert!(!is_valid_identifier("dev env")); // Contains space
        assert!(!is_valid_identifier("dev.test")); // Contains dot
        assert!(!is_valid_identifier("shared")); // Reserved
        assert!(!is_valid_identifier("active")); // Reserved
        assert!(!is_valid_identifier("$shared")); // Reserved prefix
    }

    #[test]
    fn test_dollar_prefix_shared() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "$shared": {
                "version": "v1"
            },
            "dev": {
                "url": "http://localhost"
            }
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let envs = load_environments(temp_dir.path()).unwrap();

        assert_eq!(envs.shared.len(), 1);
        assert_eq!(envs.shared.get("version").unwrap(), "v1");
    }

    #[test]
    fn test_variable_with_references() {
        let temp_dir = TempDir::new().unwrap();
        let content = r#"{
            "dev": {
                "baseUrl": "http://localhost:3000",
                "apiUrl": "{{baseUrl}}/api",
                "loginUrl": "{{apiUrl}}/login"
            }
        }"#;

        create_temp_env_file(temp_dir.path(), ".http-client-env.json", content);

        let envs = load_environments(temp_dir.path()).unwrap();
        let dev = envs.get_environment("dev").unwrap();

        // Variables are stored as-is with {{}} syntax
        // Substitution happens later in the variable resolver
        assert_eq!(dev.get("apiUrl").unwrap(), "{{baseUrl}}/api");
        assert_eq!(dev.get("loginUrl").unwrap(), "{{apiUrl}}/login");
    }
}
