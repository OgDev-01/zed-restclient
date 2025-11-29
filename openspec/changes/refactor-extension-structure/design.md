## Context

The REST Client extension for Zed has accumulated structural debt that prevents it from functioning correctly. The extension was developed with patterns that don't align with Zed's extension architecture, resulting in:

- Duplicate code that causes compilation conflicts
- Hard-coded paths that only work on one developer's machine
- Missing infrastructure for LSP binary distribution
- Undocumented limitations in HTTP execution

This design document outlines the technical decisions for restructuring the extension to follow Zed's extension patterns and work correctly for all users.

## Goals / Non-Goals

### Goals
- Create a clean, single-purpose extension structure that follows Zed conventions
- Enable the extension to be installed and work on any user's machine
- Provide clear documentation of capabilities and limitations
- Establish proper grammar and LSP binary distribution mechanisms
- Maintain all existing functionality during restructure

### Non-Goals
- Adding new features or capabilities
- Changing the HTTP request/response format
- Modifying the tree-sitter grammar syntax
- Performance optimization (separate effort)
- Supporting WASM-based HTTP execution with full status codes (Zed API limitation)

## Decisions

### Decision 1: Remove Duplicate Codebase in `grammars/http/src/`

**What**: Delete the entire `grammars/http/src/` directory and related files (`Cargo.toml`, `extension.toml`)

**Why**: 
- Having two `register_extension!` calls causes undefined behavior
- The `grammars/` directory should only contain tree-sitter grammar source files
- Maintaining duplicate code leads to drift and confusion

**Alternatives considered**:
- Keep both and use feature flags → Rejected: adds complexity, violates Zed patterns
- Merge the two implementations → Rejected: they're identical, no benefit

### Decision 2: Grammar Repository Strategy

**What**: Use a relative local path for grammar during development, with instructions for publishing to GitHub for distribution

**Why**:
- Local `file://` URLs don't work for other users
- Zed fetches grammars from Git repositories at install time
- The grammar needs its own repository for proper versioning

**Implementation**:
```toml
# For development (local testing):
[grammars.http]
repository = "https://github.com/ogdev-01/tree-sitter-http"
rev = "main"

# The tree-sitter grammar files stay in grammars/http/ during development
# but must be pushed to a separate repo for distribution
```

**Alternatives considered**:
- Embed grammar WASM directly → Rejected: Zed doesn't support this pattern
- Use existing tree-sitter-http repo → Considered: may work if compatible with our syntax

### Decision 3: LSP Binary Distribution via GitHub Releases

**What**: Implement download logic in `language_server_command()` to fetch the LSP binary from GitHub releases

**Why**:
- Zed extensions run in a sandbox without access to arbitrary file paths
- The `./lsp-server` relative path doesn't work
- This is the standard pattern used by other Zed extensions (rust-analyzer, etc.)

**Implementation approach**:
```rust
fn language_server_command(&mut self, ...) -> Result<zed::Command> {
    // 1. Check if binary exists in extension work directory
    // 2. If not, download from GitHub releases
    // 3. Make executable and return path
    
    let binary_path = self.download_lsp_binary_if_needed()?;
    Ok(zed::Command {
        command: binary_path,
        args: vec![],
        env: vec![],
    })
}
```

**Alternatives considered**:
- Require manual installation → Poor UX, but document as fallback
- Bundle binary in extension → Increases extension size significantly
- Skip LSP entirely → Loses code lens, completion, hover features

### Decision 4: Consolidate Highlight Queries

**What**: Keep only `languages/http/queries/highlights.scm`, remove `languages/http/highlights.scm`

**Why**:
- Having two files causes confusion about which is authoritative
- The `queries/` subdirectory is the tree-sitter standard location
- Zed looks in `queries/` by default

### Decision 5: Document HTTP Status Code Limitation

**What**: Add clear documentation and UI indication that status codes are not available

**Why**:
- Zed's `http_client` API doesn't return status codes
- Current code silently assumes 200 OK for all responses
- Users need to understand this limitation

**Implementation**:
- Update response formatter to show "Status: Unknown (Zed API limitation)"
- Add note in README under "Known Limitations"
- Consider LSP-based execution as future enhancement (LSP can use reqwest)

### Decision 6: Directory Structure After Refactor

```
rest-client/
├── extension.toml              # Single extension manifest
├── Cargo.toml                  # WASM crate (cdylib only)
├── src/
│   ├── lib.rs                  # Extension entry point
│   ├── lsp_download.rs         # NEW: LSP binary download logic
│   └── ... (existing modules)
├── languages/
│   └── http/
│       ├── config.toml         # Language configuration
│       └── queries/
│           ├── highlights.scm  # Syntax highlighting (single location)
│           └── injections.scm  # Embedded language support
├── grammars/
│   └── http/
│       ├── grammar.js          # Tree-sitter grammar definition
│       ├── package.json        # Node.js config for tree-sitter CLI
│       └── src/                # Generated parser (from tree-sitter generate)
│           ├── parser.c
│           └── ...
└── lsp-server/                 # Separate directory for LSP binary source
    ├── Cargo.toml              # Native binary crate
    └── src/
        └── main.rs
```

## Risks / Trade-offs

### Risk 1: Grammar Repository Dependency
- **Risk**: Users need grammar repo to be accessible during installation
- **Mitigation**: Publish to GitHub, use stable commit hashes, document offline installation

### Risk 2: LSP Download Failures
- **Risk**: Network issues prevent LSP binary download
- **Mitigation**: 
  - Cache downloaded binary
  - Provide clear error messages
  - Document manual installation steps
  - Extension works without LSP (just loses advanced features)

### Risk 3: Breaking Existing Installations
- **Risk**: Users with current (broken) installation may have issues
- **Mitigation**: 
  - Bump version number
  - Document clean installation process
  - Provide uninstall instructions

### Trade-off: LSP Binary Size
- Downloading ~3MB binary on first use adds latency
- Alternative (bundling) would bloat extension package
- Chosen approach: download on demand, cache indefinitely

## Migration Plan

### Phase 1: Cleanup (Non-Breaking)
1. Delete `grammars/http/src/`, `grammars/http/Cargo.toml`, `grammars/http/extension.toml`
2. Delete `languages/http/highlights.scm`
3. Delete redundant documentation files in `grammars/http/`
4. Verify WASM build still works

### Phase 2: Grammar Repository
1. Create separate `tree-sitter-http` repository (or use existing)
2. Push grammar files to repository
3. Update `extension.toml` with GitHub URL and commit hash
4. Test grammar compilation via Zed

### Phase 3: LSP Download Implementation
1. Add `lsp_download.rs` module with download logic
2. Update `language_server_command()` to use download mechanism
3. Create GitHub release workflow for LSP binary
4. Test end-to-end LSP functionality

### Phase 4: Documentation
1. Update README with accurate installation instructions
2. Document known limitations (status codes)
3. Add troubleshooting guide for common issues
4. Update CHANGELOG

### Rollback Plan
- If Phase 2/3 cause issues, can revert to "LSP disabled" mode
- Grammar can temporarily use local path during development
- Each phase is independently deployable

## Open Questions

1. **Existing tree-sitter-http grammar**: Should we use https://github.com/rest-nvim/tree-sitter-http or create our own? Need to verify syntax compatibility.

2. **LSP binary hosting**: GitHub releases vs. dedicated CDN? GitHub releases is simpler and sufficient for now.

3. **Cross-platform binaries**: Need to build and release for macOS (x64, arm64), Linux (x64), Windows (x64). CI/CD pipeline required.

4. **Version synchronization**: How to keep extension version and LSP binary version in sync? Consider embedding expected version in extension code.

5. **Fallback for HTTP execution**: Should we add LSP-based HTTP execution to get proper status codes? This would be a separate feature proposal.