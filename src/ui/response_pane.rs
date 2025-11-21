//! Response Pane Management
//!
//! This module manages response display state and tab metadata for the REST client.
//!
//! # Architecture Note
//!
//! The current Zed WASM extension API (v0.7.0) does not provide programmatic pane/tab
//! creation APIs. Extensions can only return formatted text through `SlashCommandOutput`.
//!
//! This module provides the **data structures and logic** for response pane management,
//! which can be used for:
//! 1. Formatting responses consistently
//! 2. Tracking response metadata (for history, search, etc.)
//! 3. Managing response lifecycle (creation, display, cleanup)
//! 4. Future compatibility if Zed adds pane APIs to WASM extensions
//!
//! The actual display happens via `SlashCommandOutput` text sections in `lib.rs`.

use crate::formatter::FormattedResponse;
use crate::models::request::HttpRequest;
use std::collections::VecDeque;
use uuid::Uuid;

/// Maximum number of response tabs to keep in memory
const MAX_RESPONSE_TABS: usize = 10;

/// Position where response pane should be displayed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanePosition {
    /// Display on the right side (default)
    Right,
    /// Display on the left side
    Left,
    /// Display below
    Bottom,
    /// Display above
    Top,
}

impl Default for PanePosition {
    fn default() -> Self {
        PanePosition::Right
    }
}

impl PanePosition {
    /// Parse position from string configuration
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "right" => Some(PanePosition::Right),
            "left" => Some(PanePosition::Left),
            "bottom" => Some(PanePosition::Bottom),
            "top" => Some(PanePosition::Top),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PanePosition::Right => "right",
            PanePosition::Left => "left",
            PanePosition::Bottom => "bottom",
            PanePosition::Top => "top",
        }
    }
}

/// Metadata for a response tab
#[derive(Debug, Clone)]
pub struct ResponseTab {
    /// Unique identifier for this response tab
    pub id: String,

    /// Request ID that generated this response
    pub request_id: String,

    /// Tab title (e.g., "GET example.com")
    pub title: String,

    /// Full formatted response
    pub response: FormattedResponse,

    /// Original request that generated this response
    pub request: HttpRequest,

    /// Timestamp when this response was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Whether this tab is currently active
    pub is_active: bool,
}

impl ResponseTab {
    /// Create a new response tab
    ///
    /// # Arguments
    ///
    /// * `response` - The formatted response to display
    /// * `request` - The original HTTP request
    /// * `request_id` - Unique identifier for the request
    ///
    /// # Returns
    ///
    /// A new `ResponseTab` with generated title and ID
    pub fn new(response: FormattedResponse, request: HttpRequest, request_id: &str) -> Self {
        let title = Self::generate_title(&request);
        let id = Uuid::new_v4().to_string();

        Self {
            id,
            request_id: request_id.to_string(),
            title,
            response,
            request,
            created_at: chrono::Utc::now(),
            is_active: false,
        }
    }

    /// Generate a concise tab title from the request
    ///
    /// Format: "<METHOD> <domain/path>"
    /// Truncates long URLs to keep titles readable
    fn generate_title(request: &HttpRequest) -> String {
        const MAX_URL_LENGTH: usize = 50;

        // Try to extract just domain and path
        let url_display = if let Ok(parsed_url) = url::Url::parse(&request.url) {
            let host = parsed_url.host_str().unwrap_or("unknown");
            let path = parsed_url.path();

            // Combine host and path
            let full = if path == "/" {
                host.to_string()
            } else {
                format!("{}{}", host, path)
            };

            // Truncate if too long
            if full.len() > MAX_URL_LENGTH {
                format!("{}...", &full[..MAX_URL_LENGTH - 3])
            } else {
                full
            }
        } else {
            // Fallback to truncated raw URL
            if request.url.len() > MAX_URL_LENGTH {
                format!("{}...", &request.url[..MAX_URL_LENGTH - 3])
            } else {
                request.url.clone()
            }
        };

        format!("{} {}", request.method, url_display)
    }

    /// Get a display string for this response tab
    ///
    /// Includes tab header with metadata and the formatted response content
    pub fn to_display_string(&self) -> String {
        let mut output = String::new();

        // Tab header
        output.push_str(&format!("╔══ {} ══╗\n", self.title));
        output.push_str(&format!("║ Request ID: {}\n", self.request_id));
        output.push_str(&format!(
            "║ Timestamp: {}\n",
            self.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!("║ Status: {}\n", self.response.status_line));
        output.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");

        // Response content
        output.push_str(&self.response.to_display_string());

        output
    }
}

/// Response pane manager
///
/// Manages a collection of response tabs with a maximum limit.
/// When the limit is reached, the oldest tab is removed (LRU policy).
pub struct ResponsePane {
    /// Active response tabs (newest last)
    tabs: VecDeque<ResponseTab>,

    /// Maximum number of tabs to keep
    max_tabs: usize,

    /// Configured pane position
    position: PanePosition,

    /// ID of the currently active tab
    active_tab_id: Option<String>,
}

impl ResponsePane {
    /// Create a new response pane manager
    ///
    /// # Arguments
    ///
    /// * `position` - Where the response pane should be positioned
    ///
    /// # Returns
    ///
    /// A new `ResponsePane` with default settings
    pub fn new(position: PanePosition) -> Self {
        Self {
            tabs: VecDeque::with_capacity(MAX_RESPONSE_TABS),
            max_tabs: MAX_RESPONSE_TABS,
            position,
            active_tab_id: None,
        }
    }

    /// Create a new response tab and add it to the pane
    ///
    /// If the maximum number of tabs is reached, removes the oldest tab.
    ///
    /// # Arguments
    ///
    /// * `response` - The formatted response to display
    /// * `request` - The original HTTP request
    /// * `request_id` - Unique identifier for the request
    ///
    /// # Returns
    ///
    /// The ID of the newly created tab
    pub fn create_response_tab(
        &mut self,
        response: FormattedResponse,
        request: HttpRequest,
        request_id: &str,
    ) -> String {
        // Create new tab
        let mut tab = ResponseTab::new(response, request, request_id);
        tab.is_active = true;
        let tab_id = tab.id.clone();

        // Deactivate all other tabs
        for existing_tab in &mut self.tabs {
            existing_tab.is_active = false;
        }

        // Add new tab
        self.tabs.push_back(tab);

        // Remove oldest tab if we exceed the limit
        if self.tabs.len() > self.max_tabs {
            self.tabs.pop_front();
        }

        // Update active tab ID
        self.active_tab_id = Some(tab_id.clone());

        tab_id
    }

    /// Close a specific response tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to close
    ///
    /// # Returns
    ///
    /// `true` if the tab was found and closed, `false` otherwise
    pub fn close_response_tab(&mut self, tab_id: &str) -> bool {
        let initial_len = self.tabs.len();
        self.tabs.retain(|tab| tab.id != tab_id);

        // If we closed the active tab, activate the most recent tab
        if self.active_tab_id.as_deref() == Some(tab_id) {
            self.active_tab_id = self.tabs.back().map(|tab| tab.id.clone());
            if let Some(last_tab) = self.tabs.back_mut() {
                last_tab.is_active = true;
            }
        }

        self.tabs.len() < initial_len
    }

    /// Close all response tabs
    pub fn close_all_responses(&mut self) {
        self.tabs.clear();
        self.active_tab_id = None;
    }

    /// Get the active response tab
    ///
    /// # Returns
    ///
    /// Reference to the active tab, or `None` if no tabs are open
    pub fn get_active_tab(&self) -> Option<&ResponseTab> {
        self.active_tab_id
            .as_ref()
            .and_then(|id| self.tabs.iter().find(|tab| &tab.id == id))
    }

    /// Get a specific response tab by ID
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to retrieve
    ///
    /// # Returns
    ///
    /// Reference to the tab, or `None` if not found
    pub fn get_tab(&self, tab_id: &str) -> Option<&ResponseTab> {
        self.tabs.iter().find(|tab| tab.id == tab_id)
    }

    /// Switch to a specific tab
    ///
    /// # Arguments
    ///
    /// * `tab_id` - ID of the tab to activate
    ///
    /// # Returns
    ///
    /// `true` if the tab was found and activated, `false` otherwise
    pub fn switch_to_tab(&mut self, tab_id: &str) -> bool {
        // Check if tab exists
        if !self.tabs.iter().any(|tab| tab.id == tab_id) {
            return false;
        }

        // Deactivate all tabs
        for tab in &mut self.tabs {
            tab.is_active = false;
        }

        // Activate the requested tab
        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            tab.is_active = true;
            self.active_tab_id = Some(tab_id.to_string());
            true
        } else {
            false
        }
    }

    /// Get a list of all tab titles and IDs
    ///
    /// # Returns
    ///
    /// Vector of (tab_id, title, is_active) tuples
    pub fn list_tabs(&self) -> Vec<(String, String, bool)> {
        self.tabs
            .iter()
            .map(|tab| (tab.id.clone(), tab.title.clone(), tab.is_active))
            .collect()
    }

    /// Get the number of open tabs
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Get the configured pane position
    pub fn position(&self) -> PanePosition {
        self.position
    }

    /// Set the pane position
    pub fn set_position(&mut self, position: PanePosition) {
        self.position = position;
    }

    /// Get the maximum number of tabs
    pub fn max_tabs(&self) -> usize {
        self.max_tabs
    }

    /// Set the maximum number of tabs
    ///
    /// If the new limit is lower than the current number of tabs,
    /// oldest tabs will be removed to meet the limit.
    pub fn set_max_tabs(&mut self, max_tabs: usize) {
        self.max_tabs = max_tabs.max(1); // At least 1 tab

        // Remove oldest tabs if we exceed the new limit
        while self.tabs.len() > self.max_tabs {
            self.tabs.pop_front();
        }
    }
}

impl Default for ResponsePane {
    fn default() -> Self {
        Self::new(PanePosition::default())
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
    fn test_pane_position_from_str() {
        assert_eq!(PanePosition::from_str("right"), Some(PanePosition::Right));
        assert_eq!(PanePosition::from_str("left"), Some(PanePosition::Left));
        assert_eq!(PanePosition::from_str("bottom"), Some(PanePosition::Bottom));
        assert_eq!(PanePosition::from_str("top"), Some(PanePosition::Top));
        assert_eq!(PanePosition::from_str("invalid"), None);
    }

    #[test]
    fn test_generate_title() {
        let req1 = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        assert_eq!(
            ResponseTab::generate_title(&req1),
            "GET api.example.com/users"
        );

        let req2 = create_test_request(HttpMethod::POST, "https://example.com/");
        assert_eq!(ResponseTab::generate_title(&req2), "POST example.com");

        // Test truncation
        let long_url = format!("https://example.com/{}", "a".repeat(100));
        let req3 = create_test_request(HttpMethod::GET, &long_url);
        let title = ResponseTab::generate_title(&req3);
        assert!(title.len() <= 54); // "GET " + 50 chars
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_create_response_tab() {
        let mut pane = ResponsePane::new(PanePosition::Right);
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let response = create_test_response();

        let tab_id = pane.create_response_tab(response, request, "req-123");

        assert_eq!(pane.tab_count(), 1);
        assert_eq!(pane.active_tab_id, Some(tab_id.clone()));

        let tab = pane.get_tab(&tab_id).unwrap();
        assert_eq!(tab.request_id, "req-123");
        assert!(tab.is_active);
    }

    #[test]
    fn test_max_tabs_limit() {
        let mut pane = ResponsePane::new(PanePosition::Right);
        pane.set_max_tabs(3);

        // Add 5 tabs
        for i in 0..5 {
            let request =
                create_test_request(HttpMethod::GET, &format!("https://api.example.com/{}", i));
            let response = create_test_response();
            pane.create_response_tab(response, request, &format!("req-{}", i));
        }

        // Should only keep the most recent 3
        assert_eq!(pane.tab_count(), 3);

        let tabs = pane.list_tabs();
        assert_eq!(tabs.len(), 3);
    }

    #[test]
    fn test_close_response_tab() {
        let mut pane = ResponsePane::new(PanePosition::Right);
        let request = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let response = create_test_response();

        let tab_id = pane.create_response_tab(response, request, "req-123");
        assert_eq!(pane.tab_count(), 1);

        let closed = pane.close_response_tab(&tab_id);
        assert!(closed);
        assert_eq!(pane.tab_count(), 0);
        assert!(pane.active_tab_id.is_none());
    }

    #[test]
    fn test_close_all_responses() {
        let mut pane = ResponsePane::new(PanePosition::Right);

        for i in 0..3 {
            let request =
                create_test_request(HttpMethod::GET, &format!("https://api.example.com/{}", i));
            let response = create_test_response();
            pane.create_response_tab(response, request, &format!("req-{}", i));
        }

        assert_eq!(pane.tab_count(), 3);

        pane.close_all_responses();
        assert_eq!(pane.tab_count(), 0);
        assert!(pane.active_tab_id.is_none());
    }

    #[test]
    fn test_switch_to_tab() {
        let mut pane = ResponsePane::new(PanePosition::Right);

        let request1 = create_test_request(HttpMethod::GET, "https://api.example.com/1");
        let response1 = create_test_response();
        let tab_id1 = pane.create_response_tab(response1, request1, "req-1");

        let request2 = create_test_request(HttpMethod::GET, "https://api.example.com/2");
        let response2 = create_test_response();
        let tab_id2 = pane.create_response_tab(response2, request2, "req-2");

        // Tab 2 should be active
        assert_eq!(pane.active_tab_id, Some(tab_id2.clone()));

        // Switch back to tab 1
        let switched = pane.switch_to_tab(&tab_id1);
        assert!(switched);
        assert_eq!(pane.active_tab_id, Some(tab_id1.clone()));

        let tab1 = pane.get_tab(&tab_id1).unwrap();
        assert!(tab1.is_active);

        let tab2 = pane.get_tab(&tab_id2).unwrap();
        assert!(!tab2.is_active);
    }

    #[test]
    fn test_list_tabs() {
        let mut pane = ResponsePane::new(PanePosition::Right);

        let request1 = create_test_request(HttpMethod::GET, "https://api.example.com/users");
        let response1 = create_test_response();
        pane.create_response_tab(response1, request1, "req-1");

        let request2 = create_test_request(HttpMethod::POST, "https://api.example.com/posts");
        let response2 = create_test_response();
        pane.create_response_tab(response2, request2, "req-2");

        let tabs = pane.list_tabs();
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].1, "GET api.example.com/users");
        assert_eq!(tabs[1].1, "POST api.example.com/posts");
        assert!(!tabs[0].2); // First tab not active
        assert!(tabs[1].2); // Second tab is active
    }
}
