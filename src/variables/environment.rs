//! Environment variable resolution for REST Client
//!
//! This module provides functions for resolving environment variables from
//! the active environment and shared variables, with proper fallback logic.

use crate::environment::{Environment, Environments};
use std::collections::HashMap;

/// Resolves an environment variable by name
///
/// Looks up the variable in the environment first, then falls back to shared variables.
/// This function follows the precedence:
/// 1. Environment-specific variables (if environment is provided)
/// 2. Shared variables
///
/// # Arguments
///
/// * `name` - The variable name to resolve
/// * `env` - Optional environment to search first
/// * `shared` - Shared variables to use as fallback
///
/// # Returns
///
/// The resolved variable value, or None if not found in either source
///
/// # Example
///
/// ```no_run
/// use std::collections::HashMap;
/// use rest_client::environment::Environment;
/// use rest_client::variables::environment::resolve_environment_variable;
///
/// let mut env = Environment::new("dev");
/// env.set("baseUrl", "http://localhost:3000");
///
/// let mut shared = HashMap::new();
/// shared.insert("apiVersion".to_string(), "v1".to_string());
///
/// // Resolves from environment
/// let url = resolve_environment_variable("baseUrl", Some(&env), &shared);
/// assert_eq!(url, Some("http://localhost:3000".to_string()));
///
/// // Falls back to shared
/// let version = resolve_environment_variable("apiVersion", Some(&env), &shared);
/// assert_eq!(version, Some("v1".to_string()));
///
/// // Not found
/// let missing = resolve_environment_variable("notFound", Some(&env), &shared);
/// assert_eq!(missing, None);
/// ```
pub fn resolve_environment_variable(
    name: &str,
    env: Option<&Environment>,
    shared: &HashMap<String, String>,
) -> Option<String> {
    // First try environment-specific variables
    if let Some(environment) = env {
        if let Some(value) = environment.get(name) {
            return Some(value.clone());
        }
    }

    // Fall back to shared variables
    shared.get(name).cloned()
}

/// Resolves a variable with fallback from Environments struct
///
/// This function tries to resolve a variable using the following precedence:
/// 1. Active environment variables (if an environment is active)
/// 2. Shared variables
/// 3. None if not found
///
/// This is a convenience wrapper around `resolve_environment_variable` that
/// works directly with the `Environments` struct.
///
/// # Arguments
///
/// * `name` - The variable name to resolve
/// * `environments` - The Environments struct containing all environments and shared variables
///
/// # Returns
///
/// The resolved variable value, or None if not found
///
/// # Example
///
/// ```no_run
/// use rest_client::environment::{Environment, Environments};
/// use rest_client::variables::environment::resolve_with_fallback;
///
/// let mut envs = Environments::new();
/// envs.set_shared("sharedVar", "shared value");
///
/// let mut dev = Environment::new("dev");
/// dev.set("devVar", "dev value");
/// envs.add_environment(dev);
/// envs.set_active("dev");
///
/// // Resolves from active environment
/// assert_eq!(
///     resolve_with_fallback("devVar", &envs),
///     Some("dev value".to_string())
/// );
///
/// // Falls back to shared
/// assert_eq!(
///     resolve_with_fallback("sharedVar", &envs),
///     Some("shared value".to_string())
/// );
///
/// // Not found
/// assert_eq!(resolve_with_fallback("missing", &envs), None);
/// ```
pub fn resolve_with_fallback(name: &str, environments: &Environments) -> Option<String> {
    // This uses the get_variable method which already implements the correct precedence
    environments.get_variable(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::{Environment, Environments};

    #[test]
    fn test_resolve_environment_variable_from_env() {
        let mut env = Environment::new("dev");
        env.set("baseUrl", "http://localhost:3000");
        env.set("apiKey", "dev-key-123");

        let shared = HashMap::new();

        let result = resolve_environment_variable("baseUrl", Some(&env), &shared);
        assert_eq!(result, Some("http://localhost:3000".to_string()));
    }

    #[test]
    fn test_resolve_environment_variable_from_shared() {
        let env = Environment::new("dev");

        let mut shared = HashMap::new();
        shared.insert("apiVersion".to_string(), "v1".to_string());
        shared.insert("timeout".to_string(), "30".to_string());

        let result = resolve_environment_variable("apiVersion", Some(&env), &shared);
        assert_eq!(result, Some("v1".to_string()));
    }

    #[test]
    fn test_resolve_environment_variable_env_overrides_shared() {
        let mut env = Environment::new("dev");
        env.set("baseUrl", "http://dev.example.com");

        let mut shared = HashMap::new();
        shared.insert(
            "baseUrl".to_string(),
            "http://shared.example.com".to_string(),
        );

        // Environment variable should take precedence
        let result = resolve_environment_variable("baseUrl", Some(&env), &shared);
        assert_eq!(result, Some("http://dev.example.com".to_string()));
    }

    #[test]
    fn test_resolve_environment_variable_not_found() {
        let env = Environment::new("dev");
        let shared = HashMap::new();

        let result = resolve_environment_variable("nonexistent", Some(&env), &shared);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_environment_variable_no_env() {
        let mut shared = HashMap::new();
        shared.insert("sharedVar".to_string(), "shared value".to_string());

        // No environment provided, should still resolve from shared
        let result = resolve_environment_variable("sharedVar", None, &shared);
        assert_eq!(result, Some("shared value".to_string()));
    }

    #[test]
    fn test_resolve_environment_variable_case_sensitive() {
        let mut env = Environment::new("dev");
        env.set("BaseUrl", "http://uppercase.com");
        env.set("baseUrl", "http://lowercase.com");

        let shared = HashMap::new();

        // Variable names are case-sensitive
        assert_eq!(
            resolve_environment_variable("BaseUrl", Some(&env), &shared),
            Some("http://uppercase.com".to_string())
        );
        assert_eq!(
            resolve_environment_variable("baseUrl", Some(&env), &shared),
            Some("http://lowercase.com".to_string())
        );
        assert_eq!(
            resolve_environment_variable("BASEURL", Some(&env), &shared),
            None
        );
    }

    #[test]
    fn test_resolve_with_fallback_active_env() {
        let mut envs = Environments::new();

        let mut dev = Environment::new("dev");
        dev.set("devVar", "dev value");
        dev.set("override", "from dev");
        envs.add_environment(dev);

        envs.set_shared("sharedVar", "shared value");
        envs.set_shared("override", "from shared");

        envs.set_active("dev");

        // Resolves from active environment
        assert_eq!(
            resolve_with_fallback("devVar", &envs),
            Some("dev value".to_string())
        );

        // Environment overrides shared
        assert_eq!(
            resolve_with_fallback("override", &envs),
            Some("from dev".to_string())
        );

        // Falls back to shared
        assert_eq!(
            resolve_with_fallback("sharedVar", &envs),
            Some("shared value".to_string())
        );
    }

    #[test]
    fn test_resolve_with_fallback_no_active_env() {
        let mut envs = Environments::new();

        envs.set_shared("sharedVar", "shared value");

        let mut dev = Environment::new("dev");
        dev.set("devVar", "dev value");
        envs.add_environment(dev);

        // No active environment set, should only resolve from shared
        assert_eq!(
            resolve_with_fallback("sharedVar", &envs),
            Some("shared value".to_string())
        );

        // Environment variable not accessible without active environment
        assert_eq!(resolve_with_fallback("devVar", &envs), None);
    }

    #[test]
    fn test_resolve_with_fallback_not_found() {
        let mut envs = Environments::new();
        envs.set_shared("existing", "value");

        let result = resolve_with_fallback("nonexistent", &envs);
        assert_eq!(result, None);
    }

    #[test]
    fn test_resolve_with_fallback_multiple_environments() {
        let mut envs = Environments::new();

        let mut dev = Environment::new("dev");
        dev.set("url", "http://dev.example.com");
        envs.add_environment(dev);

        let mut staging = Environment::new("staging");
        staging.set("url", "http://staging.example.com");
        envs.add_environment(staging);

        let mut prod = Environment::new("prod");
        prod.set("url", "http://prod.example.com");
        envs.add_environment(prod);

        // Activate dev
        envs.set_active("dev");
        assert_eq!(
            resolve_with_fallback("url", &envs),
            Some("http://dev.example.com".to_string())
        );

        // Switch to staging
        envs.set_active("staging");
        assert_eq!(
            resolve_with_fallback("url", &envs),
            Some("http://staging.example.com".to_string())
        );

        // Switch to prod
        envs.set_active("prod");
        assert_eq!(
            resolve_with_fallback("url", &envs),
            Some("http://prod.example.com".to_string())
        );
    }
}
