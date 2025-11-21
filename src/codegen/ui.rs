//! Code generation UI and command handling.
//!
//! This module provides the command interface for generating code from HTTP requests.
//! It handles language and library selection, code generation, and result formatting
//! for display in Zed.

use crate::codegen::{generate_code, Language, Library};
use crate::models::request::HttpRequest;

/// Result of a code generation command.
#[derive(Debug)]
pub struct CodeGenerationResult {
    /// Success status of the operation.
    pub success: bool,

    /// Message describing the result (for notifications).
    pub message: String,

    /// The generated code (if successful).
    pub generated_code: Option<String>,

    /// The language used for generation.
    pub language: Option<Language>,

    /// The library used for generation.
    pub library: Option<Library>,

    /// The original request that was used.
    pub request: Option<HttpRequest>,
}

impl CodeGenerationResult {
    /// Creates a successful code generation result.
    pub fn success(
        code: String,
        language: Language,
        library: Library,
        request: HttpRequest,
    ) -> Self {
        Self {
            success: true,
            message: format!(
                "Generated {} code using {}",
                language.as_str(),
                library.as_str()
            ),
            generated_code: Some(code),
            language: Some(language),
            library: Some(library),
            request: Some(request),
        }
    }

    /// Creates a failed code generation result.
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            generated_code: None,
            language: None,
            library: None,
            request: None,
        }
    }

    /// Formats the result for display in Zed.
    ///
    /// Returns a string that includes the generated code with metadata header.
    pub fn to_display_string(&self) -> String {
        if !self.success {
            return format!("❌ Code Generation Failed\n\n{}", self.message);
        }

        let code = self.generated_code.as_ref().unwrap();
        let language = self.language.as_ref().unwrap();
        let library = self.library.as_ref().unwrap();
        let request = self.request.as_ref().unwrap();

        let mut output = String::new();

        // Header with metadata
        output.push_str(&format!(
            "# Generated {} Code ({})\n",
            language.as_str(),
            library.as_str()
        ));
        output.push_str(&format!("# Request: {} {}\n", request.method, request.url));
        output.push_str("# \n");
        output.push_str("# This code has been automatically generated from your HTTP request.\n");
        output.push_str("# You can copy and paste it into your project.\n");
        output.push_str(&format!(
            "# Generated code is ready to run with {} library.\n",
            library.as_str()
        ));
        output.push_str("#\n");
        output.push_str("# Instructions:\n");

        match language {
            Language::JavaScript => match library {
                Library::Fetch => {
                    output.push_str("# - Copy this code into a .js or .ts file\n");
                    output.push_str(
                        "# - This code uses native fetch API (no dependencies required)\n",
                    );
                    output.push_str("# - Run in browser or Node.js 18+\n");
                }
                Library::Axios => {
                    output.push_str("# - Install axios: npm install axios\n");
                    output.push_str("# - Copy this code into a .js or .ts file\n");
                    output.push_str("# - Run with: node your-file.js\n");
                }
                _ => {}
            },
            Language::Python => match library {
                Library::Requests => {
                    output.push_str("# - Install requests: pip install requests\n");
                    output.push_str("# - Copy this code into a .py file\n");
                    output.push_str("# - Run with: python your-file.py\n");
                }
                Library::Urllib => {
                    output.push_str("# - No installation required (standard library)\n");
                    output.push_str("# - Copy this code into a .py file\n");
                    output.push_str("# - Run with: python your-file.py\n");
                }
                _ => {}
            },
            _ => {}
        }

        output.push_str("#\n");
        output.push_str("# Code copied to clipboard! ✓\n");
        output.push_str("\n");

        // The actual generated code
        output.push_str(code);

        output
    }
}

/// Generates code for a given request with specified language and library.
///
/// This is the main command function that orchestrates code generation.
///
/// # Arguments
///
/// * `request` - The HTTP request to generate code for
/// * `language` - The target programming language
/// * `library` - The HTTP client library to use (optional, uses default if None)
///
/// # Returns
///
/// A `CodeGenerationResult` containing the generated code or an error message.
pub fn generate_code_command(
    request: &HttpRequest,
    language: Language,
    library: Option<Library>,
) -> CodeGenerationResult {
    // Use default library if not specified
    let selected_library = library.unwrap_or_else(|| language.default_library());

    // Validate library is compatible with language
    if selected_library.language() != language {
        return CodeGenerationResult::failure(format!(
            "Library {} is not compatible with {}",
            selected_library.as_str(),
            language.as_str()
        ));
    }

    // Generate the code
    match generate_code(request, language, Some(selected_library)) {
        Ok(code) => {
            CodeGenerationResult::success(code, language, selected_library, request.clone())
        }
        Err(e) => CodeGenerationResult::failure(format!("Code generation failed: {}", e)),
    }
}

/// Lists available languages for code generation.
///
/// Returns a formatted string listing all supported languages.
pub fn list_available_languages() -> String {
    let languages = Language::all();
    let mut output = String::from("Available Languages for Code Generation:\n\n");

    for lang in languages {
        let libraries = lang.available_libraries();
        output.push_str(&format!(
            "• {} (default: {})\n",
            lang.as_str(),
            lang.default_library().as_str()
        ));
        output.push_str("  Libraries: ");
        let lib_names: Vec<String> = libraries.iter().map(|l| l.as_str().to_string()).collect();
        output.push_str(&lib_names.join(", "));
        output.push_str("\n");
    }

    output.push_str("\nUsage:\n");
    output.push_str("/generate-code <language> [library]\n\n");
    output.push_str("Examples:\n");
    output.push_str("  /generate-code javascript        # Uses fetch (default)\n");
    output.push_str("  /generate-code javascript axios  # Uses axios\n");
    output.push_str("  /generate-code python            # Uses requests (default)\n");
    output.push_str("  /generate-code python urllib     # Uses urllib\n");

    output
}

/// Parses language and library from command arguments.
///
/// # Arguments
///
/// * `args` - Command arguments (first is language, second is optional library)
///
/// # Returns
///
/// `Ok((Language, Option<Library>))` if parsing succeeds, `Err(String)` with error message if not.
pub fn parse_generation_options(args: &[String]) -> Result<(Language, Option<Library>), String> {
    if args.is_empty() {
        return Err("Language not specified. Use /generate-code <language> [library]".to_string());
    }

    let lang_str = args[0].trim().to_lowercase();

    // Parse language
    let language = match lang_str.as_str() {
        "javascript" | "js" => Language::JavaScript,
        "python" | "py" => Language::Python,
        "rust" | "rs" => Language::Rust,
        _ => {
            return Err(format!(
                "Unknown language '{}'. Available: javascript, python",
                args[0]
            ))
        }
    };

    // Parse library if provided
    let library = if args.len() > 1 {
        let lib_str = args[1].trim().to_lowercase();
        let lib = match lib_str.as_str() {
            "fetch" => Library::Fetch,
            "axios" => Library::Axios,
            "requests" => Library::Requests,
            "urllib" => Library::Urllib,
            "reqwest" => Library::Reqwest,
            _ => {
                return Err(format!(
                    "Unknown library '{}' for {}. Available: {}",
                    args[1],
                    language.as_str(),
                    language
                        .available_libraries()
                        .iter()
                        .map(|l| l.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            }
        };

        // Validate library is for this language
        if lib.language() != language {
            return Err(format!(
                "Library {} is not available for {}. Use one of: {}",
                lib.as_str(),
                language.as_str(),
                language
                    .available_libraries()
                    .iter()
                    .map(|l| l.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Some(lib)
    } else {
        None
    };

    Ok((language, library))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::request::{HttpMethod, HttpRequest};

    fn create_test_request() -> HttpRequest {
        HttpRequest {
            id: "test-request-1".to_string(),
            method: HttpMethod::GET,
            url: "https://api.example.com/users".to_string(),
            http_version: None,
            headers: vec![("Authorization".to_string(), "Bearer token123".to_string())]
                .into_iter()
                .collect(),
            body: None,
            file_path: std::path::PathBuf::from("test.http"),
            line_number: 1,
        }
    }

    #[test]
    fn test_parse_generation_options_javascript() {
        let args = vec!["javascript".to_string()];
        let result = parse_generation_options(&args);
        assert!(result.is_ok());
        let (lang, lib) = result.unwrap();
        assert_eq!(lang, Language::JavaScript);
        assert_eq!(lib, None);
    }

    #[test]
    fn test_parse_generation_options_javascript_with_axios() {
        let args = vec!["javascript".to_string(), "axios".to_string()];
        let result = parse_generation_options(&args);
        assert!(result.is_ok());
        let (lang, lib) = result.unwrap();
        assert_eq!(lang, Language::JavaScript);
        assert_eq!(lib, Some(Library::Axios));
    }

    #[test]
    fn test_parse_generation_options_python() {
        let args = vec!["python".to_string()];
        let result = parse_generation_options(&args);
        assert!(result.is_ok());
        let (lang, lib) = result.unwrap();
        assert_eq!(lang, Language::Python);
        assert_eq!(lib, None);
    }

    #[test]
    fn test_parse_generation_options_invalid_language() {
        let args = vec!["invalid".to_string()];
        let result = parse_generation_options(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_generation_options_incompatible_library() {
        let args = vec!["javascript".to_string(), "requests".to_string()];
        let result = parse_generation_options(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_generation_options_empty() {
        let args: Vec<String> = vec![];
        let result = parse_generation_options(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_code_command_success() {
        let request = create_test_request();
        let result = generate_code_command(&request, Language::JavaScript, None);
        assert!(result.success);
        assert!(result.generated_code.is_some());
        assert_eq!(result.language, Some(Language::JavaScript));
        assert_eq!(result.library, Some(Library::Fetch));
    }

    #[test]
    fn test_generate_code_command_with_library() {
        let request = create_test_request();
        let result = generate_code_command(&request, Language::Python, Some(Library::Requests));
        assert!(result.success);
        assert!(result.generated_code.is_some());
        assert_eq!(result.language, Some(Language::Python));
        assert_eq!(result.library, Some(Library::Requests));
    }

    #[test]
    fn test_generate_code_command_incompatible_library() {
        let request = create_test_request();
        let result = generate_code_command(&request, Language::JavaScript, Some(Library::Requests));
        assert!(!result.success);
        assert!(result.generated_code.is_none());
    }

    #[test]
    fn test_code_generation_result_display() {
        let request = create_test_request();
        let result = generate_code_command(&request, Language::JavaScript, None);
        let display = result.to_display_string();
        assert!(display.contains("Generated JavaScript Code"));
        assert!(display.contains("fetch"));
        assert!(display.contains("GET https://api.example.com/users"));
    }

    #[test]
    fn test_list_available_languages() {
        let list = list_available_languages();
        assert!(list.contains("JavaScript"));
        assert!(list.contains("Python"));
        assert!(list.contains("fetch"));
        assert!(list.contains("axios"));
        assert!(list.contains("requests"));
    }
}
