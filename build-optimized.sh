#!/bin/bash
# Optimized build script for REST Client WASM extension
# This script builds the extension with maximum optimizations and minimal binary size

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}REST Client - Optimized WASM Build${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if wasm32-wasip1 target is installed
echo -e "${YELLOW}[1/6]${NC} Checking WASM target..."
if ! rustup target list --installed | grep -q "wasm32-wasip1"; then
    echo -e "${YELLOW}Installing wasm32-wasip1 target...${NC}"
    rustup target add wasm32-wasip1
else
    echo -e "${GREEN}✓ wasm32-wasip1 target already installed${NC}"
fi
echo ""

# Build with release profile
echo -e "${YELLOW}[2/6]${NC} Building WASM binary with release optimizations..."
echo -e "       opt-level = 3 (maximum optimization)"
echo -e "       lto = true (link-time optimization)"
echo -e "       codegen-units = 1 (better optimization)"
echo -e "       strip = true (remove debug symbols)"
echo -e "       panic = abort (smaller binary)"
echo ""

cargo build --target wasm32-wasip1 --release

WASM_FILE="target/wasm32-wasip1/release/rest_client.wasm"
OPTIMIZED_FILE="target/wasm32-wasip1/release/rest_client_optimized.wasm"

if [ ! -f "$WASM_FILE" ]; then
    echo -e "${RED}✗ Build failed: WASM file not found${NC}"
    exit 1
fi

# Get original size
ORIGINAL_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
ORIGINAL_SIZE_MB=$(echo "scale=2; $ORIGINAL_SIZE / 1024 / 1024" | bc)
echo -e "${GREEN}✓ Build successful${NC}"
echo -e "   Original size: ${ORIGINAL_SIZE_MB} MB (${ORIGINAL_SIZE} bytes)"
echo ""

# Check if wasm-opt is available
echo -e "${YELLOW}[3/6]${NC} Checking for wasm-opt..."
if command -v wasm-opt &> /dev/null; then
    echo -e "${GREEN}✓ wasm-opt found${NC}"

    echo -e "${YELLOW}[4/6]${NC} Running wasm-opt optimization (-Oz for size)..."
    wasm-opt -Oz -o "$OPTIMIZED_FILE" "$WASM_FILE"

    OPTIMIZED_SIZE=$(stat -f%z "$OPTIMIZED_FILE" 2>/dev/null || stat -c%s "$OPTIMIZED_FILE" 2>/dev/null)
    OPTIMIZED_SIZE_MB=$(echo "scale=2; $OPTIMIZED_SIZE / 1024 / 1024" | bc)
    REDUCTION=$(echo "scale=1; 100 - ($OPTIMIZED_SIZE * 100 / $ORIGINAL_SIZE)" | bc)

    echo -e "${GREEN}✓ Optimization complete${NC}"
    echo -e "   Optimized size: ${OPTIMIZED_SIZE_MB} MB (${OPTIMIZED_SIZE} bytes)"
    echo -e "   Reduction: ${REDUCTION}%"

    # Replace original with optimized
    mv "$OPTIMIZED_FILE" "$WASM_FILE"
    FINAL_SIZE=$OPTIMIZED_SIZE
    FINAL_SIZE_MB=$OPTIMIZED_SIZE_MB
else
    echo -e "${YELLOW}⚠ wasm-opt not found${NC}"
    echo -e "   Install with: cargo install wasm-opt"
    echo -e "   Skipping additional optimization..."
    FINAL_SIZE=$ORIGINAL_SIZE
    FINAL_SIZE_MB=$ORIGINAL_SIZE_MB
fi
echo ""

# Strip WASM (additional size reduction)
echo -e "${YELLOW}[5/6]${NC} Stripping WASM binary..."
if command -v wasm-strip &> /dev/null; then
    wasm-strip "$WASM_FILE"
    STRIPPED_SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null)
    STRIPPED_SIZE_MB=$(echo "scale=2; $STRIPPED_SIZE / 1024 / 1024" | bc)
    echo -e "${GREEN}✓ Stripped successfully${NC}"
    echo -e "   Final size: ${STRIPPED_SIZE_MB} MB (${STRIPPED_SIZE} bytes)"
    FINAL_SIZE=$STRIPPED_SIZE
    FINAL_SIZE_MB=$STRIPPED_SIZE_MB
else
    echo -e "${YELLOW}⚠ wasm-strip not found (part of wabt toolkit)${NC}"
    echo -e "   Install with: brew install wabt (macOS) or apt install wabt (Linux)"
fi
echo ""

# Performance target check
echo -e "${YELLOW}[6/6]${NC} Checking size against target (<2MB)..."
TARGET_SIZE=$((2 * 1024 * 1024))

if [ "$FINAL_SIZE" -lt "$TARGET_SIZE" ]; then
    echo -e "${GREEN}✓ SUCCESS: Binary is within target size${NC}"
    MARGIN=$(echo "scale=1; ($TARGET_SIZE - $FINAL_SIZE) / 1024 / 1024" | bc)
    echo -e "   Margin: ${MARGIN} MB under target"
else
    echo -e "${RED}✗ WARNING: Binary exceeds 2MB target${NC}"
    EXCESS=$(echo "scale=2; ($FINAL_SIZE - $TARGET_SIZE) / 1024 / 1024" | bc)
    echo -e "   Excess: ${EXCESS} MB over target"
fi
echo ""

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Build Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo -e "Output: ${WASM_FILE}"
echo -e "Size:   ${FINAL_SIZE_MB} MB (${FINAL_SIZE} bytes)"
if [ "$FINAL_SIZE" -lt "$TARGET_SIZE" ]; then
    echo -e "Status: ${GREEN}✓ Ready for deployment${NC}"
else
    echo -e "Status: ${YELLOW}⚠ Consider further optimization${NC}"
fi
echo ""

# Optional: Run size analysis
if command -v twiggy &> /dev/null; then
    echo -e "${BLUE}Top 10 largest functions (twiggy):${NC}"
    twiggy top -n 10 "$WASM_FILE"
    echo ""
    echo -e "For detailed analysis: twiggy top -n 20 $WASM_FILE"
    echo -e "For monomorphizations: twiggy monos $WASM_FILE"
elif command -v cargo-bloat &> /dev/null; then
    echo -e "${BLUE}Binary size breakdown (cargo-bloat):${NC}"
    cargo bloat --release --target wasm32-wasip1 -n 10
else
    echo -e "${YELLOW}Tip: Install twiggy or cargo-bloat for size analysis${NC}"
    echo -e "     cargo install twiggy"
    echo -e "     cargo install cargo-bloat"
fi
echo ""

echo -e "${GREEN}Build complete!${NC}"
echo ""
echo -e "Next steps:"
echo -e "  1. Test the extension: ./install-dev.sh"
echo -e "  2. Run benchmarks: cargo bench"
echo -e "  3. Profile if needed: cargo flamegraph"
echo ""
