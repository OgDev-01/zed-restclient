## ADDED Requirements

### Requirement: Grammar Repository Configuration
The extension SHALL configure its tree-sitter grammar via a publicly accessible Git repository URL in `extension.toml`.

#### Scenario: Valid grammar repository URL
- **WHEN** `extension.toml` is parsed
- **THEN** the `[grammars.http]` section SHALL contain a `repository` field
- **AND** the repository URL SHALL use `https://` protocol (not `file://`)
- **AND** the URL SHALL point to a valid, accessible Git repository

#### Scenario: Grammar revision pinning
- **WHEN** the grammar configuration is read
- **THEN** a `rev` field SHALL be present specifying a Git commit hash or branch name
- **AND** the revision SHALL exist in the specified repository

#### Scenario: Grammar fetching by Zed
- **WHEN** a user installs the extension via Zed
- **THEN** Zed SHALL be able to fetch the grammar from the specified repository
- **AND** the grammar SHALL compile to WASM successfully

### Requirement: Grammar and Language Name Alignment
The grammar name in `extension.toml` SHALL match the grammar reference in `languages/http/config.toml`.

#### Scenario: Consistent grammar naming
- **WHEN** `extension.toml` defines `[grammars.http]`
- **THEN** `languages/http/config.toml` SHALL have `grammar = "http"`
- **AND** the `name` field in config.toml SHALL be `"HTTP"`

#### Scenario: Language server language reference
- **WHEN** `extension.toml` defines `[language_servers.rest-client-lsp]`
- **THEN** the `languages` array SHALL contain `"HTTP"` (matching config.toml name)

### Requirement: Tree-sitter Grammar Source Files
The `grammars/http/` directory SHALL contain valid tree-sitter grammar source files that can be compiled by the tree-sitter CLI.

#### Scenario: Required grammar files present
- **WHEN** the `grammars/http/` directory is inspected
- **THEN** it SHALL contain:
  - `grammar.js` (tree-sitter grammar definition)
  - `package.json` (with tree-sitter-cli as dependency)

#### Scenario: Grammar compilation
- **WHEN** `tree-sitter generate` is run in `grammars/http/`
- **THEN** the command SHALL complete successfully
- **AND** `src/parser.c` SHALL be generated

#### Scenario: WASM grammar compilation
- **WHEN** `tree-sitter build --wasm` is run
- **THEN** a valid WASM binary SHALL be produced
- **AND** the binary SHALL be usable by Zed for syntax parsing

### Requirement: Highlight Query Configuration
Syntax highlighting queries SHALL be defined in the standard tree-sitter location within the language directory.

#### Scenario: Single highlight query location
- **WHEN** the `languages/http/` directory is inspected
- **THEN** highlight queries SHALL exist at `queries/highlights.scm`
- **AND** no `highlights.scm` file SHALL exist at the root of `languages/http/`

#### Scenario: Valid highlight query syntax
- **WHEN** `languages/http/queries/highlights.scm` is parsed
- **THEN** it SHALL contain valid tree-sitter query syntax
- **AND** all referenced node types SHALL exist in the grammar

#### Scenario: Injection queries for embedded languages
- **WHEN** the extension supports embedded languages (JSON in request body)
- **THEN** `languages/http/queries/injections.scm` SHALL define injection rules
- **AND** the injection patterns SHALL correctly identify embedded content

### Requirement: Local Development Workflow
The extension SHALL support local grammar development without requiring pushes to a remote repository for every change.

#### Scenario: Development with local grammar
- **WHEN** a developer is iterating on grammar changes locally
- **THEN** they SHALL be able to test grammar changes by rebuilding the WASM
- **AND** documentation SHALL explain the local development workflow

#### Scenario: Publishing grammar changes
- **WHEN** grammar changes are ready for distribution
- **THEN** the developer SHALL push changes to the grammar repository
- **AND** update the `rev` field in `extension.toml` to the new commit hash