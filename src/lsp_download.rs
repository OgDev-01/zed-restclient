//! LSP Binary Download Module
//!
//! This module handles downloading and managing the REST Client LSP server binary
//! from GitHub releases. It provides automatic download on first use with caching
//! in the extension's work directory.

use std::fs;
use zed_extension_api::{self as zed, http_client, Architecture, Os};

/// GitHub repository information for LSP binary releases
const GITHUB_OWNER: &str = "OgDev-01";
const GITHUB_REPO: &str = "zed-restclient";

/// Current version of the LSP binary expected by this extension
const LSP_VERSION: &str = "0.2.0";

/// Name of the LSP binary (without platform-specific extension)
const LSP_BINARY_NAME: &str = "lsp-server";

/// Errors that can occur during LSP binary download
#[derive(Debug)]
pub enum LspDownloadError {
    /// Failed to detect the current platform
    UnsupportedPlatform(String),
    /// Failed to create directory for binary
    DirectoryCreationFailed(String),
    /// Failed to download binary from GitHub
    DownloadFailed(String),
    /// Failed to write binary to disk
    WriteFailed(String),
    /// Failed to set executable permissions
    PermissionFailed(String),
    /// Binary not found and download is disabled
    BinaryNotFound(String),
}

impl std::fmt::Display for LspDownloadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspDownloadError::UnsupportedPlatform(message) => {
                write!(formatter, "Unsupported platform: {}", message)
            }
            LspDownloadError::DirectoryCreationFailed(message) => {
                write!(formatter, "Failed to create directory: {}", message)
            }
            LspDownloadError::DownloadFailed(message) => {
                write!(formatter, "Failed to download LSP binary: {}", message)
            }
            LspDownloadError::WriteFailed(message) => {
                write!(formatter, "Failed to write binary: {}", message)
            }
            LspDownloadError::PermissionFailed(message) => {
                write!(formatter, "Failed to set permissions: {}", message)
            }
            LspDownloadError::BinaryNotFound(message) => {
                write!(formatter, "LSP binary not found: {}", message)
            }
        }
    }
}

impl From<LspDownloadError> for String {
    fn from(error: LspDownloadError) -> Self {
        error.to_string()
    }
}

/// Represents the target platform for LSP binary download
#[derive(Debug, Clone, Copy)]
pub enum Platform {
    MacOsX64,
    MacOsArm64,
    LinuxX64,
    LinuxArm64,
    WindowsX64,
}

impl Platform {
    /// Detect the current platform using Zed's API
    pub fn detect() -> Result<Self, LspDownloadError> {
        let (os, arch) = zed::current_platform();

        match (os, arch) {
            (Os::Mac, Architecture::X8664) => Ok(Platform::MacOsX64),
            (Os::Mac, Architecture::Aarch64) => Ok(Platform::MacOsArm64),
            (Os::Linux, Architecture::X8664) => Ok(Platform::LinuxX64),
            (Os::Linux, Architecture::Aarch64) => Ok(Platform::LinuxArm64),
            (Os::Windows, Architecture::X8664) => Ok(Platform::WindowsX64),
            _ => Err(LspDownloadError::UnsupportedPlatform(format!(
                "OS: {:?}, Architecture: {:?}. Supported platforms: macOS (x64, arm64), Linux (x64, arm64), Windows (x64)",
                os, arch
            ))),
        }
    }

    /// Get the platform-specific binary name
    pub fn binary_name(&self) -> String {
        match self {
            Platform::WindowsX64 => format!("{}.exe", LSP_BINARY_NAME),
            _ => LSP_BINARY_NAME.to_string(),
        }
    }

    /// Get the platform-specific asset name for GitHub releases
    pub fn asset_name(&self) -> String {
        match self {
            Platform::MacOsX64 => format!("{}-darwin-x64", LSP_BINARY_NAME),
            Platform::MacOsArm64 => format!("{}-darwin-arm64", LSP_BINARY_NAME),
            Platform::LinuxX64 => format!("{}-linux-x64", LSP_BINARY_NAME),
            Platform::LinuxArm64 => format!("{}-linux-arm64", LSP_BINARY_NAME),
            Platform::WindowsX64 => format!("{}-windows-x64.exe", LSP_BINARY_NAME),
        }
    }
}

/// Manager for LSP binary download and caching
pub struct LspBinaryManager {
    /// Extension work directory where binaries are cached
    work_directory: String,
    /// Target platform for downloads
    platform: Platform,
}

impl LspBinaryManager {
    /// Create a new LSP binary manager
    ///
    /// # Arguments
    /// * `work_directory` - The extension's work directory for caching binaries
    pub fn new(work_directory: String) -> Result<Self, LspDownloadError> {
        let platform = Platform::detect()?;
        Ok(Self {
            work_directory,
            platform,
        })
    }

    /// Get the path where the LSP binary should be stored
    pub fn binary_path(&self) -> String {
        let binary_name = self.platform.binary_name();
        format!("{}/{}", self.work_directory, binary_name)
    }

    /// Get the path to the version file
    fn version_file_path(&self) -> String {
        format!("{}/lsp-version.txt", self.work_directory)
    }

    /// Check if the LSP binary exists and is the correct version
    pub fn is_binary_installed(&self) -> bool {
        let binary_path = self.binary_path();
        let version_path = self.version_file_path();

        // Check if binary exists
        if fs::metadata(&binary_path).is_err() {
            return false;
        }

        // Check version matches
        if let Ok(installed_version) = fs::read_to_string(&version_path) {
            installed_version.trim() == LSP_VERSION
        } else {
            // No version file means we should re-download
            false
        }
    }

    /// Download the LSP binary from GitHub releases
    pub fn download_binary(&self) -> Result<String, LspDownloadError> {
        // Ensure work directory exists
        fs::create_dir_all(&self.work_directory).map_err(|error| {
            LspDownloadError::DirectoryCreationFailed(format!(
                "Could not create directory '{}': {}",
                self.work_directory, error
            ))
        })?;

        // Construct download URL
        let asset_name = self.platform.asset_name();
        let download_url = format!(
            "https://github.com/{}/{}/releases/download/v{}/{}",
            GITHUB_OWNER, GITHUB_REPO, LSP_VERSION, asset_name
        );

        // Download the binary using Zed's HTTP client
        let http_request = http_client::HttpRequest::builder()
            .method(http_client::HttpMethod::Get)
            .url(&download_url)
            .header("Accept", "application/octet-stream")
            .header("User-Agent", "zed-rest-client-extension")
            .build()
            .map_err(LspDownloadError::DownloadFailed)?;

        let response = http_request
            .fetch()
            .map_err(LspDownloadError::DownloadFailed)?;

        // Check if we got a valid response (non-empty body)
        if response.body.is_empty() {
            return Err(LspDownloadError::DownloadFailed(
                "Received empty response from GitHub. The release may not exist yet.".to_string(),
            ));
        }

        // Write binary to disk
        let binary_path = self.binary_path();
        fs::write(&binary_path, &response.body).map_err(|error| {
            LspDownloadError::WriteFailed(format!(
                "Could not write binary to '{}': {}",
                binary_path, error
            ))
        })?;

        // Set executable permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&binary_path)
                .map_err(|error| {
                    LspDownloadError::PermissionFailed(format!(
                        "Could not read metadata: {}",
                        error
                    ))
                })?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&binary_path, permissions).map_err(|error| {
                LspDownloadError::PermissionFailed(format!(
                    "Could not set executable permissions: {}",
                    error
                ))
            })?;
        }

        // Write version file
        let version_path = self.version_file_path();
        fs::write(&version_path, LSP_VERSION).map_err(|error| {
            LspDownloadError::WriteFailed(format!(
                "Could not write version file '{}': {}",
                version_path, error
            ))
        })?;

        Ok(binary_path)
    }

    /// Ensure the LSP binary is available, downloading if necessary
    ///
    /// Returns the path to the binary
    pub fn ensure_binary(&self) -> Result<String, LspDownloadError> {
        if self.is_binary_installed() {
            Ok(self.binary_path())
        } else {
            self.download_binary()
        }
    }
}

/// Check if the LSP binary exists in the worktree PATH
pub fn find_binary_in_path(worktree: &zed::Worktree) -> Option<String> {
    let binary_name = if cfg!(target_os = "windows") {
        format!("{}.exe", LSP_BINARY_NAME)
    } else {
        LSP_BINARY_NAME.to_string()
    };

    worktree.which(&binary_name)
}

/// Get a user-friendly error message for LSP download failures
pub fn format_error_message(error: &LspDownloadError) -> String {
    match error {
        LspDownloadError::UnsupportedPlatform(message) => {
            format!(
                "REST Client LSP: {}\n\n\
                The LSP server provides advanced features like code completion and hover information.\n\
                You can still use the extension's slash commands without the LSP server.",
                message
            )
        }
        LspDownloadError::DownloadFailed(message) => {
            format!(
                "REST Client LSP: Failed to download LSP binary.\n\n\
                Error: {}\n\n\
                You can manually download the binary from:\n\
                https://github.com/{}/{}/releases\n\n\
                Place the binary in your PATH or the extension's work directory.",
                message, GITHUB_OWNER, GITHUB_REPO
            )
        }
        LspDownloadError::DirectoryCreationFailed(message)
        | LspDownloadError::WriteFailed(message)
        | LspDownloadError::PermissionFailed(message) => {
            format!(
                "REST Client LSP: Failed to install LSP binary.\n\n\
                Error: {}\n\n\
                Please check file system permissions.",
                message
            )
        }
        LspDownloadError::BinaryNotFound(message) => {
            format!(
                "REST Client LSP: {}\n\n\
                The extension will continue to work, but LSP features will be unavailable.",
                message
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_binary_names() {
        assert_eq!(Platform::MacOsX64.binary_name(), "lsp-server");
        assert_eq!(Platform::MacOsArm64.binary_name(), "lsp-server");
        assert_eq!(Platform::LinuxX64.binary_name(), "lsp-server");
        assert_eq!(Platform::LinuxArm64.binary_name(), "lsp-server");
        assert_eq!(Platform::WindowsX64.binary_name(), "lsp-server.exe");
    }

    #[test]
    fn test_platform_asset_names() {
        assert_eq!(Platform::MacOsX64.asset_name(), "lsp-server-darwin-x64");
        assert_eq!(Platform::MacOsArm64.asset_name(), "lsp-server-darwin-arm64");
        assert_eq!(Platform::LinuxX64.asset_name(), "lsp-server-linux-x64");
        assert_eq!(Platform::LinuxArm64.asset_name(), "lsp-server-linux-arm64");
        assert_eq!(
            Platform::WindowsX64.asset_name(),
            "lsp-server-windows-x64.exe"
        );
    }

    #[test]
    fn test_error_display() {
        let error = LspDownloadError::UnsupportedPlatform("test".to_string());
        assert!(error.to_string().contains("Unsupported platform"));

        let error = LspDownloadError::DownloadFailed("network error".to_string());
        assert!(error.to_string().contains("Failed to download"));
    }
}
