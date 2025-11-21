//! UI Module for REST Client Extension
//!
//! This module provides user interface components and layout management for the REST client.
//!
//! # Architecture
//!
//! The UI module is organized into:
//! - **response_pane**: Response tab management and display state
//! - **layout**: Layout configuration and response formatting
//!
//! # Current Limitations
//!
//! The Zed WASM extension API (v0.7.0) does not provide programmatic APIs for:
//! - Creating editor panes
//! - Managing tabs
//! - Split view manipulation
//! - Custom UI rendering
//!
//! Extensions can only return formatted text through `SlashCommandOutput`.
//!
//! # What This Module Provides
//!
//! Despite the API limitations, this module provides:
//! 1. **Response state management**: Track multiple responses with metadata
//! 2. **Professional formatting**: Structured, readable response display
//! 3. **Tab simulation**: Virtual tab management for organizing responses
//! 4. **Configuration**: Layout preferences and display options
//! 5. **Future-proofing**: Architecture ready for when Zed adds pane APIs
//!
//! # Usage
//!
//! ## Basic Response Display
//!
//! ```ignore
//! use rest_client::ui::{LayoutManager, LayoutConfig};
//! use rest_client::formatter::FormattedResponse;
//! use rest_client::models::request::HttpRequest;
//!
//! let mut manager = LayoutManager::with_defaults();
//!
//! // Display a response
//! let output = manager.manage_pane_layout(response, request, "req-123");
//! // Output can be returned in SlashCommandOutput
//! ```
//!
//! ## Tab Management
//!
//! ```ignore
//! use rest_client::ui::LayoutManager;
//!
//! let mut manager = LayoutManager::with_defaults();
//!
//! // List all open tabs
//! let tabs_list = manager.list_open_tabs();
//!
//! // Close a specific tab
//! let result = manager.close_tab("abc12345");
//!
//! // Close all tabs
//! let result = manager.close_all_tabs();
//! ```
//!
//! ## Custom Layout Configuration
//!
//! ```ignore
//! use rest_client::ui::{LayoutManager, LayoutConfig};
//! use rest_client::ui::response_pane::PanePosition;
//!
//! let config = LayoutConfig::new()
//!     .with_position(PanePosition::Bottom)
//!     .with_max_tabs(5)
//!     .with_compact_mode(true);
//!
//! let mut manager = LayoutManager::new(config);
//! ```
//!
//! # Integration with Commands
//!
//! The UI module is designed to integrate seamlessly with command handlers:
//!
//! ```ignore
//! use rest_client::ui::LayoutManager;
//! use rest_client::executor::execute_request;
//! use rest_client::formatter::format_response;
//!
//! async fn send_request_with_ui(
//!     request: HttpRequest,
//!     manager: &mut LayoutManager,
//! ) -> Result<String, String> {
//!     let config = ExecutionConfig::default();
//!     let response = execute_request(&request, &config)?;
//!     let formatted = format_response(&response);
//!
//!     let request_id = request.id.clone();
//!     let display = manager.manage_pane_layout(formatted, request, &request_id);
//!
//!     Ok(display)
//! }
//! ```

pub mod layout;
pub mod response_actions;
pub mod response_pane;

// Re-export commonly used types for convenience
pub use layout::{LayoutConfig, LayoutManager};
pub use response_actions::{
    copy_response, fold_response, format_action_menu, save_response, suggest_filename,
    toggle_raw_view, CopyOption, CopyResponseResult, FoldResponseResult, SaveOption,
    SaveResponseResult,
};
pub use response_pane::{PanePosition, ResponsePane, ResponseTab};

/// Create a default layout manager instance
///
/// This is a convenience function for quick setup.
///
/// # Returns
///
/// A `LayoutManager` with default configuration
///
/// # Example
///
/// ```no_run
/// use rest_client::ui::create_default_layout_manager;
///
/// let mut manager = create_default_layout_manager();
/// ```
pub fn create_default_layout_manager() -> LayoutManager {
    LayoutManager::with_defaults()
}

/// Create a layout manager with custom configuration
///
/// # Arguments
///
/// * `position` - Pane position preference
/// * `max_tabs` - Maximum number of tabs to keep in memory
/// * `compact` - Whether to use compact display mode
///
/// # Returns
///
/// A configured `LayoutManager`
///
/// # Example
///
/// ```no_run
/// use rest_client::ui::{create_layout_manager, PanePosition};
///
/// let mut manager = create_layout_manager(PanePosition::Left, 5, true);
/// ```
pub fn create_layout_manager(
    position: PanePosition,
    max_tabs: usize,
    compact: bool,
) -> LayoutManager {
    let config = LayoutConfig::new()
        .with_position(position)
        .with_max_tabs(max_tabs)
        .with_compact_mode(compact);

    LayoutManager::new(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::{ContentType, FormattedResponse, ResponseMetadata};
    use crate::models::request::{HttpMethod, HttpRequest};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;
    use uuid::Uuid;

    fn create_test_request() -> HttpRequest {
        HttpRequest {
            id: Uuid::new_v4().to_string(),
            method: HttpMethod::GET,
            url: "https://api.example.com/test".to_string(),
            http_version: Some("HTTP/1.1".to_string()),
            headers: HashMap::new(),
            body: None,
            line_number: 0,
            file_path: PathBuf::from("test.http"),
        }
    }

    fn create_test_response() -> FormattedResponse {
        FormattedResponse {
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
        }
    }

    #[test]
    fn test_create_default_layout_manager() {
        let manager = create_default_layout_manager();
        assert_eq!(manager.config().position, PanePosition::Right);
        assert_eq!(manager.config().max_tabs, 10);
    }

    #[test]
    fn test_create_layout_manager_custom() {
        let manager = create_layout_manager(PanePosition::Bottom, 3, true);
        assert_eq!(manager.config().position, PanePosition::Bottom);
        assert_eq!(manager.config().max_tabs, 3);
        assert!(manager.config().compact_mode);
    }

    #[test]
    fn test_ui_module_integration() {
        let mut manager = create_default_layout_manager();
        let request = create_test_request();
        let response = create_test_response();

        let output = manager.manage_pane_layout(response, request, "test-req-1");

        assert!(output.contains("GET api.example.com/test"));
        assert!(output.contains("HTTP/1.1 200 OK"));
    }

    #[test]
    fn test_multiple_responses() {
        let mut manager = create_layout_manager(PanePosition::Right, 3, false);

        // Add multiple responses
        for i in 0..3 {
            let request = create_test_request();
            let response = create_test_response();
            manager.manage_pane_layout(response, request, &format!("req-{}", i));
        }

        let tabs_list = manager.list_open_tabs();
        assert!(tabs_list.contains("3 / 3 tabs"));
    }
}
