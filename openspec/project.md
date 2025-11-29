# Project Context

## Purpose

This is a **REST Client extension for the Zed editor** that brings professional API testing and HTTP request execution directly into the editor workflow. Inspired by the popular VS Code REST Client extension, it allows developers to:

- Send HTTP requests from `.http` and `.rest` files without leaving the editor
- View beautifully formatted responses with syntax highlighting
- Manage multiple environments (dev, staging, production) with variable substitution
- Chain requests together using JSONPath to capture response values
- Generate code snippets in multiple languages from HTTP requests
- Import/export cURL commands
- Test GraphQL APIs with full query and mutation support

The extension is designed to replace standalone API testing tools like Postman/Insomnia for developers who prefer to keep their entire workflow in the editor.

## Tech Stack

### Core Technologies
- **Rust** (Edition 2021) - Primary language for both WASM extension and LSP server
- **Zed Extension API** (v0.7.0) - Zed editor integration
- **WebAssembly (WASM)** - Extension runtime format for Zed

### Language Server
- **tower-lsp** (v0.20) - LSP protocol implementation
- **tokio** (v1.35) - Async runtime for LSP server
- **dashmap** (v5.5) - Concurrent HashMap for LSP state management
- **lsp-types** (v0.95) - LSP type definitions

### HTTP & Networking
- **reqwest** (v0.11) - HTTP client with JSON, gzip, brotli, and deflate support
- **url** (v2.5) - URL parsing and manipulation

### Data Processing
- **serde** & **serde_json** (v1.0) - JSON serialization/deserialization
- **regex** (v1.10) - Pattern matching for variable substitution and parsing
- **base64** (v0.21) - Basic auth encoding
- **uuid** (v1.7) - GUID generation for system variables
- **chrono** (v0.4) - Timestamp generation for system variables

### Tree-sitter Grammar
- Custom HTTP grammar for syntax highlighting and parsing
- Node.js-based tree-sitter CLI for grammar compilation

### Testing & Development
- **tokio** (test runtime)
- **tempfile** - Temporary file handling in tests
- **serial_test** - Sequential test execution
- **wiremock**, **mockito**, **httpmock** - HTTP mocking frameworks
- **proptest** - Property-based testing
- **criterion** - Benchmarking framework with HTML reports

## Project Conventions

### Code Style

#### General Principles
- **High-verbosity code**: Prioritize clarity and readability over brevity
- **Explicit over implicit**: Use descriptive names, avoid abbreviations
- **Early returns**: Use guard clauses and handle errors/edge cases first
- **Minimal nesting**: Keep nesting levels to 2-3 maximum

#### Naming Conventions
- **Functions**: verb/verb-phrases (e.g., `execute_request`, `parse_http_file`, `format_response`)
- **Variables**: descriptive nouns/noun-phrases (e.g., `http_request`, `active_environment`, `response_body`)
- **NO single-letter variables** except in very limited iterator contexts
- **NO abbreviations**: `generateDateString` not `genYmdStr`, `numSuccessfulRequests` not `n`
- **Types**: PascalCase (e.g., `HttpRequest`, `EnvironmentSession`, `AuthScheme`)
- **Modules**: snake_case (e.g., `language_server`, `code_generation`, `request_parser`)

#### Rust-Specific
- **Explicit type annotations** for function signatures and public APIs
- **No type annotations** for trivially inferred local variables
- **Avoid `unwrap()`** in production code - use proper error handling with `Result` and `Option`
- **Match exhaustively** - no wildcard patterns unless truly necessary
- **Document public APIs** with doc comments (`///`)

#### Comments
- **Minimal comments** - code should be self-documenting through good naming
- **Comments explain "why"**, not "what" or "how"
- **Module-level docs** (`//!`) for architectural overview
- **No inline comments** - place above the line being explained
- **No TODO comments** - create issues or implement immediately

### Architecture Patterns

#### Modular Organization
The codebase follows a clear separation of concerns:

- **`models/`** - Core data structures (`HttpRequest`, `HttpResponse`, `Variable`, etc.)
- **`parser/`** - `.http` file parsing logic (request extraction, header parsing, body detection)
- **`executor/`** - HTTP request execution with `reqwest`
- **`formatter/`** - Response formatting (JSON, XML, HTML, plain text)
- **`variables/`** - Variable substitution (system vars like `{{$timestamp}}`, environment vars)
- **`environment/`** - Environment management (dev/staging/prod switching)
- **`language_server/`** - LSP provider for code lenses, diagnostics, hover, completion
- **`lsp_server/`** - Standalone LSP server binary (uses `tower-lsp`)
- **`commands/`** - Slash command handlers for Zed integration
- **`auth/`** - Authentication schemes (Basic, Bearer)
- **`codegen/`** - Code generation (JavaScript/fetch, JavaScript/axios, Python/requests)
- **`curl/`** - cURL import/export
- **`graphql/`** - GraphQL query/mutation handling
- **`history/`** - Request/response history tracking
- **`ui/`** - User interface components for environment switching, code generation dialogs

#### Design Patterns
- **Builder pattern** for complex request construction
- **Strategy pattern** for formatter selection based on content type
- **State management** via `Arc<Mutex<T>>` for thread-safe shared state (environment session)
- **Error propagation** using `Result<T, E>` throughout
- **Feature flags** (`lsp` feature for conditional LSP server compilation)

#### Extension Architecture
- **WASM extension** (`lib.rs`) implements Zed extension API for editor integration
- **Separate LSP binary** (`bin/lsp_server.rs`) runs as standalone process
- **Dual-purpose crate**: Library for WASM + optional binary for LSP

### Testing Strategy

#### Test Organization
- **Unit tests** inline with modules (`#[cfg(test)] mod tests`)
- **Integration tests** in `tests/` directory
- **Benchmarks** in `benches/` directory (parser, formatter, variable substitution)

#### Test Categories
1. **Parser tests** - Validate HTTP request parsing for various formats
2. **Executor tests** - Mock HTTP requests with wiremock/mockito
3. **Formatter tests** - Ensure correct content-type detection and formatting
4. **Variable substitution tests** - Test system vars, env vars, request chaining
5. **LSP integration tests** - Code lenses, diagnostics, completion
6. **End-to-end tests** - Full request → response workflows
7. **Property-based tests** - Using proptest for edge case discovery

#### Testing Requirements
- **Mock external HTTP calls** - Use wiremock/mockito/httpmock
- **Serial execution** for tests that modify shared state (use `#[serial]`)
- **Comprehensive error cases** - Test malformed requests, network errors, timeouts

### Git Workflow

#### Branching Strategy
- **`main`** branch for stable releases
- Feature branches for new capabilities
- Use descriptive branch names: `feature/graphql-support`, `fix/variable-substitution-bug`

#### Commit Conventions
Follow conventional commits format:
- `feat: Add GraphQL mutation support`
- `fix: Resolve variable substitution in request body`
- `docs: Update environment configuration guide`
- `perf: Optimize request parser for large files`
- `test: Add integration tests for request chaining`
- `refactor: Extract auth logic into separate module`

#### Release Process
- Documented in `RELEASE_CHECKLIST.md`
- Versioning follows SemVer
- Changelog maintained in `CHANGELOG.md`

## Domain Context

### HTTP Client Semantics
- **Request blocks** are delimited by `###` separators in `.http` files
- **Variable syntax**: `{{variableName}}` for substitution
- **System variables**: `{{$guid}}`, `{{$timestamp}}`, `{{$randomInt}}`, `{{$processEnv VAR_NAME}}`
- **Request chaining**: Capture response values with `@name("requestName")` and reference with JSONPath

### Environment Files
- **`.http-client-env.json`** - Environment variable definitions
- Structure: `{ "development": { "key": "value" }, "production": { "key": "value" } }`
- **Shared variables** apply across all environments

### LSP Features
- **Code lenses** - "Send Request" buttons inline in editor
- **Diagnostics** - Real-time validation of HTTP syntax
- **Hover** - Variable value preview on hover
- **Completion** - Variable autocompletion with `{{`

### File Format
- Supports **`.http`** and **`.rest`** file extensions
- Standard HTTP message format:
  ```
  METHOD URL
  Header-Name: Header-Value
  
  Request Body
  ```

## Important Constraints

### Technical Constraints
- **WASM compilation** - Extension must compile to WebAssembly (no native OS APIs)
- **LSP as separate binary** - LSP server runs as standalone process (not in WASM)
- **Zed Extension API limitations** - Must work within Zed's extension capabilities
- **No built-in HTTP client in WASM** - Extension delegates to LSP server for actual requests
- **File system access** via Zed API only

### Performance Requirements
- **Fast parsing** - Large `.http` files must parse quickly (see `benches/`)
- **Non-blocking LSP** - Asynchronous request execution
- **Optimized builds** - Release profile uses LTO, single codegen unit, strip symbols

### Security Constraints
- **Never hardcode secrets** - Use environment variables for API keys
- **SSL validation** configurable but enabled by default
- **Auth credentials** in Authorization headers or secure environment files

### Compatibility
- **Rust 2021 edition** minimum
- **Zed Extension API v0.7.0+**
- **VS Code REST Client syntax compatibility** (migration path from VS Code)

## External Dependencies

### Zed Editor Integration
- **Zed Extension API** - Core extension host interface
- **LSP protocol** - Communication between extension and LSP server
- **Tree-sitter** - Syntax highlighting grammar

### HTTP Services
- **Any HTTP/HTTPS endpoint** - Extension executes requests against user-specified URLs
- **GraphQL endpoints** - Special handling for GraphQL queries/mutations

### Build Tools
- **Cargo** - Rust build system
- **wasm-pack** (implied) - WASM compilation
- **tree-sitter-cli** - Grammar compilation for syntax highlighting

### Runtime Dependencies
- **Rust toolchain** - Required for building from source
- **Shell scripts** - Installation scripts (`install-dev.sh`, `build-lsp.sh`)
- **PowerShell scripts** - Windows installation support (`.ps1` files)

### Documentation Tools
- Markdown-based documentation in `docs/`
- Examples in `examples/` directory

## File Organization

```
rest-client/
├── src/                      # Main Rust source
│   ├── lib.rs               # WASM extension entry point
│   ├── auth/                # Authentication handlers
│   ├── codegen/             # Code generation (JS, Python)
│   ├── commands/            # Slash command handlers
│   ├── config/              # Configuration management
│   ├── curl/                # cURL import/export
│   ├── environment/         # Environment switching
│   ├── executor/            # HTTP request execution
│   ├── formatter/           # Response formatting
│   ├── graphql/             # GraphQL support
│   ├── history/             # Request history
│   ├── language_server/     # LSP provider
│   ├── lsp_server/          # LSP server implementation
│   ├── models/              # Core data structures
│   ├── parser/              # HTTP file parsing
│   ├── ui/                  # UI components
│   └── variables/           # Variable substitution
├── tests/                   # Integration tests
├── benches/                 # Performance benchmarks
├── docs/                    # User documentation
├── examples/                # Example .http files
├── grammars/http/           # Tree-sitter grammar
├── languages/http/          # Language configuration
└── openspec/                # Project specifications
```
