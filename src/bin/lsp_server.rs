//! REST Client Language Server Binary Entry Point
//!
//! This binary provides a Language Server Protocol implementation for .http files.
//! It uses tower-lsp for the LSP framework and tokio for async runtime.
//!
//! # Communication
//!
//! The LSP server communicates via stdin/stdout using the JSON-RPC protocol.
//! All logging is sent to stderr to avoid interfering with the LSP protocol.
//!
//! # Features
//!
//! - Code lenses for "Send Request" buttons
//! - Autocompletion for variables
//! - Hover information for variable values
//! - Diagnostics for syntax errors
//! - Command execution for HTTP requests

use rest_client::lsp_server::backend::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Log startup to stderr (stdout is reserved for LSP protocol)
    eprintln!("[rest-client-lsp] Starting REST Client Language Server...");
    eprintln!("[rest-client-lsp] Version: {}", env!("CARGO_PKG_VERSION"));

    // Create stdin/stdout for LSP communication
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Create the LSP service with our Backend implementation
    let (service, socket) = LspService::new(|client| {
        eprintln!("[rest-client-lsp] Initializing backend with client connection");
        Backend::new(client)
    });

    eprintln!("[rest-client-lsp] Server ready, listening on stdin/stdout");

    // Start the LSP server with graceful shutdown handling
    let server = Server::new(stdin, stdout, socket);

    // Run the server with signal handling for graceful shutdown
    tokio::select! {
        _ = server.serve(service) => {
            eprintln!("[rest-client-lsp] Server finished");
        }
        _ = shutdown_signal() => {
            eprintln!("[rest-client-lsp] Received shutdown signal");
        }
    }

    eprintln!("[rest-client-lsp] Server shutting down gracefully");
}

/// Wait for a shutdown signal (Ctrl+C or SIGTERM)
///
/// This function waits for either:
/// - SIGINT (Ctrl+C) on Unix systems
/// - SIGTERM on Unix systems
/// - Ctrl+C on Windows
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");

        tokio::select! {
            _ = sigint.recv() => {
                eprintln!("[rest-client-lsp] Received SIGINT");
            }
            _ = sigterm.recv() => {
                eprintln!("[rest-client-lsp] Received SIGTERM");
            }
        }
    }

    #[cfg(windows)]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
        eprintln!("[rest-client-lsp] Received Ctrl+C");
    }
}
