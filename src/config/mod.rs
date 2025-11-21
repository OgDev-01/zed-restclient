//! Configuration management for the REST Client extension.
//!
//! This module provides configuration loading, validation, and access through a singleton pattern.
//! Configuration is loaded from Zed settings under the "rest-client" key and merged with defaults.

pub mod schema;

pub use schema::{ResponsePanePosition, RestClientConfig};

use once_cell::sync::Lazy;
use serde_json::Value;
use std::sync::RwLock;

/// Global configuration instance.
///
/// This is lazily initialized on first access and can be updated when settings change.
static CONFIG: Lazy<RwLock<RestClientConfig>> =
    Lazy::new(|| RwLock::new(RestClientConfig::default()));

/// Loads configuration from Zed settings or a JSON value.
///
/// This function reads the "rest-client" settings, merges them with defaults,
/// validates the result, and updates the global configuration.
///
/// # Arguments
///
/// * `settings_json` - Optional JSON value containing user settings under "rest-client" key
///
/// # Returns
///
/// `Ok(RestClientConfig)` with the loaded configuration, or `Err` if validation fails.
///
/// # Example
///
/// ```no_run
/// use rest_client::config::load_config;
/// use serde_json::json;
///
/// let settings = json!({
///     "rest-client": {
///         "timeout": 60000,
///         "validateSSL": false
///     }
/// });
///
/// let config = load_config(Some(settings)).unwrap();
/// assert_eq!(config.timeout, 60000);
/// ```
pub fn load_config(settings_json: Option<Value>) -> Result<RestClientConfig, String> {
    let mut config = RestClientConfig::default();

    if let Some(settings) = settings_json {
        // Extract rest-client settings if present
        if let Some(rest_client_settings) = settings.get("rest-client") {
            // Deserialize user settings
            match serde_json::from_value::<RestClientConfig>(rest_client_settings.clone()) {
                Ok(user_config) => {
                    // Merge with defaults (user settings take precedence)
                    config = config.merge(&user_config);
                }
                Err(e) => {
                    // Log error but continue with defaults
                    eprintln!(
                        "Warning: Failed to parse rest-client settings: {}. Using defaults.",
                        e
                    );
                }
            }
        }
    }

    // Validate the merged configuration
    config
        .validate()
        .map_err(|e| format!("Invalid configuration: {}. Using defaults.", e))?;

    // Update the global configuration
    if let Ok(mut global_config) = CONFIG.write() {
        *global_config = config.clone();
    }

    Ok(config)
}

/// Gets the current global configuration.
///
/// This is a singleton accessor that returns a clone of the current configuration.
/// If configuration has not been loaded yet, returns the default configuration.
///
/// # Returns
///
/// A cloned `RestClientConfig` instance.
///
/// # Example
///
/// ```no_run
/// use rest_client::config::get_config;
///
/// let config = get_config();
/// println!("Timeout: {}ms", config.timeout);
/// ```
pub fn get_config() -> RestClientConfig {
    CONFIG
        .read()
        .map(|c| c.clone())
        .unwrap_or_else(|_| RestClientConfig::default())
}

/// Updates a specific configuration setting.
///
/// This allows for granular updates to the configuration without replacing
/// the entire config object.
///
/// # Arguments
///
/// * `updater` - A closure that modifies the configuration
///
/// # Example
///
/// ```no_run
/// use rest_client::config::update_config;
///
/// update_config(|config| {
///     config.timeout = 60000;
/// });
/// ```
pub fn update_config<F>(updater: F)
where
    F: FnOnce(&mut RestClientConfig),
{
    if let Ok(mut config) = CONFIG.write() {
        updater(&mut config);

        // Validate after update
        if let Err(e) = config.validate() {
            eprintln!(
                "Warning: Configuration validation failed after update: {}",
                e
            );
            // Revert to defaults if validation fails
            *config = RestClientConfig::default();
        }
    }
}

/// Resets the configuration to defaults.
///
/// This is useful for testing or when user wants to clear custom settings.
pub fn reset_config() {
    if let Ok(mut config) = CONFIG.write() {
        *config = RestClientConfig::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_load_config_with_defaults() {
        let config = load_config(None).unwrap();
        assert_eq!(config.timeout, 30000);
        assert_eq!(config.validate_ssl, true);
        assert_eq!(config.history_limit, 1000);
    }

    #[test]
    fn test_load_config_with_user_settings() {
        let settings = json!({
            "rest-client": {
                "timeout": 60000,
                "validateSsl": false,
                "historyLimit": 500
            }
        });

        let config = load_config(Some(settings)).unwrap();
        assert_eq!(config.timeout, 60000);
        assert_eq!(config.validate_ssl, false);
        assert_eq!(config.history_limit, 500);
        // Other settings should still have defaults
        assert_eq!(config.follow_redirects, true);
    }

    #[test]
    fn test_load_config_partial_settings() {
        let settings = json!({
            "rest-client": {
                "timeout": 45000
            }
        });

        let config = load_config(Some(settings)).unwrap();
        assert_eq!(config.timeout, 45000);
        // All other settings should be defaults
        assert_eq!(config.validate_ssl, true);
        assert_eq!(config.history_limit, 1000);
        assert_eq!(config.max_redirects, 10);
    }

    #[test]
    fn test_load_config_invalid_json() {
        let settings = json!({
            "rest-client": {
                "timeout": "not-a-number"
            }
        });

        // Should fall back to defaults on parse error
        let config = load_config(Some(settings)).unwrap();
        assert_eq!(config.timeout, 30000); // Default
    }

    #[test]
    fn test_load_config_validation_error() {
        let settings = json!({
            "rest-client": {
                "timeout": 0
            }
        });

        let result = load_config(Some(settings));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("timeout must be greater than 0"));
    }

    #[test]
    fn test_get_config() {
        // Reset to ensure clean state
        reset_config();

        let config = get_config();
        assert_eq!(config.timeout, 30000);

        // Load custom config
        let settings = json!({
            "rest-client": {
                "timeout": 90000
            }
        });
        load_config(Some(settings)).unwrap();

        // Get config should return updated value
        let config = get_config();
        assert_eq!(config.timeout, 90000);

        // Reset for other tests
        reset_config();
    }

    #[test]
    fn test_update_config() {
        reset_config();

        update_config(|config| {
            config.timeout = 120000;
            config.validate_ssl = false;
        });

        let config = get_config();
        assert_eq!(config.timeout, 120000);
        assert_eq!(config.validate_ssl, false);

        reset_config();
    }

    #[test]
    fn test_update_config_with_invalid_value() {
        reset_config();

        // Try to set invalid value
        update_config(|config| {
            config.timeout = 0; // Invalid
        });

        // Should revert to defaults
        let config = get_config();
        assert_eq!(config.timeout, 30000); // Default

        reset_config();
    }

    #[test]
    fn test_reset_config() {
        let settings = json!({
            "rest-client": {
                "timeout": 75000,
                "validateSsl": false
            }
        });
        load_config(Some(settings)).unwrap();

        reset_config();

        let config = get_config();
        assert_eq!(config.timeout, 30000);
        assert_eq!(config.validate_ssl, true);
    }

    #[test]
    fn test_no_rest_client_key() {
        let settings = json!({
            "other-extension": {
                "someSetting": true
            }
        });

        let config = load_config(Some(settings)).unwrap();
        // Should use all defaults
        assert_eq!(config.timeout, 30000);
        assert_eq!(config.validate_ssl, true);
    }

    #[test]
    fn test_empty_settings() {
        let settings = json!({});

        let config = load_config(Some(settings)).unwrap();
        assert_eq!(config.timeout, 30000);
        assert_eq!(config.validate_ssl, true);
    }

    #[test]
    fn test_complex_settings() {
        let settings = json!({
            "rest-client": {
                "timeout": 45000,
                "followRedirects": false,
                "maxRedirects": 5,
                "validateSsl": false,
                "responsePane": "below",
                "historyLimit": 2000,
                "previewResponseInTab": true,
                "environmentFile": "custom-env.json",
                "excludeHostsFromProxy": ["localhost", "*.internal.com"],
                "defaultHeaders": {
                    "X-API-Key": "test123",
                    "Accept": "application/json"
                }
            }
        });

        let config = load_config(Some(settings)).unwrap();
        assert_eq!(config.timeout, 45000);
        assert_eq!(config.follow_redirects, false);
        assert_eq!(config.max_redirects, 5);
        assert_eq!(config.validate_ssl, false);
        assert_eq!(config.response_pane, ResponsePanePosition::Below);
        assert_eq!(config.history_limit, 2000);
        assert_eq!(config.preview_response_in_tab, true);
        assert_eq!(config.environment_file, "custom-env.json");
        assert_eq!(config.exclude_hosts_from_proxy.len(), 2);
        assert_eq!(config.default_headers.len(), 2);
    }
}
