//! REST Client Extension for Zed Editor
//!
//! This extension provides HTTP request execution capabilities within Zed,
//! allowing users to send requests from `.http` and `.rest` files and view
//! formatted responses.
//!
//! # Architecture
//!
//! The extension is organized into several modules:
//!
//! - **models**: Core data structures for HTTP requests and responses
//! - **parser**: Parses `.http` files into structured request objects
//! - **executor**: Executes HTTP requests using reqwest
//! - **formatter**: Formats responses for display with content type detection
//! - **commands**: Command handlers for Zed integration
//! - **environment**: Environment management for variables and settings
//! - **variables**: Variable substitution and resolution
//! - **language_server**: LSP features for variable autocompletion and hover
//!
//! # Command Integration
//!
//! The main command is `send_request_command` which:
//! 1. Extracts the request block at the cursor position (bounded by `###` delimiters)
//! 2. Parses the request using the HTTP parser
//! 3. Executes the request with configurable timeout
//! 4. Formats the response based on content type (JSON, XML, HTML, etc.)
//! 5. Returns formatted output for display in a new editor buffer
//!
//! # Environment Switching
//!
//! The extension supports environment-based variable management:
//! - Use `/switch-environment` slash command to change active environment
//! - Variables are resolved from the active environment or shared variables
//! - Environment configuration is loaded from `.http-client-env.json` files
//!
//! # Usage
//!
//! To use this extension in Zed:
//! 1. Create a `.http` or `.rest` file
//! 2. Write HTTP requests using standard syntax:
//!    ```http
//!    GET https://api.example.com/users
//!    Authorization: Bearer token123
//!
//!    ###
//!
//!    POST https://api.example.com/users
//!    Content-Type: application/json
//!
//!    {"name": "John", "email": "john@example.com"}
//!    ```
//! 3. Place cursor within a request block
//! 4. Execute the "Send Request" command
//! 5. View the formatted response in a new buffer
//!
//! # Note on Zed Extension API
//!
//! This extension is designed to work with the Zed extension API. The actual
//! command registration and editor integration will depend on the Zed extension
//! system capabilities. The core logic is implemented in the `commands` module
//! and can be called by the extension host.

use std::sync::{Arc, Mutex};
use zed_extension_api as zed;

pub mod auth;
pub mod codegen;
pub mod commands;
pub mod config;
pub mod curl;
pub mod environment;
pub mod executor;
pub mod formatter;
pub mod graphql;
pub mod history;
pub mod language_server;
#[cfg(feature = "lsp")]
pub mod lsp_server;
pub mod models;
pub mod parser;
pub mod ui;
pub mod variables;

use executor::{execute_request, ExecutionConfig};
use formatter::format_response;
use parser::parse_request;

/// REST Client extension for Zed.
///
/// This extension provides HTTP request execution capabilities directly
/// within the Zed editor for `.http` and `.rest` files.
///
/// The extension maintains state for environment management, allowing
/// users to switch between different environments (dev, staging, production)
/// and have variables resolved accordingly.
struct RestClientExtension {
    /// Session for managing environment state across requests
    /// Wrapped in Arc<Mutex> for thread-safe mutable access
    environment_session: Arc<Mutex<Option<environment::EnvironmentSession>>>,
}

impl zed::Extension for RestClientExtension {
    fn new() -> Self {
        Self {
            environment_session: Arc::new(Mutex::new(None)),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        // The lsp-server binary should be in the extension directory
        // Zed extensions run from their installation directory, so we can use a relative path
        // The binary is installed alongside extension.wasm

        // Determine binary name based on platform
        let binary_name = if cfg!(target_os = "windows") {
            "lsp-server.exe"
        } else {
            "lsp-server"
        };

        // First try to find it on PATH (in case user has it installed)
        let command = worktree.which(binary_name).unwrap_or_else(|| {
            // Fallback to relative path in extension directory
            // This works because Zed runs the extension from its install directory
            if cfg!(target_os = "windows") {
                ".\\lsp-server.exe".to_string()
            } else {
                "./lsp-server".to_string()
            }
        });

        Ok(zed::Command {
            command,
            args: vec![],
            env: vec![],
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        Ok(None)
    }

    fn run_slash_command(
        &self,
        command: zed::SlashCommand,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        match command.name.as_str() {
            "switch-environment" => self.handle_switch_environment(args, worktree),
            "generate-code" => self.handle_generate_code(args, worktree),
            "paste-curl" => self.handle_paste_curl(args),
            "copy-as-curl" => self.handle_copy_as_curl(args),
            "send-request" => {
                // Argument patterns supported:
                // 1 arg: selection-only (HTTP request text)
                // 2 args: full editor text, cursor byte offset -> attempt block extraction
                // If extraction fails, fall back to treating first arg as direct request text.
                if args.is_empty() {
                    return Err("Send Request: no input provided. Supply selection text or file content + cursor.".to_string());
                }

                let (request_text, _start_line) = if args.len() >= 2 {
                    // Try cursor-based extraction
                    if let Ok(cursor_pos) = args[1].parse::<usize>() {
                        let editor_text = &args[0];
                        match crate::commands::extract_request_at_cursor(editor_text, cursor_pos) {
                            Ok((extracted, start_line)) => (extracted, start_line),
                            Err(_) => (editor_text.clone(), 0),
                        }
                    } else {
                        (args[0].clone(), 0)
                    }
                } else {
                    (args[0].clone(), 0)
                };

                if request_text.trim().is_empty() {
                    return Err(
                        "Send Request: resolved request text is empty after extraction."
                            .to_string(),
                    );
                }

                // Parse the request
                let lines: Vec<String> = request_text.lines().map(|s| s.to_string()).collect();
                let indexed_lines: Vec<(usize, &str)> = lines
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (i, s.as_str()))
                    .collect();
                let file_path = std::path::PathBuf::from("slash-command");
                let request = parse_request(&indexed_lines, 0, &file_path)
                    .map_err(|e| format!("Failed to parse request: {}", e))?;

                // Execute the request
                let config = ExecutionConfig::default();
                let response = execute_request(&request, &config)
                    .map_err(|e| format!("Failed to execute request: {}", e))?;

                // Format the response
                let formatted = format_response(&response);
                let output_text = formatted.to_display_string();

                // Return as slash command output
                Ok(zed::SlashCommandOutput {
                    sections: vec![zed::SlashCommandOutputSection {
                        range: (0..output_text.len()).into(),
                        label: format!("{} {}", request.method, request.url),
                    }],
                    text: output_text,
                })
            }
            _ => Err(format!("Unknown command: {}", command.name)),
        }
    }
}

impl RestClientExtension {
    /// Handles the switch-environment slash command
    ///
    /// Lists available environments and allows switching between them.
    /// If no arguments provided, lists all available environments.
    /// If an environment name is provided, switches to that environment.
    fn handle_switch_environment(
        &self,
        args: Vec<String>,
        worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        // Get workspace path from worktree
        let workspace_path = worktree
            .map(|w| std::path::PathBuf::from(w.root_path()))
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            });

        // Try to load or get existing environment session
        let mut session_lock = self
            .environment_session
            .lock()
            .map_err(|e| format!("Failed to acquire session lock: {}", e))?;

        // Load environments if not already loaded
        if session_lock.is_none() {
            match environment::load_environments(&workspace_path) {
                Ok(envs) => {
                    *session_lock = Some(environment::EnvironmentSession::new(envs));
                }
                Err(e) => {
                    // No environment file found - provide helpful message
                    return Ok(zed::SlashCommandOutput {
                        sections: vec![zed::SlashCommandOutputSection {
                            range: (0_usize..0_usize).into(),
                            label: "Environment Setup Required".to_string(),
                        }],
                        text: format!(
                            "No environment configuration found.\n\n\
                            To use environments, create a `.http-client-env.json` file in your workspace:\n\n\
                            ```json\n{{\n  \"$shared\": {{\n    \"apiVersion\": \"v1\"\n  }},\n  \
                            \"dev\": {{\n    \"baseUrl\": \"http://localhost:3000\"\n  }},\n  \
                            \"production\": {{\n    \"baseUrl\": \"https://api.example.com\"\n  }},\n  \
                            \"active\": \"dev\"\n}}\n```\n\n\
                            See examples/environment-variables.http for more details.\n\nError: {}", e
                        ),
                    });
                }
            }
        }

        let session = session_lock.as_ref().unwrap();

        // If args provided, try to switch to that environment
        if !args.is_empty() {
            let env_name = args[0].trim();

            match session.set_active_environment(env_name) {
                Ok(_) => {
                    let output_text = format!(
                        "✓ Switched to '{}' environment\n\n\
                        Variables from this environment are now active.\n\
                        Any requests you send will use variables from '{}'.",
                        env_name, env_name
                    );

                    Ok(zed::SlashCommandOutput {
                        sections: vec![zed::SlashCommandOutputSection {
                            range: (0_usize..output_text.len()).into(),
                            label: format!("Environment: {}", env_name),
                        }],
                        text: output_text,
                    })
                }
                Err(e) => {
                    let available = session.list_environment_names();
                    Err(format!(
                        "Failed to switch to environment '{}': {}\n\n\
                        Available environments: {}",
                        env_name,
                        e,
                        available.join(", ")
                    ))
                }
            }
        } else {
            // No args - list available environments
            let env_names = session.list_environment_names();
            let active_env = session.get_active_environment_name();

            if env_names.is_empty() {
                return Ok(zed::SlashCommandOutput {
                    sections: vec![],
                    text: "No environments defined in configuration file.".to_string(),
                });
            }

            let mut output = String::from("Available Environments:\n\n");

            for name in &env_names {
                let is_active = active_env.as_ref().map_or(false, |a| a == name);
                let marker = if is_active { "→ " } else { "  " };
                let indicator = if is_active { " (active)" } else { "" };
                output.push_str(&format!("{}{}{}\n", marker, name, indicator));
            }

            output.push_str("\n");
            if let Some(active) = &active_env {
                output.push_str(&format!("Current active environment: {}\n\n", active));
            } else {
                output.push_str("No environment currently active.\n\n");
            }

            output.push_str("To switch environment, use:\n");
            output.push_str("/switch-environment <environment-name>\n\n");
            output.push_str("Example: /switch-environment production");

            Ok(zed::SlashCommandOutput {
                sections: vec![zed::SlashCommandOutputSection {
                    range: (0_usize..output.len()).into(),
                    label: "Environments".to_string(),
                }],
                text: output,
            })
        }
    }

    /// Handles the generate-code slash command
    ///
    /// Generates executable code from an HTTP request in the specified language.
    /// Usage: /generate-code <language> [library]
    /// Example: /generate-code javascript fetch
    fn handle_generate_code(
        &self,
        args: Vec<String>,
        _worktree: Option<&zed::Worktree>,
    ) -> Result<zed::SlashCommandOutput, String> {
        use codegen::ui::{
            generate_code_command, list_available_languages, parse_generation_options,
        };

        // If no args, show help
        if args.is_empty() {
            let help_text = list_available_languages();
            return Ok(zed::SlashCommandOutput {
                sections: vec![zed::SlashCommandOutputSection {
                    range: (0..help_text.len()).into(),
                    label: "Code Generation Options".to_string(),
                }],
                text: help_text,
            });
        }

        // First arg should be the request text (selected by user)
        // Remaining args are language and library options
        let request_text = &args[0];
        let generation_args: Vec<String> = args.iter().skip(1).cloned().collect();

        // Parse generation options
        let (language, library) = parse_generation_options(&generation_args)?;

        // Parse the HTTP request
        let lines: Vec<String> = request_text.lines().map(|s| s.to_string()).collect();
        let indexed_lines: Vec<(usize, &str)> = lines
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.as_str()))
            .collect();
        let file_path = std::path::PathBuf::from("slash-command");
        let request = parse_request(&indexed_lines, 0, &file_path)
            .map_err(|e| format!("Failed to parse request: {}", e))?;

        // Generate code
        let result = generate_code_command(&request, language, library);

        if !result.success {
            return Err(result.message);
        }

        let output_text = result.to_display_string();

        Ok(zed::SlashCommandOutput {
            sections: vec![zed::SlashCommandOutputSection {
                range: (0..output_text.len()).into(),
                label: format!("Generated {} Code", result.language.unwrap().as_str()),
            }],
            text: output_text,
        })
    }

    /// Handles the paste-curl slash command
    ///
    /// Converts a cURL command (from clipboard or selection) to HTTP request format.
    /// Usage: /paste-curl <curl-command>
    fn handle_paste_curl(&self, args: Vec<String>) -> Result<zed::SlashCommandOutput, String> {
        use commands::paste_curl_from_clipboard;

        if args.is_empty() {
            return Err("No cURL command provided. Usage: /paste-curl <curl-command>".to_string());
        }

        // Join all args as they might be the full curl command
        let curl_text = args.join(" ");

        // Paste and convert the cURL command
        let result = paste_curl_from_clipboard(&curl_text);

        if !result.success {
            return Err(result.message);
        }

        let output_text = result.to_display_string();

        Ok(zed::SlashCommandOutput {
            sections: vec![zed::SlashCommandOutputSection {
                range: (0..output_text.len()).into(),
                label: "Converted from cURL".to_string(),
            }],
            text: output_text,
        })
    }

    /// Handles the copy-as-curl slash command
    ///
    /// Converts an HTTP request to a cURL command.
    /// Usage: /copy-as-curl (with HTTP request text in selection)
    fn handle_copy_as_curl(&self, args: Vec<String>) -> Result<zed::SlashCommandOutput, String> {
        if args.is_empty() {
            return Err(
                "No HTTP request provided. Please select an HTTP request and use /copy-as-curl"
                    .to_string(),
            );
        }

        // First arg should be the request text (selected by user)
        let request_text = args.join("\n");

        // Parse the HTTP request
        let lines: Vec<String> = request_text.lines().map(|s| s.to_string()).collect();
        let indexed_lines: Vec<(usize, &str)> = lines
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s.as_str()))
            .collect();
        let file_path = std::path::PathBuf::from("slash-command");
        let request = parse_request(&indexed_lines, 0, &file_path)
            .map_err(|e| format!("Failed to parse request: {}", e))?;

        // Generate cURL command
        let result = curl::ui::copy_as_curl_command(&request);

        if !result.success {
            return Err(result.message);
        }

        let output_text = result.to_display_string();

        Ok(zed::SlashCommandOutput {
            sections: vec![zed::SlashCommandOutputSection {
                range: (0..output_text.len()).into(),
                label: format!("cURL Command ({})", result.preview),
            }],
            text: output_text,
        })
    }

    /// Gets the current environment session for use in request execution
    pub fn get_environment_session(&self) -> Option<environment::EnvironmentSession> {
        self.environment_session
            .lock()
            .ok()
            .and_then(|session| session.clone())
    }
}

zed::register_extension!(RestClientExtension);
