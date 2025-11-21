//! Request history tracking and persistence.
//!
//! This module provides functionality for storing and retrieving HTTP request/response
//! history, allowing users to review past requests and re-execute them.
//!
//! # Features
//!
//! - Save request/response pairs to persistent storage
//! - Load history on startup
//! - Automatic history limit enforcement
//! - Sensitive data sanitization
//! - JSONL format for efficient append operations
//!
//! # Example
//!
//! ```ignore
//! use history::{HistoryEntry, save_entry, load_history};
//!
//! // Save a history entry
//! let entry = HistoryEntry::new(request, response);
//! save_entry(&entry)?;
//!
//! // Load all history
//! let entries = load_history()?;
//! ```

pub mod models;
pub mod search;
pub mod storage;
pub mod ui;

// Re-export commonly used types
pub use models::{HistoryEntry, HistoryError};
pub use search::{
    filter_by_method, filter_by_status, filter_by_tag, filter_errors, filter_successful,
    get_recent_entries, search_history, sort_by_timestamp_desc,
};
pub use storage::{clear_history, load_history, maintain_history_limit, save_entry};
pub use ui::{
    format_history_compact, format_history_details, format_history_entry,
    format_history_entry_relative, format_history_list, format_history_stats,
};
