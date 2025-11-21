//! Environment data models for REST Client
//!
//! This module defines the core data structures for managing environment configurations.
//! Environments allow users to define different sets of variables for different contexts
//! (e.g., dev, staging, production) and switch between them easily.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single environment with its variables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    /// Environment name (e.g., "dev", "staging", "production")
    pub name: String,

    /// Variable key-value pairs for this environment
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

impl Environment {
    /// Creates a new environment with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variables: HashMap::new(),
        }
    }

    /// Creates a new environment with name and variables
    pub fn with_variables(name: impl Into<String>, variables: HashMap<String, String>) -> Self {
        Self {
            name: name.into(),
            variables,
        }
    }

    /// Gets a variable value by name
    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    /// Sets a variable value
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    /// Checks if a variable exists
    pub fn contains(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    /// Returns the number of variables
    pub fn len(&self) -> usize {
        self.variables.len()
    }

    /// Checks if the environment has no variables
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
}

/// Container for all environments and shared variables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environments {
    /// Named environments (e.g., "dev", "staging", "production")
    #[serde(default)]
    pub environments: HashMap<String, Environment>,

    /// Shared variables available in all environments
    #[serde(default)]
    pub shared: HashMap<String, String>,

    /// Currently active environment name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
}

impl Environments {
    /// Creates a new empty Environments collection
    pub fn new() -> Self {
        Self {
            environments: HashMap::new(),
            shared: HashMap::new(),
            active: None,
        }
    }

    /// Adds an environment to the collection
    pub fn add_environment(&mut self, env: Environment) {
        let name = env.name.clone();
        self.environments.insert(name, env);
    }

    /// Gets an environment by name
    pub fn get_environment(&self, name: &str) -> Option<&Environment> {
        self.environments.get(name)
    }

    /// Gets a mutable reference to an environment by name
    pub fn get_environment_mut(&mut self, name: &str) -> Option<&mut Environment> {
        self.environments.get_mut(name)
    }

    /// Sets the active environment
    pub fn set_active(&mut self, name: impl Into<String>) -> bool {
        let name = name.into();
        if self.environments.contains_key(&name) {
            self.active = Some(name);
            true
        } else {
            false
        }
    }

    /// Gets the currently active environment
    pub fn get_active(&self) -> Option<&Environment> {
        self.active
            .as_ref()
            .and_then(|name| self.environments.get(name))
    }

    /// Gets a variable value, checking active environment first, then shared
    ///
    /// This method follows the precedence:
    /// 1. Active environment variables (if an environment is active)
    /// 2. Shared variables
    pub fn get_variable(&self, key: &str) -> Option<String> {
        // First check active environment
        if let Some(env) = self.get_active() {
            if let Some(value) = env.get(key) {
                return Some(value.clone());
            }
        }

        // Fall back to shared variables
        self.shared.get(key).cloned()
    }

    /// Sets a shared variable
    pub fn set_shared(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.shared.insert(key.into(), value.into());
    }

    /// Gets all merged variables for the active environment
    ///
    /// Returns a HashMap with shared variables merged with active environment variables.
    /// Environment-specific variables take precedence over shared variables.
    pub fn get_merged_variables(&self) -> HashMap<String, String> {
        let mut merged = self.shared.clone();

        if let Some(env) = self.get_active() {
            // Environment variables override shared variables
            merged.extend(env.variables.clone());
        }

        merged
    }

    /// Lists all environment names
    pub fn list_environments(&self) -> Vec<String> {
        self.environments.keys().cloned().collect()
    }

    /// Checks if an environment exists
    pub fn has_environment(&self, name: &str) -> bool {
        self.environments.contains_key(name)
    }

    /// Returns the number of environments
    pub fn len(&self) -> usize {
        self.environments.len()
    }

    /// Checks if there are no environments
    pub fn is_empty(&self) -> bool {
        self.environments.is_empty()
    }
}

impl Default for Environments {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_new() {
        let env = Environment::new("dev");
        assert_eq!(env.name, "dev");
        assert!(env.variables.is_empty());
    }

    #[test]
    fn test_environment_with_variables() {
        let mut vars = HashMap::new();
        vars.insert("baseUrl".to_string(), "http://localhost:3000".to_string());
        vars.insert("apiKey".to_string(), "dev-key-123".to_string());

        let env = Environment::with_variables("dev", vars);
        assert_eq!(env.name, "dev");
        assert_eq!(env.len(), 2);
        assert_eq!(env.get("baseUrl").unwrap(), "http://localhost:3000");
    }

    #[test]
    fn test_environment_set_get() {
        let mut env = Environment::new("test");
        env.set("key1", "value1");
        env.set("key2", "value2");

        assert_eq!(env.get("key1").unwrap(), "value1");
        assert_eq!(env.get("key2").unwrap(), "value2");
        assert!(env.get("nonexistent").is_none());
    }

    #[test]
    fn test_environment_contains() {
        let mut env = Environment::new("test");
        env.set("existing", "value");

        assert!(env.contains("existing"));
        assert!(!env.contains("missing"));
    }

    #[test]
    fn test_environments_new() {
        let envs = Environments::new();
        assert!(envs.environments.is_empty());
        assert!(envs.shared.is_empty());
        assert!(envs.active.is_none());
    }

    #[test]
    fn test_environments_add_get() {
        let mut envs = Environments::new();
        let dev = Environment::with_variables(
            "dev",
            [("url".to_string(), "http://dev".to_string())]
                .into_iter()
                .collect(),
        );

        envs.add_environment(dev.clone());
        assert_eq!(envs.get_environment("dev").unwrap(), &dev);
    }

    #[test]
    fn test_environments_set_active() {
        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        envs.add_environment(Environment::new("prod"));

        assert!(envs.set_active("dev"));
        assert_eq!(envs.active.as_ref().unwrap(), "dev");

        assert!(!envs.set_active("nonexistent"));
        assert_eq!(envs.active.as_ref().unwrap(), "dev"); // Should remain unchanged
    }

    #[test]
    fn test_environments_get_active() {
        let mut envs = Environments::new();
        let dev = Environment::new("dev");
        envs.add_environment(dev.clone());
        envs.set_active("dev");

        assert_eq!(envs.get_active().unwrap(), &dev);
    }

    #[test]
    fn test_environments_variable_precedence() {
        let mut envs = Environments::new();

        // Set shared variable
        envs.set_shared("url", "http://shared");
        envs.set_shared("shared_only", "shared_value");

        // Create dev environment that overrides 'url'
        let mut dev = Environment::new("dev");
        dev.set("url", "http://dev");
        dev.set("dev_only", "dev_value");
        envs.add_environment(dev);

        envs.set_active("dev");

        // Environment variable should take precedence
        assert_eq!(envs.get_variable("url").unwrap(), "http://dev");

        // Shared variable should be accessible
        assert_eq!(envs.get_variable("shared_only").unwrap(), "shared_value");

        // Environment-specific variable should be accessible
        assert_eq!(envs.get_variable("dev_only").unwrap(), "dev_value");
    }

    #[test]
    fn test_environments_get_merged_variables() {
        let mut envs = Environments::new();

        envs.set_shared("shared1", "s1");
        envs.set_shared("shared2", "s2");
        envs.set_shared("override_me", "shared_value");

        let mut dev = Environment::new("dev");
        dev.set("env1", "e1");
        dev.set("override_me", "env_value");
        envs.add_environment(dev);

        envs.set_active("dev");

        let merged = envs.get_merged_variables();
        assert_eq!(merged.len(), 4);
        assert_eq!(merged.get("shared1").unwrap(), "s1");
        assert_eq!(merged.get("shared2").unwrap(), "s2");
        assert_eq!(merged.get("env1").unwrap(), "e1");
        assert_eq!(merged.get("override_me").unwrap(), "env_value"); // Env takes precedence
    }

    #[test]
    fn test_environments_no_active() {
        let mut envs = Environments::new();
        envs.set_shared("shared", "value");

        // No active environment, should only get shared variables
        assert_eq!(envs.get_variable("shared").unwrap(), "value");
        assert!(envs.get_active().is_none());

        let merged = envs.get_merged_variables();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged.get("shared").unwrap(), "value");
    }

    #[test]
    fn test_environments_list() {
        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        envs.add_environment(Environment::new("staging"));
        envs.add_environment(Environment::new("prod"));

        let mut names = envs.list_environments();
        names.sort();

        assert_eq!(names, vec!["dev", "prod", "staging"]);
    }

    #[test]
    fn test_environments_serialization() {
        let mut envs = Environments::new();

        let mut dev = Environment::new("dev");
        dev.set("url", "http://dev");
        envs.add_environment(dev);

        envs.set_shared("api_version", "v1");
        envs.set_active("dev");

        // Serialize to JSON
        let json = serde_json::to_string(&envs).unwrap();
        assert!(json.contains("dev"));
        assert!(json.contains("api_version"));

        // Deserialize back
        let deserialized: Environments = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, envs);
    }

    #[test]
    fn test_environment_is_empty() {
        let env = Environment::new("test");
        assert!(env.is_empty());

        let mut env = Environment::new("test");
        env.set("key", "value");
        assert!(!env.is_empty());
    }

    #[test]
    fn test_environments_is_empty() {
        let envs = Environments::new();
        assert!(envs.is_empty());

        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        assert!(!envs.is_empty());
    }
}
