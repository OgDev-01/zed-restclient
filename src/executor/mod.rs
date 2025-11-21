//! HTTP request executor.
//!
//! This module provides functionality to execute HTTP requests using the reqwest
//! library, with support for timeouts, timing measurements, and comprehensive
//! error handling.

pub mod config;
pub mod error;

pub use config::ExecutionConfig;
pub use error::RequestError;

use crate::models::request::{HttpMethod, HttpRequest};
use crate::models::response::{HttpResponse, RequestTiming};
use std::time::{Duration, Instant};

/// Executes an HTTP request and returns the response.
///
/// This is the main entry point for executing HTTP requests. It builds a reqwest
/// request from the HttpRequest model, executes it with the configured timeout,
/// measures timing, and captures the complete response.
///
/// # Arguments
///
/// * `request` - The HTTP request to execute
/// * `config` - Execution configuration (timeout, etc.)
///
/// # Returns
///
/// `Ok(HttpResponse)` on success, or `Err(RequestError)` if the request fails.
///
/// # Examples
///
/// ```no_run
/// use rest_client::executor::{execute_request, ExecutionConfig};
/// use rest_client::models::request::{HttpRequest, HttpMethod};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = HttpRequest::new(
///     "test-1".to_string(),
///     HttpMethod::GET,
///     "https://httpbin.org/get".to_string(),
/// );
///
/// let config = ExecutionConfig::default();
/// let response = execute_request(&request, &config).await?;
///
/// println!("Status: {}", response.status_code);
/// # Ok(())
/// # }
/// ```
pub async fn execute_request(
    request: &HttpRequest,
    config: &ExecutionConfig,
) -> Result<HttpResponse, RequestError> {
    // Start timing the entire request
    let start_time = Instant::now();

    // Validate URL and check protocol
    validate_url(&request.url)?;

    // Build the reqwest client with default configuration
    let client = reqwest::Client::builder()
        .timeout(config.timeout_duration())
        .build()
        .map_err(|e| RequestError::BuildError(e.to_string()))?;

    // Build the request
    let mut req_builder = match request.method {
        HttpMethod::GET => client.get(&request.url),
        HttpMethod::POST => client.post(&request.url),
        HttpMethod::PUT => client.put(&request.url),
        HttpMethod::DELETE => client.delete(&request.url),
        HttpMethod::PATCH => client.patch(&request.url),
        HttpMethod::HEAD => client.head(&request.url),
        HttpMethod::OPTIONS => client.request(reqwest::Method::OPTIONS, &request.url),
        HttpMethod::TRACE => client.request(reqwest::Method::TRACE, &request.url),
        HttpMethod::CONNECT => client.request(reqwest::Method::CONNECT, &request.url),
    };

    // Add headers
    for (name, value) in &request.headers {
        req_builder = req_builder.header(name, value);
    }

    // Add body if present
    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    // Build the final request
    let req = req_builder
        .build()
        .map_err(|e| RequestError::BuildError(e.to_string()))?;

    // Execute the request with timeout
    let response = tokio::time::timeout(config.timeout_duration(), client.execute(req))
        .await
        .map_err(|_| RequestError::Timeout)??;

    // Measure total duration
    let total_duration = start_time.elapsed();

    // Extract status information
    let status_code = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    // Extract headers
    let mut headers = std::collections::HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // Read response body
    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| RequestError::NetworkError(e.to_string()))?
        .to_vec();

    // Calculate response size (headers + body)
    let headers_size: usize = headers
        .iter()
        .map(|(k, v)| k.len() + v.len() + 4) // +4 for ": " and "\r\n"
        .sum();
    let total_size = headers_size + body_bytes.len();

    // Create basic timing information
    // For MVP, we only measure total duration
    // Detailed timing (DNS, TCP, TLS breakdown) will be added in Phase 3
    let timing = RequestTiming {
        dns_lookup: Duration::from_secs(0),
        tcp_connection: Duration::from_secs(0),
        tls_handshake: if request.url.starts_with("https://") {
            Some(Duration::from_secs(0))
        } else {
            None
        },
        first_byte: Duration::from_secs(0),
        download: Duration::from_secs(0),
    };

    // Build and return the HttpResponse
    let mut http_response = HttpResponse::new(status_code, status_text);
    http_response.headers = headers;
    http_response.body = body_bytes;
    http_response.duration = total_duration;
    http_response.timing = timing;
    http_response.size = total_size;

    Ok(http_response)
}

/// Validates that the URL is well-formed and uses a supported protocol.
///
/// # Arguments
///
/// * `url` - The URL string to validate
///
/// # Returns
///
/// `Ok(())` if the URL is valid, or `Err(RequestError)` if invalid.
fn validate_url(url: &str) -> Result<(), RequestError> {
    // Parse the URL to ensure it's well-formed
    let parsed = url::Url::parse(url).map_err(|e| RequestError::InvalidUrl(e.to_string()))?;

    // Check that the protocol is HTTP or HTTPS
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(RequestError::UnsupportedProtocol(format!(
            "Only HTTP and HTTPS are supported, got: {}",
            scheme
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::HttpMethod;

    #[test]
    fn test_validate_url_valid_http() {
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("http://example.com/path").is_ok());
        assert!(validate_url("http://example.com:8080").is_ok());
    }

    #[test]
    fn test_validate_url_valid_https() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("https://api.example.com/v1/users").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("not a url").is_err());
        assert!(validate_url("").is_err());
        assert!(validate_url("://missing-scheme").is_err());
    }

    #[test]
    fn test_validate_url_unsupported_protocol() {
        let result = validate_url("ftp://example.com");
        assert!(result.is_err());
        match result {
            Err(RequestError::UnsupportedProtocol(msg)) => {
                assert!(msg.contains("ftp"));
            }
            _ => panic!("Expected UnsupportedProtocol error"),
        }
    }

    #[tokio::test]
    async fn test_execute_request_get_success() {
        let request = HttpRequest::new(
            "test-1".to_string(),
            HttpMethod::GET,
            "https://httpbin.org/get".to_string(),
        );

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status_code, 200);
        assert!(response.is_success());
        assert!(response.duration.as_secs() < 30);
        assert!(response.size > 0);
    }

    #[tokio::test]
    async fn test_execute_request_post_with_json() {
        let mut request = HttpRequest::new(
            "test-2".to_string(),
            HttpMethod::POST,
            "https://httpbin.org/post".to_string(),
        );

        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"key": "value"}"#.to_string());

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status_code, 200);
        assert!(response.size > 0);
    }

    #[tokio::test]
    async fn test_execute_request_404() {
        let request = HttpRequest::new(
            "test-3".to_string(),
            HttpMethod::GET,
            "https://httpbin.org/status/404".to_string(),
        );

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status_code, 404);
        assert!(response.is_client_error());
    }

    #[tokio::test]
    async fn test_execute_request_invalid_url() {
        let request = HttpRequest::new(
            "test-4".to_string(),
            HttpMethod::GET,
            "not a valid url".to_string(),
        );

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_err());
        match response {
            Err(RequestError::InvalidUrl(_)) => {}
            _ => panic!("Expected InvalidUrl error"),
        }
    }

    #[tokio::test]
    async fn test_execute_request_timeout() {
        let request = HttpRequest::new(
            "test-5".to_string(),
            HttpMethod::GET,
            "https://httpbin.org/delay/5".to_string(),
        );

        // Set very short timeout
        let config = ExecutionConfig::new(1);
        let response = execute_request(&request, &config).await;

        assert!(response.is_err());
        match response {
            Err(RequestError::Timeout) => {}
            other => panic!("Expected Timeout error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_execute_request_with_headers() {
        let mut request = HttpRequest::new(
            "test-6".to_string(),
            HttpMethod::GET,
            "https://httpbin.org/headers".to_string(),
        );

        request.add_header("X-Custom-Header".to_string(), "test-value".to_string());
        request.add_header("User-Agent".to_string(), "REST-Client/1.0".to_string());

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status_code, 200);

        // The response should contain our headers echoed back
        let body_str = response.body_as_string().unwrap();
        assert!(body_str.contains("X-Custom-Header") || body_str.contains("test-value"));
    }

    #[tokio::test]
    async fn test_execute_request_unsupported_protocol() {
        let request = HttpRequest::new(
            "test-7".to_string(),
            HttpMethod::GET,
            "ftp://example.com/file.txt".to_string(),
        );

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_err());
        match response {
            Err(RequestError::UnsupportedProtocol(msg)) => {
                assert!(msg.contains("ftp"));
            }
            _ => panic!("Expected UnsupportedProtocol error"),
        }
    }

    #[tokio::test]
    async fn test_execute_request_put() {
        let mut request = HttpRequest::new(
            "test-8".to_string(),
            HttpMethod::PUT,
            "https://httpbin.org/put".to_string(),
        );

        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request.set_body(r#"{"updated": true}"#.to_string());

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_execute_request_delete() {
        let request = HttpRequest::new(
            "test-9".to_string(),
            HttpMethod::DELETE,
            "https://httpbin.org/delete".to_string(),
        );

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_response_timing_populated() {
        let request = HttpRequest::new(
            "test-10".to_string(),
            HttpMethod::GET,
            "https://httpbin.org/get".to_string(),
        );

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await.unwrap();

        // Total duration should be non-zero
        assert!(response.duration.as_millis() > 0);

        // TLS handshake should be present for HTTPS
        assert!(response.timing.tls_handshake.is_some());
    }

    #[tokio::test]
    async fn test_response_size_calculated() {
        let request = HttpRequest::new(
            "test-11".to_string(),
            HttpMethod::GET,
            "https://httpbin.org/get".to_string(),
        );

        let config = ExecutionConfig::default();
        let response = execute_request(&request, &config).await.unwrap();

        // Size should include both headers and body
        assert!(response.size > 0);
        assert!(response.size >= response.body.len());
    }
}
