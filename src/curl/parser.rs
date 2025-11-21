//! cURL command parser.
//!
//! This module provides functionality to parse cURL commands into HttpRequest structures.
//! Supports common cURL flags including headers, methods, bodies, and authentication.

use crate::models::request::{HttpMethod, HttpRequest};
use std::collections::HashMap;
use std::path::PathBuf;

/// Errors that can occur during cURL parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// The input string is empty or contains only whitespace.
    EmptyInput,
    /// The command doesn't start with "curl".
    NotACurlCommand,
    /// No URL was found in the command.
    MissingUrl,
    /// Invalid HTTP method specified.
    InvalidMethod(String),
    /// Invalid header format.
    InvalidHeader(String),
    /// Unsupported cURL flag encountered.
    UnsupportedFlag(String),
    /// Quote mismatch in the command.
    UnbalancedQuotes,
    /// General parsing error.
    ParseError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "Input is empty"),
            ParseError::NotACurlCommand => write!(f, "Command does not start with 'curl'"),
            ParseError::MissingUrl => write!(f, "No URL found in cURL command"),
            ParseError::InvalidMethod(m) => write!(f, "Invalid HTTP method: {}", m),
            ParseError::InvalidHeader(h) => write!(f, "Invalid header format: {}", h),
            ParseError::UnsupportedFlag(flag) => {
                write!(f, "Unsupported cURL flag: {} (will be ignored)", flag)
            }
            ParseError::UnbalancedQuotes => write!(f, "Unbalanced quotes in command"),
            ParseError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a cURL command string into an HttpRequest.
///
/// # Arguments
///
/// * `curl_str` - The cURL command string to parse
///
/// # Returns
///
/// `Result<HttpRequest, ParseError>` - The parsed request or an error
///
/// # Examples
///
/// ```
/// use rest_client::curl::parser::parse_curl_command;
///
/// let curl = r#"curl -X POST https://api.example.com/users -H "Content-Type: application/json" -d '{"name":"John"}'"#;
/// let request = parse_curl_command(curl).unwrap();
/// assert_eq!(request.url, "https://api.example.com/users");
/// ```
pub fn parse_curl_command(curl_str: &str) -> Result<HttpRequest, ParseError> {
    let trimmed = curl_str.trim();

    if trimmed.is_empty() {
        return Err(ParseError::EmptyInput);
    }

    // Check if it starts with "curl"
    if !trimmed.starts_with("curl") {
        return Err(ParseError::NotACurlCommand);
    }

    // Tokenize the command, respecting quotes
    let tokens = tokenize(trimmed)?;

    // Parse tokens into request components
    parse_tokens(&tokens)
}

/// Tokenizes a cURL command, respecting quoted strings.
fn tokenize(input: &str) -> Result<Vec<String>, ParseError> {
    // First, remove line continuation backslashes (backslash followed by newline)
    let cleaned = input.replace("\\\n", " ").replace("\\\r\n", " ");

    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;
    let chars: Vec<char> = cleaned.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if escape_next {
            current_token.push(ch);
            escape_next = false;
            i += 1;
            continue;
        }

        if ch == '\\' && (in_single_quote || in_double_quote) {
            // Check if next char exists
            if i + 1 < chars.len() {
                escape_next = true;
                i += 1;
                continue;
            }
        }

        match ch {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' | '\n' | '\r' if !in_single_quote && !in_double_quote => {
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear();
                }
            }
            _ => {
                current_token.push(ch);
            }
        }

        i += 1;
    }

    if in_single_quote || in_double_quote {
        return Err(ParseError::UnbalancedQuotes);
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    Ok(tokens)
}

/// Parses tokens into an HttpRequest.
fn parse_tokens(tokens: &[String]) -> Result<HttpRequest, ParseError> {
    let mut method = HttpMethod::GET; // Default method
    let mut url: Option<String> = None;
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body: Option<String> = None;
    let mut unsupported_flags: Vec<String> = Vec::new();

    let mut i = 0;

    // Skip "curl" command itself
    if i < tokens.len() && tokens[i] == "curl" {
        i += 1;
    }

    while i < tokens.len() {
        let token = &tokens[i];

        // Handle flags
        if token.starts_with('-') {
            match token.as_str() {
                // Method flags
                "-X" | "--request" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(ParseError::ParseError(
                            "Missing method after -X".to_string(),
                        ));
                    }
                    let method_str = &tokens[i];
                    method = HttpMethod::from_str(method_str)
                        .ok_or_else(|| ParseError::InvalidMethod(method_str.to_string()))?;
                }

                // Header flags
                "-H" | "--header" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(ParseError::ParseError(
                            "Missing header after -H".to_string(),
                        ));
                    }
                    let header_str = &tokens[i];
                    parse_header(header_str, &mut headers)?;
                }

                // Data/body flags
                "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(ParseError::ParseError("Missing data after -d".to_string()));
                    }
                    let data = &tokens[i];

                    // Concatenate multiple -d flags
                    if let Some(existing_body) = &body {
                        body = Some(format!("{}&{}", existing_body, data));
                    } else {
                        body = Some(data.clone());

                        // Auto-detect JSON and set Content-Type if not already set
                        if data.trim().starts_with('{') || data.trim().starts_with('[') {
                            headers
                                .entry("Content-Type".to_string())
                                .or_insert_with(|| "application/json".to_string());
                        }
                    }

                    // Set method to POST if still GET and body is present
                    if method == HttpMethod::GET {
                        method = HttpMethod::POST;
                    }
                }

                // Authentication flag
                "-u" | "--user" => {
                    i += 1;
                    if i >= tokens.len() {
                        return Err(ParseError::ParseError(
                            "Missing credentials after -u".to_string(),
                        ));
                    }
                    let credentials = &tokens[i];
                    let encoded = base64_encode(credentials);
                    headers.insert("Authorization".to_string(), format!("Basic {}", encoded));
                }

                // Common flags that we can safely ignore
                "--compressed" | "-k" | "--insecure" | "-L" | "--location" | "-s" | "--silent"
                | "-v" | "--verbose" | "-i" | "--include" => {
                    // These flags don't affect the HTTP request itself
                }

                // User-Agent (handle specially since it's a header)
                "-A" | "--user-agent" => {
                    i += 1;
                    if i < tokens.len() {
                        headers.insert("User-Agent".to_string(), tokens[i].clone());
                    }
                }

                // Flags with arguments that we ignore
                "-o" | "--output" | "-w" | "--write-out" | "--max-time" | "-m"
                | "--connect-timeout" => {
                    unsupported_flags.push(token.clone());
                    i += 1; // Skip the argument too
                }

                // Other unknown flags
                _ => {
                    unsupported_flags.push(token.clone());
                }
            }
        } else {
            // Not a flag, likely the URL
            if url.is_none() {
                url = Some(token.clone());
            }
        }

        i += 1;
    }

    // Validate we found a URL
    let url = url.ok_or(ParseError::MissingUrl)?;

    // Create the request
    let request = HttpRequest {
        id: uuid::Uuid::new_v4().to_string(),
        method,
        url,
        http_version: Some("HTTP/1.1".to_string()),
        headers,
        body,
        line_number: 0,
        file_path: PathBuf::new(),
    };

    Ok(request)
}

/// Parses a header string in the format "Name: Value".
fn parse_header(header_str: &str, headers: &mut HashMap<String, String>) -> Result<(), ParseError> {
    if let Some(colon_pos) = header_str.find(':') {
        let name = header_str[..colon_pos].trim().to_string();
        let value = header_str[colon_pos + 1..].trim().to_string();
        headers.insert(name, value);
        Ok(())
    } else {
        Err(ParseError::InvalidHeader(header_str.to_string()))
    }
}

/// Base64 encodes a string (for Basic authentication).
fn base64_encode(input: &str) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_get_request() {
        let curl = "curl https://api.example.com/users";
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(result.method, HttpMethod::GET);
        assert_eq!(result.url, "https://api.example.com/users");
        assert!(result.headers.is_empty());
        assert_eq!(result.body, None);
    }

    #[test]
    fn test_post_with_data() {
        let curl = r#"curl -X POST https://api.example.com/users -d '{"name":"John"}'"#;
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(result.method, HttpMethod::POST);
        assert_eq!(result.url, "https://api.example.com/users");
        assert_eq!(result.body, Some(r#"{"name":"John"}"#.to_string()));
    }

    #[test]
    fn test_headers() {
        let curl = r#"curl -H "Content-Type: application/json" -H "Authorization: Bearer token123" https://api.example.com"#;
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(
            result.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            result.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_long_flags() {
        let curl = r#"curl --request PUT --header "Content-Type: application/json" --data '{"update":true}' https://api.example.com/resource/1"#;
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(result.method, HttpMethod::PUT);
        assert_eq!(result.url, "https://api.example.com/resource/1");
        assert_eq!(result.body, Some(r#"{"update":true}"#.to_string()));
    }

    #[test]
    fn test_basic_auth() {
        let curl = r#"curl -u username:password https://api.example.com"#;
        let result = parse_curl_command(curl).unwrap();

        assert!(result.headers.contains_key("Authorization"));
        let auth_header = result.headers.get("Authorization").unwrap();
        assert!(auth_header.starts_with("Basic "));
    }

    #[test]
    fn test_auto_content_type_for_json() {
        let curl = r#"curl -d '{"key":"value"}' https://api.example.com"#;
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(
            result.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(result.method, HttpMethod::POST); // Should auto-switch to POST
    }

    #[test]
    fn test_multiple_data_flags() {
        let curl = r#"curl -d "name=John" -d "age=30" https://api.example.com"#;
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(result.body, Some("name=John&age=30".to_string()));
    }

    #[test]
    fn test_empty_input() {
        let result = parse_curl_command("");
        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    #[test]
    fn test_not_curl_command() {
        let result = parse_curl_command("wget https://example.com");
        assert!(matches!(result, Err(ParseError::NotACurlCommand)));
    }

    #[test]
    fn test_missing_url() {
        let result = parse_curl_command("curl -X POST");
        assert!(matches!(result, Err(ParseError::MissingUrl)));
    }

    #[test]
    fn test_invalid_method() {
        let result = parse_curl_command("curl -X INVALID https://example.com");
        assert!(matches!(result, Err(ParseError::InvalidMethod(_))));
    }

    #[test]
    fn test_unbalanced_quotes() {
        let result =
            parse_curl_command(r#"curl -H "Content-Type: application/json https://example.com"#);
        assert!(matches!(result, Err(ParseError::UnbalancedQuotes)));
    }

    #[test]
    fn test_complex_real_world_example() {
        let curl = r#"curl -X POST 'https://api.github.com/repos/owner/repo/issues' \
  -H 'Accept: application/vnd.github.v3+json' \
  -H 'Authorization: Bearer ghp_token123' \
  -d '{"title":"Bug report","body":"Description here","labels":["bug"]}'"#;

        let result = parse_curl_command(curl).unwrap();

        assert_eq!(result.method, HttpMethod::POST);
        assert_eq!(result.url, "https://api.github.com/repos/owner/repo/issues");
        assert_eq!(
            result.headers.get("Accept"),
            Some(&"application/vnd.github.v3+json".to_string())
        );
        assert_eq!(
            result.headers.get("Authorization"),
            Some(&"Bearer ghp_token123".to_string())
        );
        assert!(result.body.is_some());
    }

    #[test]
    fn test_tokenize_with_quotes() {
        let input = r#"curl -H "Content-Type: application/json" https://example.com"#;
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens[0], "curl");
        assert_eq!(tokens[1], "-H");
        assert_eq!(tokens[2], "Content-Type: application/json");
        assert_eq!(tokens[3], "https://example.com");
    }

    #[test]
    fn test_tokenize_with_single_quotes() {
        let input = r#"curl -d '{"key":"value"}' https://example.com"#;
        let tokens = tokenize(input).unwrap();

        assert_eq!(tokens[2], r#"{"key":"value"}"#);
    }

    #[test]
    fn test_compressed_flag_ignored() {
        let curl = "curl --compressed https://api.example.com";
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(result.url, "https://api.example.com");
    }

    #[test]
    fn test_insecure_flag_ignored() {
        let curl = "curl -k https://api.example.com";
        let result = parse_curl_command(curl).unwrap();

        assert_eq!(result.url, "https://api.example.com");
    }
}
