#!/bin/bash
# Smoke test script for Docker image verification
# Verifies each tool image can run basic commands

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="${PROJECT_ROOT}/test-results/smoke"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED_TESTS=()
PASSED_TESTS=()

# Setup results directory
setup_results_dir() {
    mkdir -p "$RESULTS_DIR"
    echo "Smoke test results - $(date)" > "$RESULTS_DIR/summary.txt"
    echo "================================" >> "$RESULTS_DIR/summary.txt"
}

# Log function
log() {
    echo -e "$1"
}

# Smoke test function
# Args: name, image, command, expected (optional grep pattern)
smoke_test() {
    local name="$1"
    local image="$2"
    local command="$3"
    local expected="$4"  # Optional grep pattern

    local log_file="$RESULTS_DIR/${name}.log"
    local status="PASS"
    local exit_code=0

    log "${YELLOW}Testing: ${name}${NC}"
    log "  Image: $image"
    log "  Command: $command"

    # Run the docker command and capture output
    if docker run --rm "$image" $command > "$log_file" 2>&1; then
        exit_code=0
    else
        exit_code=$?
    fi

    # Check if expected pattern was provided and if output matches
    if [ -n "$expected" ]; then
        if grep -qi "$expected" "$log_file" 2>/dev/null; then
            log "  Expected pattern '$expected': ${GREEN}found${NC}"
        else
            log "  Expected pattern '$expected': ${RED}not found${NC}"
            status="FAIL"
        fi
    else
        # No expected pattern, just check exit code
        if [ $exit_code -eq 0 ]; then
            log "  Exit code: ${GREEN}0 (success)${NC}"
        else
            log "  Exit code: ${RED}${exit_code} (failure)${NC}"
            status="FAIL"
        fi
    fi

    # Record result
    if [ "$status" = "PASS" ]; then
        log "  Result: ${GREEN}PASS${NC}"
        PASSED_TESTS+=("$name")
        echo "PASS: $name" >> "$RESULTS_DIR/summary.txt"
    else
        log "  Result: ${RED}FAIL${NC}"
        FAILED_TESTS+=("$name")
        echo "FAIL: $name" >> "$RESULTS_DIR/summary.txt"
    fi

    log ""
}

# Print summary
print_summary() {
    local total=$((${#PASSED_TESTS[@]} + ${#FAILED_TESTS[@]}))

    echo ""
    echo "========================================"
    echo "           SMOKE TEST SUMMARY"
    echo "========================================"
    echo ""
    echo "Total tests: $total"
    echo -e "Passed: ${GREEN}${#PASSED_TESTS[@]}${NC}"
    echo -e "Failed: ${RED}${#FAILED_TESTS[@]}${NC}"
    echo ""

    # Append to summary file
    echo "" >> "$RESULTS_DIR/summary.txt"
    echo "Total: $total | Passed: ${#PASSED_TESTS[@]} | Failed: ${#FAILED_TESTS[@]}" >> "$RESULTS_DIR/summary.txt"

    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        echo -e "${RED}Failed tests:${NC}"
        for test in "${FAILED_TESTS[@]}"; do
            echo "  - $test"
            echo "    Log: $RESULTS_DIR/${test}.log"
        done
        echo ""
        echo "FAILED: ${FAILED_TESTS[*]}" >> "$RESULTS_DIR/summary.txt"
    fi

    echo "Results saved to: $RESULTS_DIR"
}

# Main execution
main() {
    log "${YELLOW}Starting smoke tests...${NC}"
    log ""

    setup_results_dir

    # Run smoke tests for each tool

    # Claude CLI
    smoke_test "claude-help" \
        "repo-test/claude:latest" \
        "--help" \
        "claude"

    # Aider
    smoke_test "aider-version" \
        "repo-test/aider:latest" \
        "--version" \
        "aider"

    # Gemini CLI
    smoke_test "gemini-help" \
        "repo-test/gemini:latest" \
        "--help" \
        ""  # Just check command succeeds

    # Cursor CLI
    smoke_test "cursor-help" \
        "repo-test/cursor:latest" \
        "--help" \
        ""  # Just check command succeeds

    # Cline (VS Code extension)
    smoke_test "cline-extensions" \
        "repo-test/cline:latest" \
        "--list-extensions" \
        "claude-dev"

    # Roo (VS Code extension)
    smoke_test "roo-extensions" \
        "repo-test/roo:latest" \
        "--list-extensions" \
        "roo-cline"

    # Print summary
    print_summary

    # Exit with appropriate code
    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        exit 1
    else
        exit 0
    fi
}

# Run main
main "$@"
