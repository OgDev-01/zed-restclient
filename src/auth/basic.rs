//! Basic HTTP authentication implementation.
//!
//! This module provides functions for encoding and decoding HTTP Basic authentication
//! credentials according to RFC 7617.

use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Encodes username and password into a Basic authentication header value.
///
/// The function takes a username and password, concatenates them with a colon,
/// encodes the result in base64, and returns a properly formatted Basic auth
/// header value.
///
/// # Arguments
///
/// * `username` - The username for authentication
/// * `password` - The password for authentication
///
/// # Returns
///
/// A `String` containing the formatted Basic auth header value in the format
/// "Basic <base64_encoded_credentials>"
///
/// # Examples
///
/// ```
/// use rest_client::auth::basic::basic_auth;
///
/// let auth_header = basic_auth("user", "pass123");
/// assert_eq!(auth_header, "Basic dXNlcjpwYXNzMTIz");
/// ```
pub fn basic_auth(username: &str, password: &str) -> String {
    let credentials = format!("{}:{}", username, password);
    let encoded = STANDARD.encode(credentials.as_bytes());
    format!("Basic {}", encoded)
}

/// Parses a Basic authentication header value and extracts the username and password.
///
/// This function decodes a Basic auth header value and returns the username
/// and password as a tuple. Returns `None` if the header is malformed or
/// cannot be decoded.
///
/// # Arguments
///
/// * `header` - The Authorization header value (e.g., "Basic dXNlcjpwYXNz")
///
/// # Returns
///
/// `Some((username, password))` if the header is valid, `None` otherwise.
///
/// # Examples
///
/// ```
/// use rest_client::auth::basic::parse_basic_auth_header;
///
/// let result = parse_basic_auth_header("Basic dXNlcjpwYXNzMTIz");
/// assert_eq!(result, Some(("user".to_string(), "pass123".to_string())));
///
/// let invalid = parse_basic_auth_header("Bearer token123");
/// assert_eq!(invalid, None);
/// ```
pub fn parse_basic_auth_header(header: &str) -> Option<(String, String)> {
    // Remove leading/trailing whitespace
    let header = header.trim();

    // Check if header starts with "Basic "
    if !header.starts_with("Basic ") {
        return None;
    }

    // Extract the base64-encoded part
    let encoded = header.strip_prefix("Basic ")?.trim();

    // Decode from base64
    let decoded_bytes = STANDARD.decode(encoded).ok()?;
    let decoded_str = String::from_utf8(decoded_bytes).ok()?;

    // Split on the first colon to separate username and password
    let colon_pos = decoded_str.find(':')?;
    let username = decoded_str[..colon_pos].to_string();
    let password = decoded_str[colon_pos + 1..].to_string();

    Some((username, password))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_simple() {
        let result = basic_auth("user", "pass");
        assert_eq!(result, "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn test_basic_auth_with_special_chars() {
        let result = basic_auth("admin@example.com", "p@ss:w0rd!");
        // Verify it starts with "Basic " and contains valid base64
        assert!(result.starts_with("Basic "));
        let encoded = result.strip_prefix("Basic ").unwrap();
        // Decode to verify it's correct
        let decoded = STANDARD.decode(encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, "admin@example.com:p@ss:w0rd!");
    }

    #[test]
    fn test_basic_auth_empty_password() {
        let result = basic_auth("user", "");
        assert_eq!(result, "Basic dXNlcjo=");
    }

    #[test]
    fn test_basic_auth_empty_username() {
        let result = basic_auth("", "password");
        let encoded = result.strip_prefix("Basic ").unwrap();
        let decoded = STANDARD.decode(encoded).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();
        assert_eq!(decoded_str, ":password");
    }

    #[test]
    fn test_parse_basic_auth_header_valid() {
        let header = "Basic dXNlcjpwYXNz";
        let result = parse_basic_auth_header(header);
        assert_eq!(result, Some(("user".to_string(), "pass".to_string())));
    }

    #[test]
    fn test_parse_basic_auth_header_with_whitespace() {
        let header = "  Basic   dXNlcjpwYXNz  ";
        let result = parse_basic_auth_header(header);
        assert_eq!(result, Some(("user".to_string(), "pass".to_string())));
    }

    #[test]
    fn test_parse_basic_auth_header_with_colon_in_password() {
        let header = basic_auth("user", "pass:with:colons");
        let result = parse_basic_auth_header(&header);
        assert_eq!(
            result,
            Some(("user".to_string(), "pass:with:colons".to_string()))
        );
    }

    #[test]
    fn test_parse_basic_auth_header_invalid_prefix() {
        let header = "Bearer token123";
        let result = parse_basic_auth_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_basic_auth_header_no_prefix() {
        let header = "dXNlcjpwYXNz";
        let result = parse_basic_auth_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_basic_auth_header_invalid_base64() {
        let header = "Basic !!!invalid!!!";
        let result = parse_basic_auth_header(header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_basic_auth_header_no_colon() {
        // Encode a string without colon
        let encoded = STANDARD.encode("usernameonly");
        let header = format!("Basic {}", encoded);
        let result = parse_basic_auth_header(&header);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_basic_auth_header_empty_values() {
        let header = basic_auth("", "");
        let result = parse_basic_auth_header(&header);
        assert_eq!(result, Some(("".to_string(), "".to_string())));
    }

    #[test]
    fn test_roundtrip() {
        let username = "test_user";
        let password = "test_pass_123!@#";

        let header = basic_auth(username, password);
        let parsed = parse_basic_auth_header(&header);

        assert_eq!(parsed, Some((username.to_string(), password.to_string())));
    }

    #[test]
    fn test_roundtrip_with_unicode() {
        let username = "用户";
        let password = "密码🔒";

        let header = basic_auth(username, password);
        let parsed = parse_basic_auth_header(&header);

        assert_eq!(parsed, Some((username.to_string(), password.to_string())));
    }
}
