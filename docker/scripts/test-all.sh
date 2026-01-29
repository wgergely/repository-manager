#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results"

echo "=============================================="
echo "Repository Manager Docker Integration Tests"
echo "=============================================="

# Clear results
rm -rf "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR"

FAILED_SUITES=()

run_test_suite() {
    local name="$1"
    local script="$2"

    echo ""
    echo "======================================"
    echo "Running: $name"
    echo "======================================"

    if "$SCRIPT_DIR/$script" 2>&1 | tee "$RESULTS_DIR/${name}.log"; then
        echo "✓ $name PASSED"
    else
        echo "✗ $name FAILED"
        FAILED_SUITES+=("$name")
    fi
}

run_test_suite "build-verify" "build-verify.sh" || true
run_test_suite "smoke-test" "smoke-test.sh" || true
run_test_suite "config-generation" "test-config-generation.sh" || true
run_test_suite "tool-reads-config" "test-tool-reads-config.sh" || true
run_test_suite "e2e" "test-e2e.sh" || true

echo ""
echo "=============================================="
echo "Final Summary"
echo "=============================================="

if [ ${#FAILED_SUITES[@]} -eq 0 ]; then
    echo "✓ ALL TEST SUITES PASSED"
    exit 0
else
    echo "✗ FAILED: ${FAILED_SUITES[*]}"
    exit 1
fi
