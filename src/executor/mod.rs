//! HTTP request executor.
//!
//! This module provides functionality to execute HTTP requests using the Zed
//! extension API's built-in HTTP client, which is WASM-compatible.
//!
//! **IMPORTANT LIMITATION**: The Zed HTTP client API (as of v0.7.0) does not
//! provide HTTP status codes in the response. Success is determined by whether
//! the request completes without error. This is a fundamental limitation that
//! affects the REST client's ability to distinguish between different HTTP
//! response codes (200 OK vs 404 Not Found, etc.).

pub mod cancellation;
pub mod config;
pub mod error;
pub mod timing;

// Native HTTP executor for LSP server (non-WASM)
#[cfg(feature = "lsp")]
pub mod native;

pub use cancellation::{CancelError, RequestHandle, RequestTracker, SharedRequestTracker};
pub use config::ExecutionConfig;
pub use error::RequestError;
pub use timing::{format_timing_breakdown, format_timing_compact, TimingCheckpoints};

#[cfg(feature = "lsp")]
pub use native::execute_request_native;

use crate::graphql::parser::{is_graphql_request, parse_graphql_request};
use crate::models::request::{HttpMethod, HttpRequest};
use crate::models::response::HttpResponse;
use std::sync::{Arc, Mutex};
use zed_extension_api::http_client::{self, HttpMethod as ZedHttpMethod};

/// Global request tracker for managing active requests.
/// This is lazily initialized and shared across all requests.
static GLOBAL_TRACKER: Mutex<Option<SharedRequestTracker>> = Mutex::new(None);

/// Gets or initializes the global request tracker.
fn get_global_tracker() -> SharedRequestTracker {
    let mut tracker_opt = GLOBAL_TRACKER.lock().unwrap();
    if tracker_opt.is_none() {
        *tracker_opt = Some(SharedRequestTracker::new());
    }
    tracker_opt.as_ref().unwrap().clone()
}

/// Cancels a request by its ID.
///
/// # Arguments
///
/// * `request_id` - The ID of the request to cancel
///
/// # Returns
///
/// `Ok(())` if the request was successfully cancelled, or `Err(CancelError)` if
/// the request was not found or already completed.
///
/// # Examples
///
/// ```no_run
/// use rest_client::executor::cancel_request;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// cancel_request("req-123")?;
/// # Ok(())
/// # }
/// ```
pub fn cancel_request(request_id: &str) -> Result<(), CancelError> {
    let tracker = get_global_tracker();
    tracker.cancel_request(request_id)
}

/// Cancels the most recently started request.
///
/// This is useful for implementing a "Cancel Request" command that cancels
/// the most recent active request without needing to know its ID.
///
/// # Returns
///
/// `Ok(request_id)` with the ID of the cancelled request, or `Err(CancelError)`
/// if there are no active requests.
///
/// # Examples
///
/// ```no_run
/// use rest_client::executor::cancel_most_recent_request;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let cancelled_id = cancel_most_recent_request()?;
/// println!("Cancelled request: {}", cancelled_id);
/// # Ok(())
/// # }
/// ```
pub fn cancel_most_recent_request() -> Result<String, CancelError> {
    let tracker = get_global_tracker();
    tracker.cancel_most_recent()
}

/// Gets the number of currently active requests.
///
/// # Returns
///
/// The count of active requests being tracked.
pub fn get_active_request_count() -> usize {
    let tracker = get_global_tracker();
    tracker.active_count().unwrap_or(0)
}

/// Gets a list of all active request IDs.
///
/// # Returns
///
/// A vector of request IDs for all currently active requests.
pub fn get_active_request_ids() -> Vec<String> {
    let tracker = get_global_tracker();
    tracker.active_request_ids().unwrap_or_default()
}

/// Executes an HTTP request and returns the response.
///
/// This is the main entry point for executing HTTP requests. It builds a Zed HTTP
/// request from the HttpRequest model, executes it, measures timing, and captures
/// the complete response.
///
/// **Note**: Due to limitations in the Zed HTTP client API, the status code is
/// always set to 200 for successful requests. Actual HTTP status codes are not
/// available through the current API.
///
/// # Arguments
///
/// * `request` - The HTTP request to execute
/// * `config` - Execution configuration (currently unused due to API limitations)
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
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = HttpRequest::new(
///     "test-1".to_string(),
///     HttpMethod::GET,
///     "https://httpbin.org/get".to_string(),
/// );
///
/// let config = ExecutionConfig::default();
/// let response = execute_request(&request, &config)?;
///
/// println!("Status: {}", response.status_code);
/// # Ok(())
/// # }
/// ```
pub fn execute_request(
    request: &HttpRequest,
    _config: &ExecutionConfig,
) -> Result<HttpResponse, RequestError> {
    execute_request_internal(request, _config, None)
}

/// Executes an HTTP request with cancellation support.
///
/// This function registers the request with the global tracker and allows it to
/// be cancelled via the `cancel_request` or `cancel_most_recent_request` functions.
///
/// # Arguments
///
/// * `request` - The HTTP request to execute
/// * `config` - Execution configuration (currently unused due to API limitations)
///
/// # Returns
///
/// `Ok((HttpResponse, String))` with the response and request ID on success,
/// or `Err(RequestError)` if the request fails or is cancelled.
///
/// # Examples
///
/// ```no_run
/// use rest_client::executor::{execute_request_with_cancellation, ExecutionConfig};
/// use rest_client::models::request::{HttpRequest, HttpMethod};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = HttpRequest::new(
///     "test-1".to_string(),
///     HttpMethod::GET,
///     "https://httpbin.org/get".to_string(),
/// );
///
/// let config = ExecutionConfig::default();
/// let (response, request_id) = execute_request_with_cancellation(&request, &config)?;
///
/// println!("Request ID: {}", request_id);
/// println!("Status: {}", response.status_code);
/// # Ok(())
/// # }
/// ```
pub fn execute_request_with_cancellation(
    request: &HttpRequest,
    config: &ExecutionConfig,
) -> Result<(HttpResponse, String), RequestError> {
    // Create a request handle for tracking
    let handle = RequestHandle::new();
    let request_id = handle.request_id.clone();
    let cancelled_flag = handle.cancelled.clone();

    // Register with the global tracker
    let tracker = get_global_tracker();
    tracker
        .register(handle)
        .map_err(|e| RequestError::BuildError(format!("Failed to register request: {}", e)))?;

    // Execute the request with cancellation support
    let result = execute_request_internal(request, config, Some(cancelled_flag.clone()));

    // Unregister from tracker when done
    let _ = tracker.unregister(&request_id);

    // Return the response along with the request ID
    result.map(|response| (response, request_id))
}

/// Internal implementation of execute_request with optional cancellation support.
fn execute_request_internal(
    request: &HttpRequest,
    _config: &ExecutionConfig,
    cancelled_flag: Option<Arc<Mutex<bool>>>,
) -> Result<HttpResponse, RequestError> {
    // Check if request was cancelled before starting
    if let Some(ref flag) = cancelled_flag {
        if *flag.lock().unwrap() {
            return Err(RequestError::BuildError("Request cancelled".to_string()));
        }
    }

    // Initialize timing checkpoints
    let is_https = request.url.starts_with("https://");
    let mut timing_checkpoints = TimingCheckpoints::new(is_https);

    // Validate URL and check protocol
    validate_url(&request.url)?;

    // Check cancellation again
    if let Some(ref flag) = cancelled_flag {
        if *flag.lock().unwrap() {
            return Err(RequestError::BuildError("Request cancelled".to_string()));
        }
    }

    // Process GraphQL requests
    let (processed_body, processed_headers) = if let Some(ref body) = request.body {
        let content_type = request.content_type();
        if is_graphql_request(body, content_type) {
            process_graphql_request(body, &request.headers)?
        } else {
            (request.body.clone(), request.headers.clone())
        }
    } else {
        (request.body.clone(), request.headers.clone())
    };

    // Convert our HttpMethod to Zed's HttpMethod
    let method = match request.method {
        HttpMethod::GET => ZedHttpMethod::Get,
        HttpMethod::POST => ZedHttpMethod::Post,
        HttpMethod::PUT => ZedHttpMethod::Put,
        HttpMethod::DELETE => ZedHttpMethod::Delete,
        HttpMethod::PATCH => ZedHttpMethod::Patch,
        HttpMethod::HEAD => ZedHttpMethod::Head,
        HttpMethod::OPTIONS => ZedHttpMethod::Options,
        HttpMethod::TRACE => {
            return Err(RequestError::UnsupportedMethod(
                "TRACE method is not supported by Zed HTTP client".to_string(),
            ))
        }
        HttpMethod::CONNECT => {
            return Err(RequestError::UnsupportedMethod(
                "CONNECT method is not supported by Zed HTTP client".to_string(),
            ))
        }
    };

    // Mark client start (after validation)
    timing_checkpoints.mark_client_start();

    // Build the request using Zed's HTTP client API
    let mut req_builder = http_client::HttpRequest::builder()
        .method(method)
        .url(&request.url);

    // Add headers (use processed headers for GraphQL)
    for (name, value) in &processed_headers {
        req_builder = req_builder.header(name, value);
    }

    // Add body if present (use processed body for GraphQL)
    if let Some(body) = &processed_body {
        req_builder = req_builder.body(body.as_bytes().to_vec());
    }

    // Check cancellation before building
    if let Some(ref flag) = cancelled_flag {
        if *flag.lock().unwrap() {
            return Err(RequestError::BuildError("Request cancelled".to_string()));
        }
    }

    // Build the final request
    let http_request = req_builder
        .build()
        .map_err(|e| RequestError::BuildError(e))?;

    // Check cancellation before executing
    if let Some(ref flag) = cancelled_flag {
        if *flag.lock().unwrap() {
            return Err(RequestError::BuildError("Request cancelled".to_string()));
        }
    }

    // Mark when request is about to be sent
    timing_checkpoints.mark_request_sent();

    // Execute the request
    let response = http_request
        .fetch()
        .map_err(|e| RequestError::NetworkError(e))?;

    // Mark when first byte received (response arrived)
    timing_checkpoints.mark_first_byte_received();

    // Check cancellation after execution
    if let Some(ref flag) = cancelled_flag {
        if *flag.lock().unwrap() {
            return Err(RequestError::BuildError("Request cancelled".to_string()));
        }
    }

    // Mark when response is completely received
    timing_checkpoints.mark_response_complete();

    // Convert timing checkpoints to RequestTiming
    let timing = timing_checkpoints.to_request_timing();
    let total_duration = timing.total();

    // KNOWN LIMITATION: Zed's WASM HTTP client API does not return HTTP status codes
    // The zed_extension_api::http_client module only provides headers and body.
    // As a result, we cannot distinguish between 200 OK, 201 Created, 204 No Content, etc.
    // All successful responses are reported as "200 OK (assumed)".
    //
    // Workaround options for users needing accurate status codes:
    // 1. Use the LSP server which uses reqwest and has full status code support
    // 2. Check response headers for status-related information
    // 3. Use an external HTTP client for critical status code checks
    //
    // See: https://github.com/zed-industries/zed/issues/XXXX (if tracking issue exists)
    let status_code = 200u16;
    let status_text = "OK (assumed - Zed API limitation)".to_string();

    // Extract headers from response
    let mut headers = std::collections::HashMap::new();
    for (name, value) in &response.headers {
        headers.insert(name.clone(), value.clone());
    }

    // Get response body
    let body_bytes = response.body.clone();

    // Calculate response size (headers + body)
    let headers_size: usize = headers
        .iter()
        .map(|(k, v)| k.len() + v.len() + 4) // +4 for ": " and "\r\n"
        .sum();
    let total_size = headers_size + body_bytes.len();

    // Build and return the HttpResponse
    let mut http_response = HttpResponse::new(status_code, status_text);
    http_response.headers = headers;
    http_response.body = body_bytes;
    http_response.duration = total_duration;
    http_response.timing = timing;
    http_response.size = total_size;

    Ok(http_response)
}

/// Processes a GraphQL request by converting it to JSON format for HTTP transport.
///
/// This function:
/// 1. Parses the GraphQL query and variables
/// 2. Converts them to a JSON object with {query: "...", variables: {...}}
/// 3. Sets Content-Type to application/json if not already set
///
/// # Arguments
///
/// * `body` - The request body containing GraphQL query and variables
/// * `headers` - The original request headers
///
/// # Returns
///
/// A tuple of (processed_body, processed_headers) ready for HTTP transport
fn process_graphql_request(
    body: &str,
    headers: &std::collections::HashMap<String, String>,
) -> Result<(Option<String>, std::collections::HashMap<String, String>), RequestError> {
    // Parse the GraphQL request
    let graphql_request = parse_graphql_request(body)
        .map_err(|e| RequestError::BuildError(format!("GraphQL parsing error: {}", e)))?;

    // Convert to JSON for HTTP transport
    let json_body = graphql_request.to_json().map_err(|e| {
        RequestError::BuildError(format!("Failed to serialize GraphQL request: {}", e))
    })?;

    // Ensure Content-Type is set to application/json
    let mut processed_headers = headers.clone();
    let has_content_type = processed_headers
        .keys()
        .any(|k| k.eq_ignore_ascii_case("content-type"));

    if !has_content_type {
        processed_headers.insert("Content-Type".to_string(), "application/json".to_string());
    }

    Ok((Some(json_body), processed_headers))
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

    #[test]
    fn test_global_tracker_functions() {
        // Test getting active count (should work even with no requests)
        let _count = get_active_request_count();

        // Test getting active IDs
        let _ids = get_active_request_ids();
        // Either state is valid - function should not panic
    }

    #[test]
    fn test_cancel_nonexistent_request() {
        let result = cancel_request("nonexistent-id-12345");
        assert!(result.is_err());
        assert!(matches!(result, Err(CancelError::NotFound(_))));
    }

    // Note: Integration tests that actually make HTTP requests cannot be run
    // in a standard cargo test environment because:
    // 1. They require the Zed WASM runtime
    // 2. The http_client module is only available in the WASM context
    //
    // These tests would need to be performed manually within Zed itself.
}
