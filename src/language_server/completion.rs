//! Variable completion provider for REST Client
//!
//! This module provides autocompletion functionality for variables in .http files.
//! Completions are triggered when the user types `{{` and include:
//! - System variables ($guid, $timestamp, etc.)
//! - Environment variables from the active environment
//! - Shared variables
//! - File-level variables

use crate::environment::Environments;
use std::collections::HashMap;

/// Represents a completion item to be shown to the user
#[derive(Debug, Clone, PartialEq)]
pub struct CompletionItem {
    /// The text to insert when this completion is selected
    pub label: String,

    /// The kind of completion (system variable, environment variable, etc.)
    pub kind: CompletionKind,

    /// Description or documentation for this completion
    pub detail: Option<String>,

    /// The text to insert (may differ from label if we need to insert more than displayed)
    pub insert_text: String,
}

/// The kind of completion item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// System variable (e.g., $guid, $timestamp)
    SystemVariable,
    /// Environment variable from active environment
    EnvironmentVariable,
    /// Shared variable available in all environments
    SharedVariable,
    /// File-level custom variable
    FileVariable,
}

impl CompletionItem {
    /// Creates a new completion item
    pub fn new(
        label: impl Into<String>,
        kind: CompletionKind,
        detail: Option<String>,
        insert_text: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            kind,
            detail,
            insert_text: insert_text.into(),
        }
    }

    /// Creates a system variable completion
    pub fn system_variable(name: &str, description: &str) -> Self {
        Self {
            label: format!("${}", name),
            kind: CompletionKind::SystemVariable,
            detail: Some(description.to_string()),
            insert_text: format!("${}}}}}", name),
        }
    }

    /// Creates an environment variable completion
    pub fn environment_variable(name: &str, value: &str) -> Self {
        Self {
            label: name.to_string(),
            kind: CompletionKind::EnvironmentVariable,
            detail: Some(format!("= {}", value)),
            insert_text: format!("{}}}}}", name),
        }
    }

    /// Creates a shared variable completion
    pub fn shared_variable(name: &str, value: &str) -> Self {
        Self {
            label: name.to_string(),
            kind: CompletionKind::SharedVariable,
            detail: Some(format!("(shared) = {}", value)),
            insert_text: format!("{}}}}}", name),
        }
    }

    /// Creates a file variable completion
    pub fn file_variable(name: &str, value: &str) -> Self {
        Self {
            label: name.to_string(),
            kind: CompletionKind::FileVariable,
            detail: Some(format!("(file) = {}", value)),
            insert_text: format!("{}}}}}", name),
        }
    }
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

/// Provides completion suggestions for variables at the given position
///
/// # Arguments
/// * `position` - The cursor position in the document
/// * `document` - The full text of the document
/// * `environments` - Available environments and variables
/// * `file_variables` - File-level custom variables (optional)
///
/// # Returns
/// A list of completion items if completions should be shown, or empty if not
///
/// # Examples
/// ```ignore
/// use rest_client::language_server::{provide_completions, Position};
/// use rest_client::environment::Environments;
/// use std::collections::HashMap;
///
/// let doc = "GET https://api.example.com/{{";
/// let pos = Position::new(0, 31); // After {{
/// let envs = Environments::new();
/// let completions = provide_completions(pos, doc, &envs, &HashMap::new());
/// // Returns system variables and any available environment variables
/// ```
pub fn provide_completions(
    position: Position,
    document: &str,
    environments: &Environments,
    file_variables: &HashMap<String, String>,
) -> Vec<CompletionItem> {
    // Check if we should trigger completions (user just typed {{)
    if !should_trigger_completion(position, document) {
        return Vec::new();
    }

    let mut completions = Vec::new();

    // Add environment variables first (highest priority)
    if let Some(active_env) = environments.get_active() {
        for (name, value) in &active_env.variables {
            completions.push(CompletionItem::environment_variable(name, value));
        }
    }

    // Add shared variables
    for (name, value) in &environments.shared {
        // Only add if not already present from environment
        if !completions.iter().any(|c| c.label == *name) {
            completions.push(CompletionItem::shared_variable(name, value));
        }
    }

    // Add file-level variables
    for (name, value) in file_variables {
        if !completions.iter().any(|c| c.label == *name) {
            completions.push(CompletionItem::file_variable(name, value));
        }
    }

    // Add system variables
    completions.extend(get_system_variable_completions());

    completions
}

/// Checks if completion should be triggered at the given position
///
/// Completions are triggered when the user has just typed `{{`
fn should_trigger_completion(position: Position, document: &str) -> bool {
    let lines: Vec<&str> = document.lines().collect();

    // Check if position is valid
    if position.line >= lines.len() {
        return false;
    }

    let line = lines[position.line];

    // Check if we have at least 2 characters before cursor
    if position.character < 2 {
        return false;
    }

    // Check if the two characters before cursor are {{
    if position.character > line.len() {
        return false;
    }

    let text_before = &line[..position.character];
    text_before.ends_with("{{")
}

/// Returns all available system variable completions
fn get_system_variable_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem::system_variable("guid", "Generates a new UUID v4"),
        CompletionItem::system_variable(
            "timestamp",
            "Current Unix timestamp in seconds (use with offset: {{$timestamp -1 d}})",
        ),
        CompletionItem::system_variable(
            "datetime",
            "Formatted datetime (requires format: {{$datetime iso8601}} or {{$datetime rfc1123}})",
        ),
        CompletionItem::system_variable(
            "randomInt",
            "Random integer in range (requires min max: {{$randomInt 1 100}})",
        ),
        CompletionItem::system_variable(
            "processEnv",
            "Process environment variable (requires name: {{$processEnv PATH}})",
        ),
        CompletionItem::system_variable(
            "dotenv",
            "Variable from .env file (requires name: {{$dotenv API_KEY}})",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;

    #[test]
    fn test_should_trigger_completion_after_double_brace() {
        let doc = "GET https://api.example.com/{{";
        let pos = Position::new(0, 30); // Position after {{
        assert!(should_trigger_completion(pos, doc));
    }

    #[test]
    fn test_should_not_trigger_completion_without_double_brace() {
        let doc = "GET https://api.example.com/{";
        let pos = Position::new(0, 30);
        assert!(!should_trigger_completion(pos, doc));
    }

    #[test]
    fn test_should_not_trigger_at_start_of_line() {
        let doc = "{{";
        let pos = Position::new(0, 1);
        assert!(!should_trigger_completion(pos, doc));
    }

    #[test]
    fn test_should_trigger_on_multiline() {
        let doc = "GET https://api.example.com\nAuthorization: Bearer {{";
        let pos = Position::new(1, 24); // Position after {{
        assert!(should_trigger_completion(pos, doc));
    }

    #[test]
    fn test_system_variable_completions() {
        let completions = get_system_variable_completions();
        assert_eq!(completions.len(), 6);

        let guid = completions.iter().find(|c| c.label == "$guid").unwrap();
        assert_eq!(guid.kind, CompletionKind::SystemVariable);
        assert!(guid.detail.is_some());
        assert_eq!(guid.insert_text, "$guid}}");

        let timestamp = completions
            .iter()
            .find(|c| c.label == "$timestamp")
            .unwrap();
        assert_eq!(timestamp.kind, CompletionKind::SystemVariable);
        assert_eq!(timestamp.insert_text, "$timestamp}}");
    }

    #[test]
    fn test_provide_completions_with_environment() {
        let mut envs = Environments::new();
        let mut dev = Environment::new("dev");
        dev.set("baseUrl", "http://localhost:3000");
        dev.set("apiKey", "dev-key-123");
        envs.add_environment(dev);
        envs.set_active("dev");

        let doc = "GET {{baseUrl}}/users\nAuthorization: {{";
        let pos = Position::new(1, 17); // Position after {{
        let file_vars = HashMap::new();

        let completions = provide_completions(pos, doc, &envs, &file_vars);

        // Should have environment variables + system variables
        assert!(completions.len() >= 8); // 2 env + 6 system

        // Check environment variables are present
        let base_url = completions.iter().find(|c| c.label == "baseUrl").unwrap();
        assert_eq!(base_url.kind, CompletionKind::EnvironmentVariable);
        assert!(base_url
            .detail
            .as_ref()
            .unwrap()
            .contains("http://localhost:3000"));

        let api_key = completions.iter().find(|c| c.label == "apiKey").unwrap();
        assert_eq!(api_key.kind, CompletionKind::EnvironmentVariable);
    }

    #[test]
    fn test_provide_completions_with_shared_variables() {
        let mut envs = Environments::new();
        envs.set_shared("apiVersion", "v1");
        envs.set_shared("timeout", "30");

        let doc = "GET https://api.example.com/{{";
        let pos = Position::new(0, 30); // Position after {{
        let file_vars = HashMap::new();

        let completions = provide_completions(pos, doc, &envs, &file_vars);

        let api_version = completions
            .iter()
            .find(|c| c.label == "apiVersion")
            .unwrap();
        assert_eq!(api_version.kind, CompletionKind::SharedVariable);
        assert!(api_version.detail.as_ref().unwrap().contains("(shared)"));
    }

    #[test]
    fn test_provide_completions_with_file_variables() {
        let envs = Environments::new();
        let mut file_vars = HashMap::new();
        file_vars.insert("userId".to_string(), "12345".to_string());
        file_vars.insert("token".to_string(), "abc-xyz".to_string());

        let doc = "GET https://api.example.com/users/{{";
        let pos = Position::new(0, 36); // Position after {{

        let completions = provide_completions(pos, doc, &envs, &file_vars);

        let user_id = completions.iter().find(|c| c.label == "userId").unwrap();
        assert_eq!(user_id.kind, CompletionKind::FileVariable);
        assert!(user_id.detail.as_ref().unwrap().contains("(file)"));
        assert_eq!(user_id.insert_text, "userId}}");
    }

    #[test]
    fn test_environment_variables_override_shared() {
        let mut envs = Environments::new();
        envs.set_shared("baseUrl", "http://shared");

        let mut dev = Environment::new("dev");
        dev.set("baseUrl", "http://dev");
        envs.add_environment(dev);
        envs.set_active("dev");

        let doc = "GET {{";
        let pos = Position::new(0, 6);
        let file_vars = HashMap::new();

        let completions = provide_completions(pos, doc, &envs, &file_vars);

        // Should only have one baseUrl entry (from environment, not shared)
        let base_url_completions: Vec<_> = completions
            .iter()
            .filter(|c| c.label == "baseUrl")
            .collect();

        assert_eq!(base_url_completions.len(), 1);
        assert_eq!(
            base_url_completions[0].kind,
            CompletionKind::EnvironmentVariable
        );
    }

    #[test]
    fn test_no_completions_without_trigger() {
        let envs = Environments::new();
        let file_vars = HashMap::new();

        let doc = "GET https://api.example.com/users";
        let pos = Position::new(0, 20);

        let completions = provide_completions(pos, doc, &envs, &file_vars);
        assert!(completions.is_empty());
    }

    #[test]
    fn test_completion_item_insert_text() {
        let item = CompletionItem::system_variable("guid", "test");
        assert_eq!(item.insert_text, "$guid}}");

        let item = CompletionItem::environment_variable("baseUrl", "http://localhost");
        assert_eq!(item.insert_text, "baseUrl}}");
    }

    #[test]
    fn test_position_creation() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.character, 10);
    }
}
