# Build script for REST Client LSP Server (Windows PowerShell)
# Builds optimized LSP server binaries for multiple platforms

$ErrorActionPreference = "Stop"

# Colors for output
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Write-Header($message) {
    Write-ColorOutput Blue "========================================"
    Write-ColorOutput Blue $message
    Write-ColorOutput Blue "========================================"
    Write-Output ""
}

function Write-Success($message) {
    Write-ColorOutput Green $message
}

function Write-Warning($message) {
    Write-ColorOutput Yellow $message
}

function Write-Error($message) {
    Write-ColorOutput Red $message
}

Write-Header "REST Client LSP Server - Build"

# Parse command line arguments
$BuildAll = $false
$Targets = @()
$SkipTests = $false

for ($i = 0; $i -lt $args.Count; $i++) {
    switch ($args[$i]) {
        "--all" {
            $BuildAll = $true
        }
        "--target" {
            $i++
            if ($i -lt $args.Count) {
                $Targets += $args[$i]
            }
        }
        "--skip-tests" {
            $SkipTests = $true
        }
        "--help" {
            Write-Output "Usage: .\build-lsp.ps1 [OPTIONS]"
            Write-Output ""
            Write-Output "Options:"
            Write-Output "  --all           Build for all supported platforms"
            Write-Output "  --target TARGET Build for specific target (can be used multiple times)"
            Write-Output "  --skip-tests    Skip running tests before build"
            Write-Output "  --help          Show this help message"
            Write-Output ""
            Write-Output "Supported targets:"
            Write-Output "  x86_64-apple-darwin       macOS (Intel)"
            Write-Output "  aarch64-apple-darwin      macOS (Apple Silicon)"
            Write-Output "  x86_64-unknown-linux-gnu  Linux (x86_64)"
            Write-Output "  x86_64-pc-windows-msvc    Windows (x86_64)"
            Write-Output ""
            Write-Output "Examples:"
            Write-Output "  .\build-lsp.ps1                                  # Build for current platform"
            Write-Output "  .\build-lsp.ps1 --all                            # Build for all platforms"
            Write-Output "  .\build-lsp.ps1 --target x86_64-pc-windows-msvc  # Build for Windows"
            Write-Output ""
            exit 0
        }
        default {
            Write-Error "Unknown option: $($args[$i])"
            Write-Output "Use --help for usage information"
            exit 1
        }
    }
}

# Determine targets to build
if ($BuildAll) {
    $Targets = @(
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "x86_64-pc-windows-msvc"
    )
} elseif ($Targets.Count -eq 0) {
    # Default to current platform
    $rustcOutput = rustc -vV | Select-String "host:"
    $CurrentTarget = $rustcOutput.ToString().Replace("host: ", "").Trim()
    $Targets = @($CurrentTarget)
    Write-Warning "No target specified, building for current platform: $CurrentTarget"
    Write-Output ""
}

# Run tests unless skipped
if (-not $SkipTests) {
    Write-Warning "[1/3] Running tests..."
    cargo test --bin lsp-server --lib
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tests failed"
        exit 1
    }
    Write-Success "✓ Tests passed"
    Write-Output ""
} else {
    Write-Warning "Skipping tests (--skip-tests flag)"
    Write-Output ""
}

# Create output directory
$OutputDir = "target\lsp-binaries"
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

Write-Warning "[2/3] Building LSP server binaries..."
Write-Output "       Optimization level: 3 (maximum)"
Write-Output "       Link-time optimization: enabled"
Write-Output "       Debug symbols: stripped"
Write-Output "       Panic strategy: abort"
Write-Output ""

# Build for each target
$SuccessfulBuilds = @()
$FailedBuilds = @()

foreach ($Target in $Targets) {
    Write-ColorOutput Blue "Building for $Target..."

    # Check if target is installed
    $installedTargets = rustup target list --installed
    if ($installedTargets -notcontains $Target) {
        Write-Warning "Installing target $Target..."
        rustup target add $Target
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Failed to install target $Target"
            $FailedBuilds += $Target
            Write-Output ""
            continue
        }
    }

    # Build the binary
    cargo build --bin lsp-server --release --target $Target 2>&1 | Select-Object -Last 10

    if ($LASTEXITCODE -eq 0) {
        # Determine binary name based on platform
        if ($Target -like "*windows*") {
            $BinaryName = "lsp-server.exe"
        } else {
            $BinaryName = "lsp-server"
        }

        $SourcePath = "target\$Target\release\$BinaryName"
        $DestName = "lsp-server-$Target"
        if ($Target -like "*windows*") {
            $DestName += ".exe"
        }
        $DestPath = "$OutputDir\$DestName"

        if (Test-Path $SourcePath) {
            Copy-Item $SourcePath $DestPath -Force

            # Get binary size
            $Size = (Get-Item $DestPath).Length
            $SizeMB = [math]::Round($Size / 1MB, 2)

            Write-Success "✓ Build successful"
            Write-Output "   Binary: $DestPath"
            Write-Output "   Size: $SizeMB MB ($Size bytes)"

            # Check size against 10MB target
            $TargetSize = 10 * 1024 * 1024
            if ($Size -lt $TargetSize) {
                Write-ColorOutput Green "   Status: ✓ Within 10MB target"
            } else {
                Write-ColorOutput Red "   Status: ✗ Exceeds 10MB target"
            }

            $SuccessfulBuilds += $Target
        } else {
            Write-Error "✗ Build failed: binary not found"
            $FailedBuilds += $Target
        }
    } else {
        Write-Error "✗ Build failed for $Target"
        $FailedBuilds += $Target
    }

    Write-Output ""
}

# Copy to extension directory (for current platform only)
Write-Warning "[3/3] Copying binary to extension directory..."
$rustcOutput = rustc -vV | Select-String "host:"
$CurrentTarget = $rustcOutput.ToString().Replace("host: ", "").Trim()

if ($SuccessfulBuilds -contains $CurrentTarget) {
    if ($CurrentTarget -like "*windows*") {
        $BinaryName = "lsp-server.exe"
    } else {
        $BinaryName = "lsp-server"
    }

    $DestName = "lsp-server-$CurrentTarget"
    if ($CurrentTarget -like "*windows*") {
        $DestName += ".exe"
    }
    $Source = "$OutputDir\$DestName"
    $Dest = $BinaryName

    if (Test-Path $Source) {
        Copy-Item $Source $Dest -Force
        Write-Success "✓ Copied $BinaryName to extension root"
    } else {
        Write-Warning "⚠ Binary for current platform not found"
    }
} else {
    Write-Warning "⚠ Current platform not in successful builds"
}
Write-Output ""

# Summary
Write-Header "Build Summary"

if ($SuccessfulBuilds.Count -gt 0) {
    Write-ColorOutput Green "Successful builds ($($SuccessfulBuilds.Count)):"
    foreach ($Target in $SuccessfulBuilds) {
        $DestName = "lsp-server-$Target"
        if ($Target -like "*windows*") {
            $DestName += ".exe"
        }
        $BinaryPath = "$OutputDir\$DestName"

        if (Test-Path $BinaryPath) {
            $Size = (Get-Item $BinaryPath).Length
            $SizeMB = [math]::Round($Size / 1MB, 2)
            Write-Output "  ✓ $Target - $SizeMB MB"
        } else {
            Write-Output "  ✓ $Target"
        }
    }
    Write-Output ""
}

if ($FailedBuilds.Count -gt 0) {
    Write-ColorOutput Red "Failed builds ($($FailedBuilds.Count)):"
    foreach ($Target in $FailedBuilds) {
        Write-Output "  ✗ $Target"
    }
    Write-Output ""
}

Write-Output "Output directory: $OutputDir\"
Write-Output "Extension binary: lsp-server (current platform)"
Write-Output ""

if ($FailedBuilds.Count -eq 0) {
    Write-Success "All builds completed successfully!"
    Write-Output ""
    Write-Output "Next steps:"
    Write-Output "  1. Test the LSP server: cargo run --bin lsp-server"
    Write-Output "  2. Install extension: .\install-dev.ps1"
    Write-Output "  3. Verify in Zed with a .http file"
} else {
    Write-Warning "Some builds failed. Check errors above."
    exit 1
}
Write-Output ""
