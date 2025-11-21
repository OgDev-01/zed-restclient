//! cURL command parsing and generation.
//!
//! This module provides functionality to convert between cURL commands and HTTP requests.
//! It supports parsing cURL commands into `HttpRequest` structures and generating valid
//! cURL commands from `HttpRequest` objects.
//!
//! # Examples
//!
//! ## Parsing a cURL command
//!
//! ```
//! use rest_client::curl::parse_curl_command;
//!
//! let curl = r#"curl -X POST https://api.example.com/users \
//!   -H "Content-Type: application/json" \
//!   -d '{"name":"John Doe"}'"#;
//!
//! let request = parse_curl_command(curl).unwrap();
//! assert_eq!(request.url, "https://api.example.com/users");
//! ```
//!
//! ## Generating a cURL command
//!
//! ```
//! use rest_client::curl::generate_curl_command;
//! use rest_client::models::request::{HttpRequest, HttpMethod};
//!
//! let mut request = HttpRequest::new(
//!     "test".to_string(),
//!     HttpMethod::POST,
//!     "https://api.example.com/users".to_string()
//! );
//! request.add_header("Content-Type".to_string(), "application/json".to_string());
//! request.set_body(r#"{"name":"John"}"#.to_string());
//!
//! let curl = generate_curl_command(&request);
//! println!("{}", curl);
//! ```
//!
//! # Supported cURL Flags
//!
//! ## Parser Support
//!
//! The parser supports the following common cURL flags:
//!
//! - `-X`, `--request` - HTTP method (GET, POST, PUT, DELETE, etc.)
//! - `-H`, `--header` - HTTP headers
//! - `-d`, `--data`, `--data-raw`, `--data-binary` - Request body
//! - `-u`, `--user` - Basic authentication (converts to Authorization header)
//! - `--compressed` - Ignored (doesn't affect HTTP request)
//! - `-k`, `--insecure` - Ignored (doesn't affect HTTP request)
//! - `-L`, `--location` - Ignored (doesn't affect HTTP request)
//! - `-s`, `--silent` - Ignored (output option)
//! - `-v`, `--verbose` - Ignored (output option)
//! - `-i`, `--include` - Ignored (output option)
//!
//! ## Unsupported Flags
//!
//! Some flags are not supported as they don't translate to HTTP request properties:
//!
//! - `-o`, `--output` - File output (not relevant for HTTP request)
//! - `-w`, `--write-out` - Output formatting
//! - `--max-time`, `-m` - Timeout settings
//! - `--connect-timeout` - Connection timeout
//!
//! These flags will be ignored with a warning during parsing.
//!
//! # Features
//!
//! - **Auto-detection**: Automatically detects JSON bodies and sets Content-Type
//! - **Shell escaping**: Properly escapes special characters for safe shell execution
//! - **Multi-line formatting**: Generates readable cURL with line continuations
//! - **Quote handling**: Correctly handles single and double quotes in parsing
//! - **Basic auth**: Converts `-u username:password` to Authorization header
//! - **Multiple data flags**: Concatenates multiple `-d` flags with `&`

pub mod generator;
pub mod parser;
pub mod ui;

// Re-export main functions for convenience
pub use generator::{
    generate_curl_command, generate_curl_command_compact, generate_curl_with_options, CurlOptions,
};
pub use parser::{parse_curl_command, ParseError};
pub use ui::{
    copy_as_curl_command, paste_curl_command, validate_curl_command, CopyCurlResult,
    PasteCurlResult,
};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::models::request::HttpMethod;

    #[test]
    fn test_round_trip_simple_get() {
        let original_curl = "curl https://api.example.com/users";
        let request = parse_curl_command(original_curl).unwrap();
        let generated_curl = generate_curl_command(&request);

        // Parse the generated curl back
        let request2 = parse_curl_command(&generated_curl).unwrap();

        assert_eq!(request.method, request2.method);
        assert_eq!(request.url, request2.url);
    }

    #[test]
    fn test_round_trip_post_with_json() {
        let original_curl = r#"curl -X POST https://api.example.com/users -H "Content-Type: application/json" -d '{"name":"John","age":30}'"#;
        let request = parse_curl_command(original_curl).unwrap();

        assert_eq!(request.method, HttpMethod::POST);
        assert_eq!(request.url, "https://api.example.com/users");
        assert!(request.body.is_some());

        let generated_curl = generate_curl_command(&request);
        let request2 = parse_curl_command(&generated_curl).unwrap();

        assert_eq!(request.method, request2.method);
        assert_eq!(request.url, request2.url);
        assert_eq!(request.body, request2.body);
    }

    #[test]
    fn test_round_trip_with_headers() {
        let original_curl = r#"curl -H "Authorization: Bearer token123" -H "Accept: application/json" https://api.example.com/data"#;
        let request = parse_curl_command(original_curl).unwrap();

        assert_eq!(request.headers.len(), 2);

        let generated_curl = generate_curl_command(&request);
        let request2 = parse_curl_command(&generated_curl).unwrap();

        assert_eq!(
            request.headers.get("Authorization"),
            request2.headers.get("Authorization")
        );
        assert_eq!(
            request.headers.get("Accept"),
            request2.headers.get("Accept")
        );
    }

    #[test]
    fn test_round_trip_with_auth() {
        let original_curl = "curl -u user:pass https://api.example.com";
        let request = parse_curl_command(original_curl).unwrap();

        assert!(request.headers.contains_key("Authorization"));

        let generated_curl = generate_curl_command(&request);

        // Generated curl will have Authorization header instead of -u flag
        assert!(generated_curl.contains("Authorization"));
    }

    #[test]
    fn test_compact_vs_multiline() {
        let original_curl = r#"curl -X POST https://api.example.com/users -H "Content-Type: application/json" -d '{"name":"John"}'"#;
        let request = parse_curl_command(original_curl).unwrap();

        let compact = generate_curl_command_compact(&request);
        let multiline = generate_curl_command(&request);

        // Compact should not have newlines
        assert!(!compact.contains('\n'));

        // Both should be valid and parseable
        let req_compact = parse_curl_command(&compact).unwrap();
        let req_multiline = parse_curl_command(&multiline).unwrap();

        assert_eq!(req_compact.method, req_multiline.method);
        assert_eq!(req_compact.url, req_multiline.url);
    }

    #[test]
    fn test_special_characters_preserved() {
        let original_curl = r#"curl -d 'name=John Doe&city=New York' https://api.example.com/form"#;
        let request = parse_curl_command(original_curl).unwrap();

        assert!(request.body.as_ref().unwrap().contains("John Doe"));
        assert!(request.body.as_ref().unwrap().contains("New York"));

        let generated_curl = generate_curl_command(&request);
        let request2 = parse_curl_command(&generated_curl).unwrap();

        assert_eq!(request.body, request2.body);
    }

    #[test]
    fn test_github_api_example() {
        let github_curl = r#"curl -X POST https://api.github.com/repos/owner/repo/issues \
            -H "Accept: application/vnd.github.v3+json" \
            -H "Authorization: Bearer ghp_token" \
            -d '{"title":"Found a bug","body":"Description"}'
        "#;

        let request = parse_curl_command(github_curl).unwrap();

        assert_eq!(request.method, HttpMethod::POST);
        assert_eq!(
            request.url,
            "https://api.github.com/repos/owner/repo/issues"
        );
        assert_eq!(
            request.headers.get("Accept"),
            Some(&"application/vnd.github.v3+json".to_string())
        );
        assert!(request.headers.contains_key("Authorization"));
        assert!(request.body.is_some());
    }

    #[test]
    fn test_stripe_api_example() {
        let stripe_curl = r#"curl https://api.stripe.com/v1/charges \
            -u sk_test_key: \
            -d amount=2000 \
            -d currency=usd \
            -d source=tok_visa
        "#;

        let request = parse_curl_command(stripe_curl).unwrap();

        assert_eq!(request.url, "https://api.stripe.com/v1/charges");
        assert!(request.headers.contains_key("Authorization"));
        // Multiple -d flags should be concatenated
        assert!(request.body.is_some());
    }
}
