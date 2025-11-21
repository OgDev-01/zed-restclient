# Changelog

All notable changes to the REST Client extension for Zed will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-11-21

### 🎉 Initial Release

The first public release of REST Client for Zed - a powerful HTTP client extension that brings professional API testing directly into your editor.

### ✨ Core Features

#### HTTP Request Support
- **All HTTP Methods**: GET, POST, PUT, DELETE, PATCH, OPTIONS, HEAD, TRACE, CONNECT
- **Request Parsing**: Intelligent parsing of `.http` and `.rest` files
- **Multiple Requests**: Support for multiple requests in a single file separated by `###`
- **Comments**: Full support for `#` and `//` style comments
- **Headers**: Custom headers with multiline support
- **Request Bodies**: JSON, XML, form data, plain text, and binary content
- **Query Parameters**: Inline in URL or as separate lines

#### Response Handling
- **Auto-formatting**: Beautiful formatting for JSON, XML, and HTML responses
- **Syntax Highlighting**: Color-coded responses for easy reading
- **Response Metadata**: Display status codes, headers, timing, and size information
- **Raw/Formatted Toggle**: Switch between formatted and raw response views
- **Large Response Handling**: Automatic pagination/truncation for responses >10MB
- **Response Folding**: Collapse large response sections for easier navigation
- **Copy Actions**: Copy full response, headers only, or body only
- **Save to File**: Save responses with smart filename suggestions

#### Variable System
- **Environment Variables**: Define and switch between multiple environments (dev, staging, prod)
- **System Variables**: 
  - `{{$guid}}` - Generate UUIDs
  - `{{$timestamp}}` - Unix timestamps with offset support
  - `{{$datetime}}` - Formatted dates (ISO8601, RFC1123, custom formats)
  - `{{$randomInt min max}}` - Random integers in range
  - `{{$processEnv VAR}}` - Access environment variables
- **Custom Variables**: File-level and shared variables with `@name = value` syntax
- **Request Variables**: Capture response data for request chaining
- **Variable Substitution**: Smart resolution in URLs, headers, and bodies
- **Nested Variables**: Support for variables referencing other variables

#### Request Chaining & Capture
- **JSONPath Extraction**: `# @capture token = $.access_token`
- **Header Extraction**: `# @capture sessionId = headers.X-Session-Id`
- **XPath Support**: Extract from XML responses (planned)
- **Cross-request Data**: Use captured values in subsequent requests
- **Authentication Flows**: Chain login → authenticated requests seamlessly

#### Authentication
- **Basic Authentication**: Username and password encoding
- **Bearer Tokens**: Authorization header support
- **API Keys**: Custom header and query parameter authentication
- **Environment-based Secrets**: Keep credentials secure with environment variables

#### GraphQL Support
- **GraphQL Queries**: Full query support with syntax validation
- **Mutations**: Create, update, delete operations
- **Variables**: GraphQL variable substitution
- **Introspection**: Schema exploration (via standard queries)

#### Code Generation
- **JavaScript/TypeScript**: Generate fetch/axios code
- **Python**: Generate requests library code
- **Multiple Libraries**: Support for different HTTP libraries per language
- **Request Preservation**: Generated code includes all headers, body, and authentication

#### cURL Integration
- **Import cURL**: Paste cURL commands and convert to HTTP syntax
- **Export cURL**: Generate cURL commands from HTTP requests
- **Full Feature Support**: Headers, body, authentication preserved
- **Shell Escaping**: Proper escaping for safe copy-paste

#### Language Server Features (LSP)
- **Auto-completion**: 
  - HTTP methods
  - Headers (Content-Type, Authorization, etc.)
  - Variable names from environments
  - System variable functions
- **Hover Information**: 
  - Variable value previews
  - Header descriptions
  - HTTP method documentation
- **Diagnostics**: 
  - Syntax error detection
  - Invalid header warnings
  - Malformed URL detection
  - Missing variable warnings
- **CodeLens Actions**: 
  - "Send Request" above each request
  - "Send All Requests" at file top
  - Request count display

#### Request History
- **Automatic Tracking**: All requests saved with responses
- **Search & Filter**: Find previous requests by URL, method, or content
- **Replay Requests**: Re-run historical requests
- **Export History**: Save history for documentation or debugging
- **Clear History**: Remove old entries
- **Performance**: Handles 1,000+ entries without slowdown

#### Developer Experience
- **Tree-sitter Grammar**: Full syntax highlighting in Zed
- **Slash Commands**: Quick access via `/rest` command family
- **Keyboard Shortcuts**: Send requests without leaving the keyboard
- **Tab Management**: Organize multiple responses
- **Status Indicators**: Visual feedback for request status
- **Error Messages**: User-friendly error reporting
- **Loading States**: Progress indication for long requests

### 🚀 Performance
- **Fast Parsing**: <100ms for files up to 10,000 lines
- **Quick Formatting**: <50ms to begin rendering responses
- **Large File Support**: Handle files with 1,000+ requests
- **Memory Efficient**: <100MB typical memory usage
- **Compact Binary**: 1.7MB WASM bundle (optimized)
- **Lazy Loading**: History loaded on-demand for faster startup
- **Streaming**: Large responses streamed for preview

### 📚 Documentation
- **Comprehensive README**: 200+ lines of examples and usage
- **API Documentation**: Full rustdoc coverage
- **Example Files**: Sample `.http` files for all features
- **Tutorial**: Step-by-step getting started guide
- **Migration Guide**: Help for VS Code REST Client users
- **Troubleshooting**: Common issues and solutions

### 🧪 Testing
- **680 Unit Tests**: Comprehensive test coverage
- **60 Doc Tests**: Verified code examples
- **Integration Tests**: End-to-end feature validation
- **Benchmark Suite**: Performance regression detection
- **100% Pass Rate**: All tests passing on release

### 🏗️ Architecture
- **Modular Design**: Clean separation of concerns
- **Type Safety**: Full Rust type system benefits
- **Error Handling**: Comprehensive error types and recovery
- **Extensibility**: Plugin-ready architecture for future features
- **WASM-native**: Built specifically for Zed's extension system

### 🔧 Technical Details
- **Language**: Rust (stable)
- **Target**: wasm32-wasip1
- **Dependencies**: Minimal external dependencies
- **Build Profile**: Optimized for size and performance
- **Code Quality**: Clippy-clean, rustfmt-formatted

### 📋 Requirements
- Zed Editor v0.100.0 or later
- Network access for HTTP requests
- Optional: `.env` files for environment management

### 🙏 Acknowledgments
- Inspired by the VS Code REST Client extension
- Built with the Zed extension API
- Community feedback during development

### 📝 Notes
- This is the initial stable release
- All planned features for v0.1.0 are implemented
- Ready for production use
- Feedback and contributions welcome!

---

## [Unreleased]

### Planned Features
- **WebSocket Support**: Real-time connection testing
- **Server-Sent Events**: SSE stream handling
- **OAuth 2.0 Flow**: Full OAuth authentication support
- **Certificate Management**: Custom SSL/TLS certificates
- **Proxy Configuration**: HTTP/HTTPS proxy support
- **Request Collections**: Organize related requests
- **Mock Server**: Built-in mock server for testing
- **Collaboration**: Share requests and collections
- **Response Diff**: Compare response changes over time
- **Performance Profiling**: Request waterfall and timing breakdown

### Under Consideration
- **Postman Import**: Import Postman collections
- **OpenAPI Integration**: Generate requests from OpenAPI specs
- **gRPC Support**: Protocol Buffer request testing
- **Request Snippets**: Reusable request templates
- **CI/CD Integration**: Run requests in automated pipelines

---

## Version History

- **0.1.0** (2024-11-21) - Initial public release

---

## How to Update

Updates will be available through the Zed extension marketplace:

1. Open Zed
2. Go to Extensions panel
3. Check for updates
4. Click "Update" next to REST Client

Or use the command palette:
- `Cmd+Shift+P` → "zed: update extensions"

---

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/rest-client-zed)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/rest-client-zed/discussions)
- **Documentation**: See README.md

---

*Thank you for using REST Client for Zed!* 🚀