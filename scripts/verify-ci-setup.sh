#!/usr/bin/env bash
# Verify CI Setup Script
# This script checks if your branch has the necessary CI configuration
# and system dependencies to pass GitHub Actions workflows.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== Vintage Game Generator CI Setup Verification ==="
echo ""

# Check 1: Workflow file exists
echo -n "Checking for workflow file... "
if [ -f ".github/workflows/rust.yml" ]; then
    echo -e "${GREEN}✓${NC} Found"
else
    echo -e "${RED}✗${NC} Missing"
    echo -e "${YELLOW}Action required:${NC} Run 'git rebase origin/main' or manually copy workflow from main"
    exit 1
fi

# Check 2: Workflow contains ALSA dependency installation
echo -n "Checking for ALSA dependency installation... "
if grep -q "Install system dependencies" ".github/workflows/rust.yml" && \
   grep -q "libasound2-dev" ".github/workflows/rust.yml"; then
    echo -e "${GREEN}✓${NC} Configured"
else
    echo -e "${RED}✗${NC} Missing"
    echo -e "${YELLOW}Action required:${NC} Workflow file is outdated. Run 'git rebase origin/main'"
    exit 1
fi

# Check 3: System dependencies (if running locally)
if [ "$(uname)" = "Linux" ]; then
    echo -n "Checking for ALSA development libraries... "
    if dpkg -l | grep -q libasound2-dev; then
        echo -e "${GREEN}✓${NC} Installed"
    else
        echo -e "${YELLOW}⚠${NC} Not installed (OK in CI, needed for local builds)"
        echo -e "  To install: ${YELLOW}sudo apt-get install -y libasound2-dev libudev-dev${NC}"
    fi
fi

# Check 4: Rust formatting
echo -n "Checking code formatting... "
if cargo fmt --all -- --check > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Properly formatted"
else
    echo -e "${RED}✗${NC} Formatting issues found"
    echo -e "${YELLOW}Action required:${NC} Run 'cargo fmt --all'"
    exit 1
fi

# Check 5: Cargo check
echo -n "Running cargo check... "
CARGO_LOG=$(mktemp)
if cargo check --all-targets --all-features > "$CARGO_LOG" 2>&1; then
    echo -e "${GREEN}✓${NC} No build errors"
    rm -f "$CARGO_LOG"
else
    echo -e "${RED}✗${NC} Build failed"
    echo -e "${YELLOW}Review errors:${NC}"
    tail -20 "$CARGO_LOG"
    rm -f "$CARGO_LOG"
    exit 1
fi

# Check 6: Clippy
echo -n "Running clippy... "
CLIPPY_LOG=$(mktemp)
if cargo clippy --all-targets --all-features -- -D warnings > "$CLIPPY_LOG" 2>&1; then
    echo -e "${GREEN}✓${NC} No clippy warnings"
    rm -f "$CLIPPY_LOG"
else
    echo -e "${RED}✗${NC} Clippy warnings/errors found"
    echo -e "${YELLOW}Review output:${NC}"
    tail -20 "$CLIPPY_LOG"
    rm -f "$CLIPPY_LOG"
    exit 1
fi

echo ""
echo -e "${GREEN}=== All checks passed! ===${NC}"
echo "Your branch should pass CI workflows."
echo ""
echo "Next steps:"
echo "1. Review changes: git status"
echo "2. Stage changes: git add -A (review carefully first!)"
echo "3. Commit: git commit -m 'fix: apply CI requirements'"
echo "4. Push to your branch: git push origin <branch-name>"
echo "5. Check the GitHub Actions tab for workflow status"
