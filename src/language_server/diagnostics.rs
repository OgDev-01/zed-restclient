//! Diagnostic provider for REST Client
//!
//! This module provides LSP-style diagnostics for .http files, including:
//! - Syntax error detection (invalid methods, malformed URLs, invalid headers)
//! - Variable validation (undefined variables, malformed variable syntax)
//! - URL format validation
//! - Header name validation (with typo suggestions)
//! - JSON body validation when Content-Type is application/json
//! - Missing required headers for POST/PUT/PATCH requests

use crate::models::HttpMethod;
use crate::parser::{error::ParseError, parse_file};
use crate::variables::{substitute_variables, VarError, VariableContext};
use regex::Regex;
use std::collections::HashMap;

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error - code will not work correctly
    Error,
    /// Warning - code may work but has issues
    Warning,
    /// Info - informational message
    Info,
}

/// Position in a text document (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// Zero-based line number
    pub line: usize,
    /// Zero-based character offset in the line
    pub character: usize,
}

impl Position {
    /// Creates a new position
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

/// Range in a text document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Range {
    /// Creates a new range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Creates a range for an entire line
    pub fn line(line: usize) -> Self {
        Self {
            start: Position::new(line, 0),
            end: Position::new(line, usize::MAX),
        }
    }

    /// Creates a range for a specific part of a line
    pub fn at_line(line: usize, start_char: usize, end_char: usize) -> Self {
        Self {
            start: Position::new(line, start_char),
            end: Position::new(line, end_char),
        }
    }
}

/// A diagnostic message for a document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The range where the diagnostic applies
    pub range: Range,
    /// The severity of the diagnostic
    pub severity: DiagnosticSeverity,
    /// The diagnostic message
    pub message: String,
    /// Optional diagnostic code
    pub code: Option<String>,
    /// Optional related information or suggestions
    pub suggestion: Option<String>,
}

impl Diagnostic {
    /// Creates a new error diagnostic
    pub fn error(range: Range, message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            code: None,
            suggestion: None,
        }
    }

    /// Creates a new warning diagnostic
    pub fn warning(range: Range, message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            code: None,
            suggestion: None,
        }
    }

    /// Creates a new info diagnostic
    pub fn info(range: Range, message: impl Into<String>) -> Self {
        Self {
            range,
            severity: DiagnosticSeverity::Info,
            message: message.into(),
            code: None,
            suggestion: None,
        }
    }

    /// Sets the diagnostic code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets a suggestion for fixing the issue
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Provides comprehensive diagnostics for an HTTP document
///
/// # Arguments
/// * `document` - The full text of the .http file
/// * `context` - Variable context for resolving variables
///
/// # Returns
/// A vector of diagnostics found in the document
///
/// # Examples
/// ```
/// use rest_client::language_server::diagnostics::provide_diagnostics;
/// use rest_client::variables::VariableContext;
/// use std::path::PathBuf;
///
/// let doc = "GET https://api.example.com\nContent-Type: application/json";
/// let context = VariableContext::new(PathBuf::from("."));
/// let diagnostics = provide_diagnostics(doc, &context);
/// ```
pub fn provide_diagnostics(document: &str, context: &VariableContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // 1. Parse the document and collect syntax errors
    diagnostics.extend(check_syntax_errors(document));

    // 2. Check for variable issues
    diagnostics.extend(check_variable_issues(document, context));

    // 3. Validate URLs
    diagnostics.extend(check_url_format(document));

    // 4. Validate headers
    diagnostics.extend(check_header_issues(document));

    // 5. Validate JSON bodies
    diagnostics.extend(check_json_bodies(document));

    // 6. Check for missing required headers
    diagnostics.extend(check_required_headers(document));

    diagnostics
}

/// Checks for syntax errors by parsing the document
fn check_syntax_errors(document: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Use a dummy file path for parsing
    let file_path = std::path::PathBuf::from(".");

    match parse_file(document, &file_path) {
        Ok(_) => {
            // No syntax errors
        }
        Err(error) => {
            // parse_file returns a single ParseError, not a Vec
            let diagnostic = parse_error_to_diagnostic(&error);
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

/// Converts a ParseError to a Diagnostic
fn parse_error_to_diagnostic(error: &ParseError) -> Diagnostic {
    // Convert 1-based line number from parser to 0-based for LSP
    let line = error.line().saturating_sub(1);

    match error {
        ParseError::InvalidMethod { method, .. } => Diagnostic::error(
            Range::line(line),
            format!("Invalid HTTP method '{}'", method),
        )
        .with_code("invalid-method")
        .with_suggestion(
            "Expected one of: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, CONNECT",
        ),

        ParseError::InvalidUrl { url, .. } => {
            Diagnostic::error(Range::line(line), format!("Invalid URL format '{}'", url))
                .with_code("invalid-url")
                .with_suggestion("URL must start with http:// or https://")
        }

        ParseError::InvalidHeader { header, .. } => Diagnostic::error(
            Range::line(line),
            format!("Invalid header format '{}'", header),
        )
        .with_code("invalid-header")
        .with_suggestion("Headers must be in format 'Header-Name: value'"),

        ParseError::MissingUrl { .. } => {
            Diagnostic::error(Range::line(line), "Missing URL in request line")
                .with_code("missing-url")
                .with_suggestion("Expected format: METHOD URL [HTTP/VERSION]")
        }

        ParseError::EmptyRequest { .. } => {
            Diagnostic::warning(Range::line(line), "Empty request block").with_code("empty-request")
        }

        ParseError::InvalidHttpVersion { version, .. } => Diagnostic::error(
            Range::line(line),
            format!("Invalid HTTP version '{}'", version),
        )
        .with_code("invalid-http-version")
        .with_suggestion("Use HTTP/1.1 or HTTP/2"),
    }
}

/// Checks for variable-related issues
fn check_variable_issues(document: &str, context: &VariableContext) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Regex to find all {{variable}} patterns (including empty ones)
    let var_pattern = Regex::new(r"\{\{([^}]*)\}\}").unwrap();

    for (line_idx, line) in document.lines().enumerate() {
        for cap in var_pattern.captures_iter(line) {
            let var_name = cap.get(1).unwrap().as_str().trim();
            let match_start = cap.get(0).unwrap().start();
            let match_end = cap.get(0).unwrap().end();

            // Check for malformed variable syntax
            if var_name.is_empty() {
                diagnostics.push(
                    Diagnostic::error(
                        Range::at_line(line_idx, match_start, match_end),
                        "Empty variable name",
                    )
                    .with_code("empty-variable"),
                );
                continue;
            }

            // Check if variable is defined (skip system variables)
            if !var_name.starts_with('$') {
                // Try to resolve the variable
                let test_text = format!("{{{{{}}}}}", var_name);
                match substitute_variables(&test_text, context) {
                    Ok(_) => {
                        // Variable is defined
                    }
                    Err(VarError::UndefinedVariable(_)) => {
                        diagnostics.push(
                            Diagnostic::warning(
                                Range::at_line(line_idx, match_start, match_end),
                                format!("Undefined variable '{}'", var_name),
                            )
                            .with_code("undefined-variable")
                            .with_suggestion(
                                "Define this variable in your environment file or .http file",
                            ),
                        );
                    }
                    Err(_) => {
                        // Other errors (circular reference, etc.)
                        // These will be caught at runtime
                    }
                }
            }
        }

        // Check for unmatched braces
        if let Some(diagnostics_for_braces) = check_unmatched_braces(line, line_idx) {
            diagnostics.extend(diagnostics_for_braces);
        }
    }

    diagnostics
}

/// Checks for unmatched {{ or }} braces
fn check_unmatched_braces(line: &str, line_idx: usize) -> Option<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut open_count = 0;
    let mut last_open_pos = None;

    for (idx, ch) in line.chars().enumerate() {
        if ch == '{' && line.chars().nth(idx + 1) == Some('{') {
            open_count += 1;
            last_open_pos = Some(idx);
        } else if ch == '}' && idx > 0 && line.chars().nth(idx - 1) == Some('}') {
            if open_count > 0 {
                open_count -= 1;
            }
        }
    }

    if open_count > 0 {
        if let Some(pos) = last_open_pos {
            diagnostics.push(
                Diagnostic::error(
                    Range::at_line(line_idx, pos, pos + 2),
                    "Unclosed variable braces",
                )
                .with_code("unclosed-braces")
                .with_suggestion("Add closing }} to complete the variable"),
            );
        }
    }

    if diagnostics.is_empty() {
        None
    } else {
        Some(diagnostics)
    }
}

/// Validates URL formats in the document
fn check_url_format(document: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Regex to match request lines (METHOD URL [HTTP/VERSION])
    let request_line_pattern =
        Regex::new(r"^(GET|POST|PUT|DELETE|PATCH|OPTIONS|HEAD|TRACE|CONNECT)\s+(\S+)").unwrap();

    for (line_idx, line) in document.lines().enumerate() {
        let trimmed = line.trim();

        if let Some(cap) = request_line_pattern.captures(trimmed) {
            if let Some(url_match) = cap.get(2) {
                let url = url_match.as_str();

                // Skip if URL contains variables (will be validated at runtime)
                if url.contains("{{") {
                    continue;
                }

                // Basic URL validation
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    let start = line.find(url).unwrap_or(0);
                    diagnostics.push(
                        Diagnostic::warning(
                            Range::at_line(line_idx, start, start + url.len()),
                            format!("URL '{}' should start with http:// or https://", url),
                        )
                        .with_code("url-scheme-missing"),
                    );
                }

                // Check for spaces in URL (common mistake)
                if url.contains(' ') {
                    let start = line.find(url).unwrap_or(0);
                    diagnostics.push(
                        Diagnostic::error(
                            Range::at_line(line_idx, start, start + url.len()),
                            "URL cannot contain spaces",
                        )
                        .with_code("url-contains-spaces")
                        .with_suggestion("Use %20 for spaces or remove them"),
                    );
                }
            }
        }
    }

    diagnostics
}

/// Validates header names and checks for common typos
fn check_header_issues(document: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Common header names and their typos
    let common_headers = get_common_header_typos();

    for (line_idx, line) in document.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines, comments, and request lines
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Check if this looks like a header (contains :)
        if let Some(colon_pos) = trimmed.find(':') {
            // Skip if this is a URL (contains ://)
            if trimmed.contains("://") {
                continue;
            }

            let header_name = trimmed[..colon_pos].trim();

            // Check for empty header name
            if header_name.is_empty() {
                diagnostics.push(
                    Diagnostic::error(Range::line(line_idx), "Empty header name")
                        .with_code("empty-header-name"),
                );
                continue;
            }

            // Check for common typos
            if let Some(suggestion) = common_headers.get(header_name.to_lowercase().as_str()) {
                let start = line.find(header_name).unwrap_or(0);
                diagnostics.push(
                    Diagnostic::warning(
                        Range::at_line(line_idx, start, start + header_name.len()),
                        format!("Possible typo in header name '{}'", header_name),
                    )
                    .with_code("header-typo")
                    .with_suggestion(format!("Did you mean '{}'?", suggestion)),
                );
            }

            // Check for spaces in header name
            if header_name.contains(' ') && !header_name.starts_with('@') {
                let start = line.find(header_name).unwrap_or(0);
                diagnostics.push(
                    Diagnostic::error(
                        Range::at_line(line_idx, start, start + header_name.len()),
                        "Header names cannot contain spaces",
                    )
                    .with_code("header-name-spaces"),
                );
            }
        }
    }

    diagnostics
}

/// Returns a map of common header typos to their correct forms
fn get_common_header_typos() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("conten-type", "Content-Type");
    map.insert("content-typo", "Content-Type");
    map.insert("contenttype", "Content-Type");
    map.insert("content_type", "Content-Type");
    map.insert("authorisation", "Authorization");
    map.insert("auth", "Authorization");
    map.insert("accept-language", "Accept-Language");
    map.insert("user-agent", "User-Agent");
    map.insert("cache-control", "Cache-Control");
    map.insert("content-length", "Content-Length");
    map.insert("content-encoding", "Content-Encoding");
    map.insert("accept-encoding", "Accept-Encoding");
    map
}

/// Validates JSON bodies when Content-Type is application/json
fn check_json_bodies(document: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = document.lines().collect();

    let mut in_request = false;
    let mut content_type_is_json = false;
    let mut body_start_line = None;
    let mut current_line_idx = 0;

    while current_line_idx < lines.len() {
        let line = lines[current_line_idx];
        let trimmed = line.trim();

        // Check for request separator
        if trimmed == "###" {
            // Reset state for new request
            in_request = true;
            content_type_is_json = false;
            body_start_line = None;
            current_line_idx += 1;
            continue;
        }

        // Check for request line
        if is_request_line(trimmed) {
            in_request = true;
            content_type_is_json = false;
            body_start_line = None;
            current_line_idx += 1;
            continue;
        }

        // Check for Content-Type header
        if trimmed.to_lowercase().starts_with("content-type:") {
            if trimmed.to_lowercase().contains("application/json") {
                content_type_is_json = true;
            }
            current_line_idx += 1;
            continue;
        }

        // Empty line indicates start of body
        if in_request && trimmed.is_empty() && body_start_line.is_none() {
            body_start_line = Some(current_line_idx + 1);
            current_line_idx += 1;
            continue;
        }

        // If we have a JSON body, collect it and validate
        if let Some(start) = body_start_line {
            if content_type_is_json && current_line_idx == start {
                let body_lines = collect_body_lines(&lines, start);
                if !body_lines.is_empty() {
                    let body = body_lines.join("\n");
                    if let Err(e) = serde_json::from_str::<serde_json::Value>(&body) {
                        diagnostics.push(
                            Diagnostic::error(
                                Range::line(start),
                                format!("Invalid JSON in request body: {}", e),
                            )
                            .with_code("invalid-json")
                            .with_suggestion(
                                "Check JSON syntax - ensure proper quotes, commas, and brackets",
                            ),
                        );
                    }
                }
                // Skip past the body
                current_line_idx = start + body_lines.len();
                body_start_line = None;
                continue;
            }
        }

        current_line_idx += 1;
    }

    diagnostics
}

/// Collects body lines until the next request or separator
fn collect_body_lines<'a>(lines: &'a [&'a str], start: usize) -> Vec<&'a str> {
    let mut body_lines = Vec::new();

    for i in start..lines.len() {
        let trimmed = lines[i].trim();

        // Stop at next request separator or request line
        if trimmed == "###" || is_request_line(trimmed) {
            break;
        }

        body_lines.push(lines[i]);
    }

    body_lines
}

/// Checks if a line is a request line (starts with HTTP method)
fn is_request_line(line: &str) -> bool {
    let methods = [
        "GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD", "TRACE", "CONNECT",
    ];
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.is_empty() {
        return false;
    }

    methods.contains(&parts[0].to_uppercase().as_str())
}

/// Checks for missing required headers based on HTTP method
fn check_required_headers(document: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = document.lines().collect();

    let mut current_method: Option<HttpMethod> = None;
    let mut headers_for_current_request: Vec<String> = Vec::new();
    let mut request_line_idx = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for request separator
        if trimmed == "###" {
            // Check previous request
            if let Some(method) = current_method {
                check_missing_headers_for_method(
                    method,
                    &headers_for_current_request,
                    request_line_idx,
                    &mut diagnostics,
                );
            }
            // Reset for new request
            current_method = None;
            headers_for_current_request.clear();
            continue;
        }

        // Check for request line
        if let Some(method) = extract_method_from_line(trimmed) {
            // Check previous request first
            if let Some(prev_method) = current_method {
                check_missing_headers_for_method(
                    prev_method,
                    &headers_for_current_request,
                    request_line_idx,
                    &mut diagnostics,
                );
            }
            // Start new request
            current_method = Some(method);
            request_line_idx = line_idx;
            headers_for_current_request.clear();
            continue;
        }

        // Collect headers
        if current_method.is_some() && trimmed.contains(':') && !trimmed.contains("://") {
            if let Some(colon_pos) = trimmed.find(':') {
                let header_name = trimmed[..colon_pos].trim().to_lowercase();
                headers_for_current_request.push(header_name);
            }
        }
    }

    // Check last request
    if let Some(method) = current_method {
        check_missing_headers_for_method(
            method,
            &headers_for_current_request,
            request_line_idx,
            &mut diagnostics,
        );
    }

    diagnostics
}

/// Extracts HTTP method from a line if it's a request line
fn extract_method_from_line(line: &str) -> Option<HttpMethod> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    HttpMethod::from_str(parts[0])
}

/// Checks if required headers are missing for a specific HTTP method
fn check_missing_headers_for_method(
    method: HttpMethod,
    headers: &[String],
    line_idx: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // POST, PUT, and PATCH should have Content-Type when sending a body
    match method {
        HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH => {
            let has_content_type = headers.iter().any(|h| h == "content-type");
            if !has_content_type {
                diagnostics.push(
                    Diagnostic::warning(
                        Range::line(line_idx),
                        format!(
                            "{} request should include Content-Type header when sending a body",
                            method.as_str()
                        ),
                    )
                    .with_code("missing-content-type")
                    .with_suggestion(
                        "Add 'Content-Type: application/json' or appropriate content type",
                    ),
                );
            }
        }
        _ => {
            // Other methods don't have strict header requirements
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error(Range::line(5), "Test error");
        assert_eq!(diag.severity, DiagnosticSeverity::Error);
        assert_eq!(diag.message, "Test error");

        let diag = diag.with_code("test-code").with_suggestion("Fix it");
        assert_eq!(diag.code, Some("test-code".to_string()));
        assert_eq!(diag.suggestion, Some("Fix it".to_string()));
    }

    #[test]
    fn test_check_syntax_errors() {
        let doc = "INVALID https://example.com\n";
        let diagnostics = check_syntax_errors(doc);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
        assert!(diagnostics[0].message.contains("Invalid HTTP method"));
    }

    #[test]
    fn test_check_variable_issues_undefined() {
        let doc = "GET https://api.example.com/{{undefinedVar}}\n";
        let context = VariableContext::new(PathBuf::from("."));
        let diagnostics = check_variable_issues(doc, &context);

        assert!(!diagnostics.is_empty());
        let undefined_diag = diagnostics
            .iter()
            .find(|d| d.code.as_deref() == Some("undefined-variable"));
        assert!(undefined_diag.is_some());
    }

    #[test]
    fn test_check_variable_issues_system_variables_ignored() {
        let doc = "GET https://api.example.com/{{$guid}}\n";
        let context = VariableContext::new(PathBuf::from("."));
        let diagnostics = check_variable_issues(doc, &context);

        // System variables should not generate warnings
        let undefined_diag = diagnostics
            .iter()
            .find(|d| d.code.as_deref() == Some("undefined-variable"));
        assert!(undefined_diag.is_none());
    }

    #[test]
    fn test_check_url_format() {
        let doc = "GET api.example.com/users\n";
        let diagnostics = check_url_format(doc);

        assert!(!diagnostics.is_empty());
        assert!(diagnostics[0].message.contains("should start with"));
    }

    #[test]
    fn test_check_header_typo() {
        let doc = "GET https://example.com\nConten-Type: application/json\n";
        let diagnostics = check_header_issues(doc);

        assert!(!diagnostics.is_empty());
        let typo_diag = diagnostics
            .iter()
            .find(|d| d.code.as_deref() == Some("header-typo"));
        assert!(typo_diag.is_some());
        assert!(typo_diag.unwrap().message.contains("Conten-Type"));
    }

    #[test]
    fn test_check_json_body_valid() {
        let doc = r#"POST https://api.example.com
Content-Type: application/json

{"name": "test", "value": 123}
"#;
        let diagnostics = check_json_bodies(doc);

        let json_errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid-json"))
            .collect();
        assert!(json_errors.is_empty());
    }

    #[test]
    fn test_check_json_body_invalid() {
        let doc = r#"POST https://api.example.com
Content-Type: application/json

{invalid json}
"#;
        let diagnostics = check_json_bodies(doc);

        let json_errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.code.as_deref() == Some("invalid-json"))
            .collect();
        assert!(!json_errors.is_empty());
    }

    #[test]
    fn test_check_required_headers_post_without_content_type() {
        let doc = "POST https://api.example.com/users\n\n{\"name\": \"test\"}\n";
        let diagnostics = check_required_headers(doc);

        let missing_ct = diagnostics
            .iter()
            .find(|d| d.code.as_deref() == Some("missing-content-type"));
        assert!(missing_ct.is_some());
    }

    #[test]
    fn test_check_required_headers_get_no_warning() {
        let doc = "GET https://api.example.com/users\n";
        let diagnostics = check_required_headers(doc);

        let missing_ct = diagnostics
            .iter()
            .find(|d| d.code.as_deref() == Some("missing-content-type"));
        assert!(missing_ct.is_none());
    }

    #[test]
    fn test_provide_diagnostics_comprehensive() {
        let doc = r#"INVALID https://example.com
GET api.example.com/{{undefinedVar}}
Conten-Type: application/json

{"invalid": json}
"#;
        let context = VariableContext::new(PathBuf::from("."));
        let diagnostics = provide_diagnostics(doc, &context);

        // Should have multiple diagnostics
        assert!(diagnostics.len() >= 3);

        // Should have syntax error
        assert!(diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("invalid-method")));

        // Should have undefined variable
        assert!(diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("undefined-variable")));

        // Should have header typo
        assert!(diagnostics
            .iter()
            .any(|d| d.code.as_deref() == Some("header-typo")));
    }
}
