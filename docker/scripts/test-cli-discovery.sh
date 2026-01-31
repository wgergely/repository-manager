#!/bin/bash
# CLI Discovery Tests
# Tests the new list-tools, list-presets, status, and completions commands

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/cli-discovery"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log_test() {
    local name="$1"
    local status="$2"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if [ "$status" = "PASS" ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "  ${GREEN}✓${NC} $name"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "  ${RED}✗${NC} $name"
    fi
}

WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}CLI Discovery Command Tests${NC}"
echo "Testing list-tools, list-presets, status, and completions"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

REPO_BIN="$PROJECT_ROOT/target/debug/repo"

# Build if needed
if [ ! -f "$REPO_BIN" ]; then
    echo "Building repo CLI..."
    cd "$PROJECT_ROOT" && cargo build -p repo-cli --quiet
fi

# ============================================
# list-tools Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: list-tools${NC}"

if $REPO_BIN list-tools 2>&1 | grep -q "Available Tools"; then
    log_test "list-tools shows Available Tools header" "PASS"
else
    log_test "list-tools shows Available Tools header" "FAIL"
fi

if $REPO_BIN list-tools 2>&1 | grep -q "claude"; then
    log_test "list-tools shows claude" "PASS"
else
    log_test "list-tools shows claude" "FAIL"
fi

if $REPO_BIN list-tools 2>&1 | grep -q "cursor"; then
    log_test "list-tools shows cursor" "PASS"
else
    log_test "list-tools shows cursor" "FAIL"
fi

if $REPO_BIN list-tools --category ide 2>&1 | grep -q "IDE Tools"; then
    log_test "list-tools --category ide works" "PASS"
else
    log_test "list-tools --category ide works" "FAIL"
fi

if $REPO_BIN list-tools 2>&1 | grep -q "Total:"; then
    log_test "list-tools shows total count" "PASS"
else
    log_test "list-tools shows total count" "FAIL"
fi

# ============================================
# list-presets Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: list-presets${NC}"

if $REPO_BIN list-presets 2>&1 | grep -q "Available Presets"; then
    log_test "list-presets shows Available Presets header" "PASS"
else
    log_test "list-presets shows Available Presets header" "FAIL"
fi

if $REPO_BIN list-presets 2>&1 | grep -q "env:python"; then
    log_test "list-presets shows env:python" "PASS"
else
    log_test "list-presets shows env:python" "FAIL"
fi

# ============================================
# status Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: status${NC}"

cd "$WORK_DIR"
if $REPO_BIN status 2>&1 | grep -q "Not a repository"; then
    log_test "status in non-repo shows not initialized" "PASS"
else
    log_test "status in non-repo shows not initialized" "FAIL"
fi

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor"]

[core]
mode = "standard"
EOF
if $REPO_BIN status 2>&1 | grep -q "Repository Status"; then
    log_test "status in repo shows Repository Status" "PASS"
else
    log_test "status in repo shows Repository Status" "FAIL"
fi

if $REPO_BIN status 2>&1 | grep -q "Enabled Tools"; then
    log_test "status shows Enabled Tools section" "PASS"
else
    log_test "status shows Enabled Tools section" "FAIL"
fi

if $REPO_BIN status 2>&1 | grep -q "standard"; then
    log_test "status shows mode" "PASS"
else
    log_test "status shows mode" "FAIL"
fi

# ============================================
# completions Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: completions${NC}"

if $REPO_BIN completions bash 2>&1 | grep -q "complete"; then
    log_test "completions bash produces script" "PASS"
else
    log_test "completions bash produces script" "FAIL"
fi

if $REPO_BIN completions zsh 2>&1 | grep -q "#compdef"; then
    log_test "completions zsh produces script" "PASS"
else
    log_test "completions zsh produces script" "FAIL"
fi

# ============================================
# --help Tests
# ============================================
echo ""
echo -e "${YELLOW}Testing: --help improvements${NC}"

if $REPO_BIN add-tool --help 2>&1 | grep -q "list-tools"; then
    log_test "add-tool --help references list-tools" "PASS"
else
    log_test "add-tool --help references list-tools" "FAIL"
fi

if $REPO_BIN list-tools --help 2>&1 | grep -qi "example"; then
    log_test "list-tools --help shows examples" "PASS"
else
    log_test "list-tools --help shows examples" "FAIL"
fi

# ============================================
# Summary
# ============================================
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}CLI Discovery Test Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  Total Tests:  $TOTAL_TESTS"
echo -e "  Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "  Failed:       ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}${BOLD}ALL CLI DISCOVERY TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}SOME TESTS FAILED${NC}"
    exit 1
fi
