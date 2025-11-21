//! Variable hover provider for REST Client
//!
//! This module provides hover tooltips that show variable values when the cursor
//! is positioned over a variable reference in .http files.

use crate::environment::Environments;
use crate::variables::{resolve_system_variable, VarError};
use std::collections::HashMap;

/// Represents hover information to display to the user
#[derive(Debug, Clone, PartialEq)]
pub struct Hover {
    /// The main content to display in the hover tooltip
    pub contents: String,

    /// Optional range in the document that this hover applies to
    pub range: Option<Range>,
}

/// A range in a text document
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    /// Start position of the range
    pub start: Position,
    /// End position of the range
    pub end: Position,
}

/// Position in a text document (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Zero-based line number
    pub line: usize,
    /// Zero-based character offset in the line
    pub character: usize,
}

impl Position {
    /// Creates a new position
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

impl Range {
    /// Creates a new range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

impl Hover {
    /// Creates a new hover with content
    pub fn new(contents: impl Into<String>) -> Self {
        Self {
            contents: contents.into(),
            range: None,
        }
    }

    /// Creates a new hover with content and range
    pub fn with_range(contents: impl Into<String>, range: Range) -> Self {
        Self {
            contents: contents.into(),
            range: Some(range),
        }
    }
}

/// Context for variable resolution containing all available variable sources
#[derive(Debug, Clone)]
pub struct VariableContext {
    /// Available environments and active environment
    pub environments: Environments,

    /// File-level custom variables defined in the .http file
    pub file_variables: HashMap<String, String>,

    /// Request-level variables captured from previous request responses
    pub request_variables: HashMap<String, String>,
}

impl VariableContext {
    /// Creates a new VariableContext
    pub fn new(environments: Environments) -> Self {
        Self {
            environments,
            file_variables: HashMap::new(),
            request_variables: HashMap::new(),
        }
    }

    /// Creates a new VariableContext with all variable sources
    pub fn with_variables(
        environments: Environments,
        file_variables: HashMap<String, String>,
        request_variables: HashMap<String, String>,
    ) -> Self {
        Self {
            environments,
            file_variables,
            request_variables,
        }
    }
}

/// Provides hover information for variables at the given position
///
/// # Arguments
/// * `position` - The cursor position in the document
/// * `document` - The full text of the document
/// * `context` - Variable context with all available variables
///
/// # Returns
/// Hover information if the cursor is over a variable, None otherwise
///
/// # Examples
/// ```ignore
/// use rest_client::language_server::{provide_hover, Position};
/// use rest_client::language_server::hover::VariableContext;
/// use rest_client::environment::Environments;
///
/// let doc = "GET https://api.example.com/{{baseUrl}}/users";
/// let pos = Position::new(0, 32); // Inside {{baseUrl}}
/// let context = VariableContext::new(Environments::new());
/// let hover = provide_hover(pos, doc, &context);
/// // Returns hover with baseUrl's value or "will be resolved at runtime"
/// ```
pub fn provide_hover(
    position: Position,
    document: &str,
    context: &VariableContext,
) -> Option<Hover> {
    // Find the variable at the current position
    let (variable_name, range) = find_variable_at_position(position, document)?;

    // Resolve the variable value
    let value = resolve_variable_value(&variable_name, context);

    // Create hover content
    let contents = format_hover_contents(&variable_name, &value);

    Some(Hover::with_range(contents, range))
}

/// Finds a variable reference at the given position
///
/// Returns the variable name and its range in the document
fn find_variable_at_position(position: Position, document: &str) -> Option<(String, Range)> {
    let lines: Vec<&str> = document.lines().collect();

    if position.line >= lines.len() {
        return None;
    }

    let line = lines[position.line];

    // Find all {{...}} patterns in the line
    let mut start_idx = 0;
    while let Some(open_pos) = line[start_idx..].find("{{") {
        let open_pos = start_idx + open_pos;
        let search_start = open_pos + 2;

        if let Some(close_offset) = line[search_start..].find("}}") {
            let close_pos = search_start + close_offset;

            // Check if cursor is within this variable reference
            if position.character >= open_pos && position.character <= close_pos + 2 {
                let var_name = line[search_start..close_pos].trim().to_string();
                let range = Range::new(
                    Position::new(position.line, open_pos),
                    Position::new(position.line, close_pos + 2),
                );
                return Some((var_name, range));
            }

            start_idx = close_pos + 2;
        } else {
            break;
        }
    }

    None
}

/// Resolves a variable value from the context
fn resolve_variable_value(name: &str, context: &VariableContext) -> VariableValue {
    // System variables (e.g., {{$guid}}, {{$timestamp}})
    if name.starts_with('$') {
        return resolve_system_variable_value(name);
    }

    // Request variables (highest priority for non-system variables)
    if let Some(value) = context.request_variables.get(name) {
        return VariableValue::Resolved(value.clone(), "request variable".to_string());
    }

    // File-level variables
    if let Some(value) = context.file_variables.get(name) {
        return VariableValue::Resolved(value.clone(), "file variable".to_string());
    }

    // Environment variables (active environment takes precedence)
    if let Some(env) = context.environments.get_active() {
        if let Some(value) = env.get(name) {
            return VariableValue::Resolved(
                value.clone(),
                format!("environment variable ({})", env.name),
            );
        }
    }

    // Shared variables (fallback when not in active environment)
    if let Some(value) = context.environments.shared.get(name) {
        return VariableValue::Resolved(value.clone(), "shared variable".to_string());
    }

    // Variable not found in any source
    VariableValue::Undefined
}

/// Resolves a system variable value for hover display
fn resolve_system_variable_value(name: &str) -> VariableValue {
    // Parse variable name and args
    let parts: Vec<&str> = name.trim_start_matches('$').split_whitespace().collect();
    if parts.is_empty() {
        return VariableValue::Undefined;
    }

    let var_name = parts[0];
    let args: Vec<&str> = parts[1..].to_vec();

    match resolve_system_variable(var_name, &args) {
        Ok(value) => {
            let description = get_system_variable_description(var_name);
            VariableValue::RuntimeResolved(value, description)
        }
        Err(VarError::UndefinedVariable(_)) => VariableValue::Undefined,
        Err(err) => VariableValue::Error(err.to_string()),
    }
}

/// Gets a description for a system variable
fn get_system_variable_description(name: &str) -> String {
    match name {
        "guid" => "generates a new UUID v4".to_string(),
        "timestamp" => "current Unix timestamp (can use offset like -1 d)".to_string(),
        "datetime" => "formatted datetime (requires format: iso8601 or rfc1123)".to_string(),
        "randomInt" => "random integer (requires min and max)".to_string(),
        "processEnv" => "process environment variable".to_string(),
        "dotenv" => "variable from .env file".to_string(),
        _ => "system variable".to_string(),
    }
}

/// Represents the resolved value of a variable
#[derive(Debug, Clone, PartialEq)]
enum VariableValue {
    /// Variable is resolved to a static value
    Resolved(String, String), // (value, source)
    /// Variable will be resolved at runtime (e.g., system variables)
    RuntimeResolved(String, String), // (example_value, description)
    /// Variable is undefined
    Undefined,
    /// Error resolving variable
    Error(String),
}

/// Formats hover contents based on the variable value
fn format_hover_contents(name: &str, value: &VariableValue) -> String {
    match value {
        VariableValue::Resolved(val, source) => {
            format!(
                "**Variable:** `{}`\n\n**Value:** `{}`\n\n**Source:** {}",
                name, val, source
            )
        }
        VariableValue::RuntimeResolved(example, desc) => {
            format!(
                "**System Variable:** `{}`\n\n**Description:** {}\n\n**Example value:** `{}`\n\n*Will be resolved at runtime*",
                name, desc, example
            )
        }
        VariableValue::Undefined => {
            format!(
                "**Variable:** `{}`\n\n⚠️ **Undefined variable**\n\nThis variable is not defined in:\n- Request variables\n- File variables\n- Environment variables\n- Shared variables",
                name
            )
        }
        VariableValue::Error(err) => {
            format!("**Variable:** `{}`\n\n❌ **Error:** {}", name, err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;

    #[test]
    fn test_find_variable_at_position_simple() {
        let doc = "GET https://api.example.com/{{baseUrl}}/users";
        let pos = Position::new(0, 32); // Inside {{baseUrl}}

        let result = find_variable_at_position(pos, doc);
        assert!(result.is_some());

        let (var_name, range) = result.unwrap();
        assert_eq!(var_name, "baseUrl");
        assert_eq!(range.start.character, 28);
        assert_eq!(range.end.character, 39);
    }

    #[test]
    fn test_find_variable_at_position_on_braces() {
        let doc = "Authorization: Bearer {{token}}";
        let pos = Position::new(0, 22); // On opening {{

        let result = find_variable_at_position(pos, doc);
        assert!(result.is_some());

        let (var_name, _) = result.unwrap();
        assert_eq!(var_name, "token");
    }

    #[test]
    fn test_find_variable_not_in_variable() {
        let doc = "GET https://api.example.com/users";
        let pos = Position::new(0, 15);

        let result = find_variable_at_position(pos, doc);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_variable_multiple_on_line() {
        let doc = "GET {{baseUrl}}/api/{{version}}/users";
        let pos = Position::new(0, 25); // Inside {{version}}

        let result = find_variable_at_position(pos, doc);
        assert!(result.is_some());

        let (var_name, _) = result.unwrap();
        assert_eq!(var_name, "version");
    }

    #[test]
    fn test_resolve_environment_variable() {
        let mut envs = Environments::new();
        let mut dev = Environment::new("dev");
        dev.set("baseUrl", "http://localhost:3000");
        envs.add_environment(dev);
        envs.set_active("dev");

        let context = VariableContext::new(envs);

        let value = resolve_variable_value("baseUrl", &context);
        match value {
            VariableValue::Resolved(val, source) => {
                assert_eq!(val, "http://localhost:3000");
                assert!(source.contains("dev"));
            }
            _ => panic!("Expected Resolved variant"),
        }
    }

    #[test]
    fn test_resolve_shared_variable() {
        let mut envs = Environments::new();
        envs.set_shared("apiVersion", "v1");

        let context = VariableContext::new(envs);

        let value = resolve_variable_value("apiVersion", &context);
        match value {
            VariableValue::Resolved(val, source) => {
                assert_eq!(val, "v1");
                assert_eq!(source, "shared variable");
            }
            _ => panic!("Expected Resolved variant"),
        }
    }

    #[test]
    fn test_resolve_file_variable() {
        let envs = Environments::new();
        let mut file_vars = HashMap::new();
        file_vars.insert("userId".to_string(), "12345".to_string());

        let context = VariableContext::with_variables(envs, file_vars, HashMap::new());

        let value = resolve_variable_value("userId", &context);
        match value {
            VariableValue::Resolved(val, source) => {
                assert_eq!(val, "12345");
                assert_eq!(source, "file variable");
            }
            _ => panic!("Expected Resolved variant"),
        }
    }

    #[test]
    fn test_resolve_request_variable() {
        let envs = Environments::new();
        let mut request_vars = HashMap::new();
        request_vars.insert("authToken".to_string(), "abc-xyz-123".to_string());

        let context = VariableContext::with_variables(envs, HashMap::new(), request_vars);

        let value = resolve_variable_value("authToken", &context);
        match value {
            VariableValue::Resolved(val, source) => {
                assert_eq!(val, "abc-xyz-123");
                assert_eq!(source, "request variable");
            }
            _ => panic!("Expected Resolved variant"),
        }
    }

    #[test]
    fn test_resolve_system_variable() {
        let value = resolve_system_variable_value("$guid");
        match value {
            VariableValue::RuntimeResolved(val, desc) => {
                assert!(val.len() > 0); // UUID should be generated
                assert!(desc.contains("UUID"));
            }
            _ => panic!("Expected RuntimeResolved variant"),
        }
    }

    #[test]
    fn test_resolve_undefined_variable() {
        let envs = Environments::new();
        let context = VariableContext::new(envs);

        let value = resolve_variable_value("nonexistent", &context);
        assert_eq!(value, VariableValue::Undefined);
    }

    #[test]
    fn test_provide_hover_with_variable() {
        let mut envs = Environments::new();
        let mut dev = Environment::new("dev");
        dev.set("baseUrl", "http://localhost:3000");
        envs.add_environment(dev);
        envs.set_active("dev");

        let context = VariableContext::new(envs);
        let doc = "GET {{baseUrl}}/users";
        let pos = Position::new(0, 7); // Inside {{baseUrl}}

        let hover = provide_hover(pos, doc, &context);
        assert!(hover.is_some());

        let hover = hover.unwrap();
        assert!(hover.contents.contains("baseUrl"));
        assert!(hover.contents.contains("http://localhost:3000"));
        assert!(hover.range.is_some());
    }

    #[test]
    fn test_provide_hover_without_variable() {
        let envs = Environments::new();
        let context = VariableContext::new(envs);
        let doc = "GET https://api.example.com/users";
        let pos = Position::new(0, 10);

        let hover = provide_hover(pos, doc, &context);
        assert!(hover.is_none());
    }

    #[test]
    fn test_provide_hover_undefined_variable() {
        let envs = Environments::new();
        let context = VariableContext::new(envs);
        let doc = "GET {{unknownVar}}/users";
        let pos = Position::new(0, 7);

        let hover = provide_hover(pos, doc, &context);
        assert!(hover.is_some());

        let hover = hover.unwrap();
        assert!(hover.contents.contains("Undefined variable"));
    }

    #[test]
    fn test_provide_hover_system_variable() {
        let envs = Environments::new();
        let context = VariableContext::new(envs);
        let doc = "X-Request-ID: {{$guid}}";
        let pos = Position::new(0, 17);

        let hover = provide_hover(pos, doc, &context);
        assert!(hover.is_some());

        let hover = hover.unwrap();
        assert!(hover.contents.contains("$guid"));
        assert!(hover.contents.contains("Will be resolved at runtime"));
    }

    #[test]
    fn test_format_hover_resolved() {
        let value = VariableValue::Resolved(
            "http://localhost".to_string(),
            "environment variable (dev)".to_string(),
        );
        let contents = format_hover_contents("baseUrl", &value);

        assert!(contents.contains("baseUrl"));
        assert!(contents.contains("http://localhost"));
        assert!(contents.contains("environment variable (dev)"));
    }

    #[test]
    fn test_format_hover_runtime_resolved() {
        let value = VariableValue::RuntimeResolved(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "generates a new UUID v4".to_string(),
        );
        let contents = format_hover_contents("$guid", &value);

        assert!(contents.contains("$guid"));
        assert!(contents.contains("UUID"));
        assert!(contents.contains("Will be resolved at runtime"));
    }

    #[test]
    fn test_format_hover_undefined() {
        let value = VariableValue::Undefined;
        let contents = format_hover_contents("missing", &value);

        assert!(contents.contains("missing"));
        assert!(contents.contains("Undefined variable"));
    }

    #[test]
    fn test_variable_precedence() {
        let mut envs = Environments::new();
        envs.set_shared("var", "shared_value");

        let mut dev = Environment::new("dev");
        dev.set("var", "env_value");
        envs.add_environment(dev);
        envs.set_active("dev");

        let mut file_vars = HashMap::new();
        file_vars.insert("var".to_string(), "file_value".to_string());

        let mut request_vars = HashMap::new();
        request_vars.insert("var".to_string(), "request_value".to_string());

        let context = VariableContext::with_variables(envs, file_vars, request_vars);

        // Request variables should have highest priority
        let value = resolve_variable_value("var", &context);
        match value {
            VariableValue::Resolved(val, source) => {
                assert_eq!(val, "request_value");
                assert_eq!(source, "request variable");
            }
            _ => panic!("Expected Resolved variant"),
        }
    }
}
