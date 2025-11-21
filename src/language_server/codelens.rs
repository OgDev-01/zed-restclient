//! CodeLens provider for REST Client
//!
//! This module provides CodeLens functionality for .http files, displaying
//! clickable "Send Request" lenses above each valid HTTP request block.
//! CodeLens appears on the first non-comment line of each request, allowing
//! users to execute requests directly from the editor.

use regex::Regex;

/// Represents a position in a text document
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

/// Represents a range in a text document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Range {
    /// Creates a new range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Creates a range for an entire line
    pub fn line(line: usize) -> Self {
        Self {
            start: Position::new(line, 0),
            end: Position::new(line, usize::MAX),
        }
    }
}

/// Represents a command that can be executed from a CodeLens
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The command identifier
    pub command: String,
    /// Human-readable title shown in the lens
    pub title: String,
    /// Optional arguments to pass to the command
    pub arguments: Option<Vec<String>>,
}

impl Command {
    /// Creates a new command
    pub fn new(command: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            title: title.into(),
            arguments: None,
        }
    }

    /// Adds arguments to the command
    pub fn with_arguments(mut self, args: Vec<String>) -> Self {
        self.arguments = Some(args);
        self
    }
}

/// Represents a CodeLens in the document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeLens {
    /// The range where the CodeLens should appear
    pub range: Range,
    /// The command to execute when clicked
    pub command: Option<Command>,
    /// Optional data for resolving the lens later
    pub data: Option<String>,
}

impl CodeLens {
    /// Creates a new CodeLens
    pub fn new(range: Range) -> Self {
        Self {
            range,
            command: None,
            data: None,
        }
    }

    /// Sets the command for this CodeLens
    pub fn with_command(mut self, command: Command) -> Self {
        self.command = Some(command);
        self
    }

    /// Sets data for this CodeLens
    pub fn with_data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }
}

/// Provides CodeLens for all valid HTTP requests in a document
///
/// Scans the document for request blocks (separated by ###) and creates
/// CodeLens entries for each valid request. The CodeLens appears on the
/// first non-comment line of each request.
///
/// # Arguments
/// * `document` - The full text of the .http file
///
/// # Returns
/// A vector of CodeLens objects, one for each valid request
///
/// # Examples
/// ```
/// use rest_client::language_server::codelens::provide_code_lens;
///
/// let doc = "GET https://api.example.com\n###\nPOST https://api.example.com";
/// let lenses = provide_code_lens(doc);
/// assert_eq!(lenses.len(), 2);
/// ```
pub fn provide_code_lens(document: &str) -> Vec<CodeLens> {
    let mut lenses = Vec::new();
    let lines: Vec<&str> = document.lines().collect();

    // Pattern to match @name comments
    let name_pattern = Regex::new(r"^[#/]+\s*@name\s+(.+)$").unwrap();
    // Pattern to match HTTP methods at the start of a line
    let method_pattern =
        Regex::new(r"^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS|CONNECT|TRACE)\s+").unwrap();

    let mut last_name: Option<String> = None;

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for @name comment
        if let Some(captures) = name_pattern.captures(trimmed) {
            if let Some(captured_name) = captures.get(1) {
                last_name = Some(captured_name.as_str().trim().to_string());
            }
            continue;
        }

        // Reset name if we encounter a delimiter (signals start of new section)
        if trimmed == "###" {
            last_name = None;
            continue;
        }

        // Skip other comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Skip variable assignments
        if trimmed.starts_with('@') {
            continue;
        }

        // Check if this line starts with an HTTP method
        if method_pattern.is_match(trimmed) {
            // Create a CodeLens for this request
            let range = Range::line(line_num);
            let title = if let Some(name) = &last_name {
                format!("▶ Send Request: {}", name)
            } else {
                "▶ Send Request".to_string()
            };

            let send_command = Command::new("rest-client.send", title);
            let lens = CodeLens::new(range).with_command(send_command);
            lenses.push(lens);

            // Reset the name after using it (so it doesn't apply to subsequent requests)
            last_name = None;
        }
    }

    lenses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provide_code_lens_single_request() {
        let doc = "GET https://api.example.com/users";
        let lenses = provide_code_lens(doc);

        assert_eq!(lenses.len(), 1);
        assert_eq!(lenses[0].range.start.line, 0);
        assert!(lenses[0].command.is_some());
        assert_eq!(
            lenses[0].command.as_ref().unwrap().command,
            "rest-client.send"
        );
    }

    #[test]
    fn test_provide_code_lens_multiple_requests() {
        let doc = r#"
GET https://api.example.com/users

###

POST https://api.example.com/users
Content-Type: application/json

{"name": "John"}

###

DELETE https://api.example.com/users/1
"#;
        let lenses = provide_code_lens(doc);

        assert_eq!(lenses.len(), 3);
        // Verify each lens has the correct command
        for lens in &lenses {
            assert!(lens.command.is_some());
            assert_eq!(lens.command.as_ref().unwrap().command, "rest-client.send");
        }
    }

    #[test]
    fn test_provide_code_lens_with_name() {
        let doc = r#"
# @name GetUsers
GET https://api.example.com/users
"#;
        let lenses = provide_code_lens(doc);

        assert_eq!(lenses.len(), 1);
        assert!(lenses[0].command.is_some());
        let command = lenses[0].command.as_ref().unwrap();
        assert!(command.title.contains("GetUsers"));
    }

    #[test]
    fn test_provide_code_lens_with_comments() {
        let doc = r#"
# This is a comment
// Another comment
GET https://api.example.com/users
"#;
        let lenses = provide_code_lens(doc);

        assert_eq!(lenses.len(), 1);
        // CodeLens should be on the line with GET (line 3 in zero-based indexing)
        assert_eq!(lenses[0].range.start.line, 3);
    }

    #[test]
    fn test_provide_code_lens_empty_blocks() {
        let doc = r#"
GET https://api.example.com/users

###

# Just a comment

###

POST https://api.example.com/users
"#;
        let lenses = provide_code_lens(doc);

        // Should only have 2 lenses (the comment-only block is ignored)
        assert_eq!(lenses.len(), 2);
    }

    #[test]
    fn test_provide_code_lens_no_requests() {
        let doc = r#"
# Just comments
// More comments
"#;
        let lenses = provide_code_lens(doc);

        assert_eq!(lenses.len(), 0);
    }

    #[test]
    fn test_code_lens_range() {
        let range = Range::line(5);
        assert_eq!(range.start.line, 5);
        assert_eq!(range.start.character, 0);
        assert_eq!(range.end.line, 5);
    }

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("rest-client.send", "Send Request");
        assert_eq!(cmd.command, "rest-client.send");
        assert_eq!(cmd.title, "Send Request");
        assert!(cmd.arguments.is_none());
    }

    #[test]
    fn test_command_with_arguments() {
        let cmd = Command::new("rest-client.send", "Send Request")
            .with_arguments(vec!["arg1".to_string(), "arg2".to_string()]);
        assert!(cmd.arguments.is_some());
        assert_eq!(cmd.arguments.unwrap().len(), 2);
    }
}
