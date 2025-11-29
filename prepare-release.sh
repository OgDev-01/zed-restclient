#!/bin/bash
# Prepare LSP binaries for GitHub release
# This script builds and prepares binaries for upload to GitHub releases

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Prepare LSP Binaries for GitHub Release${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Get version from extension.toml
VERSION=$(grep '^version' extension.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "Extension version: ${GREEN}v${VERSION}${NC}"
echo ""

# Detect current platform
ARCH=$(uname -m)
OS=$(uname -s)

case "$OS" in
    Darwin)
        case "$ARCH" in
            arm64)
                PLATFORM="darwin-arm64"
                RUST_TARGET="aarch64-apple-darwin"
                ;;
            x86_64)
                PLATFORM="darwin-x64"
                RUST_TARGET="x86_64-apple-darwin"
                ;;
            *)
                echo -e "${RED}Unsupported architecture: $ARCH${NC}"
                exit 1
                ;;
        esac
        ;;
    Linux)
        case "$ARCH" in
            aarch64)
                PLATFORM="linux-arm64"
                RUST_TARGET="aarch64-unknown-linux-gnu"
                ;;
            x86_64)
                PLATFORM="linux-x64"
                RUST_TARGET="x86_64-unknown-linux-gnu"
                ;;
            *)
                echo -e "${RED}Unsupported architecture: $ARCH${NC}"
                exit 1
                ;;
        esac
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

echo -e "Detected platform: ${GREEN}${PLATFORM}${NC}"
echo -e "Rust target: ${GREEN}${RUST_TARGET}${NC}"
echo ""

# Create release directory
RELEASE_DIR="release-binaries"
mkdir -p "$RELEASE_DIR"

# Build optimized binary
echo -e "${YELLOW}Building optimized LSP server...${NC}"
cargo build --bin lsp-server --release --target "$RUST_TARGET" --features lsp

# Copy and rename binary
BINARY_NAME="lsp-server-${PLATFORM}"
SOURCE_PATH="target/${RUST_TARGET}/release/lsp-server"

if [ -f "$SOURCE_PATH" ]; then
    cp "$SOURCE_PATH" "$RELEASE_DIR/$BINARY_NAME"
    chmod +x "$RELEASE_DIR/$BINARY_NAME"

    # Get binary size
    SIZE=$(stat -f%z "$RELEASE_DIR/$BINARY_NAME" 2>/dev/null || stat -c%s "$RELEASE_DIR/$BINARY_NAME" 2>/dev/null)
    SIZE_MB=$(echo "scale=2; $SIZE / 1024 / 1024" | bc)

    echo ""
    echo -e "${GREEN}✓ Binary prepared successfully!${NC}"
    echo -e "   Path: ${RELEASE_DIR}/${BINARY_NAME}"
    echo -e "   Size: ${SIZE_MB} MB"
    echo ""
else
    echo -e "${RED}✗ Build failed: binary not found at ${SOURCE_PATH}${NC}"
    exit 1
fi

# Instructions for manual release
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Next Steps: Create GitHub Release${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "1. Go to: ${GREEN}https://github.com/OgDev-01/zed-restclient/releases/new${NC}"
echo ""
echo -e "2. Fill in release details:"
echo -e "   - Tag: ${GREEN}v${VERSION}${NC}"
echo -e "   - Title: ${GREEN}REST Client LSP v${VERSION}${NC}"
echo ""
echo -e "3. Upload the binary:"
echo -e "   ${GREEN}${RELEASE_DIR}/${BINARY_NAME}${NC}"
echo ""
echo -e "4. Publish the release"
echo ""
echo -e "${YELLOW}Note:${NC} For full cross-platform support, you'll need to build"
echo -e "      and upload binaries for all platforms:"
echo -e "      - lsp-server-darwin-arm64  (macOS Apple Silicon)"
echo -e "      - lsp-server-darwin-x64    (macOS Intel)"
echo -e "      - lsp-server-linux-x64     (Linux x64)"
echo -e "      - lsp-server-linux-arm64   (Linux ARM64)"
echo -e "      - lsp-server-windows-x64.exe (Windows)"
echo ""

# Check for GitHub CLI
if command -v gh &> /dev/null; then
    echo -e "${GREEN}GitHub CLI detected!${NC} You can create the release with:"
    echo ""
    echo -e "  ${BLUE}gh release create v${VERSION} ${RELEASE_DIR}/${BINARY_NAME} \\${NC}"
    echo -e "  ${BLUE}  --title \"REST Client LSP v${VERSION}\" \\${NC}"
    echo -e "  ${BLUE}  --notes \"LSP server binary for ${PLATFORM}\"${NC}"
    echo ""
fi

echo -e "${GREEN}Done!${NC}"
