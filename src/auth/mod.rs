//! HTTP authentication module.
//!
//! This module provides authentication handlers for HTTP requests, supporting
//! Basic and Bearer authentication schemes. Authentication can be detected
//! from Authorization headers or special comment directives in .http files.

pub mod basic;
pub mod bearer;

use crate::models::request::HttpRequest;
use std::fmt;

/// Authentication scheme types supported by the REST client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthScheme {
    /// HTTP Basic authentication (RFC 7617)
    Basic { username: String, password: String },
    /// Bearer token authentication (RFC 6750)
    Bearer { token: String },
    /// No authentication
    None,
}

/// Errors that can occur during authentication processing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// Invalid authentication format or encoding
    InvalidFormat(String),
    /// Missing required authentication credentials
    MissingCredentials(String),
    /// Invalid base64 encoding in Basic auth
    InvalidEncoding(String),
    /// Unsupported authentication scheme
    UnsupportedScheme(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidFormat(msg) => write!(f, "Invalid authentication format: {}", msg),
            AuthError::MissingCredentials(msg) => write!(f, "Missing credentials: {}", msg),
            AuthError::InvalidEncoding(msg) => write!(f, "Invalid encoding: {}", msg),
            AuthError::UnsupportedScheme(msg) => write!(f, "Unsupported scheme: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// Detects the authentication scheme from a request.
///
/// This function examines the request's Authorization header and any auth-related
/// comments to determine which authentication scheme should be used.
///
/// # Arguments
///
/// * `request` - The HTTP request to analyze
///
/// # Returns
///
/// The detected `AuthScheme` or `AuthScheme::None` if no authentication is found.
pub fn detect_auth_scheme(request: &HttpRequest) -> AuthScheme {
    // First, check for Authorization header
    if let Some(auth_header) = request
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
        .map(|(_, v)| v)
    {
        // Check if it's Basic auth
        if auth_header.trim().starts_with("Basic ") {
            if let Some((username, password)) = basic::parse_basic_auth_header(auth_header) {
                return AuthScheme::Basic { username, password };
            }
        }
        // Check if it's Bearer auth
        else if auth_header.trim().starts_with("Bearer ") {
            if let Some(token) = bearer::parse_bearer_token_header(auth_header) {
                return AuthScheme::Bearer { token };
            }
        }
    }

    // If no Authorization header, return None
    // Note: Comment-based auth detection would be implemented in the parser
    // and would set the Authorization header before this function is called
    AuthScheme::None
}

/// Applies authentication to an HTTP request.
///
/// This function detects the authentication scheme from the request and ensures
/// the Authorization header is properly formatted. If variables are present in
/// the auth values (like {{username}} or {{token}}), they should be resolved
/// before calling this function.
///
/// # Arguments
///
/// * `request` - A mutable reference to the HTTP request
///
/// # Returns
///
/// `Ok(())` if authentication was applied successfully, or an `AuthError` if
/// something went wrong.
///
/// # Examples
///
/// ```
/// use rest_client::models::request::{HttpRequest, HttpMethod};
/// use rest_client::auth::apply_authentication;
///
/// let mut request = HttpRequest::new(
///     "test-id".to_string(),
///     HttpMethod::GET,
///     "https://api.example.com/data".to_string()
/// );
///
/// request.add_header("Authorization".to_string(), "Basic dXNlcjpwYXNz".to_string());
///
/// // Apply authentication (validates and normalizes the header)
/// let result = apply_authentication(&mut request);
/// assert!(result.is_ok());
/// ```
pub fn apply_authentication(request: &mut HttpRequest) -> Result<(), AuthError> {
    let auth_scheme = detect_auth_scheme(request);

    match auth_scheme {
        AuthScheme::Basic { username, password } => {
            // Re-encode to ensure proper formatting
            let auth_value = basic::basic_auth(&username, &password);

            // Update or add the Authorization header
            update_auth_header(request, auth_value);
            Ok(())
        }
        AuthScheme::Bearer { token } => {
            // Re-format to ensure proper formatting
            let auth_value = bearer::bearer_token(&token);

            // Update or add the Authorization header
            update_auth_header(request, auth_value);
            Ok(())
        }
        AuthScheme::None => {
            // No authentication needed
            Ok(())
        }
    }
}

/// Helper function to update the Authorization header in a request.
///
/// This handles the case-insensitive nature of HTTP headers by removing
/// any existing Authorization header and adding the new one.
fn update_auth_header(request: &mut HttpRequest, auth_value: String) {
    // Remove any existing Authorization header (case-insensitive)
    request
        .headers
        .retain(|k, _| !k.eq_ignore_ascii_case("authorization"));

    // Add the new Authorization header
    request
        .headers
        .insert("Authorization".to_string(), auth_value);
}

/// Parses authentication from a comment directive.
///
/// Supports the following formats:
/// - `# @basic username password` - Basic authentication
/// - `# @bearer token` - Bearer token authentication
///
/// # Arguments
///
/// * `comment` - The comment line to parse
///
/// # Returns
///
/// The detected `AuthScheme` or `AuthScheme::None` if no valid auth directive found.
///
/// # Examples
///
/// ```
/// use rest_client::auth::{parse_auth_comment, AuthScheme};
///
/// let scheme = parse_auth_comment("# @basic myuser mypass");
/// match scheme {
///     AuthScheme::Basic { username, password } => {
///         assert_eq!(username, "myuser");
///         assert_eq!(password, "mypass");
///     }
///     _ => panic!("Expected Basic auth"),
/// }
/// ```
pub fn parse_auth_comment(comment: &str) -> AuthScheme {
    let comment = comment.trim();

    // Remove comment prefix if present
    let content = if let Some(stripped) = comment.strip_prefix('#') {
        stripped.trim()
    } else if let Some(stripped) = comment.strip_prefix("//") {
        stripped.trim()
    } else {
        comment
    };

    // Check for @basic directive
    if let Some(rest) = content.strip_prefix("@basic") {
        let rest = rest.trim();
        let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();

        if parts.len() == 2 {
            return AuthScheme::Basic {
                username: parts[0].to_string(),
                password: parts[1].to_string(),
            };
        }
    }

    // Check for @bearer directive
    if let Some(rest) = content.strip_prefix("@bearer") {
        let token = rest.trim();
        if !token.is_empty() {
            return AuthScheme::Bearer {
                token: token.to_string(),
            };
        }
    }

    AuthScheme::None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::{HttpMethod, HttpRequest};

    #[test]
    fn test_detect_auth_scheme_basic() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header(
            "Authorization".to_string(),
            "Basic dXNlcjpwYXNz".to_string(),
        );

        let scheme = detect_auth_scheme(&request);
        match scheme {
            AuthScheme::Basic { username, password } => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("Expected Basic auth scheme"),
        }
    }

    #[test]
    fn test_detect_auth_scheme_bearer() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer mytoken123".to_string());

        let scheme = detect_auth_scheme(&request);
        match scheme {
            AuthScheme::Bearer { token } => {
                assert_eq!(token, "mytoken123");
            }
            _ => panic!("Expected Bearer auth scheme"),
        }
    }

    #[test]
    fn test_detect_auth_scheme_none() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );

        let scheme = detect_auth_scheme(&request);
        assert_eq!(scheme, AuthScheme::None);
    }

    #[test]
    fn test_detect_auth_scheme_case_insensitive() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header(
            "authorization".to_string(),
            "Basic dXNlcjpwYXNz".to_string(),
        );

        let scheme = detect_auth_scheme(&request);
        assert!(matches!(scheme, AuthScheme::Basic { .. }));
    }

    #[test]
    fn test_apply_authentication_basic() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header(
            "Authorization".to_string(),
            "Basic dXNlcjpwYXNz".to_string(),
        );

        let result = apply_authentication(&mut request);
        assert!(result.is_ok());

        let auth_header = request.headers.get("Authorization").unwrap();
        assert_eq!(auth_header, "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn test_apply_authentication_bearer() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer token123".to_string());

        let result = apply_authentication(&mut request);
        assert!(result.is_ok());

        let auth_header = request.headers.get("Authorization").unwrap();
        assert_eq!(auth_header, "Bearer token123");
    }

    #[test]
    fn test_apply_authentication_none() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );

        let result = apply_authentication(&mut request);
        assert!(result.is_ok());

        // Should not have Authorization header
        assert!(!request.headers.contains_key("Authorization"));
    }

    #[test]
    fn test_update_auth_header_case_insensitive() {
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );

        // Add lowercase authorization header
        request.add_header("authorization".to_string(), "Basic old".to_string());

        // Update should remove old and add new
        update_auth_header(&mut request, "Bearer new".to_string());

        assert!(!request.headers.contains_key("authorization"));
        assert_eq!(
            request.headers.get("Authorization"),
            Some(&"Bearer new".to_string())
        );
    }

    #[test]
    fn test_parse_auth_comment_basic() {
        let scheme = parse_auth_comment("# @basic username password");
        match scheme {
            AuthScheme::Basic { username, password } => {
                assert_eq!(username, "username");
                assert_eq!(password, "password");
            }
            _ => panic!("Expected Basic auth"),
        }
    }

    #[test]
    fn test_parse_auth_comment_basic_with_special_chars() {
        let scheme = parse_auth_comment("# @basic user@example.com p@ss:word");
        match scheme {
            AuthScheme::Basic { username, password } => {
                assert_eq!(username, "user@example.com");
                assert_eq!(password, "p@ss:word");
            }
            _ => panic!("Expected Basic auth"),
        }
    }

    #[test]
    fn test_parse_auth_comment_bearer() {
        let scheme = parse_auth_comment("# @bearer mytoken123");
        match scheme {
            AuthScheme::Bearer { token } => {
                assert_eq!(token, "mytoken123");
            }
            _ => panic!("Expected Bearer auth"),
        }
    }

    #[test]
    fn test_parse_auth_comment_bearer_jwt() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let scheme = parse_auth_comment(&format!("# @bearer {}", jwt));
        match scheme {
            AuthScheme::Bearer { token } => {
                assert_eq!(token, jwt);
            }
            _ => panic!("Expected Bearer auth"),
        }
    }

    #[test]
    fn test_parse_auth_comment_with_double_slash() {
        let scheme = parse_auth_comment("// @basic user pass");
        match scheme {
            AuthScheme::Basic { username, password } => {
                assert_eq!(username, "user");
                assert_eq!(password, "pass");
            }
            _ => panic!("Expected Basic auth"),
        }
    }

    #[test]
    fn test_parse_auth_comment_no_directive() {
        let scheme = parse_auth_comment("# This is just a comment");
        assert_eq!(scheme, AuthScheme::None);
    }

    #[test]
    fn test_parse_auth_comment_invalid_basic() {
        // Missing password
        let scheme = parse_auth_comment("# @basic username");
        assert_eq!(scheme, AuthScheme::None);
    }

    #[test]
    fn test_parse_auth_comment_invalid_bearer() {
        // Missing token
        let scheme = parse_auth_comment("# @bearer");
        assert_eq!(scheme, AuthScheme::None);
    }

    #[test]
    fn test_auth_error_display() {
        let error = AuthError::InvalidFormat("test".to_string());
        assert_eq!(format!("{}", error), "Invalid authentication format: test");

        let error = AuthError::MissingCredentials("username".to_string());
        assert_eq!(format!("{}", error), "Missing credentials: username");

        let error = AuthError::InvalidEncoding("base64".to_string());
        assert_eq!(format!("{}", error), "Invalid encoding: base64");

        let error = AuthError::UnsupportedScheme("Digest".to_string());
        assert_eq!(format!("{}", error), "Unsupported scheme: Digest");
    }
}
