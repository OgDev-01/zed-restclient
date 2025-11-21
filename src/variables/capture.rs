//! Capture directive parsing for extracting variables from HTTP responses.
//!
//! This module provides functionality to parse `@capture` directives in .http files,
//! which allow users to extract values from HTTP responses and store them as variables
//! for use in subsequent requests.
//!
//! # Syntax
//!
//! ```text
//! # @capture variableName = JSONPath
//! # @capture variableName = XPath
//! # @capture variableName = headers.HeaderName
//! ```
//!
//! # Examples
//!
//! ```text
//! POST https://api.example.com/auth/login
//! Content-Type: application/json
//!
//! {"username": "test", "password": "pass"}
//!
//! # @capture authToken = $.token
//! # @capture userId = $.user.id
//! # @capture sessionId = headers.X-Session-Id
//! ```

use regex::Regex;
use std::sync::LazyLock;

/// Regex pattern for matching capture directives.
///
/// Matches: `# @capture variableName = path`
/// - Allows optional whitespace around components
/// - Variable names must be valid identifiers (alphanumeric + underscore)
/// - Path can contain dots, brackets, and other JSONPath/XPath characters
static CAPTURE_DIRECTIVE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*#\s*@capture\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+?)\s*$")
        .expect("Failed to compile capture directive regex")
});

/// Type of extraction path used in a capture directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathType {
    /// JSONPath expression for extracting from JSON responses.
    ///
    /// Example: `$.user.name` or `$.items[0].id`
    JsonPath(String),

    /// XPath expression for extracting from XML responses.
    ///
    /// Example: `/root/user/name` or `//item[@id='1']`
    XPath(String),

    /// Header extraction path.
    ///
    /// Example: `headers.Authorization` or `headers.Content-Type`
    Header(String),
}

impl PathType {
    /// Determines the path type from a path string.
    ///
    /// # Arguments
    ///
    /// * `path` - The path string to analyze
    ///
    /// # Returns
    ///
    /// The appropriate `PathType` variant based on path syntax.
    ///
    /// # Logic
    ///
    /// - If starts with "headers." -> Header extraction
    /// - If starts with "$" or contains JSONPath syntax -> JSONPath
    /// - Otherwise -> XPath (default for XML)
    pub fn from_path(path: &str) -> Self {
        let trimmed = path.trim();

        // Check for header extraction
        if let Some(header_name) = trimmed.strip_prefix("headers.") {
            return PathType::Header(header_name.trim().to_string());
        }

        // Check for JSONPath (starts with $ or @ or contains typical JSONPath syntax)
        if trimmed.starts_with('$') || trimmed.starts_with("@.") || trimmed.contains("$.") {
            return PathType::JsonPath(trimmed.to_string());
        }

        // Check if it contains brackets but doesn't start with / or // (likely JSONPath array access)
        if trimmed.contains('[') && trimmed.contains(']') && !trimmed.starts_with('/') {
            return PathType::JsonPath(trimmed.to_string());
        }

        // Default to XPath for XML
        PathType::XPath(trimmed.to_string())
    }
}

/// A parsed capture directive from an .http file.
///
/// Represents a single `@capture` comment that extracts a value from
/// a response and stores it in a named variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureDirective {
    /// Name of the variable to store the captured value.
    ///
    /// Must be a valid identifier (alphanumeric + underscore).
    pub variable_name: String,

    /// Type and value of the extraction path.
    pub path: PathType,
}

impl CaptureDirective {
    /// Creates a new CaptureDirective.
    ///
    /// # Arguments
    ///
    /// * `variable_name` - Name of the variable
    /// * `path` - Extraction path (JSONPath, XPath, or header)
    ///
    /// # Returns
    ///
    /// A new `CaptureDirective` instance.
    pub fn new(variable_name: String, path: PathType) -> Self {
        Self {
            variable_name,
            path,
        }
    }
}

/// Parses a capture directive from a comment line.
///
/// # Arguments
///
/// * `comment` - A line from an .http file that may contain a capture directive
///
/// # Returns
///
/// `Some(CaptureDirective)` if the line is a valid capture directive,
/// `None` otherwise.
///
/// # Examples
///
/// ```
/// use rest_client::variables::capture::parse_capture_directive;
///
/// let directive = parse_capture_directive("# @capture token = $.access_token");
/// assert!(directive.is_some());
///
/// let invalid = parse_capture_directive("# This is just a comment");
/// assert!(invalid.is_none());
/// ```
pub fn parse_capture_directive(comment: &str) -> Option<CaptureDirective> {
    let captures = CAPTURE_DIRECTIVE_REGEX.captures(comment)?;

    // Extract variable name (group 1)
    let variable_name = captures.get(1)?.as_str().to_string();

    // Extract path (group 2)
    let path_str = captures.get(2)?.as_str();
    let path = PathType::from_path(path_str);

    Some(CaptureDirective::new(variable_name, path))
}

/// Parses multiple capture directives from a block of text.
///
/// # Arguments
///
/// * `text` - Multi-line text that may contain capture directives
///
/// # Returns
///
/// A vector of all valid `CaptureDirective` instances found in the text.
///
/// # Examples
///
/// ```
/// use rest_client::variables::capture::parse_capture_directives;
///
/// let text = "# @capture token = $.access_token\n# @capture userId = $.user.id\n# Regular comment\n# @capture sessionId = headers.X-Session-Id";
///
/// let directives = parse_capture_directives(text);
/// assert_eq!(directives.len(), 3);
/// ```
pub fn parse_capture_directives(text: &str) -> Vec<CaptureDirective> {
    text.lines().filter_map(parse_capture_directive).collect()
}

/// Validates a JSONPath expression for syntax correctness.
///
/// # Arguments
///
/// * `path` - JSONPath expression to validate
///
/// # Returns
///
/// `true` if the path appears to be valid, `false` otherwise.
///
/// # Note
///
/// This performs basic syntax validation. Full validation happens
/// during execution when the JSONPath is evaluated against actual data.
pub fn validate_jsonpath(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

    // Must start with $ or @
    if !path.starts_with('$') && !path.starts_with('@') {
        return false;
    }

    // Check for balanced and properly ordered brackets
    let mut depth = 0;
    for ch in path.chars() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth < 0 {
                    return false; // Closing bracket before opening
                }
            }
            _ => {}
        }
    }

    if depth != 0 {
        return false; // Unbalanced brackets
    }

    // Basic validation passed
    true
}

/// Validates an XPath expression for syntax correctness.
///
/// # Arguments
///
/// * `path` - XPath expression to validate
///
/// # Returns
///
/// `true` if the path appears to be valid, `false` otherwise.
///
/// # Note
///
/// This performs basic syntax validation. Full validation happens
/// during execution when the XPath is evaluated against actual data.
pub fn validate_xpath(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }

    // XPath can start with / for absolute path or // for anywhere
    // or just an element name for relative path
    // This is a basic check - full validation during execution

    true
}

/// Validates a header name for extraction.
///
/// # Arguments
///
/// * `header_name` - Header name to validate
///
/// # Returns
///
/// `true` if the header name is valid, `false` otherwise.
pub fn validate_header_name(header_name: &str) -> bool {
    if header_name.is_empty() {
        return false;
    }

    // Header names are case-insensitive and can contain letters, numbers, and hyphens
    header_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capture_directive_jsonpath() {
        let result = parse_capture_directive("# @capture token = $.access_token");
        assert!(result.is_some());

        let directive = result.unwrap();
        assert_eq!(directive.variable_name, "token");
        assert!(matches!(directive.path, PathType::JsonPath(_)));

        if let PathType::JsonPath(path) = directive.path {
            assert_eq!(path, "$.access_token");
        }
    }

    #[test]
    fn test_parse_capture_directive_header() {
        let result = parse_capture_directive("# @capture sessionId = headers.X-Session-Id");
        assert!(result.is_some());

        let directive = result.unwrap();
        assert_eq!(directive.variable_name, "sessionId");
        assert!(matches!(directive.path, PathType::Header(_)));

        if let PathType::Header(name) = directive.path {
            assert_eq!(name, "X-Session-Id");
        }
    }

    #[test]
    fn test_parse_capture_directive_xpath() {
        let result = parse_capture_directive("# @capture userId = /root/user/id");
        assert!(result.is_some());

        let directive = result.unwrap();
        assert_eq!(directive.variable_name, "userId");
        assert!(matches!(directive.path, PathType::XPath(_)));
    }

    #[test]
    fn test_parse_capture_directive_with_whitespace() {
        let result = parse_capture_directive("  #   @capture   token   =   $.access_token  ");
        assert!(result.is_some());

        let directive = result.unwrap();
        assert_eq!(directive.variable_name, "token");
    }

    #[test]
    fn test_parse_capture_directive_invalid() {
        assert!(parse_capture_directive("# Just a comment").is_none());
        assert!(parse_capture_directive("# @capture").is_none());
        assert!(parse_capture_directive("# @capture token").is_none());
        assert!(parse_capture_directive("# @capture token =").is_none());
        assert!(parse_capture_directive("@capture token = $.path").is_none()); // Missing #
    }

    #[test]
    fn test_parse_capture_directive_invalid_variable_name() {
        // Invalid variable names (must start with letter or underscore)
        assert!(parse_capture_directive("# @capture 123token = $.path").is_none());
        assert!(parse_capture_directive("# @capture token-name = $.path").is_none());
        assert!(parse_capture_directive("# @capture token.name = $.path").is_none());
    }

    #[test]
    fn test_parse_capture_directive_valid_variable_names() {
        assert!(parse_capture_directive("# @capture token = $.path").is_some());
        assert!(parse_capture_directive("# @capture _token = $.path").is_some());
        assert!(parse_capture_directive("# @capture token123 = $.path").is_some());
        assert!(parse_capture_directive("# @capture token_name = $.path").is_some());
    }

    #[test]
    fn test_parse_capture_directives_multiple() {
        let text = r#"
# @capture token = $.access_token
# @capture userId = $.user.id
# Regular comment
# @capture sessionId = headers.X-Session-Id
# Another comment
# @capture userName = $.user.name
"#;

        let directives = parse_capture_directives(text);
        assert_eq!(directives.len(), 4);
        assert_eq!(directives[0].variable_name, "token");
        assert_eq!(directives[1].variable_name, "userId");
        assert_eq!(directives[2].variable_name, "sessionId");
        assert_eq!(directives[3].variable_name, "userName");
    }

    #[test]
    fn test_path_type_from_path_jsonpath() {
        assert!(matches!(
            PathType::from_path("$.token"),
            PathType::JsonPath(_)
        ));
        assert!(matches!(
            PathType::from_path("$.user.name"),
            PathType::JsonPath(_)
        ));
        assert!(matches!(
            PathType::from_path("$.items[0]"),
            PathType::JsonPath(_)
        ));
        assert!(matches!(
            PathType::from_path("@.value"),
            PathType::JsonPath(_)
        ));
    }

    #[test]
    fn test_path_type_from_path_header() {
        let path = PathType::from_path("headers.Authorization");
        assert!(matches!(path, PathType::Header(_)));

        if let PathType::Header(name) = path {
            assert_eq!(name, "Authorization");
        }

        let path2 = PathType::from_path("headers.Content-Type");
        assert!(matches!(path2, PathType::Header(_)));
    }

    #[test]
    fn test_path_type_from_path_xpath() {
        assert!(matches!(
            PathType::from_path("/root/user/name"),
            PathType::XPath(_)
        ));
        assert!(matches!(
            PathType::from_path("//user[@id='1']"),
            PathType::XPath(_)
        ));
    }

    #[test]
    fn test_validate_jsonpath() {
        assert!(validate_jsonpath("$.token"));
        assert!(validate_jsonpath("$.user.name"));
        assert!(validate_jsonpath("$.items[0]"));
        assert!(validate_jsonpath("$.items[0].name"));
        assert!(validate_jsonpath("@.value"));

        assert!(!validate_jsonpath(""));
        assert!(!validate_jsonpath("token")); // Doesn't start with $ or @
        assert!(!validate_jsonpath("$.items[0")); // Unbalanced brackets
        assert!(!validate_jsonpath("$.items]0[")); // Unbalanced brackets
    }

    #[test]
    fn test_validate_xpath() {
        assert!(validate_xpath("/root/user"));
        assert!(validate_xpath("//user"));
        assert!(validate_xpath("user/name"));

        assert!(!validate_xpath(""));
    }

    #[test]
    fn test_validate_header_name() {
        assert!(validate_header_name("Authorization"));
        assert!(validate_header_name("Content-Type"));
        assert!(validate_header_name("X-Session-Id"));
        assert!(validate_header_name("X_Custom_Header"));

        assert!(!validate_header_name(""));
        assert!(!validate_header_name("Invalid Header")); // Space not allowed
        assert!(!validate_header_name("Invalid@Header")); // @ not allowed
    }

    #[test]
    fn test_complex_jsonpath_patterns() {
        let directives = vec![
            "# @capture id = $.data.items[0].id",
            "# @capture name = $.data.items[0].user.name",
            "# @capture count = $.data.total_count",
            "# @capture nested = $.response.data.nested.value",
        ];

        for directive_str in directives {
            let result = parse_capture_directive(directive_str);
            assert!(result.is_some(), "Failed to parse: {}", directive_str);
        }
    }

    #[test]
    fn test_capture_directive_new() {
        let directive = CaptureDirective::new(
            "token".to_string(),
            PathType::JsonPath("$.access_token".to_string()),
        );

        assert_eq!(directive.variable_name, "token");
        assert!(matches!(directive.path, PathType::JsonPath(_)));
    }
}
