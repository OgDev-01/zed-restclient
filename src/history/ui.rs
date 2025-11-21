//! UI formatting utilities for history display.
//!
//! This module provides functions for formatting history entries into
//! human-readable strings suitable for display in quick-pick lists,
//! dedicated panes, and other UI components.

use super::models::HistoryEntry;
use chrono::{DateTime, Local, Utc};

/// Formats a list of history entries for display in a quick-pick or list view.
///
/// Each entry is formatted as: "METHOD URL - STATUS (timestamp)"
/// Example: "GET https://api.example.com/users - 200 OK (2025-01-15 14:30:45)"
///
/// # Arguments
///
/// * `entries` - The history entries to format
///
/// # Returns
///
/// A vector of formatted strings, one per entry.
///
/// # Example
///
/// ```ignore
/// use history::{HistoryEntry, format_history_list};
///
/// let formatted = format_history_list(&entries);
/// for line in formatted {
///     println!("{}", line);
/// }
/// ```
pub fn format_history_list(entries: &[HistoryEntry]) -> Vec<String> {
    entries
        .iter()
        .map(|entry| format_history_entry(entry))
        .collect()
}

/// Formats a single history entry for list display.
///
/// Format: "METHOD URL - STATUS (timestamp)"
///
/// # Arguments
///
/// * `entry` - The history entry to format
///
/// # Returns
///
/// A formatted string representation of the entry.
pub fn format_history_entry(entry: &HistoryEntry) -> String {
    let method = entry.request.method.as_str();
    let url = &entry.request.url;
    let status = entry.response.status_code;
    let status_text = &entry.response.status_text;
    let timestamp = format_timestamp(&entry.timestamp);

    format!(
        "{} {} - {} {} ({})",
        method, url, status, status_text, timestamp
    )
}

/// Formats a history entry with detailed information for expanded view.
///
/// Includes method, URL, status, timestamp, request/response headers,
/// and body preview.
///
/// # Arguments
///
/// * `entry` - The history entry to format
///
/// # Returns
///
/// A multi-line formatted string with full entry details.
pub fn format_history_details(entry: &HistoryEntry) -> String {
    let mut output = String::new();

    // Header
    output.push_str("═══════════════════════════════════════════════════════════\n");
    output.push_str(&format!("Request ID: {}\n", entry.id));
    output.push_str(&format!(
        "Timestamp: {}\n",
        format_timestamp_detailed(&entry.timestamp)
    ));
    output.push_str("═══════════════════════════════════════════════════════════\n\n");

    // Request section
    output.push_str("REQUEST\n");
    output.push_str("───────────────────────────────────────────────────────────\n");
    output.push_str(&format!(
        "{} {}\n",
        entry.request.method.as_str(),
        entry.request.url
    ));

    // Request headers
    if !entry.request.headers.is_empty() {
        output.push_str("\nHeaders:\n");
        for (key, value) in &entry.request.headers {
            output.push_str(&format!("  {}: {}\n", key, value));
        }
    }

    // Request body
    if let Some(body) = &entry.request.body {
        if !body.is_empty() {
            output.push_str("\nBody:\n");
            output.push_str(&format_body_preview(body, 500));
        }
    }

    output.push_str("\n");

    // Response section
    output.push_str("RESPONSE\n");
    output.push_str("───────────────────────────────────────────────────────────\n");
    output.push_str(&format!(
        "{} {}\n",
        entry.response.status_code, entry.response.status_text
    ));

    // Response headers
    if !entry.response.headers.is_empty() {
        output.push_str("\nHeaders:\n");
        for (key, value) in &entry.response.headers {
            output.push_str(&format!("  {}: {}\n", key, value));
        }
    }

    // Response body
    if !entry.response.body.is_empty() {
        output.push_str("\nBody:\n");
        if let Ok(body_str) = std::str::from_utf8(&entry.response.body) {
            output.push_str(&format_body_preview(body_str, 1000));
        } else {
            output.push_str("  [Binary data]\n");
        }
    } else {
        output.push_str("\nBody: [Empty or truncated]\n");
    }

    // Tags
    if !entry.tags.is_empty() {
        output.push_str("\nTags: ");
        output.push_str(&entry.tags.join(", "));
        output.push_str("\n");
    }

    output.push_str("\n═══════════════════════════════════════════════════════════\n");

    output
}

/// Formats a compact summary of a history entry for inline display.
///
/// Format: "METHOD URL → STATUS"
///
/// # Arguments
///
/// * `entry` - The history entry to format
///
/// # Returns
///
/// A compact one-line summary.
pub fn format_history_compact(entry: &HistoryEntry) -> String {
    format!(
        "{} {} → {}",
        entry.request.method.as_str(),
        entry.request.url,
        entry.response.status_code
    )
}

/// Formats a timestamp in local time for display.
///
/// Format: "YYYY-MM-DD HH:MM:SS"
///
/// # Arguments
///
/// * `timestamp` - The UTC timestamp to format
///
/// # Returns
///
/// A formatted timestamp string in local time.
pub fn format_timestamp(timestamp: &DateTime<Utc>) -> String {
    let local_time: DateTime<Local> = timestamp.with_timezone(&Local);
    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Formats a timestamp with more detail including timezone.
///
/// Format: "YYYY-MM-DD HH:MM:SS TZ"
///
/// # Arguments
///
/// * `timestamp` - The UTC timestamp to format
///
/// # Returns
///
/// A detailed formatted timestamp string.
pub fn format_timestamp_detailed(timestamp: &DateTime<Utc>) -> String {
    let local_time: DateTime<Local> = timestamp.with_timezone(&Local);
    local_time.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}

/// Formats a relative time description (e.g., "2 hours ago", "yesterday").
///
/// # Arguments
///
/// * `timestamp` - The UTC timestamp to format
///
/// # Returns
///
/// A human-readable relative time string.
pub fn format_relative_time(timestamp: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*timestamp);

    if duration.num_seconds() < 60 {
        return "just now".to_string();
    } else if duration.num_minutes() < 60 {
        let minutes = duration.num_minutes();
        return format!(
            "{} minute{} ago",
            minutes,
            if minutes == 1 { "" } else { "s" }
        );
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        return format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" });
    } else if duration.num_days() < 7 {
        let days = duration.num_days();
        if days == 1 {
            return "yesterday".to_string();
        }
        return format!("{} days ago", days);
    } else if duration.num_weeks() < 4 {
        let weeks = duration.num_weeks();
        return format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" });
    } else if duration.num_days() < 365 {
        let months = duration.num_days() / 30;
        return format!("{} month{} ago", months, if months == 1 { "" } else { "s" });
    } else {
        let years = duration.num_days() / 365;
        return format!("{} year{} ago", years, if years == 1 { "" } else { "s" });
    }
}

/// Formats a history entry with relative time for recent requests.
///
/// Format: "METHOD URL - STATUS (relative time)"
///
/// # Arguments
///
/// * `entry` - The history entry to format
///
/// # Returns
///
/// A formatted string with relative timestamp.
pub fn format_history_entry_relative(entry: &HistoryEntry) -> String {
    let method = entry.request.method.as_str();
    let url = &entry.request.url;
    let status = entry.response.status_code;
    let relative_time = format_relative_time(&entry.timestamp);

    format!("{} {} - {} ({})", method, url, status, relative_time)
}

/// Formats a body preview with optional truncation.
///
/// # Arguments
///
/// * `body` - The body string to format
/// * `max_length` - Maximum length before truncation
///
/// # Returns
///
/// A formatted body string, possibly truncated with "..." indicator.
fn format_body_preview(body: &str, max_length: usize) -> String {
    let trimmed = body.trim();

    if trimmed.len() <= max_length {
        format!("  {}\n", trimmed)
    } else {
        format!(
            "  {}...\n  [Truncated - {} total characters]\n",
            &trimmed[..max_length],
            trimmed.len()
        )
    }
}

/// Creates a summary line for history statistics.
///
/// # Arguments
///
/// * `total` - Total number of entries
/// * `successful` - Number of successful requests (2xx, 3xx)
/// * `errors` - Number of error requests (4xx, 5xx)
///
/// # Returns
///
/// A formatted statistics summary.
pub fn format_history_stats(total: usize, successful: usize, errors: usize) -> String {
    format!(
        "Total: {} | Success: {} ({:.1}%) | Errors: {} ({:.1}%)",
        total,
        successful,
        if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            0.0
        },
        errors,
        if total > 0 {
            (errors as f64 / total as f64) * 100.0
        } else {
            0.0
        }
    )
}

/// Groups history entries by date for organized display.
///
/// # Arguments
///
/// * `entries` - The history entries to group
///
/// # Returns
///
/// A formatted string with entries grouped by date.
pub fn format_history_grouped_by_date(entries: &[HistoryEntry]) -> String {
    use std::collections::BTreeMap;

    let mut grouped: BTreeMap<String, Vec<&HistoryEntry>> = BTreeMap::new();

    for entry in entries {
        let local_time: DateTime<Local> = entry.timestamp.with_timezone(&Local);
        let date_key = local_time.format("%Y-%m-%d").to_string();
        grouped.entry(date_key).or_insert_with(Vec::new).push(entry);
    }

    let mut output = String::new();

    for (date, date_entries) in grouped.iter().rev() {
        output.push_str(&format!("\n{}\n", date));
        output.push_str(&"─".repeat(60));
        output.push_str("\n");

        for entry in date_entries {
            output.push_str(&format!("  {}\n", format_history_entry_relative(entry)));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HttpMethod, HttpRequest, HttpResponse};
    use chrono::{Duration, Utc};

    fn create_test_entry(method: HttpMethod, url: &str, status: u16) -> HistoryEntry {
        let request = HttpRequest::new("test-id".to_string(), method, url.to_string());
        let response = HttpResponse::new(status, "OK".to_string());
        HistoryEntry::new(request, response)
    }

    #[test]
    fn test_format_history_entry() {
        let entry = create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200);
        let formatted = format_history_entry(&entry);

        assert!(formatted.contains("GET"));
        assert!(formatted.contains("https://api.example.com/users"));
        assert!(formatted.contains("200"));
        assert!(formatted.contains("OK"));
    }

    #[test]
    fn test_format_history_list() {
        let entries = vec![
            create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200),
            create_test_entry(HttpMethod::POST, "https://api.example.com/posts", 201),
        ];

        let formatted = format_history_list(&entries);
        assert_eq!(formatted.len(), 2);
        assert!(formatted[0].contains("GET"));
        assert!(formatted[1].contains("POST"));
    }

    #[test]
    fn test_format_history_compact() {
        let entry = create_test_entry(HttpMethod::DELETE, "https://api.example.com/users/1", 204);
        let formatted = format_history_compact(&entry);

        assert_eq!(formatted, "DELETE https://api.example.com/users/1 → 204");
    }

    #[test]
    fn test_format_history_details() {
        let mut entry = create_test_entry(HttpMethod::POST, "https://api.example.com/users", 201);
        entry
            .request
            .add_header("Content-Type".to_string(), "application/json".to_string());
        entry.request.set_body("{\"name\": \"John\"}".to_string());
        entry.response.set_body(b"{\"id\": 1}".to_vec());

        let formatted = format_history_details(&entry);

        assert!(formatted.contains("REQUEST"));
        assert!(formatted.contains("RESPONSE"));
        assert!(formatted.contains("POST"));
        assert!(formatted.contains("201"));
        assert!(formatted.contains("Content-Type"));
    }

    #[test]
    fn test_format_relative_time_just_now() {
        let timestamp = Utc::now();
        let formatted = format_relative_time(&timestamp);
        assert_eq!(formatted, "just now");
    }

    #[test]
    fn test_format_relative_time_minutes() {
        let timestamp = Utc::now() - Duration::minutes(5);
        let formatted = format_relative_time(&timestamp);
        assert!(formatted.contains("5 minutes ago"));
    }

    #[test]
    fn test_format_relative_time_hours() {
        let timestamp = Utc::now() - Duration::hours(3);
        let formatted = format_relative_time(&timestamp);
        assert!(formatted.contains("3 hours ago"));
    }

    #[test]
    fn test_format_relative_time_yesterday() {
        let timestamp = Utc::now() - Duration::days(1);
        let formatted = format_relative_time(&timestamp);
        assert_eq!(formatted, "yesterday");
    }

    #[test]
    fn test_format_relative_time_days() {
        let timestamp = Utc::now() - Duration::days(3);
        let formatted = format_relative_time(&timestamp);
        assert!(formatted.contains("3 days ago"));
    }

    #[test]
    fn test_format_history_entry_relative() {
        let entry = create_test_entry(HttpMethod::GET, "https://api.example.com/users", 200);
        let formatted = format_history_entry_relative(&entry);

        assert!(formatted.contains("GET"));
        assert!(formatted.contains("https://api.example.com/users"));
        assert!(formatted.contains("200"));
        assert!(formatted.contains("just now"));
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = Utc::now();
        let formatted = format_timestamp(&timestamp);

        // Should contain date and time components
        assert!(formatted.contains("-"));
        assert!(formatted.contains(":"));
    }

    #[test]
    fn test_format_history_stats() {
        let stats = format_history_stats(100, 85, 15);
        assert!(stats.contains("Total: 100"));
        assert!(stats.contains("Success: 85"));
        assert!(stats.contains("Errors: 15"));
        assert!(stats.contains("85.0%"));
        assert!(stats.contains("15.0%"));
    }

    #[test]
    fn test_format_history_stats_zero() {
        let stats = format_history_stats(0, 0, 0);
        assert!(stats.contains("Total: 0"));
        assert!(stats.contains("0.0%"));
    }

    #[test]
    fn test_format_body_preview_short() {
        let body = "Short body";
        let formatted = format_body_preview(body, 100);
        assert!(formatted.contains("Short body"));
        assert!(!formatted.contains("Truncated"));
    }

    #[test]
    fn test_format_body_preview_long() {
        let body = "a".repeat(1000);
        let formatted = format_body_preview(&body, 100);
        assert!(formatted.contains("Truncated"));
        assert!(formatted.contains("1000 total characters"));
    }

    #[test]
    fn test_format_history_grouped_by_date() {
        let mut entry1 = create_test_entry(HttpMethod::GET, "https://api.example.com/1", 200);
        let mut entry2 = create_test_entry(HttpMethod::GET, "https://api.example.com/2", 200);

        entry1.timestamp = Utc::now() - Duration::days(1);
        entry2.timestamp = Utc::now();

        let entries = vec![entry1, entry2];
        let formatted = format_history_grouped_by_date(&entries);

        // Should contain date separators
        assert!(formatted.contains("─"));
    }
}
