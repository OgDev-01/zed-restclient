## ADDED Requirements

### Requirement: Single Extension Entry Point
The extension SHALL have exactly one Rust crate that compiles to WASM and registers with Zed via a single `register_extension!` macro call.

#### Scenario: Unique extension registration
- **WHEN** the extension codebase is searched for `register_extension!`
- **THEN** exactly one occurrence SHALL be found in `src/lib.rs`

#### Scenario: No duplicate implementations
- **WHEN** the `grammars/` directory is inspected
- **THEN** it SHALL NOT contain any Rust source files (`*.rs`), `Cargo.toml`, or `extension.toml`

### Requirement: Correct Directory Structure
The extension SHALL follow Zed's standard extension directory structure with clear separation between extension code, language definitions, and grammar sources.

#### Scenario: Extension root structure
- **WHEN** the extension root directory is listed
- **THEN** it SHALL contain:
  - `extension.toml` (single extension manifest)
  - `Cargo.toml` (WASM crate configuration)
  - `src/` (Rust source code)
  - `languages/` (language definitions)
  - `grammars/` (tree-sitter grammar sources only)

#### Scenario: Languages directory structure
- **WHEN** the `languages/http/` directory is inspected
- **THEN** it SHALL contain:
  - `config.toml` (language configuration)
  - `queries/highlights.scm` (syntax highlighting queries)
- **AND** it SHALL NOT contain a root-level `highlights.scm` file (only in `queries/` subdirectory)

#### Scenario: Grammars directory structure
- **WHEN** the `grammars/http/` directory is inspected
- **THEN** it SHALL contain only tree-sitter grammar files:
  - `grammar.js` (grammar definition)
  - `package.json` (Node.js configuration)
  - `src/` (generated parser files)
- **AND** it SHALL NOT contain Rust crate files (`Cargo.toml`, `src/*.rs`, `extension.toml`)

### Requirement: WASM-Only Crate Type
The extension Cargo.toml SHALL specify only `cdylib` crate type for WASM compilation compatibility.

#### Scenario: Correct crate type configuration
- **WHEN** `Cargo.toml` is parsed
- **THEN** the `[lib]` section SHALL have `crate-type = ["cdylib"]`
- **AND** it SHALL NOT include `"lib"` or `"rlib"` in the crate-type array

#### Scenario: WASM compilation succeeds
- **WHEN** `cargo build --target wasm32-wasip1 --release` is executed
- **THEN** the build SHALL complete successfully
- **AND** `target/wasm32-wasip1/release/rest_client.wasm` SHALL be created

### Requirement: No Hardcoded Local Paths
The extension configuration SHALL NOT contain absolute file paths or paths specific to a single developer's machine.

#### Scenario: No file:// URLs in extension.toml
- **WHEN** `extension.toml` is inspected
- **THEN** it SHALL NOT contain any `file://` URLs
- **AND** grammar repository URLs SHALL use `https://` protocol

#### Scenario: Portable path references
- **WHEN** the extension is installed on a different machine
- **THEN** all path references SHALL resolve correctly
- **AND** no errors SHALL occur due to missing local paths