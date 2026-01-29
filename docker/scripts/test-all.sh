#!/bin/bash
# Master Test Runner for Repository Manager Integration Tests
# Runs all test suites and generates comprehensive report

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "=============================================="
echo "  Repository Manager Integration Test Suite"
echo "=============================================="
echo ""
echo "Project: $PROJECT_ROOT"
echo "Results: $RESULTS_DIR"
echo ""

# Clear results directory
rm -rf "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR"

# Check Docker availability
DOCKER_AVAILABLE=false
if docker info >/dev/null 2>&1; then
    DOCKER_AVAILABLE=true
    echo -e "Docker: ${GREEN}Available${NC}"
else
    echo -e "Docker: ${YELLOW}Not Available${NC} (skipping container tests)"
fi
echo ""

# Test suite tracking
TOTAL_SUITES=0
PASSED_SUITES=0
SKIPPED_SUITES=0
FAILED_SUITES=()

run_test_suite() {
    local name="$1"
    local script="$2"
    local requires_docker="${3:-false}"

    TOTAL_SUITES=$((TOTAL_SUITES + 1))

    echo ""
    echo -e "${BLUE}══════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Running: $name${NC}"
    echo -e "${BLUE}══════════════════════════════════════════${NC}"

    # Skip Docker-dependent tests if Docker unavailable
    if [ "$requires_docker" = "true" ] && [ "$DOCKER_AVAILABLE" = "false" ]; then
        echo -e "  ${YELLOW}SKIPPED${NC} (Docker required)"
        SKIPPED_SUITES=$((SKIPPED_SUITES + 1))
        echo "SKIPPED: Docker not available" > "$RESULTS_DIR/${name}.log"
        return 0
    fi

    # Run the test suite
    if "$SCRIPT_DIR/$script" 2>&1 | tee "$RESULTS_DIR/${name}.log"; then
        echo ""
        echo -e "  ${GREEN}✓ $name PASSED${NC}"
        PASSED_SUITES=$((PASSED_SUITES + 1))
        return 0
    else
        echo ""
        echo -e "  ${RED}✗ $name FAILED${NC}"
        FAILED_SUITES+=("$name")
        return 1
    fi
}

# ============================================
# TIER 1: Local Validation (No Docker)
# ============================================
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  TIER 1: LOCAL VALIDATION TESTS${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"

run_test_suite "drift-detection" "test-drift-detection.sh" false || true
run_test_suite "developer-workflow" "test-developer-workflow.sh" false || true

# ============================================
# TIER 2: Docker Build Tests
# ============================================
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  TIER 2: DOCKER BUILD TESTS${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"

run_test_suite "build-verify" "build-verify.sh" true || true
run_test_suite "smoke-test" "smoke-test.sh" true || true

# ============================================
# TIER 3: Integration Tests
# ============================================
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  TIER 3: INTEGRATION TESTS${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"

run_test_suite "config-generation" "test-config-generation.sh" true || true
run_test_suite "tool-reads-config" "test-tool-reads-config.sh" true || true
run_test_suite "e2e" "test-e2e.sh" true || true

# ============================================
# FINAL SUMMARY
# ============================================
echo ""
echo "=============================================="
echo "           FINAL TEST SUMMARY"
echo "=============================================="
echo ""

FAILED_COUNT=${#FAILED_SUITES[@]}

echo "Total Suites:   $TOTAL_SUITES"
echo -e "Passed:         ${GREEN}$PASSED_SUITES${NC}"
echo -e "Skipped:        ${YELLOW}$SKIPPED_SUITES${NC}"
echo -e "Failed:         ${RED}$FAILED_COUNT${NC}"
echo ""

# Generate summary report
cat > "$RESULTS_DIR/SUMMARY.md" << EOF
# Test Run Summary

**Timestamp:** $(date -Iseconds)
**Docker Available:** $DOCKER_AVAILABLE

## Results

| Metric | Count |
|--------|-------|
| Total Suites | $TOTAL_SUITES |
| Passed | $PASSED_SUITES |
| Skipped | $SKIPPED_SUITES |
| Failed | $FAILED_COUNT |

## Suite Results

| Suite | Status |
|-------|--------|
EOF

for log in "$RESULTS_DIR"/*.log; do
    suite=$(basename "$log" .log)
    if grep -q "SKIPPED" "$log" 2>/dev/null; then
        status="⏭️ Skipped"
    elif grep -q "PASSED\|passed\|All.*pass" "$log" 2>/dev/null; then
        status="✅ Passed"
    else
        status="❌ Failed"
    fi
    echo "| $suite | $status |" >> "$RESULTS_DIR/SUMMARY.md"
done

cat >> "$RESULTS_DIR/SUMMARY.md" << EOF

## Failed Suites

$(if [ $FAILED_COUNT -gt 0 ]; then
    printf '%s\n' "${FAILED_SUITES[@]}" | sed 's/^/- /'
else
    echo "None"
fi)

## Log Files

$(ls -la "$RESULTS_DIR"/*.log 2>/dev/null | awk '{print "- " $NF}')
EOF

echo "Summary report: $RESULTS_DIR/SUMMARY.md"
echo ""

# Exit with appropriate code
if [ $FAILED_COUNT -gt 0 ]; then
    echo -e "${RED}══════════════════════════════════════════${NC}"
    echo -e "${RED}  TEST SUITE FAILED${NC}"
    echo -e "${RED}══════════════════════════════════════════${NC}"
    echo ""
    echo "Failed suites:"
    printf '  - %s\n' "${FAILED_SUITES[@]}"
    exit 1
elif [ $SKIPPED_SUITES -eq $TOTAL_SUITES ]; then
    echo -e "${YELLOW}══════════════════════════════════════════${NC}"
    echo -e "${YELLOW}  ALL TESTS SKIPPED (Docker unavailable)${NC}"
    echo -e "${YELLOW}══════════════════════════════════════════${NC}"
    exit 0
else
    echo -e "${GREEN}══════════════════════════════════════════${NC}"
    echo -e "${GREEN}  ALL TESTS PASSED${NC}"
    echo -e "${GREEN}══════════════════════════════════════════${NC}"
    exit 0
fi
