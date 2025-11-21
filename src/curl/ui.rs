//! UI and command integration for cURL import/export functionality.
//!
//! This module provides command functions for:
//! - Converting cURL commands to HTTP requests (paste/import)
//! - Converting HTTP requests to cURL commands (copy/export)
//!
//! These functions are designed to integrate with Zed's slash command system
//! and provide user-friendly feedback with preview, validation, and formatting.

use crate::curl::{generate_curl_command, parse_curl_command};
use crate::models::HttpRequest;

/// Result of a cURL paste operation
#[derive(Debug, Clone)]
pub struct PasteCurlResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// User-friendly message
    pub message: String,
    /// The parsed HTTP request (if successful)
    pub request: Option<HttpRequest>,
    /// Formatted HTTP request text ready to insert
    pub formatted_request: String,
    /// Preview of what will be pasted
    pub preview: String,
}

impl PasteCurlResult {
    /// Create a successful result
    pub fn success(request: HttpRequest, formatted: String) -> Self {
        let preview = if formatted.len() > 200 {
            format!("{}...", &formatted[..200])
        } else {
            formatted.clone()
        };

        Self {
            success: true,
            message: format!(
                "Successfully converted cURL command to {} request",
                request.method
            ),
            request: Some(request),
            formatted_request: formatted,
            preview,
        }
    }

    /// Create a failure result
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            request: None,
            formatted_request: String::new(),
            preview: String::new(),
        }
    }

    /// Convert to display string for editor output
    pub fn to_display_string(&self) -> String {
        if self.success {
            self.formatted_request.clone()
        } else {
            format!("Error: {}", self.message)
        }
    }
}

/// Result of a cURL copy operation
#[derive(Debug, Clone)]
pub struct CopyCurlResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// User-friendly message
    pub message: String,
    /// The generated cURL command (if successful)
    pub curl_command: String,
    /// Preview of the command (first ~50 chars)
    pub preview: String,
}

impl CopyCurlResult {
    /// Create a successful result
    pub fn success(curl_command: String) -> Self {
        let preview = if curl_command.len() > 50 {
            format!("{}...", &curl_command[..50].trim())
        } else {
            curl_command.clone()
        };

        Self {
            success: true,
            message: format!("Generated cURL command ({} chars)", curl_command.len()),
            curl_command: curl_command.clone(),
            preview,
        }
    }

    /// Create a failure result
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            curl_command: String::new(),
            preview: String::new(),
        }
    }

    /// Convert to display string for editor output
    pub fn to_display_string(&self) -> String {
        if self.success {
            format!(
                "# cURL Command Generated\n\n{}\n\n# Preview\n{}\n\nCopy the above command to use in your terminal.",
                self.curl_command,
                self.preview
            )
        } else {
            format!("Error: {}", self.message)
        }
    }
}

/// Parse and format a cURL command into an HTTP request
///
/// This function:
/// 1. Validates the input is a cURL command
/// 2. Parses it using the cURL parser
/// 3. Formats it as a clean HTTP request with comments
///
/// # Arguments
///
/// * `curl_text` - The cURL command text (should start with "curl")
///
/// # Returns
///
/// A `PasteCurlResult` containing the formatted request or error
pub fn paste_curl_command(curl_text: &str) -> PasteCurlResult {
    let trimmed = curl_text.trim();

    // Validate input
    if trimmed.is_empty() {
        return PasteCurlResult::failure("No content provided".to_string());
    }

    // Auto-detect if content is a cURL command
    if !trimmed.starts_with("curl") && !trimmed.contains("curl ") {
        return PasteCurlResult::failure(
            "Content does not appear to be a cURL command (must start with 'curl')".to_string(),
        );
    }

    // Parse the cURL command
    let request = match parse_curl_command(trimmed) {
        Ok(req) => req,
        Err(e) => {
            return PasteCurlResult::failure(format!("Failed to parse cURL command: {}", e));
        }
    };

    // Format as HTTP request with nice spacing and comments
    let formatted = format_request_from_curl(&request);

    PasteCurlResult::success(request, formatted)
}

/// Generate a cURL command from an HTTP request
///
/// This function:
/// 1. Takes a parsed HTTP request
/// 2. Generates a valid cURL command with proper escaping
/// 3. Provides a preview for user confirmation
///
/// # Arguments
///
/// * `request` - The HTTP request to convert
///
/// # Returns
///
/// A `CopyCurlResult` containing the cURL command or error
pub fn copy_as_curl_command(request: &HttpRequest) -> CopyCurlResult {
    // Validate request has minimum required fields
    if request.url.is_empty() {
        return CopyCurlResult::failure("Request has no URL".to_string());
    }

    // Generate the cURL command
    let curl_command = generate_curl_command(request);

    CopyCurlResult::success(curl_command)
}

/// Format an HTTP request nicely for insertion into a .http file
///
/// Adds:
/// - Source comment indicating it came from cURL
/// - Proper spacing between method/URL and headers
/// - Blank line before body
/// - Clean formatting
fn format_request_from_curl(request: &HttpRequest) -> String {
    let mut output = String::new();

    // Add source comment
    output.push_str("# Generated from cURL command\n");

    // Add method and URL
    output.push_str(&format!("{} {}\n", request.method, request.url));

    // Add headers with proper spacing
    if !request.headers.is_empty() {
        for (key, value) in &request.headers {
            output.push_str(&format!("{}: {}\n", key, value));
        }
    }

    // Add body if present
    if let Some(body) = &request.body {
        output.push('\n');
        output.push_str(body);
        if !body.ends_with('\n') {
            output.push('\n');
        }
    }

    output
}

/// Validate and preview a cURL command without parsing
///
/// Provides quick validation feedback to users
pub fn validate_curl_command(curl_text: &str) -> Result<String, String> {
    let trimmed = curl_text.trim();

    if trimmed.is_empty() {
        return Err("Empty content".to_string());
    }

    if !trimmed.starts_with("curl") && !trimmed.contains("curl ") {
        return Err("Not a cURL command (must start with 'curl')".to_string());
    }

    // Try to parse to validate
    match parse_curl_command(trimmed) {
        Ok(request) => {
            let preview = format!(
                "Valid cURL command:\n  Method: {}\n  URL: {}\n  Headers: {}\n  Has Body: {}",
                request.method,
                if request.url.len() > 60 {
                    format!("{}...", &request.url[..60])
                } else {
                    request.url.clone()
                },
                request.headers.len(),
                request.body.is_some()
            );
            Ok(preview)
        }
        Err(e) => Err(format!("Invalid cURL command: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paste_curl_simple_get() {
        let curl = "curl https://api.example.com/users";
        let result = paste_curl_command(curl);

        assert!(result.success);
        assert!(result.request.is_some());
        assert!(result
            .formatted_request
            .contains("GET https://api.example.com/users"));
        assert!(result.formatted_request.contains("# Generated from cURL"));
    }

    #[test]
    fn test_paste_curl_with_headers() {
        let curl = r#"curl -H "Authorization: Bearer token123" https://api.example.com/data"#;
        let result = paste_curl_command(curl);

        assert!(result.success);
        assert!(result
            .formatted_request
            .contains("Authorization: Bearer token123"));
    }

    #[test]
    fn test_paste_curl_with_post_data() {
        let curl = r#"curl -X POST -d '{"name":"John"}' https://api.example.com/users"#;
        let result = paste_curl_command(curl);

        assert!(result.success);
        assert!(result
            .formatted_request
            .contains("POST https://api.example.com/users"));
        assert!(result.formatted_request.contains(r#"{"name":"John"}"#));
    }

    #[test]
    fn test_paste_curl_empty_content() {
        let result = paste_curl_command("");
        assert!(!result.success);
        assert!(result.message.contains("No content"));
    }

    #[test]
    fn test_paste_curl_not_curl_command() {
        let result = paste_curl_command("GET https://example.com");
        assert!(!result.success);
        assert!(result
            .message
            .contains("does not appear to be a cURL command"));
    }

    #[test]
    fn test_paste_curl_invalid_syntax() {
        let curl = "curl -X";
        let result = paste_curl_command(curl);
        assert!(!result.success);
        assert!(result.message.contains("Failed to parse"));
    }

    #[test]
    fn test_copy_as_curl_simple_request() {
        let request = HttpRequest::new(
            "test-1".to_string(),
            crate::models::HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );

        let result = copy_as_curl_command(&request);
        assert!(result.success);
        assert!(result.curl_command.contains("curl"));
        assert!(result
            .curl_command
            .contains("https://api.example.com/users"));
    }

    #[test]
    fn test_copy_as_curl_with_headers() {
        let mut request = HttpRequest::new(
            "test-2".to_string(),
            crate::models::HttpMethod::POST,
            "https://api.example.com/data".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer token123".to_string());
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"key":"value"}"#.to_string());

        let result = copy_as_curl_command(&request);
        assert!(result.success);
        // The generator may omit -X POST when body is present, so just check for key elements
        assert!(result.curl_command.contains("curl"));
        assert!(result.curl_command.contains("https://api.example.com/data"));
        assert!(result.curl_command.contains("Authorization"));
        assert!(result.curl_command.contains("Bearer token123"));
    }

    #[test]
    fn test_copy_as_curl_no_url() {
        let request = HttpRequest::new(
            "test-3".to_string(),
            crate::models::HttpMethod::GET,
            String::new(),
        );

        let result = copy_as_curl_command(&request);
        assert!(!result.success);
        assert!(result.message.contains("no URL"));
    }

    #[test]
    fn test_copy_as_curl_preview_length() {
        let request = HttpRequest::new(
            "test-4".to_string(),
            crate::models::HttpMethod::GET,
            "https://api.example.com/very/long/path/that/exceeds/fifty/characters/easily"
                .to_string(),
        );

        let result = copy_as_curl_command(&request);
        assert!(result.success);
        assert!(result.preview.len() <= 60); // 50 + "..." + some tolerance
    }

    #[test]
    fn test_validate_curl_command_valid() {
        let curl = "curl -X GET https://example.com";
        let result = validate_curl_command(curl);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Valid cURL command"));
    }

    #[test]
    fn test_validate_curl_command_invalid() {
        let result = validate_curl_command("not a curl command");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_request_from_curl() {
        let mut request = HttpRequest::new(
            "test-5".to_string(),
            crate::models::HttpMethod::POST,
            "https://api.example.com/test".to_string(),
        );
        request.add_header("Accept".to_string(), "application/json".to_string());
        request.set_body("test body".to_string());

        let formatted = format_request_from_curl(&request);

        assert!(formatted.contains("# Generated from cURL"));
        assert!(formatted.contains("POST https://api.example.com/test"));
        assert!(formatted.contains("Accept: application/json"));
        assert!(formatted.contains("test body"));
    }

    #[test]
    fn test_paste_curl_multiline_command() {
        let curl = r#"curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}' \
  https://api.example.com/endpoint"#;

        let result = paste_curl_command(curl);
        assert!(result.success);
        assert!(result
            .formatted_request
            .contains("POST https://api.example.com/endpoint"));
    }

    #[test]
    fn test_result_display_strings() {
        let request = HttpRequest::new(
            "test-6".to_string(),
            crate::models::HttpMethod::GET,
            "https://example.com".to_string(),
        );

        let paste_result =
            PasteCurlResult::success(request.clone(), "GET https://example.com".to_string());
        assert!(paste_result
            .to_display_string()
            .contains("GET https://example.com"));

        let copy_result = copy_as_curl_command(&request);
        let display = copy_result.to_display_string();
        assert!(display.contains("cURL Command Generated"));
        assert!(display.contains("curl"));
    }
}
