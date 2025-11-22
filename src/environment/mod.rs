//! Environment management module for REST Client
//!
//! This module provides functionality for loading and managing environment configurations
//! from .http-client-env.json or http-client.env.json files. Environments allow users to
//! define different sets of variables for different contexts (dev, staging, production)
//! and easily switch between them.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use rest_client::environment::{load_environments, Environment, Environments, EnvironmentSession};
//!
//! let workspace = Path::new("/path/to/workspace");
//! let envs = load_environments(workspace).unwrap();
//!
//! // Create a session to manage active environment
//! let mut session = EnvironmentSession::new(envs);
//! session.set_active_environment("dev").ok();
//!
//! // Get a variable from the active environment
//! if let Some(url) = session.get_variable("baseUrl") {
//!     println!("Base URL: {}", url);
//! }
//! ```

pub mod loader;
pub mod models;

use std::sync::{Arc, RwLock};

// Re-export public types for convenience
pub use loader::{load_environments, EnvError};
pub use models::{Environment, Environments};

/// Session manager for environment variables
///
/// This struct maintains the state of loaded environments and the currently active
/// environment across request executions. It uses thread-safe interior mutability
/// to allow the active environment to be changed without requiring mutable access.
#[derive(Debug, Clone)]
pub struct EnvironmentSession {
    /// The loaded environments (wrapped in Arc<RwLock> for thread-safe shared access)
    environments: Arc<RwLock<Environments>>,
}

impl EnvironmentSession {
    /// Creates a new environment session with the given environments
    pub fn new(environments: Environments) -> Self {
        Self {
            environments: Arc::new(RwLock::new(environments)),
        }
    }

    /// Gets the currently active environment
    ///
    /// # Returns
    ///
    /// A clone of the active environment, or None if no environment is active
    pub fn get_active_environment(&self) -> Option<Environment> {
        self.environments
            .read()
            .ok()
            .and_then(|envs| envs.get_active().cloned())
    }

    /// Sets the active environment by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the environment to activate
    ///
    /// # Returns
    ///
    /// Ok(()) if the environment was successfully activated,
    /// Err(EnvError) if the environment doesn't exist
    ///
    /// # Example
    ///
    /// ```no_run
    /// use rest_client::environment::{Environments, EnvironmentSession, Environment};
    ///
    /// let mut envs = Environments::new();
    /// envs.add_environment(Environment::new("dev"));
    ///
    /// let session = EnvironmentSession::new(envs);
    /// session.set_active_environment("dev").unwrap();
    /// ```
    pub fn set_active_environment(&self, name: &str) -> Result<(), EnvError> {
        let mut envs = self
            .environments
            .write()
            .map_err(|_| EnvError::InvalidFormat("Failed to acquire write lock".to_string()))?;

        if envs.set_active(name) {
            Ok(())
        } else {
            Err(EnvError::InvalidFormat(format!(
                "Environment '{}' not found",
                name
            )))
        }
    }

    /// Reloads all environments from a new Environments struct
    ///
    /// This replaces the entire environment configuration, useful for
    /// reloading from file without requiring LSP restart.
    ///
    /// # Arguments
    ///
    /// * `new_environments` - The new environments to load
    ///
    /// # Returns
    ///
    /// Ok(()) if environments were successfully reloaded,
    /// Err(EnvError) if failed to acquire write lock
    pub fn reload_environments(&self, new_environments: Environments) -> Result<(), EnvError> {
        let mut envs = self
            .environments
            .write()
            .map_err(|_| EnvError::InvalidFormat("Failed to acquire write lock".to_string()))?;

        *envs = new_environments;
        Ok(())
    }

    /// Gets a variable value from the active environment or shared variables
    ///
    /// This method follows the precedence:
    /// 1. Active environment variables (if an environment is active)
    /// 2. Shared variables
    ///
    /// # Arguments
    ///
    /// * `name` - The variable name to resolve
    ///
    /// # Returns
    ///
    /// The resolved variable value, or None if not found
    pub fn get_variable(&self, name: &str) -> Option<String> {
        self.environments
            .read()
            .ok()
            .and_then(|envs| envs.get_variable(name))
    }

    /// Gets all environments
    pub fn get_environments(&self) -> Option<Environments> {
        self.environments.read().ok().map(|envs| envs.clone())
    }

    /// Lists all available environment names
    pub fn list_environment_names(&self) -> Vec<String> {
        self.environments
            .read()
            .ok()
            .map(|envs| envs.list_environments())
            .unwrap_or_default()
    }

    /// Gets the name of the currently active environment
    pub fn get_active_environment_name(&self) -> Option<String> {
        self.environments
            .read()
            .ok()
            .and_then(|envs| envs.active.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_session_new() {
        let envs = Environments::new();
        let session = EnvironmentSession::new(envs);
        assert!(session.get_active_environment().is_none());
    }

    #[test]
    fn test_environment_session_set_get_active() {
        let mut envs = Environments::new();
        let mut dev = Environment::new("dev");
        dev.set("url", "http://dev.example.com");
        envs.add_environment(dev);

        let session = EnvironmentSession::new(envs);
        session.set_active_environment("dev").unwrap();

        let active = session.get_active_environment();
        assert!(active.is_some());
        assert_eq!(active.unwrap().name, "dev");
    }

    #[test]
    fn test_environment_session_set_invalid_environment() {
        let envs = Environments::new();
        let session = EnvironmentSession::new(envs);

        let result = session.set_active_environment("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_environment_session_get_variable() {
        let mut envs = Environments::new();
        envs.set_shared("sharedVar", "shared value");

        let mut dev = Environment::new("dev");
        dev.set("devVar", "dev value");
        dev.set("override", "from dev");
        envs.add_environment(dev);

        envs.set_shared("override", "from shared");

        let session = EnvironmentSession::new(envs);
        session.set_active_environment("dev").unwrap();

        // From active environment
        assert_eq!(
            session.get_variable("devVar"),
            Some("dev value".to_string())
        );

        // Environment overrides shared
        assert_eq!(
            session.get_variable("override"),
            Some("from dev".to_string())
        );

        // From shared
        assert_eq!(
            session.get_variable("sharedVar"),
            Some("shared value".to_string())
        );

        // Not found
        assert_eq!(session.get_variable("missing"), None);
    }

    #[test]
    fn test_environment_session_no_active_environment() {
        let mut envs = Environments::new();
        envs.set_shared("sharedVar", "shared value");

        let session = EnvironmentSession::new(envs);

        // Should only get shared variables when no environment is active
        assert_eq!(
            session.get_variable("sharedVar"),
            Some("shared value".to_string())
        );
        assert!(session.get_active_environment().is_none());
    }

    #[test]
    fn test_environment_session_list_environments() {
        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        envs.add_environment(Environment::new("staging"));
        envs.add_environment(Environment::new("prod"));

        let session = EnvironmentSession::new(envs);

        let mut names = session.list_environment_names();
        names.sort();

        assert_eq!(names, vec!["dev", "prod", "staging"]);
    }

    #[test]
    fn test_environment_session_get_active_name() {
        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));

        let session = EnvironmentSession::new(envs);
        assert!(session.get_active_environment_name().is_none());

        session.set_active_environment("dev").unwrap();
        assert_eq!(
            session.get_active_environment_name(),
            Some("dev".to_string())
        );
    }

    #[test]
    fn test_environment_session_thread_safety() {
        use std::thread;

        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));
        envs.add_environment(Environment::new("prod"));

        let session = EnvironmentSession::new(envs);
        let session_clone = session.clone();

        let handle = thread::spawn(move || {
            session_clone.set_active_environment("prod").unwrap();
            session_clone.get_active_environment_name()
        });

        session.set_active_environment("dev").ok();

        let result = handle.join().unwrap();
        // Either thread's set could win, but it should be one of them
        let final_active = session.get_active_environment_name();
        assert!(
            final_active == Some("dev".to_string()) || final_active == Some("prod".to_string())
        );
    }

    #[test]
    fn test_environment_session_reload() {
        let mut envs = Environments::new();
        envs.add_environment(Environment::new("dev"));

        let session = EnvironmentSession::new(envs);
        session.set_active_environment("dev").unwrap();

        assert_eq!(
            session.get_active_environment_name(),
            Some("dev".to_string())
        );

        // Reload with new environments
        let mut new_envs = Environments::new();
        new_envs.add_environment(Environment::new("staging"));
        new_envs.add_environment(Environment::new("prod"));

        session.reload_environments(new_envs).unwrap();

        // Old environment should no longer be active
        assert!(session.get_active_environment_name().is_none());

        // Should be able to set new environments
        session.set_active_environment("prod").unwrap();
        assert_eq!(
            session.get_active_environment_name(),
            Some("prod".to_string())
        );
    }
}
