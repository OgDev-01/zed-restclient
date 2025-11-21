//! Error types for HTTP request parsing.
//!
//! This module defines error types that can occur during parsing of `.http` and `.rest` files.

use std::fmt;

/// Errors that can occur during HTTP request parsing.
///
/// Each error variant includes contextual information to help users locate
/// and fix syntax errors in their request files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Invalid or unsupported HTTP method.
    ///
    /// Contains the invalid method string and the line number where it occurred.
    InvalidMethod {
        /// The invalid method string that was encountered
        method: String,
        /// Line number in the source file (1-based)
        line: usize,
    },

    /// Invalid URL format.
    ///
    /// Contains the invalid URL and the line number where it occurred.
    InvalidUrl {
        /// The invalid URL string that was encountered
        url: String,
        /// Line number in the source file (1-based)
        line: usize,
    },

    /// Invalid header format.
    ///
    /// Headers must be in the format "Name: Value".
    InvalidHeader {
        /// The invalid header line that was encountered
        header: String,
        /// Line number in the source file (1-based)
        line: usize,
    },

    /// Missing URL in request line.
    ///
    /// A request line must contain at least a method and URL.
    MissingUrl {
        /// Line number in the source file (1-based)
        line: usize,
    },

    /// Empty request block.
    ///
    /// A request block was found but contains no valid request line.
    EmptyRequest {
        /// Line number where the empty block starts (1-based)
        line: usize,
    },

    /// Invalid HTTP version format.
    ///
    /// HTTP version must be in the format "HTTP/x.x".
    InvalidHttpVersion {
        /// The invalid version string
        version: String,
        /// Line number in the source file (1-based)
        line: usize,
    },
}

impl ParseError {
    /// Returns the line number associated with this error.
    pub fn line(&self) -> usize {
        match self {
            ParseError::InvalidMethod { line, .. } => *line,
            ParseError::InvalidUrl { line, .. } => *line,
            ParseError::InvalidHeader { line, .. } => *line,
            ParseError::MissingUrl { line } => *line,
            ParseError::EmptyRequest { line } => *line,
            ParseError::InvalidHttpVersion { line, .. } => *line,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidMethod { method, line } => {
                write!(
                    f,
                    "Invalid HTTP method '{}' at line {}. Expected one of: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, CONNECT",
                    method, line
                )
            }
            ParseError::InvalidUrl { url, line } => {
                write!(
                    f,
                    "Invalid URL '{}' at line {}. URL must start with http:// or https://",
                    url, line
                )
            }
            ParseError::InvalidHeader { header, line } => {
                write!(
                    f,
                    "Invalid header format '{}' at line {}. Expected format: 'Header-Name: value'",
                    header, line
                )
            }
            ParseError::MissingUrl { line } => {
                write!(
                    f,
                    "Missing URL in request line at line {}. Expected format: 'METHOD URL [HTTP/VERSION]'",
                    line
                )
            }
            ParseError::EmptyRequest { line } => {
                write!(f, "Empty request block at line {}", line)
            }
            ParseError::InvalidHttpVersion { version, line } => {
                write!(
                    f,
                    "Invalid HTTP version '{}' at line {}. Expected format: HTTP/1.1 or HTTP/2",
                    version, line
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_line() {
        let err = ParseError::InvalidMethod {
            method: "INVALID".to_string(),
            line: 5,
        };
        assert_eq!(err.line(), 5);

        let err = ParseError::MissingUrl { line: 10 };
        assert_eq!(err.line(), 10);
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::InvalidMethod {
            method: "INVALID".to_string(),
            line: 5,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid HTTP method"));
        assert!(msg.contains("INVALID"));
        assert!(msg.contains("line 5"));

        let err = ParseError::MissingUrl { line: 3 };
        let msg = format!("{}", err);
        assert!(msg.contains("Missing URL"));
        assert!(msg.contains("line 3"));
    }

    #[test]
    fn test_parse_error_equality() {
        let err1 = ParseError::InvalidMethod {
            method: "INVALID".to_string(),
            line: 5,
        };
        let err2 = ParseError::InvalidMethod {
            method: "INVALID".to_string(),
            line: 5,
        };
        assert_eq!(err1, err2);

        let err3 = ParseError::InvalidMethod {
            method: "OTHER".to_string(),
            line: 5,
        };
        assert_ne!(err1, err3);
    }
}
