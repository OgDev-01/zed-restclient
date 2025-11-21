//! Response Actions Module
//!
//! This module provides actions for interacting with HTTP responses including:
//! - Saving responses to files (full response or body only)
//! - Copying response data to clipboard (headers, body, or full response)
//! - Folding/unfolding large response sections
//! - Toggling between formatted and raw views
//!
//! # Architecture Note
//!
//! Due to Zed WASM extension API limitations (v0.7.0), these actions work by:
//! 1. Processing FormattedResponse data structures
//! 2. Returning formatted text output for display
//! 3. Simulating file save and clipboard operations through text instructions
//!
//! When Zed adds native file save/clipboard APIs to WASM extensions,
//! this module can be updated to use those directly.

use crate::formatter::{ContentType, FormattedResponse};
use crate::models::request::HttpRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Options for saving a response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SaveOption {
    /// Save the complete response (status, headers, and body)
    FullResponse,
    /// Save only the response body
    BodyOnly,
    /// Save only the headers
    HeadersOnly,
}

/// Options for copying response data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopyOption {
    /// Copy the complete response
    FullResponse,
    /// Copy only the response body
    Body,
    /// Copy only the headers
    Headers,
    /// Copy only the status line
    StatusLine,
}

/// Result of a save response action
#[derive(Debug, Clone)]
pub struct SaveResponseResult {
    /// Whether the save was successful
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// Suggested file path for the save
    pub suggested_path: PathBuf,
    /// Content that would be saved
    pub content: String,
    /// Size of the content in bytes
    pub content_size: usize,
}

/// Result of a copy response action
#[derive(Debug, Clone)]
pub struct CopyResponseResult {
    /// Whether the copy was successful
    pub success: bool,
    /// Message describing what was copied
    pub message: String,
    /// Content that was copied
    pub content: String,
    /// Size of the copied content in bytes
    pub content_size: usize,
}

/// Result of a fold response action
#[derive(Debug, Clone)]
pub struct FoldResponseResult {
    /// The response with folding applied
    pub folded_response: String,
    /// Number of sections that were folded
    pub sections_folded: usize,
    /// Whether folding was applied
    pub is_folded: bool,
}

/// Generate a suggested filename for saving a response
///
/// Creates a filename based on the HTTP method and URL domain/path.
///
/// # Arguments
///
/// * `request` - The HTTP request that generated the response
/// * `content_type` - The content type of the response
///
/// # Returns
///
/// A suggested filename (e.g., "get-users-response.json")
///
/// # Example
///
/// ```
/// use rest_client::ui::response_actions::suggest_filename;
/// use rest_client::models::request::{HttpRequest, HttpMethod};
/// use rest_client::formatter::ContentType;
/// use std::collections::HashMap;
/// use std::path::PathBuf;
///
/// let request = HttpRequest {
///     id: "test".to_string(),
///     method: HttpMethod::GET,
///     url: "https://api.example.com/users".to_string(),
///     http_version: Some("HTTP/1.1".to_string()),
///     headers: HashMap::new(),
///     body: None,
///     line_number: 0,
///     file_path: PathBuf::from("test.http"),
/// };
///
/// let filename = suggest_filename(&request, &ContentType::Json);
/// assert!(filename.to_string_lossy().contains("get"));
/// assert!(filename.to_string_lossy().contains(".json"));
/// ```
pub fn suggest_filename(request: &HttpRequest, content_type: &ContentType) -> PathBuf {
    // Extract domain and path from URL
    let url_parts: Vec<&str> = request.url.split('/').collect();

    // Get the last meaningful part of the URL path
    let path_part = url_parts
        .iter()
        .rev()
        .find(|part| !part.is_empty() && **part != "https:" && **part != "http:")
        .unwrap_or(&"response");

    // Clean up the path part (remove query params, special chars)
    let clean_path = path_part
        .split('?')
        .next()
        .unwrap_or(path_part)
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>();

    // Get method name in lowercase
    let method = request.method.to_string().to_lowercase();

    // Determine file extension based on content type
    let extension = match content_type {
        ContentType::Json => "json",
        ContentType::Xml => "xml",
        ContentType::Html => "html",
        ContentType::PlainText => "txt",
        ContentType::Image => "png",
        ContentType::Binary => "bin",
    };

    // Construct filename: method-path-response.extension
    let filename = if clean_path.is_empty() || clean_path == "response" {
        format!("{}-response.{}", method, extension)
    } else {
        format!("{}-{}-response.{}", method, clean_path, extension)
    };

    PathBuf::from(filename)
}

/// Save a response to a file
///
/// Prepares response content for saving based on the specified option.
///
/// # Arguments
///
/// * `response` - The formatted response to save
/// * `request` - The original request (for filename suggestion)
/// * `option` - What part of the response to save
///
/// # Returns
///
/// A `SaveResponseResult` with the content and metadata
///
/// # Example
///
/// ```ignore
/// use rest_client::ui::response_actions::{save_response, SaveOption};
/// use rest_client::formatter::FormattedResponse;
/// use rest_client::models::request::HttpRequest;
///
/// let result = save_response(&response, &request, SaveOption::BodyOnly);
/// println!("Suggested path: {:?}", result.suggested_path);
/// println!("Content size: {} bytes", result.content_size);
/// ```
pub fn save_response(
    response: &FormattedResponse,
    request: &HttpRequest,
    option: SaveOption,
) -> SaveResponseResult {
    let content = match option {
        SaveOption::FullResponse => {
            // Combine status, headers, and body
            format!(
                "{}\n\n{}\n\n{}",
                response.status_line,
                response.headers_text,
                if response.is_formatted {
                    &response.formatted_body
                } else {
                    &response.raw_body
                }
            )
        }
        SaveOption::BodyOnly => {
            // Just the body (formatted or raw based on current view)
            if response.is_formatted {
                response.formatted_body.clone()
            } else {
                response.raw_body.clone()
            }
        }
        SaveOption::HeadersOnly => {
            // Status line and headers
            format!("{}\n\n{}", response.status_line, response.headers_text)
        }
    };

    let content_size = content.len();
    let suggested_path = suggest_filename(request, &response.content_type);

    SaveResponseResult {
        success: true,
        message: format!(
            "Ready to save {} ({} bytes) to {:?}",
            match option {
                SaveOption::FullResponse => "full response",
                SaveOption::BodyOnly => "response body",
                SaveOption::HeadersOnly => "headers",
            },
            content_size,
            suggested_path
        ),
        suggested_path,
        content,
        content_size,
    }
}

/// Copy response data to clipboard
///
/// Prepares response content for copying based on the specified option.
///
/// # Arguments
///
/// * `response` - The formatted response to copy from
/// * `option` - What part of the response to copy
///
/// # Returns
///
/// A `CopyResponseResult` with the content and metadata
///
/// # Example
///
/// ```ignore
/// use rest_client::ui::response_actions::{copy_response, CopyOption};
/// use rest_client::formatter::FormattedResponse;
///
/// let result = copy_response(&response, CopyOption::Body);
/// println!("Copied: {}", result.message);
/// ```
pub fn copy_response(response: &FormattedResponse, option: CopyOption) -> CopyResponseResult {
    let content = match option {
        CopyOption::FullResponse => {
            format!(
                "{}\n\n{}\n\n{}",
                response.status_line,
                response.headers_text,
                if response.is_formatted {
                    &response.formatted_body
                } else {
                    &response.raw_body
                }
            )
        }
        CopyOption::Body => {
            if response.is_formatted {
                response.formatted_body.clone()
            } else {
                response.raw_body.clone()
            }
        }
        CopyOption::Headers => response.headers_text.clone(),
        CopyOption::StatusLine => response.status_line.clone(),
    };

    let content_size = content.len();

    CopyResponseResult {
        success: true,
        message: format!(
            "Copied {} ({} bytes) to clipboard",
            match option {
                CopyOption::FullResponse => "full response",
                CopyOption::Body => "response body",
                CopyOption::Headers => "headers",
                CopyOption::StatusLine => "status line",
            },
            content_size
        ),
        content,
        content_size,
    }
}

/// Fold large sections in a JSON response body
///
/// Collapses large JSON arrays and objects to make responses more manageable.
///
/// # Arguments
///
/// * `json_body` - The JSON response body to fold
/// * `fold_threshold` - Minimum number of lines before folding (default: 10)
///
/// # Returns
///
/// The folded JSON body with large sections collapsed
fn fold_json_sections(json_body: &str, fold_threshold: usize) -> (String, usize) {
    let lines: Vec<&str> = json_body.lines().collect();

    if lines.len() <= fold_threshold {
        return (json_body.to_string(), 0);
    }

    let mut folded_lines = Vec::new();
    let mut in_foldable_section = false;
    let mut section_start = 0;
    let mut brace_depth = 0;
    let mut sections_folded = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Track brace depth
        for ch in trimmed.chars() {
            match ch {
                '{' | '[' => brace_depth += 1,
                '}' | ']' => brace_depth -= 1,
                _ => {}
            }
        }

        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if !in_foldable_section && brace_depth > 1 {
                in_foldable_section = true;
                section_start = i;
            }
        }

        if in_foldable_section {
            if (trimmed.ends_with('}') || trimmed.ends_with(']')) && brace_depth <= 1 {
                // End of section
                let section_length = i - section_start + 1;

                if section_length >= fold_threshold {
                    // Fold this section
                    let first_line = lines[section_start].trim();

                    if first_line.starts_with('{') {
                        folded_lines
                            .push(format!("  {{ ... {} lines folded ... }}", section_length));
                    } else {
                        folded_lines.push(format!("  [ ... {} items folded ... ]", section_length));
                    }

                    sections_folded += 1;
                } else {
                    // Don't fold, add all lines
                    for j in section_start..=i {
                        folded_lines.push(lines[j].to_string());
                    }
                }

                in_foldable_section = false;
            }
        } else if brace_depth <= 1 {
            folded_lines.push(line.to_string());
        }
    }

    (folded_lines.join("\n"), sections_folded)
}

/// Fold large sections in an XML response body
///
/// Collapses large XML nodes to make responses more manageable.
///
/// # Arguments
///
/// * `xml_body` - The XML response body to fold
/// * `fold_threshold` - Minimum number of lines before folding (default: 10)
///
/// # Returns
///
/// The folded XML body with large sections collapsed
fn fold_xml_sections(xml_body: &str, fold_threshold: usize) -> (String, usize) {
    let lines: Vec<&str> = xml_body.lines().collect();

    if lines.len() <= fold_threshold {
        return (xml_body.to_string(), 0);
    }

    let mut folded_lines = Vec::new();
    let mut sections_folded = 0;
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check if this is an opening tag (not self-closing)
        if line.starts_with('<')
            && !line.starts_with("</")
            && !line.ends_with("/>")
            && !line.starts_with("<?")
        {
            // Extract tag name
            let tag_name = line
                .trim_start_matches('<')
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('>');

            if !tag_name.is_empty() {
                // Find the closing tag
                let closing_tag = format!("</{}>", tag_name);
                let mut j = i + 1;

                while j < lines.len() && !lines[j].trim().contains(&closing_tag) {
                    j += 1;
                }

                if j < lines.len() {
                    let section_length = j - i + 1;

                    if section_length >= fold_threshold {
                        // Fold this section
                        folded_lines.push(format!(
                            "  <{}><!-- {} lines folded --></{}>",
                            tag_name, section_length, tag_name
                        ));
                        sections_folded += 1;
                        i = j + 1;
                        continue;
                    }
                }
            }
        }

        folded_lines.push(lines[i].to_string());
        i += 1;
    }

    (folded_lines.join("\n"), sections_folded)
}

/// Fold large sections in a response body
///
/// Collapses large sections of JSON or XML responses to make them more manageable.
/// For other content types, returns the original response.
///
/// # Arguments
///
/// * `response` - The formatted response to fold
/// * `fold_threshold` - Minimum number of lines before folding (default: 10)
///
/// # Returns
///
/// A `FoldResponseResult` with the folded content
///
/// # Example
///
/// ```ignore
/// use rest_client::ui::response_actions::fold_response;
/// use rest_client::formatter::FormattedResponse;
///
/// let result = fold_response(&response, 10);
/// println!("Folded {} sections", result.sections_folded);
/// ```
pub fn fold_response(response: &FormattedResponse, fold_threshold: usize) -> FoldResponseResult {
    let body = if response.is_formatted {
        &response.formatted_body
    } else {
        &response.raw_body
    };

    let (folded_body, sections_folded) = match response.content_type {
        ContentType::Json => fold_json_sections(body, fold_threshold),
        ContentType::Xml => fold_xml_sections(body, fold_threshold),
        _ => {
            // For other content types, don't fold
            (body.to_string(), 0)
        }
    };

    FoldResponseResult {
        folded_response: format!(
            "{}\n\n{}\n\n{}",
            response.status_line, response.headers_text, folded_body
        ),
        sections_folded,
        is_folded: sections_folded > 0,
    }
}

/// Toggle between formatted and raw view of a response
///
/// Switches the response display between formatted (pretty-printed) and raw (exact bytes).
///
/// # Arguments
///
/// * `response` - The formatted response to toggle
///
/// # Returns
///
/// A new `FormattedResponse` with the view toggled
///
/// # Example
///
/// ```ignore
/// use rest_client::ui::response_actions::toggle_raw_view;
/// use rest_client::formatter::FormattedResponse;
///
/// let toggled = toggle_raw_view(&response);
/// assert_eq!(toggled.is_formatted, !response.is_formatted);
/// ```
pub fn toggle_raw_view(response: &FormattedResponse) -> FormattedResponse {
    let mut toggled = response.clone();
    toggled.is_formatted = !toggled.is_formatted;
    toggled
}

/// Create a formatted display of response action options
///
/// Generates a user-friendly menu showing available actions for a response.
///
/// # Arguments
///
/// * `response` - The response to show actions for
///
/// # Returns
///
/// A formatted string listing available actions
pub fn format_action_menu(response: &FormattedResponse) -> String {
    let mut menu = String::new();

    menu.push_str("╭─────────────────────────────────────────────────────────╮\n");
    menu.push_str("│               Response Actions Available               │\n");
    menu.push_str("├─────────────────────────────────────────────────────────┤\n");

    // Save actions
    menu.push_str("│ 💾 Save Response:                                       │\n");
    menu.push_str("│    • Full Response (status + headers + body)           │\n");
    menu.push_str("│    • Body Only                                          │\n");
    menu.push_str("│    • Headers Only                                       │\n");
    menu.push_str("├─────────────────────────────────────────────────────────┤\n");

    // Copy actions
    menu.push_str("│ 📋 Copy to Clipboard:                                   │\n");
    menu.push_str("│    • Full Response                                      │\n");
    menu.push_str("│    • Body Only                                          │\n");
    menu.push_str("│    • Headers Only                                       │\n");
    menu.push_str("│    • Status Line Only                                   │\n");
    menu.push_str("├─────────────────────────────────────────────────────────┤\n");

    // View toggles
    menu.push_str(&format!(
        "│ 🔄 View Mode: {:<42}│\n",
        if response.is_formatted {
            "Formatted (toggle to raw)"
        } else {
            "Raw (toggle to formatted)"
        }
    ));

    // Folding (only for JSON/XML)
    if matches!(response.content_type, ContentType::Json | ContentType::Xml) {
        menu.push_str("│ 📁 Fold/Unfold: Available for large sections           │\n");
    }

    menu.push_str("╰─────────────────────────────────────────────────────────╯\n");

    menu
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::{ContentType, ResponseMetadata};
    use crate::models::request::{HttpMethod, HttpRequest};
    use std::collections::HashMap;
    use std::time::Duration;

    fn create_test_request(method: HttpMethod, url: &str) -> HttpRequest {
        HttpRequest {
            id: "test-123".to_string(),
            method,
            url: url.to_string(),
            http_version: Some("HTTP/1.1".to_string()),
            headers: HashMap::new(),
            body: None,
            line_number: 0,
            file_path: PathBuf::from("test.http"),
        }
    }

    fn create_test_response(content_type: ContentType, body: &str) -> FormattedResponse {
        FormattedResponse {
            content_type: content_type.clone(),
            formatted_body: body.to_string(),
            raw_body: body.to_string(),
            status_line: "HTTP/1.1 200 OK".to_string(),
            headers_text: "Content-Type: application/json\nContent-Length: 100\n".to_string(),
            metadata: ResponseMetadata {
                status_code: 200,
                status_text: "OK".to_string(),
                duration: Duration::from_millis(150),
                size: body.len(),
                content_type,
                is_success: true,
                is_truncated: false,
                timing_breakdown: "Total: 150ms".to_string(),
            },
            highlight_info: None,
            is_formatted: true,
        }
    }

    #[test]
    fn test_suggest_filename_json() {
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let filename = suggest_filename(&request, &ContentType::Json);

        assert_eq!(filename, PathBuf::from("get-users-response.json"));
    }

    #[test]
    fn test_suggest_filename_with_query_params() {
        let request = create_test_request(
            HttpMethod::POST,
            "https://api.example.com/posts?id=123&filter=active",
        );
        let filename = suggest_filename(&request, &ContentType::Json);

        assert_eq!(filename, PathBuf::from("post-posts-response.json"));
    }

    #[test]
    fn test_suggest_filename_xml() {
        let request = create_test_request(HttpMethod::PUT, "https://api.example.com/items/42");
        let filename = suggest_filename(&request, &ContentType::Xml);

        assert_eq!(filename, PathBuf::from("put-42-response.xml"));
    }

    #[test]
    fn test_save_response_full() {
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/data");
        let response = create_test_response(ContentType::Json, r#"{"key": "value"}"#);

        let result = save_response(&response, &request, SaveOption::FullResponse);

        assert!(result.success);
        assert!(result.content.contains("HTTP/1.1 200 OK"));
        assert!(result.content.contains("Content-Type: application/json"));
        assert!(result.content.contains(r#"{"key": "value"}"#));
        assert_eq!(
            result.suggested_path,
            PathBuf::from("get-data-response.json")
        );
    }

    #[test]
    fn test_save_response_body_only() {
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/data");
        let response = create_test_response(ContentType::Json, r#"{"key": "value"}"#);

        let result = save_response(&response, &request, SaveOption::BodyOnly);

        assert!(result.success);
        assert_eq!(result.content, r#"{"key": "value"}"#);
        assert!(!result.content.contains("HTTP/1.1 200 OK"));
    }

    #[test]
    fn test_save_response_headers_only() {
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/data");
        let response = create_test_response(ContentType::Json, r#"{"key": "value"}"#);

        let result = save_response(&response, &request, SaveOption::HeadersOnly);

        assert!(result.success);
        assert!(result.content.contains("HTTP/1.1 200 OK"));
        assert!(result.content.contains("Content-Type: application/json"));
        assert!(!result.content.contains(r#"{"key": "value"}"#));
    }

    #[test]
    fn test_copy_response_body() {
        let response = create_test_response(ContentType::Json, r#"{"test": "data"}"#);

        let result = copy_response(&response, CopyOption::Body);

        assert!(result.success);
        assert_eq!(result.content, r#"{"test": "data"}"#);
        assert!(result.message.contains("response body"));
    }

    #[test]
    fn test_copy_response_headers() {
        let response = create_test_response(ContentType::Json, r#"{"test": "data"}"#);

        let result = copy_response(&response, CopyOption::Headers);

        assert!(result.success);
        assert!(result.content.contains("Content-Type: application/json"));
        assert!(!result.content.contains("test"));
    }

    #[test]
    fn test_copy_response_status_line() {
        let response = create_test_response(ContentType::Json, r#"{"test": "data"}"#);

        let result = copy_response(&response, CopyOption::StatusLine);

        assert!(result.success);
        assert_eq!(result.content, "HTTP/1.1 200 OK");
    }

    #[test]
    fn test_toggle_raw_view() {
        let response = create_test_response(ContentType::Json, r#"{"test": "data"}"#);
        assert!(response.is_formatted);

        let toggled = toggle_raw_view(&response);
        assert!(!toggled.is_formatted);

        let toggled_back = toggle_raw_view(&toggled);
        assert!(toggled_back.is_formatted);
    }

    #[test]
    fn test_fold_json_sections() {
        let json = r#"{
  "users": [
    {"id": 1, "name": "Alice"},
    {"id": 2, "name": "Bob"},
    {"id": 3, "name": "Charlie"},
    {"id": 4, "name": "David"},
    {"id": 5, "name": "Eve"}
  ],
  "total": 5
}"#;

        let (folded, count) = fold_json_sections(json, 3);
        assert!(count > 0);
        assert!(folded.contains("folded"));
    }

    #[test]
    fn test_fold_response_short_content() {
        let response = create_test_response(ContentType::Json, r#"{"short": "content"}"#);

        let result = fold_response(&response, 10);

        assert_eq!(result.sections_folded, 0);
        assert!(!result.is_folded);
    }

    #[test]
    fn test_format_action_menu() {
        let response = create_test_response(ContentType::Json, r#"{"test": "data"}"#);

        let menu = format_action_menu(&response);

        assert!(menu.contains("Response Actions Available"));
        assert!(menu.contains("Save Response"));
        assert!(menu.contains("Copy to Clipboard"));
        assert!(menu.contains("View Mode"));
        assert!(menu.contains("Fold/Unfold"));
    }

    #[test]
    fn test_format_action_menu_no_folding_for_text() {
        let response = create_test_response(ContentType::PlainText, "plain text");

        let menu = format_action_menu(&response);

        assert!(menu.contains("Response Actions Available"));
        assert!(!menu.contains("Fold/Unfold"));
    }
}
