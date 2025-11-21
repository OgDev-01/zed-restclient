#!/bin/bash

set -e  # Exit on error

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
    EXTENSIONS_DIR="$HOME/Library/Application Support/Zed/extensions/installed/rest-client"
    OS_NAME="macOS"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    EXTENSIONS_DIR="$HOME/.local/share/zed/extensions/installed/rest-client"
    OS_NAME="Linux"
else
    echo -e "${RED}❌ Unsupported OS: $OSTYPE${NC}"
    echo "This script supports macOS and Linux only."
    exit 1
fi

echo -e "${BLUE}Detected OS:${NC} $OS_NAME"
echo -e "${BLUE}Installation directory:${NC} $EXTENSIONS_DIR"
echo ""

# Build the extension
echo -e "${YELLOW}🔨 Building extension...${NC}"
cargo build --target wasm32-wasip1 --release

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed!${NC}"
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

# Create extension directory
echo -e "${YELLOW}📁 Creating extension directory...${NC}"
mkdir -p "$EXTENSIONS_DIR"

# Backup existing installation if it exists
if [ -f "$EXTENSIONS_DIR/extension.wasm" ]; then
    BACKUP_DIR="$EXTENSIONS_DIR.backup.$(date +%Y%m%d_%H%M%S)"
    echo -e "${YELLOW}⚠️  Backing up existing installation to:${NC}"
    echo "   $BACKUP_DIR"
    cp -r "$EXTENSIONS_DIR" "$BACKUP_DIR"
fi

# Copy files
echo -e "${YELLOW}📋 Copying extension files...${NC}"

# Copy extension manifest
cp extension.toml "$EXTENSIONS_DIR/"
echo "   ✓ extension.toml"

# Copy language configuration
if [ -d "languages" ]; then
    cp -r languages "$EXTENSIONS_DIR/"
    echo "   ✓ languages/"
else
    echo -e "${YELLOW}   ⚠️  languages/ directory not found (syntax highlighting may not work)${NC}"
fi

# Copy WASM binary
cp target/wasm32-wasip1/release/rest_client.wasm "$EXTENSIONS_DIR/extension.wasm"
echo "   ✓ extension.wasm"

echo ""
echo -e "${GREEN}✅ Extension installed successfully!${NC}"
echo ""

# Verify installation
echo -e "${BLUE}📊 Installation Summary:${NC}"
echo "   Location: $EXTENSIONS_DIR"
echo "   Files installed:"
ls -lh "$EXTENSIONS_DIR" | tail -n +2 | awk '{print "     - " $9 " (" $5 ")"}'

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
