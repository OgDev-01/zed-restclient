#!/bin/bash

# REST Client Extension - Release Helper Script
# This script triggers the GitHub Actions release workflow from your terminal.
#
# Usage:
#   ./scripts/release.sh [patch|minor|major] "Changelog message"
#
# Examples:
#   ./scripts/release.sh patch "Fix header parsing bug"
#   ./scripts/release.sh minor "Add support for GraphQL requests"
#   ./scripts/release.sh major "Breaking: new request syntax"

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Check for required tools
check_requirements() {
    if ! command -v gh &> /dev/null; then
        error "GitHub CLI (gh) is required. Install it with: brew install gh"
    fi

    if ! gh auth status &> /dev/null; then
        error "GitHub CLI is not authenticated. Run: gh auth login"
    fi
}

# Get current version from extension.toml
get_current_version() {
    grep '^version = ' extension.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Calculate new version
calculate_new_version() {
    local current_version="$1"
    local bump_type="$2"

    IFS='.' read -r major minor patch <<< "$current_version"

    case "$bump_type" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            error "Invalid bump type: $bump_type. Use: patch, minor, or major"
            ;;
    esac
}

# Show usage
usage() {
    echo "Usage: $0 [patch|minor|major] \"Changelog message\""
    echo ""
    echo "Arguments:"
    echo "  patch   - Increment patch version (0.0.X) for bug fixes"
    echo "  minor   - Increment minor version (0.X.0) for new features"
    echo "  major   - Increment major version (X.0.0) for breaking changes"
    echo ""
    echo "Examples:"
    echo "  $0 patch \"Fix header parsing bug\""
    echo "  $0 minor \"Add GraphQL support\""
    echo "  $0 major \"Breaking: new request syntax\""
    exit 1
}

# Main script
main() {
    # Check arguments
    if [ $# -lt 2 ]; then
        usage
    fi

    local bump_type="$1"
    local changelog="$2"

    # Validate bump type
    if [[ ! "$bump_type" =~ ^(patch|minor|major)$ ]]; then
        error "Invalid bump type: $bump_type. Use: patch, minor, or major"
    fi

    # Check requirements
    check_requirements

    # Get versions
    local current_version
    current_version=$(get_current_version)

    local new_version
    new_version=$(calculate_new_version "$current_version" "$bump_type")

    # Confirm with user
    echo ""
    info "Release Details:"
    echo "  Current version: ${YELLOW}${current_version}${NC}"
    echo "  New version:     ${GREEN}${new_version}${NC}"
    echo "  Bump type:       ${BLUE}${bump_type}${NC}"
    echo "  Changelog:       ${changelog}"
    echo ""

    read -p "Proceed with release? (y/N) " -n 1 -r
    echo ""

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        warning "Release cancelled."
        exit 0
    fi

    # Trigger the workflow
    info "Triggering release workflow..."

    gh workflow run release.yml \
        --field bump_type="$bump_type" \
        --field changelog="$changelog"

    success "Release workflow triggered!"
    echo ""
    info "Monitor progress at:"
    echo "  $(gh browse -n)/actions/workflows/release.yml"
    echo ""
    info "The workflow will:"
    echo "  1. Bump version to ${new_version}"
    echo "  2. Update CHANGELOG.md"
    echo "  3. Create git tag v${new_version}"
    echo "  4. Build LSP binaries for all platforms"
    echo "  5. Create GitHub release with binaries"
}

# Run main function
main "$@"
