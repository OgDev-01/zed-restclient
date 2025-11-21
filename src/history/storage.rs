//! Persistent storage for request history.
//!
//! This module handles reading and writing history entries to disk using
//! JSONL (JSON Lines) format for efficient append operations and easy recovery
//! from corruption.
//!
//! Performance optimizations:
//! - Lazy loading: Load history entries on demand rather than all at once
//! - Pagination: Support loading history in chunks to reduce memory usage
//! - Efficient parsing: Pre-allocate vectors based on estimated entry count

use super::models::{HistoryEntry, HistoryError};
use crate::config::get_config;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Default maximum number of history entries to retain.
/// This is used as a fallback if global config is unavailable.
pub const DEFAULT_MAX_HISTORY_ENTRIES: usize = 1000;

/// Default page size for lazy loading history entries.
pub const DEFAULT_PAGE_SIZE: usize = 100;

/// Configuration for history storage.
#[derive(Debug, Clone)]
pub struct HistoryConfig {
    /// Maximum number of entries to keep in history.
    pub max_entries: usize,

    /// Whether to sanitize sensitive headers before storage.
    pub sanitize_sensitive_headers: bool,

    /// Whether to save failed requests (4xx, 5xx status codes).
    pub save_failed_requests: bool,
}

impl Default for HistoryConfig {
    /// Creates a default HistoryConfig using global configuration.
    ///
    /// Reads `historyLimit` from the global RestClientConfig settings.
    fn default() -> Self {
        let global_config = get_config();
        Self {
            max_entries: global_config.history_limit,
            sanitize_sensitive_headers: true,
            save_failed_requests: false,
        }
    }
}

impl HistoryConfig {
    /// Creates a HistoryConfig from the global REST Client configuration.
    ///
    /// # Returns
    ///
    /// A new `HistoryConfig` instance with settings from global config.
    pub fn from_global_config() -> Self {
        let global_config = get_config();
        Self {
            max_entries: global_config.history_limit,
            sanitize_sensitive_headers: true,
            save_failed_requests: false,
        }
    }
}

/// Gets the default history file path.
///
/// Returns `~/.config/zed/extensions/rest-client/history.json` on Unix-like systems,
/// or the equivalent on Windows.
///
/// # Returns
///
/// The path to the history file, creating parent directories if needed.
///
/// # Errors
///
/// Returns `HistoryError::StorageError` if the config directory cannot be created.
pub fn get_history_file_path() -> Result<PathBuf, HistoryError> {
    get_history_file_path_internal(None)
}

fn get_history_file_path_internal(override_path: Option<PathBuf>) -> Result<PathBuf, HistoryError> {
    if let Some(path) = override_path {
        return Ok(path);
    }
    // Try to get the config directory
    let config_dir = if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".config")
    } else if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        PathBuf::from(user_profile).join("AppData").join("Roaming")
    } else {
        return Err(HistoryError::StorageError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine home directory",
        )));
    };

    let history_dir = config_dir
        .join("zed")
        .join("extensions")
        .join("rest-client");

    // Create directory if it doesn't exist
    if !history_dir.exists() {
        fs::create_dir_all(&history_dir)?;
    }

    Ok(history_dir.join("history.json"))
}

/// Saves a history entry to the history file.
///
/// Appends the entry as a single JSON line to the history file. If the file
/// doesn't exist, it will be created. The entry is prepared for storage by
/// sanitizing sensitive headers and truncating large response bodies based
/// on the configuration.
///
/// # Arguments
///
/// * `entry` - The history entry to save
///
/// # Returns
///
/// `Ok(())` if the entry was saved successfully.
///
/// # Errors
///
/// Returns `HistoryError` if:
/// - The history file cannot be opened or created
/// - The entry cannot be serialized to JSON
/// - Writing to the file fails
///
/// # Example
///
/// ```ignore
/// let entry = HistoryEntry::new(request, response);
/// save_entry(&entry)?;
/// ```
pub fn save_entry(entry: &HistoryEntry) -> Result<(), HistoryError> {
    save_entry_with_config(entry, &HistoryConfig::default())
}

fn save_entry_internal(
    entry: &HistoryEntry,
    config: &HistoryConfig,
    history_path: Option<PathBuf>,
) -> Result<(), HistoryError> {
    // Check if we should save this entry based on status code
    if !config.save_failed_requests && !entry.should_save() {
        return Ok(());
    }

    let history_path = get_history_file_path_internal(history_path)?;

    // Prepare entry for storage (sanitize and truncate as needed)
    let prepared_entry = entry.prepare_for_storage(config.sanitize_sensitive_headers);

    // Open file in append mode, create if it doesn't exist
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)?;

    // Serialize entry to JSON (single line)
    let json = serde_json::to_string(&prepared_entry)?;

    // Write JSON line to file
    writeln!(file, "{}", json)?;

    // Flush to ensure data is written
    file.flush()?;

    Ok(())
}

/// Saves a history entry with custom configuration.
///
/// # Arguments
///
/// * `entry` - The history entry to save
/// * `config` - Storage configuration
///
/// # Returns
///
/// `Ok(())` if the entry was saved successfully.
///
/// # Errors
///
/// Returns `HistoryError` if the entry cannot be saved.
pub fn save_entry_with_config(
    entry: &HistoryEntry,
    config: &HistoryConfig,
) -> Result<(), HistoryError> {
    save_entry_internal(entry, config, None)
}

/// Loads all history entries from the history file.
///
/// Reads the JSONL file and deserializes each line into a `HistoryEntry`.
/// If the file is corrupted or contains invalid JSON, those lines are skipped
/// and the function continues reading valid entries.
///
/// # Returns
///
/// A vector of all valid history entries, sorted by timestamp (oldest first).
///
/// # Errors
///
/// Returns `HistoryError::StorageError` if the history file cannot be read.
/// Individual line parsing errors are logged but don't cause the function to fail.
///
/// # Example
///
/// ```ignore
/// let entries = load_history()?;
/// for entry in entries {
///     println!("{}: {} {}", entry.timestamp, entry.request.method, entry.request.url);
/// }
/// ```
pub fn load_history() -> Result<Vec<HistoryEntry>, HistoryError> {
    load_history_internal(None)
}

fn load_history_internal(history_path: Option<PathBuf>) -> Result<Vec<HistoryEntry>, HistoryError> {
    let history_path = get_history_file_path_internal(history_path)?;

    // If file doesn't exist, return empty vector
    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&history_path)?;

    // Estimate capacity based on file size (performance optimization)
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let estimated_entries = (file_size / 200).min(DEFAULT_MAX_HISTORY_ENTRIES as u64) as usize;
    let mut entries = Vec::with_capacity(estimated_entries);

    let reader = BufReader::new(file);
    let mut corrupted_lines = 0;

    for (line_num, line_result) in reader.lines().enumerate() {
        match line_result {
            Ok(line) => {
                // Skip empty lines
                if line.trim().is_empty() {
                    continue;
                }

                // Try to parse the JSON line
                match serde_json::from_str::<HistoryEntry>(&line) {
                    Ok(entry) => entries.push(entry),
                    Err(e) => {
                        corrupted_lines += 1;
                        eprintln!(
                            "Warning: Skipping corrupted history entry at line {}: {}",
                            line_num + 1,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                corrupted_lines += 1;
                eprintln!(
                    "Warning: Error reading history line {}: {}",
                    line_num + 1,
                    e
                );
            }
        }
    }

    // If more than 50% of lines are corrupted, warn the user
    if corrupted_lines > 0 && corrupted_lines > entries.len() {
        eprintln!(
            "Warning: History file has significant corruption ({} corrupted lines, {} valid entries)",
            corrupted_lines,
            entries.len()
        );
    }

    Ok(entries)
}

/// Loads history entries with pagination for lazy loading.
///
/// This is more efficient than loading all entries at once, especially for
/// large history files. It loads only the requested page of entries.
///
/// # Arguments
///
/// * `page` - The page number (0-indexed)
/// * `page_size` - Number of entries per page
///
/// # Returns
///
/// A tuple of (entries, total_count) where entries are the requested page
/// and total_count is the total number of entries in the history file.
///
/// # Errors
///
/// Returns `HistoryError::StorageError` if the history file cannot be read.
///
/// # Example
///
/// ```ignore
/// // Load first page of 100 entries
/// let (entries, total) = load_history_paginated(0, 100)?;
/// println!("Showing {} of {} total entries", entries.len(), total);
/// ```
pub fn load_history_paginated(
    page: usize,
    page_size: usize,
) -> Result<(Vec<HistoryEntry>, usize), HistoryError> {
    load_history_paginated_internal(page, page_size, None)
}

fn load_history_paginated_internal(
    page: usize,
    page_size: usize,
    history_path: Option<PathBuf>,
) -> Result<(Vec<HistoryEntry>, usize), HistoryError> {
    let history_path = get_history_file_path_internal(history_path)?;

    // If file doesn't exist, return empty vector
    if !history_path.exists() {
        return Ok((Vec::new(), 0));
    }

    let file = File::open(&history_path)?;
    let reader = BufReader::new(file);

    let mut all_entries = Vec::with_capacity(page_size);
    let mut total_count = 0;
    let skip_count = page * page_size;

    for (_line_num, line_result) in reader.lines().enumerate() {
        if let Ok(line) = line_result {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse the JSON line
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                total_count += 1;

                // Only keep entries in the requested page
                if total_count > skip_count && all_entries.len() < page_size {
                    all_entries.push(entry);
                }
            }
        }
    }

    Ok((all_entries, total_count))
}

/// Loads the most recent N history entries efficiently.
///
/// This is optimized for the common case of showing recent history by reading
/// the file in reverse order (most recent first) and stopping once we have
/// enough entries.
///
/// # Arguments
///
/// * `count` - Maximum number of recent entries to load
///
/// # Returns
///
/// A vector of the most recent history entries, sorted by timestamp (newest first).
///
/// # Errors
///
/// Returns `HistoryError::StorageError` if the history file cannot be read.
///
/// # Example
///
/// ```ignore
/// // Load the 50 most recent requests
/// let recent = load_recent_history(50)?;
/// ```
pub fn load_recent_history(count: usize) -> Result<Vec<HistoryEntry>, HistoryError> {
    load_recent_history_internal(count, None)
}

fn load_recent_history_internal(
    count: usize,
    history_path: Option<PathBuf>,
) -> Result<Vec<HistoryEntry>, HistoryError> {
    let history_path = get_history_file_path_internal(history_path)?;

    // If file doesn't exist, return empty vector
    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&history_path)?;
    let reader = BufReader::new(file);

    // Load all entries (we need to do this to get the most recent ones)
    // In a future optimization, we could read the file backwards
    let mut entries = Vec::with_capacity(count);

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Try to parse the JSON line
            if let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) {
                entries.push(entry);
            }
        }
    }

    // Sort by timestamp descending (newest first) and take only the requested count
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    entries.truncate(count);

    Ok(entries)
}

/// Counts the total number of history entries without loading them all.
///
/// This is much faster than loading all entries when you only need the count.
///
/// # Returns
///
/// The total number of valid history entries.
///
/// # Errors
///
/// Returns `HistoryError::StorageError` if the history file cannot be read.
///
/// # Example
///
/// ```ignore
/// let total = count_history_entries()?;
/// println!("Total history entries: {}", total);
/// ```
pub fn count_history_entries() -> Result<usize, HistoryError> {
    count_history_entries_internal(None)
}

fn count_history_entries_internal(history_path: Option<PathBuf>) -> Result<usize, HistoryError> {
    let history_path = get_history_file_path_internal(history_path)?;

    // If file doesn't exist, return 0
    if !history_path.exists() {
        return Ok(0);
    }

    let file = File::open(&history_path)?;
    let reader = BufReader::new(file);

    let mut count = 0;

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Count valid JSON lines (we don't need to fully parse them)
            if serde_json::from_str::<serde_json::Value>(&line).is_ok() {
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Maintains the history file by removing old entries when the limit is exceeded.
///
/// Reads all entries, keeps only the most recent `max_entries` entries, and
/// rewrites the history file. This operation is atomic - the file is only
/// replaced if the write succeeds.
///
/// # Arguments
///
/// * `max_entries` - Maximum number of entries to keep
///
/// # Returns
///
/// The number of entries removed.
///
/// # Errors
///
/// Returns `HistoryError` if:
/// - The history file cannot be read
/// - A temporary file cannot be created
/// - The history file cannot be replaced
///
/// # Example
///
/// ```ignore
/// // Keep only the last 1000 entries
/// let removed = maintain_history_limit(1000)?;
/// println!("Removed {} old entries", removed);
/// ```
pub fn maintain_history_limit(max_entries: usize) -> Result<usize, HistoryError> {
    maintain_history_limit_internal(max_entries, None)
}

fn maintain_history_limit_internal(
    max_entries: usize,
    history_path: Option<PathBuf>,
) -> Result<usize, HistoryError> {
    let history_path = get_history_file_path_internal(history_path)?;

    // If file doesn't exist, nothing to do
    if !history_path.exists() {
        return Ok(0);
    }

    // Load all entries
    let mut entries = load_history_internal(Some(history_path.clone()))?;

    // If we're under the limit, nothing to do
    if entries.len() <= max_entries {
        return Ok(0);
    }

    let entries_to_remove = entries.len() - max_entries;

    // Sort by timestamp (oldest first) and keep only the most recent
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    entries = entries.into_iter().skip(entries_to_remove).collect();

    // Write to a temporary file first
    let temp_path = history_path.with_extension("json.tmp");
    let mut temp_file = File::create(&temp_path)?;

    for entry in &entries {
        let json = serde_json::to_string(entry)?;
        writeln!(temp_file, "{}", json)?;
    }

    temp_file.flush()?;
    drop(temp_file); // Close the file

    // Atomically replace the old file with the new one
    fs::rename(&temp_path, &history_path)?;

    Ok(entries_to_remove)
}

/// Clears all history entries.
///
/// Deletes the history file, removing all stored entries.
///
/// # Returns
///
/// `Ok(())` if the history was cleared successfully.
///
/// # Errors
///
/// Returns `HistoryError::StorageError` if the file cannot be deleted.
/// If the file doesn't exist, this is considered success.
pub fn clear_history() -> Result<(), HistoryError> {
    clear_history_internal(None)
}

fn clear_history_internal(history_path: Option<PathBuf>) -> Result<(), HistoryError> {
    let history_path = get_history_file_path_internal(history_path)?;

    if history_path.exists() {
        fs::remove_file(&history_path)?;
    }

    Ok(())
}

/// Gets the current number of entries in the history file.
///
/// # Returns
///
/// The number of valid entries in the history file.
///
/// # Errors
///
/// Returns `HistoryError` if the history file cannot be read.
pub fn get_history_count() -> Result<usize, HistoryError> {
    get_history_count_internal(None)
}

fn get_history_count_internal(history_path: Option<PathBuf>) -> Result<usize, HistoryError> {
    let entries = load_history_internal(history_path)?;
    Ok(entries.len())
}

/// Rebuilds the history file, removing corrupted entries.
///
/// Reads all valid entries and rewrites the history file, effectively
/// removing any corrupted lines. This is useful for recovering from
/// file corruption.
///
/// # Returns
///
/// A tuple of `(valid_entries, corrupted_entries)`.
///
/// # Errors
///
/// Returns `HistoryError` if the history file cannot be read or written.
pub fn rebuild_history() -> Result<(usize, usize), HistoryError> {
    let history_path = get_history_file_path()?;

    // If file doesn't exist, nothing to do
    if !history_path.exists() {
        return Ok((0, 0));
    }

    // Load all valid entries (corrupted ones are skipped)
    let entries = load_history()?;
    let valid_count = entries.len();

    // Count total lines in original file
    let file = File::open(&history_path)?;
    let reader = BufReader::new(file);
    let total_lines = reader.lines().filter(|l| l.is_ok()).count();
    let corrupted_count = total_lines.saturating_sub(valid_count);

    // Write to a temporary file
    let temp_path = history_path.with_extension("json.tmp");
    let mut temp_file = File::create(&temp_path)?;

    for entry in &entries {
        let json = serde_json::to_string(entry)?;
        writeln!(temp_file, "{}", json)?;
    }

    temp_file.flush()?;
    drop(temp_file);

    // Replace the old file
    fs::rename(&temp_path, &history_path)?;

    Ok((valid_count, corrupted_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HttpMethod, HttpRequest, HttpResponse};
    use tempfile::TempDir;

    fn create_test_entry() -> HistoryEntry {
        let request = HttpRequest::new(
            "test-id".to_string(),
            HttpMethod::GET,
            "https://api.example.com/test".to_string(),
        );
        let response = HttpResponse::new(200, "OK".to_string());
        HistoryEntry::new(request, response)
    }

    fn get_test_history_path() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let unique_file = format!("test_history_{}.json", uuid::Uuid::new_v4());
        temp_dir.join(unique_file)
    }

    #[test]
    fn test_history_config_default() {
        let config = HistoryConfig::default();
        assert_eq!(config.max_entries, DEFAULT_MAX_HISTORY_ENTRIES);
        assert!(config.sanitize_sensitive_headers);
        assert!(!config.save_failed_requests);
    }

    #[test]
    fn test_save_and_load_single_entry() {
        let test_path = get_test_history_path();
        let entry = create_test_entry();
        let entry_id = entry.id.clone();

        // Save entry
        save_entry_internal(&entry, &HistoryConfig::default(), Some(test_path.clone())).unwrap();

        // Load entries
        let loaded = load_history_internal(Some(test_path.clone())).unwrap();

        // Find our entry
        let found = loaded.iter().find(|e| e.id == entry_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().request.url, entry.request.url);

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_save_multiple_entries() {
        let test_path = get_test_history_path();

        let entry1 = create_test_entry();
        let entry2 = create_test_entry();
        let entry3 = create_test_entry();

        save_entry_internal(&entry1, &HistoryConfig::default(), Some(test_path.clone())).unwrap();
        save_entry_internal(&entry2, &HistoryConfig::default(), Some(test_path.clone())).unwrap();
        save_entry_internal(&entry3, &HistoryConfig::default(), Some(test_path.clone())).unwrap();

        let loaded = load_history_internal(Some(test_path.clone())).unwrap();
        assert_eq!(loaded.len(), 3);

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_load_empty_history() {
        let test_path = get_test_history_path();

        let loaded = load_history_internal(Some(test_path)).unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_maintain_history_limit() {
        let test_path = get_test_history_path();

        // Add more entries than the limit
        for _ in 0..15 {
            save_entry_internal(
                &create_test_entry(),
                &HistoryConfig::default(),
                Some(test_path.clone()),
            )
            .unwrap();
        }

        // Maintain with limit of 10
        let removed = maintain_history_limit_internal(10, Some(test_path.clone())).unwrap();
        assert_eq!(removed, 5);

        // Verify we have exactly 10 entries
        let loaded = load_history_internal(Some(test_path.clone())).unwrap();
        assert_eq!(loaded.len(), 10);

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_maintain_history_under_limit() {
        let test_path = get_test_history_path();

        // Add 5 entries
        for _ in 0..5 {
            save_entry_internal(
                &create_test_entry(),
                &HistoryConfig::default(),
                Some(test_path.clone()),
            )
            .unwrap();
        }

        // Maintain with limit of 10 (should remove 0)
        let removed = maintain_history_limit_internal(10, Some(test_path.clone())).unwrap();
        assert_eq!(removed, 0);

        let loaded = load_history_internal(Some(test_path.clone())).unwrap();
        assert_eq!(loaded.len(), 5);

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_clear_history() {
        let test_path = get_test_history_path();

        // Add some entries
        save_entry_internal(
            &create_test_entry(),
            &HistoryConfig::default(),
            Some(test_path.clone()),
        )
        .unwrap();
        save_entry_internal(
            &create_test_entry(),
            &HistoryConfig::default(),
            Some(test_path.clone()),
        )
        .unwrap();

        // Clear
        clear_history_internal(Some(test_path.clone())).unwrap();

        // Verify empty
        let loaded = load_history_internal(Some(test_path)).unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_get_history_count() {
        let test_path = get_test_history_path();

        assert_eq!(
            get_history_count_internal(Some(test_path.clone())).unwrap(),
            0
        );

        save_entry_internal(
            &create_test_entry(),
            &HistoryConfig::default(),
            Some(test_path.clone()),
        )
        .unwrap();
        save_entry_internal(
            &create_test_entry(),
            &HistoryConfig::default(),
            Some(test_path.clone()),
        )
        .unwrap();

        let count = get_history_count_internal(Some(test_path.clone())).unwrap();
        assert_eq!(count, 2);

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_sanitize_sensitive_headers() {
        let test_path = get_test_history_path();
        let mut request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        request.add_header("Authorization".to_string(), "Bearer secret".to_string());
        request.add_header("Content-Type".to_string(), "application/json".to_string());

        let response = HttpResponse::new(200, "OK".to_string());
        let entry = HistoryEntry::new(request, response);

        // Save with sanitization enabled
        let config = HistoryConfig {
            sanitize_sensitive_headers: true,
            ..Default::default()
        };

        save_entry_internal(&entry, &config, Some(test_path.clone())).unwrap();

        // Load and verify
        let loaded = load_history_internal(Some(test_path.clone())).unwrap();
        let found = loaded.iter().find(|e| e.id == entry.id);

        assert!(found.is_some());
        let loaded_entry = found.unwrap();
        assert!(!loaded_entry.request.headers.contains_key("Authorization"));
        assert!(loaded_entry.request.headers.contains_key("Content-Type"));

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }

    #[test]
    fn test_save_failed_requests_config() {
        let test_path = get_test_history_path();

        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com".to_string(),
        );
        let response = HttpResponse::new(404, "Not Found".to_string());
        let entry = HistoryEntry::new(request, response);

        // Default config (don't save failed)
        save_entry_internal(&entry, &HistoryConfig::default(), Some(test_path.clone())).unwrap();
        let count_before = get_history_count_internal(Some(test_path.clone())).unwrap();

        // Enable saving failed requests
        let config = HistoryConfig {
            save_failed_requests: true,
            ..Default::default()
        };

        save_entry_internal(&entry, &config, Some(test_path.clone())).unwrap();
        let count_after = get_history_count_internal(Some(test_path.clone())).unwrap();

        // Should have one more entry with save_failed_requests enabled
        assert_eq!(count_after, count_before + 1);

        // Cleanup
        let _ = std::fs::remove_file(test_path);
    }
}
