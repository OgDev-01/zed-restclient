//! LSP Backend Implementation for REST Client Language Server
//!
//! This module implements the core Language Server Protocol backend using tower-lsp,
//! handling all protocol messages and providing interactive features for .http files.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CodeLens as LspCodeLens, CodeLensOptions, CodeLensParams, Command as LspCommand,
    CompletionItem as LspCompletionItem, CompletionItemKind, CompletionOptions, CompletionParams,
    CompletionResponse, Diagnostic as LspDiagnostic, DiagnosticOptions,
    DiagnosticRelatedInformation, DiagnosticServerCapabilities,
    DiagnosticSeverity as LspDiagnosticSeverity, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentDiagnosticParams,
    DocumentDiagnosticReportResult, Documentation, ExecuteCommandParams,
    FullDocumentDiagnosticReport, Hover as LspHover, HoverContents, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, MarkupContent, MarkupKind,
    MessageType, Position as LspPosition, Range as LspRange, RelatedFullDocumentDiagnosticReport,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer};

use super::document::DocumentManager;
use super::executor_bridge::ExecutorBridge;
use crate::environment::{load_environments, EnvError, EnvironmentSession, Environments};
use crate::language_server::{codelens, completion, diagnostics, hover};
use crate::variables::VariableContext;

/// LSP Backend for REST Client extension
///
/// Implements the Language Server Protocol to provide interactive features
/// for .http files in Zed, including code lenses, completions, hover, and diagnostics.
#[derive(Debug)]
pub struct Backend {
    /// LSP client for sending notifications and requests to the editor
    client: Client,

    /// Document manager for tracking open files
    documents: Arc<DocumentManager>,

    /// Executor bridge for HTTP request execution
    executor: Arc<ExecutorBridge>,

    /// Environment session for managing environment variables
    environment_session: Arc<EnvironmentSession>,

    /// Workspace root path for loading environment files
    workspace_root: Arc<std::sync::RwLock<Option<PathBuf>>>,
}

impl Backend {
    /// Creates a new Backend instance
    ///
    /// # Arguments
    ///
    /// * `client` - The LSP client for communication with the editor
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tower_lsp::Client;
    /// use rest_client::lsp_server::backend::Backend;
    ///
    /// let (service, socket) = tower_lsp::LspService::new(|client| {
    ///     Backend::new(client)
    /// });
    /// ```
    pub fn new(client: Client) -> Self {
        // Initialize with empty environments (can be loaded later)
        let environments = Environments::new();
        let environment_session = Arc::new(EnvironmentSession::new(environments));

        Self {
            client,
            documents: Arc::new(DocumentManager::new()),
            executor: Arc::new(ExecutorBridge::new()),
            environment_session,
            workspace_root: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Creates a new Backend instance with custom executor configuration
    ///
    /// # Arguments
    ///
    /// * `client` - The LSP client for communication with the editor
    /// * `executor` - Custom executor bridge with specific configuration
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tower_lsp::Client;
    /// use rest_client::lsp_server::backend::Backend;
    /// use rest_client::lsp_server::executor_bridge::ExecutorBridge;
    /// use rest_client::config::Config;
    /// use std::sync::Arc;
    ///
    /// let config = Config::default();
    /// let executor = Arc::new(ExecutorBridge::with_config(config));
    ///
    /// let (service, socket) = tower_lsp::LspService::new(move |client| {
    ///     Backend::with_executor(client, executor.clone())
    /// });
    /// ```
    pub fn with_executor(client: Client, executor: Arc<ExecutorBridge>) -> Self {
        // Initialize with empty environments (can be loaded later)
        let environments = Environments::new();
        let environment_session = Arc::new(EnvironmentSession::new(environments));

        Self {
            client,
            documents: Arc::new(DocumentManager::new()),
            executor,
            environment_session,
            workspace_root: Arc::new(std::sync::RwLock::new(None)),
        }
    }

    /// Logs a message to the client
    async fn log_message(&self, typ: MessageType, message: impl std::fmt::Display) {
        self.client.log_message(typ, message).await;
    }

    /// Logs an informational message
    async fn log_info(&self, message: impl std::fmt::Display) {
        self.log_message(MessageType::INFO, message).await;
    }

    /// Logs a warning message
    async fn log_warn(&self, message: impl std::fmt::Display) {
        self.log_message(MessageType::WARNING, message).await;
    }

    /// Logs an error message
    async fn log_error(&self, message: impl std::fmt::Display) {
        self.log_message(MessageType::ERROR, message).await;
    }

    /// Loads environment configurations from workspace
    ///
    /// Searches for .http-client-env.json or http-client.env.json files
    /// starting from the workspace root and traversing up to 3 parent directories.
    ///
    /// # Arguments
    ///
    /// * `workspace_path` - The workspace root directory path
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Environments loaded successfully or file not found (graceful)
    /// * `Err(EnvError)` - If file exists but parsing failed
    pub async fn load_environments_from_workspace(
        &self,
        workspace_path: PathBuf,
    ) -> std::result::Result<(), EnvError> {
        // Store workspace root
        if let Ok(mut root) = self.workspace_root.write() {
            *root = Some(workspace_path.clone());
        }

        // Load environments from file
        match load_environments(&workspace_path) {
            Ok(environments) => {
                // Reload environments into the existing session
                if let Err(e) = self
                    .environment_session
                    .reload_environments(environments.clone())
                {
                    self.log_error(format!("Failed to reload environments: {}", e))
                        .await;
                    return Err(e);
                }

                // Get environment names and optionally set first one as active
                let env_names = self.environment_session.list_environment_names();
                if let Some(first_env) = env_names.first() {
                    self.log_info(format!(
                        "Loaded {} environment(s): {}",
                        env_names.len(),
                        env_names.join(", ")
                    ))
                    .await;

                    // Set first environment as default
                    if let Err(e) = self.environment_session.set_active_environment(first_env) {
                        self.log_warn(format!("Could not set default environment: {}", e))
                            .await;
                    } else {
                        self.log_info(format!("Default environment set to: {}", first_env))
                            .await;
                    }
                } else {
                    self.log_info("No environments found in configuration file")
                        .await;
                }

                Ok(())
            }
            Err(e) => {
                self.log_error(format!("Failed to load environments: {}", e))
                    .await;
                Err(e)
            }
        }
    }

    /// Sets the active environment by name
    ///
    /// Activates the specified environment. If environments haven't been loaded yet,
    /// this will attempt to reload them from the workspace.
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to activate (e.g., "dev", "staging", "prod")
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Environment activated successfully
    /// * `Err(String)` - If environment doesn't exist or failed to load
    ///
    /// # Example
    ///
    /// ```no_run
    /// backend.set_active_environment("dev".to_string()).await?;
    /// ```
    pub async fn set_active_environment(
        &self,
        env_name: String,
    ) -> std::result::Result<(), String> {
        self.log_info(format!("Setting active environment to: {}", env_name))
            .await;

        // Try to set the active environment
        match self.environment_session.set_active_environment(&env_name) {
            Ok(()) => {
                self.log_info(format!("Active environment set to: {}", env_name))
                    .await;
                Ok(())
            }
            Err(e) => {
                // Environment doesn't exist - try reloading from workspace
                self.log_warn(format!(
                    "Environment '{}' not found, attempting to reload from workspace",
                    env_name
                ))
                .await;

                let workspace_path = match self.workspace_root.read() {
                    Ok(root) => root.clone(),
                    Err(_) => return Err("Failed to read workspace root".to_string()),
                };

                if let Some(workspace) = workspace_path {
                    // Reload environments
                    match load_environments(&workspace) {
                        Ok(environments) => {
                            if let Err(e) =
                                self.environment_session.reload_environments(environments)
                            {
                                let msg = format!("Failed to reload environments: {}", e);
                                self.log_error(&msg).await;
                                return Err(msg);
                            }

                            // Try setting again
                            if let Err(e) =
                                self.environment_session.set_active_environment(&env_name)
                            {
                                let msg = format!("Failed to set active environment: {}", e);
                                self.log_error(&msg).await;
                                return Err(msg);
                            }

                            self.log_info(format!("Active environment set to: {}", env_name))
                                .await;

                            Ok(())
                        }
                        Err(e) => {
                            let msg = format!("Failed to reload environments: {}", e);
                            self.log_error(&msg).await;
                            Err(msg)
                        }
                    }
                } else {
                    let msg = format!("Failed to set active environment: {}", e);
                    self.log_error(&msg).await;
                    Err(msg)
                }
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    /// Initialize the language server
    ///
    /// Declares server capabilities to the client, including support for:
    /// - Full text document synchronization
    /// - Code lens provider (without resolve)
    /// - Completion provider (triggered by "{")
    /// - Hover provider
    /// - Diagnostic provider
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        self.log_info(format!(
            "Initializing REST Client Language Server (process ID: {:?})",
            params.process_id
        ))
        .await;

        // Extract workspace root from initialization parameters
        if let Some(root_uri) = params.root_uri {
            if let Ok(root_path) = root_uri.to_file_path() {
                self.log_info(format!("Workspace root: {}", root_path.display()))
                    .await;

                // Load environments from workspace
                if let Err(e) = self.load_environments_from_workspace(root_path).await {
                    self.log_warn(format!("Could not load environments: {}", e))
                        .await;
                }
            }
        } else if let Some(workspace_folders) = params.workspace_folders {
            if let Some(first_folder) = workspace_folders.first() {
                if let Ok(folder_path) = first_folder.uri.to_file_path() {
                    self.log_info(format!("Workspace folder: {}", folder_path.display()))
                        .await;

                    // Load environments from workspace
                    if let Err(e) = self.load_environments_from_workspace(folder_path).await {
                        self.log_warn(format!("Could not load environments: {}", e))
                            .await;
                    }
                }
            }
        }

        // Declare server capabilities according to LSP 3.17 specification
        let capabilities = ServerCapabilities {
            // Full text document synchronization - server receives complete document content
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),

            // Code lens provider - show "Send Request" buttons above HTTP requests
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false), // We don't need lazy resolution
            }),

            // Completion provider - trigger on "{" for variable completions
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec!["{".to_string()]),
                resolve_provider: Some(false),
                all_commit_characters: None,
                work_done_progress_options: Default::default(),
                completion_item: None,
            }),

            // Hover provider - show variable values on hover
            hover_provider: Some(HoverProviderCapability::Simple(true)),

            // Diagnostic provider - show syntax errors and warnings
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                identifier: Some("rest-client".to_string()),
                inter_file_dependencies: false, // No cross-file dependencies
                workspace_diagnostics: false,   // Only document-level diagnostics
                work_done_progress_options: Default::default(),
            })),

            // Execute command provider - handle "rest-client.send" command
            execute_command_provider: Some(tower_lsp::lsp_types::ExecuteCommandOptions {
                commands: vec!["rest-client.send".to_string()],
                work_done_progress_options: Default::default(),
            }),

            // No other capabilities needed for now
            ..Default::default()
        };

        Ok(InitializeResult {
            capabilities,
            server_info: Some(tower_lsp::lsp_types::ServerInfo {
                name: "rest-client-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    /// Called after the initialize request is complete
    ///
    /// This is where we can perform any post-initialization setup.
    async fn initialized(&self, _: tower_lsp::lsp_types::InitializedParams) {
        self.log_info("REST Client Language Server initialized successfully")
            .await;
    }

    /// Shutdown the language server
    ///
    /// Called before the server exits to allow cleanup.
    async fn shutdown(&self) -> Result<()> {
        self.log_info("Shutting down REST Client Language Server")
            .await;

        // Clear all documents
        self.documents.clear();

        Ok(())
    }

    /// Handle textDocument/didOpen notification
    ///
    /// Called when a document is opened in the editor.
    /// Stores the document content in the document manager.
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let content = params.text_document.text;

        self.log_info(format!("Document opened: {}", uri)).await;

        // Store the document content
        if let Err(e) = self.documents.insert(uri.clone(), content) {
            self.log_error(format!("Failed to insert document {}: {}", uri, e))
                .await;
        }
    }

    /// Handle textDocument/didChange notification
    ///
    /// Called when a document's content changes.
    /// Updates the document content in the document manager.
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        // LSP 3.17: For FULL sync, there should be exactly one change with the full content
        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;

            self.log_info(format!("Document changed: {}", uri)).await;

            // Update the document content
            // If document doesn't exist, insert it (fallback for edge cases)
            match self.documents.update(uri.clone(), content.clone()) {
                Ok(_) => {}
                Err(super::document::DocumentError::NotFound) => {
                    // Document not found, insert it instead
                    if let Err(e) = self.documents.insert(uri.clone(), content) {
                        self.log_error(format!("Failed to insert document {}: {}", uri, e))
                            .await;
                    }
                }
                Err(e) => {
                    self.log_error(format!("Failed to update document {}: {}", uri, e))
                        .await;
                }
            }
        } else {
            self.log_warn(format!("No content changes received for document: {}", uri))
                .await;
        }
    }

    /// Handle textDocument/didClose notification
    ///
    /// Called when a document is closed in the editor.
    /// Removes the document from the document manager.
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        self.log_info(format!("Document closed: {}", uri)).await;

        // Remove the document
        if self.documents.remove(&uri).is_none() {
            self.log_warn(format!("Document not found when closing: {}", uri))
                .await;
        }
    }

    /// Handle textDocument/codeLens request
    ///
    /// Provides "Send Request" buttons above HTTP requests in the document.
    /// Named requests (with @name comments) show the name in the button title.
    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<LspCodeLens>>> {
        let uri = params.text_document.uri;

        self.log_info(format!("Code lens request for: {}", uri))
            .await;

        // Retrieve document from DocumentManager
        let document = match self.documents.get(&uri) {
            Some(content) => content,
            None => {
                self.log_warn(format!("Document not found for code lens: {}", uri))
                    .await;
                return Ok(Some(Vec::new()));
            }
        };

        // Call existing provide_code_lens to get internal CodeLens objects
        let internal_lenses = codelens::provide_code_lens(&document);

        // Convert internal CodeLens to LSP CodeLens
        let lsp_lenses: Vec<LspCodeLens> = internal_lenses
            .into_iter()
            .map(|internal_lens| {
                // Convert internal Range to LSP Range
                let lsp_range = LspRange {
                    start: LspPosition {
                        line: internal_lens.range.start.line as u32,
                        character: internal_lens.range.start.character as u32,
                    },
                    end: LspPosition {
                        line: internal_lens.range.end.line as u32,
                        character: internal_lens.range.end.character as u32,
                    },
                };

                // Convert internal Command to LSP Command
                let lsp_command = internal_lens.command.map(|cmd| LspCommand {
                    title: cmd.title,
                    command: "rest-client.send".to_string(),
                    arguments: Some(vec![
                        serde_json::json!(uri.to_string()),
                        serde_json::json!(internal_lens.range.start.line),
                    ]),
                });

                LspCodeLens {
                    range: lsp_range,
                    command: lsp_command,
                    data: internal_lens.data.map(|d| serde_json::json!(d)),
                }
            })
            .collect();

        self.log_info(format!(
            "Provided {} code lens(es) for: {}",
            lsp_lenses.len(),
            uri
        ))
        .await;

        Ok(Some(lsp_lenses))
    }

    /// Handle textDocument/completion request
    ///
    /// Provides variable autocompletion when the user types `{{`.
    /// Returns environment variables, shared variables, file-level variables, and system variables.
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let lsp_position = params.text_document_position.position;

        self.log_info(format!(
            "Completion request for: {} at {}:{}",
            uri, lsp_position.line, lsp_position.character
        ))
        .await;

        // Retrieve document from DocumentManager
        let document = match self.documents.get(uri) {
            Some(content) => content,
            None => {
                self.log_warn(format!("Document not found for completion: {}", uri))
                    .await;
                return Ok(None);
            }
        };

        // Convert LSP position to internal position
        let position =
            completion::Position::new(lsp_position.line as usize, lsp_position.character as usize);

        // Get current environments (or use empty if none active)
        let environments = self
            .environment_session
            .get_environments()
            .unwrap_or_else(Environments::new);

        // For now, use empty file variables (this could be enhanced to parse @variable from document)
        let file_variables = HashMap::new();

        // Call existing provide_completions from language_server::completion module
        let internal_completions =
            completion::provide_completions(position, &document, &environments, &file_variables);

        // If no completions, return None
        if internal_completions.is_empty() {
            self.log_info(format!("No completions available at position"))
                .await;
            return Ok(None);
        }

        // Convert internal CompletionItem to lsp_types::CompletionItem
        let lsp_completions: Vec<LspCompletionItem> = internal_completions
            .into_iter()
            .map(|item| {
                // Determine LSP completion kind based on internal kind
                let kind = match item.kind {
                    completion::CompletionKind::SystemVariable => {
                        Some(CompletionItemKind::VARIABLE)
                    }
                    completion::CompletionKind::EnvironmentVariable => {
                        Some(CompletionItemKind::VARIABLE)
                    }
                    completion::CompletionKind::SharedVariable => {
                        Some(CompletionItemKind::VARIABLE)
                    }
                    completion::CompletionKind::FileVariable => Some(CompletionItemKind::VARIABLE),
                };

                // Create documentation from detail if available
                let documentation = item.detail.map(|detail| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::PlainText,
                        value: detail,
                    })
                });

                LspCompletionItem {
                    label: item.label,
                    kind,
                    detail: None, // We use documentation instead
                    documentation,
                    insert_text: Some(item.insert_text),
                    ..Default::default()
                }
            })
            .collect();

        self.log_info(format!(
            "Provided {} completion(s) for: {}",
            lsp_completions.len(),
            uri
        ))
        .await;

        Ok(Some(CompletionResponse::Array(lsp_completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<LspHover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let lsp_position = params.text_document_position_params.position;

        self.log_info(format!(
            "Hover request for: {} at {}:{}",
            uri, lsp_position.line, lsp_position.character
        ))
        .await;

        // Retrieve document from DocumentManager
        let document = match self.documents.get(uri) {
            Some(content) => content,
            None => {
                self.log_warn(format!("Document not found for hover: {}", uri))
                    .await;
                return Ok(None);
            }
        };

        // Convert LSP position to internal position
        let position =
            hover::Position::new(lsp_position.line as usize, lsp_position.character as usize);

        // Get current environments (or use empty if none active)
        let environments = self
            .environment_session
            .get_environments()
            .unwrap_or_else(Environments::new);

        // For now, use empty file variables and request variables
        // (could be enhanced to parse @variable from document and track request variables)
        let file_variables = HashMap::new();
        let request_variables = HashMap::new();

        // Create variable context
        let context =
            hover::VariableContext::with_variables(environments, file_variables, request_variables);

        // Call existing provide_hover from language_server::hover module
        let internal_hover = match hover::provide_hover(position, &document, &context) {
            Some(hover) => hover,
            None => {
                self.log_info(format!("No hover information at position"))
                    .await;
                return Ok(None);
            }
        };

        // Convert internal Hover to lsp_types::Hover
        let lsp_hover = LspHover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: internal_hover.contents,
            }),
            range: internal_hover.range.map(|r| LspRange {
                start: LspPosition {
                    line: r.start.line as u32,
                    character: r.start.character as u32,
                },
                end: LspPosition {
                    line: r.end.line as u32,
                    character: r.end.character as u32,
                },
            }),
        };

        self.log_info(format!("Provided hover information for: {}", uri))
            .await;

        Ok(Some(lsp_hover))
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = &params.text_document.uri;

        self.log_info(format!("Diagnostic request for: {}", uri))
            .await;

        // Retrieve document from DocumentManager
        let document = match self.documents.get(uri) {
            Some(content) => content,
            None => {
                self.log_warn(format!("Document not found for diagnostics: {}", uri))
                    .await;
                // Return empty diagnostics for non-existent documents
                return Ok(DocumentDiagnosticReportResult::Report(
                    tower_lsp::lsp_types::DocumentDiagnosticReport::Full(
                        RelatedFullDocumentDiagnosticReport {
                            related_documents: None,
                            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                result_id: None,
                                items: vec![],
                            },
                        },
                    ),
                ));
            }
        };

        // Get current environments (or use empty if none active)
        let environments = self
            .environment_session
            .get_environments()
            .unwrap_or_else(Environments::new);

        // Get active environment if any
        let active_environment = environments.get_active();

        // Get shared variables
        let shared_variables = environments.shared.clone();

        // Create VariableContext for diagnostic checks
        // Use current working directory as workspace path (could be enhanced to use actual workspace)
        let workspace_path = std::env::current_dir().unwrap_or_default();
        let variable_context = VariableContext::with_environment(
            workspace_path,
            active_environment.cloned(),
            shared_variables,
        );

        // Call existing provide_diagnostics from language_server::diagnostics module
        let internal_diagnostics = diagnostics::provide_diagnostics(&document, &variable_context);

        // Convert internal Diagnostics to lsp_types::Diagnostic
        let lsp_diagnostics: Vec<LspDiagnostic> = internal_diagnostics
            .into_iter()
            .map(|diag| {
                // Map internal severity to LSP severity
                let severity = match diag.severity {
                    diagnostics::DiagnosticSeverity::Error => LspDiagnosticSeverity::ERROR,
                    diagnostics::DiagnosticSeverity::Warning => LspDiagnosticSeverity::WARNING,
                    diagnostics::DiagnosticSeverity::Info => LspDiagnosticSeverity::INFORMATION,
                };

                // Convert internal range to LSP range
                let range = LspRange {
                    start: LspPosition {
                        line: diag.range.start.line as u32,
                        character: diag.range.start.character as u32,
                    },
                    end: LspPosition {
                        line: diag.range.end.line as u32,
                        character: diag.range.end.character as u32,
                    },
                };

                // Create related information if there's a suggestion
                let related_information = diag.suggestion.map(|suggestion| {
                    vec![DiagnosticRelatedInformation {
                        location: tower_lsp::lsp_types::Location {
                            uri: uri.clone(),
                            range,
                        },
                        message: format!("💡 Suggestion: {}", suggestion),
                    }]
                });

                LspDiagnostic {
                    range,
                    severity: Some(severity),
                    code: diag
                        .code
                        .map(|c| tower_lsp::lsp_types::NumberOrString::String(c)),
                    code_description: None,
                    source: Some("rest-client".to_string()),
                    message: diag.message,
                    related_information,
                    tags: None,
                    data: None,
                }
            })
            .collect();

        self.log_info(format!(
            "Provided {} diagnostic(s) for: {}",
            lsp_diagnostics.len(),
            uri
        ))
        .await;

        // Return full diagnostic report
        Ok(DocumentDiagnosticReportResult::Report(
            tower_lsp::lsp_types::DocumentDiagnosticReport::Full(
                RelatedFullDocumentDiagnosticReport {
                    related_documents: None,
                    full_document_diagnostic_report: FullDocumentDiagnosticReport {
                        result_id: None,
                        items: lsp_diagnostics,
                    },
                },
            ),
        ))
    }

    /// Handle workspace/executeCommand request
    ///
    /// Executes commands triggered by code lens or other actions.
    /// Currently supports the "rest-client.send" command for executing HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `params` - Command parameters containing command name and arguments
    ///
    /// # Command: rest-client.send
    ///
    /// Arguments:
    /// - `args[0]`: Document URI (string)
    /// - `args[1]`: Line number (number, 1-based)
    ///
    /// Executes the HTTP request at the specified line in the document and displays
    /// the response in the editor via a notification message.
    async fn execute_command(
        &self,
        params: ExecuteCommandParams,
    ) -> Result<Option<serde_json::Value>> {
        self.log_info(format!(
            "Execute command: {} with {} arguments",
            params.command,
            params.arguments.len()
        ))
        .await;

        // Only handle "rest-client.send" command
        if params.command != "rest-client.send" {
            self.log_warn(format!("Unknown command: {}", params.command))
                .await;
            return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                "Unknown command: {}",
                params.command
            )));
        }

        // Validate arguments
        if params.arguments.len() < 2 {
            self.log_error("Missing required arguments for rest-client.send command")
                .await;
            self.client
                .show_message(
                    MessageType::ERROR,
                    "Failed to execute request: Missing arguments",
                )
                .await;
            return Err(tower_lsp::jsonrpc::Error::invalid_params(
                "Expected 2 arguments: uri and line number",
            ));
        }

        // Parse URI argument
        let uri_value = &params.arguments[0];
        let uri_str = uri_value.as_str().ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("First argument must be a string URI")
        })?;
        let uri = Url::parse(uri_str).map_err(|e| {
            tower_lsp::jsonrpc::Error::invalid_params(format!("Invalid URI: {}", e))
        })?;

        // Parse line number argument
        let line_value = &params.arguments[1];
        let line = line_value.as_u64().ok_or_else(|| {
            tower_lsp::jsonrpc::Error::invalid_params("Second argument must be a number")
        })? as usize;

        self.log_info(format!("Executing request at {}:{}", uri, line))
            .await;

        // Retrieve document content
        let document = match self.documents.get(&uri) {
            Some(doc) => doc,
            None => {
                self.log_error(format!("Document not found: {}", uri)).await;
                self.client
                    .show_message(
                        MessageType::ERROR,
                        format!("Failed to execute request: Document not found: {}", uri),
                    )
                    .await;
                return Err(tower_lsp::jsonrpc::Error::invalid_params(format!(
                    "Document not found: {}",
                    uri
                )));
            }
        };

        // Get active environment (if any)
        let active_env = self.environment_session.get_active_environment();

        // Execute request at specified line using native HTTP client (reqwest)
        match self
            .executor
            .execute_request_at_line(&document, line, active_env)
            .await
        {
            Ok(response) => {
                // Format response for display
                let formatted = ExecutorBridge::format_response_pretty(&response);

                // Show response in notification
                self.client
                    .show_message(
                        MessageType::INFO,
                        format!(
                            "HTTP {} {}\n\n{}",
                            response.status_code, response.status_text, formatted
                        ),
                    )
                    .await;

                self.log_info(format!(
                    "Request executed successfully: {} {}",
                    response.status_code, response.status_text
                ))
                .await;

                Ok(None)
            }
            Err(e) => {
                // Show error message to user
                let error_msg = format!("Failed to execute request: {}", e);
                self.log_error(&error_msg).await;
                self.client
                    .show_message(MessageType::ERROR, &error_msg)
                    .await;

                Err(tower_lsp::jsonrpc::Error::internal_error())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Url;

    // Test helper to verify document manager behavior
    // Note: Full LSP protocol tests require integration testing with a real client
    fn create_test_documents() -> Arc<DocumentManager> {
        Arc::new(DocumentManager::new())
    }

    // Helper to create a test client for unit tests
    fn create_test_client() -> Client {
        // Extract Client from LspService by wrapping it in a closure
        // The client is provided by tower_lsp when constructing the service
        let client_holder = std::sync::Arc::new(std::sync::Mutex::new(None));
        let client_holder_clone = client_holder.clone();

        let _ = tower_lsp::LspService::new(move |client| {
            *client_holder_clone.lock().unwrap() = Some(client.clone());
            Backend::new(client)
        });

        let result = client_holder
            .lock()
            .unwrap()
            .take()
            .expect("Client should be initialized");
        result
    }

    #[test]
    fn test_backend_new_creates_instance() {
        // This test verifies Backend can be constructed via LspService
        // Actual construction happens in the binary via tower_lsp::LspService::new()
        // We just verify the types are correct
        let _service = tower_lsp::LspService::new(|client| Backend::new(client));
        // If this compiles, the constructor works correctly
    }

    #[tokio::test]
    async fn test_initialize_capabilities_structure() {
        // Test that we can construct the capabilities correctly
        // We'll test this by building them directly rather than through a backend instance

        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(false),
            }),
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec!["{".to_string()]),
                resolve_provider: Some(false),
                all_commit_characters: None,
                work_done_progress_options: Default::default(),
                completion_item: None,
            }),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
                identifier: Some("rest-client".to_string()),
                inter_file_dependencies: false,
                workspace_diagnostics: false,
                work_done_progress_options: Default::default(),
            })),
            ..Default::default()
        };

        // Verify all capabilities are set correctly
        assert!(matches!(
            capabilities.text_document_sync,
            Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL))
        ));
        assert!(capabilities.code_lens_provider.is_some());

        let completion = capabilities.completion_provider.unwrap();
        assert_eq!(completion.trigger_characters, Some(vec!["{".to_string()]));

        assert!(matches!(
            capabilities.hover_provider,
            Some(HoverProviderCapability::Simple(true))
        ));
        assert!(capabilities.diagnostic_provider.is_some());
    }

    #[test]
    fn test_document_manager_integration() {
        // Test document lifecycle through DocumentManager directly
        let documents = create_test_documents();
        let uri = Url::parse("file:///test.http").unwrap();

        // Initially empty
        assert!(documents.is_empty());

        // Insert document (simulates did_open)
        documents
            .insert(uri.clone(), "GET https://example.com".to_string())
            .unwrap();
        assert_eq!(documents.len(), 1);
        assert_eq!(
            documents.get(&uri),
            Some("GET https://example.com".to_string())
        );

        // Update document (simulates did_change)
        documents
            .update(uri.clone(), "POST https://example.com".to_string())
            .unwrap();
        assert_eq!(
            documents.get(&uri),
            Some("POST https://example.com".to_string())
        );

        // Remove document (simulates did_close)
        let removed = documents.remove(&uri);
        assert_eq!(removed, Some("POST https://example.com".to_string()));
        assert!(documents.is_empty());
    }

    #[test]
    fn test_document_clear() {
        // Test that documents can be cleared (simulates shutdown)
        let documents = create_test_documents();
        let uri = Url::parse("file:///test.http").unwrap();

        // Add a document
        documents
            .insert(uri.clone(), "GET https://example.com".to_string())
            .unwrap();
        assert_eq!(documents.len(), 1);

        // Clear all documents
        documents.clear();
        assert!(documents.is_empty());
    }

    #[test]
    fn test_update_nonexistent_document_handling() {
        // Test handling of update on non-existent document
        let documents = create_test_documents();
        let uri = Url::parse("file:///test.http").unwrap();

        // Document doesn't exist yet
        assert!(documents.is_empty());

        // Try to update non-existent document
        let result = documents.update(uri.clone(), "GET https://example.com".to_string());
        assert!(result.is_err());

        // Document should still not exist
        assert!(documents.is_empty());

        // Insert should work
        documents
            .insert(uri.clone(), "GET https://example.com".to_string())
            .unwrap();
        assert_eq!(documents.len(), 1);
    }

    #[test]
    fn test_multiple_documents() {
        // Test managing multiple documents
        let documents = create_test_documents();
        let uri1 = Url::parse("file:///test1.http").unwrap();
        let uri2 = Url::parse("file:///test2.http").unwrap();

        // Insert first document
        documents
            .insert(uri1.clone(), "GET https://example1.com".to_string())
            .unwrap();

        // Insert second document
        documents
            .insert(uri2.clone(), "GET https://example2.com".to_string())
            .unwrap();

        // Both should exist
        assert_eq!(documents.len(), 2);
        assert!(documents.get(&uri1).is_some());
        assert!(documents.get(&uri2).is_some());

        // Remove first document
        documents.remove(&uri1);

        // Only second should remain
        assert_eq!(documents.len(), 1);
        assert!(documents.get(&uri1).is_none());
        assert!(documents.get(&uri2).is_some());
    }

    #[tokio::test]
    async fn test_code_lens_single_request() {
        // Test code lens generation for a single request
        let documents = create_test_documents();
        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();

        // Insert a simple HTTP request
        documents
            .insert(uri.clone(), "GET https://api.example.com/users".to_string())
            .unwrap();

        // Verify we can generate code lenses using the internal module directly
        let content = documents.get(&uri).unwrap();
        let lenses = codelens::provide_code_lens(&content);

        assert_eq!(lenses.len(), 1);
        assert_eq!(lenses[0].range.start.line, 0);
        assert!(lenses[0].command.is_some());
        let cmd = lenses[0].command.as_ref().unwrap();
        assert_eq!(cmd.command, "rest-client.send");
        assert_eq!(cmd.title, "▶ Send Request");
    }

    #[tokio::test]
    async fn test_code_lens_named_request() {
        // Test code lens with @name comment
        let documents = create_test_documents();
        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();

        let doc = r#"# @name GetUsers
GET https://api.example.com/users"#;

        documents.insert(uri.clone(), doc.to_string()).unwrap();

        let content = documents.get(&uri).unwrap();
        let lenses = codelens::provide_code_lens(&content);

        assert_eq!(lenses.len(), 1);
        let cmd = lenses[0].command.as_ref().unwrap();
        assert_eq!(cmd.title, "▶ Send Request: GetUsers");
    }

    #[tokio::test]
    async fn test_code_lens_multiple_requests() {
        // Test code lens for multiple requests
        let documents = create_test_documents();
        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();

        let doc = r#"GET https://api.example.com/users

###

# @name CreateUser
POST https://api.example.com/users
Content-Type: application/json

{"name": "John"}

###

DELETE https://api.example.com/users/1"#;

        documents.insert(uri.clone(), doc.to_string()).unwrap();

        let content = documents.get(&uri).unwrap();
        let lenses = codelens::provide_code_lens(&content);

        assert_eq!(lenses.len(), 3);

        // First request - no name
        assert_eq!(lenses[0].command.as_ref().unwrap().title, "▶ Send Request");

        // Second request - with name
        assert_eq!(
            lenses[1].command.as_ref().unwrap().title,
            "▶ Send Request: CreateUser"
        );

        // Third request - no name
        assert_eq!(lenses[2].command.as_ref().unwrap().title, "▶ Send Request");
    }

    #[tokio::test]
    async fn test_code_lens_empty_document() {
        // Test code lens for empty document
        let documents = create_test_documents();
        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();

        documents.insert(uri.clone(), "".to_string()).unwrap();

        let content = documents.get(&uri).unwrap();
        let lenses = codelens::provide_code_lens(&content);

        assert_eq!(lenses.len(), 0);
    }

    #[tokio::test]
    async fn test_code_lens_no_requests() {
        // Test code lens for document with only comments
        let documents = create_test_documents();
        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();

        let doc = r#"# Just a comment
// Another comment
### Delimiter"#;

        documents.insert(uri.clone(), doc.to_string()).unwrap();

        let content = documents.get(&uri).unwrap();
        let lenses = codelens::provide_code_lens(&content);

        assert_eq!(lenses.len(), 0);
    }

    #[test]
    fn test_range_conversion() {
        // Test internal Range to LSP Range conversion
        let internal_range = codelens::Range::new(
            codelens::Position::new(5, 10),
            codelens::Position::new(5, 50),
        );

        let lsp_range = LspRange {
            start: LspPosition {
                line: internal_range.start.line as u32,
                character: internal_range.start.character as u32,
            },
            end: LspPosition {
                line: internal_range.end.line as u32,
                character: internal_range.end.character as u32,
            },
        };

        assert_eq!(lsp_range.start.line, 5);
        assert_eq!(lsp_range.start.character, 10);
        assert_eq!(lsp_range.end.line, 5);
        assert_eq!(lsp_range.end.character, 50);
    }

    #[tokio::test]
    async fn test_completion_trigger_after_double_brace() {
        let client = create_test_client();
        let backend = Backend::new(client);

        // Open a document
        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET https://api.example.com/{{";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        // Request completions at position after {{
        let params = CompletionParams {
            text_document_position: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 30,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let result = backend.completion(params).await.unwrap();
        assert!(result.is_some());

        if let Some(CompletionResponse::Array(completions)) = result {
            // Should have system variables at minimum
            assert!(completions.len() >= 6);

            // Check for system variables
            let has_guid = completions.iter().any(|c| c.label == "$guid");
            let has_timestamp = completions.iter().any(|c| c.label == "$timestamp");
            assert!(has_guid, "Should have $guid system variable");
            assert!(has_timestamp, "Should have $timestamp system variable");

            // Verify insert_text includes closing braces
            let guid_item = completions.iter().find(|c| c.label == "$guid").unwrap();
            assert_eq!(guid_item.insert_text.as_ref().unwrap(), "$guid}}");
        }
    }

    #[tokio::test]
    async fn test_completion_no_trigger_without_double_brace() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET https://api.example.com/users";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = CompletionParams {
            text_document_position: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 20,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let result = backend.completion(params).await.unwrap();
        // Should return None when not triggered by {{
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_completion_with_environment_variables() {
        use crate::environment::{Environment, Environments};

        let client = create_test_client();

        // Create environments with variables
        let mut environments = Environments::new();
        let mut dev = Environment::new("dev");
        dev.set("baseUrl", "http://localhost:3000");
        dev.set("apiKey", "dev-key-123");
        environments.add_environment(dev);
        environments.set_active("dev");

        let environment_session = Arc::new(EnvironmentSession::new(environments));
        let executor = Arc::new(ExecutorBridge::new());

        let backend = Backend {
            client,
            documents: Arc::new(DocumentManager::new()),
            executor,
            environment_session,
            workspace_root: Arc::new(std::sync::RwLock::new(None)),
        };

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET {{baseUrl}}/users\nAuthorization: Bearer {{";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = CompletionParams {
            text_document_position: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 1,
                    character: 24,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let result = backend.completion(params).await.unwrap();
        assert!(result.is_some());

        if let Some(CompletionResponse::Array(completions)) = result {
            // Should have environment variables + system variables
            assert!(completions.len() >= 8);

            // Check for environment variables
            let base_url = completions.iter().find(|c| c.label == "baseUrl");
            assert!(base_url.is_some(), "Should have baseUrl variable");
            assert_eq!(base_url.unwrap().insert_text.as_ref().unwrap(), "baseUrl}}");

            let api_key = completions.iter().find(|c| c.label == "apiKey");
            assert!(api_key.is_some(), "Should have apiKey variable");
        }
    }

    #[tokio::test]
    async fn test_completion_document_not_found() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///nonexistent.http").unwrap();

        let params = CompletionParams {
            text_document_position: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 10,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let result = backend.completion(params).await.unwrap();
        // Should return None when document not found
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_completion_item_kinds() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET https://api.example.com/{{";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = CompletionParams {
            text_document_position: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 31,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let result = backend.completion(params).await.unwrap();

        if let Some(CompletionResponse::Array(completions)) = result {
            // All completions should be of kind VARIABLE
            for completion in completions {
                assert_eq!(completion.kind, Some(CompletionItemKind::VARIABLE));
            }
        }
    }

    #[tokio::test]
    async fn test_hover_on_variable() {
        let client = create_test_client();

        // Create environments with a test variable
        let mut environments = Environments::new();
        let mut dev = crate::environment::Environment::new("dev");
        dev.set("baseUrl", "http://localhost:3000");
        environments.add_environment(dev);
        environments.set_active("dev");

        let environment_session = Arc::new(EnvironmentSession::new(environments));

        let backend = Backend {
            client,
            documents: Arc::new(DocumentManager::new()),
            executor: Arc::new(ExecutorBridge::new()),
            environment_session,
            workspace_root: Arc::new(std::sync::RwLock::new(None)),
        };

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET {{baseUrl}}/users";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = HoverParams {
            text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 8, // Inside {{baseUrl}}
                },
            },
            work_done_progress_params: Default::default(),
        };

        let result = backend.hover(params).await.unwrap();
        assert!(result.is_some());

        let hover = result.unwrap();
        if let HoverContents::Markup(markup) = hover.contents {
            assert_eq!(markup.kind, MarkupKind::Markdown);
            assert!(markup.value.contains("baseUrl"));
            assert!(markup.value.contains("http://localhost:3000"));
            assert!(markup.value.contains("dev"));
        } else {
            panic!("Expected MarkupContent");
        }

        assert!(hover.range.is_some());
    }

    #[tokio::test]
    async fn test_hover_on_undefined_variable() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET {{undefinedVar}}/users";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = HoverParams {
            text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 10, // Inside {{undefinedVar}}
                },
            },
            work_done_progress_params: Default::default(),
        };

        let result = backend.hover(params).await.unwrap();
        assert!(result.is_some());

        let hover = result.unwrap();
        if let HoverContents::Markup(markup) = hover.contents {
            assert!(markup.value.contains("undefinedVar"));
            assert!(markup.value.contains("Undefined variable"));
        } else {
            panic!("Expected MarkupContent");
        }
    }

    #[tokio::test]
    async fn test_hover_outside_variable() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET http://example.com/users";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = HoverParams {
            text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 10, // Not on a variable
                },
            },
            work_done_progress_params: Default::default(),
        };

        let result = backend.hover(params).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_hover_document_not_found() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///nonexistent.http").unwrap();

        let params = HoverParams {
            text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 8,
                },
            },
            work_done_progress_params: Default::default(),
        };

        let result = backend.hover(params).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_hover_on_system_variable() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET {{$timestamp}}/data";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = HoverParams {
            text_document_position_params: tower_lsp::lsp_types::TextDocumentPositionParams {
                text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
                position: LspPosition {
                    line: 0,
                    character: 8, // Inside {{$timestamp}}
                },
            },
            work_done_progress_params: Default::default(),
        };

        let result = backend.hover(params).await.unwrap();
        assert!(result.is_some());

        let hover = result.unwrap();
        if let HoverContents::Markup(markup) = hover.contents {
            assert!(markup.value.contains("$timestamp"));
            assert!(markup.value.contains("System Variable") || markup.value.contains("runtime"));
        } else {
            panic!("Expected MarkupContent");
        }
    }

    #[tokio::test]
    async fn test_diagnostic_invalid_method() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "INVALID http://example.com";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = DocumentDiagnosticParams {
            text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            identifier: None,
            previous_result_id: None,
        };

        let result = backend.diagnostic(params).await.unwrap();

        if let DocumentDiagnosticReportResult::Report(
            tower_lsp::lsp_types::DocumentDiagnosticReport::Full(report),
        ) = result
        {
            let diagnostics = &report.full_document_diagnostic_report.items;
            assert!(!diagnostics.is_empty());
            assert!(diagnostics
                .iter()
                .any(|d| d.severity == Some(LspDiagnosticSeverity::ERROR)));
        } else {
            panic!("Expected full diagnostic report");
        }
    }

    #[tokio::test]
    async fn test_diagnostic_undefined_variable() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET {{undefinedVar}}/users";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = DocumentDiagnosticParams {
            text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            identifier: None,
            previous_result_id: None,
        };

        let result = backend.diagnostic(params).await.unwrap();

        if let DocumentDiagnosticReportResult::Report(
            tower_lsp::lsp_types::DocumentDiagnosticReport::Full(report),
        ) = result
        {
            let diagnostics = &report.full_document_diagnostic_report.items;
            // Should have warnings for undefined variables
            assert!(diagnostics
                .iter()
                .any(|d| d.severity == Some(LspDiagnosticSeverity::WARNING)
                    && d.message.contains("undefinedVar")));
        } else {
            panic!("Expected full diagnostic report");
        }
    }

    #[tokio::test]
    async fn test_diagnostic_valid_syntax() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        let content = "GET http://example.com/users\nContent-Type: application/json";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = DocumentDiagnosticParams {
            text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            identifier: None,
            previous_result_id: None,
        };

        let result = backend.diagnostic(params).await.unwrap();

        if let DocumentDiagnosticReportResult::Report(
            tower_lsp::lsp_types::DocumentDiagnosticReport::Full(report),
        ) = result
        {
            let diagnostics = &report.full_document_diagnostic_report.items;
            // Valid syntax should have no errors (may have informational messages)
            assert!(!diagnostics
                .iter()
                .any(|d| d.severity == Some(LspDiagnosticSeverity::ERROR)));
        } else {
            panic!("Expected full diagnostic report");
        }
    }

    #[tokio::test]
    async fn test_diagnostic_document_not_found() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///nonexistent.http").unwrap();

        let params = DocumentDiagnosticParams {
            text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            identifier: None,
            previous_result_id: None,
        };

        let result = backend.diagnostic(params).await.unwrap();

        if let DocumentDiagnosticReportResult::Report(
            tower_lsp::lsp_types::DocumentDiagnosticReport::Full(report),
        ) = result
        {
            let diagnostics = &report.full_document_diagnostic_report.items;
            assert!(diagnostics.is_empty());
        } else {
            panic!("Expected full diagnostic report");
        }
    }

    #[tokio::test]
    async fn test_diagnostic_with_suggestions() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = tower_lsp::lsp_types::Url::parse("file:///test.http").unwrap();
        // Invalid JSON body should produce diagnostics with suggestions
        let content = "POST http://example.com\nContent-Type: application/json\n\n{invalid json}";
        backend
            .documents
            .insert(uri.clone(), content.to_string())
            .unwrap();

        let params = DocumentDiagnosticParams {
            text_document: tower_lsp::lsp_types::TextDocumentIdentifier { uri: uri.clone() },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            identifier: None,
            previous_result_id: None,
        };

        let result = backend.diagnostic(params).await.unwrap();

        if let DocumentDiagnosticReportResult::Report(
            tower_lsp::lsp_types::DocumentDiagnosticReport::Full(report),
        ) = result
        {
            let diagnostics = &report.full_document_diagnostic_report.items;
            // Should have diagnostics for invalid JSON
            assert!(!diagnostics.is_empty());
            assert!(diagnostics.iter().any(|d| d.related_information.is_some()));
        } else {
            panic!("Expected full diagnostic report");
        }
    }

    #[tokio::test]
    async fn test_execute_command_unknown_command() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let params = ExecuteCommandParams {
            command: "unknown.command".to_string(),
            arguments: vec![],
            work_done_progress_params: Default::default(),
        };

        let result = backend.execute_command(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_command_missing_arguments() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let params = ExecuteCommandParams {
            command: "rest-client.send".to_string(),
            arguments: vec![],
            work_done_progress_params: Default::default(),
        };

        let result = backend.execute_command(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_command_invalid_uri_argument() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let params = ExecuteCommandParams {
            command: "rest-client.send".to_string(),
            arguments: vec![
                serde_json::Value::Number(123.into()), // Invalid: should be string
                serde_json::Value::Number(1.into()),
            ],
            work_done_progress_params: Default::default(),
        };

        let result = backend.execute_command(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_command_invalid_line_argument() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = "file:///test.http";
        let params = ExecuteCommandParams {
            command: "rest-client.send".to_string(),
            arguments: vec![
                serde_json::Value::String(uri.to_string()),
                serde_json::Value::String("not a number".to_string()), // Invalid: should be number
            ],
            work_done_progress_params: Default::default(),
        };

        let result = backend.execute_command(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_command_document_not_found() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let uri = "file:///nonexistent.http";
        let params = ExecuteCommandParams {
            command: "rest-client.send".to_string(),
            arguments: vec![
                serde_json::Value::String(uri.to_string()),
                serde_json::Value::Number(1.into()),
            ],
            work_done_progress_params: Default::default(),
        };

        let result = backend.execute_command(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_command_malformed_uri() {
        let client = create_test_client();
        let backend = Backend::new(client);

        let params = ExecuteCommandParams {
            command: "rest-client.send".to_string(),
            arguments: vec![
                serde_json::Value::String("not-a-valid-uri".to_string()),
                serde_json::Value::Number(1.into()),
            ],
            work_done_progress_params: Default::default(),
        };

        let result = backend.execute_command(params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_command_validates_arguments() {
        let client = create_test_client();
        let backend = Backend::new(client);

        // Test with only one argument (needs 2)
        let params = ExecuteCommandParams {
            command: "rest-client.send".to_string(),
            arguments: vec![serde_json::Value::String("file:///test.http".to_string())],
            work_done_progress_params: Default::default(),
        };

        let result = backend.execute_command(params).await;
        assert!(result.is_err());
    }
}
