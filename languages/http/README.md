# HTTP Language Configuration for Zed

This directory contains the language configuration for HTTP request files (`.http` and `.rest` extensions).

## Files

### `config.toml`
The main language configuration file that defines:
- **Language Name**: "HTTP"
- **File Extensions**: `.http` and `.rest`
- **Comment Styles**: Single-line (`#`, `//`) and block comments (`/* */`)
- **Bracket Pairs**: Curly braces for variables `{{}}`, parentheses, square brackets
- **Autoclose Pairs**: Automatic closing of quotes, braces, brackets, and parentheses

### `highlights.scm`
Tree-sitter query file for syntax highlighting:
- **HTTP Methods**: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`, `OPTIONS`, `HEAD`, `TRACE`, `CONNECT`
- **URLs**: HTTP and HTTPS URLs highlighted as special strings
- **Headers**: Header names (e.g., `Content-Type:`) highlighted as properties
- **Variables**: Template variables `{{variableName}}` highlighted as parameters
- **Comments**: Both `#` and `//` style comments
- **Request Delimiter**: `###` separator between requests
- **Special Headers**: `Content-Type`, `Authorization`, `Accept`, `User-Agent`

### `grammar.js`
Minimal Tree-sitter grammar for HTTP request files. This provides basic parsing support for:
- HTTP request lines (method, URL, version)
- Headers (name-value pairs)
- Request bodies
- Variables in `{{}}` syntax
- Comments (single-line and block)
- Request separators (`###`)

**Note**: This is a simplified grammar for MVP. A full Tree-sitter grammar with complete parsing will be implemented in Phase 4.

## Usage

Once the extension is installed in Zed, any file with `.http` or `.rest` extension will automatically use this language configuration for syntax highlighting and editing features.

### Example

Create a file named `test.http`:

```http
### GET Request
GET https://api.github.com/users/octocat
Accept: application/json

### POST Request with Variable
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "{{userName}}",
  "timestamp": "{{$timestamp}}"
}
```

The syntax highlighting will recognize:
- `GET`, `POST` as keywords
- URLs as special strings
- `Accept`, `Content-Type` as properties
- `{{userName}}`, `{{$timestamp}}` as variables
- `###` as request delimiters

## Future Enhancements (Phase 4)

- Complete Tree-sitter grammar with full parsing capabilities
- Advanced syntax error detection
- Code folding support
- IntelliSense for HTTP methods and headers
- Variable reference highlighting and navigation
- Request/response pairing