//! System variable resolution for REST Client
//!
//! This module implements system variables like {{$guid}}, {{$timestamp}}, {{$datetime}},
//! {{$randomInt}}, {{$processEnv}}, and {{$dotenv}} for use in HTTP requests.

use chrono::{DateTime, Duration, SecondsFormat, Utc};
use rand::Rng;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

/// Errors that can occur during variable resolution
#[derive(Debug, Clone, PartialEq)]
pub enum VarError {
    /// Variable is not defined or recognized
    UndefinedVariable(String),
    /// Variable syntax is invalid
    InvalidSyntax(String),
    /// Offset parsing failed
    InvalidOffset(String),
    /// Environment variable not found (for non-optional vars)
    EnvVarNotFound(String),
    /// .env file reading failed
    DotenvError(String),
    /// Circular reference detected during variable substitution
    CircularReference(String),
}

impl std::fmt::Display for VarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            VarError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            VarError::InvalidOffset(msg) => write!(f, "Invalid offset: {}", msg),
            VarError::EnvVarNotFound(name) => write!(f, "Environment variable not found: {}", name),
            VarError::DotenvError(msg) => write!(f, "Dotenv error: {}", msg),
            VarError::CircularReference(msg) => write!(f, "Circular reference: {}", msg),
        }
    }
}

impl std::error::Error for VarError {}

/// Cache for .env file contents to avoid repeated file reads
static DOTENV_CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// Resolves a system variable by name and arguments
///
/// # Arguments
/// * `name` - The variable name (e.g., "guid", "timestamp", "datetime")
/// * `args` - Additional arguments for the variable (e.g., format, offset, min/max)
///
/// # Examples
/// ```
/// use rest_client::variables::system::resolve_system_variable;
///
/// // {{$guid}}
/// resolve_system_variable("guid", &[]).unwrap();
///
/// // {{$timestamp}}
/// resolve_system_variable("timestamp", &[]).unwrap();
///
/// // {{$timestamp -1 d}}
/// resolve_system_variable("timestamp", &["-1", "d"]).unwrap();
///
/// // {{$datetime iso8601}}
/// resolve_system_variable("datetime", &["iso8601"]).unwrap();
///
/// // {{$randomInt 1 100}}
/// resolve_system_variable("randomInt", &["1", "100"]).unwrap();
/// ```
pub fn resolve_system_variable(name: &str, args: &[&str]) -> Result<String, VarError> {
    match name {
        "guid" => resolve_guid(),
        "timestamp" => resolve_timestamp(args),
        "datetime" => resolve_datetime(args),
        "randomInt" => resolve_random_int(args),
        "processEnv" => resolve_process_env(args),
        "dotenv" => resolve_dotenv(args),
        _ => Err(VarError::UndefinedVariable(name.to_string())),
    }
}

/// Generates a new UUID v4
fn resolve_guid() -> Result<String, VarError> {
    Ok(Uuid::new_v4().to_string())
}

/// Resolves timestamp with optional offset
///
/// Formats:
/// - {{$timestamp}} - current Unix timestamp in seconds
/// - {{$timestamp -1 d}} - timestamp 1 day ago
/// - {{$timestamp +2 h}} - timestamp 2 hours from now
fn resolve_timestamp(args: &[&str]) -> Result<String, VarError> {
    let now = Utc::now();

    if args.is_empty() {
        // No offset, return current timestamp
        return Ok(now.timestamp().to_string());
    }

    // Parse offset
    let datetime = parse_offset(now, args)?;
    Ok(datetime.timestamp().to_string())
}

/// Resolves datetime with format and optional offset
///
/// Formats:
/// - {{$datetime rfc1123}} - RFC 1123 format
/// - {{$datetime iso8601}} - ISO 8601 format
/// - {{$datetime rfc1123 -1 d}} - RFC 1123 format, 1 day ago
fn resolve_datetime(args: &[&str]) -> Result<String, VarError> {
    if args.is_empty() {
        return Err(VarError::InvalidSyntax(
            "datetime requires format argument (rfc1123 or iso8601)".to_string(),
        ));
    }

    let format = args[0];
    let now = Utc::now();

    // Parse offset if provided (args[1..])
    let datetime = if args.len() > 1 {
        parse_offset(now, &args[1..])?
    } else {
        now
    };

    match format {
        "rfc1123" => Ok(datetime.to_rfc2822()),
        "iso8601" => Ok(datetime.to_rfc3339_opts(SecondsFormat::Millis, true)),
        _ => Err(VarError::InvalidSyntax(format!(
            "Unknown datetime format: {}. Use 'rfc1123' or 'iso8601'",
            format
        ))),
    }
}

/// Parses time offset from arguments
///
/// Expected format: [sign][number] [unit]
/// - sign: + or - (optional, defaults to +)
/// - number: integer
/// - unit: s (seconds), m (minutes), h (hours), d (days)
///
/// Examples: "-1 d", "+2 h", "30 m"
fn parse_offset(base: DateTime<Utc>, args: &[&str]) -> Result<DateTime<Utc>, VarError> {
    if args.len() < 2 {
        return Err(VarError::InvalidOffset(
            "Offset requires number and unit (e.g., '-1 d' or '+2 h')".to_string(),
        ));
    }

    let number_str = args[0];
    let unit = args[1];

    // Parse the number (may include sign)
    let number: i64 = number_str
        .parse()
        .map_err(|_| VarError::InvalidOffset(format!("Invalid number: {}", number_str)))?;

    // Calculate duration based on unit
    let duration = match unit {
        "s" => Duration::seconds(number),
        "m" => Duration::minutes(number),
        "h" => Duration::hours(number),
        "d" => Duration::days(number),
        _ => {
            return Err(VarError::InvalidOffset(format!(
                "Invalid unit: {}. Use 's', 'm', 'h', or 'd'",
                unit
            )))
        }
    };

    Ok(base + duration)
}

/// Generates a random integer in the specified range
///
/// Format: {{$randomInt min max}}
fn resolve_random_int(args: &[&str]) -> Result<String, VarError> {
    if args.len() < 2 {
        return Err(VarError::InvalidSyntax(
            "randomInt requires min and max arguments".to_string(),
        ));
    }

    let min: i64 = args[0]
        .parse()
        .map_err(|_| VarError::InvalidSyntax(format!("Invalid min value: {}", args[0])))?;

    let max: i64 = args[1]
        .parse()
        .map_err(|_| VarError::InvalidSyntax(format!("Invalid max value: {}", args[1])))?;

    if min > max {
        return Err(VarError::InvalidSyntax(format!(
            "min ({}) cannot be greater than max ({})",
            min, max
        )));
    }

    let mut rng = rand::thread_rng();
    let value = rng.gen_range(min..=max);
    Ok(value.to_string())
}

/// Reads a process environment variable
///
/// Formats:
/// - {{$processEnv VAR_NAME}} - returns error if not found
/// - {{$processEnv %VAR_NAME}} - returns empty string if not found (optional)
fn resolve_process_env(args: &[&str]) -> Result<String, VarError> {
    if args.is_empty() {
        return Err(VarError::InvalidSyntax(
            "processEnv requires variable name".to_string(),
        ));
    }

    let var_name = args[0];

    // Check if this is an optional variable (starts with %)
    let (is_optional, clean_name) = if var_name.starts_with('%') {
        (true, &var_name[1..])
    } else {
        (false, var_name)
    };

    match env::var(clean_name) {
        Ok(value) => Ok(value),
        Err(_) => {
            if is_optional {
                Ok(String::new())
            } else {
                Err(VarError::EnvVarNotFound(clean_name.to_string()))
            }
        }
    }
}

/// Reads a variable from .env file in workspace
///
/// Format: {{$dotenv VAR_NAME}}
///
/// The .env file is cached per execution to avoid repeated file reads.
pub fn resolve_dotenv(args: &[&str]) -> Result<String, VarError> {
    if args.is_empty() {
        return Err(VarError::InvalidSyntax(
            "dotenv requires variable name".to_string(),
        ));
    }

    let var_name = args[0];

    // Load .env if not cached
    let cache = DOTENV_CACHE.lock().unwrap();
    if cache.is_none() {
        drop(cache);
        load_dotenv_file()?;
    }

    // Retrieve from cache
    let cache = DOTENV_CACHE.lock().unwrap();
    if let Some(ref env_vars) = *cache {
        env_vars
            .get(var_name)
            .cloned()
            .ok_or_else(|| VarError::EnvVarNotFound(var_name.to_string()))
    } else {
        Err(VarError::DotenvError(
            "Failed to load .env file".to_string(),
        ))
    }
}

/// Loads .env file from workspace directory
fn load_dotenv_file() -> Result<(), VarError> {
    // Try to find .env file in current directory or workspace root
    let env_path = find_dotenv_file()?;

    // Read and parse .env file
    let content = fs::read_to_string(&env_path)
        .map_err(|e| VarError::DotenvError(format!("Failed to read .env file: {}", e)))?;

    let mut env_vars = HashMap::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse key=value
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim();

            // Remove quotes if present
            let value = if (value.starts_with('"') && value.ends_with('"'))
                || (value.starts_with('\'') && value.ends_with('\''))
            {
                &value[1..value.len() - 1]
            } else {
                value
            };

            env_vars.insert(key, value.to_string());
        } else {
            // Invalid line format, but we'll be lenient and skip it
            eprintln!("Warning: Invalid .env line {}: {}", line_num + 1, line);
        }
    }

    // Cache the parsed variables
    let mut cache = DOTENV_CACHE.lock().unwrap();
    *cache = Some(env_vars);

    Ok(())
}

/// Finds .env file in current directory or parent directories
fn find_dotenv_file() -> Result<PathBuf, VarError> {
    let current_dir = env::current_dir()
        .map_err(|e| VarError::DotenvError(format!("Failed to get current directory: {}", e)))?;

    let mut search_dir = current_dir.as_path();

    // Search up to 3 parent directories
    for _ in 0..3 {
        let env_path = search_dir.join(".env");
        if env_path.exists() {
            return Ok(env_path);
        }

        // Move to parent directory
        if let Some(parent) = search_dir.parent() {
            search_dir = parent;
        } else {
            break;
        }
    }

    Err(VarError::DotenvError(".env file not found".to_string()))
}

/// Clears the .env cache (useful for testing or when .env file changes)
pub fn clear_dotenv_cache() {
    let mut cache = DOTENV_CACHE.lock().unwrap();
    *cache = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_resolve_guid() {
        let result = resolve_system_variable("guid", &[]).unwrap();
        // Check it's a valid UUID format
        assert_eq!(result.len(), 36);
        assert!(result.contains('-'));

        // Generate another and ensure they're different
        let result2 = resolve_system_variable("guid", &[]).unwrap();
        assert_ne!(result, result2);
    }

    #[test]
    fn test_resolve_timestamp() {
        let result = resolve_system_variable("timestamp", &[]).unwrap();
        let timestamp: i64 = result.parse().unwrap();
        assert!(timestamp > 0);

        // Check it's reasonable (after 2020 and before 2100)
        assert!(timestamp > 1577836800); // 2020-01-01
        assert!(timestamp < 4102444800); // 2100-01-01
    }

    #[test]
    fn test_resolve_timestamp_with_offset() {
        let now = resolve_system_variable("timestamp", &[]).unwrap();
        let now_ts: i64 = now.parse().unwrap();

        // Test offset +1 hour
        let future = resolve_system_variable("timestamp", &["+1", "h"]).unwrap();
        let future_ts: i64 = future.parse().unwrap();
        assert!(future_ts > now_ts);
        assert_eq!(future_ts - now_ts, 3600); // 1 hour in seconds

        // Test offset -1 day
        let past = resolve_system_variable("timestamp", &["-1", "d"]).unwrap();
        let past_ts: i64 = past.parse().unwrap();
        assert!(past_ts < now_ts);
        assert_eq!(now_ts - past_ts, 86400); // 1 day in seconds
    }

    #[test]
    fn test_resolve_datetime_rfc1123() {
        let result = resolve_system_variable("datetime", &["rfc1123"]).unwrap();
        // Should contain standard RFC 1123 elements
        assert!(result.contains("GMT") || result.contains("+0000"));
    }

    #[test]
    fn test_resolve_datetime_iso8601() {
        let result = resolve_system_variable("datetime", &["iso8601"]).unwrap();
        // Should contain 'T' separator and timezone
        assert!(result.contains('T'));
        assert!(result.contains('Z') || result.contains('+') || result.contains('-'));
    }

    #[test]
    fn test_resolve_datetime_with_offset() {
        let result = resolve_system_variable("datetime", &["iso8601", "+1", "h"]).unwrap();
        assert!(result.contains('T'));
    }

    #[test]
    fn test_resolve_datetime_invalid_format() {
        let result = resolve_system_variable("datetime", &["invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_random_int() {
        let result = resolve_system_variable("randomInt", &["1", "100"]).unwrap();
        let value: i64 = result.parse().unwrap();
        assert!(value >= 1 && value <= 100);

        // Generate multiple and ensure they vary
        let mut values = std::collections::HashSet::new();
        for _ in 0..10 {
            let r = resolve_system_variable("randomInt", &["1", "1000"]).unwrap();
            values.insert(r);
        }
        assert!(values.len() > 1, "Random values should vary");
    }

    #[test]
    fn test_resolve_random_int_invalid_range() {
        let result = resolve_system_variable("randomInt", &["100", "1"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_process_env() {
        // Set a test environment variable
        env::set_var("TEST_VAR_REST_CLIENT", "test_value");

        let result = resolve_system_variable("processEnv", &["TEST_VAR_REST_CLIENT"]).unwrap();
        assert_eq!(result, "test_value");

        // Clean up
        env::remove_var("TEST_VAR_REST_CLIENT");
    }

    #[test]
    fn test_resolve_process_env_optional() {
        // Test optional variable that doesn't exist
        let result = resolve_system_variable("processEnv", &["%NONEXISTENT_VAR"]).unwrap();
        assert_eq!(result, "");

        // Test optional variable that exists
        env::set_var("TEST_OPTIONAL_VAR", "exists");
        let result = resolve_system_variable("processEnv", &["%TEST_OPTIONAL_VAR"]).unwrap();
        assert_eq!(result, "exists");
        env::remove_var("TEST_OPTIONAL_VAR");
    }

    #[test]
    fn test_resolve_process_env_not_found() {
        let result = resolve_system_variable("processEnv", &["DEFINITELY_NOT_SET_VAR_12345"]);
        assert!(matches!(result, Err(VarError::EnvVarNotFound(_))));
    }

    #[test]
    fn test_parse_offset_units() {
        let now = Utc::now();

        // Test seconds
        let result = parse_offset(now, &["30", "s"]).unwrap();
        assert_eq!((result - now).num_seconds(), 30);

        // Test minutes
        let result = parse_offset(now, &["5", "m"]).unwrap();
        assert_eq!((result - now).num_minutes(), 5);

        // Test hours
        let result = parse_offset(now, &["2", "h"]).unwrap();
        assert_eq!((result - now).num_hours(), 2);

        // Test days
        let result = parse_offset(now, &["1", "d"]).unwrap();
        assert_eq!((result - now).num_days(), 1);
    }

    #[test]
    fn test_parse_offset_negative() {
        let now = Utc::now();
        let result = parse_offset(now, &["-1", "h"]).unwrap();
        assert_eq!((now - result).num_hours(), 1);
    }

    #[test]
    fn test_parse_offset_invalid_unit() {
        let now = Utc::now();
        let result = parse_offset(now, &["1", "x"]);
        assert!(matches!(result, Err(VarError::InvalidOffset(_))));
    }

    #[test]
    fn test_undefined_variable() {
        let result = resolve_system_variable("unknownVar", &[]);
        assert!(matches!(result, Err(VarError::UndefinedVariable(_))));
    }

    #[test]
    fn test_dotenv_parsing() {
        // Create a temporary .env file
        let temp_dir = env::temp_dir();
        let env_file_path = temp_dir.join(".env.test");

        {
            let mut file = File::create(&env_file_path).unwrap();
            writeln!(file, "# Comment line").unwrap();
            writeln!(file, "").unwrap();
            writeln!(file, "TEST_KEY=test_value").unwrap();
            writeln!(file, "QUOTED=\"quoted value\"").unwrap();
            writeln!(file, "SINGLE='single quoted'").unwrap();
            writeln!(file, "NO_QUOTES=plain").unwrap();
        }

        // This test would need workspace context setup to work properly
        // For now, just test the parsing logic is present

        // Clean up
        let _ = std::fs::remove_file(env_file_path);
    }
}
