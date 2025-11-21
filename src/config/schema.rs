//! Configuration schema for the REST Client extension.
//!
//! This module defines the configuration structure and validation logic for all
//! user-configurable settings in the REST Client extension.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration structure for the REST Client extension.
///
/// All settings can be configured via Zed's settings under the "rest-client" key.
/// Missing or invalid settings will fall back to sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestClientConfig {
    /// Request timeout in milliseconds.
    ///
    /// Maximum time to wait for a complete response (including connection,
    /// headers, and body download). Defaults to 30000ms (30 seconds).
    ///
    /// Must be greater than 0.
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Whether to automatically follow HTTP redirects.
    ///
    /// When enabled, the HTTP client will automatically follow 3xx redirect
    /// responses up to `max_redirects` times. Defaults to true.
    #[serde(default = "default_follow_redirects")]
    pub follow_redirects: bool,

    /// Maximum number of redirects to follow.
    ///
    /// Only used when `follow_redirects` is true. Defaults to 10.
    ///
    /// Must be >= 0.
    #[serde(default = "default_max_redirects")]
    pub max_redirects: u32,

    /// Whether to validate SSL/TLS certificates.
    ///
    /// When enabled, requests to HTTPS endpoints will fail if the certificate
    /// is invalid, self-signed, or expired. Defaults to true for security.
    ///
    /// **Warning:** Disabling SSL validation can expose you to security risks.
    #[serde(default = "default_validate_ssl")]
    pub validate_ssl: bool,

    /// Position of the response pane.
    ///
    /// Controls where the response is displayed relative to the request file.
    /// Valid values: "right", "below", "tab". Defaults to "right".
    #[serde(default = "default_response_pane")]
    pub response_pane: ResponsePanePosition,

    /// Maximum number of requests to keep in history.
    ///
    /// Older requests beyond this limit will be automatically removed.
    /// Defaults to 1000.
    ///
    /// Must be > 0.
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,

    /// Whether to preview responses in a new tab instead of a pane.
    ///
    /// When enabled, responses will open in a new editor tab rather than
    /// a split pane. Defaults to false.
    #[serde(default = "default_preview_response_in_tab")]
    pub preview_response_in_tab: bool,

    /// Path to the environment variables file.
    ///
    /// Relative to the workspace root. The extension will search for this file
    /// in the workspace root and up to 3 parent directories. Defaults to
    /// ".http-client-env.json".
    #[serde(default = "default_environment_file")]
    pub environment_file: String,

    /// List of hostnames to exclude from proxy settings.
    ///
    /// Even if system proxy is configured, requests to these hosts will bypass
    /// the proxy. Defaults to empty array.
    #[serde(default = "default_exclude_hosts_from_proxy")]
    pub exclude_hosts_from_proxy: Vec<String>,

    /// Default headers to include in all requests.
    ///
    /// These headers will be added to every request unless overridden by
    /// request-specific headers. Defaults to User-Agent header only.
    #[serde(default = "default_headers")]
    pub default_headers: HashMap<String, String>,
}

/// Position of the response pane relative to the request file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponsePanePosition {
    /// Display response in a pane to the right of the request file.
    Right,
    /// Display response in a pane below the request file.
    Below,
    /// Display response in a new editor tab.
    Tab,
}

impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            timeout: default_timeout(),
            follow_redirects: default_follow_redirects(),
            max_redirects: default_max_redirects(),
            validate_ssl: default_validate_ssl(),
            response_pane: default_response_pane(),
            history_limit: default_history_limit(),
            preview_response_in_tab: default_preview_response_in_tab(),
            environment_file: default_environment_file(),
            exclude_hosts_from_proxy: default_exclude_hosts_from_proxy(),
            default_headers: default_headers(),
        }
    }
}

impl RestClientConfig {
    /// Validates the configuration and returns errors if any settings are invalid.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all settings are valid, or `Err` with a descriptive error message.
    pub fn validate(&self) -> Result<(), String> {
        // Validate timeout
        if self.timeout == 0 {
            return Err("timeout must be greater than 0".to_string());
        }

        // Validate history limit
        if self.history_limit == 0 {
            return Err("historyLimit must be greater than 0".to_string());
        }

        // max_redirects can be 0 (no redirects), so no validation needed

        Ok(())
    }

    /// Returns the timeout as a `std::time::Duration`.
    ///
    /// # Returns
    ///
    /// Duration representing the configured timeout in milliseconds.
    pub fn timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout)
    }

    /// Returns the timeout in seconds for compatibility with ExecutionConfig.
    ///
    /// # Returns
    ///
    /// Timeout duration in seconds (rounded up from milliseconds).
    pub fn timeout_secs(&self) -> u64 {
        (self.timeout + 999) / 1000 // Round up
    }

    /// Merges this configuration with another, using values from `other` where present.
    ///
    /// This is useful for applying user settings on top of defaults.
    ///
    /// # Arguments
    ///
    /// * `other` - Configuration to merge with (takes precedence)
    ///
    /// # Returns
    ///
    /// A new `RestClientConfig` with merged values.
    pub fn merge(&self, other: &RestClientConfig) -> Self {
        Self {
            timeout: other.timeout,
            follow_redirects: other.follow_redirects,
            max_redirects: other.max_redirects,
            validate_ssl: other.validate_ssl,
            response_pane: other.response_pane,
            history_limit: other.history_limit,
            preview_response_in_tab: other.preview_response_in_tab,
            environment_file: other.environment_file.clone(),
            exclude_hosts_from_proxy: other.exclude_hosts_from_proxy.clone(),
            default_headers: other.default_headers.clone(),
        }
    }
}

// Default value functions for serde

fn default_timeout() -> u64 {
    30000 // 30 seconds in milliseconds
}

fn default_follow_redirects() -> bool {
    true
}

fn default_max_redirects() -> u32 {
    10
}

fn default_validate_ssl() -> bool {
    true
}

fn default_response_pane() -> ResponsePanePosition {
    ResponsePanePosition::Right
}

fn default_history_limit() -> usize {
    1000
}

fn default_preview_response_in_tab() -> bool {
    false
}

fn default_environment_file() -> String {
    ".http-client-env.json".to_string()
}

fn default_exclude_hosts_from_proxy() -> Vec<String> {
    Vec::new()
}

fn default_headers() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Zed-REST-Client/1.0".to_string());
    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RestClientConfig::default();
        assert_eq!(config.timeout, 30000);
        assert_eq!(config.follow_redirects, true);
        assert_eq!(config.max_redirects, 10);
        assert_eq!(config.validate_ssl, true);
        assert_eq!(config.response_pane, ResponsePanePosition::Right);
        assert_eq!(config.history_limit, 1000);
        assert_eq!(config.preview_response_in_tab, false);
        assert_eq!(config.environment_file, ".http-client-env.json");
        assert_eq!(config.exclude_hosts_from_proxy.len(), 0);
        assert_eq!(config.default_headers.len(), 1);
        assert_eq!(
            config.default_headers.get("User-Agent"),
            Some(&"Zed-REST-Client/1.0".to_string())
        );
    }

    #[test]
    fn test_config_validation_valid() {
        let config = RestClientConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_zero_timeout() {
        let mut config = RestClientConfig::default();
        config.timeout = 0;
        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "timeout must be greater than 0"
        );
    }

    #[test]
    fn test_config_validation_zero_history_limit() {
        let mut config = RestClientConfig::default();
        config.history_limit = 0;
        assert!(config.validate().is_err());
        assert_eq!(
            config.validate().unwrap_err(),
            "historyLimit must be greater than 0"
        );
    }

    #[test]
    fn test_config_validation_zero_redirects_allowed() {
        let mut config = RestClientConfig::default();
        config.max_redirects = 0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_timeout_duration() {
        let config = RestClientConfig {
            timeout: 5000,
            ..Default::default()
        };
        assert_eq!(
            config.timeout_duration(),
            std::time::Duration::from_millis(5000)
        );
    }

    #[test]
    fn test_timeout_secs() {
        let config = RestClientConfig {
            timeout: 5000,
            ..Default::default()
        };
        assert_eq!(config.timeout_secs(), 5);

        let config2 = RestClientConfig {
            timeout: 5500,
            ..Default::default()
        };
        assert_eq!(config2.timeout_secs(), 6); // Rounds up
    }

    #[test]
    fn test_merge_config() {
        let base = RestClientConfig::default();
        let mut custom = RestClientConfig::default();
        custom.timeout = 60000;
        custom.validate_ssl = false;
        custom.history_limit = 500;

        let merged = base.merge(&custom);
        assert_eq!(merged.timeout, 60000);
        assert_eq!(merged.validate_ssl, false);
        assert_eq!(merged.history_limit, 500);
        assert_eq!(merged.follow_redirects, true); // Unchanged
    }

    #[test]
    fn test_deserialization_with_defaults() {
        let json = r#"{
            "timeout": 60000,
            "validateSsl": false
        }"#;

        let config: RestClientConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.timeout, 60000);
        assert_eq!(config.validate_ssl, false);
        // Other fields should have defaults
        assert_eq!(config.follow_redirects, true);
        assert_eq!(config.max_redirects, 10);
        assert_eq!(config.history_limit, 1000);
    }

    #[test]
    fn test_serialization() {
        let config = RestClientConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("timeout"));
        assert!(json.contains("30000"));
        assert!(json.contains("validateSsl"));
    }

    #[test]
    fn test_response_pane_position_deserialization() {
        let json_right = r#"{"responsePane": "right"}"#;
        let config: RestClientConfig = serde_json::from_str(json_right).unwrap();
        assert_eq!(config.response_pane, ResponsePanePosition::Right);

        let json_below = r#"{"responsePane": "below"}"#;
        let config: RestClientConfig = serde_json::from_str(json_below).unwrap();
        assert_eq!(config.response_pane, ResponsePanePosition::Below);

        let json_tab = r#"{"responsePane": "tab"}"#;
        let config: RestClientConfig = serde_json::from_str(json_tab).unwrap();
        assert_eq!(config.response_pane, ResponsePanePosition::Tab);
    }

    #[test]
    fn test_default_headers() {
        let json = r#"{
            "defaultHeaders": {
                "Authorization": "Bearer token123",
                "X-Custom": "value"
            }
        }"#;

        let config: RestClientConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.default_headers.len(), 2);
        assert_eq!(
            config.default_headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
        assert_eq!(
            config.default_headers.get("X-Custom"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_exclude_hosts_from_proxy() {
        let json = r#"{
            "excludeHostsFromProxy": ["localhost", "127.0.0.1", "*.internal.example.com"]
        }"#;

        let config: RestClientConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.exclude_hosts_from_proxy.len(), 3);
        assert!(config
            .exclude_hosts_from_proxy
            .contains(&"localhost".to_string()));
        assert!(config
            .exclude_hosts_from_proxy
            .contains(&"*.internal.example.com".to_string()));
    }
}
