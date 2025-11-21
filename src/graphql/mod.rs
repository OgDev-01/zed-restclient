//! GraphQL support for REST Client.
//!
//! This module provides parsing, formatting, and execution support for GraphQL
//! queries over HTTP. It implements the GraphQL over HTTP specification,
//! allowing users to write GraphQL queries in `.http` files and execute them
//! against GraphQL endpoints.
//!
//! # Features
//!
//! - Parse GraphQL queries with variables from request bodies
//! - Validate basic GraphQL syntax (keywords, braces, parentheses)
//! - Format GraphQL requests as JSON for HTTP transport
//! - Pretty-print GraphQL queries for readability
//! - Handle GraphQL errors in responses
//!
//! # GraphQL over HTTP
//!
//! GraphQL requests are sent via HTTP POST with a JSON body containing:
//! - `query`: The GraphQL query string
//! - `variables`: Optional JSON object with variable values
//! - `operationName`: Optional operation name for multi-operation documents
//!
//! # Example
//!
//! ```http
//! POST https://api.example.com/graphql
//! Content-Type: application/json
//!
//! query GetUser($id: ID!) {
//!   user(id: $id) {
//!     id
//!     name
//!     email
//!   }
//! }
//!
//! {
//!   "id": "123"
//! }
//! ```

pub mod parser;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a parsed GraphQL request ready for HTTP transport.
///
/// This structure separates the GraphQL query from its variables, making it
/// easy to serialize as JSON for sending over HTTP according to the GraphQL
/// over HTTP specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLRequest {
    /// The GraphQL query or mutation string.
    ///
    /// This should be a valid GraphQL operation (query, mutation, subscription,
    /// or fragment definition).
    pub query: String,

    /// Optional variables as a JSON object.
    ///
    /// Variables are passed separately from the query and must be valid JSON.
    /// The keys should match variable names in the query (without the $ prefix).
    pub variables: Option<serde_json::Value>,

    /// Optional operation name for multi-operation documents.
    ///
    /// When a GraphQL document contains multiple named operations, this field
    /// specifies which one to execute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
}

impl GraphQLRequest {
    /// Creates a new GraphQL request with just a query.
    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL query string
    ///
    /// # Returns
    ///
    /// A new `GraphQLRequest` with no variables or operation name.
    pub fn new(query: String) -> Self {
        Self {
            query,
            variables: None,
            operation_name: None,
        }
    }

    /// Creates a new GraphQL request with query and variables.
    ///
    /// # Arguments
    ///
    /// * `query` - The GraphQL query string
    /// * `variables` - JSON object containing variable values
    ///
    /// # Returns
    ///
    /// A new `GraphQLRequest` with the specified query and variables.
    pub fn with_variables(query: String, variables: serde_json::Value) -> Self {
        Self {
            query,
            variables: Some(variables),
            operation_name: None,
        }
    }

    /// Sets the operation name for this request.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the operation to execute
    pub fn set_operation_name(&mut self, name: String) {
        self.operation_name = Some(name);
    }

    /// Converts this GraphQL request to a JSON string for HTTP transport.
    ///
    /// # Returns
    ///
    /// A JSON string representation of the request, or an error if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this GraphQL request to a pretty-printed JSON string.
    ///
    /// # Returns
    ///
    /// A formatted JSON string representation of the request, or an error if serialization fails.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Checks if this request has variables.
    ///
    /// # Returns
    ///
    /// `true` if variables are present and non-null, `false` otherwise.
    pub fn has_variables(&self) -> bool {
        self.variables.is_some()
    }
}

/// Errors that can occur during GraphQL parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// The request body is empty or contains only whitespace.
    EmptyBody,

    /// The GraphQL query syntax is invalid.
    ///
    /// Contains a description of the syntax error.
    InvalidSyntax(String),

    /// The variables section contains invalid JSON.
    ///
    /// Contains the JSON parsing error message.
    InvalidVariables(String),

    /// Missing closing delimiter for a GraphQL construct.
    ///
    /// Contains information about what was expected (e.g., "}", ")", "]").
    UnmatchedDelimiter(String),

    /// Variables were provided but are not a JSON object.
    ///
    /// GraphQL variables must be a JSON object, not an array or primitive.
    VariablesNotObject,

    /// The query contains an unknown GraphQL keyword or construct.
    UnknownConstruct(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyBody => {
                write!(f, "GraphQL request body is empty")
            }
            ParseError::InvalidSyntax(msg) => {
                write!(f, "Invalid GraphQL syntax: {}", msg)
            }
            ParseError::InvalidVariables(msg) => {
                write!(f, "Invalid GraphQL variables (must be valid JSON): {}", msg)
            }
            ParseError::UnmatchedDelimiter(delimiter) => {
                write!(
                    f,
                    "Unmatched delimiter in GraphQL query: expected {}",
                    delimiter
                )
            }
            ParseError::VariablesNotObject => {
                write!(
                    f,
                    "GraphQL variables must be a JSON object, not an array or primitive"
                )
            }
            ParseError::UnknownConstruct(construct) => {
                write!(f, "Unknown GraphQL construct: {}", construct)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Represents a GraphQL error from a server response.
///
/// GraphQL responses can contain an "errors" array even when the HTTP status
/// is 200 OK. This structure represents a single error from that array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    /// The error message.
    pub message: String,

    /// Optional locations in the query where the error occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locations: Option<Vec<GraphQLErrorLocation>>,

    /// Optional path to the field that caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<serde_json::Value>>,

    /// Optional extensions with additional error information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

/// Represents a location in a GraphQL query where an error occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLErrorLocation {
    /// Line number (1-based) in the query.
    pub line: u32,

    /// Column number (1-based) in the query.
    pub column: u32,
}

/// Represents a GraphQL response that may contain errors.
///
/// GraphQL uses a different error model than HTTP - even successful HTTP
/// responses (200 OK) may contain errors in the response body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse {
    /// The response data (may be null if errors occurred).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Array of errors that occurred during execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GraphQLError>>,

    /// Optional extensions with additional response metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<serde_json::Value>,
}

impl GraphQLResponse {
    /// Checks if this response contains any errors.
    ///
    /// # Returns
    ///
    /// `true` if the errors array is present and non-empty, `false` otherwise.
    pub fn has_errors(&self) -> bool {
        self.errors.as_ref().map_or(false, |e| !e.is_empty())
    }

    /// Gets the number of errors in this response.
    ///
    /// # Returns
    ///
    /// The count of errors, or 0 if no errors are present.
    pub fn error_count(&self) -> usize {
        self.errors.as_ref().map_or(0, |e| e.len())
    }

    /// Formats the errors as a human-readable string.
    ///
    /// # Returns
    ///
    /// A formatted string with all error messages, or an empty string if no errors.
    pub fn format_errors(&self) -> String {
        match &self.errors {
            Some(errors) if !errors.is_empty() => {
                let mut output = String::from("GraphQL Errors:\n\n");
                for (i, error) in errors.iter().enumerate() {
                    output.push_str(&format!("{}. {}\n", i + 1, error.message));

                    if let Some(locations) = &error.locations {
                        for loc in locations {
                            output.push_str(&format!(
                                "   at line {}, column {}\n",
                                loc.line, loc.column
                            ));
                        }
                    }

                    if let Some(path) = &error.path {
                        let path_str = path
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(" > ");
                        output.push_str(&format!("   path: {}\n", path_str));
                    }

                    output.push('\n');
                }
                output
            }
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_request_new() {
        let query = "query { user { id name } }".to_string();
        let request = GraphQLRequest::new(query.clone());

        assert_eq!(request.query, query);
        assert_eq!(request.variables, None);
        assert_eq!(request.operation_name, None);
        assert!(!request.has_variables());
    }

    #[test]
    fn test_graphql_request_with_variables() {
        let query = "query($id: ID!) { user(id: $id) { name } }".to_string();
        let variables = serde_json::json!({"id": "123"});
        let request = GraphQLRequest::with_variables(query.clone(), variables.clone());

        assert_eq!(request.query, query);
        assert_eq!(request.variables, Some(variables));
        assert!(request.has_variables());
    }

    #[test]
    fn test_graphql_request_to_json() {
        let request = GraphQLRequest::new("query { test }".to_string());
        let json = request.to_json().unwrap();

        assert!(json.contains("query { test }"));
        assert!(json.contains("\"query\""));
    }

    #[test]
    fn test_graphql_request_serialization() {
        let request = GraphQLRequest::with_variables(
            "query($id: ID!) { user(id: $id) { name } }".to_string(),
            serde_json::json!({"id": "123"}),
        );

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: GraphQLRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.query, request.query);
        assert_eq!(deserialized.variables, request.variables);
    }

    #[test]
    fn test_parse_error_display() {
        assert_eq!(
            ParseError::EmptyBody.to_string(),
            "GraphQL request body is empty"
        );

        assert_eq!(
            ParseError::InvalidSyntax("unexpected token".to_string()).to_string(),
            "Invalid GraphQL syntax: unexpected token"
        );

        assert_eq!(
            ParseError::UnmatchedDelimiter("}".to_string()).to_string(),
            "Unmatched delimiter in GraphQL query: expected }"
        );
    }

    #[test]
    fn test_graphql_response_has_errors() {
        let mut response = GraphQLResponse {
            data: Some(serde_json::json!({"user": null})),
            errors: None,
            extensions: None,
        };

        assert!(!response.has_errors());
        assert_eq!(response.error_count(), 0);

        response.errors = Some(vec![GraphQLError {
            message: "Field not found".to_string(),
            locations: None,
            path: None,
            extensions: None,
        }]);

        assert!(response.has_errors());
        assert_eq!(response.error_count(), 1);
    }

    #[test]
    fn test_graphql_response_format_errors() {
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
                    message: "Invalid argument".to_string(),
                    locations: None,
                    path: None,
                    extensions: None,
                },
            ]),
            extensions: None,
        };

        let formatted = response.format_errors();

        assert!(formatted.contains("GraphQL Errors:"));
        assert!(formatted.contains("Field 'user' not found"));
        assert!(formatted.contains("Invalid argument"));
        assert!(formatted.contains("line 2, column 5"));
        assert!(formatted.contains("path: \"user\""));
    }

    #[test]
    fn test_graphql_request_operation_name() {
        let mut request = GraphQLRequest::new("query GetUser { user { id } }".to_string());

        assert_eq!(request.operation_name, None);

        request.set_operation_name("GetUser".to_string());
        assert_eq!(request.operation_name, Some("GetUser".to_string()));
    }
}
