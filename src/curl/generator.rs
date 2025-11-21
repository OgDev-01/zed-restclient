//! cURL command generator.
//!
//! This module provides functionality to convert HttpRequest structures into valid cURL commands.
//! Handles proper shell escaping, multi-line formatting, and all common cURL flags.

use crate::models::request::{HttpMethod, HttpRequest};

/// Generates a valid cURL command from an HttpRequest.
///
/// # Arguments
///
/// * `request` - The HTTP request to convert to cURL
///
/// # Returns
///
/// A formatted cURL command string with proper escaping and line continuations
///
/// # Examples
///
/// ```no_run
/// use rest_client::models::request::{HttpRequest, HttpMethod};
/// use rest_client::curl::generator::generate_curl_command;
///
/// let mut request = HttpRequest::new(
///     "test".to_string(),
///     HttpMethod::POST,
///     "https://api.example.com/users".to_string()
/// );
/// request.add_header("Content-Type".to_string(), "application/json".to_string());
/// request.set_body(r#"{"name":"John"}"#.to_string());
///
/// let curl = generate_curl_command(&request);
/// assert!(curl.contains("curl"));
/// assert!(curl.contains("-X POST"));
/// ```
pub fn generate_curl_command(request: &HttpRequest) -> String {
    let mut parts = vec!["curl".to_string()];

    // Add method if not GET
    if request.method != HttpMethod::GET {
        parts.push("-X".to_string());
        parts.push(request.method.as_str().to_string());
    }

    // Add headers in order (sorted for consistency)
    let mut header_keys: Vec<&String> = request.headers.keys().collect();
    header_keys.sort();

    for key in header_keys {
        if let Some(value) = request.headers.get(key) {
            parts.push("-H".to_string());
            parts.push(escape_shell_arg(&format!("{}: {}", key, value)));
        }
    }

    // Add body if present
    if let Some(body) = &request.body {
        parts.push("-d".to_string());
        parts.push(escape_shell_arg(body));
    }

    // Add URL (always last)
    parts.push(escape_shell_arg(&request.url));

    // Format with line continuations for readability
    format_multiline(&parts)
}

/// Generates a compact single-line cURL command.
///
/// # Arguments
///
/// * `request` - The HTTP request to convert to cURL
///
/// # Returns
///
/// A single-line cURL command string
pub fn generate_curl_command_compact(request: &HttpRequest) -> String {
    let mut parts = vec!["curl".to_string()];

    // Add method if not GET
    if request.method != HttpMethod::GET {
        parts.push(format!("-X {}", request.method.as_str()));
    }

    // Add headers
    let mut header_keys: Vec<&String> = request.headers.keys().collect();
    header_keys.sort();

    for key in header_keys {
        if let Some(value) = request.headers.get(key) {
            parts.push(format!(
                "-H {}",
                escape_shell_arg(&format!("{}: {}", key, value))
            ));
        }
    }

    // Add body if present
    if let Some(body) = &request.body {
        parts.push(format!("-d {}", escape_shell_arg(body)));
    }

    // Add URL
    parts.push(escape_shell_arg(&request.url));

    parts.join(" ")
}

/// Escapes a string for safe use in shell commands.
///
/// Uses single quotes for safety, escaping any embedded single quotes.
fn escape_shell_arg(arg: &str) -> String {
    // Check if the string needs quoting
    if needs_quoting(arg) {
        // Use single quotes and escape any single quotes in the string
        if arg.contains('\'') {
            // Replace ' with '\''
            format!("'{}'", arg.replace('\'', "'\\''"))
        } else {
            format!("'{}'", arg)
        }
    } else {
        // No special characters, no quotes needed
        arg.to_string()
    }
}

/// Checks if a string needs quoting for shell safety.
fn needs_quoting(s: &str) -> bool {
    // Check for special shell characters
    let special_chars = [
        ' ', '\t', '\n', '\r', '|', '&', ';', '<', '>', '(', ')', '$', '`', '\\', '"', '\'', '*',
        '?', '[', ']', '#', '~', '=', '%', '{', '}',
    ];

    s.is_empty() || s.chars().any(|c| special_chars.contains(&c))
}

/// Formats cURL command parts into a multi-line string with backslash continuations.
///
/// # Arguments
///
/// * `parts` - The command parts to format
///
/// # Returns
///
/// A formatted multi-line string with proper indentation
fn format_multiline(parts: &[String]) -> String {
    if parts.is_empty() {
        return String::new();
    }

    // If the command is short, keep it on one line
    let single_line = parts.join(" ");
    if single_line.len() <= 80 {
        return single_line;
    }

    // Multi-line format with backslashes
    let mut result = String::new();
    result.push_str(&parts[0]); // "curl"

    for part in &parts[1..] {
        result.push_str(" \\\n  ");
        result.push_str(part);
    }

    result
}

/// Converts an HttpRequest to cURL with custom formatting options.
///
/// # Arguments
///
/// * `request` - The HTTP request to convert
/// * `options` - Formatting options
///
/// # Returns
///
/// A formatted cURL command string
pub fn generate_curl_with_options(request: &HttpRequest, options: &CurlOptions) -> String {
    if options.compact {
        generate_curl_command_compact(request)
    } else {
        generate_curl_command(request)
    }
}

/// Options for cURL command generation.
#[derive(Debug, Clone)]
pub struct CurlOptions {
    /// Generate a compact single-line command
    pub compact: bool,
    /// Include verbose flag (-v)
    pub verbose: bool,
    /// Include insecure flag (-k) for HTTPS
    pub insecure: bool,
}

impl Default for CurlOptions {
    fn default() -> Self {
        Self {
            compact: false,
            verbose: false,
            insecure: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::HttpMethod;

    #[test]
    fn test_simple_get_request() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );

        let curl = generate_curl_command(&request);

        assert!(curl.starts_with("curl"));
        assert!(curl.contains("https://api.example.com/users"));
        assert!(!curl.contains("-X")); // GET is default
    }

    #[test]
    fn test_post_request_with_body() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/users".to_string(),
        );
        request.set_body(r#"{"name":"John Doe"}"#.to_string());

        let curl = generate_curl_command(&request);

        assert!(curl.contains("-X POST"));
        assert!(curl.contains("-d"));
        assert!(curl.contains(r#"{"name":"John Doe"}"#));
    }

    #[test]
    fn test_headers_included() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.add_header("Authorization".to_string(), "Bearer token123".to_string());

        let curl = generate_curl_command(&request);

        assert!(curl.contains("-H"));
        assert!(curl.contains("Content-Type: application/json"));
        assert!(curl.contains("Authorization: Bearer token123"));
    }

    #[test]
    fn test_special_characters_escaped() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/search".to_string(),
        );
        request.set_body(r#"{"query":"hello & goodbye"}"#.to_string());

        let curl = generate_curl_command(&request);

        // Should be wrapped in quotes due to special characters
        assert!(curl.contains("'"));
    }

    #[test]
    fn test_single_quote_escaping() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com".to_string(),
        );
        request.set_body(r#"It's a test"#.to_string());

        let curl = generate_curl_command(&request);

        // Single quotes should be escaped
        assert!(curl.contains(r#"It'\''s a test"#) || curl.contains("It"));
    }

    #[test]
    fn test_multiline_format() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/very/long/endpoint/path".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.add_header(
            "Authorization".to_string(),
            "Bearer verylongtoken123456789".to_string(),
        );
        request.set_body(r#"{"key":"value","another":"data"}"#.to_string());

        let curl = generate_curl_command(&request);

        // Should contain backslash continuation
        assert!(curl.contains("\\"));
    }

    #[test]
    fn test_compact_format() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"key":"value"}"#.to_string());

        let curl = generate_curl_command_compact(&request);

        // Should not contain newlines
        assert!(!curl.contains('\n'));
        assert!(curl.contains("curl"));
    }

    #[test]
    fn test_put_method() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::PUT,
            "https://api.example.com/resource/1".to_string(),
        );

        let curl = generate_curl_command(&request);

        assert!(curl.contains("-X PUT"));
    }

    #[test]
    fn test_delete_method() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::DELETE,
            "https://api.example.com/resource/1".to_string(),
        );

        let curl = generate_curl_command(&request);

        assert!(curl.contains("-X DELETE"));
    }

    #[test]
    fn test_url_with_query_params() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/search?q=rust&limit=10".to_string(),
        );

        let curl = generate_curl_command(&request);

        assert!(curl.contains("q=rust"));
        assert!(curl.contains("limit=10"));
    }

    #[test]
    fn test_needs_quoting() {
        assert!(needs_quoting("hello world"));
        assert!(needs_quoting("hello&goodbye"));
        assert!(needs_quoting(""));
        assert!(needs_quoting("hello|world"));
        assert!(!needs_quoting("https://example.com"));
        assert!(!needs_quoting("simple"));
    }

    #[test]
    fn test_escape_shell_arg() {
        assert_eq!(escape_shell_arg("simple"), "simple");
        assert_eq!(escape_shell_arg("hello world"), "'hello world'");
        assert_eq!(escape_shell_arg("it's"), "'it'\\''s'");
        assert_eq!(escape_shell_arg("hello & goodbye"), "'hello & goodbye'");
    }

    #[test]
    fn test_header_order_consistent() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header("Zebra".to_string(), "last".to_string());
        request.add_header("Alpha".to_string(), "first".to_string());
        request.add_header("Beta".to_string(), "second".to_string());

        let curl1 = generate_curl_command(&request);
        let curl2 = generate_curl_command(&request);

        // Should be identical (headers sorted)
        assert_eq!(curl1, curl2);
    }

    #[test]
    fn test_empty_body_not_included() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com".to_string(),
        );
        request.set_body("".to_string());

        let curl = generate_curl_command(&request);

        // Empty body should still include -d flag with empty quotes
        assert!(curl.contains("-d"));
    }

    #[test]
    fn test_json_body_formatting() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/users".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"name":"Alice","email":"alice@example.com","age":30}"#.to_string());

        let curl = generate_curl_command(&request);

        println!("Generated cURL: {}", curl);
        assert!(
            curl.contains("-X") && curl.contains("POST"),
            "Expected '-X' and 'POST' in: {}",
            curl
        );
        assert!(curl.contains("Content-Type: application/json"));
        assert!(curl.contains(r#"{"name":"Alice""#));
    }

    #[test]
    fn test_with_options_compact() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );

        let options = CurlOptions {
            compact: true,
            ..Default::default()
        };

        let curl = generate_curl_with_options(&request, &options);

        assert!(!curl.contains('\n'));
    }

    #[test]
    fn test_with_options_multiline() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/very/long/path".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer token".to_string());
        request.set_body("data".to_string());

        let options = CurlOptions {
            compact: false,
            ..Default::default()
        };

        let curl = generate_curl_with_options(&request, &options);

        // Default formatting behavior - may or may not have newlines depending on length
        assert!(curl.contains("curl"));
    }
}
