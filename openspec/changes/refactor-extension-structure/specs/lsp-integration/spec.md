## ADDED Requirements

### Requirement: LSP Binary Download Mechanism
The extension SHALL automatically download the LSP server binary from GitHub releases when the language server is first needed.

#### Scenario: First-time LSP binary download
- **WHEN** a user opens an `.http` file for the first time after installation
- **AND** no LSP binary exists in the extension work directory
- **THEN** the extension SHALL download the appropriate binary from GitHub releases
- **AND** the binary SHALL be cached for future use

#### Scenario: Platform-specific binary selection
- **WHEN** the LSP binary download is initiated
- **THEN** the extension SHALL detect the current platform (OS and architecture)
- **AND** download the correct binary variant:
  - macOS x64: `lsp-server-x86_64-apple-darwin`
  - macOS arm64: `lsp-server-aarch64-apple-darwin`
  - Linux x64: `lsp-server-x86_64-unknown-linux-gnu`
  - Windows x64: `lsp-server-x86_64-pc-windows-msvc.exe`

#### Scenario: Binary caching
- **WHEN** the LSP binary has been downloaded previously
- **THEN** subsequent requests SHALL use the cached binary
- **AND** no network request SHALL be made

### Requirement: LSP Binary Executable Permissions
The extension SHALL ensure the downloaded LSP binary has executable permissions on Unix-like systems.

#### Scenario: Unix executable permissions
- **WHEN** the LSP binary is downloaded on macOS or Linux
- **THEN** the binary SHALL have executable permission set (chmod +x equivalent)
- **AND** the binary SHALL be launchable by the extension

#### Scenario: Windows binary execution
- **WHEN** the LSP binary is downloaded on Windows
- **THEN** the binary SHALL have `.exe` extension
- **AND** no additional permission changes SHALL be required

### Requirement: LSP Download Error Handling
The extension SHALL gracefully handle failures during LSP binary download and provide clear error messages to users.

#### Scenario: Network failure during download
- **WHEN** the LSP binary download fails due to network issues
- **THEN** the extension SHALL display a user-friendly error message
- **AND** the extension SHALL continue to function without LSP features
- **AND** syntax highlighting and slash commands SHALL still work

#### Scenario: Missing GitHub release
- **WHEN** the expected binary is not found in GitHub releases
- **THEN** the extension SHALL log the error with details
- **AND** suggest manual installation as an alternative

#### Scenario: Download retry capability
- **WHEN** a previous download attempt failed
- **AND** the user reopens an `.http` file
- **THEN** the extension SHALL retry the download
- **AND** not permanently give up after a single failure

### Requirement: LSP Version Management
The extension SHALL track LSP binary versions and update when necessary.

#### Scenario: Version checking on extension update
- **WHEN** the extension is updated to a new version
- **THEN** the extension SHALL check if a newer LSP binary is required
- **AND** download the new binary if the version has changed

#### Scenario: Version mismatch detection
- **WHEN** the cached LSP binary version doesn't match the expected version
- **THEN** the extension SHALL download the correct version
- **AND** replace the outdated binary

### Requirement: Language Server Command Configuration
The extension SHALL correctly configure the language server command in `extension.toml` and implement `language_server_command()` to return the binary path.

#### Scenario: Extension manifest configuration
- **WHEN** `extension.toml` is parsed
- **THEN** the `[language_servers.rest-client-lsp]` section SHALL exist
- **AND** `name` SHALL be set to `"REST Client LSP"`
- **AND** `languages` SHALL include `["HTTP"]`

#### Scenario: Language server command return
- **WHEN** Zed calls `language_server_command()` on the extension
- **THEN** the extension SHALL return a `zed::Command` with:
  - `command`: path to the downloaded LSP binary
  - `args`: empty vector (no arguments required)
  - `env`: empty vector (no special environment variables)

#### Scenario: Binary not available fallback
- **WHEN** `language_server_command()` is called but binary download failed
- **THEN** the extension SHALL return an appropriate error
- **AND** Zed SHALL disable LSP features gracefully

### Requirement: Manual LSP Installation Fallback
The extension SHALL document and support manual installation of the LSP binary as a fallback option.

#### Scenario: Manual installation path
- **WHEN** a user installs the LSP binary manually
- **AND** places it in a location on their PATH
- **THEN** the extension SHALL detect and use the manually installed binary
- **AND** skip the download process

#### Scenario: Manual installation documentation
- **WHEN** automatic download fails repeatedly
- **THEN** the error message SHALL include instructions for manual installation
- **AND** link to the GitHub releases page

### Requirement: LSP Feature Availability
The extension SHALL provide core functionality even when LSP features are unavailable.

#### Scenario: Extension without LSP
- **WHEN** the LSP binary is not available (download failed or not installed)
- **THEN** the following features SHALL still work:
  - Syntax highlighting for `.http` and `.rest` files
  - Slash commands (`/send-request`, `/switch-environment`, etc.)
  - Variable substitution in requests
- **AND** the following features SHALL be unavailable:
  - Code lenses ("Send Request" buttons)
  - Variable autocompletion
  - Hover information for variables
  - Real-time diagnostics

#### Scenario: Clear feature degradation messaging
- **WHEN** LSP features are unavailable
- **THEN** the extension SHALL NOT produce errors in normal operation
- **AND** users SHALL be informed that advanced features require the LSP server