//! Integration tests for code generation.
//!
//! These tests verify that the generated code is syntactically valid and
//! follows best practices for each language/library combination.

// Note: Using the crate name with underscores as per Rust conventions
extern crate rest_client;

use rest_client::codegen::{generate_code, Language, Library};
use rest_client::models::request::{HttpMethod, HttpRequest};
use std::fs;
use std::path::PathBuf;

/// Helper function to create a test output directory
fn get_test_output_dir() -> PathBuf {
    let dir = PathBuf::from("target/codegen-test-output");
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn test_generate_javascript_fetch_get_request() {
    let request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::GET,
        "https://jsonplaceholder.typicode.com/posts/1".to_string(),
    );

    let code = generate_code(&request, Language::JavaScript, Some(Library::Fetch)).unwrap();

    // Write to file for manual inspection
    let output_path = get_test_output_dir().join("fetch_get.js");
    fs::write(&output_path, &code).unwrap();

    // Verify code structure
    assert!(code.contains("async function makeRequest()"));
    assert!(code.contains("method: 'GET'"));
    assert!(code.contains("await fetch"));
    assert!(code.contains("response.ok"));
    assert!(code.contains("console.log"));
    assert!(code.contains("makeRequest();"));

    println!(
        "✓ Generated fetch GET code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_javascript_fetch_post_json() {
    let mut request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::POST,
        "https://jsonplaceholder.typicode.com/posts".to_string(),
    );
    request.add_header("Content-Type".to_string(), "application/json".to_string());
    request
        .set_body(r#"{"title": "Test Post", "body": "This is a test", "userId": 1}"#.to_string());

    let code = generate_code(&request, Language::JavaScript, Some(Library::Fetch)).unwrap();

    let output_path = get_test_output_dir().join("fetch_post_json.js");
    fs::write(&output_path, &code).unwrap();

    assert!(code.contains("method: 'POST'"));
    assert!(code.contains("JSON.stringify"));
    assert!(code.contains("Content-Type"));
    assert!(code.contains("body:"));

    println!(
        "✓ Generated fetch POST code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_javascript_axios_get_request() {
    let mut request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::GET,
        "https://jsonplaceholder.typicode.com/users".to_string(),
    );
    request.add_header(
        "Authorization".to_string(),
        "Bearer test-token-123".to_string(),
    );

    let code = generate_code(&request, Language::JavaScript, Some(Library::Axios)).unwrap();

    let output_path = get_test_output_dir().join("axios_get.js");
    fs::write(&output_path, &code).unwrap();

    assert!(code.contains("const axios = require('axios')"));
    assert!(code.contains("method: 'get'"));
    assert!(code.contains("Authorization"));
    assert!(code.contains("Bearer test-token-123"));
    assert!(code.contains("error.response"));

    println!(
        "✓ Generated axios GET code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_javascript_axios_post_json() {
    let mut request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::POST,
        "https://jsonplaceholder.typicode.com/posts".to_string(),
    );
    request.add_header("Content-Type".to_string(), "application/json".to_string());
    request.set_body(r#"{"name": "John Doe", "email": "john@example.com"}"#.to_string());

    let code = generate_code(&request, Language::JavaScript, Some(Library::Axios)).unwrap();

    let output_path = get_test_output_dir().join("axios_post_json.js");
    fs::write(&output_path, &code).unwrap();

    assert!(code.contains("method: 'post'"));
    assert!(code.contains("data:"));
    assert!(code.contains("name"));

    println!(
        "✓ Generated axios POST code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_python_requests_get() {
    let request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::GET,
        "https://jsonplaceholder.typicode.com/posts/1".to_string(),
    );

    let code = generate_code(&request, Language::Python, Some(Library::Requests)).unwrap();

    let output_path = get_test_output_dir().join("requests_get.py");
    fs::write(&output_path, &code).unwrap();

    assert!(code.contains("import requests"));
    assert!(code.contains("def make_request():"));
    assert!(code.contains("requests.get("));
    assert!(code.contains("response.raise_for_status()"));
    assert!(code.contains("if __name__ == '__main__':"));

    println!(
        "✓ Generated requests GET code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_python_requests_post_json() {
    let mut request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::POST,
        "https://jsonplaceholder.typicode.com/posts".to_string(),
    );
    request.add_header("Content-Type".to_string(), "application/json".to_string());
    request.add_header("Authorization".to_string(), "Bearer secret123".to_string());
    request.set_body(
        r#"{"title": "Python Test", "body": "Testing Python code generation"}"#.to_string(),
    );

    let code = generate_code(&request, Language::Python, Some(Library::Requests)).unwrap();

    let output_path = get_test_output_dir().join("requests_post_json.py");
    fs::write(&output_path, &code).unwrap();

    assert!(code.contains("requests.post("));
    assert!(code.contains("json=data"));
    assert!(code.contains("Authorization"));
    assert!(code.contains("Bearer secret123"));

    println!(
        "✓ Generated requests POST code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_python_urllib_get() {
    let mut request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::GET,
        "https://jsonplaceholder.typicode.com/users/1".to_string(),
    );
    request.add_header("Accept".to_string(), "application/json".to_string());

    let code = generate_code(&request, Language::Python, Some(Library::Urllib)).unwrap();

    let output_path = get_test_output_dir().join("urllib_get.py");
    fs::write(&output_path, &code).unwrap();

    assert!(code.contains("import urllib.request"));
    assert!(code.contains("def make_request():"));
    assert!(code.contains("method='GET'"));
    assert!(code.contains("urllib.request.urlopen"));
    assert!(code.contains("req.add_header"));

    println!(
        "✓ Generated urllib GET code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_python_urllib_post_json() {
    let mut request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::POST,
        "https://jsonplaceholder.typicode.com/posts".to_string(),
    );
    request.add_header("Content-Type".to_string(), "application/json".to_string());
    request.set_body(r#"{"userId": 1, "title": "Test", "body": "Content"}"#.to_string());

    let code = generate_code(&request, Language::Python, Some(Library::Urllib)).unwrap();

    let output_path = get_test_output_dir().join("urllib_post_json.py");
    fs::write(&output_path, &code).unwrap();

    assert!(code.contains("method='POST'"));
    assert!(code.contains("json.dumps(data).encode"));
    assert!(code.contains("Content-Type"));

    println!(
        "✓ Generated urllib POST code written to: {}",
        output_path.display()
    );
}

#[test]
fn test_generate_with_special_characters() {
    let mut request = HttpRequest::new(
        "test".to_string(),
        HttpMethod::POST,
        "https://api.example.com/search?q=hello%20world".to_string(),
    );
    request.add_header(
        "X-Custom".to_string(),
        "value with \"quotes\" and 'apostrophes'".to_string(),
    );
    request.set_body("Line 1\nLine 2\tTabbed\r\nWindows line".to_string());

    // Test JavaScript
    let js_code = generate_code(&request, Language::JavaScript, Some(Library::Fetch)).unwrap();
    let output_path = get_test_output_dir().join("special_chars.js");
    fs::write(&output_path, &js_code).unwrap();

    assert!(js_code.contains("\\n"));
    assert!(js_code.contains("\\t"));

    // Test Python
    let py_code = generate_code(&request, Language::Python, Some(Library::Requests)).unwrap();
    let output_path = get_test_output_dir().join("special_chars.py");
    fs::write(&output_path, &py_code).unwrap();

    assert!(py_code.contains("\\n"));
    assert!(py_code.contains("\\t"));

    println!("✓ Generated code with special characters handles escaping correctly");
}

#[test]
fn test_all_http_methods() {
    let methods = vec![
        HttpMethod::GET,
        HttpMethod::POST,
        HttpMethod::PUT,
        HttpMethod::DELETE,
        HttpMethod::PATCH,
    ];

    for method in methods {
        let request = HttpRequest::new(
            "test".to_string(),
            method,
            "https://api.example.com/resource".to_string(),
        );

        // Test JavaScript
        let js_code = generate_code(&request, Language::JavaScript, Some(Library::Fetch)).unwrap();
        assert!(js_code.contains(&format!("method: '{}'", method.as_str())));

        // Test Python
        let py_code = generate_code(&request, Language::Python, Some(Library::Requests)).unwrap();
        assert!(py_code.contains(&format!("requests.{}(", method.as_str().to_lowercase())));
    }

    println!("✓ All HTTP methods generate correctly");
}

#[test]
fn test_generated_files_summary() {
    let output_dir = get_test_output_dir();

    println!("\n========================================");
    println!("Generated Code Files Summary");
    println!("========================================");
    println!("Output directory: {}", output_dir.display());
    println!("\nGenerated files:");

    if let Ok(entries) = fs::read_dir(&output_dir) {
        let mut files: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        files.sort_by_key(|e| e.path());

        for entry in files {
            let path = entry.path();
            if let Ok(metadata) = fs::metadata(&path) {
                println!(
                    "  - {} ({} bytes)",
                    path.file_name().unwrap().to_string_lossy(),
                    metadata.len()
                );
            }
        }
    }

    println!("\nYou can manually test these files:");
    println!("  JavaScript: node target/codegen-test-output/<filename>.js");
    println!("  Python: python3 target/codegen-test-output/<filename>.py");
    println!("========================================\n");
}
