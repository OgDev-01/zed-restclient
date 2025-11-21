//! Content type detection and classification.
//!
//! This module provides functionality to detect and classify HTTP response content types,
//! enabling appropriate formatting for different data formats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content type classification for HTTP responses.
///
/// Represents the detected content type of an HTTP response, used to
/// determine the appropriate formatting strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// JSON data (application/json)
    Json,
    /// XML data (application/xml, text/xml)
    Xml,
    /// HTML content (text/html)
    Html,
    /// Plain text (text/plain)
    PlainText,
    /// Binary data (application/octet-stream, etc.)
    Binary,
    /// Image data (image/*)
    Image,
}

impl ContentType {
    /// Returns a human-readable string representation of the content type.
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Json => "JSON",
            ContentType::Xml => "XML",
            ContentType::Html => "HTML",
            ContentType::PlainText => "Plain Text",
            ContentType::Binary => "Binary",
            ContentType::Image => "Image",
        }
    }

    /// Checks if the content type is textual (can be displayed as text).
    pub fn is_textual(&self) -> bool {
        matches!(
            self,
            ContentType::Json | ContentType::Xml | ContentType::Html | ContentType::PlainText
        )
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detects the content type from HTTP headers and body content.
///
/// Uses the Content-Type header as the primary source, with fallback to
/// body inspection for cases where the header is missing or ambiguous.
///
/// # Arguments
///
/// * `headers` - HTTP response headers
/// * `body` - Response body bytes
///
/// # Returns
///
/// The detected `ContentType`.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use rest_client::formatter::content_type::detect_content_type;
///
/// let mut headers = HashMap::new();
/// headers.insert("Content-Type".to_string(), "application/json".to_string());
/// let body = br#"{"key": "value"}"#;
///
/// let content_type = detect_content_type(&headers, body);
/// ```
pub fn detect_content_type(headers: &HashMap<String, String>, body: &[u8]) -> ContentType {
    // First, check the Content-Type header
    if let Some(content_type_header) = find_content_type_header(headers) {
        let content_type_lower = content_type_header.to_lowercase();

        // Parse the content type, ignoring charset and other parameters
        let mime_type = content_type_lower
            .split(';')
            .next()
            .unwrap_or(&content_type_lower)
            .trim();

        // Match against known content types
        if mime_type.contains("json") {
            return ContentType::Json;
        } else if mime_type.contains("xml") {
            return ContentType::Xml;
        } else if mime_type.contains("html") {
            return ContentType::Html;
        } else if mime_type.starts_with("text/") {
            return ContentType::PlainText;
        } else if mime_type.starts_with("image/") {
            return ContentType::Image;
        } else if mime_type == "application/octet-stream"
            || mime_type.contains("binary")
            || mime_type.contains("pdf")
            || mime_type.contains("zip")
            || mime_type.contains("tar")
            || mime_type.contains("gzip")
        {
            return ContentType::Binary;
        }
    }

    // If Content-Type header is missing or unrecognized, inspect the body
    inspect_body_content(body)
}

/// Finds the Content-Type header in a case-insensitive manner.
///
/// # Arguments
///
/// * `headers` - HTTP response headers
///
/// # Returns
///
/// `Some(&str)` with the content type value, or `None` if not found.
fn find_content_type_header(headers: &HashMap<String, String>) -> Option<&str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.as_str())
}

/// Inspects the body content to guess the content type.
///
/// Uses heuristics like looking for JSON/XML/HTML markers to determine
/// the likely content type when headers don't provide clear information.
///
/// # Arguments
///
/// * `body` - Response body bytes
///
/// # Returns
///
/// The guessed `ContentType`.
fn inspect_body_content(body: &[u8]) -> ContentType {
    // Try to interpret as UTF-8 text
    if let Ok(text) = std::str::from_utf8(body) {
        let trimmed = text.trim();

        // Check for JSON markers
        if (trimmed.starts_with('{') && trimmed.ends_with('}'))
            || (trimmed.starts_with('[') && trimmed.ends_with(']'))
        {
            return ContentType::Json;
        }

        // Check for XML/HTML markers
        if trimmed.starts_with("<?xml") || trimmed.starts_with("<xml") {
            return ContentType::Xml;
        }

        if trimmed.starts_with("<!DOCTYPE html")
            || trimmed.starts_with("<!doctype html")
            || trimmed.starts_with("<html")
            || trimmed.starts_with("<HTML")
        {
            return ContentType::Html;
        }

        // Check for generic XML tags
        if trimmed.starts_with('<') && trimmed.contains('>') && trimmed.contains("</") {
            return ContentType::Xml;
        }

        // If it's valid UTF-8 and doesn't match specific formats, treat as plain text
        return ContentType::PlainText;
    }

    // Check for common binary file signatures
    if is_image_signature(body) {
        return ContentType::Image;
    }

    // If we can't decode as UTF-8, assume binary
    ContentType::Binary
}

/// Checks if the body starts with a known image file signature.
///
/// # Arguments
///
/// * `body` - Response body bytes
///
/// # Returns
///
/// `true` if the body appears to be an image, `false` otherwise.
fn is_image_signature(body: &[u8]) -> bool {
    if body.len() < 2 {
        return false;
    }

    // BMP signature: 42 4D (check this first since it only needs 2 bytes)
    if body.len() >= 2 && body[0..2] == [0x42, 0x4D] {
        return true;
    }

    if body.len() < 3 {
        return false;
    }

    // JPEG signature: FF D8 FF
    if body.len() >= 3 && body[0..3] == [0xFF, 0xD8, 0xFF] {
        return true;
    }

    if body.len() < 4 {
        return false;
    }

    // PNG signature: 89 50 4E 47
    if body[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        return true;
    }

    // GIF signature: 47 49 46 38
    if body[0..4] == [0x47, 0x49, 0x46, 0x38] {
        return true;
    }

    // WebP signature: 52 49 46 46 ... 57 45 42 50
    if body.len() >= 12
        && body[0..4] == [0x52, 0x49, 0x46, 0x46]
        && body[8..12] == [0x57, 0x45, 0x42, 0x50]
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_as_str() {
        assert_eq!(ContentType::Json.as_str(), "JSON");
        assert_eq!(ContentType::Xml.as_str(), "XML");
        assert_eq!(ContentType::Html.as_str(), "HTML");
        assert_eq!(ContentType::PlainText.as_str(), "Plain Text");
        assert_eq!(ContentType::Binary.as_str(), "Binary");
        assert_eq!(ContentType::Image.as_str(), "Image");
    }

    #[test]
    fn test_content_type_is_textual() {
        assert!(ContentType::Json.is_textual());
        assert!(ContentType::Xml.is_textual());
        assert!(ContentType::Html.is_textual());
        assert!(ContentType::PlainText.is_textual());
        assert!(!ContentType::Binary.is_textual());
        assert!(!ContentType::Image.is_textual());
    }

    #[test]
    fn test_detect_content_type_from_header_json() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        let body = b"{}";

        assert_eq!(detect_content_type(&headers, body), ContentType::Json);
    }

    #[test]
    fn test_detect_content_type_from_header_json_with_charset() {
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        );
        let body = b"{}";

        assert_eq!(detect_content_type(&headers, body), ContentType::Json);
    }

    #[test]
    fn test_detect_content_type_from_header_xml() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/xml".to_string());
        let body = b"<root></root>";

        assert_eq!(detect_content_type(&headers, body), ContentType::Xml);
    }

    #[test]
    fn test_detect_content_type_from_header_html() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html".to_string());
        let body = b"<html></html>";

        assert_eq!(detect_content_type(&headers, body), ContentType::Html);
    }

    #[test]
    fn test_detect_content_type_from_header_plain_text() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        let body = b"Hello, World!";

        assert_eq!(detect_content_type(&headers, body), ContentType::PlainText);
    }

    #[test]
    fn test_detect_content_type_from_header_image() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "image/png".to_string());
        let body = b"\x89PNG\r\n\x1a\n";

        assert_eq!(detect_content_type(&headers, body), ContentType::Image);
    }

    #[test]
    fn test_detect_content_type_from_header_binary() {
        let mut headers = HashMap::new();
        headers.insert(
            "Content-Type".to_string(),
            "application/octet-stream".to_string(),
        );
        let body = b"\x00\x01\x02\x03";

        assert_eq!(detect_content_type(&headers, body), ContentType::Binary);
    }

    #[test]
    fn test_detect_content_type_case_insensitive_header() {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        let body = b"{}";

        assert_eq!(detect_content_type(&headers, body), ContentType::Json);
    }

    #[test]
    fn test_inspect_body_json_object() {
        let body = br#"{"key": "value"}"#;
        assert_eq!(inspect_body_content(body), ContentType::Json);
    }

    #[test]
    fn test_inspect_body_json_array() {
        let body = br#"[1, 2, 3]"#;
        assert_eq!(inspect_body_content(body), ContentType::Json);
    }

    #[test]
    fn test_inspect_body_xml_declaration() {
        let body = b"<?xml version=\"1.0\"?><root></root>";
        assert_eq!(inspect_body_content(body), ContentType::Xml);
    }

    #[test]
    fn test_inspect_body_xml_tags() {
        let body = b"<root><child>text</child></root>";
        assert_eq!(inspect_body_content(body), ContentType::Xml);
    }

    #[test]
    fn test_inspect_body_html_doctype() {
        let body = b"<!DOCTYPE html><html><body></body></html>";
        assert_eq!(inspect_body_content(body), ContentType::Html);
    }

    #[test]
    fn test_inspect_body_html_tag() {
        let body = b"<html><head><title>Test</title></head></html>";
        assert_eq!(inspect_body_content(body), ContentType::Html);
    }

    #[test]
    fn test_inspect_body_plain_text() {
        let body = b"Hello, World! This is plain text.";
        assert_eq!(inspect_body_content(body), ContentType::PlainText);
    }

    #[test]
    fn test_inspect_body_binary() {
        let body = b"\x00\x01\x02\x03\xFF\xFE\xFD";
        assert_eq!(inspect_body_content(body), ContentType::Binary);
    }

    #[test]
    fn test_is_image_signature_png() {
        let png = b"\x89PNG\r\n\x1a\n";
        assert!(is_image_signature(png));
    }

    #[test]
    fn test_is_image_signature_jpeg() {
        let jpeg = b"\xFF\xD8\xFF\xE0";
        assert!(is_image_signature(jpeg));
    }

    #[test]
    fn test_is_image_signature_gif() {
        let gif = b"GIF89a";
        assert!(is_image_signature(gif));
    }

    #[test]
    fn test_is_image_signature_bmp() {
        let bmp = b"BM";
        assert!(is_image_signature(bmp));
    }

    #[test]
    fn test_is_image_signature_webp() {
        let webp = b"RIFF\x00\x00\x00\x00WEBP";
        assert!(is_image_signature(webp));
    }

    #[test]
    fn test_is_image_signature_not_image() {
        let text = b"Hello, World!";
        assert!(!is_image_signature(text));
    }

    #[test]
    fn test_detect_content_type_empty_body() {
        let headers = HashMap::new();
        let body = b"";
        assert_eq!(detect_content_type(&headers, body), ContentType::PlainText);
    }
}
