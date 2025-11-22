#!/bin/bash

# Don't exit on error for cp commands (files might be identical)
set +e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}REST Client Extension - Development Installation${NC}"
echo ""

# Detect OS and set extensions directory
if [[ "$OSTYPE" == "darwin"* ]]; then
    INSTALLED_DIR="$HOME/Library/Application Support/Zed/extensions/installed/rest-client"
    WORK_DIR="$HOME/Library/Application Support/Zed/extensions/work/rest-client"
    OS_NAME="macOS"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    INSTALLED_DIR="$HOME/.local/share/zed/extensions/installed/rest-client"
    WORK_DIR="$HOME/.local/share/zed/extensions/work/rest-client"
    OS_NAME="Linux"
else
    echo -e "${RED}❌ Unsupported OS: $OSTYPE${NC}"
    echo "This script supports macOS and Linux only."
    exit 1
fi

echo -e "${BLUE}Detected OS:${NC} $OS_NAME"
echo -e "${BLUE}Installed directory:${NC} $INSTALLED_DIR"
echo -e "${BLUE}Work directory:${NC} $WORK_DIR"
echo ""

# Check prerequisites
echo -e "${YELLOW}🔍 Checking prerequisites...${NC}"

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}❌ Rust is not installed!${NC}"
    echo ""
    echo "Please install Rust first:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "After installation, restart your terminal and run this script again."
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo is not installed!${NC}"
    echo ""
    echo "Please install Rust (which includes Cargo):"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "After installation, restart your terminal and run this script again."
    exit 1
fi

echo -e "${GREEN}✓ Rust installed:${NC} $(rustc --version)"
echo -e "${GREEN}✓ Cargo installed:${NC} $(cargo --version)"

# Check if wasm32-wasip1 target is installed
if ! rustup target list | grep -q "wasm32-wasip1 (installed)"; then
    echo -e "${RED}❌ wasm32-wasip1 target is not installed!${NC}"
    echo ""
    echo "Installing wasm32-wasip1 target..."
    rustup target add wasm32-wasip1

    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Failed to install wasm32-wasip1 target!${NC}"
        echo ""
        echo "Please run this command manually:"
        echo "  rustup target add wasm32-wasip1"
        echo ""
        echo "Then run this script again."
        exit 1
    fi
    echo -e "${GREEN}✓ wasm32-wasip1 target installed successfully!${NC}"
else
    echo -e "${GREEN}✓ wasm32-wasip1 target already installed${NC}"
fi

echo ""

# Build the LSP server first (native binary)
echo -e "${YELLOW}🔨 Building LSP server...${NC}"
cargo build --release --bin lsp-server --features lsp

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ LSP server build failed!${NC}"
    exit 1
fi

# Check if LSP server binary was created
if [ ! -f "target/release/lsp-server" ]; then
    echo -e "${RED}❌ LSP server binary not found after build!${NC}"
    echo "Expected: target/release/lsp-server"
    exit 1
fi

LSP_SIZE=$(du -h target/release/lsp-server | cut -f1)
echo -e "${GREEN}✅ LSP server built successfully!${NC} (Binary size: $LSP_SIZE)"
echo ""

# Build the extension (WASM)
echo -e "${YELLOW}🔨 Building extension WASM...${NC}"
cargo build --target wasm32-wasip1 --release

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Extension WASM build failed!${NC}"
    exit 1
fi

# Check if WASM file was created
if [ ! -f "target/wasm32-wasip1/release/rest_client.wasm" ]; then
    echo -e "${RED}❌ WASM file not found after build!${NC}"
    echo "Expected: target/wasm32-wasip1/release/rest_client.wasm"
    exit 1
fi

# Get WASM file size
WASM_SIZE=$(du -h target/wasm32-wasip1/release/rest_client.wasm | cut -f1)
echo -e "${GREEN}✅ Build successful!${NC} (WASM size: $WASM_SIZE)"
echo ""

# Create extension directories
echo -e "${YELLOW}📁 Creating extension directories...${NC}"
mkdir -p "$INSTALLED_DIR"
mkdir -p "$WORK_DIR"

# Backup existing installation if it exists
if [ -f "$INSTALLED_DIR/extension.wasm" ]; then
    BACKUP_DIR="$INSTALLED_DIR.backup.$(date +%Y%m%d_%H%M%S)"
    echo -e "${YELLOW}⚠️  Backing up existing installation to:${NC}"
    echo "   $BACKUP_DIR"
    cp -r "$INSTALLED_DIR" "$BACKUP_DIR"
fi

# Copy files to both directories
echo -e "${YELLOW}📋 Copying extension files to installed directory...${NC}"

# Copy extension manifest
cp -f extension.toml "$INSTALLED_DIR/"
echo "   ✓ extension.toml"

# Copy language configuration
if [ -d "languages" ]; then
    cp -r languages "$INSTALLED_DIR/"
    echo "   ✓ languages/"
else
    echo -e "${YELLOW}   ⚠️  languages/ directory not found (syntax highlighting may not work)${NC}"
fi

# Copy WASM binary
cp -f target/wasm32-wasip1/release/rest_client.wasm "$INSTALLED_DIR/extension.wasm"
echo "   ✓ extension.wasm"

# Copy LSP server binary
cp -f target/release/lsp-server "$INSTALLED_DIR/"
chmod +x "$INSTALLED_DIR/lsp-server"
echo "   ✓ lsp-server"

echo ""
echo -e "${YELLOW}📋 Copying extension files to work directory...${NC}"

# Copy essential files to work directory
cp -f extension.toml "$WORK_DIR/"
echo "   ✓ extension.toml"

cp -rf languages "$WORK_DIR/"
echo "   ✓ languages/"

cp -f target/wasm32-wasip1/release/rest_client.wasm "$WORK_DIR/extension.wasm"
echo "   ✓ extension.wasm"

cp -f target/release/lsp-server "$WORK_DIR/"
chmod +x "$WORK_DIR/lsp-server"
echo "   ✓ lsp-server"

# Copy grammars if they exist in installed directory
if [ -d "$INSTALLED_DIR/grammars" ]; then
    cp -r "$INSTALLED_DIR/grammars" "$WORK_DIR/"
    echo "   ✓ grammars/"
fi

echo ""
echo -e "${GREEN}✅ Extension installed successfully!${NC}"
echo ""

# Verify installation
echo -e "${BLUE}📊 Installation Summary:${NC}"
echo ""
echo "   Installed directory: $INSTALLED_DIR"
echo "   Files installed:"
ls -lh "$INSTALLED_DIR" | tail -n +2 | awk '{print "     - " $9 " (" $5 ")"}'
echo ""
echo "   Work directory: $WORK_DIR"
echo "   Files installed:"
ls -lh "$WORK_DIR" | tail -n +2 | awk '{print "     - " $9 " (" $5 ")"}'

echo ""
echo -e "${YELLOW}⚠️  IMPORTANT:${NC}"
echo "   1. Completely quit Zed (Cmd+Q on macOS, Alt+F4 on Linux)"
echo "   2. Restart Zed"
echo "   3. Check extensions: Cmd+Shift+P → 'zed: extensions'"
echo ""
echo -e "${BLUE}📝 Next Steps:${NC}"
echo "   - Create a test.http file"
echo "   - Write an HTTP request (see examples/ directory)"
echo "   - Place cursor in request block"
echo "   - Run 'Send Request' command"
echo ""
echo -e "${GREEN}Happy testing! 🚀${NC}"
