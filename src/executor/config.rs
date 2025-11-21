//! HTTP request execution configuration.
//!
//! This module defines configuration options for HTTP request execution,
//! including timeout settings and other execution parameters.

use serde::{Deserialize, Serialize};

/// Configuration for HTTP request execution.
///
/// Defines parameters that control how HTTP requests are executed,
/// such as timeout durations and retry behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Request timeout in seconds.
    ///
    /// Maximum time to wait for a complete response (including connection,
    /// headers, and body download). Defaults to 30 seconds.
    pub timeout_secs: u64,
}

impl ExecutionConfig {
    /// Creates a new ExecutionConfig with the given timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout_secs` - Timeout duration in seconds
    ///
    /// # Returns
    ///
    /// A new `ExecutionConfig` instance.
    pub fn new(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }

    /// Returns the timeout as a `std::time::Duration`.
    ///
    /// # Returns
    ///
    /// Duration representing the configured timeout.
    pub fn timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_secs)
    }
}

impl Default for ExecutionConfig {
    /// Creates a default ExecutionConfig with 30 second timeout.
    fn default() -> Self {
        Self { timeout_secs: 30 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_config_new() {
        let config = ExecutionConfig::new(60);
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_timeout_duration() {
        let config = ExecutionConfig::new(45);
        assert_eq!(
            config.timeout_duration(),
            std::time::Duration::from_secs(45)
        );
    }

    #[test]
    fn test_serialization() {
        let config = ExecutionConfig::new(120);
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("120"));

        let deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.timeout_secs, 120);
    }
}
