//! Integration tests entry point for REST Client
//!
//! This file serves as the main entry point for integration tests,
//! allowing Rust's test framework to discover and run tests in the
//! integration subdirectory.

mod integration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_tests_module_loads() {
        // This ensures the integration module loads correctly
        integration::init_test_env();
    }
}
