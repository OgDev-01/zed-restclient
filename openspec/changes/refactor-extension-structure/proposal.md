# Change: Refactor Extension Structure for Zed Compatibility

## Why

The REST Client extension has several fundamental structural issues that prevent it from working correctly as a Zed extension:

1. **Duplicate codebase**: There are TWO complete Rust implementations (`src/` and `grammars/http/src/`) with separate `register_extension!` macros, causing compilation and loading conflicts
2. **Invalid grammar configuration**: The `extension.toml` uses an absolute `file://` path that only works on the original developer's machine
3. **Duplicate extension manifests**: Both `extension.toml` and `grammars/http/extension.toml` define complete extensions
4. **LSP binary cannot be located**: The extension expects `./lsp-server` which doesn't work in Zed's sandbox environment
5. **Duplicate highlight queries**: Both `languages/http/highlights.scm` and `languages/http/queries/highlights.scm` exist
6. **HTTP status code limitation undocumented**: Zed's HTTP client doesn't return status codes, silently assuming 200 OK

These issues make the extension non-functional for anyone except the original developer and violate Zed extension architecture patterns.

## What Changes

### Structure Cleanup
- **BREAKING**: Remove entire `grammars/http/src/` directory (duplicate Rust implementation)
- **BREAKING**: Remove `grammars/http/extension.toml` (duplicate manifest)
- **BREAKING**: Remove `grammars/http/Cargo.toml` (duplicate crate config)
- Remove `languages/http/highlights.scm` (keep only `queries/highlights.scm`)
- Clean up `grammars/http/` to only contain tree-sitter grammar files

### Grammar Configuration
- Update `extension.toml` to use proper grammar reference (GitHub URL or relative path)
- Document local development workflow for grammar iteration
- Ensure grammar compiles to WASM correctly via tree-sitter CLI

### LSP Binary Handling
- Implement runtime download of LSP binary from GitHub releases
- Add version checking and caching in extension work directory
- Provide fallback error messages when binary unavailable
- Document manual installation as alternative

### HTTP Client Limitations
- Document that Zed's HTTP client doesn't provide status codes
- Update response formatting to clearly indicate this limitation
- Consider alternative approaches (LSP-based execution)

### Configuration Fixes
- Fix `extension.toml` language server configuration
- Ensure `languages/http/config.toml` aligns with grammar name
- Validate all cross-references between config files

## Impact

- **Affected specs**: 
  - `extension-structure` (new capability)
  - `grammar-configuration` (new capability)
  - `lsp-integration` (new capability)
  - `http-execution` (new capability)

- **Affected code**:
  - `extension.toml` - Grammar and LSP configuration
  - `Cargo.toml` - Remove rlib crate type if present
  - `src/lib.rs` - LSP binary download logic
  - `grammars/http/` - Directory cleanup
  - `languages/http/` - Highlight query consolidation

- **Breaking changes**:
  - Users must reinstall extension after restructure
  - Grammar repository URL will change
  - LSP binary location/installation method changes

- **User impact**:
  - Extension will actually install and work for all users
  - Clear documentation of limitations
  - Proper LSP features (if binary available)

## Success Criteria

1. Extension installs successfully via "Install Dev Extension" on any machine
2. Syntax highlighting works for `.http` and `.rest` files
3. Slash commands (`/send-request`, `/switch-environment`, etc.) function correctly
4. Grammar configuration uses distributable URL (not local file path)
5. No duplicate `register_extension!` calls in codebase
6. LSP binary downloads automatically or provides clear error message
7. All configuration files pass `openspec validate --strict`

## Out of Scope

- Adding new features (GraphQL improvements, new auth schemes)
- Performance optimizations
- UI/UX improvements beyond error messaging
- Supporting additional languages in code generation