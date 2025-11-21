//! Command handlers for REST Client extension.
//!
//! This module provides the command logic for sending HTTP requests from within
//! the Zed editor, including request extraction, execution, and response formatting.
//! Also includes environment switching functionality for managing variable contexts.

use crate::codegen::ui::{generate_code_command, parse_generation_options, CodeGenerationResult};
use crate::codegen::Language;
use crate::curl::ui::{copy_as_curl_command, paste_curl_command, CopyCurlResult, PasteCurlResult};
use crate::environment::{load_environments, EnvironmentSession};
use crate::executor::{
    cancel_most_recent_request, execute_request, get_active_request_count, get_active_request_ids,
    ExecutionConfig,
};
use crate::formatter::{format_response, FormattedResponse};
use crate::history::{
    clear_history, format_history_entry, get_recent_entries, load_history, search_history,
    sort_by_timestamp_desc, HistoryEntry,
};
use crate::models::request::HttpRequest;
use crate::parser::parse_request;
use crate::ui::response_actions::{
    copy_response, fold_response, save_response, toggle_raw_view, CopyOption, CopyResponseResult,
    FoldResponseResult, SaveOption, SaveResponseResult,
};
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

/// Result of a cancel request command.
#[derive(Debug)]
pub struct CancelRequestResult {
    /// Success status of the cancellation operation.
    pub success: bool,

    /// Message describing the result (for notifications).
    pub message: String,

    /// ID of the request that was cancelled (if successful).
    pub cancelled_request_id: Option<String>,

    /// Number of requests still active after cancellation.
    pub remaining_active_count: usize,
}

/// Result of an environment switch command.
#[derive(Debug)]
pub struct EnvironmentSwitchResult {
    /// Success status of the switch operation.
    pub success: bool,

    /// Message describing the result (for notifications).
    pub message: String,

    /// Name of the environment that is now active (if successful).
    pub active_environment: Option<String>,

    /// List of all available environments.
    pub available_environments: Vec<String>,
}

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

/// Result of a view history command.
#[derive(Debug)]
pub struct HistoryViewResult {
    /// Success status of the operation.
    pub success: bool,

    /// Message describing the result (for notifications).
    pub message: String,

    /// Formatted history entries for display.
    pub formatted_entries: Vec<String>,

    /// The actual history entries (for further processing).
    pub entries: Vec<HistoryEntry>,

    /// Total count of entries in history.
    pub total_count: usize,
}

/// Result of a rerun request command.
#[derive(Debug)]
pub struct RerunHistoryResult {
    /// Success status of the operation.
    pub success: bool,

    /// Message describing the result.
    pub message: String,

    /// The command result from executing the request.
    pub command_result: Option<CommandResult>,

    /// The history entry that was re-executed.
    pub entry: HistoryEntry,
}

/// Result of a clear history command.
#[derive(Debug)]
pub struct ClearHistoryResult {
    /// Success status of the operation.
    pub success: bool,

    /// Message describing the result.
    pub message: String,

    /// Number of entries that were cleared.
    pub cleared_count: usize,
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

/// Switches the active environment for variable resolution.
///
/// This command lists available environments and switches to a specified environment.
/// If no environment name is provided, it returns a list of available environments
/// with the currently active one highlighted.
///
/// # Arguments
///
/// * `workspace_path` - Path to the workspace for locating environment files
/// * `environment_name` - Optional name of the environment to switch to
/// * `current_session` - Optional current environment session (will be loaded if None)
///
/// # Returns
///
/// `Ok(EnvironmentSwitchResult)` with the result of the operation,
/// or `Err(String)` if the operation fails.
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use rest_client::commands::switch_environment_command;
///
/// // List available environments
/// let result = switch_environment_command(
///     &PathBuf::from("/workspace"),
///     None,
///     None
/// ).unwrap();
/// println!("Available: {:?}", result.available_environments);
///
/// // Switch to production environment
/// let result = switch_environment_command(
///     &PathBuf::from("/workspace"),
///     Some("production".to_string()),
///     None
/// ).unwrap();
/// assert!(result.success);
/// ```
pub fn switch_environment_command(
    workspace_path: &PathBuf,
    environment_name: Option<String>,
    current_session: Option<EnvironmentSession>,
) -> Result<EnvironmentSwitchResult, String> {
    // Load or use existing session
    let session = match current_session {
        Some(s) => s,
        None => {
            // Load environments from workspace
            let environments = load_environments(workspace_path).map_err(|e| {
                format!(
                    "No environment configuration found.\n\n\
                    To use environments, create a `.http-client-env.json` file in your workspace:\n\n\
                    {{\n  \"$shared\": {{\n    \"apiVersion\": \"v1\"\n  }},\n  \
                    \"dev\": {{\n    \"baseUrl\": \"http://localhost:3000\"\n  }},\n  \
                    \"production\": {{\n    \"baseUrl\": \"https://api.example.com\"\n  }},\n  \
                    \"active\": \"dev\"\n}}\n\n\
                    See examples/environment-variables.http for more details.\n\nError: {}",
                    e
                )
            })?;

            EnvironmentSession::new(environments)
        }
    };

    let available = session.list_environment_names();

    // If no environment name provided, just return list
    if environment_name.is_none() {
        let active = session.get_active_environment_name();
        let message = if available.is_empty() {
            "No environments defined in configuration file.".to_string()
        } else {
            let mut msg = String::from("Available environments:\n");
            for name in &available {
                let is_active = active.as_ref().map_or(false, |a| a == name);
                let marker = if is_active { "→ " } else { "  " };
                let indicator = if is_active { " (active)" } else { "" };
                msg.push_str(&format!("{}{}{}\n", marker, name, indicator));
            }
            if let Some(active_name) = &active {
                msg.push_str(&format!("\nCurrently active: {}", active_name));
            }
            msg
        };

        return Ok(EnvironmentSwitchResult {
            success: true,
            message,
            active_environment: session.get_active_environment_name(),
            available_environments: available,
        });
    }

    // Switch to the specified environment
    let env_name = environment_name.unwrap();

    match session.set_active_environment(&env_name) {
        Ok(_) => Ok(EnvironmentSwitchResult {
            success: true,
            message: format!(
                "Switched to '{}' environment. Variables from this environment are now active.",
                env_name
            ),
            active_environment: Some(env_name),
            available_environments: available,
        }),
        Err(e) => Err(format!(
            "Failed to switch to environment '{}': {}\n\nAvailable environments: {}",
            env_name,
            e,
            available.join(", ")
        )),
    }
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

/// Views request history with optional search filtering.
///
/// Loads history entries from storage, optionally filters them by search query,
/// and formats them for display in a quick-pick list or dedicated pane.
///
/// # Arguments
///
/// * `query` - Optional search query to filter entries
/// * `limit` - Maximum number of entries to return (default: 100)
///
/// # Returns
///
/// `Ok(HistoryViewResult)` with formatted history entries,
/// or `Err(String)` if loading history fails.
///
/// # Example
///
/// ```no_run
/// use rest_client::commands::view_history_command;
///
/// // View all recent history
/// let result = view_history_command(None, 100).unwrap();
/// for entry in result.formatted_entries {
///     println!("{}", entry);
/// }
///
/// // Search history
/// let result = view_history_command(Some("api/users".to_string()), 50).unwrap();
/// ```
pub fn view_history_command(
    query: Option<String>,
    limit: usize,
) -> Result<HistoryViewResult, String> {
    // Load history from storage
    let all_entries = load_history().map_err(|e| format!("Failed to load history: {}", e))?;

    // Sort by timestamp descending (newest first)
    let sorted_entries = sort_by_timestamp_desc(&all_entries);

    // Apply search filter if provided
    let filtered_entries = if let Some(q) = query.as_ref() {
        search_history(q, &sorted_entries)
    } else {
        sorted_entries
    };

    // Get the most recent entries up to the limit
    let limited_entries = get_recent_entries(limit, &filtered_entries);

    // Format entries for display
    let formatted_entries: Vec<String> = limited_entries
        .iter()
        .map(|entry| format_history_entry(entry))
        .collect();

    let total_count = all_entries.len();
    let shown_count = limited_entries.len();

    let message = if let Some(q) = query {
        format!(
            "Found {} matching entries (showing {} of {} total)",
            shown_count, shown_count, total_count
        )
    } else {
        format!("Showing {} of {} total entries", shown_count, total_count)
    };

    Ok(HistoryViewResult {
        success: true,
        message,
        formatted_entries,
        entries: limited_entries,
        total_count,
    })
}

/// Re-executes a request from history using current environment variables.
///
/// Takes a history entry and executes it again, using the current environment
/// for variable resolution (not the environment from the original execution).
///
/// # Arguments
///
/// * `entry` - The history entry to re-execute
/// * `current_session` - Optional current environment session for variable resolution
///
/// # Returns
///
/// `Ok(RerunHistoryResult)` with the execution result,
/// or `Err(String)` if execution fails.
///
/// # Example
///
/// ```no_run
/// use rest_client::commands::{view_history_command, rerun_from_history_command};
///
/// // Get history and rerun the first entry
/// let history = view_history_command(None, 10).unwrap();
/// if let Some(entry) = history.entries.first() {
///     let result = rerun_from_history_command(entry.clone(), None).unwrap();
///     println!("{}", result.message);
/// }
/// ```
pub fn rerun_from_history_command(
    entry: HistoryEntry,
    _current_session: Option<EnvironmentSession>,
) -> Result<RerunHistoryResult, String> {
    // Create execution config
    // Note: Current environment variables are not yet supported in the executor
    let config = ExecutionConfig::default();

    // Execute the request
    let response = execute_request(&entry.request, &config)
        .map_err(|e| format!("Failed to re-execute request: {}", e))?;

    // Format the response
    let formatted_response = format_response(&response);

    let command_result = CommandResult {
        formatted_response: formatted_response.to_display_string(),
        request: entry.request.clone(),
        success: response.is_success(),
        status_message: format!(
            "Re-executed request: {} {} - {}",
            entry.request.method.as_str(),
            entry.request.url,
            response.status_code
        ),
    };

    Ok(RerunHistoryResult {
        success: true,
        message: format!(
            "Successfully re-executed: {} {}",
            entry.request.method.as_str(),
            entry.request.url
        ),
        command_result: Some(command_result),
        entry,
    })
}

/// Clears all history entries after confirmation.
///
/// Deletes the entire history file, removing all stored request/response pairs.
/// This operation cannot be undone.
///
/// # Arguments
///
/// * `confirmed` - Whether the user has confirmed the deletion
///
/// # Returns
///
/// `Ok(ClearHistoryResult)` with the number of cleared entries,
/// or `Err(String)` if the operation fails.
///
/// # Example
///
/// ```no_run
/// use rest_client::commands::clear_history_command;
///
/// // Clear history with confirmation
/// let result = clear_history_command(true).unwrap();
/// println!("{}", result.message);
/// ```
pub fn clear_history_command(confirmed: bool) -> Result<ClearHistoryResult, String> {
    if !confirmed {
        return Ok(ClearHistoryResult {
            success: false,
            message: "History clear cancelled - confirmation required".to_string(),
            cleared_count: 0,
        });
    }

    // Get count before clearing
    let entries = load_history().map_err(|e| format!("Failed to load history: {}", e))?;
    let count = entries.len();

    // Clear the history
    clear_history().map_err(|e| format!("Failed to clear history: {}", e))?;

    Ok(ClearHistoryResult {
        success: true,
        message: format!("Successfully cleared {} history entries", count),
        cleared_count: count,
    })
}

/// Cancels the most recently started active request.
///
/// This command provides a user-friendly way to abort in-flight HTTP requests.
/// It cancels the most recent active request and provides status information.
///
/// # Returns
///
/// `Ok(CancelRequestResult)` with the result of the cancellation operation.
///
/// # Examples
///
/// ```no_run
/// use rest_client::commands::cancel_request_command;
///
/// // Cancel the most recent request
/// let result = cancel_request_command();
/// if result.success {
///     println!("Cancelled: {}", result.message);
/// }
/// ```
pub fn cancel_request_command() -> CancelRequestResult {
    // Get the current active request count before attempting cancellation
    let initial_count = get_active_request_count();

    if initial_count == 0 {
        return CancelRequestResult {
            success: false,
            message: "No active requests to cancel".to_string(),
            cancelled_request_id: None,
            remaining_active_count: 0,
        };
    }

    // Attempt to cancel the most recent request
    match cancel_most_recent_request() {
        Ok(request_id) => {
            let remaining_count = get_active_request_count();
            let message = if remaining_count > 0 {
                format!(
                    "Request cancelled successfully (ID: {})\n{} request(s) still active",
                    request_id, remaining_count
                )
            } else {
                format!("Request cancelled successfully (ID: {})", request_id)
            };

            CancelRequestResult {
                success: true,
                message,
                cancelled_request_id: Some(request_id),
                remaining_active_count: remaining_count,
            }
        }
        Err(e) => CancelRequestResult {
            success: false,
            message: format!("Failed to cancel request: {}", e),
            cancelled_request_id: None,
            remaining_active_count: get_active_request_count(),
        },
    }
}

/// Gets information about currently active requests.
///
/// This command provides status information about in-flight requests,
/// useful for debugging or displaying request activity to users.
///
/// # Returns
///
/// A tuple of `(count, request_ids)` with the number of active requests
/// and their IDs.
///
/// # Examples
///
/// ```no_run
/// use rest_client::commands::get_active_requests_info;
///
/// let (count, ids) = get_active_requests_info();
/// println!("Active requests: {}", count);
/// for id in ids {
///     println!("  - {}", id);
/// }
/// ```
pub fn get_active_requests_info() -> (usize, Vec<String>) {
    let count = get_active_request_count();
    let ids = get_active_request_ids();
    (count, ids)
}

/// Generates code from the HTTP request at the cursor position.
///
/// This command extracts the request block at the cursor, parses it,
/// and generates executable code in the specified language and library.
///
/// # Arguments
///
/// * `editor_text` - Complete text content of the editor
/// * `cursor_position` - Byte offset of the cursor in the text
/// * `file_path` - Path to the .http file being edited
/// * `language_str` - Target language (e.g., "javascript", "python")
/// * `library_str` - Optional library name (e.g., "fetch", "axios", "requests")
///
/// # Returns
///
/// `Ok(CodeGenerationResult)` with the generated code, or `Err(CommandError)` if extraction or generation fails.
///
/// # Examples
///
/// ```no_run
/// use rest_client::commands::generate_code_from_cursor;
/// use std::path::PathBuf;
///
/// let editor_text = "GET https://api.example.com/users\nAuthorization: Bearer token123";
/// let cursor_pos = 10;
/// let file_path = PathBuf::from("test.http");
///
/// let result = generate_code_from_cursor(
///     editor_text,
///     cursor_pos,
///     &file_path,
///     "javascript",
///     Some("fetch")
/// ).unwrap();
///
/// println!("Generated code:\n{}", result.to_display_string());
/// ```
pub fn generate_code_from_cursor(
    editor_text: &str,
    cursor_position: usize,
    file_path: &PathBuf,
    language_str: &str,
    library_str: Option<&str>,
) -> Result<CodeGenerationResult, CommandError> {
    // Extract the request at cursor position
    let (request_text, start_line) = extract_request_at_cursor(editor_text, cursor_position)?;

    // Parse the request
    let lines: Vec<String> = request_text.lines().map(|s| s.to_string()).collect();
    let indexed_lines: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .map(|(i, s)| (i + start_line, s.as_str()))
        .collect();

    let request = parse_request(&indexed_lines, 0, file_path)
        .map_err(|e| CommandError::ParseError(e.to_string()))?;

    // Build args for parsing generation options
    let mut args = vec![language_str.to_string()];
    if let Some(lib) = library_str {
        args.push(lib.to_string());
    }

    // Parse generation options
    let (language, library) =
        parse_generation_options(&args).map_err(|e| CommandError::ExecutionError(e))?;

    // Generate the code
    let result = generate_code_command(&request, language, library);

    Ok(result)
}

/// Generates code from a parsed HTTP request with the specified language.
///
/// This is a convenience function when you already have a parsed request.
///
/// # Arguments
///
/// * `request` - The parsed HTTP request
/// * `language` - Target language for code generation
/// * `library` - Optional library (uses default if None)
///
/// # Returns
///
/// `CodeGenerationResult` with the generated code.
///
/// # Examples
///
/// ```ignore
/// use rest_client::commands::generate_code_from_request;
/// use rest_client::models::request::{HttpRequest, HttpMethod};
/// use rest_client::codegen::Language;
/// use std::path::PathBuf;
///
/// let request = HttpRequest {
///     method: HttpMethod::Get,
///     url: "https://api.example.com/users".to_string(),
///     headers: Default::default(),
///     body: None,
///     file_path: PathBuf::from("test.http"),
///     line_number: 1,
/// };
///
/// let result = generate_code_from_request(&request, Language::JavaScript, None);
/// assert!(result.success);
/// ```
pub fn generate_code_from_request(
    request: &HttpRequest,
    language: Language,
    library: Option<crate::codegen::Library>,
) -> CodeGenerationResult {
    generate_code_command(request, language, library)
}

/// Pastes and converts a cURL command to an HTTP request format.
///
/// This function takes clipboard content (expected to be a cURL command),
/// validates it, parses it, and formats it as a clean HTTP request ready
/// to be inserted into a .http file.
///
/// # Arguments
///
/// * `clipboard_content` - The text from clipboard (should be a cURL command)
///
/// # Returns
///
/// `PasteCurlResult` containing the formatted HTTP request or error message.
///
/// # Examples
///
/// ```no_run
/// use rest_client::commands::paste_curl_from_clipboard;
///
/// let curl = "curl -X POST https://api.example.com/users -H 'Content-Type: application/json'";
/// let result = paste_curl_from_clipboard(curl);
///
/// if result.success {
///     println!("Formatted request:\n{}", result.formatted_request);
/// } else {
///     eprintln!("Error: {}", result.message);
/// }
/// ```
pub fn paste_curl_from_clipboard(clipboard_content: &str) -> PasteCurlResult {
    paste_curl_command(clipboard_content)
}

/// Converts an HTTP request at cursor position to a cURL command.
///
/// This function extracts the HTTP request at the given cursor position,
/// parses it, and generates a valid cURL command that can be copied to clipboard.
///
/// # Arguments
///
/// * `editor_text` - The full text content of the editor
/// * `cursor_position` - The byte offset of the cursor position
/// * `file_path` - Path to the .http file (for error reporting)
///
/// # Returns
///
/// `Ok(CopyCurlResult)` with the cURL command, or `Err(CommandError)` if extraction fails.
///
/// # Examples
///
/// ```no_run
/// use rest_client::commands::copy_as_curl_from_cursor;
/// use std::path::PathBuf;
///
/// let editor_text = "GET https://api.example.com/users\nAuthorization: Bearer token123";
/// let file_path = PathBuf::from("test.http");
///
/// match copy_as_curl_from_cursor(editor_text, 0, &file_path) {
///     Ok(result) => {
///         println!("cURL command: {}", result.curl_command);
///         println!("Preview: {}", result.preview);
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn copy_as_curl_from_cursor(
    editor_text: &str,
    cursor_position: usize,
    file_path: &PathBuf,
) -> Result<CopyCurlResult, CommandError> {
    // Extract the request at cursor position
    let (request_text, start_line) = extract_request_at_cursor(editor_text, cursor_position)?;

    // Parse the request
    let lines: Vec<String> = request_text.lines().map(|s| s.to_string()).collect();
    let indexed_lines: Vec<(usize, &str)> = lines
        .iter()
        .enumerate()
        .map(|(i, s)| (i + start_line, s.as_str()))
        .collect();

    let request = parse_request(&indexed_lines, 0, file_path)
        .map_err(|e| CommandError::ParseError(e.to_string()))?;

    // Generate cURL command
    let result = copy_as_curl_command(&request);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::HttpMethod;

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
    #[ignore] // Requires network access
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
    #[ignore] // Requires network access
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
    #[ignore] // Requires network access
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

    #[test]
    fn test_switch_environment_list_environments() {
        use crate::environment::{Environment, Environments};

        // Create test environments
        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        envs.add_environment(Environment::new("staging"));
        envs.add_environment(Environment::new("prod"));
        envs.set_active("dev");

        let session = EnvironmentSession::new(envs);
        let workspace = PathBuf::from("/test");

        // List environments (no name provided)
        let result = switch_environment_command(&workspace, None, Some(session));
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.available_environments.len(), 3);
        assert!(result.available_environments.contains(&"dev".to_string()));
        assert!(result
            .available_environments
            .contains(&"staging".to_string()));
        assert!(result.available_environments.contains(&"prod".to_string()));
        assert_eq!(result.active_environment, Some("dev".to_string()));
        assert!(result.message.contains("dev"));
        assert!(result.message.contains("active"));
    }

    #[test]
    fn test_switch_environment_successful_switch() {
        use crate::environment::{Environment, Environments};

        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        envs.add_environment(Environment::new("production"));
        envs.set_active("dev");

        let session = EnvironmentSession::new(envs);
        let workspace = PathBuf::from("/test");

        // Switch to production
        let result = switch_environment_command(
            &workspace,
            Some("production".to_string()),
            Some(session.clone()),
        );
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.active_environment, Some("production".to_string()));
        assert!(result.message.contains("production"));
        assert!(result.message.contains("Switched"));
    }

    #[test]
    fn test_switch_environment_invalid_environment() {
        use crate::environment::{Environment, Environments};

        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));

        let session = EnvironmentSession::new(envs);
        let workspace = PathBuf::from("/test");

        // Try to switch to non-existent environment
        let result =
            switch_environment_command(&workspace, Some("nonexistent".to_string()), Some(session));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.contains("nonexistent"));
        assert!(error.contains("Available environments"));
    }

    #[test]
    fn test_switch_environment_no_environments() {
        use crate::environment::Environments;

        let envs = Environments::new(); // Empty
        let session = EnvironmentSession::new(envs);
        let workspace = PathBuf::from("/test");

        // List when no environments exist
        let result = switch_environment_command(&workspace, None, Some(session));
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert!(result.available_environments.is_empty());
        assert!(result.message.contains("No environments"));
    }

    #[test]
    fn test_switch_environment_multiple_switches() {
        use crate::environment::{Environment, Environments};

        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        envs.add_environment(Environment::new("staging"));
        envs.add_environment(Environment::new("prod"));
        envs.set_active("dev");

        let session = EnvironmentSession::new(envs);
        let workspace = PathBuf::from("/test");

        // Switch to staging
        let result = switch_environment_command(
            &workspace,
            Some("staging".to_string()),
            Some(session.clone()),
        );
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().active_environment,
            Some("staging".to_string())
        );

        // Switch to prod
        let result =
            switch_environment_command(&workspace, Some("prod".to_string()), Some(session.clone()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().active_environment, Some("prod".to_string()));

        // Switch back to dev
        let result = switch_environment_command(&workspace, Some("dev".to_string()), Some(session));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().active_environment, Some("dev".to_string()));
    }

    #[test]
    fn test_environment_switch_result_structure() {
        let result = EnvironmentSwitchResult {
            success: true,
            message: "Test message".to_string(),
            active_environment: Some("dev".to_string()),
            available_environments: vec!["dev".to_string(), "prod".to_string()],
        };

        assert!(result.success);
        assert_eq!(result.message, "Test message");
        assert_eq!(result.active_environment, Some("dev".to_string()));
        assert_eq!(result.available_environments.len(), 2);
    }

    #[test]
    #[serial_test::serial]
    fn test_view_history_command_empty() {
        use crate::history::storage::clear_history;

        // Clear history first
        let _ = clear_history();

        let result = view_history_command(None, 100);
        assert!(result.is_ok());

        let view_result = result.unwrap();
        assert!(view_result.success);
        assert_eq!(view_result.entries.len(), 0);
        assert_eq!(view_result.formatted_entries.len(), 0);
        assert_eq!(view_result.total_count, 0);
    }

    #[test]
    #[serial_test::serial]
    fn test_view_history_command_with_entries() {
        use crate::history::storage::{clear_history, load_history, save_entry};
        use crate::models::{HttpMethod, HttpRequest, HttpResponse};

        // Clear history first to ensure clean state
        let _ = clear_history();

        // Create and save test entries
        for i in 1..=5 {
            let request = HttpRequest::new(
                format!("test-{}", i),
                HttpMethod::GET,
                format!("https://api.example.com/test/{}", i),
            );
            let response = HttpResponse::new(200, "OK".to_string());
            let entry = HistoryEntry::new(request, response);
            let _ = save_entry(&entry);
        }

        // Verify entries were saved
        let loaded = load_history().unwrap();
        assert_eq!(
            loaded.len(),
            5,
            "Expected 5 entries in history after saving"
        );

        let result = view_history_command(None, 100);
        assert!(result.is_ok());

        let view_result = result.unwrap();
        assert!(view_result.success);
        assert_eq!(view_result.entries.len(), 5);
        assert_eq!(view_result.formatted_entries.len(), 5);
        assert_eq!(view_result.total_count, 5);

        // Clean up
        let _ = clear_history();
    }

    #[test]
    #[serial_test::serial]
    fn test_view_history_command_with_limit() {
        use crate::history::storage::{clear_history, load_history, save_entry};
        use crate::models::{HttpMethod, HttpRequest, HttpResponse};

        // Clear history first to ensure clean state
        let _ = clear_history();

        // Create and save 10 test entries
        for i in 1..=10 {
            let request = HttpRequest::new(
                format!("test-limit-{}", i),
                HttpMethod::GET,
                format!("https://api.example.com/limit-test/{}", i),
            );
            let response = HttpResponse::new(200, "OK".to_string());
            let entry = HistoryEntry::new(request, response);
            let _ = save_entry(&entry);
        }

        // Verify 10 entries were saved
        let loaded = load_history().unwrap();
        assert_eq!(
            loaded.len(),
            10,
            "Expected 10 entries in history after saving"
        );

        // Request only 5 entries
        let result = view_history_command(None, 5);
        assert!(result.is_ok());

        let view_result = result.unwrap();
        assert!(view_result.success);
        assert_eq!(
            view_result.entries.len(),
            5,
            "Should return only 5 entries when limit is 5"
        );
        assert_eq!(view_result.total_count, 10, "Total count should be 10");

        // Clean up
        let _ = clear_history();
    }

    #[test]
    #[serial_test::serial]
    fn test_view_history_command_with_search() {
        use crate::history::storage::{clear_history, save_entry};
        use crate::models::{HttpMethod, HttpRequest, HttpResponse};

        // Clear history first
        let _ = clear_history();

        // Create test entries with different URLs
        let request1 = HttpRequest::new(
            "test-1".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );
        let response1 = HttpResponse::new(200, "OK".to_string());
        let _ = save_entry(&HistoryEntry::new(request1, response1));

        let request2 = HttpRequest::new(
            "test-2".to_string(),
            HttpMethod::POST,
            "https://api.example.com/posts".to_string(),
        );
        let response2 = HttpResponse::new(201, "Created".to_string());
        let _ = save_entry(&HistoryEntry::new(request2, response2));

        // Search for "users"
        let result = view_history_command(Some("users".to_string()), 100);
        assert!(result.is_ok());

        let view_result = result.unwrap();
        assert!(view_result.success);
        assert_eq!(view_result.entries.len(), 1);
        assert!(view_result.formatted_entries[0].contains("users"));

        // Clean up
        let _ = clear_history();
    }

    // Note: This test is commented out because it requires network access
    // and makes actual HTTP requests. In a real test environment, this would
    // need to be mocked or run as an integration test.
    #[test]
    #[ignore]
    fn test_rerun_from_history_command() {
        use crate::models::{HttpMethod, HttpRequest, HttpResponse};

        let request = HttpRequest::new(
            "test-rerun".to_string(),
            HttpMethod::GET,
            "https://httpbin.org/get".to_string(),
        );
        let response = HttpResponse::new(200, "OK".to_string());
        let entry = HistoryEntry::new(request, response);

        let result = rerun_from_history_command(entry.clone(), None);
        assert!(result.is_ok());

        let rerun_result = result.unwrap();
        assert!(rerun_result.success);
        assert!(rerun_result.command_result.is_some());
        assert_eq!(rerun_result.entry.id, entry.id);
    }

    #[test]
    #[serial_test::serial]
    fn test_clear_history_command_without_confirmation() {
        let result = clear_history_command(false);
        assert!(result.is_ok());

        let clear_result = result.unwrap();
        assert!(!clear_result.success);
        assert!(clear_result.message.contains("cancelled"));
        assert_eq!(clear_result.cleared_count, 0);
    }

    #[test]
    #[serial_test::serial]
    fn test_clear_history_command_with_confirmation() {
        use crate::history::storage::{clear_history, load_history, save_entry};
        use crate::models::{HttpMethod, HttpRequest, HttpResponse};

        // Clear history first to ensure clean state
        let _ = clear_history();

        // Add some entries
        for i in 1..=3 {
            let request = HttpRequest::new(
                format!("test-clear-{}", i),
                HttpMethod::GET,
                format!("https://api.example.com/clear-test/{}", i),
            );
            let response = HttpResponse::new(200, "OK".to_string());
            let entry = HistoryEntry::new(request, response);
            let _ = save_entry(&entry);
        }

        // Verify entries were saved
        let loaded = load_history().unwrap();
        assert_eq!(loaded.len(), 3, "Expected 3 entries before clearing");

        // Clear with confirmation
        let result = clear_history_command(true);
        assert!(result.is_ok());

        let clear_result = result.unwrap();
        assert!(clear_result.success);
        assert_eq!(clear_result.cleared_count, 3);
        assert!(clear_result.message.contains("Successfully cleared"));

        // Verify history is empty
        let view_result = view_history_command(None, 100).unwrap();
        assert_eq!(view_result.total_count, 0);
    }

    #[test]
    fn test_history_view_result_structure() {
        let result = HistoryViewResult {
            success: true,
            message: "Test message".to_string(),
            formatted_entries: vec!["GET /api/test - 200".to_string()],
            entries: vec![],
            total_count: 1,
        };

        assert!(result.success);
        assert_eq!(result.message, "Test message");
        assert_eq!(result.formatted_entries.len(), 1);
        assert_eq!(result.total_count, 1);
    }

    #[test]
    fn test_clear_history_result_structure() {
        let result = ClearHistoryResult {
            success: true,
            message: "Cleared 5 entries".to_string(),
            cleared_count: 5,
        };

        assert!(result.success);
        assert_eq!(result.message, "Cleared 5 entries");
        assert_eq!(result.cleared_count, 5);
    }

    #[test]
    fn test_generate_code_from_cursor_javascript() {
        let editor_text = "GET https://api.example.com/users\nAuthorization: Bearer token123";
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        let result = generate_code_from_cursor(
            editor_text,
            cursor_pos,
            &file_path,
            "javascript",
            Some("fetch"),
        );

        assert!(result.is_ok());
        let code_result = result.unwrap();
        assert!(code_result.success);
        assert!(code_result.generated_code.is_some());
        let code = code_result.generated_code.unwrap();
        assert!(code.contains("fetch"));
        assert!(code.contains("https://api.example.com/users"));
        assert!(code.contains("Bearer token123"));
    }

    #[test]
    fn test_generate_code_from_cursor_python() {
        let editor_text = "POST https://api.example.com/data\nContent-Type: application/json\n\n{\"key\": \"value\"}";
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        let result = generate_code_from_cursor(
            editor_text,
            cursor_pos,
            &file_path,
            "python",
            Some("requests"),
        );

        assert!(result.is_ok());
        let code_result = result.unwrap();
        assert!(code_result.success);
        assert!(code_result.generated_code.is_some());
        let code = code_result.generated_code.unwrap();
        assert!(code.contains("requests"));
        assert!(code.contains("https://api.example.com/data"));
        assert!(code.contains("POST"));
    }

    #[test]
    fn test_generate_code_from_cursor_with_delimiter() {
        let editor_text = "GET https://api.example.com/first\n\n###\n\nPOST https://api.example.com/second\n\n###\n\nGET https://api.example.com/third";
        let cursor_pos = 60; // In the second request
        let file_path = PathBuf::from("test.http");

        let result =
            generate_code_from_cursor(editor_text, cursor_pos, &file_path, "javascript", None);

        assert!(result.is_ok());
        let code_result = result.unwrap();
        assert!(code_result.success);
        let code = code_result.generated_code.unwrap();
        assert!(code.contains("https://api.example.com/second"));
        assert!(!code.contains("https://api.example.com/first"));
        assert!(!code.contains("https://api.example.com/third"));
    }

    #[test]
    fn test_generate_code_from_request_direct() {
        let request = HttpRequest::new(
            "test-123".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );

        let result = generate_code_from_request(&request, Language::JavaScript, None);

        assert!(result.success);
        assert!(result.generated_code.is_some());
        assert_eq!(result.language, Some(Language::JavaScript));
    }

    #[test]
    fn test_generate_code_from_request_with_library() {
        let request = HttpRequest::new(
            "test-456".to_string(),
            HttpMethod::POST,
            "https://api.example.com/data".to_string(),
        );

        let result = generate_code_from_request(
            &request,
            Language::Python,
            Some(crate::codegen::Library::Requests),
        );

        assert!(result.success);
        assert!(result.generated_code.is_some());
        assert_eq!(result.language, Some(Language::Python));
        assert_eq!(result.library, Some(crate::codegen::Library::Requests));
    }

    #[test]
    fn test_generate_code_invalid_cursor() {
        let editor_text = "";
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        let result =
            generate_code_from_cursor(editor_text, cursor_pos, &file_path, "javascript", None);

        assert!(result.is_err());
    }

    #[test]
    fn test_generate_code_invalid_language() {
        let editor_text = "GET https://api.example.com/users";
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        let result =
            generate_code_from_cursor(editor_text, cursor_pos, &file_path, "invalid", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_paste_curl_from_clipboard_simple() {
        let curl = "curl https://api.example.com/users";
        let result = paste_curl_from_clipboard(curl);

        assert!(result.success);
        assert!(result.request.is_some());
        assert!(result
            .formatted_request
            .contains("GET https://api.example.com/users"));
        assert!(result.formatted_request.contains("# Generated from cURL"));
    }

    #[test]
    fn test_paste_curl_from_clipboard_with_headers() {
        let curl = r#"curl -H "Authorization: Bearer token123" -H "Accept: application/json" https://api.example.com/data"#;
        let result = paste_curl_from_clipboard(curl);

        assert!(result.success);
        assert!(result
            .formatted_request
            .contains("Authorization: Bearer token123"));
        assert!(result
            .formatted_request
            .contains("Accept: application/json"));
    }

    #[test]
    fn test_paste_curl_from_clipboard_post_with_body() {
        let curl = r#"curl -X POST -d '{"name":"John"}' https://api.example.com/users"#;
        let result = paste_curl_from_clipboard(curl);

        assert!(result.success);
        assert!(result
            .formatted_request
            .contains("POST https://api.example.com/users"));
        assert!(result.formatted_request.contains(r#"{"name":"John"}"#));
    }

    #[test]
    fn test_paste_curl_from_clipboard_empty() {
        let result = paste_curl_from_clipboard("");
        assert!(!result.success);
        assert!(result.message.contains("No content"));
    }

    #[test]
    fn test_paste_curl_from_clipboard_not_curl() {
        let result = paste_curl_from_clipboard("GET https://example.com");
        assert!(!result.success);
        assert!(result
            .message
            .contains("does not appear to be a cURL command"));
    }

    #[test]
    fn test_paste_curl_from_clipboard_multiline() {
        let curl = r#"curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}' \
  https://api.example.com/endpoint"#;
        let result = paste_curl_from_clipboard(curl);

        assert!(result.success);
        assert!(result
            .formatted_request
            .contains("POST https://api.example.com/endpoint"));
        assert!(result
            .formatted_request
            .contains("Content-Type: application/json"));
    }

    #[test]
    fn test_copy_as_curl_from_cursor_simple() {
        let editor_text = "GET https://api.example.com/users";
        let cursor_pos = 5;
        let file_path = PathBuf::from("test.http");

        let result = copy_as_curl_from_cursor(editor_text, cursor_pos, &file_path);
        assert!(result.is_ok());

        let curl_result = result.unwrap();
        assert!(curl_result.success);
        assert!(curl_result.curl_command.contains("curl"));
        assert!(curl_result
            .curl_command
            .contains("https://api.example.com/users"));
    }

    #[test]
    fn test_copy_as_curl_from_cursor_with_headers() {
        let editor_text = "POST https://api.example.com/data\nAuthorization: Bearer token123\nContent-Type: application/json\n\n{\"key\": \"value\"}";
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        let result = copy_as_curl_from_cursor(editor_text, cursor_pos, &file_path);
        assert!(result.is_ok());

        let curl_result = result.unwrap();
        assert!(curl_result.success);
        assert!(curl_result.curl_command.contains("curl"));
        assert!(curl_result
            .curl_command
            .contains("https://api.example.com/data"));
        assert!(curl_result.curl_command.contains("Authorization"));
        assert!(curl_result.curl_command.contains("Bearer token123"));
        assert!(curl_result.preview.len() <= 60);
    }

    #[test]
    fn test_copy_as_curl_from_cursor_with_delimiter() {
        let editor_text = "GET https://example.com/first\n\n###\n\nPOST https://example.com/second\nContent-Type: application/json";
        let cursor_pos = 50; // Position in second request
        let file_path = PathBuf::from("test.http");

        let result = copy_as_curl_from_cursor(editor_text, cursor_pos, &file_path);
        assert!(result.is_ok());

        let curl_result = result.unwrap();
        assert!(curl_result.success);
        assert!(curl_result
            .curl_command
            .contains("https://example.com/second"));
        assert!(!curl_result
            .curl_command
            .contains("https://example.com/first"));
    }

    #[test]
    fn test_copy_as_curl_from_cursor_invalid_position() {
        let editor_text = "GET https://example.com";
        let cursor_pos = 1000; // Out of bounds
        let file_path = PathBuf::from("test.http");

        let result = copy_as_curl_from_cursor(editor_text, cursor_pos, &file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_as_curl_preview_truncation() {
        let editor_text =
            "GET https://api.example.com/very/long/path/that/exceeds/fifty/characters/for/sure";
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        let result = copy_as_curl_from_cursor(editor_text, cursor_pos, &file_path);
        assert!(result.is_ok());

        let curl_result = result.unwrap();
        assert!(curl_result.preview.len() <= 60);
        if curl_result.curl_command.len() > 50 {
            assert!(curl_result.preview.contains("..."));
        }
    }

    #[test]
    fn test_curl_round_trip() {
        // Test that converting to cURL and back preserves the request
        let original_text = "POST https://api.example.com/users\nContent-Type: application/json\nAuthorization: Bearer token123\n\n{\"name\": \"John\"}";
        let cursor_pos = 10;
        let file_path = PathBuf::from("test.http");

        // Convert to cURL
        let copy_result = copy_as_curl_from_cursor(original_text, cursor_pos, &file_path).unwrap();
        assert!(copy_result.success);

        // Convert back from cURL
        let paste_result = paste_curl_from_clipboard(&copy_result.curl_command);
        assert!(paste_result.success);

        // Verify key elements are preserved
        assert!(paste_result
            .formatted_request
            .contains("POST https://api.example.com/users"));
        assert!(paste_result
            .formatted_request
            .contains("Content-Type: application/json"));
        assert!(paste_result
            .formatted_request
            .contains("Authorization: Bearer token123"));
        assert!(paste_result
            .formatted_request
            .contains(r#"{"name": "John"}"#));
    }

    // Response actions tests
    #[test]
    fn test_save_response_command() {
        use crate::formatter::{ContentType, ResponseMetadata};
        use std::collections::HashMap;
        use std::time::Duration;

        let request = HttpRequest {
            id: "test-123".to_string(),
            method: crate::models::request::HttpMethod::GET,
            url: "https://api.example.com/users".to_string(),
            http_version: Some("HTTP/1.1".to_string()),
            headers: HashMap::new(),
            body: None,
            line_number: 0,
            file_path: PathBuf::from("test.http"),
        };

        let response = FormattedResponse {
            content_type: ContentType::Json,
            formatted_body: r#"{"users": []}"#.to_string(),
            raw_body: r#"{"users": []}"#.to_string(),
            status_line: "HTTP/1.1 200 OK".to_string(),
            headers_text: "Content-Type: application/json\n".to_string(),
            metadata: ResponseMetadata {
                status_code: 200,
                status_text: "OK".to_string(),
                duration: Duration::from_millis(100),
                size: 13,
                content_type: ContentType::Json,
                is_success: true,
                is_truncated: false,
                timing_breakdown: "Total: 100ms".to_string(),
            },
            highlight_info: None,
            is_formatted: true,
        };

        let result = save_response_command(&response, &request, SaveOption::BodyOnly);
        assert!(result.success);
        assert!(result.message.contains("response body"));
    }

    #[test]
    fn test_copy_response_command() {
        use crate::formatter::{ContentType, ResponseMetadata};
        use std::time::Duration;

        let response = FormattedResponse {
            content_type: ContentType::Json,
            formatted_body: r#"{"test": "data"}"#.to_string(),
            raw_body: r#"{"test": "data"}"#.to_string(),
            status_line: "HTTP/1.1 200 OK".to_string(),
            headers_text: "Content-Type: application/json\n".to_string(),
            metadata: ResponseMetadata {
                status_code: 200,
                status_text: "OK".to_string(),
                duration: Duration::from_millis(100),
                size: 16,
                content_type: ContentType::Json,
                is_success: true,
                is_truncated: false,
                timing_breakdown: "Total: 100ms".to_string(),
            },
            highlight_info: None,
            is_formatted: true,
        };

        let result = copy_response_command(&response, CopyOption::Body);
        assert!(result.success);
        assert_eq!(result.content, r#"{"test": "data"}"#);
    }

    #[test]
    fn test_toggle_raw_view_command() {
        use crate::formatter::{ContentType, ResponseMetadata};
        use std::time::Duration;

        let response = FormattedResponse {
            content_type: ContentType::Json,
            formatted_body: r#"{"test": "data"}"#.to_string(),
            raw_body: r#"{"test":"data"}"#.to_string(),
            status_line: "HTTP/1.1 200 OK".to_string(),
            headers_text: "Content-Type: application/json\n".to_string(),
            metadata: ResponseMetadata {
                status_code: 200,
                status_text: "OK".to_string(),
                duration: Duration::from_millis(100),
                size: 16,
                content_type: ContentType::Json,
                is_success: true,
                is_truncated: false,
                timing_breakdown: "Total: 100ms".to_string(),
            },
            highlight_info: None,
            is_formatted: true,
        };

        let toggled = toggle_raw_view_command(&response);
        assert!(!toggled.is_formatted);
        assert_eq!(toggled.raw_body, r#"{"test":"data"}"#);
    }
}

/// Save a response to a file
///
/// Prepares response content for saving with suggested filename.
///
/// # Arguments
///
/// * `response` - The formatted response to save
/// * `request` - The original HTTP request
/// * `option` - What part of the response to save
///
/// # Returns
///
/// A `SaveResponseResult` with the content and metadata
///
/// # Examples
///
/// ```ignore
/// use rest_client::commands::save_response_command;
/// use rest_client::ui::response_actions::SaveOption;
/// use rest_client::formatter::FormattedResponse;
/// use rest_client::models::request::HttpRequest;
///
/// let result = save_response_command(&response, &request, SaveOption::BodyOnly);
/// println!("Suggested path: {:?}", result.suggested_path);
/// ```
pub fn save_response_command(
    response: &FormattedResponse,
    request: &HttpRequest,
    option: SaveOption,
) -> SaveResponseResult {
    save_response(response, request, option)
}

/// Copy response data to clipboard
///
/// Prepares response content for copying based on the specified option.
///
/// # Arguments
///
/// * `response` - The formatted response to copy from
/// * `option` - What part of the response to copy
///
/// # Returns
///
/// A `CopyResponseResult` with the content and metadata
///
/// # Examples
///
/// ```ignore
/// use rest_client::commands::copy_response_command;
/// use rest_client::ui::response_actions::CopyOption;
/// use rest_client::formatter::FormattedResponse;
///
/// let result = copy_response_command(&response, CopyOption::Headers);
/// println!("{}", result.message);
/// ```
pub fn copy_response_command(
    response: &FormattedResponse,
    option: CopyOption,
) -> CopyResponseResult {
    copy_response(response, option)
}

/// Toggle between formatted and raw view of a response
///
/// Switches the response display between formatted (pretty-printed) and raw (exact bytes).
///
/// # Arguments
///
/// * `response` - The formatted response to toggle
///
/// # Returns
///
/// A new `FormattedResponse` with the view toggled
///
/// # Examples
///
/// ```ignore
/// use rest_client::commands::toggle_raw_view_command;
/// use rest_client::formatter::FormattedResponse;
///
/// let toggled = toggle_raw_view_command(&response);
/// assert_eq!(toggled.is_formatted, !response.is_formatted);
/// ```
pub fn toggle_raw_view_command(response: &FormattedResponse) -> FormattedResponse {
    toggle_raw_view(response)
}

/// Fold large sections in a response body
///
/// Collapses large sections of JSON or XML responses to make them more manageable.
///
/// # Arguments
///
/// * `response` - The formatted response to fold
/// * `fold_threshold` - Minimum number of lines before folding (default: 10)
///
/// # Returns
///
/// A `FoldResponseResult` with the folded content
///
/// # Examples
///
/// ```ignore
/// use rest_client::commands::fold_response_command;
/// use rest_client::formatter::FormattedResponse;
///
/// let result = fold_response_command(&response, 10);
/// println!("Folded {} sections", result.sections_folded);
/// ```
pub fn fold_response_command(
    response: &FormattedResponse,
    fold_threshold: usize,
) -> FoldResponseResult {
    fold_response(response, fold_threshold)
}
