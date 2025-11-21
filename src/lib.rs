//! REST Client Extension for Zed Editor
//!
//! This extension provides HTTP request execution capabilities within Zed,
//! allowing users to send requests from `.http` and `.rest` files and view
//! formatted responses.
//!
//! # Architecture
//!
//! The extension is organized into several modules:
//!
//! - **models**: Core data structures for HTTP requests and responses
//! - **parser**: Parses `.http` files into structured request objects
//! - **executor**: Executes HTTP requests using reqwest
//! - **formatter**: Formats responses for display with content type detection
//! - **commands**: Command handlers for Zed integration
//!
//! # Command Integration
//!
//! The main command is `send_request_command` which:
//! 1. Extracts the request block at the cursor position (bounded by `###` delimiters)
//! 2. Parses the request using the HTTP parser
//! 3. Executes the request with configurable timeout
//! 4. Formats the response based on content type (JSON, XML, HTML, etc.)
//! 5. Returns formatted output for display in a new editor buffer
//!
//! # Usage
//!
//! To use this extension in Zed:
//! 1. Create a `.http` or `.rest` file
//! 2. Write HTTP requests using standard syntax:
//!    ```http
//!    GET https://api.example.com/users
//!    Authorization: Bearer token123
//!
//!    ###
//!
//!    POST https://api.example.com/users
//!    Content-Type: application/json
//!
//!    {"name": "John", "email": "john@example.com"}
//!    ```
//! 3. Place cursor within a request block
//! 4. Execute the "Send Request" command
//! 5. View the formatted response in a new buffer
//!
//! # Note on Zed Extension API
//!
//! This extension is designed to work with the Zed extension API. The actual
//! command registration and editor integration will depend on the Zed extension
//! system capabilities. The core logic is implemented in the `commands` module
//! and can be called by the extension host.

use zed_extension_api as zed;

pub mod commands;
pub mod executor;
pub mod formatter;
pub mod models;
pub mod parser;

/// REST Client extension for Zed.
///
/// This extension provides HTTP request execution capabilities directly
/// within the Zed editor for `.http` and `.rest` files.
struct RestClientExtension;

impl zed::Extension for RestClientExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        Err("Language server not yet implemented".to_string())
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        Ok(None)
    }
}

zed::register_extension!(RestClientExtension);
