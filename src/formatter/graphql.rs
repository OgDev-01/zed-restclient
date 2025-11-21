//! GraphQL formatter for pretty-printing queries and responses.
//!
//! This module provides formatting utilities for GraphQL queries and responses,
//! making them more readable in the REST client output.

use crate::graphql::{GraphQLRequest, GraphQLResponse};

/// Formats a GraphQL query for display.
///
/// This function takes a raw GraphQL query string and formats it with proper
/// indentation and line breaks for better readability.
///
/// # Arguments
///
/// * `query` - The GraphQL query string to format
///
/// # Returns
///
/// A formatted query string with proper indentation.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::graphql::format_graphql_query;
///
/// let query = "query{user{id name}}";
/// let formatted = format_graphql_query(query);
/// assert!(formatted.contains("  user"));
/// ```
pub fn format_graphql_query(query: &str) -> String {
    let mut result = String::new();
    let mut indent_level = 0;
    let indent_size = 2;
    let mut chars = query.chars().peekable();
    let mut in_string = false;
    let mut last_was_newline = false;

    while let Some(ch) = chars.next() {
        // Handle string literals
        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            continue;
        }

        if in_string {
            result.push(ch);
            continue;
        }

        match ch {
            '{' => {
                result.push(ch);
                result.push('\n');
                indent_level += 1;
                result.push_str(&" ".repeat(indent_level * indent_size));
                last_was_newline = true;
            }
            '}' => {
                if !last_was_newline {
                    result.push('\n');
                }
                indent_level = indent_level.saturating_sub(1);
                result.push_str(&" ".repeat(indent_level * indent_size));
                result.push(ch);
                last_was_newline = false;
            }
            '(' => {
                result.push(ch);
                last_was_newline = false;
            }
            ')' => {
                result.push(ch);
                last_was_newline = false;
            }
            '\n' => {
                // Preserve intentional newlines but adjust indentation
                if !last_was_newline {
                    result.push('\n');
                    result.push_str(&" ".repeat(indent_level * indent_size));
                    last_was_newline = true;
                }
            }
            ' ' | '\t' | '\r' => {
                // Skip redundant whitespace after newlines
                if !last_was_newline {
                    // Collapse multiple spaces
                    if !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
            }
            _ => {
                result.push(ch);
                last_was_newline = false;
            }
        }
    }

    result.trim().to_string()
}

/// Formats a GraphQL request (query + variables) for display.
///
/// # Arguments
///
/// * `request` - The GraphQL request to format
///
/// # Returns
///
/// A formatted string containing the query and variables.
pub fn format_graphql_request(request: &GraphQLRequest) -> String {
    let mut output = String::new();

    // Add operation name if present
    if let Some(ref op_name) = request.operation_name {
        output.push_str(&format!("# Operation: {}\n\n", op_name));
    }

    // Format the query
    output.push_str("# Query\n");
    output.push_str(&format_graphql_query(&request.query));
    output.push_str("\n\n");

    // Add variables if present
    if let Some(ref vars) = request.variables {
        output.push_str("# Variables\n");
        match serde_json::to_string_pretty(vars) {
            Ok(formatted) => output.push_str(&formatted),
            Err(_) => output.push_str(&vars.to_string()),
        }
        output.push('\n');
    }

    output
}

/// Formats a GraphQL response for display.
///
/// This function formats the response data and handles GraphQL errors,
/// which are separate from HTTP errors.
///
/// # Arguments
///
/// * `response` - The GraphQL response to format
///
/// # Returns
///
/// A formatted string containing the response data and any errors.
pub fn format_graphql_response(response: &GraphQLResponse) -> String {
    let mut output = String::new();

    // Show errors first if present
    if response.has_errors() {
        output.push_str("# GraphQL Errors\n\n");
        output.push_str(&response.format_errors());
        output.push('\n');
    }

    // Show data if present
    if let Some(ref data) = response.data {
        output.push_str("# Response Data\n\n");
        match serde_json::to_string_pretty(data) {
            Ok(formatted) => output.push_str(&formatted),
            Err(_) => output.push_str(&data.to_string()),
        }
        output.push('\n');
    }

    // Show extensions if present
    if let Some(ref extensions) = response.extensions {
        output.push_str("\n# Extensions\n\n");
        match serde_json::to_string_pretty(extensions) {
            Ok(formatted) => output.push_str(&formatted),
            Err(_) => output.push_str(&extensions.to_string()),
        }
        output.push('\n');
    }

    if output.is_empty() {
        output.push_str("# Empty Response\n");
    }

    output
}

/// Detects GraphQL keywords in a query and returns them for syntax highlighting hints.
///
/// # Arguments
///
/// * `query` - The GraphQL query to analyze
///
/// # Returns
///
/// A vector of tuples containing (keyword, start_position, end_position).
pub fn detect_graphql_keywords(query: &str) -> Vec<(String, usize, usize)> {
    let keywords = [
        "query",
        "mutation",
        "subscription",
        "fragment",
        "on",
        "type",
        "interface",
        "union",
        "enum",
        "input",
        "extend",
        "scalar",
        "directive",
        "schema",
    ];

    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    for keyword in &keywords {
        let mut start = 0;
        while let Some(pos) = query_lower[start..].find(keyword) {
            let absolute_pos = start + pos;

            // Check if it's a whole word (not part of another word)
            let is_start_valid = absolute_pos == 0
                || !query
                    .chars()
                    .nth(absolute_pos - 1)
                    .map_or(false, |c| c.is_alphanumeric() || c == '_');

            let end_pos = absolute_pos + keyword.len();
            let is_end_valid = end_pos >= query.len()
                || !query
                    .chars()
                    .nth(end_pos)
                    .map_or(false, |c| c.is_alphanumeric() || c == '_');

            if is_start_valid && is_end_valid {
                results.push((keyword.to_string(), absolute_pos, end_pos));
            }

            start = absolute_pos + keyword.len();
            if start >= query.len() {
                break;
            }
        }
    }

    // Sort by position
    results.sort_by_key(|(_, start, _)| *start);
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphql::{GraphQLError, GraphQLErrorLocation};

    #[test]
    fn test_format_simple_query() {
        let query = "query{user{id name}}";
        let formatted = format_graphql_query(query);

        assert!(formatted.contains("query"));
        assert!(formatted.contains("user"));
        assert!(formatted.contains("id"));
        assert!(formatted.contains("name"));
        // Check that it has proper structure with newlines
        assert!(formatted.contains('\n'));
    }

    #[test]
    fn test_format_query_preserves_strings() {
        let query = r#"query{user(name:"John Doe"){id}}"#;
        let formatted = format_graphql_query(query);

        assert!(formatted.contains(r#""John Doe""#));
    }

    #[test]
    fn test_format_query_with_parentheses() {
        let query = "query GetUser($id:ID!){user(id:$id){name}}";
        let formatted = format_graphql_query(query);

        assert!(formatted.contains("query GetUser($id:ID!)"));
        assert!(formatted.contains("user(id:$id)"));
    }

    #[test]
    fn test_format_graphql_request_no_variables() {
        let request = GraphQLRequest::new("query { users { id } }".to_string());
        let formatted = format_graphql_request(&request);

        assert!(formatted.contains("# Query"));
        assert!(formatted.contains("users"));
        assert!(!formatted.contains("# Variables"));
    }

    #[test]
    fn test_format_graphql_request_with_variables() {
        let request = GraphQLRequest::with_variables(
            "query($id: ID!) { user(id: $id) { name } }".to_string(),
            serde_json::json!({"id": "123"}),
        );
        let formatted = format_graphql_request(&request);

        assert!(formatted.contains("# Query"));
        assert!(formatted.contains("# Variables"));
        assert!(formatted.contains("\"id\""));
        assert!(formatted.contains("\"123\""));
    }

    #[test]
    fn test_format_graphql_request_with_operation_name() {
        let mut request = GraphQLRequest::new("query GetUser { user { id } }".to_string());
        request.set_operation_name("GetUser".to_string());
        let formatted = format_graphql_request(&request);

        assert!(formatted.contains("# Operation: GetUser"));
    }

    #[test]
    fn test_format_graphql_response_with_data() {
        let response = GraphQLResponse {
            data: Some(serde_json::json!({
                "user": {
                    "id": "123",
                    "name": "John Doe"
                }
            })),
            errors: None,
            extensions: None,
        };

        let formatted = format_graphql_response(&response);

        assert!(formatted.contains("# Response Data"));
        assert!(formatted.contains("\"id\""));
        assert!(formatted.contains("\"123\""));
        assert!(formatted.contains("John Doe"));
    }

    #[test]
    fn test_format_graphql_response_with_errors() {
        let response = GraphQLResponse {
            data: None,
            errors: Some(vec![GraphQLError {
                message: "Field not found".to_string(),
                locations: Some(vec![GraphQLErrorLocation { line: 2, column: 5 }]),
                path: None,
                extensions: None,
            }]),
            extensions: None,
        };

        let formatted = format_graphql_response(&response);

        assert!(formatted.contains("# GraphQL Errors"));
        assert!(formatted.contains("Field not found"));
        assert!(formatted.contains("line 2, column 5"));
    }

    #[test]
    fn test_format_graphql_response_with_extensions() {
        let response = GraphQLResponse {
            data: Some(serde_json::json!({"test": true})),
            errors: None,
            extensions: Some(serde_json::json!({
                "tracing": {
                    "duration": 42
                }
            })),
        };

        let formatted = format_graphql_response(&response);

        assert!(formatted.contains("# Extensions"));
        assert!(formatted.contains("tracing"));
        assert!(formatted.contains("duration"));
    }

    #[test]
    fn test_detect_graphql_keywords() {
        let query = "query GetUser { user { id } }";
        let keywords = detect_graphql_keywords(query);

        assert!(!keywords.is_empty());
        assert_eq!(keywords[0].0, "query");
        assert!(keywords.iter().any(|(kw, _, _)| kw == "query"));
    }

    #[test]
    fn test_detect_graphql_keywords_mutation() {
        let query = "mutation CreateUser { createUser { id } }";
        let keywords = detect_graphql_keywords(query);

        assert!(keywords.iter().any(|(kw, _, _)| kw == "mutation"));
    }

    #[test]
    fn test_format_empty_response() {
        let response = GraphQLResponse {
            data: None,
            errors: None,
            extensions: None,
        };

        let formatted = format_graphql_response(&response);

        assert!(formatted.contains("# Empty Response"));
    }
}
