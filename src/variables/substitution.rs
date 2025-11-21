//! Variable substitution engine for REST Client
//!
//! This module provides the core substitution logic that replaces {{variable}} patterns
//! in HTTP request text with their resolved values. It supports nested variables,
//! circular reference detection, and multiple variable types (system, environment, request, file).

use super::{resolve_system_variable, VarError};
use crate::environment::Environment;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Maximum recursion depth for nested variable substitution
const MAX_RECURSION_DEPTH: usize = 10;

/// Cached regex pattern for matching {{variableName}} with optional whitespace.
/// This is compiled once and reused to avoid repeated regex compilation overhead.
static VARIABLE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{\{([^}]+)\}\}").expect("Failed to compile variable regex"));

/// Context for variable resolution containing all available variable sources
#[derive(Debug, Clone)]
pub struct VariableContext {
    /// Current environment variables (if an environment is active)
    pub environment: Option<Environment>,

    /// Shared variables available across all environments
    pub shared_variables: HashMap<String, String>,

    /// File-level custom variables defined in the .http file
    pub file_variables: HashMap<String, String>,

    /// Request-level variables captured from previous request responses
    pub request_variables: HashMap<String, String>,

    /// Workspace path for resolving relative file paths
    pub workspace_path: PathBuf,
}

impl VariableContext {
    /// Creates a new VariableContext with default values
    pub fn new(workspace_path: PathBuf) -> Self {
        Self {
            environment: None,
            shared_variables: HashMap::new(),
            file_variables: HashMap::new(),
            request_variables: HashMap::new(),
            workspace_path,
        }
    }

    /// Creates a new VariableContext with environment and shared variables
    pub fn with_environment(
        workspace_path: PathBuf,
        environment: Option<Environment>,
        shared_variables: HashMap<String, String>,
    ) -> Self {
        Self {
            environment,
            shared_variables,
            file_variables: HashMap::new(),
            request_variables: HashMap::new(),
            workspace_path,
        }
    }

    /// Resolves a variable by name, checking all available sources in priority order
    ///
    /// Priority order:
    /// 1. System variables ($ prefix)
    /// 2. Request variables (from previous responses)
    /// 3. File variables (defined in .http file)
    /// 4. Environment variables (from active environment)
    /// 5. Shared variables (fallback from all environments)
    fn resolve_variable(&self, name: &str) -> Result<String, VarError> {
        // System variables (e.g., {{$guid}}, {{$timestamp}})
        if name.starts_with('$') {
            return self.resolve_system_variable_with_args(name);
        }

        // Request variables (highest priority for non-system variables)
        if let Some(value) = self.request_variables.get(name) {
            return Ok(value.clone());
        }

        // File-level variables
        if let Some(value) = self.file_variables.get(name) {
            return Ok(value.clone());
        }

        // Environment variables (active environment takes precedence)
        if let Some(env) = &self.environment {
            if let Some(value) = env.get(name) {
                return Ok(value.clone());
            }
        }

        // Shared variables (fallback when not in active environment)
        if let Some(value) = self.shared_variables.get(name) {
            return Ok(value.clone());
        }

        // Variable not found in any source
        Err(VarError::UndefinedVariable(name.to_string()))
    }

    /// Resolves a system variable, parsing arguments if present
    fn resolve_system_variable_with_args(&self, name: &str) -> Result<String, VarError> {
        // Parse system variable name and arguments
        // Format: $variableName arg1 arg2 ...
        let parts: Vec<&str> = name.split_whitespace().collect();
        if parts.is_empty() {
            return Err(VarError::InvalidSyntax(
                "Empty system variable name".to_string(),
            ));
        }

        // Extract variable name and strip $ prefix
        let var_name = parts[0];
        if !var_name.starts_with('$') {
            return Err(VarError::InvalidSyntax(format!(
                "System variable must start with $: {}",
                var_name
            )));
        }

        // Remove $ prefix as resolve_system_variable expects name without it
        let var_name_without_prefix = &var_name[1..];
        let args = &parts[1..];

        resolve_system_variable(var_name_without_prefix, args)
    }
}

/// Substitutes all {{variable}} patterns in the input text with their resolved values
///
/// This function:
/// - Finds all {{variableName}} patterns using regex
/// - Handles escaped braces (\{{ and \}}) as literal text
/// - Resolves nested variables recursively (inner-first)
/// - Detects circular references
/// - Preserves original formatting and whitespace
///
/// # Arguments
///
/// * `text` - The input text containing {{variable}} patterns
/// * `context` - The VariableContext containing all available variables
///
/// # Returns
///
/// Returns the text with all variables substituted, or an error if:
/// - A variable is undefined
/// - A circular reference is detected
/// - Maximum recursion depth is exceeded
///
/// # Examples
///
/// ```
/// use rest_client::variables::substitution::{substitute_variables, VariableContext};
/// use std::path::PathBuf;
///
/// let mut context = VariableContext::new(PathBuf::from("/workspace"));
/// context.file_variables.insert("baseUrl".to_string(), "https://api.example.com".to_string());
///
/// let text = "GET {{baseUrl}}/users";
/// let result = substitute_variables(text, &context).unwrap();
/// assert_eq!(result, "GET https://api.example.com/users");
/// ```
pub fn substitute_variables(text: &str, context: &VariableContext) -> Result<String, VarError> {
    // Fast path: if there are no variable markers at all, return original text
    if !text.contains("{{") {
        return Ok(text.to_string());
    }

    substitute_variables_with_depth(text, context, 0, &mut HashSet::new())
}

/// Internal recursive substitution function with depth tracking and cycle detection
fn substitute_variables_with_depth(
    text: &str,
    context: &VariableContext,
    depth: usize,
    visiting: &mut HashSet<String>,
) -> Result<String, VarError> {
    // Check recursion depth limit
    if depth >= MAX_RECURSION_DEPTH {
        return Err(VarError::CircularReference(
            "Maximum recursion depth exceeded - possible circular reference".to_string(),
        ));
    }

    // Handle escaped braces first - replace \{{ and \}} with placeholders
    let text = text.replace("\\{{", "\u{E000}").replace("\\}}", "\u{E001}");

    // Use cached regex to avoid repeated compilations (performance optimization)
    let re = &*VARIABLE_REGEX;

    // Pre-allocate result string with estimated capacity to reduce reallocations
    let mut result = String::with_capacity(text.len() + (text.len() / 4));
    let mut last_match_end = 0;

    // Process each variable match
    for cap in re.captures_iter(&text) {
        let full_match = cap.get(0).unwrap();
        let var_name = cap.get(1).unwrap().as_str().trim();

        // Add text before this match
        result.push_str(&text[last_match_end..full_match.start()]);

        // Check for circular reference
        if visiting.contains(var_name) {
            return Err(VarError::CircularReference(format!(
                "Circular reference detected for variable '{}'",
                var_name
            )));
        }

        // Mark this variable as being visited
        visiting.insert(var_name.to_string());

        // Resolve the variable
        let resolved_value = context.resolve_variable(var_name)?;

        // Recursively substitute variables in the resolved value
        let substituted_value =
            substitute_variables_with_depth(&resolved_value, context, depth + 1, visiting)?;

        result.push_str(&substituted_value);

        // Unmark this variable after processing
        visiting.remove(var_name);

        last_match_end = full_match.end();
    }

    // Add remaining text after last match
    result.push_str(&text[last_match_end..]);

    // Restore escaped braces to literal {{ and }}
    let result = result.replace("\u{E000}", "{{").replace("\u{E001}", "}}");

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> VariableContext {
        let mut context = VariableContext::new(PathBuf::from("/test/workspace"));

        // Add some test variables
        context
            .file_variables
            .insert("baseUrl".to_string(), "https://api.example.com".to_string());
        context
            .file_variables
            .insert("apiKey".to_string(), "secret-key-123".to_string());
        context
            .file_variables
            .insert("port".to_string(), "8080".to_string());

        context
            .request_variables
            .insert("userId".to_string(), "12345".to_string());
        context
            .request_variables
            .insert("token".to_string(), "bearer-token-xyz".to_string());

        // Add environment
        let mut env_vars = HashMap::new();
        env_vars.insert("host".to_string(), "staging.example.com".to_string());
        env_vars.insert("timeout".to_string(), "30".to_string());

        context.environment = Some(Environment {
            name: "staging".to_string(),
            variables: env_vars,
        });

        context
    }

    #[test]
    fn test_simple_substitution() {
        let context = create_test_context();

        let text = "GET {{baseUrl}}/users";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "GET https://api.example.com/users");
    }

    #[test]
    fn test_multiple_variables() {
        let context = create_test_context();

        let text = "GET {{baseUrl}}:{{port}}/api?key={{apiKey}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(
            result,
            "GET https://api.example.com:8080/api?key=secret-key-123"
        );
    }

    #[test]
    fn test_request_variable_priority() {
        let context = create_test_context();

        // Request variables should have priority over file variables
        let text = "User ID: {{userId}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "User ID: 12345");
    }

    #[test]
    fn test_environment_variable() {
        let context = create_test_context();

        let text = "GET https://{{host}}/api";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "GET https://staging.example.com/api");
    }

    #[test]
    fn test_nested_variables() {
        let mut context = create_test_context();
        context
            .file_variables
            .insert("fullUrl".to_string(), "{{baseUrl}}/users".to_string());

        let text = "GET {{fullUrl}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "GET https://api.example.com/users");
    }

    #[test]
    fn test_deeply_nested_variables() {
        let mut context = create_test_context();
        context
            .file_variables
            .insert("level1".to_string(), "{{level2}}".to_string());
        context
            .file_variables
            .insert("level2".to_string(), "{{level3}}".to_string());
        context
            .file_variables
            .insert("level3".to_string(), "final-value".to_string());

        let text = "Value: {{level1}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "Value: final-value");
    }

    #[test]
    fn test_circular_reference_detection() {
        let mut context = create_test_context();
        context
            .file_variables
            .insert("var1".to_string(), "{{var2}}".to_string());
        context
            .file_variables
            .insert("var2".to_string(), "{{var1}}".to_string());

        let text = "Value: {{var1}}";
        let result = substitute_variables(text, &context);

        assert!(result.is_err());
        match result {
            Err(VarError::CircularReference(_)) => (),
            _ => panic!("Expected CircularReference error"),
        }
    }

    #[test]
    fn test_max_recursion_depth() {
        let mut context = create_test_context();

        // Create a chain of 15 nested variables (exceeds MAX_RECURSION_DEPTH of 10)
        for i in 0..15 {
            let var_name = format!("var{}", i);
            let next_var = format!("var{}", i + 1);
            context
                .file_variables
                .insert(var_name, format!("{{{{{}}}}}", next_var));
        }
        context
            .file_variables
            .insert("var15".to_string(), "end".to_string());

        let text = "{{var0}}";
        let result = substitute_variables(text, &context);

        assert!(result.is_err());
        match result {
            Err(VarError::CircularReference(_)) => (),
            _ => panic!("Expected CircularReference error for max depth"),
        }
    }

    #[test]
    fn test_escaped_braces() {
        let context = create_test_context();

        let text = "This is a literal \\{{variable\\}} and this is real: {{baseUrl}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(
            result,
            "This is a literal {{variable}} and this is real: https://api.example.com"
        );
    }

    #[test]
    fn test_whitespace_preservation() {
        let context = create_test_context();

        let text = "GET {{  baseUrl  }}/users";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "GET https://api.example.com/users");
    }

    #[test]
    fn test_undefined_variable() {
        let context = create_test_context();

        let text = "GET {{undefinedVar}}/users";
        let result = substitute_variables(text, &context);

        assert!(result.is_err());
        match result {
            Err(VarError::UndefinedVariable(var)) => {
                assert_eq!(var, "undefinedVar");
            }
            _ => panic!("Expected UndefinedVariable error"),
        }
    }

    #[test]
    fn test_system_variable_guid() {
        let context = create_test_context();

        let text = "Request-ID: {{$guid}}";
        let result = substitute_variables(text, &context).unwrap();

        assert!(result.starts_with("Request-ID: "));
        // GUID should be 36 characters (including hyphens)
        let guid_part = &result[12..];
        assert_eq!(guid_part.len(), 36);
    }

    #[test]
    fn test_system_variable_timestamp() {
        let context = create_test_context();

        let text = "Timestamp: {{$timestamp}}";
        let result = substitute_variables(text, &context).unwrap();

        assert!(result.starts_with("Timestamp: "));
        // Should be a valid number
        let timestamp_str = &result[11..];
        assert!(timestamp_str.parse::<i64>().is_ok());
    }

    #[test]
    fn test_system_variable_random_int() {
        let context = create_test_context();

        let text = "Random: {{$randomInt 1 100}}";
        let result = substitute_variables(text, &context).unwrap();

        assert!(result.starts_with("Random: "));
        let random_str = &result[8..];
        let random_num: i32 = random_str.parse().unwrap();
        assert!(random_num >= 1 && random_num <= 100);
    }

    #[test]
    fn test_mixed_variable_types() {
        let context = create_test_context();

        let text = "POST {{baseUrl}}/users/{{userId}}?key={{apiKey}}&ts={{$timestamp}}";
        let result = substitute_variables(text, &context).unwrap();

        assert!(
            result.starts_with("POST https://api.example.com/users/12345?key=secret-key-123&ts=")
        );
    }

    #[test]
    fn test_empty_text() {
        let context = create_test_context();

        let result = substitute_variables("", &context).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_no_variables() {
        let context = create_test_context();

        let text = "GET https://example.com/users";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "GET https://example.com/users");
    }

    #[test]
    fn test_variable_in_header() {
        let context = create_test_context();

        let text = "Authorization: Bearer {{token}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "Authorization: Bearer bearer-token-xyz");
    }

    #[test]
    fn test_variable_in_json_body() {
        let context = create_test_context();

        let text = r#"{"userId": "{{userId}}", "apiKey": "{{apiKey}}"}"#;
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, r#"{"userId": "12345", "apiKey": "secret-key-123"}"#);
    }

    #[test]
    fn test_nested_variables_with_system_var() {
        let mut context = create_test_context();
        context
            .file_variables
            .insert("timestamp".to_string(), "{{$timestamp}}".to_string());

        let text = "Time: {{timestamp}}";
        let result = substitute_variables(text, &context).unwrap();

        assert!(result.starts_with("Time: "));
        let timestamp_str = &result[6..];
        assert!(timestamp_str.parse::<i64>().is_ok());
    }

    #[test]
    fn test_multiple_same_variable() {
        let context = create_test_context();

        let text = "{{baseUrl}}/users and {{baseUrl}}/posts";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(
            result,
            "https://api.example.com/users and https://api.example.com/posts"
        );
    }

    #[test]
    fn test_shared_variables_fallback() {
        use crate::environment::Environment;
        use std::path::PathBuf;

        // Create environment with some variables
        let mut env = Environment::new("dev");
        env.set("envVar", "from environment");

        // Create shared variables
        let mut shared = HashMap::new();
        shared.insert("sharedVar".to_string(), "from shared".to_string());
        shared.insert("envVar".to_string(), "shared fallback".to_string()); // Should be overridden

        let mut context =
            VariableContext::with_environment(PathBuf::from("/workspace"), Some(env), shared);

        // Test environment variable takes precedence
        let text = "Env: {{envVar}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "Env: from environment");

        // Test shared variable fallback
        let text = "Shared: {{sharedVar}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "Shared: from shared");

        // Test both together
        let text = "{{envVar}} and {{sharedVar}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "from environment and from shared");
    }

    #[test]
    fn test_shared_variables_no_environment() {
        use std::path::PathBuf;

        // No active environment, only shared variables
        let mut shared = HashMap::new();
        shared.insert("apiVersion".to_string(), "v1".to_string());
        shared.insert("timeout".to_string(), "30".to_string());

        let context = VariableContext::with_environment(PathBuf::from("/workspace"), None, shared);

        let text = "API version {{apiVersion}} with timeout {{timeout}}s";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "API version v1 with timeout 30s");
    }

    #[test]
    fn test_nested_variables_with_shared() {
        use crate::environment::Environment;
        use std::path::PathBuf;

        let mut env = Environment::new("dev");
        env.set("baseUrl", "http://localhost:3000");
        env.set("endpoint", "{{baseUrl}}/api");

        let mut shared = HashMap::new();
        shared.insert("version".to_string(), "v2".to_string());
        shared.insert(
            "fullUrl".to_string(),
            "{{endpoint}}/{{version}}".to_string(),
        );

        let mut context =
            VariableContext::with_environment(PathBuf::from("/workspace"), Some(env), shared);

        // Nested substitution with environment and shared variables
        let text = "URL: {{fullUrl}}";
        let result = substitute_variables(text, &context).unwrap();
        assert_eq!(result, "URL: http://localhost:3000/api/v2");
    }
}
