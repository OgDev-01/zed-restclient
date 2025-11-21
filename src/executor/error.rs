//! HTTP request execution error types.
//!
//! This module defines error types that can occur during HTTP request execution,
//! including network errors, timeouts, and protocol issues.

use std::fmt;

/// Errors that can occur during HTTP request execution.
///
/// Provides detailed error information to help users diagnose issues
/// with their HTTP requests.
#[derive(Debug)]
pub enum RequestError {
    /// Network error occurred during request execution.
    ///
    /// This includes connection failures, DNS resolution errors,
    /// and other network-level issues.
    NetworkError(String),

    /// Request timed out before completion.
    ///
    /// Occurs when the request takes longer than the configured timeout duration.
    Timeout,

    /// Invalid URL provided in the request.
    ///
    /// The URL could not be parsed or is malformed.
    InvalidUrl(String),

    /// TLS/SSL error occurred during HTTPS connection.
    ///
    /// This includes certificate validation errors, handshake failures,
    /// and other TLS-related issues.
    TlsError(String),

    /// HTTP protocol error.
    ///
    /// Issues with the HTTP protocol itself, such as invalid headers
    /// or malformed responses.
    ProtocolError(String),

    /// Request building error.
    ///
    /// Errors that occur when constructing the HTTP request from
    /// the parsed request data.
    BuildError(String),

    /// Unsupported protocol.
    ///
    /// Only HTTP and HTTPS protocols are supported in the MVP.
    UnsupportedProtocol(String),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            RequestError::Timeout => write!(f, "Request timed out"),
            RequestError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            RequestError::TlsError(msg) => write!(f, "TLS/SSL error: {}", msg),
            RequestError::ProtocolError(msg) => write!(f, "HTTP protocol error: {}", msg),
            RequestError::BuildError(msg) => write!(f, "Request build error: {}", msg),
            RequestError::UnsupportedProtocol(protocol) => {
                write!(f, "Unsupported protocol: {}", protocol)
            }
        }
    }
}

impl std::error::Error for RequestError {}

/// Convert reqwest errors to RequestError.
///
/// Maps reqwest's error types to our custom error variants for consistent
/// error handling throughout the application.
impl From<reqwest::Error> for RequestError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            RequestError::Timeout
        } else if err.is_connect() || err.is_request() {
            RequestError::NetworkError(err.to_string())
        } else if err.is_builder() {
            RequestError::BuildError(err.to_string())
        } else if err.to_string().contains("certificate")
            || err.to_string().contains("TLS")
            || err.to_string().contains("SSL")
        {
            RequestError::TlsError(err.to_string())
        } else {
            RequestError::NetworkError(err.to_string())
        }
    }
}

/// Convert URL parsing errors to RequestError.
impl From<url::ParseError> for RequestError {
    fn from(err: url::ParseError) -> Self {
        RequestError::InvalidUrl(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let network_err = RequestError::NetworkError("Connection refused".to_string());
        assert_eq!(
            format!("{}", network_err),
            "Network error: Connection refused"
        );

        let timeout_err = RequestError::Timeout;
        assert_eq!(format!("{}", timeout_err), "Request timed out");

        let invalid_url_err = RequestError::InvalidUrl("not a url".to_string());
        assert_eq!(format!("{}", invalid_url_err), "Invalid URL: not a url");

        let tls_err = RequestError::TlsError("Certificate invalid".to_string());
        assert_eq!(format!("{}", tls_err), "TLS/SSL error: Certificate invalid");
    }

    #[test]
    fn test_error_is_error_trait() {
        let err: &dyn std::error::Error = &RequestError::Timeout;
        assert_eq!(format!("{}", err), "Request timed out");
    }
}
