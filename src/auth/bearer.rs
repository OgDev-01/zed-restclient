//! Bearer token authentication implementation.
//!
//! This module provides functions for formatting Bearer token authentication
//! headers according to RFC 6750.

/// Formats a token into a Bearer authentication header value.
///
/// The function takes a token string and returns a properly formatted Bearer
/// auth header value.
///
/// # Arguments
///
/// * `token` - The authentication token
///
/// # Returns
///
/// A `String` containing the formatted Bearer auth header value in the format
/// "Bearer <token>"
///
/// # Examples
///
/// ```
/// use rest_client::auth::bearer::bearer_token;
///
/// let auth_header = bearer_token("abc123xyz");
/// assert_eq!(auth_header, "Bearer abc123xyz");
/// ```
pub fn bearer_token(token: &str) -> String {
    format!("Bearer {}", token)
}

/// Parses a Bearer authentication header value and extracts the token.
///
/// This function extracts the token from a Bearer auth header value.
/// Returns `None` if the header is malformed or doesn't start with "Bearer ".
///
/// # Arguments
///
/// * `header` - The Authorization header value (e.g., "Bearer abc123")
///
/// # Returns
///
/// `Some(token)` if the header is valid, `None` otherwise.
///
/// # Examples
///
/// ```
/// use rest_client::auth::bearer::parse_bearer_token_header;
///
/// let result = parse_bearer_token_header("Bearer abc123xyz");
/// assert_eq!(result, Some("abc123xyz".to_string()));
///
/// let invalid = parse_bearer_token_header("Basic dXNlcjpwYXNz");
/// assert_eq!(invalid, None);
/// ```
pub fn parse_bearer_token_header(header: &str) -> Option<String> {
    // Remove leading/trailing whitespace
    let header = header.trim();

    // Check if header starts with "Bearer "
    if !header.starts_with("Bearer ") {
        return None;
    }

    // Extract the token part
    let token = header.strip_prefix("Bearer ")?.trim();

    // Return None if token is empty
    if token.is_empty() {
        return None;
    }

    Some(token.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_token_simple() {
        let result = bearer_token("abc123");
        assert_eq!(result, "Bearer abc123");
    }

    #[test]
    fn test_bearer_token_with_special_chars() {
        let result = bearer_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U");
        assert!(result.starts_with("Bearer "));
        assert!(result.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    }

    #[test]
    fn test_bearer_token_empty() {
        let result = bearer_token("");
        assert_eq!(result, "Bearer ");
    }

    #[test]
    fn test_bearer_token_with_spaces() {
        let result = bearer_token("token with spaces");
        assert_eq!(result, "Bearer token with spaces");
    }

    #[test]
    fn test_parse_bearer_token_header_valid() {
        let header = "Bearer abc123xyz";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, Some("abc123xyz".to_string()));
    }

    #[test]
    fn test_parse_bearer_token_header_with_whitespace() {
        let header = "  Bearer   token123  ";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, Some("token123".to_string()));
    }

    #[test]
    fn test_parse_bearer_token_header_jwt() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let header = format!("Bearer {}", jwt);
        let result = parse_bearer_token_header(&header);
        assert_eq!(result, Some(jwt.to_string()));
    }

    #[test]
    fn test_parse_bearer_token_header_invalid_prefix() {
        let header = "Basic dXNlcjpwYXNz";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_bearer_token_header_no_prefix() {
        let header = "abc123xyz";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_bearer_token_header_empty_token() {
        let header = "Bearer ";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_bearer_token_header_only_whitespace() {
        let header = "Bearer   ";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_roundtrip() {
        let token = "my_secret_token_123";
        let header = bearer_token(token);
        let parsed = parse_bearer_token_header(&header);
        assert_eq!(parsed, Some(token.to_string()));
    }

    #[test]
    fn test_roundtrip_with_complex_token() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let header = bearer_token(token);
        let parsed = parse_bearer_token_header(&header);
        assert_eq!(parsed, Some(token.to_string()));
    }

    #[test]
    fn test_case_sensitivity() {
        // "bearer" (lowercase) should not match
        let header = "bearer abc123";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, None);

        // "BEARER" (uppercase) should not match
        let header = "BEARER abc123";
        let result = parse_bearer_token_header(header);
        assert_eq!(result, None);
    }
}
