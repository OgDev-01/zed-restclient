//! HTTP request data models.
//!
//! This module defines the core data structures for representing HTTP requests,
//! including the request method, headers, body, and metadata.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// HTTP request method.
///
/// Represents all standard HTTP methods as defined in RFC 7231 and RFC 5789.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HttpMethod {
    /// HTTP GET method - retrieve a resource
    GET,
    /// HTTP POST method - submit data to create a resource
    POST,
    /// HTTP PUT method - replace a resource
    PUT,
    /// HTTP DELETE method - remove a resource
    DELETE,
    /// HTTP PATCH method - partially modify a resource
    PATCH,
    /// HTTP OPTIONS method - describe communication options
    OPTIONS,
    /// HTTP HEAD method - retrieve headers only
    HEAD,
    /// HTTP TRACE method - perform a message loop-back test
    TRACE,
    /// HTTP CONNECT method - establish a tunnel to the server
    CONNECT,
}

impl HttpMethod {
    /// Returns the string representation of the HTTP method.
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::GET => "GET",
            HttpMethod::POST => "POST",
            HttpMethod::PUT => "PUT",
            HttpMethod::DELETE => "DELETE",
            HttpMethod::PATCH => "PATCH",
            HttpMethod::OPTIONS => "OPTIONS",
            HttpMethod::HEAD => "HEAD",
            HttpMethod::TRACE => "TRACE",
            HttpMethod::CONNECT => "CONNECT",
        }
    }

    /// Parses a string into an HttpMethod.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice representing the HTTP method
    ///
    /// # Returns
    ///
    /// `Some(HttpMethod)` if the string is a valid HTTP method, `None` otherwise.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(HttpMethod::GET),
            "POST" => Some(HttpMethod::POST),
            "PUT" => Some(HttpMethod::PUT),
            "DELETE" => Some(HttpMethod::DELETE),
            "PATCH" => Some(HttpMethod::PATCH),
            "OPTIONS" => Some(HttpMethod::OPTIONS),
            "HEAD" => Some(HttpMethod::HEAD),
            "TRACE" => Some(HttpMethod::TRACE),
            "CONNECT" => Some(HttpMethod::CONNECT),
            _ => None,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents an HTTP request parsed from a `.http` or `.rest` file.
///
/// This structure contains all the information needed to execute an HTTP request,
/// including the method, URL, headers, body, and metadata about its location in
/// the source file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// Unique identifier for tracking this request.
    ///
    /// Used for request history, cancellation, and correlation with responses.
    pub id: String,

    /// HTTP method (GET, POST, PUT, DELETE, etc.).
    pub method: HttpMethod,

    /// Target URL for the request.
    ///
    /// May contain variables in the format `{{variableName}}` that will be
    /// resolved before execution.
    pub url: String,

    /// Optional HTTP version specification.
    ///
    /// If not specified, defaults to HTTP/1.1. Example: "HTTP/1.1", "HTTP/2"
    pub http_version: Option<String>,

    /// Request headers as key-value pairs.
    ///
    /// Header names are case-insensitive but are stored as provided in the
    /// source file. Common headers include Content-Type, Authorization, etc.
    pub headers: HashMap<String, String>,

    /// Optional request body.
    ///
    /// Contains the raw body content which may be JSON, XML, form data, or
    /// plain text depending on the Content-Type header.
    pub body: Option<String>,

    /// Line number in the source file where this request starts.
    ///
    /// Used for error reporting and diagnostics to help users locate issues
    /// in their `.http` files.
    pub line_number: usize,

    /// Path to the source file containing this request.
    ///
    /// Used for resolving relative paths and providing context in error messages.
    pub file_path: PathBuf,
}

impl HttpRequest {
    /// Creates a new HttpRequest with the given parameters.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the request
    /// * `method` - HTTP method
    /// * `url` - Target URL
    ///
    /// # Returns
    ///
    /// A new `HttpRequest` with default values for optional fields.
    pub fn new(id: String, method: HttpMethod, url: String) -> Self {
        Self {
            id,
            method,
            url,
            http_version: None,
            headers: HashMap::new(),
            body: None,
            line_number: 0,
            file_path: PathBuf::new(),
        }
    }

    /// Adds a header to the request.
    ///
    /// # Arguments
    ///
    /// * `name` - Header name
    /// * `value` - Header value
    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    /// Sets the request body.
    ///
    /// # Arguments
    ///
    /// * `body` - The body content
    pub fn set_body(&mut self, body: String) {
        self.body = Some(body);
    }

    /// Checks if the request has a body.
    ///
    /// # Returns
    ///
    /// `true` if the request has a non-empty body, `false` otherwise.
    pub fn has_body(&self) -> bool {
        self.body.as_ref().map_or(false, |b| !b.is_empty())
    }

    /// Gets the Content-Type header value if present.
    ///
    /// # Returns
    ///
    /// `Some(&str)` with the content type, or `None` if not set.
    pub fn content_type(&self) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::GET.as_str(), "GET");
        assert_eq!(HttpMethod::POST.as_str(), "POST");
        assert_eq!(HttpMethod::DELETE.as_str(), "DELETE");
    }

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("GET"), Some(HttpMethod::GET));
        assert_eq!(HttpMethod::from_str("get"), Some(HttpMethod::GET));
        assert_eq!(HttpMethod::from_str("Post"), Some(HttpMethod::POST));
        assert_eq!(HttpMethod::from_str("INVALID"), None);
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(format!("{}", HttpMethod::GET), "GET");
        assert_eq!(format!("{}", HttpMethod::PATCH), "PATCH");
    }

    #[test]
    fn test_http_request_new() {
        let request = HttpRequest::new(
            "test-id".to_string(),
            HttpMethod::GET,
            "https://example.com".to_string(),
        );

        assert_eq!(request.id, "test-id");
        assert_eq!(request.method, HttpMethod::GET);
        assert_eq!(request.url, "https://example.com");
        assert_eq!(request.http_version, None);
        assert!(request.headers.is_empty());
        assert_eq!(request.body, None);
    }

    #[test]
    fn test_http_request_add_header() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://example.com".to_string(),
        );

        request.add_header("Content-Type".to_string(), "application/json".to_string());
        assert_eq!(request.headers.len(), 1);
        assert_eq!(
            request.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_http_request_set_body() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://example.com".to_string(),
        );

        request.set_body(r#"{"key": "value"}"#.to_string());
        assert!(request.has_body());
        assert_eq!(request.body, Some(r#"{"key": "value"}"#.to_string()));
    }

    #[test]
    fn test_http_request_content_type() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://example.com".to_string(),
        );

        assert_eq!(request.content_type(), None);

        request.add_header("Content-Type".to_string(), "application/json".to_string());
        assert_eq!(request.content_type(), Some("application/json"));

        // Test case-insensitive lookup
        request.headers.clear();
        request.add_header("content-type".to_string(), "text/plain".to_string());
        assert_eq!(request.content_type(), Some("text/plain"));
    }

    #[test]
    fn test_serialization() {
        let request = HttpRequest::new(
            "test-123".to_string(),
            HttpMethod::GET,
            "https://api.example.com/data".to_string(),
        );

        // Test that serialization works
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test-123"));
        assert!(json.contains("GET"));

        // Test deserialization
        let deserialized: HttpRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, request.id);
        assert_eq!(deserialized.method, request.method);
        assert_eq!(deserialized.url, request.url);
    }
}
