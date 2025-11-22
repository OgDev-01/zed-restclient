#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}REST Client Extension - Installation Verification${NC}"
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
    exit 1
fi

echo -e "${BLUE}Checking installation for:${NC} $OS_NAME"
echo ""

ISSUES_FOUND=0

# Check installed directory
echo -e "${YELLOW}📁 Checking installed directory...${NC}"
echo "   Location: $INSTALLED_DIR"

if [ ! -d "$INSTALLED_DIR" ]; then
    echo -e "${RED}   ❌ Directory does not exist!${NC}"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    echo -e "${GREEN}   ✓ Directory exists${NC}"

    # Check for required files
    echo ""
    echo -e "${YELLOW}📋 Checking required files in installed directory...${NC}"

    if [ -f "$INSTALLED_DIR/extension.toml" ]; then
        echo -e "${GREEN}   ✓ extension.toml${NC}"
    else
        echo -e "${RED}   ❌ extension.toml missing${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    if [ -f "$INSTALLED_DIR/extension.wasm" ]; then
        WASM_SIZE=$(du -h "$INSTALLED_DIR/extension.wasm" | cut -f1)
        echo -e "${GREEN}   ✓ extension.wasm ($WASM_SIZE)${NC}"
    else
        echo -e "${RED}   ❌ extension.wasm missing${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    if [ -f "$INSTALLED_DIR/lsp-server" ]; then
        LSP_SIZE=$(du -h "$INSTALLED_DIR/lsp-server" | cut -f1)
        if [ -x "$INSTALLED_DIR/lsp-server" ]; then
            echo -e "${GREEN}   ✓ lsp-server ($LSP_SIZE, executable)${NC}"
        else
            echo -e "${YELLOW}   ⚠️  lsp-server ($LSP_SIZE, NOT executable)${NC}"
            echo -e "${YELLOW}      Run: chmod +x \"$INSTALLED_DIR/lsp-server\"${NC}"
            ISSUES_FOUND=$((ISSUES_FOUND + 1))
        fi
    else
        echo -e "${RED}   ❌ lsp-server missing${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    if [ -d "$INSTALLED_DIR/languages" ]; then
        echo -e "${GREEN}   ✓ languages/ directory${NC}"
    else
        echo -e "${YELLOW}   ⚠️  languages/ directory missing (syntax highlighting may not work)${NC}"
    fi

    if [ -d "$INSTALLED_DIR/grammars" ]; then
        echo -e "${GREEN}   ✓ grammars/ directory${NC}"
    else
        echo -e "${YELLOW}   ⚠️  grammars/ directory missing (may be generated on first load)${NC}"
    fi
fi

echo ""

# Check work directory
echo -e "${YELLOW}📁 Checking work directory...${NC}"
echo "   Location: $WORK_DIR"

if [ ! -d "$WORK_DIR" ]; then
    echo -e "${RED}   ❌ Directory does not exist!${NC}"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
else
    echo -e "${GREEN}   ✓ Directory exists${NC}"

    # Check for required files in work directory
    echo ""
    echo -e "${YELLOW}📋 Checking required files in work directory...${NC}"

    if [ -f "$WORK_DIR/extension.toml" ]; then
        echo -e "${GREEN}   ✓ extension.toml${NC}"
    else
        echo -e "${RED}   ❌ extension.toml missing${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    if [ -f "$WORK_DIR/extension.wasm" ]; then
        WASM_SIZE=$(du -h "$WORK_DIR/extension.wasm" | cut -f1)
        echo -e "${GREEN}   ✓ extension.wasm ($WASM_SIZE)${NC}"
    else
        echo -e "${RED}   ❌ extension.wasm missing${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    if [ -f "$WORK_DIR/lsp-server" ]; then
        LSP_SIZE=$(du -h "$WORK_DIR/lsp-server" | cut -f1)
        if [ -x "$WORK_DIR/lsp-server" ]; then
            echo -e "${GREEN}   ✓ lsp-server ($LSP_SIZE, executable)${NC}"
        else
            echo -e "${YELLOW}   ⚠️  lsp-server ($LSP_SIZE, NOT executable)${NC}"
            echo -e "${YELLOW}      Run: chmod +x \"$WORK_DIR/lsp-server\"${NC}"
            ISSUES_FOUND=$((ISSUES_FOUND + 1))
        fi
    else
        echo -e "${RED}   ❌ lsp-server missing${NC}"
        ISSUES_FOUND=$((ISSUES_FOUND + 1))
    fi

    if [ -d "$WORK_DIR/languages" ]; then
        echo -e "${GREEN}   ✓ languages/ directory${NC}"
    else
        echo -e "${YELLOW}   ⚠️  languages/ directory missing${NC}"
    fi
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [ $ISSUES_FOUND -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! Extension appears to be installed correctly.${NC}"
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "   1. ${BLUE}Completely quit Zed${NC} (Cmd+Q on macOS, don't just close windows)"
    echo "   2. ${BLUE}Restart Zed${NC}"
    echo "   3. Open command palette (Cmd+Shift+P)"
    echo "   4. Type 'zed: extensions'"
    echo "   5. Look for 'REST Client' in the list"
    echo ""
    echo -e "${YELLOW}If you still don't see the extension:${NC}"
    echo "   - Check Zed's logs: Help → View Error Log"
    echo "   - Try running the install script again"
    echo "   - Make sure you're using Zed version 0.100.0 or later"
else
    echo -e "${RED}❌ Found $ISSUES_FOUND issue(s) with the installation.${NC}"
    echo ""
    echo -e "${YELLOW}To fix these issues:${NC}"
    echo "   Run the installation script again:"
    echo "   ${BLUE}./install-dev.sh${NC}"
    echo ""
    echo "   If problems persist, try a clean install:"
    echo "   ${BLUE}rm -rf \"$INSTALLED_DIR\" \"$WORK_DIR\"${NC}"
    echo "   ${BLUE}./install-dev.sh${NC}"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Show file listings if directories exist
if [ -d "$INSTALLED_DIR" ]; then
    echo -e "${BLUE}Installed directory contents:${NC}"
    ls -lh "$INSTALLED_DIR" | tail -n +2 | awk '{print "   " $9 " (" $5 ")"}'
    echo ""
fi

if [ -d "$WORK_DIR" ]; then
    echo -e "${BLUE}Work directory contents:${NC}"
    ls -lh "$WORK_DIR" | tail -n +2 | awk '{print "   " $9 " (" $5 ")"}'
    echo ""
fi
