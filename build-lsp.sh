#!/bin/bash
# Build script for REST Client LSP Server
# Builds optimized LSP server binaries for multiple platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}REST Client LSP Server - Build${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Parse command line arguments
TARGETS=()
BUILD_ALL=false
SKIP_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            BUILD_ALL=true
            shift
            ;;
        --target)
            TARGETS+=("$2")
            shift 2
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --help)
            echo "Usage: ./build-lsp.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --all           Build for all supported platforms"
            echo "  --target TARGET Build for specific target (can be used multiple times)"
            echo "  --skip-tests    Skip running tests before build"
            echo "  --help          Show this help message"
            echo ""
            echo "Supported targets:"
            echo "  x86_64-apple-darwin       macOS (Intel)"
            echo "  aarch64-apple-darwin      macOS (Apple Silicon)"
            echo "  x86_64-unknown-linux-gnu  Linux (x86_64)"
            echo "  x86_64-pc-windows-msvc    Windows (x86_64)"
            echo ""
            echo "Examples:"
            echo "  ./build-lsp.sh                                  # Build for current platform"
            echo "  ./build-lsp.sh --all                            # Build for all platforms"
            echo "  ./build-lsp.sh --target x86_64-apple-darwin     # Build for macOS Intel"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Determine targets to build
if [ "$BUILD_ALL" = true ]; then
    TARGETS=(
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "x86_64-unknown-linux-gnu"
        "x86_64-pc-windows-msvc"
    )
elif [ ${#TARGETS[@]} -eq 0 ]; then
    # Default to current platform
    CURRENT_TARGET=$(rustc -vV | sed -n 's|host: ||p')
    TARGETS=("$CURRENT_TARGET")
    echo -e "${YELLOW}No target specified, building for current platform: ${CURRENT_TARGET}${NC}"
    echo ""
fi

# Run tests unless skipped
if [ "$SKIP_TESTS" = false ]; then
    echo -e "${YELLOW}[1/3]${NC} Running tests..."
    cargo test --bin lsp-server --lib
    echo -e "${GREEN}✓ Tests passed${NC}"
    echo ""
else
    echo -e "${YELLOW}Skipping tests (--skip-tests flag)${NC}"
    echo ""
fi

# Create output directory
OUTPUT_DIR="target/lsp-binaries"
mkdir -p "$OUTPUT_DIR"

echo -e "${YELLOW}[2/3]${NC} Building LSP server binaries..."
echo -e "       Optimization level: 3 (maximum)"
echo -e "       Link-time optimization: enabled"
echo -e "       Debug symbols: stripped"
echo -e "       Panic strategy: abort"
echo ""

# Build for each target
SUCCESSFUL_BUILDS=()
FAILED_BUILDS=()

for TARGET in "${TARGETS[@]}"; do
    echo -e "${BLUE}Building for ${TARGET}...${NC}"

    # Check if target is installed
    if ! rustup target list --installed | grep -q "^${TARGET}$"; then
        echo -e "${YELLOW}Installing target ${TARGET}...${NC}"
        rustup target add "$TARGET"
    fi

    # Build the binary
    if cargo build --bin lsp-server --release --target "$TARGET" 2>&1 | tail -10; then
        # Determine binary name based on platform
        if [[ "$TARGET" == *"windows"* ]]; then
            BINARY_NAME="lsp-server.exe"
        else
            BINARY_NAME="lsp-server"
        fi

        SOURCE_PATH="target/${TARGET}/release/${BINARY_NAME}"
        DEST_PATH="${OUTPUT_DIR}/lsp-server-${TARGET}${BINARY_NAME##lsp-server}"

        if [ -f "$SOURCE_PATH" ]; then
            cp "$SOURCE_PATH" "$DEST_PATH"

            # Get binary size
            SIZE=$(stat -f%z "$DEST_PATH" 2>/dev/null || stat -c%s "$DEST_PATH" 2>/dev/null || echo "unknown")
            if [ "$SIZE" != "unknown" ]; then
                SIZE_MB=$(echo "scale=2; $SIZE / 1024 / 1024" | bc)
                echo -e "${GREEN}✓ Build successful${NC}"
                echo -e "   Binary: ${DEST_PATH}"
                echo -e "   Size: ${SIZE_MB} MB (${SIZE} bytes)"

                # Check size against 10MB target
                TARGET_SIZE=$((10 * 1024 * 1024))
                if [ "$SIZE" -lt "$TARGET_SIZE" ]; then
                    echo -e "   Status: ${GREEN}✓ Within 10MB target${NC}"
                else
                    echo -e "   Status: ${RED}✗ Exceeds 10MB target${NC}"
                fi
            else
                echo -e "${GREEN}✓ Build successful${NC}"
                echo -e "   Binary: ${DEST_PATH}"
            fi

            SUCCESSFUL_BUILDS+=("$TARGET")
        else
            echo -e "${RED}✗ Build failed: binary not found${NC}"
            FAILED_BUILDS+=("$TARGET")
        fi
    else
        echo -e "${RED}✗ Build failed for ${TARGET}${NC}"
        FAILED_BUILDS+=("$TARGET")
    fi

    echo ""
done

# Copy to extension directory (for current platform only)
echo -e "${YELLOW}[3/3]${NC} Copying binary to extension directory..."
CURRENT_TARGET=$(rustc -vV | sed -n 's|host: ||p')

if [[ " ${SUCCESSFUL_BUILDS[@]} " =~ " ${CURRENT_TARGET} " ]]; then
    if [[ "$CURRENT_TARGET" == *"windows"* ]]; then
        BINARY_NAME="lsp-server.exe"
    else
        BINARY_NAME="lsp-server"
    fi

    SOURCE="${OUTPUT_DIR}/lsp-server-${CURRENT_TARGET}${BINARY_NAME##lsp-server}"
    DEST="lsp-server${BINARY_NAME##lsp-server}"

    if [ -f "$SOURCE" ]; then
        cp "$SOURCE" "$DEST"
        echo -e "${GREEN}✓ Copied ${BINARY_NAME} to extension root${NC}"
    else
        echo -e "${YELLOW}⚠ Binary for current platform not found${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Current platform not in successful builds${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Build Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [ ${#SUCCESSFUL_BUILDS[@]} -gt 0 ]; then
    echo -e "${GREEN}Successful builds (${#SUCCESSFUL_BUILDS[@]}):${NC}"
    for TARGET in "${SUCCESSFUL_BUILDS[@]}"; do
        if [[ "$TARGET" == *"windows"* ]]; then
            BINARY_NAME="lsp-server.exe"
        else
            BINARY_NAME="lsp-server"
        fi
        BINARY_PATH="${OUTPUT_DIR}/lsp-server-${TARGET}${BINARY_NAME##lsp-server}"
        SIZE=$(stat -f%z "$BINARY_PATH" 2>/dev/null || stat -c%s "$BINARY_PATH" 2>/dev/null || echo "unknown")
        if [ "$SIZE" != "unknown" ]; then
            SIZE_MB=$(echo "scale=2; $SIZE / 1024 / 1024" | bc)
            echo -e "  ✓ ${TARGET} - ${SIZE_MB} MB"
        else
            echo -e "  ✓ ${TARGET}"
        fi
    done
    echo ""
fi

if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
    echo -e "${RED}Failed builds (${#FAILED_BUILDS[@]}):${NC}"
    for TARGET in "${FAILED_BUILDS[@]}"; do
        echo -e "  ✗ ${TARGET}"
    done
    echo ""
fi

echo -e "Output directory: ${OUTPUT_DIR}/"
echo -e "Extension binary: lsp-server (current platform)"
echo ""

if [ ${#FAILED_BUILDS[@]} -eq 0 ]; then
    echo -e "${GREEN}All builds completed successfully!${NC}"
    echo ""
    echo -e "Next steps:"
    echo -e "  1. Test the LSP server: cargo run --bin lsp-server"
    echo -e "  2. Install extension: ./install-dev.sh"
    echo -e "  3. Verify in Zed with a .http file"
else
    echo -e "${YELLOW}Some builds failed. Check errors above.${NC}"
    exit 1
fi
echo ""
