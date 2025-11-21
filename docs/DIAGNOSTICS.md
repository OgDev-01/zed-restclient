# LSP Diagnostics for REST Client

## Overview

The REST Client extension provides comprehensive real-time diagnostics for `.http` and `.rest` files. Diagnostics help you catch errors and potential issues before sending requests, improving your workflow and reducing mistakes.

## Features

### 1. Syntax Error Detection

The diagnostics system validates HTTP request syntax and reports errors with precise line numbers:

- **Invalid HTTP Methods**: Detects unsupported or misspelled HTTP methods
  ```http
  INVALID https://api.example.com  # Error: Invalid HTTP method 'INVALID'
  ```

- **Missing URLs**: Catches requests without URLs
  ```http
  GET  # Error: Missing URL in request line
  ```

- **Invalid Header Format**: Validates header syntax
  ```http
  InvalidHeaderNoColon  # Error: Invalid header format
  ```

### 2. Variable Validation

Comprehensive variable checking with smart detection:

- **Undefined Variables**: Warns about variables that aren't defined
  ```http
  GET https://api.example.com/{{undefinedVar}}
  # Warning: Undefined variable 'undefinedVar'
  # Suggestion: Define this variable in your environment file or .http file
  ```

- **System Variables**: No warnings for system variables (always available)
  ```http
  GET https://api.example.com/{{$guid}}  # ✓ OK - system variable
  GET https://api.example.com/{{$timestamp}}  # ✓ OK - system variable
  ```

- **Empty Variables**: Catches empty variable declarations
  ```http
  GET https://api.example.com/{{}}
  # Error: Empty variable name
  ```

- **Unclosed Braces**: Detects malformed variable syntax
  ```http
  GET https://api.example.com/{{unclosed
  # Error: Unclosed variable braces
  # Suggestion: Add closing }} to complete the variable
  ```

### 3. URL Validation

Validates URL format and structure:

- **Missing URL Scheme**: Warns when URLs don't start with http:// or https://
  ```http
  GET api.example.com/users
  # Warning: URL 'api.example.com/users' should start with http:// or https://
  ```

- **Spaces in URLs**: Errors on URLs containing spaces
  ```http
  GET https://api.example.com/users with spaces
  # Error: URL cannot contain spaces
  # Suggestion: Use %20 for spaces or remove them
  ```

### 4. Header Validation

Smart header validation with typo detection:

- **Common Typos**: Detects and suggests corrections for common header typos
  ```http
  Conten-Type: application/json
  # Warning: Possible typo in header name 'Conten-Type'
  # Suggestion: Did you mean 'Content-Type'?
  ```

- **Detected Typos**:
  - `Conten-Type` → `Content-Type`
  - `contenttype` → `Content-Type`
  - `content_type` → `Content-Type`
  - `authorisation` → `Authorization`
  - And more...

- **Spaces in Header Names**: Errors on invalid header names
  ```http
  My Header: value
  # Error: Header names cannot contain spaces
  ```

### 5. JSON Body Validation

Validates JSON syntax when Content-Type is application/json:

- **Valid JSON**: No errors for properly formatted JSON
  ```http
  POST https://api.example.com/users
  Content-Type: application/json

  {
    "name": "John Doe",
    "email": "john@example.com"
  }
  # ✓ OK
  ```

- **Invalid JSON**: Reports JSON syntax errors
  ```http
  POST https://api.example.com/users
  Content-Type: application/json

  {invalid json}
  # Error: Invalid JSON in request body: expected value at line 1 column 2
  # Suggestion: Check JSON syntax - ensure proper quotes, commas, and brackets
  ```

### 6. Required Headers Check

Validates that appropriate headers are present for request methods:

- **POST/PUT/PATCH without Content-Type**: Warns when body-sending methods lack Content-Type
  ```http
  POST https://api.example.com/users

  {"name": "test"}
  # Warning: POST request should include Content-Type header when sending a body
  # Suggestion: Add 'Content-Type: application/json' or appropriate content type
  ```

- **GET Requests**: No warnings (Content-Type not required)
  ```http
  GET https://api.example.com/users
  # ✓ OK - no Content-Type needed for GET
  ```

## Diagnostic Severity Levels

Diagnostics are categorized by severity:

### Error (Red Squigglies)
Critical issues that will prevent the request from working correctly:
- Invalid HTTP methods
- Invalid URL format
- Invalid header format
- Invalid JSON syntax
- Empty variable names
- Unclosed variable braces

### Warning (Yellow Squigglies)
Potential issues that may cause problems:
- Undefined variables
- URL without scheme (http:// or https://)
- Header name typos
- Missing Content-Type for POST/PUT/PATCH
- Empty request blocks

### Info (Blue Squigglies)
Informational messages:
- Currently not used, reserved for future features

## Diagnostic Codes

Each diagnostic has a unique code for programmatic filtering:

| Code | Description |
|------|-------------|
| `invalid-method` | Invalid HTTP method |
| `invalid-url` | Invalid URL format |
| `invalid-header` | Invalid header format |
| `missing-url` | Missing URL in request line |
| `empty-request` | Empty request block |
| `invalid-http-version` | Invalid HTTP version |
| `undefined-variable` | Undefined variable reference |
| `empty-variable` | Empty variable name |
| `unclosed-braces` | Unclosed variable braces |
| `url-scheme-missing` | URL missing http:// or https:// |
| `url-contains-spaces` | URL contains spaces |
| `header-typo` | Possible typo in header name |
| `header-name-spaces` | Header name contains spaces |
| `empty-header-name` | Empty header name |
| `invalid-json` | Invalid JSON in request body |
| `missing-content-type` | Missing Content-Type header |

## Usage

### Programmatic Access

```rust
use rest_client::language_server::diagnostics::provide_diagnostics;
use rest_client::variables::VariableContext;
use std::path::PathBuf;

// Create variable context
let context = VariableContext::new(PathBuf::from("."));

// Get diagnostics for a document
let document = r#"
GET https://api.example.com/{{undefined}}
Conten-Type: application/json
"#;

let diagnostics = provide_diagnostics(document, &context);

// Process diagnostics
for diagnostic in diagnostics {
    println!("{:?}: {} at line {}", 
        diagnostic.severity, 
        diagnostic.message,
        diagnostic.range.start.line
    );
    
    if let Some(suggestion) = diagnostic.suggestion {
        println!("  Suggestion: {}", suggestion);
    }
}
```

### API Reference

```rust
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>,
    pub suggestion: Option<String>,
}

pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
}

pub struct Range {
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub line: usize,      // Zero-based line number
    pub character: usize, // Zero-based character offset
}
```

## Performance

The diagnostics system is designed for real-time feedback:

- **Fast parsing**: Efficient regex-based validation
- **Incremental checking**: Only affected requests are re-validated
- **Debounced updates**: Diagnostics update after typing pauses
- **Non-blocking**: Runs asynchronously without freezing the editor

## Integration with Zed

The diagnostics integrate seamlessly with Zed's LSP infrastructure:

1. **Real-time Updates**: Diagnostics appear as you type
2. **Error Squigglies**: Visual indicators in the editor
3. **Hover Information**: Detailed error messages on hover
4. **Quick Fixes**: Suggestions appear in the problems panel
5. **Keyboard Navigation**: Jump between errors with keyboard shortcuts

## Examples

### Example 1: Multiple Issues

```http
INVALID api.example.com/{{undefined}}
Conten-Type: application/json

{invalid json}
```

**Diagnostics**:
1. Error: Invalid HTTP method 'INVALID'
2. Warning: URL should start with http:// or https://
3. Warning: Undefined variable 'undefined'
4. Warning: Possible typo in header name 'Conten-Type'
5. Error: Invalid JSON in request body

### Example 2: Valid Request

```http
POST https://api.example.com/users
Content-Type: application/json
Authorization: Bearer {{$guid}}

{
  "name": "Alice",
  "email": "alice@example.com"
}
```

**Diagnostics**: None ✓

### Example 3: Missing Required Header

```http
POST https://api.example.com/users

{"name": "test"}
```

**Diagnostics**:
1. Warning: POST request should include Content-Type header when sending a body

## Best Practices

1. **Fix Errors First**: Address red squigglies before warnings
2. **Define Variables**: Create environment files for undefined variables
3. **Use Standard Headers**: Follow HTTP header naming conventions
4. **Validate JSON**: Use a JSON validator for complex bodies
5. **Include Content-Type**: Always specify Content-Type for POST/PUT/PATCH

## Future Enhancements

Planned diagnostic features:

- [ ] Authentication header validation
- [ ] Response assertion validation
- [ ] Cross-request variable dependencies
- [ ] GraphQL query validation
- [ ] XML body validation
- [ ] Custom diagnostic rules via configuration
- [ ] Code actions for quick fixes

## Troubleshooting

### Diagnostics Not Showing

1. Ensure the file extension is `.http` or `.rest`
2. Check that the Zed REST Client extension is installed and enabled
3. Verify that LSP features are enabled in Zed settings

### False Positives

If you see incorrect warnings:

1. **Undefined Variables**: Ensure variables are defined in your environment file
2. **Header Typos**: Some custom headers may trigger warnings - they're safe to ignore if intentional
3. **URL Scheme**: URLs with variables may show warnings - they'll resolve at runtime

### Performance Issues

If diagnostics are slow:

1. Large files may take longer to validate
2. Complex JSON bodies increase validation time
3. Consider splitting large files into smaller ones

## Related Documentation

- [Variable System](VARIABLES.md)
- [Environment Management](ENVIRONMENTS.md)
- [LSP Features](LSP.md)
- [Grammar Specification](../languages/http/README.md)

## Contributing

To improve diagnostics:

1. Report false positives/negatives as GitHub issues
2. Suggest new validation rules
3. Contribute code for additional checks
4. Improve error messages and suggestions

---

**Note**: This documentation reflects diagnostics as of version 0.1.0. Features and behavior may change in future versions.