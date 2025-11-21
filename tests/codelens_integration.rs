//! Integration tests for CodeLens provider
//!
//! These tests validate that the CodeLens provider correctly identifies
//! request blocks and creates appropriate CodeLens entries.

use rest_client::language_server::codelens::provide_code_lens;

#[test]
fn test_codelens_simple_get_request() {
    let doc = "GET https://api.example.com/users";
    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 1);
    assert_eq!(lenses[0].range.start.line, 0);
    assert!(lenses[0].command.is_some());

    let command = lenses[0].command.as_ref().unwrap();
    assert_eq!(command.command, "rest-client.send");
    assert!(command.title.contains("Send Request"));
}

#[test]
fn test_codelens_multiple_requests_with_delimiters() {
    let doc = r#"GET https://api.example.com/users

###

POST https://api.example.com/users
Content-Type: application/json

{"name": "Alice"}

###

DELETE https://api.example.com/users/123"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 3, "Should have 3 CodeLens for 3 requests");

    // All should have the send command
    for lens in &lenses {
        assert!(lens.command.is_some());
        assert_eq!(lens.command.as_ref().unwrap().command, "rest-client.send");
    }

    // Line numbers should be correct (0, 4, 11 in zero-based indexing)
    assert_eq!(lenses[0].range.start.line, 0);
    assert_eq!(lenses[1].range.start.line, 4);
    assert_eq!(lenses[2].range.start.line, 11);
}

#[test]
fn test_codelens_with_name_annotation() {
    let doc = r#"# @name GetAllUsers
GET https://api.example.com/users

###

# @name CreateUser
POST https://api.example.com/users
Content-Type: application/json

{"name": "Bob"}"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 2);

    let first_command = lenses[0].command.as_ref().unwrap();
    assert!(
        first_command.title.contains("GetAllUsers"),
        "Title should include the @name annotation: {}",
        first_command.title
    );

    let second_command = lenses[1].command.as_ref().unwrap();
    assert!(
        second_command.title.contains("CreateUser"),
        "Title should include the @name annotation: {}",
        second_command.title
    );
}

#[test]
fn test_codelens_with_comments_before_request() {
    let doc = r#"# This is a regular comment
// Another comment explaining the request
# More documentation
GET https://api.example.com/users
Accept: application/json"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 1);
    // CodeLens should appear on the GET line (line 3)
    assert_eq!(lenses[0].range.start.line, 3);
}

#[test]
fn test_codelens_empty_blocks_ignored() {
    let doc = r#"GET https://api.example.com/users

###

# Just a comment block with no request
// More comments

###

POST https://api.example.com/data"#;

    let lenses = provide_code_lens(doc);

    // Should only have 2 lenses (empty comment block is ignored)
    assert_eq!(lenses.len(), 2);
}

#[test]
fn test_codelens_no_requests() {
    let doc = r#"# Just comments
// No actual requests here
# More documentation"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(
        lenses.len(),
        0,
        "No CodeLens should appear for comment-only file"
    );
}

#[test]
fn test_codelens_all_http_methods() {
    let methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

    for method in methods {
        let doc = format!("{} https://api.example.com/endpoint", method);
        let lenses = provide_code_lens(&doc);

        assert_eq!(lenses.len(), 1, "Method {} should have CodeLens", method);
        assert!(lenses[0].command.is_some());
    }
}

#[test]
fn test_codelens_complex_request_with_headers_and_body() {
    let doc = r#"POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer token123
Accept: application/json
X-Custom-Header: value

{
  "name": "Charlie",
  "email": "charlie@example.com",
  "age": 30
}"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 1);
    assert_eq!(
        lenses[0].range.start.line, 0,
        "CodeLens should be on first line"
    );
}

#[test]
fn test_codelens_request_with_query_parameters() {
    let doc = r#"GET https://api.example.com/search?q=rust&limit=10&offset=0
Accept: application/json"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 1);
    assert!(lenses[0].command.is_some());
}

#[test]
fn test_codelens_multiple_named_requests() {
    let doc = r#"# @name HealthCheck
GET https://api.example.com/health

###

# @name ListUsers
# Get all users from the system
GET https://api.example.com/users?limit=100

###

# @name CreateProduct
POST https://api.example.com/products
Content-Type: application/json

{"title": "New Product", "price": 29.99}

###

# @name UpdateProduct
PUT https://api.example.com/products/123
Content-Type: application/json

{"price": 24.99}"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 4);

    // Verify all have proper names
    assert!(lenses[0]
        .command
        .as_ref()
        .unwrap()
        .title
        .contains("HealthCheck"));
    assert!(lenses[1]
        .command
        .as_ref()
        .unwrap()
        .title
        .contains("ListUsers"));
    assert!(lenses[2]
        .command
        .as_ref()
        .unwrap()
        .title
        .contains("CreateProduct"));
    assert!(lenses[3]
        .command
        .as_ref()
        .unwrap()
        .title
        .contains("UpdateProduct"));
}

#[test]
fn test_codelens_with_variables() {
    let doc = r#"@baseUrl = https://api.example.com

###

GET {{baseUrl}}/users
Authorization: Bearer {{token}}"#;

    let lenses = provide_code_lens(doc);

    // Should have 1 lens for the GET request (variable assignment is not a request)
    assert_eq!(lenses.len(), 1);
    assert_eq!(lenses[0].range.start.line, 4);
}

#[test]
fn test_codelens_mixed_comment_styles() {
    let doc = r#"# Hash-style comment
// Double-slash comment
# @name MixedComments
GET https://api.example.com/test"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 1);
    assert!(lenses[0]
        .command
        .as_ref()
        .unwrap()
        .title
        .contains("MixedComments"));
}

#[test]
fn test_codelens_whitespace_handling() {
    let doc = r#"


GET https://api.example.com/users


###


POST https://api.example.com/data


"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 2, "Should handle whitespace correctly");
}

#[test]
fn test_codelens_realistic_api_file() {
    let doc = r#"# API Testing File for Example Service
# Base URL: https://api.example.com

### Authentication

# @name Login
POST https://api.example.com/auth/login
Content-Type: application/json

{
  "username": "testuser",
  "password": "testpass"
}

### User Management

# @name GetCurrentUser
GET https://api.example.com/users/me
Authorization: Bearer {{authToken}}

###

# @name UpdateProfile
PUT https://api.example.com/users/me
Authorization: Bearer {{authToken}}
Content-Type: application/json

{
  "name": "Updated Name",
  "bio": "New bio"
}

### Data Operations

# @name FetchData
GET https://api.example.com/data?filter=active&sort=desc
Authorization: Bearer {{authToken}}

###

# @name DeleteData
DELETE https://api.example.com/data/{{dataId}}
Authorization: Bearer {{authToken}}"#;

    let lenses = provide_code_lens(doc);

    // Should have 5 CodeLens entries for the 5 actual requests
    assert_eq!(lenses.len(), 5);

    // Verify named requests
    let titles: Vec<String> = lenses
        .iter()
        .filter_map(|l| l.command.as_ref().map(|c| c.title.clone()))
        .collect();

    assert!(titles.iter().any(|t| t.contains("Login")));
    assert!(titles.iter().any(|t| t.contains("GetCurrentUser")));
    assert!(titles.iter().any(|t| t.contains("UpdateProfile")));
    assert!(titles.iter().any(|t| t.contains("FetchData")));
    assert!(titles.iter().any(|t| t.contains("DeleteData")));
}

#[test]
fn test_codelens_case_sensitivity() {
    // HTTP methods should be case-sensitive (uppercase only)
    let doc = r#"get https://api.example.com/lowercase
GET https://api.example.com/uppercase"#;

    let lenses = provide_code_lens(doc);

    // Should only recognize the uppercase GET
    assert_eq!(lenses.len(), 1);
    assert_eq!(lenses[0].range.start.line, 1);
}

#[test]
fn test_codelens_delimiter_edge_cases() {
    let doc = r#"GET https://api.example.com/1
###
###
GET https://api.example.com/2
###
###
###
GET https://api.example.com/3"#;

    let lenses = provide_code_lens(doc);

    // Should handle multiple consecutive delimiters
    assert_eq!(lenses.len(), 3);
}

#[test]
fn test_codelens_graphql_request() {
    let doc = r#"# @name GraphQLQuery
POST https://api.example.com/graphql
Content-Type: application/json

{
  "query": "{ users { id name email } }"
}"#;

    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 1);
    assert!(lenses[0]
        .command
        .as_ref()
        .unwrap()
        .title
        .contains("GraphQLQuery"));
}

#[test]
fn test_codelens_empty_document() {
    let doc = "";
    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 0);
}

#[test]
fn test_codelens_only_delimiter() {
    let doc = "###";
    let lenses = provide_code_lens(doc);

    assert_eq!(lenses.len(), 0);
}
