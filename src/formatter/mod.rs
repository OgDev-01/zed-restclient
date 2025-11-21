//! HTTP response formatter.
//!
//! This module provides functionality to format HTTP responses for display,
//! including content type detection, pretty-printing, and metadata extraction.

pub mod content_type;
pub mod graphql;
pub mod json;
pub mod syntax;
pub mod xml;

pub use content_type::{detect_content_type, ContentType};
pub use graphql::{format_graphql_query, format_graphql_request, format_graphql_response};
pub use json::{format_json_pretty, format_json_safe, minify_json, validate_json};
pub use syntax::{apply_syntax_highlighting, detect_language, HighlightInfo, Language};
pub use xml::{format_xml_pretty, format_xml_safe, minify_xml, validate_xml};

use crate::executor::timing::format_timing_breakdown;
use crate::models::response::HttpResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

/// Maximum response size to format (1MB).
///
/// Responses larger than this will be truncated with a warning message.
const MAX_RESPONSE_SIZE: usize = 1024 * 1024; // 1MB

/// Size of hex preview for binary content (1KB).
const HEX_PREVIEW_SIZE: usize = 1024;

/// Errors that can occur during response formatting.
#[derive(Debug)]
pub enum FormatError {
    /// JSON parsing or formatting error.
    JsonError(String),

    /// XML formatting error.
    XmlError(String),

    /// UTF-8 encoding error.
    EncodingError(String),

    /// Response too large to format.
    ResponseTooLarge(usize),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::JsonError(msg) => write!(f, "JSON formatting error: {}", msg),
            FormatError::XmlError(msg) => write!(f, "XML formatting error: {}", msg),
            FormatError::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            FormatError::ResponseTooLarge(size) => {
                write!(f, "Response too large to format: {} bytes", size)
            }
        }
    }
}

impl std::error::Error for FormatError {}

/// Response metadata for display.
///
/// Contains summary information about the HTTP response useful for
/// displaying alongside the formatted content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// HTTP status code.
    pub status_code: u16,

    /// HTTP status text.
    pub status_text: String,

    /// Total request duration.
    pub duration: Duration,

    /// Response size in bytes.
    pub size: usize,

    /// Content type classification.
    pub content_type: ContentType,

    /// Whether the response was successful (2xx status).
    pub is_success: bool,

    /// Whether the response was truncated due to size.
    pub is_truncated: bool,

    /// Timing breakdown for detailed performance metrics.
    pub timing_breakdown: String,
}

impl ResponseMetadata {
    /// Creates a new ResponseMetadata from an HttpResponse.
    ///
    /// # Arguments
    ///
    /// * `response` - The HTTP response
    /// * `content_type` - Detected content type
    /// * `is_truncated` - Whether the response was truncated
    pub fn from_response(
        response: &HttpResponse,
        content_type: ContentType,
        is_truncated: bool,
    ) -> Self {
        let timing_breakdown = format_timing_breakdown(&response.timing);

        Self {
            status_code: response.status_code,
            status_text: response.status_text.clone(),
            duration: response.duration,
            size: response.size,
            content_type,
            is_success: response.is_success(),
            is_truncated,
            timing_breakdown,
        }
    }

    /// Formats the duration in a human-readable format.
    ///
    /// # Returns
    ///
    /// String representation like "1.234s" or "567ms".
    pub fn format_duration(&self) -> String {
        let millis = self.duration.as_millis();
        if millis < 1000 {
            format!("{}ms", millis)
        } else {
            format!("{:.3}s", self.duration.as_secs_f64())
        }
    }

    /// Formats the size in a human-readable format.
    ///
    /// # Returns
    ///
    /// String representation like "1.23 KB" or "456 B".
    pub fn format_size(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.2} KB", self.size as f64 / 1024.0)
        } else {
            format!("{:.2} MB", self.size as f64 / (1024.0 * 1024.0))
        }
    }
}

/// Formatted HTTP response ready for display.
///
/// Contains the formatted body along with metadata and header information
/// formatted as text for easy viewing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedResponse {
    /// Detected content type.
    pub content_type: ContentType,

    /// Formatted response body.
    pub formatted_body: String,

    /// Raw (unformatted) response body.
    pub raw_body: String,

    /// Formatted status line (e.g., "HTTP/1.1 200 OK").
    pub status_line: String,

    /// Formatted headers as text.
    pub headers_text: String,

    /// Response metadata.
    pub metadata: ResponseMetadata,

    /// Syntax highlighting information.
    pub highlight_info: Option<HighlightInfo>,

    /// Whether the response is currently showing formatted or raw view.
    pub is_formatted: bool,
}

impl FormattedResponse {
    /// Creates a complete formatted response string for display.
    ///
    /// Combines status line, headers, metadata, and body into a
    /// single string suitable for display in an editor.
    ///
    /// # Returns
    ///
    /// Complete formatted response as a string.
    pub fn to_display_string(&self) -> String {
        let mut output = String::new();

        // Status line
        output.push_str(&self.status_line);
        output.push_str("\n\n");

        // Headers
        output.push_str("Headers:\n");
        output.push_str(&self.headers_text);
        output.push_str("\n");

        // Metadata
        output.push_str(&format!(
            "Duration: {} | Size: {} | Type: {}\n",
            self.metadata.format_duration(),
            self.metadata.format_size(),
            self.content_type.as_str()
        ));

        // Timing breakdown
        output.push_str(&format!("Timing: {}\n", self.metadata.timing_breakdown));

        if self.metadata.is_truncated {
            output.push_str("⚠️  Response truncated (exceeds 1MB limit)\n");
        }

        output.push_str("\n---\n\n");

        // Body
        output.push_str(&self.formatted_body);

        output
    }

    /// Toggles between formatted and raw view.
    ///
    /// Switches the formatted_body between the pretty-printed version
    /// and the raw unformatted version.
    pub fn toggle_view(&mut self) {
        if self.is_formatted {
            // Switch to raw view
            self.formatted_body = self.raw_body.clone();
            self.is_formatted = false;
        } else {
            // Switch back to formatted view by reformatting
            self.formatted_body = match self.content_type {
                ContentType::Json => {
                    format_json_pretty(&self.raw_body).unwrap_or_else(|_| self.raw_body.clone())
                }
                ContentType::Xml => {
                    format_xml_pretty(&self.raw_body).unwrap_or_else(|_| self.raw_body.clone())
                }
                _ => self.raw_body.clone(),
            };
            self.is_formatted = true;
        }
    }

    /// Gets the current body (formatted or raw based on current view).
    pub fn get_body(&self) -> &str {
        &self.formatted_body
    }

    /// Gets the raw unformatted body.
    pub fn get_raw_body(&self) -> &str {
        &self.raw_body
    }

    /// Gets the formatted (pretty-printed) body.
    ///
    /// This will format the raw body even if currently in raw view.
    pub fn get_formatted_body(&self) -> String {
        if self.is_formatted {
            self.formatted_body.clone()
        } else {
            match self.content_type {
                ContentType::Json => {
                    format_json_pretty(&self.raw_body).unwrap_or_else(|_| self.raw_body.clone())
                }
                ContentType::Xml => {
                    format_xml_pretty(&self.raw_body).unwrap_or_else(|_| self.raw_body.clone())
                }
                _ => self.raw_body.clone(),
            }
        }
    }
}

/// Formats an HTTP response for display.
///
/// Detects the content type, applies appropriate formatting, and packages
/// everything into a FormattedResponse ready for display.
///
/// # Arguments
///
/// * `response` - The HTTP response to format
///
/// # Returns
///
/// A `FormattedResponse` containing the formatted content and metadata.
///
/// # Examples
///
/// ```no_run
/// use rest_client::formatter::format_response;
/// use rest_client::models::response::HttpResponse;
///
/// let response = HttpResponse::new(200, "OK".to_string());
/// let formatted = format_response(&response);
/// println!("{}", formatted.to_display_string());
/// ```
pub fn format_response(response: &HttpResponse) -> FormattedResponse {
    // Detect content type
    let content_type = detect_content_type(&response.headers, &response.body);

    // Check if response is too large (use 10MB limit for enhanced formatters)
    let max_size = 10 * 1024 * 1024; // 10MB for enhanced formatters
    let is_truncated = response.body.len() > max_size;
    let body_to_format = if is_truncated {
        &response.body[..max_size]
    } else {
        &response.body
    };

    // Store raw body for toggle feature
    let raw_body = if let Ok(text) = std::str::from_utf8(body_to_format) {
        text.to_string()
    } else {
        format!("[Binary data: {} bytes]", body_to_format.len())
    };

    // Check if this is a GraphQL response (JSON with "data" or "errors" fields)
    let is_graphql_response = if content_type == ContentType::Json {
        if let Ok(text) = std::str::from_utf8(body_to_format) {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(text) {
                json_value.get("data").is_some() || json_value.get("errors").is_some()
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    // Format the body based on content type using enhanced formatters
    let (formatted_body, highlight_info) = match content_type {
        ContentType::Json => {
            if let Ok(text) = std::str::from_utf8(body_to_format) {
                // Check if this is a GraphQL response and format accordingly
                if is_graphql_response {
                    if let Ok(graphql_resp) =
                        serde_json::from_str::<crate::graphql::GraphQLResponse>(text)
                    {
                        let formatted = format_graphql_response(&graphql_resp);
                        let info = HighlightInfo::new(Language::Json);
                        (formatted, Some(info))
                    } else {
                        // Fallback to regular JSON formatting if GraphQL parsing fails
                        let formatted =
                            format_json_pretty(text).unwrap_or_else(|_| text.to_string());
                        let info = HighlightInfo::new(Language::Json);
                        (formatted, Some(info))
                    }
                } else {
                    // Use enhanced JSON formatter with syntax highlighting
                    let formatted = format_json_pretty(text).unwrap_or_else(|_| text.to_string());
                    let info = HighlightInfo::new(Language::Json);
                    (formatted, Some(info))
                }
            } else {
                (
                    format!("[Error: Invalid UTF-8 encoding in JSON response]"),
                    None,
                )
            }
        }
        ContentType::Xml => {
            if let Ok(text) = std::str::from_utf8(body_to_format) {
                // Use enhanced XML formatter with syntax highlighting
                let formatted = format_xml_pretty(text).unwrap_or_else(|_| text.to_string());
                let info = HighlightInfo::new(Language::Xml);
                (formatted, Some(info))
            } else {
                (
                    format!("[Error: Invalid UTF-8 encoding in XML response]"),
                    None,
                )
            }
        }
        ContentType::Html => {
            if let Ok(text) = std::str::from_utf8(body_to_format) {
                let info = HighlightInfo::new(Language::Html);
                (text.to_string(), Some(info))
            } else {
                (
                    format!("[Error: Invalid UTF-8 encoding in HTML response]"),
                    None,
                )
            }
        }
        ContentType::PlainText => {
            if let Ok(text) = std::str::from_utf8(body_to_format) {
                (text.to_string(), None)
            } else {
                (
                    format!("[Error: Invalid UTF-8 encoding in text response]"),
                    None,
                )
            }
        }
        ContentType::Binary => (format_binary_preview(body_to_format), None),
        ContentType::Image => (format_image_info(body_to_format, response.size), None),
    };

    // Format status line
    let status_line = format!("HTTP/1.1 {} {}", response.status_code, response.status_text);

    // Format headers
    let headers_text = format_headers(&response.headers);

    // Create metadata
    let metadata = ResponseMetadata::from_response(response, content_type, is_truncated);

    FormattedResponse {
        content_type,
        formatted_body,
        raw_body,
        status_line,
        headers_text,
        metadata,
        highlight_info,
        is_formatted: true,
    }
}

/// Formats JSON with pretty-printing.
///
/// **Deprecated**: Use `format_json_pretty` from the `json` module instead.
///
/// Parses the JSON and reformats it with indentation for readability.
///
/// # Arguments
///
/// * `json` - JSON string to format
///
/// # Returns
///
/// `Ok(String)` with formatted JSON, or `Err(FormatError)` if parsing fails.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::format_json;
///
/// let json = r#"{"key":"value","nested":{"array":[1,2,3]}}"#;
/// let formatted = format_json(json).unwrap();
/// assert!(formatted.contains("  "));
/// ```
#[deprecated(since = "0.2.0", note = "Use format_json_pretty from json module")]
pub fn format_json(json: &str) -> Result<String, FormatError> {
    // Delegate to enhanced JSON formatter
    format_json_pretty(json)
}

/// Formats XML with pretty-printing.
///
/// **Deprecated**: Use `format_xml_pretty` from the `xml` module instead.
///
/// Provides XML formatting with proper indentation.
///
/// # Arguments
///
/// * `xml` - XML string to format
///
/// # Returns
///
/// `Ok(String)` with formatted XML, or `Err(FormatError)` if formatting fails.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::format_xml;
///
/// let xml = "<root><child>text</child></root>";
/// let formatted = format_xml(xml).unwrap();
/// assert!(formatted.contains("  "));
/// ```
#[deprecated(since = "0.2.0", note = "Use format_xml_pretty from xml module")]
pub fn format_xml(xml: &str) -> Result<String, FormatError> {
    // Delegate to enhanced XML formatter
    format_xml_pretty(xml)
}

/// Formats headers as human-readable text.
///
/// # Arguments
///
/// * `headers` - HTTP headers map
///
/// # Returns
///
/// Formatted headers string with each header on a new line.
fn format_headers(headers: &HashMap<String, String>) -> String {
    if headers.is_empty() {
        return "(no headers)".to_string();
    }

    let mut header_lines: Vec<String> = headers
        .iter()
        .map(|(name, value)| format!("  {}: {}", name, value))
        .collect();

    header_lines.sort();
    header_lines.join("\n")
}

/// Formats binary content as a hex preview.
///
/// Shows the first 1KB of binary data as hexadecimal bytes.
///
/// # Arguments
///
/// * `body` - Binary data bytes
///
/// # Returns
///
/// Formatted hex preview string.
fn format_binary_preview(body: &[u8]) -> String {
    let preview_size = body.len().min(HEX_PREVIEW_SIZE);
    let preview_bytes = &body[..preview_size];

    let mut output = String::new();
    output.push_str("[Binary Data - Hex Preview]\n\n");

    for (i, chunk) in preview_bytes.chunks(16).enumerate() {
        // Offset
        output.push_str(&format!("{:08x}  ", i * 16));

        // Hex bytes
        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                output.push(' ');
            }
            output.push_str(&format!("{:02x} ", byte));
        }

        // Padding for incomplete lines
        for j in chunk.len()..16 {
            if j == 8 {
                output.push(' ');
            }
            output.push_str("   ");
        }

        // ASCII representation
        output.push_str(" |");
        for byte in chunk {
            let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            };
            output.push(ch);
        }
        output.push_str("|\n");
    }

    if body.len() > HEX_PREVIEW_SIZE {
        output.push_str(&format!(
            "\n... ({} more bytes not shown)\n",
            body.len() - HEX_PREVIEW_SIZE
        ));
    }

    output
}

/// Formats image information.
///
/// Displays metadata about image content without showing the binary data.
///
/// # Arguments
///
/// * `body` - Image data bytes
/// * `total_size` - Total size of the image
///
/// # Returns
///
/// Formatted image information string.
fn format_image_info(body: &[u8], total_size: usize) -> String {
    let image_type = if body.len() >= 4 && body[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        "PNG"
    } else if body.len() >= 3 && body[0..3] == [0xFF, 0xD8, 0xFF] {
        "JPEG"
    } else if body.len() >= 4 && body[0..4] == [0x47, 0x49, 0x46, 0x38] {
        "GIF"
    } else if body.len() >= 2 && body[0..2] == [0x42, 0x4D] {
        "BMP"
    } else if body.len() >= 12
        && body[0..4] == [0x52, 0x49, 0x46, 0x46]
        && body[8..12] == [0x57, 0x45, 0x42, 0x50]
    {
        "WebP"
    } else {
        "Unknown"
    };

    format!(
        "[Image Data]\n\nType: {}\nSize: {} bytes\n\n(Binary image data not displayed)",
        image_type, total_size
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::response::HttpResponse;

    #[test]
    fn test_format_json_valid() {
        let json = r#"{"key":"value","nested":{"array":[1,2,3]}}"#;
        let formatted = format_json(json).unwrap();

        assert!(formatted.contains("  "));
        assert!(formatted.contains("\"key\""));
        assert!(formatted.contains("\"value\""));
    }

    #[test]
    fn test_format_json_invalid() {
        let json = r#"{"key": invalid}"#;
        let result = format_json(json);

        assert!(result.is_err());
    }

    #[test]
    fn test_format_xml_basic() {
        let xml = "<root><child>text</child></root>";
        let formatted = format_xml(xml).unwrap();

        assert!(formatted.contains("<root>"));
        assert!(formatted.contains("  <child>"));
    }

    #[test]
    fn test_format_xml_with_attributes() {
        let xml = r#"<root attr="value"><child>text</child></root>"#;
        let formatted = format_xml(xml).unwrap();

        assert!(formatted.contains("attr=\"value\""));
    }

    #[test]
    fn test_format_headers() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Content-Length".to_string(), "123".to_string());

        let formatted = format_headers(&headers);

        assert!(formatted.contains("Content-Type: application/json"));
        assert!(formatted.contains("Content-Length: 123"));
    }

    #[test]
    fn test_format_headers_empty() {
        let headers = HashMap::new();
        let formatted = format_headers(&headers);

        assert_eq!(formatted, "(no headers)");
    }

    #[test]
    fn test_format_binary_preview() {
        let binary = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC];
        let formatted = format_binary_preview(&binary);

        assert!(formatted.contains("Binary Data"));
        assert!(formatted.contains("00 01 02 03"));
        assert!(formatted.contains("ff fe fd fc"));
    }

    #[test]
    fn test_format_image_info_png() {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let formatted = format_image_info(&png_header, 1024);

        assert!(formatted.contains("PNG"));
        assert!(formatted.contains("1024 bytes"));
    }

    #[test]
    fn test_format_response_json() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "application/json".to_string());
        response.set_body(br#"{"key":"value"}"#.to_vec());

        let formatted = format_response(&response);

        assert_eq!(formatted.content_type, ContentType::Json);
        assert!(formatted.formatted_body.contains("\"key\""));
        assert!(formatted.status_line.contains("200 OK"));
    }

    #[test]
    fn test_format_response_xml() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "application/xml".to_string());
        response.set_body(b"<root><child>text</child></root>".to_vec());

        let formatted = format_response(&response);

        assert_eq!(formatted.content_type, ContentType::Xml);
        assert!(formatted.formatted_body.contains("<root>"));
    }

    #[test]
    fn test_format_response_plain_text() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "text/plain".to_string());
        response.set_body(b"Hello, World!".to_vec());

        let formatted = format_response(&response);

        assert_eq!(formatted.content_type, ContentType::PlainText);
        assert_eq!(formatted.formatted_body, "Hello, World!");
    }

    #[test]
    fn test_format_response_binary() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header(
            "Content-Type".to_string(),
            "application/octet-stream".to_string(),
        );
        response.set_body(vec![0x00, 0x01, 0x02, 0xFF]);

        let formatted = format_response(&response);

        assert_eq!(formatted.content_type, ContentType::Binary);
        assert!(formatted.formatted_body.contains("Binary Data"));
    }

    #[test]
    fn test_format_response_large() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "text/plain".to_string());
        // Create response larger than 10MB (new limit for enhanced formatters)
        let max_size = 10 * 1024 * 1024;
        let large_body = vec![b'A'; max_size + 1000];
        response.set_body(large_body);

        let formatted = format_response(&response);

        assert!(formatted.metadata.is_truncated);
        assert_eq!(formatted.formatted_body.len(), max_size);
    }

    #[test]
    fn test_response_metadata_format_duration() {
        let response = HttpResponse::new(200, "OK".to_string());
        let metadata = ResponseMetadata::from_response(&response, ContentType::Json, false);

        // Duration should be formatted as milliseconds or seconds
        let duration_str = metadata.format_duration();
        assert!(duration_str.ends_with("ms") || duration_str.ends_with("s"));
    }

    #[test]
    fn test_response_metadata_format_size() {
        let response = HttpResponse::new(200, "OK".to_string());
        let metadata = ResponseMetadata::from_response(&response, ContentType::Json, false);

        // Size should be formatted with appropriate unit
        let size_str = metadata.format_size();
        assert!(size_str.ends_with(" B") || size_str.ends_with(" KB") || size_str.ends_with(" MB"));
    }

    #[test]
    fn test_formatted_response_to_display_string() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "application/json".to_string());
        response.set_body(br#"{"key":"value"}"#.to_vec());

        let formatted = format_response(&response);
        let display = formatted.to_display_string();

        assert!(display.contains("HTTP/1.1 200 OK"));
        assert!(display.contains("Headers:"));
        assert!(display.contains("Duration:"));
        assert!(display.contains("Size:"));
        assert!(display.contains("Timing:"));
        assert!(display.contains("DNS:"));
        assert!(display.contains("TCP:"));
        assert!(display.contains("---"));
    }

    #[test]
    fn test_formatted_response_timing_breakdown() {
        use std::time::Duration;

        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "text/plain".to_string());
        response.set_body(b"Hello".to_vec());

        // Set timing data
        response.timing.dns_lookup = Duration::from_millis(10);
        response.timing.tcp_connection = Duration::from_millis(20);
        response.timing.tls_handshake = Some(Duration::from_millis(50));
        response.timing.first_byte = Duration::from_millis(30);
        response.timing.download = Duration::from_millis(100);

        let formatted = format_response(&response);

        // Verify timing breakdown is present
        assert!(formatted.metadata.timing_breakdown.contains("DNS: 10ms"));
        assert!(formatted.metadata.timing_breakdown.contains("TCP: 20ms"));
        assert!(formatted.metadata.timing_breakdown.contains("TLS: 50ms"));
        assert!(formatted
            .metadata
            .timing_breakdown
            .contains("First Byte: 30ms"));
        assert!(formatted
            .metadata
            .timing_breakdown
            .contains("Download: 100ms"));

        // Verify it's in the display string
        let display = formatted.to_display_string();
        assert!(display.contains("Timing:"));
        assert!(display.contains("DNS: 10ms"));
    }

    #[test]
    fn test_format_error_display() {
        let json_err = FormatError::JsonError("invalid".to_string());
        assert_eq!(format!("{}", json_err), "JSON formatting error: invalid");

        let xml_err = FormatError::XmlError("malformed".to_string());
        assert_eq!(format!("{}", xml_err), "XML formatting error: malformed");

        let encoding_err = FormatError::EncodingError("not utf-8".to_string());
        assert_eq!(format!("{}", encoding_err), "Encoding error: not utf-8");

        let size_err = FormatError::ResponseTooLarge(2000000);
        assert_eq!(
            format!("{}", size_err),
            "Response too large to format: 2000000 bytes"
        );
    }
}
