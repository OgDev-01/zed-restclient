# Optimized build script for REST Client WASM extension (PowerShell/Windows)
# This script builds the extension with maximum optimizations and minimal binary size

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

Write-ColorOutput Cyan "========================================"
Write-ColorOutput Cyan "REST Client - Optimized WASM Build"
Write-ColorOutput Cyan "========================================"
Write-Output ""

# Check if wasm32-wasip1 target is installed
Write-ColorOutput Yellow "[1/6] Checking WASM target..."
$targetInstalled = rustup target list --installed | Select-String "wasm32-wasip1"
if (-not $targetInstalled) {
    Write-ColorOutput Yellow "Installing wasm32-wasip1 target..."
    rustup target add wasm32-wasip1
} else {
    Write-ColorOutput Green "✓ wasm32-wasip1 target already installed"
}
Write-Output ""

# Build with release profile
Write-ColorOutput Yellow "[2/6] Building WASM binary with release optimizations..."
Write-Output "       opt-level = 3 (maximum optimization)"
Write-Output "       lto = true (link-time optimization)"
Write-Output "       codegen-units = 1 (better optimization)"
Write-Output "       strip = true (remove debug symbols)"
Write-Output "       panic = abort (smaller binary)"
Write-Output ""

cargo build --target wasm32-wasip1 --release

$wasmFile = "target\wasm32-wasip1\release\rest_client.wasm"
$optimizedFile = "target\wasm32-wasip1\release\rest_client_optimized.wasm"

if (-not (Test-Path $wasmFile)) {
    Write-ColorOutput Red "✗ Build failed: WASM file not found"
    exit 1
}

# Get original size
$originalSize = (Get-Item $wasmFile).Length
$originalSizeMB = [math]::Round($originalSize / 1MB, 2)
Write-ColorOutput Green "✓ Build successful"
Write-Output "   Original size: $originalSizeMB MB ($originalSize bytes)"
Write-Output ""

# Check if wasm-opt is available
Write-ColorOutput Yellow "[3/6] Checking for wasm-opt..."
$wasmOptAvailable = Get-Command wasm-opt -ErrorAction SilentlyContinue

if ($wasmOptAvailable) {
    Write-ColorOutput Green "✓ wasm-opt found"

    Write-ColorOutput Yellow "[4/6] Running wasm-opt optimization (-Oz for size)..."
    wasm-opt -Oz -o $optimizedFile $wasmFile

    $optimizedSize = (Get-Item $optimizedFile).Length
    $optimizedSizeMB = [math]::Round($optimizedSize / 1MB, 2)
    $reduction = [math]::Round(100 - ($optimizedSize * 100 / $originalSize), 1)

    Write-ColorOutput Green "✓ Optimization complete"
    Write-Output "   Optimized size: $optimizedSizeMB MB ($optimizedSize bytes)"
    Write-Output "   Reduction: $reduction%"

    # Replace original with optimized
    Move-Item -Path $optimizedFile -Destination $wasmFile -Force
    $finalSize = $optimizedSize
    $finalSizeMB = $optimizedSizeMB
} else {
    Write-ColorOutput Yellow "⚠ wasm-opt not found"
    Write-Output "   Install from: https://github.com/WebAssembly/binaryen/releases"
    Write-Output "   Or use: cargo install wasm-opt"
    Write-Output "   Skipping additional optimization..."
    $finalSize = $originalSize
    $finalSizeMB = $originalSizeMB
}
Write-Output ""

# Strip WASM (additional size reduction)
Write-ColorOutput Yellow "[5/6] Stripping WASM binary..."
$wasmStripAvailable = Get-Command wasm-strip -ErrorAction SilentlyContinue

if ($wasmStripAvailable) {
    wasm-strip $wasmFile
    $strippedSize = (Get-Item $wasmFile).Length
    $strippedSizeMB = [math]::Round($strippedSize / 1MB, 2)
    Write-ColorOutput Green "✓ Stripped successfully"
    Write-Output "   Final size: $strippedSizeMB MB ($strippedSize bytes)"
    $finalSize = $strippedSize
    $finalSizeMB = $strippedSizeMB
} else {
    Write-ColorOutput Yellow "⚠ wasm-strip not found (part of wabt toolkit)"
    Write-Output "   Download from: https://github.com/WebAssembly/wabt/releases"
}
Write-Output ""

# Performance target check
Write-ColorOutput Yellow "[6/6] Checking size against target (<2MB)..."
$targetSize = 2 * 1024 * 1024

if ($finalSize -lt $targetSize) {
    Write-ColorOutput Green "✓ SUCCESS: Binary is within target size"
    $margin = [math]::Round(($targetSize - $finalSize) / 1MB, 1)
    Write-Output "   Margin: $margin MB under target"
} else {
    Write-ColorOutput Red "✗ WARNING: Binary exceeds 2MB target"
    $excess = [math]::Round(($finalSize - $targetSize) / 1MB, 2)
    Write-Output "   Excess: $excess MB over target"
}
Write-Output ""

# Summary
Write-ColorOutput Cyan "========================================"
Write-ColorOutput Cyan "Build Summary"
Write-ColorOutput Cyan "========================================"
Write-Output "Output: $wasmFile"
Write-Output "Size:   $finalSizeMB MB ($finalSize bytes)"
if ($finalSize -lt $targetSize) {
    Write-ColorOutput Green "Status: ✓ Ready for deployment"
} else {
    Write-ColorOutput Yellow "Status: ⚠ Consider further optimization"
}
Write-Output ""

# Optional: Run size analysis
$twiggyAvailable = Get-Command twiggy -ErrorAction SilentlyContinue
$bloatAvailable = Get-Command cargo-bloat -ErrorAction SilentlyContinue

if ($twiggyAvailable) {
    Write-ColorOutput Cyan "Top 10 largest functions (twiggy):"
    twiggy top -n 10 $wasmFile
    Write-Output ""
    Write-Output "For detailed analysis: twiggy top -n 20 $wasmFile"
    Write-Output "For monomorphizations: twiggy monos $wasmFile"
} elseif ($bloatAvailable) {
    Write-ColorOutput Cyan "Binary size breakdown (cargo-bloat):"
    cargo bloat --release --target wasm32-wasip1 -n 10
} else {
    Write-ColorOutput Yellow "Tip: Install twiggy or cargo-bloat for size analysis"
    Write-Output "     cargo install twiggy"
    Write-Output "     cargo install cargo-bloat"
}
Write-Output ""

Write-ColorOutput Green "Build complete!"
Write-Output ""
Write-Output "Next steps:"
Write-Output "  1. Test the extension: .\install-dev.ps1"
Write-Output "  2. Run benchmarks: cargo bench"
Write-Output "  3. Profile if needed: cargo flamegraph"
Write-Output ""
