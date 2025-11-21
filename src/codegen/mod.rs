//! Code generation module for HTTP requests.
//!
//! This module provides functionality to generate executable HTTP client code
//! in various programming languages from parsed HTTP requests. It supports
//! multiple languages and libraries, allowing users to convert their .http
//! files into runnable code snippets.

pub mod javascript;
pub mod python;
pub mod ui;

use crate::models::request::HttpRequest;
use std::fmt;

/// Supported programming languages for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// JavaScript/TypeScript
    JavaScript,
    /// Python
    Python,
    /// Rust (future support)
    Rust,
}

impl Language {
    /// Returns the string representation of the language.
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Rust => "Rust",
        }
    }

    /// Returns all available languages.
    pub fn all() -> Vec<Language> {
        vec![Language::JavaScript, Language::Python]
    }

    /// Returns the default library for this language.
    pub fn default_library(&self) -> Library {
        match self {
            Language::JavaScript => Library::Fetch,
            Language::Python => Library::Requests,
            Language::Rust => Library::Reqwest,
        }
    }

    /// Returns all available libraries for this language.
    pub fn available_libraries(&self) -> Vec<Library> {
        match self {
            Language::JavaScript => vec![Library::Fetch, Library::Axios],
            Language::Python => vec![Library::Requests, Library::Urllib],
            Language::Rust => vec![Library::Reqwest],
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Supported HTTP client libraries for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Library {
    /// JavaScript fetch() API (browser/Node.js 18+)
    Fetch,
    /// Axios library for JavaScript
    Axios,
    /// Python requests library
    Requests,
    /// Python urllib (standard library)
    Urllib,
    /// Rust reqwest library (future)
    Reqwest,
}

impl Library {
    /// Returns the string representation of the library.
    pub fn as_str(&self) -> &'static str {
        match self {
            Library::Fetch => "fetch",
            Library::Axios => "axios",
            Library::Requests => "requests",
            Library::Urllib => "urllib",
            Library::Reqwest => "reqwest",
        }
    }

    /// Returns the language this library belongs to.
    pub fn language(&self) -> Language {
        match self {
            Library::Fetch | Library::Axios => Language::JavaScript,
            Library::Requests | Library::Urllib => Language::Python,
            Library::Reqwest => Language::Rust,
        }
    }

    /// Returns a description of the library.
    pub fn description(&self) -> &'static str {
        match self {
            Library::Fetch => "Modern browser fetch() API (no dependencies)",
            Library::Axios => "Popular promise-based HTTP client",
            Library::Requests => "Simple and elegant HTTP library",
            Library::Urllib => "Python standard library (no dependencies)",
            Library::Reqwest => "Ergonomic async HTTP client",
        }
    }
}

impl fmt::Display for Library {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Errors that can occur during code generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeGenError {
    /// The requested language is not yet supported
    UnsupportedLanguage(String),
    /// The requested library is not yet supported
    UnsupportedLibrary(String),
    /// The library is not compatible with the language
    IncompatibleLibrary { language: String, library: String },
    /// The request is invalid or missing required fields
    InvalidRequest(String),
}

impl fmt::Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodeGenError::UnsupportedLanguage(lang) => {
                write!(f, "Language '{}' is not yet supported", lang)
            }
            CodeGenError::UnsupportedLibrary(lib) => {
                write!(f, "Library '{}' is not yet supported", lib)
            }
            CodeGenError::IncompatibleLibrary { language, library } => {
                write!(
                    f,
                    "Library '{}' is not compatible with language '{}'",
                    library, language
                )
            }
            CodeGenError::InvalidRequest(msg) => {
                write!(f, "Invalid request: {}", msg)
            }
        }
    }
}

impl std::error::Error for CodeGenError {}

/// Generates HTTP client code for the given request, language, and library.
///
/// This is the main entry point for code generation. It dispatches to the
/// appropriate language-specific generator based on the parameters.
///
/// # Arguments
///
/// * `request` - The HTTP request to generate code for
/// * `language` - The target programming language
/// * `library` - Optional specific library to use (defaults to language's default)
///
/// # Returns
///
/// A `Result` containing the generated code string or a `CodeGenError`
///
/// # Examples
///
/// ```
/// use rest_client::models::request::{HttpRequest, HttpMethod};
/// use rest_client::codegen::{generate_code, Language, Library};
///
/// let request = HttpRequest::new(
///     "test".to_string(),
///     HttpMethod::GET,
///     "https://api.example.com/users".to_string(),
/// );
///
/// // Generate JavaScript fetch code
/// let code = generate_code(&request, Language::JavaScript, None).unwrap();
///
/// // Generate Python requests code
/// let code = generate_code(&request, Language::Python, Some(Library::Requests)).unwrap();
/// ```
pub fn generate_code(
    request: &HttpRequest,
    language: Language,
    library: Option<Library>,
) -> Result<String, CodeGenError> {
    // Validate request
    if request.url.is_empty() {
        return Err(CodeGenError::InvalidRequest(
            "Request URL is empty".to_string(),
        ));
    }

    // Determine which library to use
    let lib = library.unwrap_or_else(|| language.default_library());

    // Validate that the library is compatible with the language
    if lib.language() != language {
        return Err(CodeGenError::IncompatibleLibrary {
            language: language.as_str().to_string(),
            library: lib.as_str().to_string(),
        });
    }

    // Dispatch to the appropriate generator
    match (language, lib) {
        (Language::JavaScript, Library::Fetch) => Ok(javascript::generate_fetch_code(request)),
        (Language::JavaScript, Library::Axios) => Ok(javascript::generate_axios_code(request)),
        (Language::Python, Library::Requests) => Ok(python::generate_requests_code(request)),
        (Language::Python, Library::Urllib) => Ok(python::generate_urllib_code(request)),
        (Language::Rust, Library::Reqwest) => Err(CodeGenError::UnsupportedLanguage(
            "Rust support coming soon".to_string(),
        )),
        _ => Err(CodeGenError::IncompatibleLibrary {
            language: language.as_str().to_string(),
            library: lib.as_str().to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::HttpMethod;

    #[test]
    fn test_language_as_str() {
        assert_eq!(Language::JavaScript.as_str(), "JavaScript");
        assert_eq!(Language::Python.as_str(), "Python");
        assert_eq!(Language::Rust.as_str(), "Rust");
    }

    #[test]
    fn test_language_default_library() {
        assert_eq!(Language::JavaScript.default_library(), Library::Fetch);
        assert_eq!(Language::Python.default_library(), Library::Requests);
        assert_eq!(Language::Rust.default_library(), Library::Reqwest);
    }

    #[test]
    fn test_language_available_libraries() {
        let js_libs = Language::JavaScript.available_libraries();
        assert_eq!(js_libs.len(), 2);
        assert!(js_libs.contains(&Library::Fetch));
        assert!(js_libs.contains(&Library::Axios));

        let py_libs = Language::Python.available_libraries();
        assert_eq!(py_libs.len(), 2);
        assert!(py_libs.contains(&Library::Requests));
        assert!(py_libs.contains(&Library::Urllib));
    }

    #[test]
    fn test_library_as_str() {
        assert_eq!(Library::Fetch.as_str(), "fetch");
        assert_eq!(Library::Axios.as_str(), "axios");
        assert_eq!(Library::Requests.as_str(), "requests");
        assert_eq!(Library::Urllib.as_str(), "urllib");
    }

    #[test]
    fn test_library_language() {
        assert_eq!(Library::Fetch.language(), Language::JavaScript);
        assert_eq!(Library::Axios.language(), Language::JavaScript);
        assert_eq!(Library::Requests.language(), Language::Python);
        assert_eq!(Library::Urllib.language(), Language::Python);
    }

    #[test]
    fn test_generate_code_javascript_fetch() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/users".to_string(),
        );

        let code = generate_code(&request, Language::JavaScript, None).unwrap();
        assert!(code.contains("fetch"));
        assert!(code.contains("async function"));
    }

    #[test]
    fn test_generate_code_javascript_axios() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::POST,
            "https://api.example.com/users".to_string(),
        );

        let code = generate_code(&request, Language::JavaScript, Some(Library::Axios)).unwrap();
        assert!(code.contains("axios"));
        assert!(code.contains("require('axios')"));
    }

    #[test]
    fn test_generate_code_python_requests() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://api.example.com/data".to_string(),
        );

        let code = generate_code(&request, Language::Python, None).unwrap();
        assert!(code.contains("import requests"));
        assert!(code.contains("requests.get"));
    }

    #[test]
    fn test_generate_code_python_urllib() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::PUT,
            "https://api.example.com/update".to_string(),
        );

        let code = generate_code(&request, Language::Python, Some(Library::Urllib)).unwrap();
        assert!(code.contains("import urllib.request"));
        assert!(code.contains("urllib.request.urlopen"));
    }

    #[test]
    fn test_generate_code_invalid_request() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "".to_string(), // Empty URL
        );

        let result = generate_code(&request, Language::JavaScript, None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodeGenError::InvalidRequest(_)
        ));
    }

    #[test]
    fn test_generate_code_incompatible_library() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://example.com".to_string(),
        );

        // Try to use Python library with JavaScript language
        let result = generate_code(&request, Language::JavaScript, Some(Library::Requests));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodeGenError::IncompatibleLibrary { .. }
        ));
    }

    #[test]
    fn test_generate_code_rust_not_supported() {
        let request = HttpRequest::new(
            "test".to_string(),
            HttpMethod::GET,
            "https://example.com".to_string(),
        );

        let result = generate_code(&request, Language::Rust, None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CodeGenError::UnsupportedLanguage(_)
        ));
    }

    #[test]
    fn test_error_display() {
        let err = CodeGenError::UnsupportedLanguage("Go".to_string());
        assert_eq!(format!("{}", err), "Language 'Go' is not yet supported");

        let err = CodeGenError::IncompatibleLibrary {
            language: "JavaScript".to_string(),
            library: "requests".to_string(),
        };
        assert_eq!(
            format!("{}", err),
            "Library 'requests' is not compatible with language 'JavaScript'"
        );
    }
}
