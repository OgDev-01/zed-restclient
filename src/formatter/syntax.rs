//! Syntax highlighting support for formatted responses.
//!
//! This module provides syntax highlighting capabilities for JSON, XML, and HTML.
//! Since Zed extensions run in WASM without access to syntect, we use a simple
//! marker-based system that can be interpreted by the Zed editor.

/// Language types supported for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Language {
    /// JSON syntax
    Json,
    /// XML syntax
    Xml,
    /// HTML syntax
    Html,
    /// Plain text (no highlighting)
    PlainText,
}

impl Language {
    /// Converts a string to a Language variant.
    ///
    /// # Arguments
    ///
    /// * `s` - Language name as string
    ///
    /// # Returns
    ///
    /// Corresponding Language variant, defaults to PlainText if unknown.
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Language::Json,
            "xml" => Language::Xml,
            "html" => Language::Html,
            _ => Language::PlainText,
        }
    }

    /// Returns the file extension for this language.
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Json => "json",
            Language::Xml => "xml",
            Language::Html => "html",
            Language::PlainText => "txt",
        }
    }

    /// Returns the MIME type for this language.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Language::Json => "application/json",
            Language::Xml => "application/xml",
            Language::Html => "text/html",
            Language::PlainText => "text/plain",
        }
    }
}

/// Applies syntax highlighting to text.
///
/// Since we're in a WASM environment without access to full syntax highlighting
/// libraries, this function returns the text with language metadata that Zed
/// can use for its own syntax highlighting.
///
/// # Arguments
///
/// * `text` - Text to highlight
/// * `language` - Programming language for syntax highlighting
///
/// # Returns
///
/// The text string. In a full implementation, this would include highlighting
/// markers or ANSI color codes, but for Zed integration, the text is returned
/// as-is and Zed's built-in highlighter handles it.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::syntax::apply_syntax_highlighting;
///
/// let json = r#"{"name": "value"}"#;
/// let highlighted = apply_syntax_highlighting(json, "json");
/// assert_eq!(highlighted, json);
/// ```
pub fn apply_syntax_highlighting(text: &str, language: &str) -> String {
    // In a WASM environment, we don't have access to syntect or similar libraries.
    // Instead, we return the plain text and let Zed handle the syntax highlighting
    // based on the language metadata we provide in the response.
    //
    // If Zed provides a syntax highlighting API in the future, we can integrate it here.
    text.to_string()
}

/// Attempts to apply syntax highlighting, with fallback to plain text.
///
/// This is a safe wrapper around `apply_syntax_highlighting` that never fails.
///
/// # Arguments
///
/// * `text` - Text to highlight
/// * `language` - Programming language for syntax highlighting
///
/// # Returns
///
/// Highlighted text, or original text if highlighting fails.
pub fn highlight_safe(text: &str, language: &str) -> String {
    apply_syntax_highlighting(text, language)
}

/// Detects the language from content.
///
/// Attempts to auto-detect the programming language based on content patterns.
///
/// # Arguments
///
/// * `text` - Text content to analyze
///
/// # Returns
///
/// Detected Language variant.
pub fn detect_language(text: &str) -> Language {
    let trimmed = text.trim();

    // Check for JSON
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        // Try to parse as JSON to confirm
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return Language::Json;
        }
    }

    // Check for XML/HTML
    if trimmed.starts_with('<') {
        if trimmed.contains("<!DOCTYPE html") || trimmed.contains("<html") {
            return Language::Html;
        }
        if trimmed.contains("<?xml") || trimmed.contains("</") {
            return Language::Xml;
        }
        // Generic angle brackets could be XML or HTML
        return Language::Xml;
    }

    Language::PlainText
}

/// Wraps text with language metadata for Zed's syntax highlighter.
///
/// Creates a formatted output that includes language hints for proper
/// syntax highlighting in Zed's editor.
///
/// # Arguments
///
/// * `text` - Text content
/// * `language` - Programming language
///
/// # Returns
///
/// Text with language metadata that Zed can use for highlighting.
pub fn with_language_metadata(text: &str, language: Language) -> String {
    // For Zed integration, we could potentially use a format like:
    // ```language\ntext\n```
    // But for now, we just return the text as-is and rely on Zed's
    // content type detection and built-in syntax highlighting.
    text.to_string()
}

/// Response highlighting metadata.
///
/// Contains information about how to highlight a response, including
/// the language and whether highlighting is available.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HighlightInfo {
    /// Programming language
    pub language: Language,

    /// Whether syntax highlighting is available
    pub available: bool,

    /// File extension for this content type
    pub extension: String,

    /// MIME type for this content type
    pub mime_type: String,
}

impl HighlightInfo {
    /// Creates HighlightInfo for a given language.
    pub fn new(language: Language) -> Self {
        Self {
            language,
            available: language != Language::PlainText,
            extension: language.extension().to_string(),
            mime_type: language.mime_type().to_string(),
        }
    }

    /// Creates HighlightInfo from a language string.
    pub fn from_string(language: &str) -> Self {
        Self::new(Language::from_str(language))
    }

    /// Creates HighlightInfo by detecting language from content.
    pub fn detect(content: &str) -> Self {
        Self::new(detect_language(content))
    }
}

/// Formats text with syntax highlighting and returns both the text and metadata.
///
/// # Arguments
///
/// * `text` - Text to format and highlight
/// * `language` - Programming language
///
/// # Returns
///
/// Tuple of (highlighted_text, highlight_info).
pub fn format_with_highlighting(text: &str, language: &str) -> (String, HighlightInfo) {
    let lang = Language::from_str(language);
    let highlighted = apply_syntax_highlighting(text, language);
    let info = HighlightInfo::new(lang);

    (highlighted, info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!(Language::from_str("json"), Language::Json);
        assert_eq!(Language::from_str("JSON"), Language::Json);
        assert_eq!(Language::from_str("xml"), Language::Xml);
        assert_eq!(Language::from_str("html"), Language::Html);
        assert_eq!(Language::from_str("unknown"), Language::PlainText);
    }

    #[test]
    fn test_language_extension() {
        assert_eq!(Language::Json.extension(), "json");
        assert_eq!(Language::Xml.extension(), "xml");
        assert_eq!(Language::Html.extension(), "html");
        assert_eq!(Language::PlainText.extension(), "txt");
    }

    #[test]
    fn test_language_mime_type() {
        assert_eq!(Language::Json.mime_type(), "application/json");
        assert_eq!(Language::Xml.mime_type(), "application/xml");
        assert_eq!(Language::Html.mime_type(), "text/html");
        assert_eq!(Language::PlainText.mime_type(), "text/plain");
    }

    #[test]
    fn test_apply_syntax_highlighting() {
        let text = r#"{"name": "value"}"#;
        let result = apply_syntax_highlighting(text, "json");
        assert_eq!(result, text);
    }

    #[test]
    fn test_highlight_safe() {
        let text = r#"{"name": "value"}"#;
        let result = highlight_safe(text, "json");
        assert_eq!(result, text);
    }

    #[test]
    fn test_detect_language_json() {
        let json = r#"{"name": "value"}"#;
        assert_eq!(detect_language(json), Language::Json);

        let array = r#"[1, 2, 3]"#;
        assert_eq!(detect_language(array), Language::Json);
    }

    #[test]
    fn test_detect_language_xml() {
        let xml = r#"<?xml version="1.0"?><root/>"#;
        assert_eq!(detect_language(xml), Language::Xml);

        let simple = "<root><child/></root>";
        assert_eq!(detect_language(simple), Language::Xml);
    }

    #[test]
    fn test_detect_language_html() {
        let html = r#"<!DOCTYPE html><html><body></body></html>"#;
        assert_eq!(detect_language(html), Language::Html);

        let html2 = "<html><head></head></html>";
        assert_eq!(detect_language(html2), Language::Html);
    }

    #[test]
    fn test_detect_language_plain() {
        let text = "Just plain text";
        assert_eq!(detect_language(text), Language::PlainText);
    }

    #[test]
    fn test_highlight_info_new() {
        let info = HighlightInfo::new(Language::Json);
        assert_eq!(info.language, Language::Json);
        assert!(info.available);
        assert_eq!(info.extension, "json");
        assert_eq!(info.mime_type, "application/json");
    }

    #[test]
    fn test_highlight_info_from_string() {
        let info = HighlightInfo::from_string("xml");
        assert_eq!(info.language, Language::Xml);
        assert!(info.available);
    }

    #[test]
    fn test_highlight_info_detect() {
        let json = r#"{"test": true}"#;
        let info = HighlightInfo::detect(json);
        assert_eq!(info.language, Language::Json);
    }

    #[test]
    fn test_format_with_highlighting() {
        let text = r#"{"name": "value"}"#;
        let (highlighted, info) = format_with_highlighting(text, "json");

        assert_eq!(highlighted, text);
        assert_eq!(info.language, Language::Json);
        assert!(info.available);
    }

    #[test]
    fn test_with_language_metadata() {
        let text = "some content";
        let result = with_language_metadata(text, Language::Json);
        assert_eq!(result, text);
    }
}
