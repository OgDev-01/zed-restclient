//! HTTP response data models.
//!
//! This module defines the core data structures for representing HTTP responses,
//! including status information, headers, body, and performance timing metrics.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Performance timing breakdown for an HTTP request.
///
/// Tracks the duration of each phase of the HTTP request/response cycle
/// to help users identify performance bottlenecks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestTiming {
    /// Time spent on DNS lookup.
    ///
    /// Duration from request start to DNS resolution completion.
    pub dns_lookup: Duration,

    /// Time spent establishing TCP connection.
    ///
    /// Duration from DNS resolution to TCP connection establishment.
    pub tcp_connection: Duration,

    /// Time spent on TLS/SSL handshake (if HTTPS).
    ///
    /// Only present for HTTPS requests. Duration from TCP connection
    /// to TLS handshake completion.
    pub tls_handshake: Option<Duration>,

    /// Time to first byte (TTFB).
    ///
    /// Duration from request sent to receiving the first response byte.
    /// Indicates server processing time plus network latency.
    pub first_byte: Duration,

    /// Time spent downloading the response body.
    ///
    /// Duration from first byte to complete response received.
    pub download: Duration,
}

impl RequestTiming {
    /// Creates a new RequestTiming with all durations set to zero.
    ///
    /// # Returns
    ///
    /// A new `RequestTiming` instance with default zero values.
    pub fn new() -> Self {
        Self {
            dns_lookup: Duration::from_secs(0),
            tcp_connection: Duration::from_secs(0),
            tls_handshake: None,
            first_byte: Duration::from_secs(0),
            download: Duration::from_secs(0),
        }
    }

    /// Calculates the total time from all timing components.
    ///
    /// # Returns
    ///
    /// The sum of all timing durations including optional TLS handshake.
    pub fn total(&self) -> Duration {
        let base = self.dns_lookup + self.tcp_connection + self.first_byte + self.download;
        if let Some(tls) = self.tls_handshake {
            base + tls
        } else {
            base
        }
    }
}

impl Default for RequestTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an HTTP response received from a server.
///
/// This structure contains all the information about an HTTP response,
/// including status code, headers, body, and performance metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code (e.g., 200, 404, 500).
    ///
    /// Standard HTTP status codes as defined in RFC 7231.
    pub status_code: u16,

    /// HTTP status text (e.g., "OK", "Not Found", "Internal Server Error").
    ///
    /// Human-readable description of the status code.
    pub status_text: String,

    /// Response headers as key-value pairs.
    ///
    /// Contains all HTTP headers returned by the server, such as
    /// Content-Type, Content-Length, Set-Cookie, etc.
    pub headers: HashMap<String, String>,

    /// Response body as raw bytes.
    ///
    /// Contains the complete response body. Use `Vec<u8>` instead of `String`
    /// to support binary responses (images, PDFs, etc.) as well as text.
    pub body: Vec<u8>,

    /// Total request duration from start to completion.
    ///
    /// Includes all phases: DNS lookup, connection, TLS handshake (if applicable),
    /// server processing, and response download.
    pub duration: Duration,

    /// Detailed performance timing breakdown.
    ///
    /// Provides granular timing information for each phase of the request
    /// to help identify bottlenecks.
    pub timing: RequestTiming,

    /// Total response size in bytes.
    ///
    /// Includes headers and body. Useful for tracking bandwidth usage.
    pub size: usize,
}

impl HttpResponse {
    /// Creates a new HttpResponse with the given status code and text.
    ///
    /// # Arguments
    ///
    /// * `status_code` - HTTP status code
    /// * `status_text` - HTTP status text description
    ///
    /// # Returns
    ///
    /// A new `HttpResponse` with default values for optional fields.
    pub fn new(status_code: u16, status_text: String) -> Self {
        Self {
            status_code,
            status_text,
            headers: HashMap::new(),
            body: Vec::new(),
            duration: Duration::from_secs(0),
            timing: RequestTiming::new(),
            size: 0,
        }
    }

    /// Checks if the response status indicates success (2xx).
    ///
    /// # Returns
    ///
    /// `true` if status code is in the 200-299 range, `false` otherwise.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status_code)
    }

    /// Checks if the response status indicates a client error (4xx).
    ///
    /// # Returns
    ///
    /// `true` if status code is in the 400-499 range, `false` otherwise.
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status_code)
    }

    /// Checks if the response status indicates a server error (5xx).
    ///
    /// # Returns
    ///
    /// `true` if status code is in the 500-599 range, `false` otherwise.
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status_code)
    }

    /// Checks if the response status indicates a redirection (3xx).
    ///
    /// # Returns
    ///
    /// `true` if status code is in the 300-399 range, `false` otherwise.
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status_code)
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

    /// Attempts to parse the response body as UTF-8 text.
    ///
    /// # Returns
    ///
    /// `Ok(String)` if the body is valid UTF-8, `Err` otherwise.
    pub fn body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Adds a header to the response.
    ///
    /// # Arguments
    ///
    /// * `name` - Header name
    /// * `value` - Header value
    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    /// Sets the response body.
    ///
    /// # Arguments
    ///
    /// * `body` - The body content as bytes
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.size = self.calculate_headers_size() + body.len();
        self.body = body;
    }

    /// Calculates the approximate size of headers in bytes.
    ///
    /// # Returns
    ///
    /// Estimated size of all headers combined.
    fn calculate_headers_size(&self) -> usize {
        self.headers
            .iter()
            .map(|(k, v)| k.len() + v.len() + 4) // +4 for ": " and "\r\n"
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_timing_new() {
        let timing = RequestTiming::new();
        assert_eq!(timing.dns_lookup, Duration::from_secs(0));
        assert_eq!(timing.tcp_connection, Duration::from_secs(0));
        assert_eq!(timing.tls_handshake, None);
        assert_eq!(timing.first_byte, Duration::from_secs(0));
        assert_eq!(timing.download, Duration::from_secs(0));
    }

    #[test]
    fn test_request_timing_total() {
        let mut timing = RequestTiming::new();
        timing.dns_lookup = Duration::from_millis(10);
        timing.tcp_connection = Duration::from_millis(20);
        timing.first_byte = Duration::from_millis(100);
        timing.download = Duration::from_millis(50);

        assert_eq!(timing.total(), Duration::from_millis(180));

        timing.tls_handshake = Some(Duration::from_millis(30));
        assert_eq!(timing.total(), Duration::from_millis(210));
    }

    #[test]
    fn test_http_response_new() {
        let response = HttpResponse::new(200, "OK".to_string());

        assert_eq!(response.status_code, 200);
        assert_eq!(response.status_text, "OK");
        assert!(response.headers.is_empty());
        assert!(response.body.is_empty());
        assert_eq!(response.size, 0);
    }

    #[test]
    fn test_http_response_status_checks() {
        let success = HttpResponse::new(200, "OK".to_string());
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());
        assert!(!success.is_redirect());

        let redirect = HttpResponse::new(301, "Moved Permanently".to_string());
        assert!(redirect.is_redirect());
        assert!(!redirect.is_success());

        let client_error = HttpResponse::new(404, "Not Found".to_string());
        assert!(client_error.is_client_error());
        assert!(!client_error.is_success());

        let server_error = HttpResponse::new(500, "Internal Server Error".to_string());
        assert!(server_error.is_server_error());
        assert!(!server_error.is_success());
    }

    #[test]
    fn test_http_response_add_header() {
        let mut response = HttpResponse::new(200, "OK".to_string());

        response.add_header("Content-Type".to_string(), "application/json".to_string());
        assert_eq!(response.headers.len(), 1);
        assert_eq!(
            response.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_http_response_set_body() {
        let mut response = HttpResponse::new(200, "OK".to_string());

        let body_text = "Hello, World!";
        response.set_body(body_text.as_bytes().to_vec());

        assert_eq!(response.body, body_text.as_bytes());
        assert_eq!(response.size, body_text.len());
    }

    #[test]
    fn test_http_response_body_as_string() {
        let mut response = HttpResponse::new(200, "OK".to_string());

        let body_text = "Hello, World!";
        response.set_body(body_text.as_bytes().to_vec());

        assert_eq!(response.body_as_string().unwrap(), body_text);

        // Test with invalid UTF-8
        response.set_body(vec![0xFF, 0xFE, 0xFD]);
        assert!(response.body_as_string().is_err());
    }

    #[test]
    fn test_http_response_content_type() {
        let mut response = HttpResponse::new(200, "OK".to_string());

        assert_eq!(response.content_type(), None);

        response.add_header("Content-Type".to_string(), "application/json".to_string());
        assert_eq!(response.content_type(), Some("application/json"));

        // Test case-insensitive lookup
        response.headers.clear();
        response.add_header("content-type".to_string(), "text/html".to_string());
        assert_eq!(response.content_type(), Some("text/html"));
    }

    #[test]
    fn test_serialization() {
        let response = HttpResponse::new(200, "OK".to_string());

        // Test that serialization works
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("200"));
        assert!(json.contains("OK"));

        // Test deserialization
        let deserialized: HttpResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status_code, response.status_code);
        assert_eq!(deserialized.status_text, response.status_text);
    }

    #[test]
    fn test_response_size_calculation() {
        let mut response = HttpResponse::new(200, "OK".to_string());
        response.add_header("Content-Type".to_string(), "text/plain".to_string());
        response.add_header("Content-Length".to_string(), "13".to_string());

        let body = "Hello, World!";
        response.set_body(body.as_bytes().to_vec());

        // Size should include body + headers
        assert!(response.size > body.len());
    }
}
