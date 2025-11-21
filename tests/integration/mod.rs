//! Integration tests module for REST Client
//!
//! This module provides common utilities and test infrastructure
//! for integration testing of the REST Client extension.

pub mod end_to_end_test;
pub mod request_chaining_test;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment (run once)
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize logging or other global test setup if needed
    });
}

/// Clean up test artifacts
pub fn cleanup_test_artifacts() {
    // Clean up any temporary files, history files, etc.
    let _ = std::fs::remove_dir_all("test_temp");
}
