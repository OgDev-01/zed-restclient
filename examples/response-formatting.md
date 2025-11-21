# Enhanced Response Formatting

This document describes the enhanced response formatting system in the REST Client extension, which provides beautiful syntax-highlighted output for JSON, XML, and HTML responses.

## Overview

The REST Client extension now includes advanced response formatting capabilities:

- **Pretty-printing** for JSON and XML with proper indentation (2 spaces)
- **Minification** for compact view
- **Validation** to check if content is valid JSON/XML
- **Syntax highlighting metadata** for Zed's built-in highlighter
- **Graceful fallback** to raw display when formatting fails
- **Size limits** to prevent performance issues (10MB max)
- **Toggle between formatted and raw views**

## Architecture

### Module Structure

```
src/formatter/
├── mod.rs           # Main formatter with response formatting
├── json.rs          # Enhanced JSON formatting
├── xml.rs           # Enhanced XML formatting
├── syntax.rs        # Syntax highlighting support
└── content_type.rs  # Content type detection
```

### Key Components

#### 1. JSON Formatter (`json.rs`)

The JSON formatter provides:

- `format_json_pretty(json: &str) -> Result<String, FormatError>` - Pretty-prints JSON with 2-space indentation
- `minify_json(json: &str) -> Result<String, FormatError>` - Removes all unnecessary whitespace
- `validate_json(json: &str) -> bool` - Validates JSON syntax
- `format_json_safe(json: &str) -> String` - Formats with automatic fallback to raw

**Example:**

```rust
use rest_client::formatter::json::{format_json_pretty, minify_json};

let json = r#"{"name":"John","age":30,"nested":{"city":"NYC"}}"#;

// Pretty-print
let formatted = format_json_pretty(json).unwrap();
// Output:
// {
//   "name": "John",
//   "age": 30,
//   "nested": {
//     "city": "NYC"
//   }
// }

// Minify
let minified = minify_json(json).unwrap();
// Output: {"name":"John","age":30,"nested":{"city":"NYC"}}
```

#### 2. XML Formatter (`xml.rs`)

The XML formatter provides:

- `format_xml_pretty(xml: &str) -> Result<String, FormatError>` - Pretty-prints XML with proper indentation
- `minify_xml(xml: &str) -> Result<String, FormatError>` - Removes unnecessary whitespace
- `validate_xml(xml: &str) -> bool` - Validates XML structure
- `format_xml_safe(xml: &str) -> String` - Formats with automatic fallback

**Features:**
- Handles XML declarations (`<?xml version="1.0"?>`)
- Supports comments (`<!-- comment -->`)
- Handles CDATA sections (`<![CDATA[...]]>`)
- Supports self-closing tags (`<tag/>`)
- Preserves attributes

**Example:**

```rust
use rest_client::formatter::xml::format_xml_pretty;

let xml = r#"<?xml version="1.0"?><root><person name="John"><age>30</age></person></root>"#;

let formatted = format_xml_pretty(xml).unwrap();
// Output:
// <?xml version="1.0"?>
// <root>
//   <person name="John">
//     <age>30</age>
//   </person>
// </root>
```

#### 3. Syntax Highlighting (`syntax.rs`)

The syntax highlighter provides metadata for Zed's built-in syntax highlighting:

- `Language` enum: Json, Xml, Html, PlainText
- `HighlightInfo` struct: Contains language, file extension, and MIME type
- `detect_language(text: &str) -> Language` - Auto-detects language from content
- `apply_syntax_highlighting(text: &str, language: &str) -> String` - Returns text with language metadata

**Note:** Since Zed extensions run in WASM without access to full syntax highlighting libraries like syntect, the text is returned as-is. Zed's editor handles the actual syntax highlighting based on the language metadata we provide.

#### 4. Enhanced FormattedResponse

The `FormattedResponse` struct now includes:

```rust
pub struct FormattedResponse {
    pub content_type: ContentType,
    pub formatted_body: String,
    pub raw_body: String,              // NEW: Raw unformatted body
    pub status_line: String,
    pub headers_text: String,
    pub metadata: ResponseMetadata,
    pub highlight_info: Option<HighlightInfo>, // NEW: Syntax highlighting info
    pub is_formatted: bool,             // NEW: Current view mode
}
```

**New Methods:**

- `toggle_view(&mut self)` - Switches between formatted and raw view
- `get_body(&self) -> &str` - Gets current body (formatted or raw)
- `get_raw_body(&self) -> &str` - Gets raw unformatted body
- `get_formatted_body(&self) -> String` - Gets formatted body (formats if needed)

## Usage

### Formatting HTTP Responses

The main `format_response()` function automatically detects content type and applies appropriate formatting:

```rust
use rest_client::formatter::format_response;
use rest_client::models::response::HttpResponse;

let response = HttpResponse::new(200, "OK".to_string());
// ... set headers and body ...

let formatted = format_response(&response);

// Access formatted body
println!("{}", formatted.formatted_body);

// Check syntax highlighting info
if let Some(info) = formatted.highlight_info {
    println!("Language: {:?}", info.language);
    println!("Extension: {}", info.extension);
    println!("MIME type: {}", info.mime_type);
}
```

### Toggling Between Formatted and Raw Views

```rust
let mut formatted = format_response(&response);

// View formatted (default)
assert!(formatted.is_formatted);
println!("{}", formatted.get_body());

// Toggle to raw view
formatted.toggle_view();
assert!(!formatted.is_formatted);
println!("{}", formatted.get_body());

// Toggle back to formatted
formatted.toggle_view();
assert!(formatted.is_formatted);
```

### Direct Formatting

You can also use the formatters directly:

```rust
use rest_client::formatter::json::format_json_pretty;
use rest_client::formatter::xml::format_xml_pretty;

// Format JSON
let json = r#"{"compact":true}"#;
let pretty_json = format_json_pretty(json).unwrap();

// Format XML
let xml = "<root><child>value</child></root>";
let pretty_xml = format_xml_pretty(xml).unwrap();
```

## Error Handling

All formatting functions return `Result<String, FormatError>` with these error types:

- `FormatError::JsonError(String)` - JSON parsing or formatting error
- `FormatError::XmlError(String)` - XML formatting error
- `FormatError::EncodingError(String)` - UTF-8 encoding error
- `FormatError::ResponseTooLarge(usize)` - Response exceeds 10MB limit

**Graceful Fallback:**

When formatting fails, the system automatically falls back to displaying the raw content:

```rust
let malformed_json = r#"{"invalid": json}"#;

// Direct formatting returns error
assert!(format_json_pretty(malformed_json).is_err());

// Safe formatting returns original
let safe = format_json_safe(malformed_json);
assert_eq!(safe, malformed_json);
```

## Performance Considerations

### Size Limits

- **Maximum response size**: 10MB (configurable in `json.rs` and `xml.rs`)
- Responses larger than 10MB are truncated with a warning
- This prevents performance issues when formatting very large responses

### Optimization

- JSON formatting uses `serde_json` with custom formatters for efficiency
- XML formatting uses a streaming parser approach
- Large responses can be viewed in raw mode for better performance

## Testing

The enhanced formatters include comprehensive test coverage:

- **86 formatter tests** covering all formatting functions
- **401 total tests** in the entire test suite
- Tests include edge cases: malformed content, Unicode, large responses, etc.

### Running Tests

```bash
# Test formatters only
cargo test --lib formatter

# Test all
cargo test --lib
```

## Examples

### Complex Nested JSON

```rust
let json = r#"{
  "users": [
    {
      "id": 1,
      "name": "John Doe",
      "email": "john@example.com",
      "address": {
        "street": "123 Main St",
        "city": "New York",
        "zip": "10001"
      }
    }
  ],
  "meta": {
    "total": 1,
    "page": 1
  }
}"#;

let formatted = format_json_pretty(json).unwrap();
// Beautifully formatted with 2-space indentation
```

### XML with Attributes and CDATA

```rust
let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<catalog>
  <book id="1" isbn="978-0-123456-78-9">
    <title>REST APIs Explained</title>
    <author>John Smith</author>
    <description><![CDATA[A comprehensive guide to <REST> APIs]]></description>
  </book>
</catalog>"#;

let formatted = format_xml_pretty(xml).unwrap();
// Properly indented with attributes preserved
```

### Syntax Highlighting Detection

```rust
use rest_client::formatter::syntax::{detect_language, Language};

assert_eq!(detect_language(r#"{"json": true}"#), Language::Json);
assert_eq!(detect_language("<xml/>"), Language::Xml);
assert_eq!(detect_language("<!DOCTYPE html>"), Language::Html);
assert_eq!(detect_language("plain text"), Language::PlainText);
```

## Future Enhancements

Potential future improvements:

1. **Custom indentation** - Allow users to configure indent size (2, 4, tabs)
2. **Line numbers** - Add line numbers to formatted output for easier debugging
3. **Syntax themes** - Support for different color schemes
4. **Format on save** - Auto-format when saving response to file
5. **Diff view** - Compare formatted vs raw side-by-side
6. **HTML formatting** - Pretty-print HTML responses
7. **GraphQL formatting** - Support for GraphQL query formatting

## Conclusion

The enhanced response formatting system provides a powerful and user-friendly way to view HTTP responses in the Zed editor. With automatic content detection, graceful error handling, and seamless integration with Zed's syntax highlighting, it makes working with REST APIs more productive and enjoyable.

For more information, see:
- [Formatter Module Documentation](../src/formatter/mod.rs)
- [JSON Formatter](../src/formatter/json.rs)
- [XML Formatter](../src/formatter/xml.rs)
- [Syntax Highlighting](../src/formatter/syntax.rs)