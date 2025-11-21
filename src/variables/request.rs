//! Request variable extraction from HTTP responses.
//!
//! This module provides functionality to extract values from HTTP responses
//! using JSONPath, XPath, or header extraction, and store them as variables
//! for use in subsequent requests.
//!
//! # Examples
//!
//! ```
//! use rest_client::variables::request::{extract_response_variable, ContentType};
//! use rest_client::models::response::HttpResponse;
//! use std::collections::HashMap;
//!
//! let mut response = HttpResponse::new(200, "OK".to_string());
//! response.set_body(r#"{"token": "abc123"}"#.as_bytes().to_vec());
//! response.add_header("Content-Type".to_string(), "application/json".to_string());
//!
//! let value = extract_response_variable(&response, "$.token", ContentType::Json);
//! assert!(value.is_ok());
//! ```

use super::{capture::PathType, VarError};
use crate::models::response::HttpResponse;
use serde_json::Value as JsonValue;

/// Content type of an HTTP response for extraction purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    /// JSON content (application/json, application/*+json)
    Json,

    /// XML content (application/xml, text/xml, application/*+xml)
    Xml,

    /// Plain text content
    Text,

    /// HTML content
    Html,

    /// Binary or unknown content
    Binary,
}

impl ContentType {
    /// Determines the content type from a Content-Type header value.
    ///
    /// # Arguments
    ///
    /// * `content_type_header` - The Content-Type header value
    ///
    /// # Returns
    ///
    /// The corresponding `ContentType` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use rest_client::variables::request::ContentType;
    ///
    /// assert_eq!(ContentType::from_header("application/json"), ContentType::Json);
    /// assert_eq!(ContentType::from_header("text/xml; charset=utf-8"), ContentType::Xml);
    /// ```
    pub fn from_header(content_type_header: &str) -> Self {
        let lower = content_type_header.to_lowercase();

        // Remove charset and other parameters
        let media_type = lower.split(';').next().unwrap_or("").trim();

        if media_type.contains("json") {
            ContentType::Json
        } else if media_type.contains("xml") {
            ContentType::Xml
        } else if media_type.starts_with("text/html") {
            ContentType::Html
        } else if media_type.starts_with("text/") {
            ContentType::Text
        } else {
            ContentType::Binary
        }
    }

    /// Determines the content type from an HttpResponse.
    ///
    /// # Arguments
    ///
    /// * `response` - The HTTP response
    ///
    /// # Returns
    ///
    /// The content type based on the Content-Type header, or Binary if not present.
    pub fn from_response(response: &HttpResponse) -> Self {
        response
            .content_type()
            .map(Self::from_header)
            .unwrap_or(ContentType::Binary)
    }
}

/// Extracts a value from an HTTP response using the specified path.
///
/// # Arguments
///
/// * `response` - The HTTP response to extract from
/// * `path` - The extraction path (JSONPath, XPath, or header name)
/// * `content_type` - The content type of the response
///
/// # Returns
///
/// `Ok(String)` with the extracted value, or `Err(VarError)` if extraction fails.
///
/// # Examples
///
/// ```
/// use rest_client::variables::request::{extract_response_variable, ContentType};
/// use rest_client::models::response::HttpResponse;
///
/// let mut response = HttpResponse::new(200, "OK".to_string());
/// response.set_body(r#"{"user": {"id": 123, "name": "Alice"}}"#.as_bytes().to_vec());
///
/// let user_id = extract_response_variable(&response, "$.user.id", ContentType::Json).unwrap();
/// assert_eq!(user_id, "123");
/// ```
pub fn extract_response_variable(
    response: &HttpResponse,
    path: &str,
    content_type: ContentType,
) -> Result<String, VarError> {
    let path_type = PathType::from_path(path);

    match path_type {
        PathType::Header(header_name) => extract_header_value(response, &header_name),
        PathType::JsonPath(jsonpath) => {
            if content_type != ContentType::Json {
                return Err(VarError::InvalidSyntax(format!(
                    "JSONPath extraction requires JSON content type, got {:?}",
                    content_type
                )));
            }
            extract_json_value(response, &jsonpath)
        }
        PathType::XPath(xpath) => {
            if content_type != ContentType::Xml && content_type != ContentType::Html {
                return Err(VarError::InvalidSyntax(format!(
                    "XPath extraction requires XML/HTML content type, got {:?}",
                    content_type
                )));
            }
            extract_xml_value(response, &xpath)
        }
    }
}

/// Extracts a header value from an HTTP response.
///
/// # Arguments
///
/// * `response` - The HTTP response
/// * `header_name` - Name of the header to extract (case-insensitive)
///
/// # Returns
///
/// `Ok(String)` with the header value, or `Err(VarError)` if header not found.
fn extract_header_value(response: &HttpResponse, header_name: &str) -> Result<String, VarError> {
    response
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(header_name))
        .map(|(_, v)| v.clone())
        .ok_or_else(|| {
            VarError::UndefinedVariable(format!("Header '{}' not found in response", header_name))
        })
}

/// Extracts a value from a JSON response using JSONPath.
///
/// # Arguments
///
/// * `response` - The HTTP response containing JSON
/// * `path` - JSONPath expression (e.g., "$.user.id")
///
/// # Returns
///
/// `Ok(String)` with the extracted value (serialized if object/array),
/// or `Err(VarError)` if extraction fails.
fn extract_json_value(response: &HttpResponse, path: &str) -> Result<String, VarError> {
    // Parse response body as JSON
    let body_str = response
        .body_as_string()
        .map_err(|_| VarError::InvalidSyntax("Response body is not valid UTF-8".to_string()))?;

    let json: JsonValue = serde_json::from_str(&body_str)
        .map_err(|e| VarError::InvalidSyntax(format!("Failed to parse JSON response: {}", e)))?;

    // Evaluate JSONPath
    let value = evaluate_jsonpath(&json, path)?;

    // Convert value to string
    json_value_to_string(value)
}

/// Evaluates a JSONPath expression against a JSON value.
///
/// # Arguments
///
/// * `json` - The JSON value to query
/// * `path` - JSONPath expression
///
/// # Returns
///
/// The extracted JSON value, or an error if the path is invalid or not found.
fn evaluate_jsonpath(json: &JsonValue, path: &str) -> Result<JsonValue, VarError> {
    // Simple JSONPath implementation supporting common patterns
    // For production, consider using a dedicated JSONPath library like serde_json_path

    let path = path.trim();

    // Handle root reference
    if path == "$" || path == "@" {
        return Ok(json.clone());
    }

    // Remove leading $ or @
    let path = path
        .strip_prefix('$')
        .or_else(|| path.strip_prefix('@'))
        .unwrap_or(path);

    // Remove leading dot if present
    let path = path.strip_prefix('.').unwrap_or(path);

    if path.is_empty() {
        return Ok(json.clone());
    }

    // Split path into segments
    let segments = parse_jsonpath_segments(path);

    // Navigate through JSON structure
    let mut current = json;

    for segment in segments {
        current = match segment {
            PathSegment::Field(name) => current.get(&name).ok_or_else(|| {
                VarError::UndefinedVariable(format!("Field '{}' not found in JSON", name))
            })?,
            PathSegment::ArrayIndex(index) => current.get(index).ok_or_else(|| {
                VarError::UndefinedVariable(format!("Array index {} out of bounds", index))
            })?,
        };
    }

    Ok(current.clone())
}

/// Represents a segment in a JSONPath expression.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PathSegment {
    /// Object field access (e.g., "user", "name")
    Field(String),

    /// Array index access (e.g., [0], [5])
    ArrayIndex(usize),
}

/// Parses a JSONPath into segments.
///
/// # Arguments
///
/// * `path` - JSONPath string (without leading $ or @)
///
/// # Returns
///
/// Vector of path segments.
///
/// # Examples
///
/// - "user.name" -> [Field("user"), Field("name")]
/// - "items[0].id" -> [Field("items"), ArrayIndex(0), Field("id")]
fn parse_jsonpath_segments(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if !current.is_empty() {
                    segments.push(PathSegment::Field(current.clone()));
                    current.clear();
                }
            }
            '[' => {
                // Add current field if any
                if !current.is_empty() {
                    segments.push(PathSegment::Field(current.clone()));
                    current.clear();
                }

                // Parse array index
                let mut index_str = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ']' {
                        chars.next(); // consume ]
                        break;
                    }
                    index_str.push(chars.next().unwrap());
                }

                // Try to parse as integer
                if let Ok(index) = index_str.trim().parse::<usize>() {
                    segments.push(PathSegment::ArrayIndex(index));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    // Add final segment if any
    if !current.is_empty() {
        segments.push(PathSegment::Field(current));
    }

    segments
}

/// Converts a JSON value to a string representation.
///
/// # Arguments
///
/// * `value` - The JSON value to convert
///
/// # Returns
///
/// String representation of the value.
///
/// # Logic
///
/// - Strings: returned as-is (without quotes)
/// - Numbers, booleans, null: converted to string
/// - Objects, arrays: serialized as JSON
fn json_value_to_string(value: JsonValue) -> Result<String, VarError> {
    match value {
        JsonValue::String(s) => Ok(s),
        JsonValue::Number(n) => Ok(n.to_string()),
        JsonValue::Bool(b) => Ok(b.to_string()),
        JsonValue::Null => Ok("null".to_string()),
        JsonValue::Array(_) | JsonValue::Object(_) => serde_json::to_string(&value)
            .map_err(|e| VarError::InvalidSyntax(format!("Failed to serialize JSON value: {}", e))),
    }
}

/// Extracts a value from an XML response using XPath.
///
/// # Arguments
///
/// * `response` - The HTTP response containing XML
/// * `path` - XPath expression
///
/// # Returns
///
/// `Ok(String)` with the extracted value, or `Err(VarError)` if extraction fails.
///
/// # Note
///
/// This is a placeholder implementation. For production use, integrate
/// a proper XPath library like `sxd-xpath` or `xmltree`.
fn extract_xml_value(_response: &HttpResponse, _path: &str) -> Result<String, VarError> {
    // TODO: Implement XPath support with a proper XML parsing library
    // For now, return an error indicating XPath is not yet supported
    Err(VarError::InvalidSyntax(
        "XPath extraction is not yet implemented. Use JSONPath for JSON responses or header extraction.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_from_header() {
        assert_eq!(
            ContentType::from_header("application/json"),
            ContentType::Json
        );
        assert_eq!(
            ContentType::from_header("application/json; charset=utf-8"),
            ContentType::Json
        );
        assert_eq!(
            ContentType::from_header("application/vnd.api+json"),
            ContentType::Json
        );
        assert_eq!(ContentType::from_header("text/xml"), ContentType::Xml);
        assert_eq!(
            ContentType::from_header("application/xml"),
            ContentType::Xml
        );
        assert_eq!(ContentType::from_header("text/html"), ContentType::Html);
        assert_eq!(ContentType::from_header("text/plain"), ContentType::Text);
        assert_eq!(
            ContentType::from_header("application/octet-stream"),
            ContentType::Binary
        );
    }

    #[test]
    fn test_content_type_from_response() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "application/json".to_string());

        assert_eq!(ContentType::from_response(&response), ContentType::Json);
    }

    #[test]
    fn test_extract_header_value() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Authorization".to_string(), "Bearer token123".to_string());
        response.add_header("X-Session-Id".to_string(), "session456".to_string());

        let auth = extract_header_value(&response, "Authorization").unwrap();
        assert_eq!(auth, "Bearer token123");

        let session = extract_header_value(&response, "X-Session-Id").unwrap();
        assert_eq!(session, "session456");

        // Test case-insensitive lookup
        let auth_lower = extract_header_value(&response, "authorization").unwrap();
        assert_eq!(auth_lower, "Bearer token123");

        // Test missing header
        let missing = extract_header_value(&response, "X-Missing-Header");
        assert!(missing.is_err());
    }

    #[test]
    fn test_extract_json_simple_field() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"token": "abc123"}"#.as_bytes().to_vec());

        let value = extract_json_value(&response, "$.token").unwrap();
        assert_eq!(value, "abc123");
    }

    #[test]
    fn test_extract_json_nested_field() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"user": {"id": 123, "name": "Alice"}}"#.as_bytes().to_vec());

        let user_id = extract_json_value(&response, "$.user.id").unwrap();
        assert_eq!(user_id, "123");

        let user_name = extract_json_value(&response, "$.user.name").unwrap();
        assert_eq!(user_name, "Alice");
    }

    #[test]
    fn test_extract_json_array_index() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"items": [{"id": 1}, {"id": 2}, {"id": 3}]}"#.as_bytes().to_vec());

        let first_id = extract_json_value(&response, "$.items[0].id").unwrap();
        assert_eq!(first_id, "1");

        let second_id = extract_json_value(&response, "$.items[1].id").unwrap();
        assert_eq!(second_id, "2");
    }

    #[test]
    fn test_extract_json_root() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"status": "ok"}"#.as_bytes().to_vec());

        let root = extract_json_value(&response, "$").unwrap();
        assert!(root.contains("status"));
        assert!(root.contains("ok"));
    }

    #[test]
    fn test_extract_json_array_value() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"tags": ["rust", "json", "api"]}"#.as_bytes().to_vec());

        let tags = extract_json_value(&response, "$.tags").unwrap();
        assert!(tags.contains("rust"));
        assert!(tags.contains("json"));
    }

    #[test]
    fn test_extract_json_object_value() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"user": {"id": 1, "name": "Bob"}}"#.as_bytes().to_vec());

        let user = extract_json_value(&response, "$.user").unwrap();
        assert!(user.contains("id"));
        assert!(user.contains("name"));
    }

    #[test]
    fn test_extract_json_number() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"count": 42, "price": 19.99}"#.as_bytes().to_vec());

        let count = extract_json_value(&response, "$.count").unwrap();
        assert_eq!(count, "42");

        let price = extract_json_value(&response, "$.price").unwrap();
        assert_eq!(price, "19.99");
    }

    #[test]
    fn test_extract_json_boolean() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"active": true, "deleted": false}"#.as_bytes().to_vec());

        let active = extract_json_value(&response, "$.active").unwrap();
        assert_eq!(active, "true");

        let deleted = extract_json_value(&response, "$.deleted").unwrap();
        assert_eq!(deleted, "false");
    }

    #[test]
    fn test_extract_json_null() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"value": null}"#.as_bytes().to_vec());

        let value = extract_json_value(&response, "$.value").unwrap();
        assert_eq!(value, "null");
    }

    #[test]
    fn test_extract_json_invalid_path() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"user": {"id": 123}}"#.as_bytes().to_vec());

        let result = extract_json_value(&response, "$.user.nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_json_invalid_json() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(b"not valid json".to_vec());

        let result = extract_json_value(&response, "$.field");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_response_variable_header() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("X-Token".to_string(), "token123".to_string());

        let value =
            extract_response_variable(&response, "headers.X-Token", ContentType::Json).unwrap();
        assert_eq!(value, "token123");
    }

    #[test]
    fn test_extract_response_variable_json() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"token": "abc123"}"#.as_bytes().to_vec());

        let value = extract_response_variable(&response, "$.token", ContentType::Json).unwrap();
        assert_eq!(value, "abc123");
    }

    #[test]
    fn test_extract_response_variable_wrong_content_type() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(r#"{"token": "abc123"}"#.as_bytes().to_vec());

        // Try to use JSONPath on XML content type
        let result = extract_response_variable(&response, "$.token", ContentType::Xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_jsonpath_segments() {
        let segments = parse_jsonpath_segments("user.name");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], PathSegment::Field("user".to_string()));
        assert_eq!(segments[1], PathSegment::Field("name".to_string()));

        let segments = parse_jsonpath_segments("items[0].id");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], PathSegment::Field("items".to_string()));
        assert_eq!(segments[1], PathSegment::ArrayIndex(0));
        assert_eq!(segments[2], PathSegment::Field("id".to_string()));

        let segments = parse_jsonpath_segments("data.users[2].profile.email");
        assert_eq!(segments.len(), 5);
        assert_eq!(segments[0], PathSegment::Field("data".to_string()));
        assert_eq!(segments[1], PathSegment::Field("users".to_string()));
        assert_eq!(segments[2], PathSegment::ArrayIndex(2));
        assert_eq!(segments[3], PathSegment::Field("profile".to_string()));
        assert_eq!(segments[4], PathSegment::Field("email".to_string()));
    }

    #[test]
    fn test_json_value_to_string() {
        assert_eq!(
            json_value_to_string(JsonValue::String("test".to_string())).unwrap(),
            "test"
        );
        assert_eq!(
            json_value_to_string(JsonValue::Number(42.into())).unwrap(),
            "42"
        );
        assert_eq!(json_value_to_string(JsonValue::Bool(true)).unwrap(), "true");
        assert_eq!(json_value_to_string(JsonValue::Null).unwrap(), "null");

        let array = serde_json::json!(["a", "b", "c"]);
        let result = json_value_to_string(array).unwrap();
        assert!(result.contains("a"));
        assert!(result.contains("b"));
        assert!(result.contains("c"));
    }

    #[test]
    fn test_evaluate_jsonpath_complex() {
        let json: JsonValue = serde_json::from_str(
            r#"{
            "data": {
                "users": [
                    {"id": 1, "name": "Alice", "email": "alice@example.com"},
                    {"id": 2, "name": "Bob", "email": "bob@example.com"}
                ],
                "count": 2
            }
        }"#,
        )
        .unwrap();

        let result = evaluate_jsonpath(&json, "$.data.users[0].name").unwrap();
        assert_eq!(json_value_to_string(result).unwrap(), "Alice");

        let result = evaluate_jsonpath(&json, "$.data.users[1].email").unwrap();
        assert_eq!(json_value_to_string(result).unwrap(), "bob@example.com");

        let result = evaluate_jsonpath(&json, "$.data.count").unwrap();
        assert_eq!(json_value_to_string(result).unwrap(), "2");
    }

    #[test]
    fn test_extract_xml_not_implemented() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.set_body(b"<root><value>test</value></root>".to_vec());

        let result = extract_xml_value(&response, "/root/value");
        assert!(result.is_err());
    }
}
