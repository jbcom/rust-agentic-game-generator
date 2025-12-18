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
if grep -q "libasound2-dev" ".github/workflows/rust.yml"; then
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
if cargo check --all-targets --all-features > /tmp/cargo-check.log 2>&1; then
    echo -e "${GREEN}✓${NC} No build errors"
else
    echo -e "${RED}✗${NC} Build failed"
    echo -e "${YELLOW}Review errors:${NC}"
    tail -20 /tmp/cargo-check.log
    exit 1
fi

# Check 6: Clippy
echo -n "Running clippy... "
if cargo clippy --all-targets --all-features -- -D warnings > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} No clippy warnings"
else
    echo -e "${RED}✗${NC} Clippy warnings/errors found"
    echo -e "${YELLOW}Action required:${NC} Run 'cargo clippy --all-targets --all-features' to see details"
    exit 1
fi

echo ""
echo -e "${GREEN}=== All checks passed! ===${NC}"
echo "Your branch should pass CI workflows."
echo ""
echo "Next steps:"
echo "1. Commit any changes: git add . && git commit -m 'fix: apply CI requirements'"
echo "2. Push to your branch: git push origin <branch-name>"
echo "3. Check the GitHub Actions tab for workflow status"
