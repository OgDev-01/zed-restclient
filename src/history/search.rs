//! Search functionality for request history.
//!
//! This module provides search and filtering capabilities for history entries,
//! allowing users to quickly find past requests by URL, method, or content.

use super::models::HistoryEntry;

/// Searches history entries using case-insensitive substring matching.
///
/// Searches across multiple fields:
/// - URL
/// - HTTP method
/// - Request body (if present)
/// - Response body (if present and not truncated)
///
/// # Arguments
///
/// * `query` - The search term to match against
/// * `entries` - The history entries to search through
///
/// # Returns
///
/// A vector of history entries that match the query, in their original order.
///
/// # Example
///
/// ```ignore
/// use history::{HistoryEntry, search_history};
///
/// let results = search_history("api/users", &entries);
/// ```
pub fn search_history(query: &str, entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    if query.is_empty() {
        return entries.to_vec();
    }

    let query_lower = query.to_lowercase();

    entries
        .iter()
        .filter(|entry| matches_query(entry, &query_lower))
        .cloned()
        .collect()
}

/// Checks if a history entry matches the given search query.
///
/// # Arguments
///
/// * `entry` - The history entry to check
/// * `query_lower` - The lowercase search query
///
/// # Returns
///
/// `true` if the entry matches the query in any field.
fn matches_query(entry: &HistoryEntry, query_lower: &str) -> bool {
    // Search in URL
    if entry.request.url.to_lowercase().contains(query_lower) {
        return true;
    }

    // Search in HTTP method
    if entry
        .request
        .method
        .as_str()
        .to_lowercase()
        .contains(query_lower)
    {
        return true;
    }

    // Search in request body
    if let Some(body) = &entry.request.body {
        if body.to_lowercase().contains(query_lower) {
            return true;
        }
    }

    // Search in response body (if not truncated)
    if !entry.response.body.is_empty() {
        if let Ok(body_str) = std::str::from_utf8(&entry.response.body) {
            if body_str.to_lowercase().contains(query_lower) {
                return true;
            }
        }
    }

    // Search in tags
    if entry
        .tags
        .iter()
        .any(|tag| tag.to_lowercase().contains(query_lower))
    {
        return true;
    }

    false
}

/// Filters history entries by HTTP method.
///
/// # Arguments
///
/// * `method` - The HTTP method to filter by (case-insensitive)
/// * `entries` - The history entries to filter
///
/// # Returns
///
/// A vector of history entries with the specified method.
pub fn filter_by_method(method: &str, entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    let method_upper = method.to_uppercase();
    entries
        .iter()
        .filter(|entry| entry.request.method.as_str() == method_upper)
        .cloned()
        .collect()
}

/// Filters history entries by status code.
///
/// # Arguments
///
/// * `status_code` - The HTTP status code to filter by
/// * `entries` - The history entries to filter
///
/// # Returns
///
/// A vector of history entries with the specified status code.
pub fn filter_by_status(status_code: u16, entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    entries
        .iter()
        .filter(|entry| entry.response.status_code == status_code)
        .cloned()
        .collect()
}

/// Filters history entries by tag.
///
/// # Arguments
///
/// * `tag` - The tag to filter by (case-sensitive)
/// * `entries` - The history entries to filter
///
/// # Returns
///
/// A vector of history entries that have the specified tag.
pub fn filter_by_tag(tag: &str, entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    entries
        .iter()
        .filter(|entry| entry.has_tag(tag))
        .cloned()
        .collect()
}

/// Filters history entries by success status (2xx and 3xx).
///
/// # Arguments
///
/// * `entries` - The history entries to filter
///
/// # Returns
///
/// A vector of history entries with successful responses.
pub fn filter_successful(entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    entries
        .iter()
        .filter(|entry| entry.should_save())
        .cloned()
        .collect()
}

/// Filters history entries by error status (4xx and 5xx).
///
/// # Arguments
///
/// * `entries` - The history entries to filter
///
/// # Returns
///
/// A vector of history entries with error responses.
pub fn filter_errors(entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    entries
        .iter()
        .filter(|entry| !entry.should_save())
        .cloned()
        .collect()
}

/// Sorts history entries by timestamp in descending order (newest first).
///
/// # Arguments
///
/// * `entries` - The history entries to sort
///
/// # Returns
///
/// A vector of history entries sorted by timestamp (newest first).
pub fn sort_by_timestamp_desc(entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sorted
}

/// Sorts history entries by timestamp in ascending order (oldest first).
///
/// # Arguments
///
/// * `entries` - The history entries to sort
///
/// # Returns
///
/// A vector of history entries sorted by timestamp (oldest first).
pub fn sort_by_timestamp_asc(entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    sorted
}

/// Returns the most recent N history entries.
///
/// # Arguments
///
/// * `count` - Maximum number of entries to return
/// * `entries` - The history entries to filter
///
/// # Returns
///
/// A vector of the most recent history entries, up to `count` items.
pub fn get_recent_entries(count: usize, entries: &[HistoryEntry]) -> Vec<HistoryEntry> {
    let sorted = sort_by_timestamp_desc(entries);
    sorted.into_iter().take(count).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HttpMethod, HttpRequest, HttpResponse};

    fn create_test_entry(method: HttpMethod, url: &str, status: u16, body: &str) -> HistoryEntry {
        let mut request = HttpRequest::new("test-id".to_string(), method, url.to_string());
        if !body.is_empty() {
            request.set_body(body.to_string());
        }

        let mut response = HttpResponse::new(status, "OK".to_string());
        response.set_body(b"{\"result\": \"success\"}".to_vec());

        HistoryEntry::new(request, response)
    }

    #[test]
    fn test_search_history_by_url() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, ""),
            create_test_entry(HttpMethod::GET, "https://other.com/data", 200, ""),
        ];

        let results = search_history("example.com", &entries);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_history_case_insensitive() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/Users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.test.com/users", 201, ""),
        ];

        let results = search_history("USERS", &entries);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_history_by_method() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, ""),
            create_test_entry(
                HttpMethod::DELETE,
                "https://api.example.com/users/1",
                204,
                "",
            ),
        ];

        let results = search_history("post", &entries);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].request.method, HttpMethod::POST);
    }

    #[test]
    fn test_search_history_by_body() {
        let entries = vec![
            create_test_entry(
                HttpMethod::POST,
                "https://api.example.com/users",
                201,
                r#"{"name": "John Doe"}"#,
            ),
            create_test_entry(
                HttpMethod::POST,
                "https://api.example.com/posts",
                201,
                r#"{"title": "Test Post"}"#,
            ),
        ];

        let results = search_history("John", &entries);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_history_empty_query() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, ""),
        ];

        let results = search_history("", &entries);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_history_no_matches() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, ""),
        ];

        let results = search_history("nonexistent", &entries);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_filter_by_method() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, ""),
            create_test_entry(HttpMethod::GET, "https://api.example.com/data", 200, ""),
        ];

        let results = filter_by_method("GET", &entries);
        assert_eq!(results.len(), 2);

        let results_post = filter_by_method("post", &entries);
        assert_eq!(results_post.len(), 1);
    }

    #[test]
    fn test_filter_by_status() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, ""),
            create_test_entry(HttpMethod::GET, "https://api.example.com/notfound", 404, ""),
        ];

        let results = filter_by_status(200, &entries);
        assert_eq!(results.len(), 1);

        let results_404 = filter_by_status(404, &entries);
        assert_eq!(results_404.len(), 1);
    }

    #[test]
    fn test_filter_by_tag() {
        let mut entry1 =
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, "");
        entry1.add_tag("api".to_string());
        entry1.add_tag("users".to_string());

        let mut entry2 =
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, "");
        entry2.add_tag("api".to_string());

        let entry3 = create_test_entry(HttpMethod::GET, "https://other.com/data", 200, "");

        let entries = vec![entry1, entry2, entry3];

        let results = filter_by_tag("api", &entries);
        assert_eq!(results.len(), 2);

        let results_users = filter_by_tag("users", &entries);
        assert_eq!(results_users.len(), 1);
    }

    #[test]
    fn test_filter_successful() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201, ""),
            create_test_entry(HttpMethod::GET, "https://api.example.com/redirect", 301, ""),
            create_test_entry(HttpMethod::GET, "https://api.example.com/notfound", 404, ""),
        ];

        let results = filter_successful(&entries);
        assert_eq!(results.len(), 3); // 200, 201, 301
    }

    #[test]
    fn test_filter_errors() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200, ""),
            create_test_entry(HttpMethod::GET, "https://api.example.com/notfound", 404, ""),
            create_test_entry(HttpMethod::GET, "https://api.example.com/error", 500, ""),
        ];

        let results = filter_errors(&entries);
        assert_eq!(results.len(), 2); // 404, 500
    }

    #[test]
    fn test_sort_by_timestamp_desc() {
        use chrono::{Duration, Utc};

        let mut entry1 = create_test_entry(HttpMethod::GET, "https://api.example.com/1", 200, "");
        let mut entry2 = create_test_entry(HttpMethod::GET, "https://api.example.com/2", 200, "");
        let mut entry3 = create_test_entry(HttpMethod::GET, "https://api.example.com/3", 200, "");

        // Set timestamps manually for testing
        entry1.timestamp = Utc::now() - Duration::hours(2);
        entry2.timestamp = Utc::now() - Duration::hours(1);
        entry3.timestamp = Utc::now();

        let entries = vec![entry1.clone(), entry2.clone(), entry3.clone()];
        let sorted = sort_by_timestamp_desc(&entries);

        assert_eq!(sorted[0].request.url, entry3.request.url); // Newest
        assert_eq!(sorted[1].request.url, entry2.request.url);
        assert_eq!(sorted[2].request.url, entry1.request.url); // Oldest
    }

    #[test]
    fn test_sort_by_timestamp_asc() {
        use chrono::{Duration, Utc};

        let mut entry1 = create_test_entry(HttpMethod::GET, "https://api.example.com/1", 200, "");
        let mut entry2 = create_test_entry(HttpMethod::GET, "https://api.example.com/2", 200, "");
        let mut entry3 = create_test_entry(HttpMethod::GET, "https://api.example.com/3", 200, "");

        entry1.timestamp = Utc::now() - Duration::hours(2);
        entry2.timestamp = Utc::now() - Duration::hours(1);
        entry3.timestamp = Utc::now();

        let entries = vec![entry3.clone(), entry1.clone(), entry2.clone()];
        let sorted = sort_by_timestamp_asc(&entries);

        assert_eq!(sorted[0].request.url, entry1.request.url); // Oldest
        assert_eq!(sorted[1].request.url, entry2.request.url);
        assert_eq!(sorted[2].request.url, entry3.request.url); // Newest
    }

    #[test]
    fn test_get_recent_entries() {
        use chrono::{Duration, Utc};

        let mut entry1 = create_test_entry(HttpMethod::GET, "https://api.example.com/1", 200, "");
        let mut entry2 = create_test_entry(HttpMethod::GET, "https://api.example.com/2", 200, "");
        let mut entry3 = create_test_entry(HttpMethod::GET, "https://api.example.com/3", 200, "");
        let mut entry4 = create_test_entry(HttpMethod::GET, "https://api.example.com/4", 200, "");

        entry1.timestamp = Utc::now() - Duration::hours(3);
        entry2.timestamp = Utc::now() - Duration::hours(2);
        entry3.timestamp = Utc::now() - Duration::hours(1);
        entry4.timestamp = Utc::now();

        let entries = vec![entry1, entry2, entry3.clone(), entry4.clone()];

        let recent = get_recent_entries(2, &entries);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].request.url, entry4.request.url);
        assert_eq!(recent[1].request.url, entry3.request.url);
    }

    #[test]
    fn test_get_recent_entries_more_than_available() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/1", 200, ""),
            create_test_entry(HttpMethod::GET, "https://api.example.com/2", 200, ""),
        ];

        let recent = get_recent_entries(10, &entries);
        assert_eq!(recent.len(), 2);
    }
}
