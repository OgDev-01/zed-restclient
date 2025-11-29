//! LSP Binary Download Module
//!
//! This module handles downloading and managing the REST Client LSP server binary
//! from GitHub releases using Zed's native GitHub release API.

use zed_extension_api::{
    self as zed, current_platform, download_file, github_release_by_tag_name, make_file_executable,
    Architecture, DownloadedFileType, Os,
};

/// GitHub repository information for LSP binary releases
const GITHUB_OWNER: &str = "OgDev-01";
const GITHUB_REPO: &str = "zed-restclient";

/// Current version of the LSP binary expected by this extension
const LSP_VERSION: &str = "v0.2.0";

/// Name of the LSP binary (without platform-specific extension)
const LSP_BINARY_NAME: &str = "lsp-server";

/// Errors that can occur during LSP binary download
#[derive(Debug)]
pub enum LspDownloadError {
    /// Failed to detect the current platform
    UnsupportedPlatform(String),
    /// Failed to get GitHub release info
    ReleaseNotFound(String),
    /// Failed to find the appropriate asset for this platform
    AssetNotFound(String),
    /// Failed to download binary from GitHub
    DownloadFailed(String),
    /// Binary not found and download is disabled
    BinaryNotFound(String),
}

impl std::fmt::Display for LspDownloadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspDownloadError::UnsupportedPlatform(message) => {
                write!(formatter, "Unsupported platform: {}", message)
            }
            LspDownloadError::ReleaseNotFound(message) => {
                write!(formatter, "GitHub release not found: {}", message)
            }
            LspDownloadError::AssetNotFound(message) => {
                write!(formatter, "Release asset not found: {}", message)
            }
            LspDownloadError::DownloadFailed(message) => {
                write!(formatter, "Failed to download LSP binary: {}", message)
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
        let (os, arch) = current_platform();

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
    /// Target platform for downloads
    platform: Platform,
}

impl LspBinaryManager {
    /// Create a new LSP binary manager
    pub fn new() -> Result<Self, LspDownloadError> {
        let platform = Platform::detect()?;
        Ok(Self { platform })
    }

    /// Get the expected binary name for this platform
    pub fn binary_name(&self) -> String {
        self.platform.binary_name()
    }

    /// Download the LSP binary from GitHub releases using Zed's native API
    pub fn download_binary(&self) -> Result<String, LspDownloadError> {
        // Get the release info from GitHub
        let release =
            github_release_by_tag_name(&format!("{}/{}", GITHUB_OWNER, GITHUB_REPO), LSP_VERSION)
                .map_err(|error| LspDownloadError::ReleaseNotFound(error))?;

        // Find the appropriate asset for this platform
        let asset_name = self.platform.asset_name();
        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                LspDownloadError::AssetNotFound(format!(
                    "No asset named '{}' found in release {}. Available assets: {:?}",
                    asset_name,
                    LSP_VERSION,
                    release.assets.iter().map(|a| &a.name).collect::<Vec<_>>()
                ))
            })?;

        // Download the binary using Zed's download_file function
        let binary_name = self.platform.binary_name();
        download_file(
            &asset.download_url,
            &binary_name,
            DownloadedFileType::Uncompressed,
        )
        .map_err(|error| LspDownloadError::DownloadFailed(error))?;

        // Make the binary executable (on Unix systems)
        make_file_executable(&binary_name).map_err(|error| {
            LspDownloadError::DownloadFailed(format!("Failed to make binary executable: {}", error))
        })?;

        Ok(binary_name)
    }

    /// Ensure the LSP binary is available, downloading if necessary
    ///
    /// Returns the path to the binary
    pub fn ensure_binary(&self) -> Result<String, LspDownloadError> {
        // Always try to download/update the binary
        // Zed's download_file handles caching internally
        self.download_binary()
    }
}

/// Check if the LSP binary exists in the worktree PATH
pub fn find_binary_in_path(worktree: &zed::Worktree) -> Option<String> {
    let binary_name = match current_platform() {
        (Os::Windows, _) => format!("{}.exe", LSP_BINARY_NAME),
        _ => LSP_BINARY_NAME.to_string(),
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
        LspDownloadError::ReleaseNotFound(message) => {
            format!(
                "REST Client LSP: GitHub release not found.\n\n\
                Error: {}\n\n\
                Please check if release {} exists at:\n\
                https://github.com/{}/{}/releases",
                message, LSP_VERSION, GITHUB_OWNER, GITHUB_REPO
            )
        }
        LspDownloadError::AssetNotFound(message) => {
            format!(
                "REST Client LSP: Binary not available for your platform.\n\n\
                Error: {}\n\n\
                You can manually download the binary from:\n\
                https://github.com/{}/{}/releases\n\n\
                Place the binary in your PATH.",
                message, GITHUB_OWNER, GITHUB_REPO
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
