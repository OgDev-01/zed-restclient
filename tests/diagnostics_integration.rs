//! Integration tests for diagnostic provider
//!
//! These tests validate that the diagnostic provider correctly identifies
//! various error conditions and provides helpful suggestions.

use rest_client::language_server::diagnostics::{provide_diagnostics, DiagnosticSeverity};
use rest_client::variables::VariableContext;
use std::path::PathBuf;

#[test]
fn test_diagnostics_invalid_method() {
    let doc = "INVALID https://api.example.com\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("invalid-method")),
        "Should detect invalid HTTP method"
    );
}

#[test]
fn test_diagnostics_undefined_variable() {
    let doc = "GET https://api.example.com/{{undefinedVar}}\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let undefined_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("undefined-variable"))
        .collect();

    assert!(
        !undefined_diags.is_empty(),
        "Should detect undefined variable"
    );
    assert_eq!(undefined_diags[0].severity, DiagnosticSeverity::Warning);
}

#[test]
fn test_diagnostics_system_variables_no_warning() {
    let doc = "GET https://api.example.com/{{$guid}}/{{$timestamp}}\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let undefined_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("undefined-variable"))
        .collect();

    assert!(
        undefined_diags.is_empty(),
        "System variables should not generate undefined warnings"
    );
}

#[test]
fn test_diagnostics_header_typo() {
    let doc = "POST https://api.example.com\nConten-Type: application/json\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let typo_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("header-typo"))
        .collect();

    assert!(!typo_diags.is_empty(), "Should detect header typo");
    assert!(
        typo_diags[0].suggestion.is_some(),
        "Should provide suggestion"
    );
    assert!(
        typo_diags[0]
            .suggestion
            .as_ref()
            .unwrap()
            .contains("Content-Type"),
        "Should suggest correct header name"
    );
}

#[test]
fn test_diagnostics_invalid_json() {
    let doc = r#"POST https://api.example.com
Content-Type: application/json

{invalid json}
"#;
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let json_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("invalid-json"))
        .collect();

    assert!(!json_diags.is_empty(), "Should detect invalid JSON");
    assert_eq!(json_diags[0].severity, DiagnosticSeverity::Error);
}

#[test]
fn test_diagnostics_valid_json() {
    let doc = r#"POST https://api.example.com
Content-Type: application/json

{"name": "test", "value": 123}
"#;
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let json_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("invalid-json"))
        .collect();

    assert!(
        json_diags.is_empty(),
        "Valid JSON should not generate errors"
    );
}

#[test]
fn test_diagnostics_missing_content_type_post() {
    let doc = "POST https://api.example.com/users\n\n{\"data\": \"test\"}\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let missing_ct: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("missing-content-type"))
        .collect();

    assert!(
        !missing_ct.is_empty(),
        "POST with body should warn about missing Content-Type"
    );
    assert_eq!(missing_ct[0].severity, DiagnosticSeverity::Warning);
}

#[test]
fn test_diagnostics_get_no_content_type_warning() {
    let doc = "GET https://api.example.com/users\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let missing_ct: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("missing-content-type"))
        .collect();

    assert!(
        missing_ct.is_empty(),
        "GET should not warn about missing Content-Type"
    );
}

#[test]
fn test_diagnostics_url_without_scheme() {
    let doc = "GET api.example.com/users\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let url_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("url-scheme-missing"))
        .collect();

    assert!(
        !url_diags.is_empty(),
        "URL without scheme should generate warning"
    );
}

#[test]
fn test_diagnostics_multiple_errors() {
    let doc = r#"INVALID api.example.com/{{undefined}}
Conten-Type: application/json

{invalid json}
"#;
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    println!("Total diagnostics: {}", diagnostics.len());
    for (i, d) in diagnostics.iter().enumerate() {
        println!(
            "{}: {:?} - {} (code: {:?})",
            i, d.severity, d.message, d.code
        );
    }

    // Should have at least 3 diagnostics:
    // 1. Invalid method
    // 2. Undefined variable (or URL scheme missing)
    // 3. Header typo
    // Note: JSON validation might not trigger without proper Content-Type detection
    assert!(
        diagnostics.len() >= 3,
        "Should detect multiple issues in one request, found: {}",
        diagnostics.len()
    );

    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("invalid-method")),
        "Should detect invalid method"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("undefined-variable")),
        "Should detect undefined variable"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("header-typo")),
        "Should detect header typo"
    );
}

#[test]
fn test_diagnostics_empty_variable() {
    let doc = "GET https://api.example.com/{{}}\n";
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    let empty_var_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("empty-variable"))
        .collect();

    assert!(!empty_var_diags.is_empty(), "Should detect empty variable");
    assert_eq!(empty_var_diags[0].severity, DiagnosticSeverity::Error);
}

#[test]
fn test_diagnostics_valid_request_no_errors() {
    let doc = r#"POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer token123

{
  "name": "John Doe",
  "email": "john@example.com"
}
"#;
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    // Should have no errors or warnings for a perfectly valid request
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == DiagnosticSeverity::Error)
        .collect();

    assert!(errors.is_empty(), "Valid request should have no errors");
}

#[test]
fn test_diagnostics_multiple_requests() {
    let doc = r#"GET https://api.example.com/users

###

POST https://api.example.com/users
Content-Type: application/json

{"name": "test"}

###

INVALID https://api.example.com
"#;
    let context = VariableContext::new(PathBuf::from("."));
    let diagnostics = provide_diagnostics(doc, &context);

    // First request is valid
    // Second request is valid
    // Third request has invalid method
    let invalid_method_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.code.as_deref() == Some("invalid-method"))
        .collect();

    assert_eq!(
        invalid_method_diags.len(),
        1,
        "Should detect one invalid method across multiple requests"
    );
}
