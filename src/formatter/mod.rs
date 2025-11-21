//! HTTP response formatter.
//!
//! This module provides functionality to format HTTP responses for display,
//! including content type detection, pretty-printing, and metadata extraction.

pub mod content_type;

pub use content_type::{detect_content_type, ContentType};

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
        Self {
            status_code: response.status_code,
            status_text: response.status_text.clone(),
            duration: response.duration,
            size: response.size,
            content_type,
            is_success: response.is_success(),
            is_truncated,
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

    /// Formatted status line (e.g., "HTTP/1.1 200 OK").
    pub status_line: String,

    /// Formatted headers as text.
    pub headers_text: String,

    /// Response metadata.
    pub metadata: ResponseMetadata,
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

        if self.metadata.is_truncated {
            output.push_str("⚠️  Response truncated (exceeds 1MB limit)\n");
        }

        output.push_str("\n---\n\n");

        // Body
        output.push_str(&self.formatted_body);

        output
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

    // Check if response is too large
    let is_truncated = response.body.len() > MAX_RESPONSE_SIZE;
    let body_to_format = if is_truncated {
        &response.body[..MAX_RESPONSE_SIZE]
    } else {
        &response.body
    };

    // Format the body based on content type
    let formatted_body = match content_type {
        ContentType::Json => {
            if let Ok(text) = std::str::from_utf8(body_to_format) {
                format_json(text).unwrap_or_else(|_| text.to_string())
            } else {
                format!("[Error: Invalid UTF-8 encoding in JSON response]")
            }
        }
        ContentType::Xml => {
            if let Ok(text) = std::str::from_utf8(body_to_format) {
                format_xml(text).unwrap_or_else(|_| text.to_string())
            } else {
                format!("[Error: Invalid UTF-8 encoding in XML response]")
            }
        }
        ContentType::Html | ContentType::PlainText => {
            if let Ok(text) = std::str::from_utf8(body_to_format) {
                text.to_string()
            } else {
                format!("[Error: Invalid UTF-8 encoding in text response]")
            }
        }
        ContentType::Binary => format_binary_preview(body_to_format),
        ContentType::Image => format_image_info(body_to_format, response.size),
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
        status_line,
        headers_text,
        metadata,
    }
}

/// Formats JSON with pretty-printing.
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
pub fn format_json(json: &str) -> Result<String, FormatError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| FormatError::JsonError(e.to_string()))?;

    serde_json::to_string_pretty(&value).map_err(|e| FormatError::JsonError(e.to_string()))
}

/// Formats XML with basic indentation.
///
/// Provides simple XML formatting with indentation for MVP.
/// Full XML formatting with proper parsing will be added in Phase 3.
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
pub fn format_xml(xml: &str) -> Result<String, FormatError> {
    let mut formatted = String::new();
    let mut indent_level: usize = 0;
    let mut in_tag = false;
    let mut tag_start_pos = 0;
    let chars: Vec<char> = xml.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '<' {
            // Check if this is a closing tag
            let is_closing = i + 1 < chars.len() && chars[i + 1] == '/';

            if is_closing {
                indent_level = indent_level.saturating_sub(1);
            }

            // Add newline and indentation before tag (except for first tag)
            if i > 0 && !in_tag {
                formatted.push('\n');
                formatted.push_str(&"  ".repeat(indent_level));
            }

            // Add the opening bracket
            formatted.push('<');
            in_tag = true;
            tag_start_pos = i;
        } else if ch == '>' {
            formatted.push('>');
            in_tag = false;

            // Check if this was a self-closing tag or declaration
            let tag_content: String = chars[tag_start_pos..=i].iter().collect();
            let is_self_closing = tag_content.contains("/>") || tag_content.starts_with("<?");
            let is_closing = tag_content.starts_with("</");

            if !is_closing && !is_self_closing {
                indent_level += 1;
            }
        } else {
            formatted.push(ch);
        }

        i += 1;
    }

    Ok(formatted)
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
        // Create response larger than MAX_RESPONSE_SIZE
        let large_body = vec![b'A'; MAX_RESPONSE_SIZE + 1000];
        response.set_body(large_body);

        let formatted = format_response(&response);

        assert!(formatted.metadata.is_truncated);
        assert_eq!(formatted.formatted_body.len(), MAX_RESPONSE_SIZE);
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
        assert!(display.contains("---"));
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
