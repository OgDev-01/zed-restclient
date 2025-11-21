//! Data models for HTTP requests and responses.
//!
//! This module contains the core data structures used throughout the REST Client extension
//! for representing HTTP requests, responses, and related metadata.

pub mod request;
pub mod response;

pub use request::{HttpMethod, HttpRequest};
pub use response::{HttpResponse, RequestTiming};
