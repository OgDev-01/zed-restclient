//! Native HTTP executor using reqwest for LSP server binary.
//!
//! This module provides HTTP request execution using the native reqwest library,
//! which works in non-WASM environments (i.e., the LSP server binary).
//!
//! This is separate from the WASM executor which uses zed_extension_api::http_client.

use crate::executor::error::RequestError;
use crate::executor::timing::TimingCheckpoints;
use crate::models::request::{HttpMethod, HttpRequest};
use crate::models::response::HttpResponse;
use std::time::Instant;

/// Execute an HTTP request using reqwest (native client)
///
/// This function is only available when the "lsp" feature is enabled,
/// as it uses reqwest which doesn't compile to WASM.
pub async fn execute_request_native(request: &HttpRequest) -> Result<HttpResponse, RequestError> {
    let start_time = Instant::now();
    let is_https = request.url.starts_with("https://");
    let mut timing_checkpoints = TimingCheckpoints::new(is_https);

    // Convert our HttpMethod to reqwest's Method
    let method = match request.method {
        HttpMethod::GET => reqwest::Method::GET,
        HttpMethod::POST => reqwest::Method::POST,
        HttpMethod::PUT => reqwest::Method::PUT,
        HttpMethod::DELETE => reqwest::Method::DELETE,
        HttpMethod::PATCH => reqwest::Method::PATCH,
        HttpMethod::HEAD => reqwest::Method::HEAD,
        HttpMethod::OPTIONS => reqwest::Method::OPTIONS,
        HttpMethod::TRACE => reqwest::Method::TRACE,
        HttpMethod::CONNECT => reqwest::Method::CONNECT,
    };

    // Mark client start
    timing_checkpoints.mark_client_start();

    // Build the request
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| RequestError::BuildError(e.to_string()))?;

    let mut req_builder = client.request(method, &request.url);

    // Add headers
    for (name, value) in &request.headers {
        req_builder = req_builder.header(name, value);
    }

    // Add body if present
    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    // Mark request sent
    timing_checkpoints.mark_request_sent();

    // Execute the request
    let response = req_builder.send().await.map_err(|e| {
        if e.is_timeout() {
            RequestError::Timeout
        } else if e.is_connect() {
            RequestError::NetworkError(format!("Connection failed: {}", e))
        } else {
            RequestError::NetworkError(e.to_string())
        }
    })?;

    // Mark first byte received
    timing_checkpoints.mark_first_byte_received();

    // Extract response details
    let status_code = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    // Extract headers
    let mut response_headers = std::collections::HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value_str) = value.to_str() {
            response_headers.insert(name.as_str().to_string(), value_str.to_string());
        }
    }

    // Read response body
    let body = response
        .bytes()
        .await
        .map_err(|e| RequestError::NetworkError(e.to_string()))?
        .to_vec();

    // Mark response complete
    timing_checkpoints.mark_response_complete();

    // Convert timing checkpoints to RequestTiming
    let timing = timing_checkpoints.to_request_timing();
    let total_duration = timing.total();
    let size = body.len()
        + response_headers
            .iter()
            .fold(0, |acc, (k, v)| acc + k.len() + v.len());

    Ok(HttpResponse {
        status_code,
        status_text,
        headers: response_headers,
        body,
        duration: total_duration,
        timing,
        size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::HttpRequest;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_simple_get_request() {
        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "https://httpbin.org/get".to_string(),
            headers: HashMap::new(),
            body: None,
            line_number: 0,
        };

        let result = execute_request_native(&request).await;
        assert!(result.is_ok(), "Request should succeed");

        let response = result.unwrap();
        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_request_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "RestClient/1.0".to_string());
        headers.insert("Accept".to_string(), "application/json".to_string());

        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "https://httpbin.org/headers".to_string(),
            headers,
            body: None,
            line_number: 0,
        };

        let result = execute_request_native(&request).await;
        assert!(result.is_ok(), "Request should succeed");

        let response = result.unwrap();
        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_post_request_with_body() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let body = r#"{"name": "test", "value": 123}"#.to_string();

        let request = HttpRequest {
            method: HttpMethod::POST,
            url: "https://httpbin.org/post".to_string(),
            headers,
            body: Some(body),
            line_number: 0,
        };

        let result = execute_request_native(&request).await;
        assert!(result.is_ok(), "Request should succeed");

        let response = result.unwrap();
        assert_eq!(response.status_code, 200);
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "not-a-valid-url".to_string(),
            headers: HashMap::new(),
            body: None,
            line_number: 0,
        };

        let result = execute_request_native(&request).await;
        assert!(result.is_err(), "Invalid URL should fail");
    }

    #[tokio::test]
    async fn test_404_response() {
        let request = HttpRequest {
            method: HttpMethod::GET,
            url: "https://httpbin.org/status/404".to_string(),
            headers: HashMap::new(),
            body: None,
            line_number: 0,
        };

        let result = execute_request_native(&request).await;
        assert!(result.is_ok(), "Request should complete even with 404");

        let response = result.unwrap();
        assert_eq!(response.status_code, 404);
    }
}
