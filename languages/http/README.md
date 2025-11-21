# HTTP Request File Grammar

Tree-sitter grammar for HTTP request files (.http, .rest extensions).

## Overview

This directory contains a complete Tree-sitter grammar for parsing HTTP request files used by REST clients. The grammar supports:

- **HTTP Methods**: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, CONNECT
- **URLs**: Full URLs with http:// or https:// schemes
- **HTTP Versions**: HTTP/1.1, HTTP/2, etc.
- **Headers**: Standard HTTP headers with values
- **Request Bodies**: JSON, XML, GraphQL, and plain text
- **Variables**: {{variable}} syntax for templating
- **Comments**: Both `#` and `//` style comments
- **Request Separators**: `###` to separate multiple requests in one file

## File Structure

```
languages/http/
├── grammar.js              # Main Tree-sitter grammar definition
├── package.json            # NPM package configuration
├── src/                    # Generated parser (do not edit manually)
│   ├── parser.c            # Generated C parser
│   ├── grammar.json        # Generated grammar metadata
│   └── node-types.json     # Generated node type definitions
├── queries/                # Tree-sitter query files
│   ├── highlights.scm      # Syntax highlighting rules
│   └── injections.scm      # Embedded language support (JSON, XML, GraphQL)
├── config.toml             # Zed language configuration
├── highlights.scm          # Backward compatibility (redirects to queries/)
└── README.md               # This file
```

## Grammar Rules

### Source File

A source file contains multiple requests separated by `###` delimiters:

```http
### First request
GET https://api.example.com/users

### Second request
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe"
}
```

### Request Structure

Each request consists of:
1. **Method Line**: HTTP method + URL + optional HTTP version
2. **Headers** (optional): Zero or more header lines
3. **Body** (optional): Request body preceded by a blank line

```
request := method_line headers? body?
```

### Method Line

```
method_line := METHOD SPACE URL (SPACE HTTP_VERSION)? NEWLINE
```

Example:
```http
POST https://api.example.com/users HTTP/1.1
```

### Headers

```
headers := header+
header := HEADER_NAME ":" SPACE? HEADER_VALUE NEWLINE
```

Example:
```http
Content-Type: application/json
Authorization: Bearer {{token}}
Accept: application/json
```

### Body

The body starts with a blank line after the headers (or method line if no headers):

```
body := NEWLINE body_content
```

The body content can be:
- JSON (automatically detected and syntax-highlighted)
- XML (automatically detected and syntax-highlighted)
- GraphQL queries (automatically detected and syntax-highlighted)
- Plain text

Example:
```http
POST https://api.example.com/users
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com"
}
```

### Variables

Variables use the `{{variable}}` syntax and can appear in:
- URLs: `GET https://api.example.com/users/{{userId}}`
- Headers: `Authorization: Bearer {{token}}`

**Note**: In the current simplified grammar, variables are matched as part of the URL/header text rather than as separate nodes. They are highlighted using regex patterns in the highlight queries.

### Comments

Two comment styles are supported:

```http
# This is a comment
// This is also a comment
```

Comments must be on their own line.

### Request Separator

Multiple requests in a single file are separated by `###`:

```http
### First request
GET https://api.example.com/users

### Second request
POST https://api.example.com/users
```

The separator can optionally include a description:
```http
### Get all users
GET https://api.example.com/users

### Create new user
POST https://api.example.com/users
```

## Syntax Highlighting

The grammar includes comprehensive syntax highlighting through Tree-sitter queries:

### Highlight Scopes

- **Methods**: `keyword.method` (with specific scopes for GET, POST, etc.)
- **URLs**: `string.special.url`
- **HTTP Version**: `constant`
- **Request Separator**: `keyword.delimiter`
- **Header Names**: `property` (with `property.special` for important headers)
- **Header Values**: `string` (with `string.special` for certain content types)
- **Comments**: `comment`
- **Body Content**: `string` (with language injection for JSON/XML/GraphQL)
- **Punctuation**: `punctuation.delimiter` for colons

### Special Header Highlighting

The following headers receive special highlighting:
- Content-Type
- Authorization
- Accept
- User-Agent
- Accept-Encoding
- Cache-Control
- Connection
- Host
- Origin
- Referer

### Content-Type Value Highlighting

Special highlighting for common content types:
- `application/json`
- `application/xml`
- `application/graphql`
- `application/x-www-form-urlencoded`
- `text/xml`, `text/html`, `text/plain`
- `multipart/form-data`

### Authentication Pattern Highlighting

Special highlighting for authentication values:
- Bearer tokens: `Authorization: Bearer xxx`
- Basic auth: `Authorization: Basic xxx`

## Language Injection

The grammar supports injecting syntax highlighting for embedded languages in request bodies:

### Content-Type Based Injection

When a request includes a `Content-Type` header, the appropriate grammar is injected:

- `Content-Type: application/json` → JSON grammar
- `Content-Type: application/xml` → XML grammar
- `Content-Type: application/graphql` → GraphQL grammar
- `Content-Type: text/html` → HTML grammar
- `Content-Type: text/javascript` → JavaScript grammar

### Heuristic-Based Injection

Even without a Content-Type header, the grammar can detect the body type:

- Body starting with `{` or `[` → JSON
- Body starting with `<` → XML
- Body starting with `query`, `mutation`, or `subscription` → GraphQL

## Building the Parser

The parser is generated from `grammar.js` using the Tree-sitter CLI:

```bash
# Install tree-sitter-cli
npm install

# Generate the parser
npx tree-sitter generate

# Test the parser
npx tree-sitter test

# Parse a test file
npx tree-sitter parse test.http
```

## Testing

Test files are provided to validate the grammar:

- `test.http` - Comprehensive test with various request formats
- `simple-test.http` - Simple test with basic requests

Run tests with:
```bash
npx tree-sitter test
```

Parse and inspect a file:
```bash
npx tree-sitter parse simple-test.http
```

## Integration with Zed

The grammar integrates with Zed through:

1. **config.toml** - Language configuration (file extensions, comment styles)
2. **queries/highlights.scm** - Syntax highlighting rules
3. **queries/injections.scm** - Embedded language support

Zed automatically loads these files when the extension is installed.

## Grammar Design Decisions

### Simplified URL and Header Parsing

The current grammar uses regex patterns for URLs and header values rather than parsing them into substructures. This design:

- ✅ Simplifies the grammar and reduces conflicts
- ✅ Improves parsing performance
- ✅ Handles edge cases more reliably
- ⚠️ Variables (`{{var}}`) are not separate nodes (highlighted via regex)

### Body Content Detection

The grammar uses heuristics to detect body content types:
1. Check Content-Type header (if present)
2. Look at the first character of the body (`{`, `<`, etc.)
3. Fall back to plain text

This approach works well for most real-world cases.

### Request Separator Handling

The `###` separator is parsed at the top level, preventing it from being mistaken for body content or comments. This ensures reliable request boundaries.

## Performance

The grammar is designed for:
- **Speed**: Fast incremental parsing
- **Large Files**: Handles files with hundreds of requests
- **Real-time**: Updates syntax highlighting as you type

## Known Limitations

1. **Multi-line Headers**: Header continuation (values spanning multiple lines) is not currently supported
2. **Variables**: Variables are highlighted but not parsed as separate nodes
3. **Query Parameters**: URL query parameters are treated as part of the URL string

## Future Enhancements

Possible improvements for future versions:
- Parse variables as separate nodes for better tooling support
- Add support for multi-line header values (folded headers)
- Parse URL components (scheme, host, path, query)
- Support for environment file syntax
- Request metadata (test assertions, scripts)

## Contributing

When modifying the grammar:

1. Edit `grammar.js`
2. Run `npx tree-sitter generate`
3. Test with `npx tree-sitter test`
4. Verify highlighting with test files
5. Update this README if adding new features

## License

This grammar is part of the REST Client extension for Zed.

## References

- [Tree-sitter Documentation](https://tree-sitter.github.io/)
- [HTTP/1.1 RFC 7230](https://tools.ietf.org/html/rfc7230)
- [REST Client for VS Code](https://marketplace.visualstudio.com/items?itemName=humao.rest-client)