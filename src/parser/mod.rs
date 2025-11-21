//! HTTP request file parser.
//!
//! This module provides functionality to parse `.http` and `.rest` files into
//! structured `HttpRequest` objects. It handles multiple requests separated by
//! `###` delimiters, comments, headers, and request bodies.

pub mod error;

use crate::models::{HttpMethod, HttpRequest};
use error::ParseError;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

/// Parses the content of an HTTP request file into a vector of requests.
///
/// Requests are separated by lines containing only `###`. Comments (lines
/// starting with `#` or `//`) are ignored. Each request block is parsed
/// independently.
///
/// # Arguments
///
/// * `content` - The full content of the HTTP request file
/// * `file_path` - Path to the file being parsed (for error reporting)
///
/// # Returns
///
/// A `Result` containing a vector of `HttpRequest` objects on success, or a
/// `ParseError` if parsing fails.
///
/// # Examples
///
/// ```
/// use rest_client::parser::parse_file;
/// use std::path::PathBuf;
///
/// let content = r#"
/// GET https://api.example.com/users
///
/// ###
///
/// POST https://api.example.com/users
/// Content-Type: application/json
///
/// {"name": "John"}
/// "#;
///
/// let requests = parse_file(content, &PathBuf::from("test.http")).unwrap();
/// assert_eq!(requests.len(), 2);
/// ```
pub fn parse_file(content: &str, file_path: &PathBuf) -> Result<Vec<HttpRequest>, ParseError> {
    let mut requests = Vec::new();
    let mut current_block = Vec::new();
    let mut block_start_line = 1;
    let mut current_line = 1;

    // Normalize line endings (handle both \r\n and \n)
    let normalized_content = content.replace("\r\n", "\n");

    for line in normalized_content.lines() {
        // Check if this is a request delimiter
        if line.trim() == "###" {
            // Parse the accumulated block if it's not empty
            if !current_block.is_empty() {
                let request = parse_request(&current_block, block_start_line, file_path)?;
                requests.push(request);
                current_block.clear();
            }
            block_start_line = current_line + 1;
        } else {
            current_block.push((current_line, line));
        }
        current_line += 1;
    }

    // Parse the last block if it exists
    if !current_block.is_empty() {
        let request = parse_request(&current_block, block_start_line, file_path)?;
        requests.push(request);
    }

    Ok(requests)
}

/// Parses a single HTTP request block into an `HttpRequest` object.
///
/// # Arguments
///
/// * `lines` - Vector of (line_number, line_content) tuples
/// * `block_start` - Line number where this block starts
/// * `file_path` - Path to the file being parsed
///
/// # Returns
///
/// A `Result` containing an `HttpRequest` on success, or a `ParseError` if parsing fails.
pub fn parse_request(
    lines: &[(usize, &str)],
    block_start: usize,
    file_path: &PathBuf,
) -> Result<HttpRequest, ParseError> {
    if lines.is_empty() {
        return Err(ParseError::EmptyRequest { line: block_start });
    }

    // Find the first non-comment, non-empty line (the request line)
    let request_line_data = lines
        .iter()
        .find(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("//")
        })
        .ok_or(ParseError::EmptyRequest { line: block_start })?;

    let (request_line_num, request_line) = request_line_data;

    // Parse the request line (METHOD URL [HTTP_VERSION])
    let (method, url, http_version) = parse_request_line(request_line, *request_line_num)?;

    // Find where headers start (after request line) and where body starts (after blank line)
    let mut header_lines = Vec::new();
    let mut body_start_idx = None;
    let mut past_request_line = false;

    for (idx, (line_num, line)) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip until we pass the request line
        if !past_request_line {
            if line_num == request_line_num {
                past_request_line = true;
            }
            continue;
        }

        // Skip comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Empty line indicates start of body
        if trimmed.is_empty() {
            body_start_idx = Some(idx + 1);
            break;
        }

        // This is a header line
        header_lines.push((*line_num, *line));
    }

    // Extract headers
    let headers = extract_headers(&header_lines)?;

    // Extract body if present
    let body = if let Some(start_idx) = body_start_idx {
        let body_lines: Vec<&str> = lines[start_idx..]
            .iter()
            .map(|(_, line)| *line)
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with('#') && !trimmed.starts_with("//")
            })
            .collect();
        extract_body(&body_lines)
    } else {
        None
    };

    // Generate a unique ID for the request
    let id = generate_request_id(file_path, *request_line_num);

    Ok(HttpRequest {
        id,
        method,
        url,
        http_version,
        headers,
        body,
        line_number: *request_line_num,
        file_path: file_path.clone(),
    })
}

/// Parses the request line to extract method, URL, and optional HTTP version.
///
/// Supports both formats:
/// - Simple: `GET https://example.com`
/// - Full: `GET https://example.com HTTP/1.1`
///
/// # Arguments
///
/// * `line` - The request line text
/// * `line_num` - Line number for error reporting
///
/// # Returns
///
/// A tuple of (method, url, optional_http_version) on success, or a `ParseError`.
pub fn parse_request_line(
    line: &str,
    line_num: usize,
) -> Result<(HttpMethod, String, Option<String>), ParseError> {
    // Regex to match: METHOD URL [HTTP/VERSION]
    // This handles both simple and full RFC 2616 format
    let re = Regex::new(r"^([A-Z]+)\s+(\S+)(?:\s+(HTTP/\d+(?:\.\d+)?))?$").unwrap();

    let trimmed = line.trim();

    if let Some(captures) = re.captures(trimmed) {
        // Extract method
        let method_str = captures.get(1).unwrap().as_str();
        let method = HttpMethod::from_str(method_str).ok_or(ParseError::InvalidMethod {
            method: method_str.to_string(),
            line: line_num,
        })?;

        // Extract URL
        let url = captures.get(2).unwrap().as_str();

        // Validate URL format (must start with http:// or https://)
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ParseError::InvalidUrl {
                url: url.to_string(),
                line: line_num,
            });
        }

        // Extract optional HTTP version
        let http_version = captures.get(3).map(|m| m.as_str().to_string());

        Ok((method, url.to_string(), http_version))
    } else {
        // Try to extract just the method to give better error
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ParseError::MissingUrl { line: line_num });
        }

        if parts.len() == 1 {
            return Err(ParseError::MissingUrl { line: line_num });
        }

        // If we have method but invalid format, check if method is valid
        if HttpMethod::from_str(parts[0]).is_none() {
            return Err(ParseError::InvalidMethod {
                method: parts[0].to_string(),
                line: line_num,
            });
        }

        // Otherwise it's likely an invalid URL
        Err(ParseError::InvalidUrl {
            url: parts.get(1).unwrap_or(&"").to_string(),
            line: line_num,
        })
    }
}

/// Extracts headers from header lines.
///
/// Headers must be in the format "Name: Value". Lines that don't match this
/// format will result in an error.
///
/// # Arguments
///
/// * `lines` - Vector of (line_number, line_content) tuples
///
/// # Returns
///
/// A `HashMap` of header names to values on success, or a `ParseError`.
pub fn extract_headers(lines: &[(usize, &str)]) -> Result<HashMap<String, String>, ParseError> {
    let mut headers = HashMap::new();

    for (line_num, line) in lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Headers must contain a colon
        if let Some(colon_pos) = trimmed.find(':') {
            let name = trimmed[..colon_pos].trim().to_string();
            let value = trimmed[colon_pos + 1..].trim().to_string();

            if name.is_empty() {
                return Err(ParseError::InvalidHeader {
                    header: trimmed.to_string(),
                    line: *line_num,
                });
            }

            headers.insert(name, value);
        } else {
            return Err(ParseError::InvalidHeader {
                header: trimmed.to_string(),
                line: *line_num,
            });
        }
    }

    Ok(headers)
}

/// Extracts the request body from body lines.
///
/// The body is everything after the first blank line in the request block.
/// Comment lines are filtered out.
///
/// # Arguments
///
/// * `lines` - Slice of body line strings
///
/// # Returns
///
/// `Some(String)` if there's a non-empty body, `None` otherwise.
pub fn extract_body(lines: &[&str]) -> Option<String> {
    if lines.is_empty() {
        return None;
    }

    let body = lines.join("\n");
    let trimmed = body.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(body)
    }
}

/// Generates a unique ID for a request based on file path and line number.
///
/// # Arguments
///
/// * `file_path` - Path to the source file
/// * `line_num` - Line number of the request
///
/// # Returns
///
/// A unique string identifier.
fn generate_request_id(file_path: &PathBuf, line_num: usize) -> String {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    format!("{}_line_{}", file_name, line_num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_line_simple_format() {
        let result = parse_request_line("GET https://api.example.com/users", 1);
        assert!(result.is_ok());

        let (method, url, version) = result.unwrap();
        assert_eq!(method, HttpMethod::GET);
        assert_eq!(url, "https://api.example.com/users");
        assert_eq!(version, None);
    }

    #[test]
    fn test_parse_request_line_full_format() {
        let result = parse_request_line("POST https://api.example.com/data HTTP/1.1", 1);
        assert!(result.is_ok());

        let (method, url, version) = result.unwrap();
        assert_eq!(method, HttpMethod::POST);
        assert_eq!(url, "https://api.example.com/data");
        assert_eq!(version, Some("HTTP/1.1".to_string()));
    }

    #[test]
    fn test_parse_request_line_http2() {
        let result = parse_request_line("GET https://example.com HTTP/2", 1);
        assert!(result.is_ok());

        let (_, _, version) = result.unwrap();
        assert_eq!(version, Some("HTTP/2".to_string()));
    }

    #[test]
    fn test_parse_request_line_invalid_method() {
        let result = parse_request_line("INVALID https://example.com", 1);
        assert!(result.is_err());

        if let Err(ParseError::InvalidMethod { method, line }) = result {
            assert_eq!(method, "INVALID");
            assert_eq!(line, 1);
        } else {
            panic!("Expected InvalidMethod error");
        }
    }

    #[test]
    fn test_parse_request_line_missing_url() {
        let result = parse_request_line("GET", 1);
        assert!(result.is_err());

        if let Err(ParseError::MissingUrl { line }) = result {
            assert_eq!(line, 1);
        } else {
            panic!("Expected MissingUrl error");
        }
    }

    #[test]
    fn test_parse_request_line_invalid_url() {
        let result = parse_request_line("GET example.com", 1);
        assert!(result.is_err());

        if let Err(ParseError::InvalidUrl { url, line }) = result {
            assert_eq!(url, "example.com");
            assert_eq!(line, 1);
        } else {
            panic!("Expected InvalidUrl error");
        }
    }

    #[test]
    fn test_extract_headers_valid() {
        let lines = vec![
            (2, "Content-Type: application/json"),
            (3, "Authorization: Bearer token123"),
            (4, "Accept: */*"),
        ];

        let result = extract_headers(&lines);
        assert!(result.is_ok());

        let headers = result.unwrap();
        assert_eq!(headers.len(), 3);
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(headers.get("Accept"), Some(&"*/*".to_string()));
    }

    #[test]
    fn test_extract_headers_with_spaces() {
        let lines = vec![(2, "Content-Type:    application/json   ")];

        let result = extract_headers(&lines);
        assert!(result.is_ok());

        let headers = result.unwrap();
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_extract_headers_invalid_format() {
        let lines = vec![(2, "InvalidHeaderWithoutColon")];

        let result = extract_headers(&lines);
        assert!(result.is_err());

        if let Err(ParseError::InvalidHeader { header, line }) = result {
            assert_eq!(header, "InvalidHeaderWithoutColon");
            assert_eq!(line, 2);
        } else {
            panic!("Expected InvalidHeader error");
        }
    }

    #[test]
    fn test_extract_body_simple() {
        let lines = vec![r#"{"name": "John", "age": 30}"#];
        let body = extract_body(&lines);

        assert!(body.is_some());
        assert_eq!(body.unwrap(), r#"{"name": "John", "age": 30}"#);
    }

    #[test]
    fn test_extract_body_multiline() {
        let lines = vec!["{", r#"  "name": "John","#, r#"  "age": 30"#, "}"];
        let body = extract_body(&lines);

        assert!(body.is_some());
        let body_text = body.unwrap();
        assert!(body_text.contains("name"));
        assert!(body_text.contains("John"));
    }

    #[test]
    fn test_extract_body_empty() {
        let lines: Vec<&str> = vec![];
        let body = extract_body(&lines);
        assert!(body.is_none());

        let lines = vec!["   ", "  "];
        let body = extract_body(&lines);
        assert!(body.is_none());
    }

    #[test]
    fn test_parse_file_single_request() {
        let content = r#"
GET https://api.example.com/users
"#;

        let result = parse_file(content, &PathBuf::from("test.http"));
        assert!(result.is_ok());

        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, HttpMethod::GET);
        assert_eq!(requests[0].url, "https://api.example.com/users");
    }

    #[test]
    fn test_parse_file_multiple_requests() {
        let content = r#"
GET https://api.example.com/users

###

POST https://api.example.com/users
Content-Type: application/json

{"name": "John"}

###

DELETE https://api.example.com/users/1
"#;

        let result = parse_file(content, &PathBuf::from("test.http"));
        assert!(result.is_ok());

        let requests = result.unwrap();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].method, HttpMethod::GET);
        assert_eq!(requests[1].method, HttpMethod::POST);
        assert_eq!(requests[2].method, HttpMethod::DELETE);
    }

    #[test]
    fn test_parse_file_with_comments() {
        let content = r#"
# This is a comment
// This is also a comment

GET https://api.example.com/users
# Another comment
"#;

        let result = parse_file(content, &PathBuf::from("test.http"));
        assert!(result.is_ok());

        let requests = result.unwrap();
        assert_eq!(requests.len(), 1);
    }

    #[test]
    fn test_parse_file_windows_line_endings() {
        let content = "GET https://api.example.com/users\r\n\r\n###\r\n\r\nPOST https://api.example.com/data\r\n";

        let result = parse_file(content, &PathBuf::from("test.http"));
        assert!(result.is_ok());

        let requests = result.unwrap();
        assert_eq!(requests.len(), 2);
    }

    #[test]
    fn test_parse_request_with_headers_and_body() {
        let lines = vec![
            (1, "POST https://api.example.com/users HTTP/1.1"),
            (2, "Content-Type: application/json"),
            (3, "Authorization: Bearer token"),
            (4, ""),
            (5, r#"{"name": "John"}"#),
        ];

        let result = parse_request(&lines, 1, &PathBuf::from("test.http"));
        assert!(result.is_ok());

        let request = result.unwrap();
        assert_eq!(request.method, HttpMethod::POST);
        assert_eq!(request.headers.len(), 2);
        assert!(request.body.is_some());
        assert!(request.body.unwrap().contains("John"));
    }

    #[test]
    fn test_parse_request_empty_block() {
        let lines: Vec<(usize, &str)> = vec![];

        let result = parse_request(&lines, 1, &PathBuf::from("test.http"));
        assert!(result.is_err());

        if let Err(ParseError::EmptyRequest { line }) = result {
            assert_eq!(line, 1);
        } else {
            panic!("Expected EmptyRequest error");
        }
    }

    #[test]
    fn test_parse_request_only_comments() {
        let lines = vec![(1, "# Comment line"), (2, "// Another comment")];

        let result = parse_request(&lines, 1, &PathBuf::from("test.http"));
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_request_id() {
        let id = generate_request_id(&PathBuf::from("/path/to/test.http"), 42);
        assert!(id.contains("test.http"));
        assert!(id.contains("42"));
    }

    #[test]
    fn test_all_http_methods() {
        let methods = vec![
            "GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD", "TRACE", "CONNECT",
        ];

        for method in methods {
            let line = format!("{} https://example.com", method);
            let result = parse_request_line(&line, 1);
            assert!(result.is_ok(), "Failed to parse method: {}", method);
        }
    }
}
