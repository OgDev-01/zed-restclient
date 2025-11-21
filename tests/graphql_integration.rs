//! Integration tests for GraphQL support in the REST Client.
//!
//! These tests verify that GraphQL queries are correctly parsed, formatted,
//! and prepared for HTTP transport.

use rest_client::formatter::graphql::{
    format_graphql_query, format_graphql_request, format_graphql_response,
};
use rest_client::graphql::parser::{is_graphql_request, parse_graphql_request};
use rest_client::graphql::{GraphQLError, GraphQLErrorLocation, GraphQLRequest, GraphQLResponse};

#[test]
fn test_parse_simple_graphql_query() {
    let body = r#"
query {
  users {
    id
    name
    email
  }
}
    "#;

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.query.contains("users"));
    assert!(request.query.contains("id"));
    assert!(request.query.contains("name"));
    assert!(!request.has_variables());
}

#[test]
fn test_parse_graphql_query_with_variables() {
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

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.query.contains("GetUser"));
    assert!(request.has_variables());

    let vars = request.variables.unwrap();
    assert_eq!(vars["id"], "123");
}

#[test]
fn test_parse_graphql_mutation() {
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

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.query.contains("mutation"));
    assert!(request.query.contains("CreateUser"));
    assert!(request.has_variables());

    let vars = request.variables.unwrap();
    assert!(vars["input"].is_object());
}

#[test]
fn test_graphql_to_json_serialization() {
    let request = GraphQLRequest::with_variables(
        "query($id: ID!) { user(id: $id) { name } }".to_string(),
        serde_json::json!({"id": "123"}),
    );

    let json = request.to_json();
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("query"));
    assert!(json_str.contains("variables"));
    assert!(json_str.contains("123"));

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed["query"].is_string());
    assert!(parsed["variables"].is_object());
}

#[test]
fn test_format_graphql_query_basic() {
    let query = "query{user{id name}}";
    let formatted = format_graphql_query(query);

    assert!(formatted.contains("query"));
    assert!(formatted.contains("user"));
    assert!(formatted.contains("id"));
    assert!(formatted.contains("name"));
    assert!(formatted.contains('\n'));
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
    assert!(formatted.contains("user"));
    assert!(formatted.contains("\"id\""));
}

#[test]
fn test_format_graphql_response_with_data() {
    let response = GraphQLResponse {
        data: Some(serde_json::json!({
            "user": {
                "id": "123",
                "name": "John Doe",
                "email": "john@example.com"
            }
        })),
        errors: None,
        extensions: None,
    };

    let formatted = format_graphql_response(&response);

    assert!(formatted.contains("# Response Data"));
    assert!(formatted.contains("John Doe"));
    assert!(formatted.contains("john@example.com"));
    assert!(!formatted.contains("# GraphQL Errors"));
}

#[test]
fn test_format_graphql_response_with_errors() {
    let response = GraphQLResponse {
        data: None,
        errors: Some(vec![
            GraphQLError {
                message: "Field 'user' not found".to_string(),
                locations: Some(vec![GraphQLErrorLocation { line: 2, column: 5 }]),
                path: Some(vec![serde_json::json!("user")]),
                extensions: None,
            },
            GraphQLError {
                message: "Invalid argument type".to_string(),
                locations: None,
                path: None,
                extensions: Some(serde_json::json!({"code": "INVALID_ARGUMENT"})),
            },
        ]),
        extensions: None,
    };

    let formatted = format_graphql_response(&response);

    assert!(formatted.contains("# GraphQL Errors"));
    assert!(formatted.contains("Field 'user' not found"));
    assert!(formatted.contains("Invalid argument type"));
    assert!(formatted.contains("line 2, column 5"));
}

#[test]
fn test_is_graphql_request_detection() {
    // Test with explicit GraphQL content type
    assert!(is_graphql_request(
        "query { users { id } }",
        Some("application/graphql")
    ));

    // Test with query keyword
    assert!(is_graphql_request("query { test }", None));

    // Test with mutation keyword
    assert!(is_graphql_request("mutation { createUser }", None));

    // Test with subscription keyword
    assert!(is_graphql_request("subscription { onUserCreated }", None));

    // Test with fragment keyword
    assert!(is_graphql_request("fragment UserInfo on User { id }", None));

    // Test negative cases
    assert!(!is_graphql_request(
        r#"{"key": "value"}"#,
        Some("application/json")
    ));
    assert!(!is_graphql_request("GET /api/users", None));
    assert!(!is_graphql_request("<xml>test</xml>", None));
}

#[test]
fn test_graphql_request_with_nested_variables() {
    let body = r#"
mutation CreatePost($input: CreatePostInput!) {
  createPost(input: $input) {
    id
    title
    author {
      id
      name
    }
  }
}

{
  "input": {
    "title": "Hello World",
    "content": "This is my first post",
    "tags": ["intro", "test"],
    "published": true
  }
}
    "#;

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.has_variables());

    let vars = request.variables.unwrap();
    assert!(vars["input"]["tags"].is_array());
    assert_eq!(vars["input"]["published"], true);
}

#[test]
fn test_graphql_operation_name_extraction() {
    let body = "query GetAllUsers { users { id name } }";
    let request = parse_graphql_request(body).unwrap();
    assert_eq!(request.operation_name, Some("GetAllUsers".to_string()));

    let body = "mutation CreateUser { createUser { id } }";
    let request = parse_graphql_request(body).unwrap();
    assert_eq!(request.operation_name, Some("CreateUser".to_string()));

    let body = "query { users { id } }";
    let request = parse_graphql_request(body).unwrap();
    assert_eq!(request.operation_name, None);
}

#[test]
fn test_graphql_syntax_validation() {
    // Valid queries should parse
    assert!(parse_graphql_request("query { user { id } }").is_ok());

    // Unmatched braces should fail
    let result = parse_graphql_request("query { user { id }");
    assert!(result.is_err());

    // Unmatched parentheses should fail
    let result = parse_graphql_request("query GetUser($id: ID! { user }");
    assert!(result.is_err());

    // Empty body should fail
    let result = parse_graphql_request("");
    assert!(result.is_err());
}

#[test]
fn test_graphql_variables_validation() {
    // Valid JSON object variables should work
    let body = r#"
query { user { id } }

{
  "id": "123"
}
    "#;
    assert!(parse_graphql_request(body).is_ok());

    // Invalid JSON should fail
    let body = r#"
query { user { id } }

{ "id": 123, invalid }
    "#;
    assert!(parse_graphql_request(body).is_err());

    // Array variables should fail (must be object)
    let body = r#"
query { user { id } }

["not", "an", "object"]
    "#;
    assert!(parse_graphql_request(body).is_err());
}

#[test]
fn test_shorthand_query_syntax() {
    // GraphQL allows shorthand query syntax without "query" keyword
    let body = r#"
{
  users {
    id
    name
  }
}
    "#;

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.query.contains("users"));
}

#[test]
fn test_graphql_with_comments_in_query() {
    let body = r#"
query GetUser($id: ID!) {
  # Get user by ID
  user(id: $id) {
    id
    name
    # User's email address
    email
  }
}

{
  "id": "123"
}
    "#;

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.has_variables());
}

#[test]
fn test_graphql_response_serialization() {
    let response = GraphQLResponse {
        data: Some(serde_json::json!({"user": {"id": "123"}})),
        errors: None,
        extensions: None,
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: GraphQLResponse = serde_json::from_str(&json).unwrap();

    assert!(deserialized.data.is_some());
    assert!(!deserialized.has_errors());
}

#[test]
fn test_graphql_error_formatting() {
    let error = GraphQLError {
        message: "Field not found".to_string(),
        locations: Some(vec![GraphQLErrorLocation {
            line: 5,
            column: 10,
        }]),
        path: Some(vec![
            serde_json::json!("user"),
            serde_json::json!("posts"),
            serde_json::json!(0),
        ]),
        extensions: Some(serde_json::json!({"code": "FIELD_NOT_FOUND"})),
    };

    let response = GraphQLResponse {
        data: None,
        errors: Some(vec![error]),
        extensions: None,
    };

    let formatted = response.format_errors();
    assert!(formatted.contains("Field not found"));
    assert!(formatted.contains("line 5, column 10"));
    assert!(formatted.contains("user"));
}

#[test]
fn test_graphql_request_with_multiple_operations() {
    let body = r#"
query GetUser {
  user { id name }
}

query GetPosts {
  posts { id title }
}
    "#;

    // Should parse successfully
    let result = parse_graphql_request(body);
    assert!(result.is_ok());
}

#[test]
fn test_graphql_with_fragments() {
    let body = r#"
query GetUser {
  user {
    ...UserFields
  }
}

fragment UserFields on User {
  id
  name
  email
}
    "#;

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.query.contains("fragment"));
    assert!(request.query.contains("UserFields"));
}

#[test]
fn test_graphql_request_to_pretty_json() {
    let request = GraphQLRequest::with_variables(
        "query($id: ID!) { user(id: $id) { name } }".to_string(),
        serde_json::json!({"id": "123", "includeEmail": true}),
    );

    let pretty_json = request.to_json_pretty();
    assert!(pretty_json.is_ok());

    let json_str = pretty_json.unwrap();
    // Pretty JSON should have newlines and indentation
    assert!(json_str.contains('\n'));
    assert!(json_str.lines().count() > 1);
}

#[test]
fn test_complex_graphql_query_with_directives() {
    let body = r#"
query GetUser($id: ID!, $includePosts: Boolean!) {
  user(id: $id) {
    id
    name
    posts @include(if: $includePosts) {
      id
      title
    }
  }
}

{
  "id": "123",
  "includePosts": true
}
    "#;

    let result = parse_graphql_request(body);
    assert!(result.is_ok());

    let request = result.unwrap();
    assert!(request.query.contains("@include"));
    assert!(request.has_variables());

    let vars = request.variables.unwrap();
    assert_eq!(vars["includePosts"], true);
}

#[test]
fn test_graphql_response_with_extensions() {
    let response = GraphQLResponse {
        data: Some(serde_json::json!({"user": {"id": "123"}})),
        errors: None,
        extensions: Some(serde_json::json!({
            "tracing": {
                "version": 1,
                "startTime": "2023-01-01T00:00:00Z",
                "endTime": "2023-01-01T00:00:01Z",
                "duration": 1000
            }
        })),
    };

    let formatted = format_graphql_response(&response);
    assert!(formatted.contains("# Extensions"));
    assert!(formatted.contains("tracing"));
    assert!(formatted.contains("duration"));
}

#[test]
fn test_graphql_empty_response() {
    let response = GraphQLResponse {
        data: None,
        errors: None,
        extensions: None,
    };

    let formatted = format_graphql_response(&response);
    assert!(formatted.contains("# Empty Response"));
    assert_eq!(response.error_count(), 0);
    assert!(!response.has_errors());
}
