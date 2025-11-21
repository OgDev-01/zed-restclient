# REST Client Extension - Development Installation (Windows)
# PowerShell script for installing the extension to Zed

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

Write-ColorOutput Cyan "REST Client Extension - Development Installation"
Write-Output ""

# Set extensions directory
$EXTENSIONS_DIR = "$env:APPDATA\Zed\extensions\installed\rest-client"

Write-ColorOutput Cyan "Detected OS: Windows"
Write-ColorOutput Cyan "Installation directory: $EXTENSIONS_DIR"
Write-Output ""

# Build the extension
Write-ColorOutput Yellow "🔨 Building extension..."
cargo build --target wasm32-wasip1 --release

if ($LASTEXITCODE -ne 0) {
    Write-ColorOutput Red "❌ Build failed!"
    exit 1
}

# Check if WASM file was created
$WASM_PATH = "target\wasm32-wasip1\release\rest_client.wasm"
if (-not (Test-Path $WASM_PATH)) {
    Write-ColorOutput Red "❌ WASM file not found after build!"
    Write-Output "Expected: $WASM_PATH"
    exit 1
}

# Get WASM file size
$WASM_SIZE = (Get-Item $WASM_PATH).Length / 1MB
$WASM_SIZE_STR = "{0:N2} MB" -f $WASM_SIZE
Write-ColorOutput Green "✅ Build successful! (WASM size: $WASM_SIZE_STR)"
Write-Output ""

# Create extension directory
Write-ColorOutput Yellow "📁 Creating extension directory..."
New-Item -ItemType Directory -Force -Path $EXTENSIONS_DIR | Out-Null

# Backup existing installation if it exists
$EXISTING_WASM = Join-Path $EXTENSIONS_DIR "extension.wasm"
if (Test-Path $EXISTING_WASM) {
    $BACKUP_DIR = "$EXTENSIONS_DIR.backup.$(Get-Date -Format 'yyyyMMdd_HHmmss')"
    Write-ColorOutput Yellow "⚠️  Backing up existing installation to:"
    Write-Output "   $BACKUP_DIR"
    Copy-Item -Recurse -Force $EXTENSIONS_DIR $BACKUP_DIR
}

# Copy files
Write-ColorOutput Yellow "📋 Copying extension files..."

# Copy extension manifest
Copy-Item -Force "extension.toml" $EXTENSIONS_DIR\
Write-Output "   ✓ extension.toml"

# Copy language configuration
if (Test-Path "languages") {
    Copy-Item -Recurse -Force "languages" $EXTENSIONS_DIR\
    Write-Output "   ✓ languages\"
} else {
    Write-ColorOutput Yellow "   ⚠️  languages\ directory not found (syntax highlighting may not work)"
}

# Copy WASM binary
Copy-Item -Force $WASM_PATH "$EXTENSIONS_DIR\extension.wasm"
Write-Output "   ✓ extension.wasm"

Write-Output ""
Write-ColorOutput Green "✅ Extension installed successfully!"
Write-Output ""

# Verify installation
Write-ColorOutput Cyan "📊 Installation Summary:"
Write-Output "   Location: $EXTENSIONS_DIR"
Write-Output "   Files installed:"
Get-ChildItem $EXTENSIONS_DIR | ForEach-Object {
    $size = if ($_.PSIsContainer) { "DIR" } else { "{0:N2} KB" -f ($_.Length / 1KB) }
    Write-Output "     - $($_.Name) ($size)"
}

Write-Output ""
Write-ColorOutput Yellow "⚠️  IMPORTANT:"
Write-Output "   1. Completely quit Zed (Alt+F4 or close from system tray)"
Write-Output "   2. Restart Zed"
Write-Output "   3. Check extensions: Ctrl+Shift+P → 'zed: extensions'"
Write-Output ""
Write-ColorOutput Cyan "📝 Next Steps:"
Write-Output "   - Create a test.http file"
Write-Output "   - Write an HTTP request (see examples\ directory)"
Write-Output "   - Place cursor in request block"
Write-Output "   - Run 'Send Request' command"
Write-Output ""
Write-ColorOutput Green "Happy testing! 🚀"
