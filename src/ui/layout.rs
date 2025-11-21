//! Layout Management for Response Pane
//!
//! This module provides configuration and utilities for managing the response pane layout.
//!
//! # Architecture Note
//!
//! Since Zed WASM extensions (v0.7.0) cannot programmatically create panes/tabs,
//! this module focuses on:
//! 1. Configuration management for layout preferences
//! 2. Formatting responses for optimal display in slash command output
//! 3. Providing metadata and structure for future pane API integration
//!
//! The actual display happens via `SlashCommandOutput` text sections.

use super::response_actions::format_action_menu;
use super::response_pane::{PanePosition, ResponsePane, ResponseTab};
use crate::formatter::FormattedResponse;
use crate::models::request::HttpRequest;

/// Configuration for response pane layout
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Position where response pane should appear
    pub position: PanePosition,

    /// Maximum number of response tabs to keep
    pub max_tabs: usize,

    /// Whether to show response in a compact format
    pub compact_mode: bool,

    /// Whether to include request details in response display
    pub show_request_details: bool,

    /// Whether to include timing information
    pub show_timing: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            position: PanePosition::Right,
            max_tabs: 10,
            compact_mode: false,
            show_request_details: true,
            show_timing: true,
        }
    }
}

impl LayoutConfig {
    /// Create a new layout configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the pane position
    pub fn with_position(mut self, position: PanePosition) -> Self {
        self.position = position;
        self
    }

    /// Set the maximum number of tabs
    pub fn with_max_tabs(mut self, max_tabs: usize) -> Self {
        self.max_tabs = max_tabs.max(1);
        self
    }

    /// Enable or disable compact mode
    pub fn with_compact_mode(mut self, compact: bool) -> Self {
        self.compact_mode = compact;
        self
    }

    /// Load configuration from settings (placeholder for future config integration)
    pub fn from_settings() -> Self {
        // TODO: When Zed adds settings API for extensions, load from user settings
        // For now, return defaults
        Self::default()
    }
}

/// Manages the response pane layout and state
pub struct LayoutManager {
    /// Response pane instance
    pane: ResponsePane,

    /// Layout configuration
    config: LayoutConfig,
}

impl LayoutManager {
    /// Create a new layout manager with the given configuration
    pub fn new(config: LayoutConfig) -> Self {
        let mut pane = ResponsePane::new(config.position);
        pane.set_max_tabs(config.max_tabs);

        Self { pane, config }
    }

    /// Create a new layout manager with default configuration
    pub fn with_defaults() -> Self {
        Self::new(LayoutConfig::default())
    }

    /// Open or reuse the response pane with a new response
    ///
    /// This is the main entry point for displaying responses.
    ///
    /// # Arguments
    ///
    /// * `response` - The formatted response to display
    /// * `request` - The original HTTP request
    /// * `request_id` - Unique identifier for the request
    ///
    /// # Returns
    ///
    /// A formatted string ready for display in a slash command output
    pub fn manage_pane_layout(
        &mut self,
        response: FormattedResponse,
        request: HttpRequest,
        request_id: &str,
    ) -> String {
        // Create a new response tab
        let tab_id = self.pane.create_response_tab(response, request, request_id);

        // Get the newly created tab
        if let Some(tab) = self.pane.get_tab(&tab_id) {
            self.format_response_for_display(tab)
        } else {
            "Error: Failed to create response tab".to_string()
        }
    }

    /// Format a response tab for display
    ///
    /// Applies layout configuration to produce formatted output
    fn format_response_for_display(&self, tab: &ResponseTab) -> String {
        if self.config.compact_mode {
            self.format_compact(tab)
        } else {
            self.format_full(tab)
        }
    }

    /// Format response in full mode with all details
    fn format_full(&self, tab: &ResponseTab) -> String {
        let mut output = String::new();

        // Header with tab information
        output.push_str(
            "╔═══════════════════════════════════════════════════════════════════════╗\n",
        );
        output.push_str(&format!("║ {}{}║\n", self.center_text(&tab.title, 69), ""));
        output.push_str(
            "╠═══════════════════════════════════════════════════════════════════════╣\n",
        );

        // Request details
        if self.config.show_request_details {
            output.push_str(&format!(
                "║ Request:  {} {}{}║\n",
                tab.request.method,
                tab.request.url,
                self.pad_to_width(&format!("{} {}", tab.request.method, tab.request.url), 59)
            ));
        }

        // Timing information
        if self.config.show_timing {
            output.push_str(&format!(
                "║ Duration: {:?}{}║\n",
                tab.response.metadata.duration,
                self.pad_to_width(&format!("{:?}", tab.response.metadata.duration), 60)
            ));
            output.push_str(&format!(
                "║ Size:     {} bytes{}║\n",
                tab.response.metadata.size,
                self.pad_to_width(&format!("{} bytes", tab.response.metadata.size), 60)
            ));
        }

        output.push_str(&format!(
            "║ Status:   {}{}║\n",
            tab.response.status_line,
            self.pad_to_width(&tab.response.status_line, 60)
        ));
        output.push_str(&format!(
            "║ Tab ID:   {}{}║\n",
            &tab.id[..8],
            self.pad_to_width(&tab.id[..8], 60)
        ));

        output.push_str(
            "╚═══════════════════════════════════════════════════════════════════════╝\n\n",
        );

        // Response content
        output.push_str(&tab.response.to_display_string());

        // Action menu
        output.push_str("\n");
        output.push_str(&format_action_menu(&tab.response));

        // Footer with tab management hints
        output.push_str("\n");
        output.push_str(
            "─────────────────────────────────────────────────────────────────────────\n",
        );
        output.push_str(&format!(
            "Active tabs: {} of {}  |  Position: {}\n",
            self.pane.tab_count(),
            self.pane.max_tabs(),
            self.config.position.as_str()
        ));

        output
    }

    /// Format response in compact mode with minimal details
    fn format_compact(&self, tab: &ResponseTab) -> String {
        let mut output = String::new();

        // Minimal header
        output.push_str(&format!(
            "═══ {} ═══ {} ═══\n\n",
            tab.title, tab.response.status_line
        ));

        // Response body only
        output.push_str(&tab.response.formatted_body);

        output.push_str("\n");

        output
    }

    /// Get information about all open tabs
    ///
    /// # Returns
    ///
    /// Formatted string listing all tabs with their status
    pub fn list_open_tabs(&self) -> String {
        let tabs = self.pane.list_tabs();

        if tabs.is_empty() {
            return "No response tabs currently open.".to_string();
        }

        let mut output = String::new();
        output.push_str("Open Response Tabs:\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n\n");

        for (i, (id, title, is_active)) in tabs.iter().enumerate() {
            let marker = if *is_active { "→" } else { " " };
            let status = if *is_active { " (active)" } else { "" };
            output.push_str(&format!(
                "{} {}. {}{}  [{}]\n",
                marker,
                i + 1,
                title,
                status,
                &id[..8]
            ));
        }

        output.push_str(&format!(
            "\nTotal: {} / {} tabs\n",
            tabs.len(),
            self.pane.max_tabs()
        ));

        output
    }

    /// Close a specific response tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to close (can be partial, e.g., first 8 chars)
    ///
    /// # Returns
    ///
    /// Success message if closed, error message if not found
    pub fn close_tab(&mut self, tab_id: &str) -> String {
        // Try exact match first
        if self.pane.close_response_tab(tab_id) {
            return format!("✓ Closed response tab: {}", tab_id);
        }

        // Try partial match (first 8 chars)
        let tabs = self.pane.list_tabs();
        for (id, _, _) in tabs {
            if id.starts_with(tab_id) {
                if self.pane.close_response_tab(&id) {
                    return format!("✓ Closed response tab: {}", &id[..8]);
                }
            }
        }

        format!("✗ Tab not found: {}", tab_id)
    }

    /// Close all response tabs
    ///
    /// # Returns
    ///
    /// Confirmation message
    pub fn close_all_tabs(&mut self) -> String {
        let count = self.pane.tab_count();
        self.pane.close_all_responses();
        format!("✓ Closed {} response tab(s)", count)
    }

    /// Switch to a specific tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to activate
    ///
    /// # Returns
    ///
    /// Formatted display of the activated tab, or error message
    pub fn switch_tab(&mut self, tab_id: &str) -> String {
        // Try exact match first
        if self.pane.switch_to_tab(tab_id) {
            if let Some(tab) = self.pane.get_active_tab() {
                return self.format_response_for_display(tab);
            }
        }

        // Try partial match
        let tabs = self.pane.list_tabs();
        for (id, _, _) in tabs {
            if id.starts_with(tab_id) {
                if self.pane.switch_to_tab(&id) {
                    if let Some(tab) = self.pane.get_active_tab() {
                        return self.format_response_for_display(tab);
                    }
                }
            }
        }

        format!("✗ Tab not found: {}", tab_id)
    }

    /// Get the active response tab display
    ///
    /// # Returns
    ///
    /// Formatted display of active tab, or message if no tabs open
    pub fn get_active_tab_display(&self) -> String {
        if let Some(tab) = self.pane.get_active_tab() {
            self.format_response_for_display(tab)
        } else {
            "No response tabs currently open.".to_string()
        }
    }

    /// Update layout configuration
    pub fn update_config(&mut self, config: LayoutConfig) {
        self.pane.set_position(config.position);
        self.pane.set_max_tabs(config.max_tabs);
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &LayoutConfig {
        &self.config
    }

    /// Helper: Center text within a given width
    fn center_text(&self, text: &str, width: usize) -> String {
        if text.len() >= width {
            return text[..width].to_string();
        }

        let padding = width - text.len();
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;

        format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
    }

    /// Helper: Pad text to a specific width
    fn pad_to_width(&self, text: &str, width: usize) -> String {
        if text.len() >= width {
            String::new()
        } else {
            " ".repeat(width - text.len())
        }
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formatter::{ContentType, ResponseMetadata};
    use crate::models::request::HttpMethod;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;
    use uuid::Uuid;

    fn create_test_request(method: HttpMethod, url: &str) -> HttpRequest {
        HttpRequest {
            id: Uuid::new_v4().to_string(),
            method,
            url: url.to_string(),
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
            formatted_body: r#"{"status": "ok"}"#.to_string(),
            raw_body: r#"{"status": "ok"}"#.to_string(),
            status_line: "HTTP/1.1 200 OK".to_string(),
            headers_text: "Content-Type: application/json\n".to_string(),
            metadata: ResponseMetadata {
                status_code: 200,
                status_text: "OK".to_string(),
                duration: Duration::from_millis(150),
                size: 17,
                content_type: ContentType::Json,
                is_success: true,
                is_truncated: false,
                timing_breakdown: "Total: 150ms".to_string(),
            },
            highlight_info: None,
            is_formatted: true,
        }
    }

    #[test]
    fn test_layout_config_builder() {
        let config = LayoutConfig::new()
            .with_position(PanePosition::Left)
            .with_max_tabs(5)
            .with_compact_mode(true);

        assert_eq!(config.position, PanePosition::Left);
        assert_eq!(config.max_tabs, 5);
        assert!(config.compact_mode);
    }

    #[test]
    fn test_manage_pane_layout() {
        let mut manager = LayoutManager::with_defaults();
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let response = create_test_response();

        let output = manager.manage_pane_layout(response, request, "req-123");

        assert!(output.contains("GET api.example.com/users"));
        assert!(output.contains("HTTP/1.1 200 OK"));
    }

    #[test]
    fn test_compact_mode() {
        let config = LayoutConfig::new().with_compact_mode(true);
        let mut manager = LayoutManager::new(config);

        let request = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let response = create_test_response();

        let output = manager.manage_pane_layout(response, request, "req-123");

        // Compact mode should have minimal formatting
        assert!(output.contains("GET api.example.com/users"));
        assert!(!output.contains("╔═══")); // No fancy borders in compact mode
    }

    #[test]
    fn test_list_open_tabs() {
        let mut manager = LayoutManager::with_defaults();

        // Initially empty
        let list = manager.list_open_tabs();
        assert!(list.contains("No response tabs"));

        // Add a tab
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let response = create_test_response();
        manager.manage_pane_layout(response, request, "req-123");

        let list = manager.list_open_tabs();
        assert!(list.contains("GET api.example.com/users"));
        assert!(list.contains("(active)"));
    }

    #[test]
    fn test_close_tab() {
        let mut manager = LayoutManager::with_defaults();

        let request = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let response = create_test_response();
        manager.manage_pane_layout(response, request, "req-123");

        assert_eq!(manager.pane.tab_count(), 1);

        // Get tab ID and close it
        let tabs = manager.pane.list_tabs();
        let tab_id = &tabs[0].0;
        let result = manager.close_tab(tab_id);

        assert!(result.contains("✓ Closed"));
        assert_eq!(manager.pane.tab_count(), 0);
    }

    #[test]
    fn test_close_all_tabs() {
        let mut manager = LayoutManager::with_defaults();

        // Add multiple tabs
        for i in 0..3 {
            let request =
                create_test_request(HttpMethod::GET, &format!("https://api.example.com/{}", i));
            let response = create_test_response();
            manager.manage_pane_layout(response, request, &format!("req-{}", i));
        }

        assert_eq!(manager.pane.tab_count(), 3);

        let result = manager.close_all_tabs();
        assert!(result.contains("✓ Closed 3"));
        assert_eq!(manager.pane.tab_count(), 0);
    }

    #[test]
    fn test_center_text() {
        let manager = LayoutManager::with_defaults();

        let centered = manager.center_text("Hello", 10);
        assert_eq!(centered.len(), 10);
        assert!(centered.contains("Hello"));

        // Text longer than width should be truncated
        let long = manager.center_text("Very long text here", 10);
        assert_eq!(long.len(), 10);
    }
}
