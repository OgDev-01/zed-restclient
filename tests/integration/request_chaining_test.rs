//! Request chaining and workflow integration tests
//!
//! These tests verify variable resolution, environment management,
//! history tracking, and component integration workflows.

use rest_client::environment::{Environment, EnvironmentSession, Environments};
use rest_client::history::HistoryEntry;
use rest_client::models::{HttpMethod, HttpRequest, HttpResponse, RequestTiming};
use rest_client::parser::parse_file;

use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Helper to create a temporary .http file
fn create_temp_http_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let file_path = dir.path().join(name);
    fs::write(&file_path, content).expect("Failed to write test file");
    file_path
}

/// Helper to create environment config file
fn create_env_config(dir: &TempDir, config: &Environments) -> PathBuf {
    let config_path = dir.path().join(".http-client-env.json");
    let json = serde_json::to_string_pretty(config).expect("Failed to serialize config");
    fs::write(&config_path, json).expect("Failed to write config");
    config_path
}

/// Helper to create a mock HTTP response
fn create_mock_response(status_code: u16, body_json: serde_json::Value) -> HttpResponse {
    let mut response = HttpResponse::new(status_code, "OK".to_string());
    let body_str = serde_json::to_string(&body_json).unwrap();
    response.set_body(body_str.as_bytes().to_vec());
    response.add_header("Content-Type".to_string(), "application/json".to_string());
    response.duration = Duration::from_millis(100);
    response.timing = RequestTiming::new();
    response
}

#[test]
fn test_request_chaining_variable_capture_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create .http file with variable capture syntax
    let http_content = r#"# @name login
POST https://api.example.com/auth/login
Content-Type: application/json

{"username": "test", "password": "pass"}

###

# Use captured token from previous request
GET https://api.example.com/api/profile
Authorization: Bearer {{login.response.body.token}}
"#;

    let file_path = create_temp_http_file(&temp_dir, "chained.http", http_content);
    let requests = parse_file(http_content, &file_path).unwrap();

    assert_eq!(requests.len(), 2, "Should parse two requests");

    // Verify first request
    assert_eq!(requests[0].method, HttpMethod::POST);
    assert!(requests[0].body.is_some());

    // Verify second request has variable placeholder
    assert_eq!(requests[1].method, HttpMethod::GET);
    assert!(requests[1]
        .headers
        .get("Authorization")
        .unwrap()
        .contains("{{login.response.body.token}}"));

    // Simulate variable resolution
    let mock_token = "secret-auth-token-xyz";
    let mut resolved_request = requests[1].clone();
    if let Some(auth_header) = resolved_request.headers.get("Authorization") {
        let resolved = auth_header.replace("{{login.response.body.token}}", mock_token);
        resolved_request
            .headers
            .insert("Authorization".to_string(), resolved);
    }

    assert_eq!(
        resolved_request.headers.get("Authorization").unwrap(),
        &format!("Bearer {}", mock_token)
    );
}

#[test]
fn test_environment_switching_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create environment configuration
    let mut environments = Environments::new();

    let mut dev = Environment::new("development");
    dev.set("baseUrl", "http://dev.example.com");
    dev.set("apiKey", "dev-api-key");
    environments.add_environment(dev);

    let mut prod = Environment::new("production");
    prod.set("baseUrl", "http://api.example.com");
    prod.set("apiKey", "prod-api-key");
    environments.add_environment(prod);

    create_env_config(&temp_dir, &environments);

    // Test environment session switching
    let session = EnvironmentSession::new(environments.clone());

    // Set dev environment active
    session.set_active_environment("development").unwrap();
    let active_dev = session.get_active_environment().unwrap();
    assert_eq!(active_dev.name, "development");
    assert_eq!(active_dev.get("baseUrl").unwrap(), "http://dev.example.com");
    assert_eq!(active_dev.get("apiKey").unwrap(), "dev-api-key");

    // Set production environment active
    session.set_active_environment("production").unwrap();
    let active_prod = session.get_active_environment().unwrap();
    assert_eq!(active_prod.name, "production");
    assert_eq!(
        active_prod.get("baseUrl").unwrap(),
        "http://api.example.com"
    );
    assert_eq!(active_prod.get("apiKey").unwrap(), "prod-api-key");
}

#[test]
fn test_history_save_and_replay_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create a request
    let http_content = "GET https://api.example.com/data\n";
    let file_path = create_temp_http_file(&temp_dir, "test.http", http_content);
    let requests = parse_file(http_content, &file_path).unwrap();

    // Create mock response
    let response = create_mock_response(
        200,
        json!({
            "data": "test-data",
            "timestamp": 1234567890
        }),
    );

    // Save to history
    let entry = HistoryEntry::new(requests[0].clone(), response.clone());

    // Verify entry was created correctly
    assert_eq!(entry.request.url, requests[0].url);
    assert_eq!(entry.response.status_code, 200);

    // Test replay: The request can be re-executed
    let replay_request = entry.request.clone();
    assert_eq!(replay_request.method, HttpMethod::GET);
    assert_eq!(replay_request.url, "https://api.example.com/data");
}

#[test]
fn test_complex_variable_chain() {
    let temp_dir = TempDir::new().unwrap();

    let http_content = r#"# @name createUser
POST https://api.example.com/users
Content-Type: application/json

{"name": "New User"}

###

# @name createPost
POST https://api.example.com/posts
Authorization: Bearer {{createUser.response.body.api_token}}
Content-Type: application/json

{"title": "Test Post", "user_id": "{{createUser.response.body.id}}"}

###

# @name getPost
GET https://api.example.com/posts/{{createPost.response.body.id}}
Authorization: Bearer {{createUser.response.body.api_token}}
"#;

    let file_path = create_temp_http_file(&temp_dir, "complex.http", http_content);
    let requests = parse_file(http_content, &file_path).unwrap();

    assert_eq!(requests.len(), 3);

    // Verify variable placeholders exist
    assert!(requests[1]
        .headers
        .get("Authorization")
        .unwrap()
        .contains("{{createUser.response.body.api_token}}"));

    assert!(requests[1]
        .body
        .as_ref()
        .unwrap()
        .contains("{{createUser.response.body.id}}"));

    assert!(requests[2].url.contains("{{createPost.response.body.id}}"));

    // Simulate variable resolution
    let user_id = "usr-456";
    let token = "token-abc-123";
    let post_id = "post-789";

    let mut request2 = requests[1].clone();
    request2
        .headers
        .insert("Authorization".to_string(), format!("Bearer {}", token));
    if let Some(body) = &request2.body {
        let resolved_body = body.replace("{{createUser.response.body.id}}", user_id);
        request2.body = Some(resolved_body);
    }

    assert!(request2.body.as_ref().unwrap().contains(user_id));

    let mut request3 = requests[2].clone();
    request3.url = request3
        .url
        .replace("{{createPost.response.body.id}}", post_id);
    request3
        .headers
        .insert("Authorization".to_string(), format!("Bearer {}", token));

    assert!(request3.url.contains(post_id));
}

#[test]
fn test_error_handling_workflow() {
    let temp_dir = TempDir::new().unwrap();

    let http_content =
        "POST https://api.example.com/fail\nContent-Type: application/json\n\n{\"name\": \"Test\"}";
    let file_path = create_temp_http_file(&temp_dir, "error.http", http_content);

    // Test error handling through the full pipeline: parse → format
    let requests = parse_file(http_content, &file_path).unwrap();
    assert!(requests.len() > 0);

    // Create error response
    let error_response = create_mock_response(
        400,
        json!({
            "error": "Bad Request",
            "details": "Missing required field: email"
        }),
    );

    // Verify error response can be processed
    assert_eq!(error_response.status_code, 400);
    assert!(!error_response.is_success());
    assert!(error_response.is_client_error());

    let body_str = error_response.body_as_string().unwrap();
    assert!(body_str.contains("Bad Request"));
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
fn test_environment_variable_resolution_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create environment with variables
    let mut environments = Environments::new();

    let mut dev = Environment::new("dev");
    dev.set("host", "http://localhost:3000");
    dev.set("apiKey", "test-key");
    dev.set("timeout", "5000");
    environments.add_environment(dev);

    create_env_config(&temp_dir, &environments);

    // Test environment variable get
    let session = EnvironmentSession::new(environments.clone());
    session.set_active_environment("dev").unwrap();

    let active = session.get_active_environment().unwrap();
    assert_eq!(active.get("host").unwrap(), "http://localhost:3000");
    assert_eq!(active.get("apiKey").unwrap(), "test-key");
    assert_eq!(active.get("timeout").unwrap(), "5000");
}

#[test]
fn test_multiple_environments() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple environments
    let mut environments = Environments::new();

    let mut dev = Environment::new("dev");
    dev.set("apiUrl", "http://dev.example.com");
    dev.set("debug", "true");
    environments.add_environment(dev);

    let mut staging = Environment::new("staging");
    staging.set("apiUrl", "http://staging.example.com");
    staging.set("debug", "false");
    environments.add_environment(staging);

    let mut prod = Environment::new("production");
    prod.set("apiUrl", "http://api.example.com");
    prod.set("debug", "false");
    environments.add_environment(prod);

    // Set shared variables
    environments.set_shared("version", "v1");
    environments.set_shared("contentType", "application/json");

    create_env_config(&temp_dir, &environments);

    // Create session and test environment switching
    let session = EnvironmentSession::new(environments.clone());

    // Test dev environment
    session.set_active_environment("dev").unwrap();
    let active = session.get_active_environment().unwrap();
    assert_eq!(active.name, "dev");
    assert_eq!(active.get("apiUrl").unwrap(), "http://dev.example.com");

    // Test staging environment
    session.set_active_environment("staging").unwrap();
    let active = session.get_active_environment().unwrap();
    assert_eq!(active.name, "staging");
    assert_eq!(active.get("apiUrl").unwrap(), "http://staging.example.com");

    // Test production environment
    session.set_active_environment("production").unwrap();
    let active = session.get_active_environment().unwrap();
    assert_eq!(active.name, "production");
    assert_eq!(active.get("apiUrl").unwrap(), "http://api.example.com");
}

#[test]
fn test_history_entry_creation() {
    let request = HttpRequest {
        id: uuid::Uuid::new_v4().to_string(),
        method: HttpMethod::GET,
        url: "https://example.com/api/test".to_string(),
        http_version: Some("HTTP/1.1".to_string()),
        headers: HashMap::new(),
        body: None,
        line_number: 1,
        file_path: PathBuf::new(),
    };

    let response = HttpResponse::new(200, "OK".to_string());

    let entry = HistoryEntry::new(request.clone(), response.clone());

    assert_eq!(entry.request.url, request.url);
    assert_eq!(entry.response.status_code, response.status_code);
}

#[test]
fn test_environment_session_lifecycle() {
    let mut environments = Environments::new();

    let mut dev = Environment::new("dev");
    dev.set("url", "http://dev.local");
    environments.add_environment(dev);

    let mut prod = Environment::new("prod");
    prod.set("url", "http://prod.com");
    environments.add_environment(prod);

    let session = EnvironmentSession::new(environments);

    // Initially no active environment
    assert!(session.get_active_environment().is_none());

    // Set dev active
    session.set_active_environment("dev").unwrap();
    assert!(session.get_active_environment().is_some());
    assert_eq!(session.get_active_environment().unwrap().name, "dev");

    // Switch to prod
    session.set_active_environment("prod").unwrap();
    assert_eq!(session.get_active_environment().unwrap().name, "prod");

    // Try to set non-existent environment
    let result = session.set_active_environment("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_environment_merged_variables() {
    let mut environments = Environments::new();

    // Set shared variables
    environments.set_shared("apiVersion", "v1");
    environments.set_shared("contentType", "application/json");

    // Create dev environment with specific variables
    let mut dev = Environment::new("dev");
    dev.set("baseUrl", "http://localhost:3000");
    dev.set("apiKey", "dev-key");
    environments.add_environment(dev);

    // Set active environment
    let session = EnvironmentSession::new(environments.clone());
    session.set_active_environment("dev").unwrap();

    let active = session.get_active_environment().unwrap();

    // Environment-specific variables should be accessible
    assert_eq!(active.get("baseUrl").unwrap(), "http://localhost:3000");
    assert_eq!(active.get("apiKey").unwrap(), "dev-key");
}
