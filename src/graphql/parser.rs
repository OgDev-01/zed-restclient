//! GraphQL request parser.
//!
//! This module provides parsing functionality for GraphQL queries in HTTP request
//! bodies. It separates the GraphQL query from the variables section and validates
//! basic syntax.
//!
//! # Format
//!
//! GraphQL requests in `.http` files can be written in two formats:
//!
//! 1. **Query with separate variables section:**
//! ```graphql
//! query GetUser($id: ID!) {
//!   user(id: $id) {
//!     id
//!     name
//!   }
//! }
//!
//! {
//!   "id": "123"
//! }
//! ```
//!
//! 2. **Query only (no variables):**
//! ```graphql
//! query {
//!   users {
//!     id
//!     name
//!   }
//! }
//! ```

use super::{GraphQLRequest, ParseError};
use serde_json::Value;

/// Parses a GraphQL request from a request body string.
///
/// This function separates the GraphQL query from any variables section,
/// validates basic syntax, and returns a structured GraphQLRequest.
///
/// # Format
///
/// The body should contain:
/// 1. A GraphQL operation (query, mutation, subscription, or fragment)
/// 2. Optionally, a blank line followed by a JSON object with variables
///
/// # Arguments
///
/// * `body` - The request body containing the GraphQL query and optional variables
///
/// # Returns
///
/// `Ok(GraphQLRequest)` if parsing succeeds, or `Err(ParseError)` if the body
/// is invalid or contains syntax errors.
///
/// # Examples
///
/// ```
/// use rest_client::graphql::parser::parse_graphql_request;
///
/// let body = r#"
/// query GetUser($id: ID!) {
///   user(id: $id) {
///     name
///     email
///   }
/// }
///
/// {
///   "id": "123"
/// }
/// "#;
///
/// let request = parse_graphql_request(body).unwrap();
/// assert!(request.query.contains("GetUser"));
/// assert!(request.has_variables());
/// ```
pub fn parse_graphql_request(body: &str) -> Result<GraphQLRequest, ParseError> {
    // Trim and check for empty body
    let body = body.trim();
    if body.is_empty() {
        return Err(ParseError::EmptyBody);
    }

    // Split body into query and potential variables section
    let (query_part, variables_part) = split_query_and_variables(body)?;

    // Validate GraphQL syntax
    validate_graphql_syntax(&query_part)?;

    // Parse variables if present
    let variables = if let Some(vars_str) = variables_part {
        Some(parse_variables(&vars_str)?)
    } else {
        None
    };

    // Extract operation name if present
    let operation_name = extract_operation_name(&query_part);

    let mut request = GraphQLRequest::new(query_part);
    request.variables = variables;
    if let Some(name) = operation_name {
        request.set_operation_name(name);
    }

    Ok(request)
}

/// Splits the body into query and variables sections.
///
/// The query section ends when we encounter a line that starts with `{` or `[`
/// (indicating JSON) after some blank lines, or at the end of the body.
fn split_query_and_variables(body: &str) -> Result<(String, Option<String>), ParseError> {
    let lines: Vec<&str> = body.lines().collect();

    // Find where the query ends and variables begin
    // Variables start at the first line that looks like JSON after the query
    let mut query_end_idx = lines.len();
    let mut found_query_content = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines at the beginning
        if !found_query_content && trimmed.is_empty() {
            continue;
        }

        // Check if this looks like the start of GraphQL
        if !found_query_content && is_graphql_keyword(trimmed) {
            found_query_content = true;
            continue;
        }

        // If we haven't found query content yet, check if this is a non-empty line
        if !found_query_content && !trimmed.is_empty() {
            found_query_content = true;
            continue;
        }

        // After query content, look for JSON (variables section)
        if found_query_content && (trimmed.starts_with('{') || trimmed.starts_with('[')) {
            // Check if the previous line was blank or this is clearly separate JSON
            let is_after_blank = i > 0 && lines[i - 1].trim().is_empty();
            let looks_like_json = trimmed.starts_with('{') && !trimmed.contains(':');

            // If it looks like a new JSON block (not part of the query)
            if is_after_blank || (i > 0 && looks_like_json) {
                query_end_idx = i;
                break;
            }
        }
    }

    // Extract query and variables parts
    let query_lines: Vec<&str> = lines[..query_end_idx].iter().copied().collect();
    let query = query_lines.join("\n").trim().to_string();

    let variables = if query_end_idx < lines.len() {
        let var_lines: Vec<&str> = lines[query_end_idx..].iter().copied().collect();
        let vars_str = var_lines.join("\n");
        let trimmed = vars_str.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    } else {
        None
    };

    if query.is_empty() {
        return Err(ParseError::EmptyBody);
    }

    Ok((query, variables))
}

/// Checks if a line starts with a GraphQL keyword.
fn is_graphql_keyword(line: &str) -> bool {
    let keywords = ["query", "mutation", "subscription", "fragment"];
    let trimmed = line.trim().to_lowercase();

    keywords.iter().any(|&keyword| trimmed.starts_with(keyword))
}

/// Validates basic GraphQL syntax.
///
/// This performs simple validation including:
/// - Checking for balanced braces, parentheses, and brackets
/// - Verifying the presence of GraphQL keywords
fn validate_graphql_syntax(query: &str) -> Result<(), ParseError> {
    // Check for balanced delimiters
    validate_balanced_delimiters(query)?;

    // Check for at least one GraphQL keyword
    let has_keyword = query.lines().any(|line| is_graphql_keyword(line.trim()));

    if !has_keyword {
        // Allow queries that start with { (shorthand query syntax)
        if !query.trim().starts_with('{') {
            return Err(ParseError::InvalidSyntax(
                "Query must start with a GraphQL keyword (query, mutation, subscription, fragment) or '{'".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validates that braces, parentheses, and brackets are balanced.
fn validate_balanced_delimiters(query: &str) -> Result<(), ParseError> {
    let mut stack: Vec<char> = Vec::new();
    let mut in_string = false;
    let mut escape_next = false;

    for ch in query.chars() {
        // Handle string escaping
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        // Handle string literals
        if ch == '"' {
            in_string = !in_string;
            continue;
        }

        // Skip delimiter checking inside strings
        if in_string {
            continue;
        }

        // Check delimiters
        match ch {
            '{' | '(' | '[' => stack.push(ch),
            '}' => {
                if stack.pop() != Some('{') {
                    return Err(ParseError::UnmatchedDelimiter("}".to_string()));
                }
            }
            ')' => {
                if stack.pop() != Some('(') {
                    return Err(ParseError::UnmatchedDelimiter(")".to_string()));
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return Err(ParseError::UnmatchedDelimiter("]".to_string()));
                }
            }
            _ => {}
        }
    }

    if !stack.is_empty() {
        let expected = match stack.last() {
            Some('{') => "}",
            Some('(') => ")",
            Some('[') => "]",
            _ => "unknown",
        };
        return Err(ParseError::UnmatchedDelimiter(expected.to_string()));
    }

    Ok(())
}

/// Parses the variables JSON string.
fn parse_variables(vars_str: &str) -> Result<Value, ParseError> {
    let value: Value =
        serde_json::from_str(vars_str).map_err(|e| ParseError::InvalidVariables(e.to_string()))?;

    // Variables must be an object, not an array or primitive
    if !value.is_object() {
        return Err(ParseError::VariablesNotObject);
    }

    Ok(value)
}

/// Extracts the operation name from a GraphQL query.
///
/// Returns the name if found, or None for anonymous operations.
fn extract_operation_name(query: &str) -> Option<String> {
    // Look for pattern: query OperationName or mutation OperationName
    let keywords = ["query", "mutation", "subscription"];

    for line in query.lines() {
        let trimmed = line.trim();
        for keyword in &keywords {
            if trimmed.to_lowercase().starts_with(keyword) {
                // Extract the operation name (word after keyword, before '(' or '{')
                let rest = trimmed[keyword.len()..].trim();
                if rest.is_empty() {
                    continue;
                }

                // Get the first word
                let name = rest
                    .split(|c: char| c == '(' || c == '{' || c.is_whitespace())
                    .next()
                    .unwrap_or("")
                    .trim();

                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }

    None
}

/// Detects if a request body contains GraphQL content.
///
/// This is used by the executor to determine if a request should be treated
/// as a GraphQL request.
///
/// # Arguments
///
/// * `body` - The request body to check
/// * `content_type` - Optional Content-Type header value
///
/// # Returns
///
/// `true` if the body appears to contain GraphQL, `false` otherwise.
pub fn is_graphql_request(body: &str, content_type: Option<&str>) -> bool {
    // Check Content-Type first
    if let Some(ct) = content_type {
        if ct.contains("application/graphql") || ct.contains("application/json") {
            // If it's explicitly GraphQL or JSON, check the body
            return body.trim_start().starts_with("query")
                || body.trim_start().starts_with("mutation")
                || body.trim_start().starts_with("subscription")
                || body.trim_start().starts_with("fragment");
        }
    }

    // Otherwise, check if body starts with GraphQL keywords
    let trimmed = body.trim_start();
    trimmed.starts_with("query")
        || trimmed.starts_with("mutation")
        || trimmed.starts_with("subscription")
        || trimmed.starts_with("fragment")
        || (trimmed.starts_with('{') && trimmed.contains("__typename"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let body = r#"
query {
  users {
    id
    name
  }
}
        "#;

        let request = parse_graphql_request(body).unwrap();
        assert!(request.query.contains("query"));
        assert!(request.query.contains("users"));
        assert!(!request.has_variables());
    }

    #[test]
    fn test_parse_query_with_variables() {
        let body = r#"
query GetUser($id: ID!) {
  user(id: $id) {
    id
    name
    email
  }
}

{
  "id": "123"
}
        "#;

        let request = parse_graphql_request(body).unwrap();
        assert!(request.query.contains("GetUser"));
        assert!(request.has_variables());

        let vars = request.variables.unwrap();
        assert_eq!(vars["id"], "123");
    }

    #[test]
    fn test_parse_mutation() {
        let body = r#"
mutation CreateUser($input: UserInput!) {
  createUser(input: $input) {
    id
    name
  }
}

{
  "input": {
    "name": "John Doe",
    "email": "john@example.com"
  }
}
        "#;

        let request = parse_graphql_request(body).unwrap();
        assert!(request.query.contains("mutation"));
        assert!(request.query.contains("CreateUser"));
        assert!(request.has_variables());
    }

    #[test]
    fn test_parse_empty_body() {
        let result = parse_graphql_request("");
        assert!(matches!(result, Err(ParseError::EmptyBody)));

        let result = parse_graphql_request("   \n  \n  ");
        assert!(matches!(result, Err(ParseError::EmptyBody)));
    }

    #[test]
    fn test_validate_unmatched_braces() {
        let body = r#"
query {
  user {
    id
  }
        "#;

        let result = parse_graphql_request(body);
        assert!(matches!(result, Err(ParseError::UnmatchedDelimiter(_))));
    }

    #[test]
    fn test_validate_unmatched_parentheses() {
        let body = r#"
query GetUser($id: ID! {
  user(id: $id) {
    id
  }
}
        "#;

        let result = parse_graphql_request(body);
        assert!(matches!(result, Err(ParseError::UnmatchedDelimiter(_))));
    }

    #[test]
    fn test_invalid_variables_json() {
        let body = r#"
query {
  user {
    id
  }
}

{
  "id": 123,
  invalid json
}
        "#;

        let result = parse_graphql_request(body);
        assert!(matches!(result, Err(ParseError::InvalidVariables(_))));
    }

    #[test]
    fn test_variables_not_object() {
        let body = r#"
query {
  user {
    id
  }
}

["not", "an", "object"]
        "#;

        let result = parse_graphql_request(body);
        assert!(matches!(result, Err(ParseError::VariablesNotObject)));
    }

    #[test]
    fn test_extract_operation_name() {
        assert_eq!(
            extract_operation_name("query GetUser { user { id } }"),
            Some("GetUser".to_string())
        );

        assert_eq!(
            extract_operation_name("mutation CreateUser($input: UserInput!) { }"),
            Some("CreateUser".to_string())
        );

        assert_eq!(extract_operation_name("query { user { id } }"), None);

        assert_eq!(
            extract_operation_name("subscription OnUserCreated { }"),
            Some("OnUserCreated".to_string())
        );
    }

    #[test]
    fn test_is_graphql_request() {
        assert!(is_graphql_request(
            "query { users { id } }",
            Some("application/json")
        ));

        assert!(is_graphql_request(
            "mutation { createUser { id } }",
            Some("application/graphql")
        ));

        assert!(is_graphql_request("query { test }", None));

        assert!(!is_graphql_request(
            r#"{"key": "value"}"#,
            Some("application/json")
        ));

        assert!(!is_graphql_request("GET /api/users", None));
    }

    #[test]
    fn test_shorthand_query_syntax() {
        let body = r#"
{
  users {
    id
    name
  }
}
        "#;

        let request = parse_graphql_request(body).unwrap();
        assert!(request.query.contains("users"));
    }

    #[test]
    fn test_balanced_delimiters_with_strings() {
        let body = r#"
query {
  user(name: "User {with} braces") {
    id
  }
}
        "#;

        let result = parse_graphql_request(body);
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_query_with_nested_objects() {
        let body = r#"
query GetUserPosts($userId: ID!, $limit: Int) {
  user(id: $userId) {
    id
    name
    posts(limit: $limit) {
      id
      title
      author {
        name
      }
    }
  }
}

{
  "userId": "user-123",
  "limit": 10
}
        "#;

        let request = parse_graphql_request(body).unwrap();
        assert!(request.query.contains("GetUserPosts"));
        assert!(request.has_variables());

        let vars = request.variables.unwrap();
        assert_eq!(vars["userId"], "user-123");
        assert_eq!(vars["limit"], 10);
    }
}
