//! End-to-end integration tests for REST Client
//!
//! These tests verify complete user workflows from .http file parsing
//! through formatting and component integration without requiring the Zed runtime.

use rest_client::formatter::{format_response, ContentType};
use rest_client::models::{HttpMethod, HttpResponse, RequestTiming};
use rest_client::parser::parse_file;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create a temporary .http file
fn create_temp_http_file(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test.http");
    fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

/// Helper to create a mock HTTP response
fn create_mock_response(status_code: u16, body: &str, content_type: &str) -> HttpResponse {
    let mut response = HttpResponse::new(status_code, "OK".to_string());
    response.set_body(body.as_bytes().to_vec());
    response.add_header("Content-Type".to_string(), content_type.to_string());
    response.duration = Duration::from_millis(100);
    response.timing = RequestTiming::new();
    response
}

#[test]
fn test_end_to_end_parse_format_workflow() {
    // Step 1: Create .http file
    let http_content = "GET https://api.example.com/users\nAccept: application/json\n";
    let (_temp_dir, file_path) = create_temp_http_file(http_content);

    // Step 2: Parse file
    let parse_result = parse_file(http_content, &file_path);
    assert!(parse_result.is_ok(), "Failed to parse .http file");
    let requests = parse_result.unwrap();
    assert_eq!(requests.len(), 1, "Expected exactly one request");

    // Step 3: Verify parsed request
    let request = &requests[0];
    assert_eq!(request.method, HttpMethod::GET);
    assert_eq!(request.url, "https://api.example.com/users");
    assert_eq!(request.headers.len(), 1);
    assert_eq!(request.headers.get("Accept").unwrap(), "application/json");

    // Step 4: Create mock response
    let response = create_mock_response(
        200,
        r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#,
        "application/json",
    );

    // Step 5: Format response
    let formatted = format_response(&response);
    assert!(
        formatted.status_line.contains("200"),
        "Formatted response should contain status code"
    );
    assert!(
        formatted.formatted_body.contains("Alice"),
        "Formatted response should contain body content"
    );
    assert_eq!(formatted.content_type, ContentType::Json);
}

#[test]
fn test_end_to_end_multiple_requests_parsing() {
    let http_content = "GET https://api.example.com/users\n\n###\n\nGET https://api.example.com/posts\n\n###\n\nPOST https://api.example.com/login\nContent-Type: application/json\n\n{\"username\": \"test\"}";
    let (_temp_dir, file_path) = create_temp_http_file(http_content);

    let requests = parse_file(http_content, &file_path).unwrap();
    assert_eq!(requests.len(), 3, "Expected three requests");

    // Verify each request
    assert_eq!(requests[0].method, HttpMethod::GET);
    assert_eq!(requests[0].url, "https://api.example.com/users");

    assert_eq!(requests[1].method, HttpMethod::GET);
    assert_eq!(requests[1].url, "https://api.example.com/posts");

    assert_eq!(requests[2].method, HttpMethod::POST);
    assert_eq!(requests[2].url, "https://api.example.com/login");
    assert!(requests[2].body.is_some());
    assert!(requests[2].body.as_ref().unwrap().contains("username"));
}

#[test]
fn test_end_to_end_json_formatting() {
    let response = create_mock_response(
        200,
        r#"{"message":"Hello, World!","status":"success","data":{"count":42}}"#,
        "application/json",
    );

    let formatted = format_response(&response);

    // Verify formatted response structure
    assert!(!formatted.status_line.is_empty());
    assert!(!formatted.headers_text.is_empty());
    assert!(!formatted.formatted_body.is_empty());
    assert_eq!(formatted.content_type, ContentType::Json);

    // JSON should be pretty-printed (formatted_body contains the JSON)
    let body_str = std::str::from_utf8(&response.body).unwrap();
    assert!(body_str.contains("message"));
    assert!(body_str.contains("Hello, World!"));

    // Test display string
    let display_string = formatted.to_display_string();
    assert!(display_string.contains("200"));
    assert!(display_string.contains("Headers:"));
    assert!(display_string.contains("Duration:"));
}

#[test]
fn test_end_to_end_xml_formatting() {
    let response = create_mock_response(
        200,
        "<root><message>Test</message><status>ok</status></root>",
        "application/xml",
    );

    let formatted = format_response(&response);
    assert_eq!(formatted.content_type, ContentType::Xml);
    assert!(formatted.formatted_body.contains("message"));
    assert!(formatted.formatted_body.contains("Test"));
}

#[test]
fn test_end_to_end_html_formatting() {
    let response = create_mock_response(
        200,
        "<html><body><h1>Welcome</h1></body></html>",
        "text/html",
    );

    let formatted = format_response(&response);
    assert_eq!(formatted.content_type, ContentType::Html);
    assert!(formatted.formatted_body.contains("Welcome"));
}

#[test]
fn test_end_to_end_request_with_headers() {
    let http_content = "GET https://api.example.com/protected\nAuthorization: Bearer token123\nX-Custom-Header: custom-value\nAccept: application/json\n";
    let (_temp_dir, file_path) = create_temp_http_file(http_content);

    let requests = parse_file(http_content, &file_path).unwrap();
    let request = &requests[0];

    // Verify headers were parsed correctly
    assert_eq!(request.headers.len(), 3);
    assert_eq!(
        request.headers.get("Authorization").unwrap(),
        "Bearer token123"
    );
    assert_eq!(
        request.headers.get("X-Custom-Header").unwrap(),
        "custom-value"
    );
    assert_eq!(request.headers.get("Accept").unwrap(), "application/json");
}

#[test]
fn test_end_to_end_post_with_json_body() {
    let http_content =
        "POST https://api.example.com/users\nContent-Type: application/json\n\n{\"name\": \"Charlie\", \"email\": \"charlie@example.com\"}";
    let (_temp_dir, file_path) = create_temp_http_file(http_content);

    let requests = parse_file(http_content, &file_path).unwrap();
    let request = &requests[0];

    assert_eq!(request.method, HttpMethod::POST);
    assert!(request.body.is_some());
    let body = request.body.as_ref().unwrap();
    assert!(body.contains("Charlie"));
    assert!(body.contains("charlie@example.com"));
}

#[test]
fn test_end_to_end_large_response_formatting() {
    // Create large JSON response
    let large_json = serde_json::json!({
        "users": (0..100).map(|i| {
            serde_json::json!({
                "id": i,
                "name": format!("User {}", i),
                "email": format!("user{}@example.com", i)
            })
        }).collect::<Vec<_>>()
    });

    let response = create_mock_response(
        200,
        &serde_json::to_string(&large_json).unwrap(),
        "application/json",
    );

    let formatted = format_response(&response);
    assert!(
        formatted.formatted_body.len() > 1000,
        "Should format large responses"
    );
    assert!(formatted.formatted_body.contains("User 0"));
    assert!(formatted.formatted_body.contains("User 99"));
}

#[test]
fn test_parse_error_handling() {
    // Test empty file
    let empty_content = "";
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.http");
    fs::write(&file_path, empty_content).unwrap();
    let result = parse_file(empty_content, &file_path);
    assert!(result.is_ok(), "Empty file should parse to empty list");
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_formatted_response_metadata() {
    let response = create_mock_response(200, r#"{"message": "test"}"#, "application/json");

    let formatted = format_response(&response);

    // Verify metadata
    assert_eq!(formatted.metadata.status_code, 200);
    assert_eq!(formatted.metadata.status_text, "OK");
    assert!(formatted.metadata.is_success);
    assert!(!formatted.metadata.is_truncated);

    // Test formatting methods
    let duration_str = formatted.metadata.format_duration();
    assert!(duration_str.contains("ms") || duration_str.contains("s"));

    let size_str = formatted.metadata.format_size();
    assert!(size_str.contains("B") || size_str.contains("KB"));
}

#[test]
fn test_cleanup_temp_files() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.http");

    // Create a test file
    fs::write(&test_file, "GET https://example.com\n").unwrap();
    assert!(test_file.exists());

    // Verify file exists before cleanup
    assert!(temp_dir.path().exists());

    // Cleanup (temp_dir will auto-cleanup on drop)
    drop(temp_dir);
}

#[test]
fn test_response_body_conversion() {
    let response = create_mock_response(200, "Test Body Content", "text/plain");

    // Test body_as_string conversion
    let body_str = response.body_as_string().unwrap();
    assert_eq!(body_str, "Test Body Content");

    // Test that raw body is Vec<u8>
    assert_eq!(response.body, b"Test Body Content");
}

#[test]
fn test_request_parsing_comments() {
    let http_content = "# This is a comment\nGET https://api.example.com/users\n# Another comment\nAuthorization: Bearer token123\n";
    let (_temp_dir, file_path) = create_temp_http_file(http_content);

    let requests = parse_file(http_content, &file_path).unwrap();
    let request = &requests[0];

    // Comments should be ignored
    assert_eq!(request.url, "https://api.example.com/users");
    assert_eq!(
        request.headers.get("Authorization").unwrap(),
        "Bearer token123"
    );
}

#[test]
fn test_different_http_methods() {
    let methods = vec![
        ("GET", HttpMethod::GET),
        ("POST", HttpMethod::POST),
        ("PUT", HttpMethod::PUT),
        ("DELETE", HttpMethod::DELETE),
        ("PATCH", HttpMethod::PATCH),
        ("HEAD", HttpMethod::HEAD),
        ("OPTIONS", HttpMethod::OPTIONS),
    ];

    for (method_str, expected_method) in methods {
        let http_content = format!("{} https://api.example.com/test\n", method_str);
        let (_temp_dir, file_path) = create_temp_http_file(&http_content);

        let requests = parse_file(&http_content, &file_path).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, expected_method);
    }
}

#[test]
fn test_component_integration_parse_and_format() {
    // This test verifies that parsing and formatting components work together

    // Step 1: Parse request
    let http_content =
        "POST https://api.example.com/data\nContent-Type: application/json\n\n{\"key\": \"value\"}";
    let (_temp_dir, file_path) = create_temp_http_file(http_content);
    let requests = parse_file(http_content, &file_path).unwrap();
    let request = &requests[0];

    // Step 2: Create response based on request
    let response_body = serde_json::json!({
        "received": request.body.as_ref().unwrap(),
        "status": "processed"
    });

    let response = create_mock_response(
        200,
        &serde_json::to_string(&response_body).unwrap(),
        "application/json",
    );

    // Step 3: Format response
    let formatted = format_response(&response);

    // Step 4: Verify integration
    assert!(formatted.formatted_body.contains("received"));
    assert!(formatted.formatted_body.contains("processed"));
    assert_eq!(formatted.content_type, ContentType::Json);
}
