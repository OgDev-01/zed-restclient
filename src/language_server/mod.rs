//! Language Server Protocol (LSP) features for REST Client
//!
//! This module provides LSP-like features for .http files, including:
//! - Variable autocompletion (triggered by `{{`)
//! - Hover tooltips showing variable values
//! - Real-time diagnostics for syntax errors, undefined variables, and validation
//! - CodeLens for clickable "Send Request" actions above each request
//!
//! These are helper functions designed to be integrated into a full LSP server later.

pub mod codelens;
pub mod completion;
pub mod diagnostics;
pub mod hover;

pub use codelens::{provide_code_lens, CodeLens, Command};
pub use completion::{provide_completions, CompletionItem, CompletionKind};
pub use diagnostics::{provide_diagnostics, Diagnostic, DiagnosticSeverity, Position, Range};
pub use hover::{provide_hover, Hover};
