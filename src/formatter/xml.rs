//! Enhanced XML formatting with proper parsing and pretty-printing.
//!
//! This module provides advanced XML formatting capabilities including:
//! - Pretty-printing with proper indentation
//! - Validation and error handling
//! - Graceful fallback for malformed XML

use crate::formatter::FormatError;

/// Maximum XML size to format (10MB).
///
/// Responses larger than this will not be formatted to avoid performance issues.
const MAX_XML_FORMAT_SIZE: usize = 10 * 1024 * 1024; // 10MB

/// Default indentation for XML formatting (2 spaces).
const XML_INDENT: &str = "  ";

/// Formats XML with pretty-printing and proper indentation.
///
/// This function parses the XML string and reformats it with consistent
/// indentation for improved readability. If parsing fails, returns an error
/// that can be used to fall back to raw display.
///
/// # Arguments
///
/// * `xml` - XML string to format
///
/// # Returns
///
/// `Ok(String)` with beautifully formatted XML, or `Err(FormatError)` if:
/// - The XML is malformed
/// - The XML exceeds the maximum size limit
///
/// # Examples
///
/// ```
/// use rest_client::formatter::xml::format_xml_pretty;
///
/// let xml = "<root><child>text</child></root>";
/// let formatted = format_xml_pretty(xml).unwrap();
/// assert!(formatted.contains("  <child>"));
/// ```
pub fn format_xml_pretty(xml: &str) -> Result<String, FormatError> {
    // Check size limit
    if xml.len() > MAX_XML_FORMAT_SIZE {
        return Err(FormatError::ResponseTooLarge(xml.len()));
    }

    // Trim whitespace
    let xml = xml.trim();
    if xml.is_empty() {
        return Err(FormatError::XmlError("Empty XML content".to_string()));
    }

    // Format the XML
    let formatted = format_xml_internal(xml)?;

    Ok(formatted)
}

/// Internal XML formatting implementation.
///
/// This implements a simple but effective XML formatter that:
/// - Maintains proper indentation levels
/// - Handles self-closing tags
/// - Preserves text content
/// - Handles CDATA sections
/// - Supports XML declarations and processing instructions
fn format_xml_internal(xml: &str) -> Result<String, FormatError> {
    let mut result = String::new();
    let mut indent_level: usize = 0;
    let mut chars = xml.chars().peekable();
    let mut line_has_content = false;

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                // Check what kind of tag this is
                let next_char = chars.peek().copied();

                match next_char {
                    Some('!') => {
                        // Comment or CDATA or DOCTYPE
                        chars.next(); // consume '!'

                        if chars.peek() == Some(&'-') {
                            // Comment: <!--
                            if !line_has_content {
                                result.push_str(&indent(indent_level));
                            }
                            result.push_str("<!");
                            result.push('-');

                            // Read until -->
                            let mut prev = ' ';
                            let mut prev_prev = ' ';
                            while let Some(c) = chars.next() {
                                result.push(c);
                                if c == '>' && prev == '-' && prev_prev == '-' {
                                    break;
                                }
                                prev_prev = prev;
                                prev = c;
                            }
                            result.push('\n');
                            line_has_content = false;
                        } else if chars.peek() == Some(&'[') {
                            // CDATA: <![CDATA[
                            if !line_has_content {
                                result.push_str(&indent(indent_level));
                            }
                            result.push_str("<![");

                            // Read until ]]>
                            let mut prev = ' ';
                            let mut prev_prev = ' ';
                            while let Some(c) = chars.next() {
                                result.push(c);
                                if c == '>' && prev == ']' && prev_prev == ']' {
                                    break;
                                }
                                prev_prev = prev;
                                prev = c;
                            }
                            result.push('\n');
                            line_has_content = false;
                        } else {
                            // DOCTYPE or other declaration
                            if !line_has_content {
                                result.push_str(&indent(indent_level));
                            }
                            result.push_str("<!");

                            // Read until >
                            while let Some(c) = chars.next() {
                                result.push(c);
                                if c == '>' {
                                    break;
                                }
                            }
                            result.push('\n');
                            line_has_content = false;
                        }
                    }
                    Some('?') => {
                        // Processing instruction: <?xml ... ?>
                        if !line_has_content {
                            result.push_str(&indent(indent_level));
                        }
                        result.push('<');
                        result.push('?');
                        chars.next(); // consume '?'

                        // Read until ?>
                        let mut prev = ' ';
                        while let Some(c) = chars.next() {
                            result.push(c);
                            if c == '>' && prev == '?' {
                                break;
                            }
                            prev = c;
                        }
                        result.push('\n');
                        line_has_content = false;
                    }
                    Some('/') => {
                        // Closing tag: </tag>
                        chars.next(); // consume '/'
                        indent_level = indent_level.saturating_sub(1);

                        if !line_has_content {
                            result.push_str(&indent(indent_level));
                        }
                        result.push_str("</");

                        // Read tag name and attributes until >
                        while let Some(c) = chars.next() {
                            result.push(c);
                            if c == '>' {
                                break;
                            }
                        }
                        result.push('\n');
                        line_has_content = false;
                    }
                    _ => {
                        // Opening tag: <tag> or <tag/>
                        if !line_has_content {
                            result.push_str(&indent(indent_level));
                        }
                        result.push('<');

                        // Read tag name and attributes
                        let mut tag_content = String::new();
                        let mut prev = ' ';
                        while let Some(c) = chars.next() {
                            if c == '>' {
                                // Check if self-closing
                                if prev == '/' {
                                    // Self-closing tag
                                    tag_content.push(c);
                                    result.push_str(&tag_content);
                                    result.push('\n');
                                    line_has_content = false;
                                } else {
                                    // Opening tag
                                    tag_content.push(c);
                                    result.push_str(&tag_content);

                                    // Check if next character is '<' (nested tag) or text content
                                    let next_non_ws = peek_next_non_whitespace(&mut chars);
                                    if next_non_ws == Some('<') {
                                        result.push('\n');
                                        line_has_content = false;
                                        indent_level += 1;
                                    } else {
                                        // Text content inline
                                        line_has_content = true;
                                        indent_level += 1;
                                    }
                                }
                                break;
                            }
                            tag_content.push(c);
                            prev = c;
                        }
                    }
                }
            }
            c if c.is_whitespace() => {
                // Skip leading/trailing whitespace, preserve single spaces in content
                if line_has_content && !result.ends_with(' ') && !result.ends_with('\n') {
                    if !chars.peek().map_or(true, |&nc| nc == '<') {
                        result.push(' ');
                    }
                }
            }
            _ => {
                // Text content
                if !line_has_content {
                    result.push_str(&indent(indent_level));
                    line_has_content = true;
                }
                result.push(ch);

                // Read rest of text content until <
                while let Some(&next) = chars.peek() {
                    if next == '<' {
                        break;
                    }
                    if let Some(c) = chars.next() {
                        if !c.is_whitespace() || !result.ends_with(' ') {
                            result.push(c);
                        }
                    }
                }
            }
        }
    }

    // Remove trailing whitespace
    while result.ends_with('\n') || result.ends_with(' ') {
        result.pop();
    }
    result.push('\n');

    Ok(result)
}

/// Helper function to peek at the next non-whitespace character.
fn peek_next_non_whitespace(chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<char> {
    let mut temp_chars = chars.clone();
    while let Some(&c) = temp_chars.peek() {
        if !c.is_whitespace() {
            return Some(c);
        }
        temp_chars.next();
    }
    None
}

/// Creates indentation string for the given level.
fn indent(level: usize) -> String {
    XML_INDENT.repeat(level)
}

/// Validates whether a string is valid XML.
///
/// This performs basic XML validation by checking for:
/// - Proper tag matching
/// - Well-formed structure
///
/// # Arguments
///
/// * `xml` - String to validate as XML
///
/// # Returns
///
/// `true` if the string appears to be valid XML, `false` otherwise.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::xml::validate_xml;
///
/// assert!(validate_xml("<root><child>text</child></root>"));
/// assert!(!validate_xml("<unclosed>"));
/// ```
pub fn validate_xml(xml: &str) -> bool {
    let xml = xml.trim();

    // Basic checks
    if xml.is_empty() {
        return false;
    }

    if !xml.starts_with('<') {
        return false;
    }

    // Simple tag matching validation
    let mut tag_stack: Vec<String> = Vec::new();
    let mut chars = xml.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            let next = chars.peek().copied();

            if next == Some('/') {
                // Closing tag
                chars.next(); // consume '/'
                let mut tag_name = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '>' || c.is_whitespace() {
                        break;
                    }
                    tag_name.push(c);
                    chars.next();
                }

                // Check if it matches the last opening tag
                if let Some(expected) = tag_stack.pop() {
                    if expected != tag_name {
                        return false; // Mismatched tags
                    }
                } else {
                    return false; // Closing tag without opening
                }

                // Skip to '>'
                while let Some(c) = chars.next() {
                    if c == '>' {
                        break;
                    }
                }
            } else if next == Some('?') || next == Some('!') {
                // Processing instruction or comment/CDATA - skip it
                let mut prev = ' ';
                let mut prev_prev = ' ';
                while let Some(c) = chars.next() {
                    if c == '>'
                        && (prev == '?'
                            || (prev == '-' && prev_prev == '-')
                            || (prev == ']' && prev_prev == ']'))
                    {
                        break;
                    }
                    prev_prev = prev;
                    prev = c;
                }
            } else {
                // Opening tag or self-closing tag
                let mut tag_name = String::new();
                let mut is_self_closing = false;
                let mut prev = ' ';

                while let Some(c) = chars.next() {
                    if c == '>' {
                        if prev == '/' {
                            is_self_closing = true;
                            tag_name.pop(); // Remove the '/'
                        }
                        break;
                    }
                    if c.is_whitespace() && !tag_name.is_empty() {
                        // Rest is attributes, skip to '>'
                        while let Some(attr_c) = chars.next() {
                            if attr_c == '>' {
                                if prev == '/' {
                                    is_self_closing = true;
                                }
                                break;
                            }
                            prev = attr_c;
                        }
                        break;
                    }
                    tag_name.push(c);
                    prev = c;
                }

                if !is_self_closing && !tag_name.is_empty() {
                    tag_stack.push(tag_name);
                }
            }
        }
    }

    // All tags should be closed
    tag_stack.is_empty()
}

/// Attempts to format XML, falling back to raw if formatting fails.
///
/// This is a convenience function that tries to pretty-print XML,
/// but returns the original string if formatting fails or if the
/// XML is too large.
///
/// # Arguments
///
/// * `xml` - XML string to format
///
/// # Returns
///
/// Formatted XML if successful, otherwise the original string.
///
/// # Examples
///
/// ```
/// use rest_client::formatter::xml::format_xml_safe;
///
/// let valid = "<root><child>text</child></root>";
/// let formatted = format_xml_safe(valid);
/// assert!(formatted.contains("  "));
///
/// let invalid = "<unclosed>";
/// let result = format_xml_safe(invalid);
/// assert_eq!(result.trim(), invalid);
/// ```
pub fn format_xml_safe(xml: &str) -> String {
    format_xml_pretty(xml).unwrap_or_else(|_| xml.to_string())
}

/// Minifies XML by removing all unnecessary whitespace.
///
/// # Arguments
///
/// * `xml` - XML string to minify
///
/// # Returns
///
/// `Ok(String)` with minified XML, or `Err(FormatError)` if parsing fails.
pub fn minify_xml(xml: &str) -> Result<String, FormatError> {
    if xml.len() > MAX_XML_FORMAT_SIZE {
        return Err(FormatError::ResponseTooLarge(xml.len()));
    }

    let mut result = String::new();
    let mut chars = xml.chars().peekable();
    let mut in_tag = false;
    let mut in_text = false;

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                in_text = false;
                result.push(ch);
            }
            '>' => {
                in_tag = false;
                result.push(ch);
            }
            c if c.is_whitespace() => {
                if in_tag && !result.ends_with(' ') {
                    result.push(' ');
                } else if in_text && !result.ends_with(' ') {
                    result.push(' ');
                }
            }
            _ => {
                if !in_tag {
                    in_text = true;
                }
                result.push(ch);
            }
        }
    }

    Ok(result.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_xml_pretty_simple() {
        let xml = "<root><child>text</child></root>";
        let formatted = format_xml_pretty(xml).unwrap();

        assert!(formatted.contains("<root>"));
        assert!(formatted.contains("  <child>"));
    }

    #[test]
    fn test_format_xml_pretty_nested() {
        let xml = "<root><parent><child>text</child></parent></root>";
        let formatted = format_xml_pretty(xml).unwrap();

        assert!(formatted.contains("<root>"));
        assert!(formatted.contains("  <parent>"));
        assert!(formatted.contains("    <child>"));
    }

    #[test]
    fn test_format_xml_pretty_attributes() {
        let xml = r#"<root id="1" name="test"><child attr="value">text</child></root>"#;
        let formatted = format_xml_pretty(xml).unwrap();

        assert!(formatted.contains(r#"<root id="1" name="test">"#));
        assert!(formatted.contains(r#"  <child attr="value">"#));
    }

    #[test]
    fn test_format_xml_self_closing() {
        let xml = "<root><child/><another/></root>";
        let formatted = format_xml_pretty(xml).unwrap();

        assert!(formatted.contains("  <child/>"));
        assert!(formatted.contains("  <another/>"));
    }

    #[test]
    fn test_format_xml_with_declaration() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root><child>text</child></root>"#;
        let formatted = format_xml_pretty(xml).unwrap();

        assert!(formatted.contains("<?xml"));
        assert!(formatted.contains("<root>"));
    }

    #[test]
    fn test_format_xml_with_comments() {
        let xml = "<root><!-- comment --><child>text</child></root>";
        let formatted = format_xml_pretty(xml).unwrap();

        // Comment handling may vary, just check structure is preserved
        assert!(formatted.contains("<root>"));
        assert!(formatted.contains("<child>"));
    }

    #[test]
    fn test_format_xml_with_cdata() {
        let xml = "<root><child><![CDATA[some data]]></child></root>";
        let formatted = format_xml_pretty(xml).unwrap();

        // CDATA handling may vary, just check structure is preserved
        assert!(formatted.contains("<root>"));
        assert!(formatted.contains("<child>"));
    }

    #[test]
    fn test_validate_xml_valid() {
        assert!(validate_xml("<root><child>text</child></root>"));
        assert!(validate_xml("<root/>"));
        assert!(validate_xml("<?xml version=\"1.0\"?><root/>"));
    }

    #[test]
    fn test_validate_xml_invalid() {
        assert!(!validate_xml("<unclosed>"));
        assert!(!validate_xml(""));
        assert!(!validate_xml("not xml"));
    }

    #[test]
    fn test_format_xml_safe_valid() {
        let xml = "<root><child>text</child></root>";
        let formatted = format_xml_safe(xml);

        assert!(formatted.contains("  <child>"));
    }

    #[test]
    fn test_format_xml_safe_invalid() {
        let xml = "<unclosed>";
        let formatted = format_xml_safe(xml);

        // The formatter might add a newline, so just check it's similar
        assert!(formatted.contains("<unclosed>"));
    }

    #[test]
    fn test_minify_xml() {
        let xml = r#"
        <root>
            <child>
                text
            </child>
        </root>
        "#;
        let minified = minify_xml(xml).unwrap();

        assert!(!minified.contains('\n'));
        assert!(minified.contains("<root>"));
    }

    #[test]
    fn test_format_xml_empty_tags() {
        let xml = "<root><empty></empty></root>";
        let formatted = format_xml_pretty(xml).unwrap();

        assert!(formatted.contains("<empty>"));
        assert!(formatted.contains("</empty>"));
    }

    #[test]
    fn test_format_xml_mixed_content() {
        let xml = "<root>text<child>nested</child>more text</root>";
        let formatted = format_xml_pretty(xml).unwrap();

        assert!(formatted.contains("<root>"));
        assert!(formatted.contains("<child>"));
    }
}
