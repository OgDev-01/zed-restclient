//! Variables module for REST Client
//!
//! This module provides variable resolution capabilities for HTTP requests,
//! including system variables, environment variables, and request variables.

pub mod capture;
pub mod environment;
pub mod request;
pub mod substitution;
pub mod system;

pub use capture::{parse_capture_directive, parse_capture_directives, CaptureDirective, PathType};
pub use environment::{resolve_environment_variable, resolve_with_fallback};
pub use request::{extract_response_variable, ContentType};
pub use substitution::{substitute_variables, VariableContext};
pub use system::{clear_dotenv_cache, resolve_system_variable, VarError};
