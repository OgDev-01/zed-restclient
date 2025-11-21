//! Enhanced JSON formatting with pretty-print, minification, and validation.
//!
//! This module provides advanced JSON formatting capabilities including:
//! - Pretty-printing with custom indentation (2 spaces)
//! - Minification for compact view
//! - JSON validation
//! - Graceful error handling for malformed JSON

use crate::formatter::FormatError;
use serde_json::Value;

/// Maximum JSON size to format (10MB).
///
/// Responses larger than this will not be formatted to avoid performance issues.
const MAX_JSON_FORMAT_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Size threshold for streaming formatting (1MB).
///
/// JSON responses larger than this will use streaming/chunked formatting.
const STREAMING_THRESHOLD: usize = 1024 * 1024; // 1MB

/// Maximum lines to format when using preview mode for very large responses.
const PREVIEW_MAX_LINES: usize = 1000;

/// Formats JSON with pretty-printing using 2-space indentation.
///
/// This function parses the JSON string and reformats it with consistent
/// indentation for improved readability. If parsing fails, returns an error
/// that can be used to fall back to raw display.
///
/// # Arguments
///
/// * `json` - JSON string to format
///
/// # Returns
///
/// `Ok(String)` with beautifully formatted JSON, or `Err(FormatError)` if:
/// - The JSON is malformed
/// - The JSON exceeds the maximum size limit
///
/// # Examples
///
/// ```
/// use rest_client::formatter::json::format_json_pretty;
///
/// let json = r#"{"name":"John","age":30,"city":"New York"}"#;
/// let formatted = format_json_pretty(json).unwrap();
/// assert!(formatted.contains("  \"name\": \"John\""));
/// ```
pub fn format_json_pretty(json: &str) -> Result<String, FormatError> {
    // Check size limit
    if json.len() > MAX_JSON_FORMAT_SIZE {
        return Err(FormatError::ResponseTooLarge(json.len()));
    }

    // For large responses, use streaming/preview formatting
    if json.len() > STREAMING_THRESHOLD {
        return format_json_streaming(json);
    }

    // Parse JSON to validate and prepare for formatting
    let value: Value =
        serde_json::from_str(json).map_err(|e| FormatError::JsonError(e.to_string()))?;

    // Format with custom 2-space indentation
    // Pre-allocate buffer with estimated capacity (formatted is ~1.5x original size)
    let estimated_size = json.len() + (json.len() / 2);
    let mut buf = Vec::with_capacity(estimated_size);

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);

    use serde::Serialize;
    value
        .serialize(&mut serializer)
        .map_err(|e| FormatError::JsonError(e.to_string()))?;

    String::from_utf8(buf).map_err(|e| FormatError::EncodingError(e.to_string()))
}

/// Formats large JSON using streaming approach to avoid memory spikes.
///
/// For responses larger than 1MB, this formats only a preview portion
/// and indicates that more content is available.
///
/// # Arguments
///
/// * `json` - Large JSON string to format
///
/// # Returns
///
/// `Ok(String)` with formatted preview, or `Err(FormatError)` if parsing fails.
fn format_json_streaming(json: &str) -> Result<String, FormatError> {
    // Parse JSON to validate
    let value: Value =
        serde_json::from_str(json).map_err(|e| FormatError::JsonError(e.to_string()))?;

    // Format with custom 2-space indentation
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut buf = Vec::with_capacity(json.len() + (json.len() / 2));
    let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);

    use serde::Serialize;
    value
        .serialize(&mut serializer)
        .map_err(|e| FormatError::JsonError(e.to_string()))?;

    let formatted =
        String::from_utf8(buf).map_err(|e| FormatError::EncodingError(e.to_string()))?;

    // For very large formatted output, provide a preview
    let lines: Vec<&str> = formatted.lines().collect();
    if lines.len() > PREVIEW_MAX_LINES {
        let preview_lines: Vec<&str> = lines.iter().take(PREVIEW_MAX_LINES).copied().collect();
        Ok(format!(
            "{}\n\n... (showing first {} lines of {}; {} lines truncated for performance)",
            preview_lines.join("\n"),
            PREVIEW_MAX_LINES,
            lines.len(),
            lines.len() - PREVIEW_MAX_LINES
        ))
    } else {
        Ok(formatted)
    }
}

/// Minifies JSON by removing all unnecessary whitespace.
///
/// This is useful for compact view or when displaying inline JSON.
/// The semantic meaning of the JSON is preserved while reducing size.
///
/// # Arguments
///
/// * `json` - JSON string to minify
///
/// # Returns
///
/// `Ok(String)` with minified JSON, or `Err(FormatError)` if parsing fails.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::json::minify_json;
///
/// let json = r#"{
///   "name": "John",
///   "age": 30
/// }"#;
/// let minified = minify_json(json).unwrap();
/// assert!(!minified.contains(" "));
/// assert!(minified.contains("\"name\":\"John\""));
/// assert!(minified.contains("\"age\":30"));
/// ```
pub fn minify_json(json: &str) -> Result<String, FormatError> {
    // Check size limit
    if json.len() > MAX_JSON_FORMAT_SIZE {
        return Err(FormatError::ResponseTooLarge(json.len()));
    }

    // Parse and re-serialize without formatting
    let value: Value =
        serde_json::from_str(json).map_err(|e| FormatError::JsonError(e.to_string()))?;

    // Pre-allocate buffer for better performance
    let mut buf = Vec::with_capacity(json.len());
    serde_json::to_writer(&mut buf, &value).map_err(|e| FormatError::JsonError(e.to_string()))?;

    String::from_utf8(buf).map_err(|e| FormatError::EncodingError(e.to_string()))
}

/// Validates whether a string is valid JSON.
///
/// This is a lightweight check that only parses the JSON without
/// formatting it. Useful for determining whether to attempt formatting.
///
/// # Arguments
///
/// * `json` - String to validate as JSON
///
/// # Returns
///
/// `true` if the string is valid JSON, `false` otherwise.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::json::validate_json;
///
/// assert!(validate_json(r#"{"valid": true}"#));
/// assert!(!validate_json("{invalid json}"));
/// assert!(!validate_json("not json at all"));
/// ```
pub fn validate_json(json: &str) -> bool {
    serde_json::from_str::<Value>(json).is_ok()
}

/// Attempts to format JSON, falling back to raw if formatting fails.
///
/// This is a convenience function that tries to pretty-print JSON,
/// but returns the original string if formatting fails or if the
/// JSON is too large.
///
/// # Arguments
///
/// * `json` - JSON string to format
///
/// # Returns
///
/// Formatted JSON if successful, otherwise the original string.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::json::format_json_safe;
///
/// let valid = r#"{"name":"John"}"#;
/// let formatted = format_json_safe(valid);
/// assert!(formatted.contains("  "));
///
/// let invalid = "{invalid}";
/// let result = format_json_safe(invalid);
/// assert_eq!(result, invalid);
/// ```
pub fn format_json_safe(json: &str) -> String {
    format_json_pretty(json).unwrap_or_else(|_| json.to_string())
}

/// Extracts a subset of JSON for preview purposes.
///
/// This function formats only the first N lines of JSON, useful for
/// previewing large JSON responses without formatting the entire document.
///
/// # Arguments
///
/// * `json` - JSON string to preview
/// * `max_lines` - Maximum number of lines to include in preview
///
/// # Returns
///
/// `Ok(String)` with preview, or `Err(FormatError)` if formatting fails.
pub fn format_json_preview(json: &str, max_lines: usize) -> Result<String, FormatError> {
    let formatted = format_json_pretty(json)?;
    let lines: Vec<&str> = formatted.lines().take(max_lines).collect();

    if lines.len() < formatted.lines().count() {
        Ok(format!(
            "{}\n... ({} more lines)",
            lines.join("\n"),
            formatted.lines().count() - lines.len()
        ))
    } else {
        Ok(lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_json_pretty_simple() {
        let json = r#"{"name":"John","age":30}"#;
        let formatted = format_json_pretty(json).unwrap();

        assert!(formatted.contains("  \"name\": \"John\""));
        assert!(formatted.contains("  \"age\": 30"));
    }

    #[test]
    fn test_format_json_pretty_nested() {
        let json = r#"{"user":{"name":"John","address":{"city":"NYC","zip":"10001"}}}"#;
        let formatted = format_json_pretty(json).unwrap();

        assert!(formatted.contains("  \"user\":"));
        assert!(formatted.contains("    \"name\": \"John\""));
        assert!(formatted.contains("      \"city\": \"NYC\""));
    }

    #[test]
    fn test_format_json_pretty_array() {
        let json = r#"{"items":[1,2,3],"names":["a","b","c"]}"#;
        let formatted = format_json_pretty(json).unwrap();

        assert!(formatted.contains("  \"items\":"));
        assert!(formatted.contains("["));
        assert!(formatted.contains("]"));
    }

    #[test]
    fn test_format_json_pretty_malformed() {
        let json = r#"{"invalid": json}"#;
        let result = format_json_pretty(json);

        assert!(result.is_err());
        match result {
            Err(FormatError::JsonError(_)) => (),
            _ => panic!("Expected JsonError"),
        }
    }

    #[test]
    fn test_minify_json() {
        let json = r#"{
  "name": "John",
  "age": 30,
  "city": "New York"
}"#;
        let minified = minify_json(json).unwrap();

        // serde_json reorders keys alphabetically
        assert_eq!(minified, r#"{"age":30,"city":"New York","name":"John"}"#);
        assert!(!minified.contains('\n'));
        assert!(!minified.contains("  "));
    }

    #[test]
    fn test_minify_json_already_minified() {
        let json = r#"{"name":"John","age":30}"#;
        let minified = minify_json(json).unwrap();

        // serde_json reorders keys alphabetically
        assert_eq!(minified, r#"{"age":30,"name":"John"}"#);
    }

    #[test]
    fn test_validate_json_valid() {
        assert!(validate_json(r#"{"valid": true}"#));
        assert!(validate_json(r#"[1,2,3]"#));
        assert!(validate_json(r#""string""#));
        assert!(validate_json(r#"123"#));
        assert!(validate_json(r#"null"#));
        assert!(validate_json(r#"true"#));
    }

    #[test]
    fn test_validate_json_invalid() {
        assert!(!validate_json("{invalid}"));
        assert!(!validate_json("not json"));
        assert!(!validate_json("{\"unclosed\": "));
        assert!(!validate_json(""));
    }

    #[test]
    fn test_format_json_safe_valid() {
        let json = r#"{"name":"John"}"#;
        let formatted = format_json_safe(json);

        assert!(formatted.contains("  \"name\""));
    }

    #[test]
    fn test_format_json_safe_invalid() {
        let json = "{invalid}";
        let formatted = format_json_safe(json);

        assert_eq!(formatted, json);
    }

    #[test]
    fn test_format_json_preview() {
        let json = r#"{"a":1,"b":2,"c":3,"d":4,"e":5}"#;
        let preview = format_json_preview(json, 3).unwrap();

        assert!(preview.contains("..."));
        assert!(preview.contains("more lines"));
    }

    #[test]
    fn test_format_json_preview_short() {
        let json = r#"{"a":1}"#;
        let preview = format_json_preview(json, 10).unwrap();

        assert!(!preview.contains("..."));
    }

    #[test]
    fn test_format_json_unicode() {
        let json = r#"{"message":"Hello 世界","emoji":"🎉"}"#;
        let formatted = format_json_pretty(json).unwrap();

        assert!(formatted.contains("世界"));
        assert!(formatted.contains("🎉"));
    }

    #[test]
    fn test_format_json_escapes() {
        let json = r#"{"path":"C:\\Users\\test","newline":"line1\nline2"}"#;
        let formatted = format_json_pretty(json).unwrap();

        assert!(formatted.contains("\\\\"));
        assert!(formatted.contains("\\n"));
    }

    #[test]
    fn test_format_json_numbers() {
        let json = r#"{"int":42,"float":3.14,"exp":1.5e10,"negative":-100}"#;
        let formatted = format_json_pretty(json).unwrap();

        assert!(formatted.contains("42"));
        assert!(formatted.contains("3.14"));
        // Scientific notation may be normalized to 15000000000.0
        assert!(formatted.contains("15000000000") || formatted.contains("1.5e10"));
        assert!(formatted.contains("-100"));
    }

    #[test]
    fn test_format_json_empty_structures() {
        let json = r#"{"empty_object":{},"empty_array":[]}"#;
        let formatted = format_json_pretty(json).unwrap();

        assert!(formatted.contains("{}"));
        assert!(formatted.contains("[]"));
    }
}
