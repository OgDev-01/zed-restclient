//! Data models for request history.
//!
//! This module defines the core data structures for storing and managing
//! HTTP request/response history.

use crate::models::{HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum response body size to store in history (1MB).
///
/// Responses larger than this threshold will have their body excluded
/// to prevent excessive storage usage.
pub const MAX_RESPONSE_BODY_SIZE: usize = 1_048_576; // 1MB

/// Sensitive header names that should be sanitized before storage.
///
/// These headers contain authentication tokens, cookies, and other
/// sensitive information that should not be persisted by default.
pub const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "api-key",
    "auth-token",
    "x-auth-token",
    "access-token",
    "x-access-token",
    "bearer",
    "proxy-authorization",
];

/// A single entry in the request history.
///
/// Represents a complete request/response pair with metadata for
/// searching, filtering, and organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique identifier for this history entry.
    ///
    /// Generated using UUID v4 for guaranteed uniqueness.
    pub id: String,

    /// Timestamp when this request was executed.
    ///
    /// Stored in UTC for consistency across time zones.
    pub timestamp: DateTime<Utc>,

    /// The HTTP request that was sent.
    ///
    /// Contains method, URL, headers, and body. Sensitive headers
    /// may be sanitized based on configuration.
    pub request: HttpRequest,

    /// The HTTP response that was received.
    ///
    /// Contains status, headers, and body. Large response bodies
    /// (>1MB) are excluded to save storage space.
    pub response: HttpResponse,

    /// User-defined tags for organizing and filtering history.
    ///
    /// Tags can be used to categorize requests by project, environment,
    /// API endpoint type, etc.
    pub tags: Vec<String>,
}

impl HistoryEntry {
    /// Creates a new history entry from a request and response.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request that was sent
    /// * `response` - The HTTP response that was received
    ///
    /// # Returns
    ///
    /// A new `HistoryEntry` with a unique ID and current timestamp.
    pub fn new(request: HttpRequest, response: HttpResponse) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request,
            response,
            tags: Vec::new(),
        }
    }

    /// Creates a new history entry with custom tags.
    ///
    /// # Arguments
    ///
    /// * `request` - The HTTP request that was sent
    /// * `response` - The HTTP response that was received
    /// * `tags` - Tags for organizing this entry
    ///
    /// # Returns
    ///
    /// A new `HistoryEntry` with the specified tags.
    pub fn with_tags(request: HttpRequest, response: HttpResponse, tags: Vec<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request,
            response,
            tags,
        }
    }

    /// Checks if this entry should be included in history based on status code.
    ///
    /// By default, only successful requests (2xx, 3xx) are saved unless
    /// configured otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the response status indicates success (2xx or 3xx).
    pub fn should_save(&self) -> bool {
        self.response.is_success() || self.response.is_redirect()
    }

    /// Checks if the response body exceeds the storage limit.
    ///
    /// # Returns
    ///
    /// `true` if the response body is larger than 1MB.
    pub fn has_large_response(&self) -> bool {
        self.response.body.len() > MAX_RESPONSE_BODY_SIZE
    }

    /// Sanitizes sensitive headers from the request.
    ///
    /// Removes or redacts headers containing authentication tokens,
    /// cookies, and other sensitive data.
    ///
    /// # Arguments
    ///
    /// * `sanitize` - Whether to sanitize sensitive headers
    ///
    /// # Returns
    ///
    /// A new `HistoryEntry` with sanitized headers if enabled.
    pub fn sanitize_headers(&self, sanitize: bool) -> Self {
        if !sanitize {
            return self.clone();
        }

        let mut sanitized_request = self.request.clone();

        // Remove sensitive headers from request
        sanitized_request.headers.retain(|key, _| {
            !SENSITIVE_HEADERS
                .iter()
                .any(|sensitive| key.eq_ignore_ascii_case(sensitive))
        });

        let mut sanitized_response = self.response.clone();

        // Remove sensitive headers from response
        sanitized_response.headers.retain(|key, _| {
            !SENSITIVE_HEADERS
                .iter()
                .any(|sensitive| key.eq_ignore_ascii_case(sensitive))
        });

        Self {
            id: self.id.clone(),
            timestamp: self.timestamp,
            request: sanitized_request,
            response: sanitized_response,
            tags: self.tags.clone(),
        }
    }

    /// Removes the response body if it exceeds the size limit.
    ///
    /// # Returns
    ///
    /// A new `HistoryEntry` with the response body removed if too large.
    pub fn truncate_large_response(&self) -> Self {
        if !self.has_large_response() {
            return self.clone();
        }

        let mut truncated_response = self.response.clone();
        truncated_response.body = Vec::new();

        Self {
            id: self.id.clone(),
            timestamp: self.timestamp,
            request: self.request.clone(),
            response: truncated_response,
            tags: self.tags.clone(),
        }
    }

    /// Prepares the entry for storage by sanitizing and truncating as needed.
    ///
    /// # Arguments
    ///
    /// * `sanitize_sensitive` - Whether to remove sensitive headers
    ///
    /// # Returns
    ///
    /// A new `HistoryEntry` ready for safe storage.
    pub fn prepare_for_storage(&self, sanitize_sensitive: bool) -> Self {
        self.sanitize_headers(sanitize_sensitive)
            .truncate_large_response()
    }

    /// Adds a tag to this history entry.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to add
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Removes a tag from this history entry.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to remove
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    /// Checks if this entry has a specific tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to check for
    ///
    /// # Returns
    ///
    /// `true` if the entry has the specified tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

/// Errors that can occur during history operations.
#[derive(Debug)]
pub enum HistoryError {
    /// Error occurred during storage operations (file I/O).
    ///
    /// Contains the underlying I/O error for detailed diagnostics.
    StorageError(std::io::Error),

    /// Error occurred during serialization or deserialization.
    ///
    /// Contains the underlying serde_json error.
    SerializationError(serde_json::Error),

    /// History quota has been exceeded.
    ///
    /// Includes the current count and maximum allowed entries.
    QuotaExceeded {
        /// Current number of entries
        current: usize,
        /// Maximum allowed entries
        max: usize,
    },
}

impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HistoryError::StorageError(err) => {
                write!(f, "History storage error: {}", err)
            }
            HistoryError::SerializationError(err) => {
                write!(f, "History serialization error: {}", err)
            }
            HistoryError::QuotaExceeded { current, max } => {
                write!(
                    f,
                    "History quota exceeded: {} entries (max: {})",
                    current, max
                )
            }
        }
    }
}

impl std::error::Error for HistoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HistoryError::StorageError(err) => Some(err),
            HistoryError::SerializationError(err) => Some(err),
            HistoryError::QuotaExceeded { .. } => None,
        }
    }
}

impl From<std::io::Error> for HistoryError {
    fn from(err: std::io::Error) -> Self {
        HistoryError::StorageError(err)
    }
}

impl From<serde_json::Error> for HistoryError {
    fn from(err: serde_json::Error) -> Self {
        HistoryError::SerializationError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HttpMethod, HttpRequest, HttpResponse};

    fn create_test_request() -> HttpRequest {
        let mut request = HttpRequest::new(
            "test-id".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );
        request.add_header(
            "Authorization".to_string(),
            "Bearer secret-token".to_string(),
        );
        request.add_header("Content-Type".to_string(), "application/json".to_string());
        request
    }

    fn create_test_response(status_code: u16) -> HttpResponse {
        let mut response = HttpResponse::new(status_code, "OK".to_string());
        response.add_header("Content-Type".to_string(), "application/json".to_string());
        response.add_header("Set-Cookie".to_string(), "session=abc123".to_string());
        response.set_body(b"{\"id\": 1, \"name\": \"Test\"}".to_vec());
        response
    }

    #[test]
    fn test_history_entry_new() {
        let request = create_test_request();
        let response = create_test_response(200);
        let entry = HistoryEntry::new(request.clone(), response.clone());

        assert!(!entry.id.is_empty());
        assert_eq!(entry.request.id, request.id);
        assert_eq!(entry.response.status_code, response.status_code);
        assert!(entry.tags.is_empty());
    }

    #[test]
    fn test_history_entry_with_tags() {
        let request = create_test_request();
        let response = create_test_response(200);
        let tags = vec!["api".to_string(), "users".to_string()];
        let entry = HistoryEntry::with_tags(request, response, tags.clone());

        assert_eq!(entry.tags, tags);
    }

    #[test]
    fn test_should_save() {
        let request = create_test_request();

        // 2xx should save
        let entry_200 = HistoryEntry::new(request.clone(), create_test_response(200));
        assert!(entry_200.should_save());

        // 3xx should save
        let entry_301 = HistoryEntry::new(request.clone(), create_test_response(301));
        assert!(entry_301.should_save());

        // 4xx should not save by default
        let entry_404 = HistoryEntry::new(request.clone(), create_test_response(404));
        assert!(!entry_404.should_save());

        // 5xx should not save by default
        let entry_500 = HistoryEntry::new(request.clone(), create_test_response(500));
        assert!(!entry_500.should_save());
    }

    #[test]
    fn test_has_large_response() {
        let request = create_test_request();
        let mut response = create_test_response(200);

        // Small response
        assert!(!HistoryEntry::new(request.clone(), response.clone()).has_large_response());

        // Large response (>1MB)
        response.set_body(vec![0u8; MAX_RESPONSE_BODY_SIZE + 1]);
        assert!(HistoryEntry::new(request, response).has_large_response());
    }

    #[test]
    fn test_sanitize_headers() {
        let request = create_test_request();
        let response = create_test_response(200);
        let entry = HistoryEntry::new(request, response);

        // Without sanitization
        let unsanitized = entry.sanitize_headers(false);
        assert!(unsanitized.request.headers.contains_key("Authorization"));
        assert!(unsanitized.response.headers.contains_key("Set-Cookie"));

        // With sanitization
        let sanitized = entry.sanitize_headers(true);
        assert!(!sanitized.request.headers.contains_key("Authorization"));
        assert!(!sanitized.response.headers.contains_key("Set-Cookie"));
        assert!(sanitized.request.headers.contains_key("Content-Type"));
        assert!(sanitized.response.headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_truncate_large_response() {
        let request = create_test_request();
        let mut response = create_test_response(200);

        // Small response should not be truncated
        let entry_small = HistoryEntry::new(request.clone(), response.clone());
        let truncated_small = entry_small.truncate_large_response();
        assert!(!truncated_small.response.body.is_empty());

        // Large response should be truncated
        response.set_body(vec![0u8; MAX_RESPONSE_BODY_SIZE + 1]);
        let entry_large = HistoryEntry::new(request, response);
        let truncated_large = entry_large.truncate_large_response();
        assert!(truncated_large.response.body.is_empty());
    }

    #[test]
    fn test_prepare_for_storage() {
        let request = create_test_request();
        let mut response = create_test_response(200);
        response.set_body(vec![0u8; MAX_RESPONSE_BODY_SIZE + 1]);

        let entry = HistoryEntry::new(request, response);
        let prepared = entry.prepare_for_storage(true);

        // Should sanitize and truncate
        assert!(!prepared.request.headers.contains_key("Authorization"));
        assert!(prepared.response.body.is_empty());
    }

    #[test]
    fn test_tag_operations() {
        let request = create_test_request();
        let response = create_test_response(200);
        let mut entry = HistoryEntry::new(request, response);

        // Add tags
        entry.add_tag("api".to_string());
        entry.add_tag("users".to_string());
        assert_eq!(entry.tags.len(), 2);
        assert!(entry.has_tag("api"));
        assert!(entry.has_tag("users"));

        // Duplicate tags should not be added
        entry.add_tag("api".to_string());
        assert_eq!(entry.tags.len(), 2);

        // Remove tag
        entry.remove_tag("api");
        assert_eq!(entry.tags.len(), 1);
        assert!(!entry.has_tag("api"));
        assert!(entry.has_tag("users"));
    }

    #[test]
    fn test_history_error_display() {
        let io_error = HistoryError::StorageError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(format!("{}", io_error).contains("storage error"));

        let quota_error = HistoryError::QuotaExceeded {
            current: 1500,
            max: 1000,
        };
        assert!(format!("{}", quota_error).contains("1500"));
        assert!(format!("{}", quota_error).contains("1000"));
    }

    #[test]
    fn test_serialization() {
        let request = create_test_request();
        let response = create_test_response(200);
        let entry = HistoryEntry::new(request, response);

        // Test serialization
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("request"));
        assert!(json.contains("response"));

        // Test deserialization
        let deserialized: HistoryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, entry.id);
        assert_eq!(deserialized.request.url, entry.request.url);
        assert_eq!(
            deserialized.response.status_code,
            entry.response.status_code
        );
    }
}
