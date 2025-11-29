## 1. Cleanup Duplicate Code
- [x] 1.1 Delete entire `grammars/http/src/` directory (duplicate Rust implementation)
- [x] 1.2 Delete `grammars/http/Cargo.toml` (duplicate crate configuration)
- [x] 1.3 Delete `grammars/http/extension.toml` (duplicate extension manifest)
- [x] 1.4 Delete redundant documentation in `grammars/http/` (CHANGELOG.md, README.md, etc.)
- [x] 1.5 Delete `languages/http/highlights.scm` (keep only `queries/highlights.scm`)
- [x] 1.6 Verify only one `register_extension!` macro exists in codebase
- [x] 1.7 Run `cargo build --target wasm32-wasip1` to confirm WASM still compiles

## 2. Clean Up Grammars Directory
- [x] 2.1 Ensure `grammars/http/` contains only tree-sitter files (grammar.js, package.json, src/)
- [x] 2.2 Remove any Rust-specific files from grammars directory
- [x] 2.3 Verify `grammars/http/grammar.js` is valid tree-sitter grammar
- [ ] 2.4 Run `tree-sitter generate` to regenerate parser if needed
- [x] 2.5 Verify `grammars/http.wasm` exists and is valid

## 3. Fix Extension Manifest
- [x] 3.1 Update `extension.toml` grammar repository URL (remove `file://` local path)
- [x] 3.2 Set grammar `rev` to valid Git commit hash or branch name
- [x] 3.3 Verify `languages` array points to correct path (`languages/http`)
- [x] 3.4 Verify `language_servers` configuration matches language name exactly
- [x] 3.5 Validate extension.toml syntax with TOML parser

## 4. Fix Language Configuration
- [x] 4.1 Verify `languages/http/config.toml` has correct `name` and `grammar` fields
- [x] 4.2 Ensure `grammar` field in config.toml matches grammar name in extension.toml
- [x] 4.3 Verify `path_suffixes` includes both `http` and `rest`
- [x] 4.4 Consolidate highlight queries in `languages/http/queries/highlights.scm`
- [x] 4.5 Verify `languages/http/queries/injections.scm` is valid if present

## 5. Grammar Repository Setup
- [ ] 5.1 Create separate GitHub repository for tree-sitter-http (or identify existing)
- [ ] 5.2 Push grammar files (grammar.js, package.json, src/) to repository
- [ ] 5.3 Tag initial release with semantic version
- [ ] 5.4 Update extension.toml with GitHub URL and commit hash
- [ ] 5.5 Test grammar installation via Zed extension system

## 6. LSP Binary Download Implementation
- [x] 6.1 Create `src/lsp_download.rs` module for download logic
- [x] 6.2 Implement function to check if LSP binary exists in work directory
- [x] 6.3 Implement function to download binary from GitHub releases
- [x] 6.4 Implement platform detection (macOS x64/arm64, Linux x64/arm64, Windows x64)
- [x] 6.5 Implement binary extraction and permission setting
- [x] 6.6 Add version checking to re-download when extension updates
- [x] 6.7 Update `language_server_command()` in `src/lib.rs` to use download module
- [x] 6.8 Add error handling with user-friendly messages

## 7. LSP Binary Release Pipeline
- [ ] 7.1 Create GitHub Actions workflow for LSP binary builds
- [ ] 7.2 Build binaries for all target platforms (macOS x64, macOS arm64, Linux x64, Windows x64)
- [ ] 7.3 Create GitHub release with binary assets
- [ ] 7.4 Document release process in RELEASE_CHECKLIST.md
- [ ] 7.5 Test binary download on each platform

## 8. HTTP Execution Limitations
- [x] 8.1 Update response formatter to indicate unknown status code
- [x] 8.2 Add "Known Limitations" section to README.md
- [x] 8.3 Document that Zed HTTP client doesn't return status codes
- [x] 8.4 Add inline comment in executor code explaining limitation
- [ ] 8.5 Consider adding LSP-based execution as future enhancement (document in design.md)

## 9. Documentation Updates
- [x] 9.1 Update README.md installation instructions
- [x] 9.2 Remove references to local file paths in documentation
- [x] 9.3 Add troubleshooting section for common installation issues
- [x] 9.4 Document LSP binary manual installation as fallback
- [x] 9.5 Update CHANGELOG.md with restructuring changes
- [ ] 9.6 Update project.md if architecture patterns changed

## 10. Validation and Testing
- [x] 10.1 Run `cargo build --target wasm32-wasip1 --release` successfully
- [ ] 10.2 Run `cargo test` and ensure all tests pass
- [ ] 10.3 Install extension via "Install Dev Extension" in Zed
- [ ] 10.4 Verify syntax highlighting works for `.http` files
- [ ] 10.5 Test `/send-request` slash command
- [ ] 10.6 Test `/switch-environment` slash command
- [ ] 10.7 Test `/generate-code` slash command
- [ ] 10.8 Test `/paste-curl` and `/copy-as-curl` commands
- [ ] 10.9 Verify LSP features work (if binary downloads successfully)
- [ ] 10.10 Test on clean machine (no prior installation)

## 11. Cleanup and Finalization
- [ ] 11.1 Remove any unused dependencies from Cargo.toml
- [ ] 11.2 Run `cargo clippy` and fix any warnings
- [ ] 11.3 Run `cargo fmt` to ensure consistent formatting
- [ ] 11.4 Verify no compiler warnings in release build
- [x] 11.5 Update version number in extension.toml and Cargo.toml
- [ ] 11.6 Create Git tag for release
- [ ] 11.7 Run `openspec validate refactor-extension-structure --strict`
