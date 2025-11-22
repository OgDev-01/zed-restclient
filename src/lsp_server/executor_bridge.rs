//! Executor Bridge for REST Client LSP Server
//!
//! This module bridges the LSP server with the existing parser and executor
//! modules, enabling execution of HTTP requests from .http file content.

use crate::environment::Environment;
#[cfg(feature = "lsp")]
use crate::executor::execute_request_native;
use crate::executor::ExecutionConfig;
use crate::models::{HttpRequest, HttpResponse};
use crate::parser::{error::ParseError, parse_file};
use crate::variables::substitution::VariableContext;
use std::collections::HashMap;
use std::path::PathBuf;

/// Error types for executor bridge operations
#[derive(Debug)]
pub enum BridgeError {
    /// Error during parsing of the document
    ParseError(ParseError),
    /// No request found at the specified line
    NoRequestAtLine { line: usize },
    /// Error during request execution
    ExecutionError(String),
    /// Error during variable substitution
    VariableError(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::ParseError(e) => write!(f, "Parse error: {}", e),
            BridgeError::NoRequestAtLine { line } => {
                write!(f, "No request found at line {}", line)
            }
            BridgeError::ExecutionError(e) => write!(f, "Execution error: {}", e),
            BridgeError::VariableError(e) => write!(f, "Variable error: {}", e),
        }
    }
}

impl std::error::Error for BridgeError {}

impl From<ParseError> for BridgeError {
    fn from(err: ParseError) -> Self {
        BridgeError::ParseError(err)
    }
}

/// Bridge between LSP server and request execution pipeline
///
/// Coordinates parsing, variable resolution, and HTTP request execution
/// for .http files in the LSP context.
#[derive(Debug, Clone)]
pub struct ExecutorBridge {
    /// Execution configuration
    config: ExecutionConfig,
}

impl ExecutorBridge {
    /// Creates a new ExecutorBridge with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use rest_client::lsp_server::executor_bridge::ExecutorBridge;
    ///
    /// let bridge = ExecutorBridge::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: ExecutionConfig::default(),
        }
    }

    /// Creates a new ExecutorBridge with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Custom execution configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use rest_client::lsp_server::executor_bridge::ExecutorBridge;
    /// use rest_client::executor::ExecutionConfig;
    ///
    /// let config = ExecutionConfig::default();
    /// let bridge = ExecutorBridge::with_config(config);
    /// ```
    pub fn with_config(config: ExecutionConfig) -> Self {
        Self { config }
    }

    /// Executes the HTTP request at the specified line in a document
    ///
    /// This method:
    /// 1. Parses the entire document to extract all requests
    /// 2. Finds the request that contains the specified line
    /// 3. Resolves variables using the provided environment
    /// 4. Executes the request and returns the response
    ///
    /// # Arguments
    ///
    /// * `document` - The full content of the .http file
    /// * `line` - The line number (1-based) where the cursor is positioned
    /// * `env` - Optional environment for variable resolution
    ///
    /// # Returns
    ///
    /// Returns `Ok(HttpResponse)` on success, or `Err(BridgeError)` on failure
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rest_client::lsp_server::executor_bridge::ExecutorBridge;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let bridge = ExecutorBridge::new();
    /// let document = "GET https://api.example.com/users\n";
    /// let response = bridge.execute_request_at_line(document, 1, None).await?;
    /// println!("Status: {}", response.status_code);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_request_at_line(
        &self,
        document: &str,
        line: usize,
        env: Option<Environment>,
    ) -> Result<HttpResponse, BridgeError> {
        // Parse the document to get all requests
        let file_path = PathBuf::from("untitled.http");
        let requests = parse_file(document, &file_path)?;

        // Find the request that contains the specified line
        let request = self.find_request_at_line(&requests, line)?;

        // Clone the request for variable substitution
        let mut resolved_request = request.clone();

        // Create variable context and resolve variables
        let context = self.create_variable_context(env);
        self.resolve_request_variables(&mut resolved_request, &context)?;

        // Execute the request using native HTTP client (reqwest)
        // This is available because we're in the LSP server with the "lsp" feature
        #[cfg(feature = "lsp")]
        let response = execute_request_native(&resolved_request)
            .await
            .map_err(|e| BridgeError::ExecutionError(e.to_string()))?;

        // Fallback for non-LSP builds (shouldn't happen in practice)
        #[cfg(not(feature = "lsp"))]
        let response = {
            return Err(BridgeError::ExecutionError(
                "HTTP execution requires the 'lsp' feature to be enabled".to_string(),
            ));
        };

        Ok(response)
    }

    /// Finds the request that contains the specified line number
    ///
    /// Requests can span multiple lines (method, headers, body), so we need
    /// to find which request block contains the cursor position.
    fn find_request_at_line<'a>(
        &self,
        requests: &'a [HttpRequest],
        line: usize,
    ) -> Result<&'a HttpRequest, BridgeError> {
        // For each request, check if the line falls within its range
        for (i, request) in requests.iter().enumerate() {
            let request_start = request.line_number;

            // Calculate request end line
            // If there's a next request, the current request ends before it
            // Otherwise, it extends to the end of the document
            let request_end = if i + 1 < requests.len() {
                requests[i + 1].line_number - 1
            } else {
                usize::MAX // Last request extends to end of file
            };

            if line >= request_start && line <= request_end {
                return Ok(request);
            }
        }

        Err(BridgeError::NoRequestAtLine { line })
    }

    /// Creates a variable context for resolving variables in requests
    fn create_variable_context(&self, env: Option<Environment>) -> VariableContext {
        VariableContext {
            environment: env,
            shared_variables: HashMap::new(),
            file_variables: HashMap::new(),
            request_variables: HashMap::new(),
            workspace_path: PathBuf::from("."),
        }
    }

    /// Resolves variables in a request using the variable context
    fn resolve_request_variables(
        &self,
        request: &mut HttpRequest,
        context: &VariableContext,
    ) -> Result<(), BridgeError> {
        use crate::variables::substitution::substitute_variables;

        // Resolve URL variables
        request.url = substitute_variables(&request.url, context)
            .map_err(|e| BridgeError::VariableError(e.to_string()))?;

        // Resolve header variables
        let mut resolved_headers = HashMap::new();
        for (key, value) in &request.headers {
            let resolved_key = substitute_variables(key, context)
                .map_err(|e| BridgeError::VariableError(e.to_string()))?;
            let resolved_value = substitute_variables(value, context)
                .map_err(|e| BridgeError::VariableError(e.to_string()))?;
            resolved_headers.insert(resolved_key, resolved_value);
        }
        request.headers = resolved_headers;

        // Resolve body variables if present
        if let Some(body) = &request.body {
            request.body = Some(
                substitute_variables(body, context)
                    .map_err(|e| BridgeError::VariableError(e.to_string()))?,
            );
        }

        Ok(())
    }

    /// Formats an HTTP response as a human-readable string
    ///
    /// The formatted output includes:
    /// - Status line (HTTP version, status code, status text)
    /// - Response headers (one per line)
    /// - Blank line separator
    /// - Response body
    ///
    /// # Arguments
    ///
    /// * `response` - The HTTP response to format
    ///
    /// # Returns
    ///
    /// A formatted string representation of the response
    ///
    /// # Examples
    ///
    /// ```
    /// use rest_client::lsp_server::executor_bridge::ExecutorBridge;
    /// use rest_client::models::HttpResponse;
    /// use std::collections::HashMap;
    ///
    /// let bridge = ExecutorBridge::new();
    /// let mut headers = HashMap::new();
    /// headers.insert("Content-Type".to_string(), "application/json".to_string());
    ///
    /// let response = HttpResponse {
    ///     status_code: 200,
    ///     status_text: "OK".to_string(),
    ///     headers,
    ///     body: r#"{"message": "success"}"#.to_string(),
    ///     timing: None,
    /// };
    ///
    /// let formatted = ExecutorBridge::format_response(&response);
    /// assert!(formatted.contains("HTTP/1.1 200 OK"));
    /// ```
    pub fn format_response(response: &HttpResponse) -> String {
        let mut output = String::new();

        // Status line
        output.push_str(&format!(
            "HTTP/1.1 {} {}\n",
            response.status_code, response.status_text
        ));

        // Headers
        for (key, value) in &response.headers {
            output.push_str(&format!("{}: {}\n", key, value));
        }

        // Blank line separator
        output.push('\n');

        // Body - convert from Vec<u8> to String
        let body_str = String::from_utf8_lossy(&response.body);
        output.push_str(&body_str);

        // Add timing information
        let timing = &response.timing;
        output.push_str("\n\n");
        output.push_str(&format!("--- Timing Information ---\n"));
        output.push_str(&format!("Total: {} ms\n", timing.total().as_millis()));
        output.push_str(&format!(
            "DNS Lookup: {} ms\n",
            timing.dns_lookup.as_millis()
        ));
        output.push_str(&format!(
            "TCP Connection: {} ms\n",
            timing.tcp_connection.as_millis()
        ));

        if let Some(tls) = timing.tls_handshake {
            output.push_str(&format!("TLS Handshake: {} ms\n", tls.as_millis()));
        }

        output.push_str(&format!(
            "Time to First Byte: {} ms\n",
            timing.first_byte.as_millis()
        ));
        output.push_str(&format!("Download: {} ms\n", timing.download.as_millis()));

        output
    }

    /// Formats a response with pretty-printed JSON body if applicable
    ///
    /// If the response has a JSON content type and valid JSON body,
    /// it will be pretty-printed. Otherwise, returns standard formatting.
    ///
    /// # Arguments
    ///
    /// * `response` - The HTTP response to format
    ///
    /// # Returns
    ///
    /// A formatted string representation of the response
    pub fn format_response_pretty(response: &HttpResponse) -> String {
        let mut output = String::new();

        // Status line
        output.push_str(&format!(
            "HTTP/1.1 {} {}\n",
            response.status_code, response.status_text
        ));

        // Headers
        for (key, value) in &response.headers {
            output.push_str(&format!("{}: {}\n", key, value));
        }

        // Blank line separator
        output.push('\n');

        // Try to pretty-print JSON body
        let is_json = response
            .headers
            .iter()
            .any(|(k, v)| k.to_lowercase() == "content-type" && v.contains("application/json"));

        let body_str = String::from_utf8_lossy(&response.body);

        if is_json {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&body_str) {
                if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                    output.push_str(&pretty_json);
                } else {
                    output.push_str(&body_str);
                }
            } else {
                output.push_str(&body_str);
            }
        } else {
            output.push_str(&body_str);
        }

        // Add timing information
        let timing = &response.timing;
        output.push_str("\n\n");
        output.push_str(&format!("--- Timing Information ---\n"));
        output.push_str(&format!("Total: {} ms\n", timing.total().as_millis()));

        output
    }
}

impl Default for ExecutorBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::HttpMethod;

    #[test]
    fn test_new_bridge() {
        let bridge = ExecutorBridge::new();
        // Bridge should be created successfully
        assert!(std::mem::size_of_val(&bridge) > 0);
    }

    #[test]
    fn test_default_bridge() {
        let bridge = ExecutorBridge::default();
        assert!(std::mem::size_of_val(&bridge) > 0);
    }

    #[test]
    fn test_find_request_at_line_single_request() {
        let bridge = ExecutorBridge::new();
        let request = HttpRequest {
            id: "test-1".to_string(),
            method: HttpMethod::GET,
            url: "https://example.com".to_string(),
            http_version: None,
            headers: HashMap::new(),
            body: None,
            line_number: 1,
            file_path: PathBuf::from("test.http"),
        };

        let requests = vec![request];

        // Line 1 should find the request
        let found = bridge.find_request_at_line(&requests, 1);
        assert!(found.is_ok());

        // Line 5 should also find the request (it extends to end of file)
        let found = bridge.find_request_at_line(&requests, 5);
        assert!(found.is_ok());

        // Line 0 should not find anything
        let found = bridge.find_request_at_line(&requests, 0);
        assert!(found.is_err());
    }

    #[test]
    fn test_find_request_at_line_multiple_requests() {
        let bridge = ExecutorBridge::new();
        let request1 = HttpRequest {
            id: "test-1".to_string(),
            method: HttpMethod::GET,
            url: "https://example.com/1".to_string(),
            http_version: None,
            headers: HashMap::new(),
            body: None,
            line_number: 1,
            file_path: PathBuf::from("test.http"),
        };

        let request2 = HttpRequest {
            id: "test-2".to_string(),
            method: HttpMethod::POST,
            url: "https://example.com/2".to_string(),
            http_version: None,
            headers: HashMap::new(),
            body: Some("data".to_string()),
            line_number: 10,
            file_path: PathBuf::from("test.http"),
        };

        let requests = vec![request1, request2];

        // Line 1 should find first request
        let found = bridge.find_request_at_line(&requests, 1).unwrap();
        assert_eq!(found.url, "https://example.com/1");

        // Line 5 should find first request
        let found = bridge.find_request_at_line(&requests, 5).unwrap();
        assert_eq!(found.url, "https://example.com/1");

        // Line 9 should find first request
        let found = bridge.find_request_at_line(&requests, 9).unwrap();
        assert_eq!(found.url, "https://example.com/1");

        // Line 10 should find second request
        let found = bridge.find_request_at_line(&requests, 10).unwrap();
        assert_eq!(found.url, "https://example.com/2");

        // Line 15 should find second request
        let found = bridge.find_request_at_line(&requests, 15).unwrap();
        assert_eq!(found.url, "https://example.com/2");
    }

    #[test]
    fn test_format_response_basic() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/plain".to_string());
        headers.insert("Content-Length".to_string(), "13".to_string());

        let response = HttpResponse {
            status_code: 200,
            status_text: "OK".to_string(),
            headers,
            body: b"Hello, World!".to_vec(),
            duration: std::time::Duration::from_millis(100),
            timing: crate::models::RequestTiming {
                dns_lookup: std::time::Duration::from_millis(10),
                tcp_connection: std::time::Duration::from_millis(20),
                tls_handshake: None,
                first_byte: std::time::Duration::from_millis(50),
                download: std::time::Duration::from_millis(20),
            },
            size: 13,
        };

        let formatted = ExecutorBridge::format_response(&response);

        assert!(formatted.contains("HTTP/1.1 200 OK"));
        assert!(formatted.contains("Content-Type: text/plain"));
        assert!(formatted.contains("Hello, World!"));
    }

    #[test]
    fn test_format_response_with_json() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = HttpResponse {
            status_code: 201,
            status_text: "Created".to_string(),
            headers,
            body: r#"{"id":1,"name":"Test"}"#.as_bytes().to_vec(),
            duration: std::time::Duration::from_millis(150),
            timing: crate::models::RequestTiming {
                dns_lookup: std::time::Duration::from_millis(15),
                tcp_connection: std::time::Duration::from_millis(30),
                tls_handshake: Some(std::time::Duration::from_millis(40)),
                first_byte: std::time::Duration::from_millis(50),
                download: std::time::Duration::from_millis(15),
            },
            size: 23,
        };

        let formatted = ExecutorBridge::format_response_pretty(&response);

        assert!(formatted.contains("HTTP/1.1 201 Created"));
        assert!(formatted.contains("application/json"));
        // Pretty-printed JSON should have newlines
        assert!(formatted.contains("\"id\""));
        assert!(formatted.contains("\"name\""));
    }

    #[test]
    fn test_create_variable_context_without_env() {
        let bridge = ExecutorBridge::new();
        let context = bridge.create_variable_context(None);

        assert!(context.environment.is_none());
        assert!(context.shared_variables.is_empty());
        assert!(context.file_variables.is_empty());
        assert!(context.request_variables.is_empty());
    }

    #[test]
    fn test_resolve_request_variables_no_variables() {
        let bridge = ExecutorBridge::new();
        let context = bridge.create_variable_context(None);

        let mut request = HttpRequest {
            id: "test-1".to_string(),
            method: HttpMethod::GET,
            url: "https://example.com/api".to_string(),
            http_version: None,
            headers: HashMap::new(),
            body: None,
            line_number: 1,
            file_path: PathBuf::from("test.http"),
        };

        let result = bridge.resolve_request_variables(&mut request, &context);
        assert!(result.is_ok());
        assert_eq!(request.url, "https://example.com/api");
    }
}
