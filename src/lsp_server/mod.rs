//! LSP server module for REST Client extension
//!
//! This module provides the Language Server Protocol implementation
//! for .http files, enabling interactive features like code lenses,
//! autocompletion, diagnostics, and hover information.

// Module structure for LSP server components
// Submodules will be added as implementation progresses in subsequent tasks

pub mod backend;
pub mod document;
pub mod executor_bridge;

// Re-export main types for convenience
pub use backend::Backend;
pub use document::DocumentManager;
pub use executor_bridge::ExecutorBridge;
