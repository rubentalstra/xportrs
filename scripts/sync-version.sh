#!/usr/bin/env bash
#
# sync-version.sh - Synchronize or check version across all metadata files
#
# Usage:
#   ./scripts/sync-version.sh         # Sync versions (default)
#   ./scripts/sync-version.sh --check # Check versions match (for CI)
#
# This script reads the version from Cargo.toml and updates/checks:
# - CITATION.cff
# - .zenodo.json
# - codemeta.json
# - README.md
# - docs/src/README.md
# - docs/src/guides/quickstart.md

set -euo pipefail

# Parse arguments
CHECK_MODE=false
if [[ "${1:-}" == "--check" ]]; then
    CHECK_MODE=true
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Extract version from Cargo.toml (source of truth)
CARGO_VERSION=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [ -z "$CARGO_VERSION" ]; then
    echo -e "${RED}Error: Could not extract version from Cargo.toml${NC}"
    exit 1
fi

# Extract versions from all files
CITATION_VERSION=$(grep -m1 '^version:' CITATION.cff 2>/dev/null | sed 's/version: //' || echo "")
ZENODO_VERSION=$(grep -m1 '"version"' .zenodo.json 2>/dev/null | sed 's/.*"version": "\([^"]*\)".*/\1/' || echo "")
CODEMETA_VERSION=$(grep -m1 '"version"' codemeta.json 2>/dev/null | sed 's/.*"version": "\([^"]*\)".*/\1/' || echo "")
README_VERSION=$(grep -oE 'xportrs = "[^"]+"' README.md 2>/dev/null | head -1 | sed 's/xportrs = "\([^"]*\)"/\1/' || echo "")
DOCS_README_VERSION=$(grep -oE 'xportrs = "[^"]+"' docs/src/README.md 2>/dev/null | head -1 | sed 's/xportrs = "\([^"]*\)"/\1/' || echo "")
QUICKSTART_VERSION=$(grep -oE 'xportrs = "[^"]+"' docs/src/guides/quickstart.md 2>/dev/null | head -1 | sed 's/xportrs = "\([^"]*\)"/\1/' || echo "")

if [ "$CHECK_MODE" = true ]; then
    # Check mode: validate all versions match
    echo "Checking version consistency..."
    echo ""
    echo -e "  Cargo.toml (source): ${GREEN}$CARGO_VERSION${NC}"
    echo ""

    MISMATCH=0

    # Check each file
    if [ "$CARGO_VERSION" != "$CITATION_VERSION" ]; then
        echo -e "  CITATION.cff: ${RED}$CITATION_VERSION${NC} (mismatch!)"
        MISMATCH=1
    else
        echo -e "  CITATION.cff: ${GREEN}$CITATION_VERSION${NC}"
    fi

    if [ "$CARGO_VERSION" != "$ZENODO_VERSION" ]; then
        echo -e "  .zenodo.json: ${RED}$ZENODO_VERSION${NC} (mismatch!)"
        MISMATCH=1
    else
        echo -e "  .zenodo.json: ${GREEN}$ZENODO_VERSION${NC}"
    fi

    if [ "$CARGO_VERSION" != "$CODEMETA_VERSION" ]; then
        echo -e "  codemeta.json: ${RED}$CODEMETA_VERSION${NC} (mismatch!)"
        MISMATCH=1
    else
        echo -e "  codemeta.json: ${GREEN}$CODEMETA_VERSION${NC}"
    fi

    if [ "$CARGO_VERSION" != "$README_VERSION" ]; then
        echo -e "  README.md: ${RED}$README_VERSION${NC} (mismatch!)"
        MISMATCH=1
    else
        echo -e "  README.md: ${GREEN}$README_VERSION${NC}"
    fi

    if [ "$CARGO_VERSION" != "$DOCS_README_VERSION" ]; then
        echo -e "  docs/src/README.md: ${RED}$DOCS_README_VERSION${NC} (mismatch!)"
        MISMATCH=1
    else
        echo -e "  docs/src/README.md: ${GREEN}$DOCS_README_VERSION${NC}"
    fi

    if [ "$CARGO_VERSION" != "$QUICKSTART_VERSION" ]; then
        echo -e "  docs/src/guides/quickstart.md: ${RED}$QUICKSTART_VERSION${NC} (mismatch!)"
        MISMATCH=1
    else
        echo -e "  docs/src/guides/quickstart.md: ${GREEN}$QUICKSTART_VERSION${NC}"
    fi

    echo ""
    if [ $MISMATCH -eq 1 ]; then
        echo -e "${RED}Version mismatch detected!${NC}"
        echo "Run './scripts/sync-version.sh' to fix version mismatches"
        exit 1
    fi

    echo -e "${GREEN}All versions match: $CARGO_VERSION${NC}"
    exit 0
fi

# Sync mode: update all files
DATE=$(date +%Y-%m-%d)

echo -e "${YELLOW}Syncing version ${GREEN}$CARGO_VERSION${YELLOW} (date: $DATE)${NC}"
echo ""

# Update CITATION.cff
echo -n "  Updating CITATION.cff... "
sed -i.bak "s/^version: .*/version: $CARGO_VERSION/" CITATION.cff
sed -i.bak "s/^date-released: .*/date-released: '$DATE'/" CITATION.cff
rm -f CITATION.cff.bak
echo -e "${GREEN}done${NC}"

# Update .zenodo.json
echo -n "  Updating .zenodo.json... "
sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$CARGO_VERSION\"/" .zenodo.json
rm -f .zenodo.json.bak
echo -e "${GREEN}done${NC}"

# Update codemeta.json
echo -n "  Updating codemeta.json... "
sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$CARGO_VERSION\"/" codemeta.json
sed -i.bak "s/\"dateModified\": \"[^\"]*\"/\"dateModified\": \"$DATE\"/" codemeta.json
rm -f codemeta.json.bak
echo -e "${GREEN}done${NC}"

# Update README.md
echo -n "  Updating README.md... "
sed -i.bak "s/xportrs = \"[^\"]*\"/xportrs = \"$CARGO_VERSION\"/" README.md
rm -f README.md.bak
echo -e "${GREEN}done${NC}"

# Update docs/src/README.md
echo -n "  Updating docs/src/README.md... "
sed -i.bak "s/xportrs = \"[^\"]*\"/xportrs = \"$CARGO_VERSION\"/" docs/src/README.md
rm -f docs/src/README.md.bak
echo -e "${GREEN}done${NC}"

# Update docs/src/guides/quickstart.md
echo -n "  Updating docs/src/guides/quickstart.md... "
sed -i.bak "s/xportrs = \"[^\"]*\"/xportrs = \"$CARGO_VERSION\"/" docs/src/guides/quickstart.md
rm -f docs/src/guides/quickstart.md.bak
echo -e "${GREEN}done${NC}"

echo ""
echo -e "${GREEN}All files updated to version $CARGO_VERSION${NC}"
echo ""

# Verify JSON files are valid
echo "Validating JSON files..."
if command -v python3 &> /dev/null; then
    python3 -m json.tool .zenodo.json > /dev/null && echo -e "  .zenodo.json: ${GREEN}valid${NC}"
    python3 -m json.tool codemeta.json > /dev/null && echo -e "  codemeta.json: ${GREEN}valid${NC}"
else
    echo -e "  ${YELLOW}Skipping JSON validation (python3 not found)${NC}"
fi

echo ""
echo -e "${GREEN}Version sync complete!${NC}"
