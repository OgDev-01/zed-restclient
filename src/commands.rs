//! Command handlers for REST Client extension.
//!
//! This module provides the command logic for sending HTTP requests from within
//! the Zed editor, including request extraction, execution, and response formatting.

use crate::executor::{execute_request, ExecutionConfig};
use crate::formatter::format_response;
use crate::models::request::HttpRequest;
use crate::parser::parse_request;
use std::path::PathBuf;

/// Error types for command execution.
#[derive(Debug)]
pub enum CommandError {
    /// No request found at cursor position.
    NoRequestFound,

    /// Failed to parse the request.
    ParseError(String),

    /// Failed to execute the request.
    ExecutionError(String),

    /// Invalid cursor position.
    InvalidCursorPosition,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NoRequestFound => {
                write!(f, "No HTTP request found at cursor position")
            }
            CommandError::ParseError(msg) => write!(f, "Failed to parse request: {}", msg),
            CommandError::ExecutionError(msg) => write!(f, "Failed to execute request: {}", msg),
            CommandError::InvalidCursorPosition => write!(f, "Invalid cursor position"),
        }
    }
}

impl std::error::Error for CommandError {}

/// Result of a send request command.
#[derive(Debug)]
pub struct CommandResult {
    /// The formatted response ready for display.
    pub formatted_response: String,

    /// The original request that was executed.
    pub request: HttpRequest,

    /// Success status.
    pub success: bool,

    /// Status message for notifications.
    pub status_message: String,
}

/// Extracts the request block at the given cursor position.
///
/// Searches backward and forward from the cursor to find the complete request
/// block bounded by `###` delimiters or file boundaries.
///
/// # Arguments
///
/// * `editor_text` - Complete text content of the editor
/// * `cursor_position` - Byte offset of the cursor in the text
///
/// # Returns
///
/// `Ok((request_text, start_line))` with the extracted request and its starting line number,
/// or `Err(CommandError)` if no valid request block is found.
pub fn extract_request_at_cursor(
    editor_text: &str,
    cursor_position: usize,
) -> Result<(String, usize), CommandError> {
    if cursor_position > editor_text.len() {
        return Err(CommandError::InvalidCursorPosition);
    }

    // Find the start and end of the current request block
    let (block_start, block_end) = find_request_boundaries(editor_text, cursor_position)?;

    // Extract the request block text
    let request_text = editor_text[block_start..block_end].to_string();

    // Calculate the line number for the start of the block
    let start_line = editor_text[..block_start].lines().count() + 1;

    Ok((request_text, start_line))
}

/// Finds the boundaries of a request block around the cursor position.
///
/// # Arguments
///
/// * `text` - Complete editor text
/// * `cursor_pos` - Cursor position in bytes
///
/// # Returns
///
/// `Ok((start_byte, end_byte))` or `Err(CommandError)` if no valid block found.
fn find_request_boundaries(text: &str, cursor_pos: usize) -> Result<(usize, usize), CommandError> {
    let delimiter = "###";

    // Find all delimiter positions
    let mut delimiter_positions: Vec<usize> =
        text.match_indices(delimiter).map(|(pos, _)| pos).collect();

    // Add file boundaries
    delimiter_positions.insert(0, 0);
    delimiter_positions.push(text.len());

    // Find which block contains the cursor
    for i in 0..delimiter_positions.len() - 1 {
        let block_start = delimiter_positions[i];
        let block_end = delimiter_positions[i + 1];

        // Skip the delimiter itself if we're starting at one
        let actual_start = if block_start > 0 && text[block_start..].starts_with(delimiter) {
            // Skip past the delimiter and any following whitespace
            let after_delimiter = block_start + delimiter.len();
            skip_whitespace(text, after_delimiter)
        } else {
            block_start
        };

        if cursor_pos >= actual_start && cursor_pos < block_end {
            // Found the block containing the cursor
            let block_text = text[actual_start..block_end].trim();

            // Check if this block actually contains a request (not just empty or comments)
            if is_valid_request_block(block_text) {
                return Ok((actual_start, block_end));
            }
        }
    }

    Err(CommandError::NoRequestFound)
}

/// Skips whitespace characters from the given position.
fn skip_whitespace(text: &str, start: usize) -> usize {
    let mut pos = start;
    for ch in text[start..].chars() {
        if ch.is_whitespace() {
            pos += ch.len_utf8();
        } else {
            break;
        }
    }
    pos
}

/// Checks if a block contains a valid request (has at least one non-comment line).
fn is_valid_request_block(block: &str) -> bool {
    for line in block.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.starts_with("//") {
            return true;
        }
    }
    false
}

/// Sends an HTTP request from the editor.
///
/// This is the main command handler that orchestrates the entire request lifecycle:
/// 1. Extracts the request block at the cursor position
/// 2. Parses the request using the parser
/// 3. Executes the request using the executor
/// 4. Formats the response using the formatter
/// 5. Returns the formatted result for display
///
/// # Arguments
///
/// * `editor_text` - Complete text content from the editor
/// * `cursor_position` - Byte offset of the cursor position
/// * `file_path` - Path to the .http/.rest file being edited
///
/// # Returns
///
/// `Ok(CommandResult)` with the formatted response and metadata,
/// or `Err(CommandError)` if any step fails.
///
/// # Examples
///
/// ```no_run
/// use rest_client::commands::send_request_command;
/// use std::path::PathBuf;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let editor_text = "GET https://httpbin.org/get\n";
/// let cursor_pos = 5;
/// let file_path = PathBuf::from("test.http");
///
/// let result = send_request_command(editor_text, cursor_pos, &file_path).await?;
/// println!("{}", result.formatted_response);
/// # Ok(())
/// # }
/// ```
pub async fn send_request_command(
    editor_text: &str,
    cursor_position: usize,
    file_path: &PathBuf,
) -> Result<CommandResult, CommandError> {
    // Step 1: Extract the request block at cursor
    let (request_text, start_line) = extract_request_at_cursor(editor_text, cursor_position)?;

    // Step 2: Parse the request
    let lines: Vec<(usize, &str)> = request_text
        .lines()
        .enumerate()
        .map(|(i, line)| (start_line + i, line))
        .collect();

    let request = parse_request(&lines, start_line, file_path)
        .map_err(|e| CommandError::ParseError(e.to_string()))?;

    // Step 3: Execute the request
    let config = ExecutionConfig::default();
    let response = execute_request(&request, &config)
        .await
        .map_err(|e| CommandError::ExecutionError(e.to_string()))?;

    // Step 4: Format the response
    let formatted = format_response(&response);

    // Step 5: Create the result
    let success = response.is_success();
    let status_message = if success {
        format!(
            "Request completed: {} {} ({})",
            request.method, request.url, response.status_code
        )
    } else {
        format!(
            "Request failed: {} {} ({})",
            request.method, request.url, response.status_code
        )
    };

    Ok(CommandResult {
        formatted_response: formatted.to_display_string(),
        request,
        success,
        status_message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_request_single() {
        let text = "GET https://example.com\n";
        let cursor_pos = 5; // Middle of "GET"

        let result = extract_request_at_cursor(text, cursor_pos);
        assert!(result.is_ok());

        let (request_text, start_line) = result.unwrap();
        assert_eq!(request_text.trim(), "GET https://example.com");
        assert_eq!(start_line, 1);
    }

    #[test]
    fn test_extract_request_with_delimiters() {
        let text = r#"
GET https://example.com/1

###

POST https://example.com/2
Content-Type: application/json

{"key": "value"}

###

DELETE https://example.com/3
"#;

        // Cursor in the POST request
        let cursor_pos = text.find("POST").unwrap();
        let result = extract_request_at_cursor(text, cursor_pos);
        assert!(result.is_ok());

        let (request_text, _) = result.unwrap();
        assert!(request_text.contains("POST"));
        assert!(request_text.contains("Content-Type"));
        assert!(!request_text.contains("GET"));
        assert!(!request_text.contains("DELETE"));
    }

    #[test]
    fn test_extract_request_first_block() {
        let text = r#"GET https://example.com/1

###

POST https://example.com/2"#;

        let cursor_pos = 5; // In GET request
        let result = extract_request_at_cursor(text, cursor_pos);
        assert!(result.is_ok());

        let (request_text, _) = result.unwrap();
        assert!(request_text.contains("GET"));
        assert!(!request_text.contains("POST"));
    }

    #[test]
    fn test_extract_request_last_block() {
        let text = r#"GET https://example.com/1

###

POST https://example.com/2"#;

        let cursor_pos = text.find("POST").unwrap();
        let result = extract_request_at_cursor(text, cursor_pos);
        assert!(result.is_ok());

        let (request_text, _) = result.unwrap();
        assert!(request_text.contains("POST"));
        assert!(!request_text.contains("GET"));
    }

    #[test]
    fn test_extract_request_with_comments() {
        let text = r#"
# This is a comment
GET https://example.com
"#;

        let cursor_pos = text.find("GET").unwrap();
        let result = extract_request_at_cursor(text, cursor_pos);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_request_invalid_cursor() {
        let text = "GET https://example.com\n";
        let cursor_pos = 1000; // Beyond text length

        let result = extract_request_at_cursor(text, cursor_pos);
        assert!(matches!(result, Err(CommandError::InvalidCursorPosition)));
    }

    #[test]
    fn test_extract_request_empty_block() {
        let text = r#"
###

# Just comments

###

GET https://example.com
"#;

        // Cursor in the empty/comment-only block
        let cursor_pos = text.find("# Just").unwrap();
        let result = extract_request_at_cursor(text, cursor_pos);
        assert!(matches!(result, Err(CommandError::NoRequestFound)));
    }

    #[test]
    fn test_is_valid_request_block() {
        assert!(is_valid_request_block("GET https://example.com"));
        assert!(is_valid_request_block("# Comment\nGET https://example.com"));
        assert!(!is_valid_request_block("# Only comments\n# More comments"));
        assert!(!is_valid_request_block("   \n\n   "));
        assert!(!is_valid_request_block(""));
    }

    #[test]
    fn test_find_request_boundaries_simple() {
        let text = "GET https://example.com\n";
        let cursor_pos = 5;

        let result = find_request_boundaries(text, cursor_pos);
        assert!(result.is_ok());

        let (start, end) = result.unwrap();
        assert_eq!(start, 0);
        assert_eq!(end, text.len());
    }

    #[test]
    fn test_find_request_boundaries_with_delimiter() {
        let text = "GET https://example.com/1\n###\nPOST https://example.com/2\n";
        let delimiter_pos = text.find("###").unwrap();

        // Cursor after delimiter, in POST request
        let cursor_pos = delimiter_pos + 10;
        let result = find_request_boundaries(text, cursor_pos);
        assert!(result.is_ok());

        let (start, end) = result.unwrap();
        let block = &text[start..end];
        assert!(block.contains("POST"));
        assert!(!block.contains("GET"));
    }

    #[tokio::test]
    async fn test_send_request_command_simple() {
        let text = "GET https://httpbin.org/get\n";
        let cursor_pos = 5;
        let file_path = PathBuf::from("test.http");

        let result = send_request_command(text, cursor_pos, &file_path).await;
        assert!(result.is_ok());

        let cmd_result = result.unwrap();
        assert!(cmd_result.success);
        assert!(cmd_result.formatted_response.contains("200"));
        assert!(cmd_result.status_message.contains("completed"));
    }

    #[tokio::test]
    async fn test_send_request_command_with_headers() {
        let text = r#"POST https://httpbin.org/post
Content-Type: application/json

{"test": "data"}
"#;
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        let result = send_request_command(text, cursor_pos, &file_path).await;
        assert!(result.is_ok());

        let cmd_result = result.unwrap();
        assert!(cmd_result.success);
        assert!(cmd_result.formatted_response.contains("200"));
    }

    #[tokio::test]
    async fn test_send_request_command_404() {
        let text = "GET https://httpbin.org/status/404\n";
        let cursor_pos = 5;
        let file_path = PathBuf::from("test.http");

        let result = send_request_command(text, cursor_pos, &file_path).await;
        assert!(result.is_ok());

        let cmd_result = result.unwrap();
        assert!(!cmd_result.success);
        assert!(cmd_result.formatted_response.contains("404"));
        assert!(cmd_result.status_message.contains("failed"));
    }

    #[test]
    fn test_command_error_display() {
        let err = CommandError::NoRequestFound;
        assert_eq!(
            format!("{}", err),
            "No HTTP request found at cursor position"
        );

        let err = CommandError::ParseError("invalid syntax".to_string());
        assert_eq!(
            format!("{}", err),
            "Failed to parse request: invalid syntax"
        );

        let err = CommandError::ExecutionError("timeout".to_string());
        assert_eq!(format!("{}", err), "Failed to execute request: timeout");

        let err = CommandError::InvalidCursorPosition;
        assert_eq!(format!("{}", err), "Invalid cursor position");
    }
}
