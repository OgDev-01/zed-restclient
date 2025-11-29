# Capability: Extension Installation

## ADDED Requirements

### Requirement: WASM Binary Compilation
The extension build system SHALL compile the Rust codebase to a WebAssembly binary compatible with Zed's extension runtime.

#### Scenario: Single crate type configuration
- **WHEN** Cargo.toml is configured with `crate-type = ["cdylib"]` only
- **THEN** the WASM binary SHALL compile successfully to `target/wasm32-wasip1/release/rest_client.wasm`
- **AND** the binary size SHALL be approximately 1.7-2.0 MB
- **AND** the binary SHALL be a valid WebAssembly module (version 0x1)

#### Scenario: Mixed crate types rejected
- **WHEN** Cargo.toml contains multiple crate types (e.g., `["cdylib", "lib"]`)
- **THEN** the installation process SHALL NOT hang indefinitely
- **AND** documentation SHALL warn against mixed crate type configurations

### Requirement: Development Installation Workflow
The extension SHALL support installation via Zed's "Install Dev Extension" feature for local development and testing.

#### Scenario: Successful dev extension installation
- **WHEN** a user selects "Install Dev Extension" from Zed's command palette
- **AND** navigates to the extension directory containing a valid extension.toml and WASM binary
- **THEN** the installation SHALL complete within 60 seconds
- **AND** the extension SHALL appear in the Extensions panel with "Installed" or "Enabled" status
- **AND** no error messages SHALL be displayed during installation

#### Scenario: Installation progress visibility
- **WHEN** installation is in progress
- **THEN** Zed SHALL display an "Installing..." status message
- **AND** the user SHALL be able to monitor progress via `zed --foreground` verbose logging
- **AND** installation completion SHALL be clearly indicated

### Requirement: Extension Activation
After successful installation, the extension SHALL activate and register its language support without requiring manual intervention.

#### Scenario: HTTP language registration
- **WHEN** the extension is installed and Zed is restarted
- **THEN** the HTTP language SHALL be registered for `.http` and `.rest` file extensions
- **AND** opening an `.http` file SHALL display "HTTP" in the language status bar
- **AND** syntax highlighting SHALL be active for HTTP methods, URLs, and headers

#### Scenario: Extension persistence
- **WHEN** the extension is installed successfully
- **THEN** the extension SHALL remain installed across Zed restarts
- **AND** the installation SHALL persist until explicitly removed or updated

### Requirement: Installation Error Handling
The extension installation process SHALL provide clear feedback when errors occur and SHALL NOT leave the editor in an unresponsive state.

#### Scenario: Invalid WASM binary
- **WHEN** the WASM binary is missing or corrupted
- **THEN** installation SHALL fail with a descriptive error message
- **AND** the Extensions panel SHALL display the error status
- **AND** Zed SHALL remain responsive and usable

#### Scenario: Configuration validation
- **WHEN** extension.toml has invalid syntax or missing required fields
- **THEN** installation SHALL fail before attempting to load the WASM module
- **AND** error messages SHALL indicate which fields are invalid or missing

### Requirement: Build Toolchain Requirements
The extension build process SHALL require standard Rust tooling with WASM target support.

#### Scenario: Prerequisites check
- **WHEN** building the extension from source
- **THEN** Rust toolchain (rustc and cargo) MUST be installed
- **AND** the wasm32-wasip1 target MUST be installed via `rustup target add wasm32-wasip1`
- **AND** build scripts SHALL validate these prerequisites before attempting compilation

#### Scenario: Clean build process
- **WHEN** running `cargo clean` followed by `cargo build --target wasm32-wasip1 --release`
- **THEN** all build artifacts SHALL be regenerated cleanly
- **AND** the WASM binary SHALL be reproducible across builds
- **AND** no cached artifacts SHALL cause installation conflicts

### Requirement: Installation Debugging Support
The extension SHALL support debugging workflows to diagnose installation issues.

#### Scenario: Verbose logging enabled
- **WHEN** Zed is launched with `zed --foreground` flag
- **THEN** detailed installation logs SHALL be output to the terminal
- **AND** logs SHALL include extension compilation status, WASM loading messages, and any error stack traces
- **AND** timestamps SHALL be included for performance analysis

#### Scenario: Log file persistence
- **WHEN** installation occurs in normal (non-foreground) mode
- **THEN** installation events SHALL be logged to `~/Library/Logs/Zed/Zed.log` (macOS) or equivalent platform path
- **AND** logs SHALL be searchable by extension name "rest-client"
- **AND** error messages SHALL include sufficient context for troubleshooting