#!/bin/bash
# Config generation test script
# Tests that Repository Manager generates configs for Cursor, Claude, and Aider

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/test-results/config-gen"
FIXTURES_DIR="$PROJECT_ROOT/test-fixtures"

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
    echo "Config generation test results - $(date)" > "$RESULTS_DIR/summary.txt"
    echo "============================================" >> "$RESULTS_DIR/summary.txt"
}

# Log function
log() {
    echo -e "$1"
}

# Create working directory
WORK_DIR=""
cleanup() {
    if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
        log "${YELLOW}Cleaning up temporary directory...${NC}"
        rm -rf "$WORK_DIR"
    fi
}
trap cleanup EXIT

setup_work_dir() {
    WORK_DIR=$(mktemp -d)
    log "Working directory: $WORK_DIR"

    # Copy test fixture
    if [ ! -d "$FIXTURES_DIR/repos/config-test" ]; then
        log "${RED}ERROR: Test fixture not found at $FIXTURES_DIR/repos/config-test${NC}"
        exit 1
    fi

    cp -r "$FIXTURES_DIR/repos/config-test/." "$WORK_DIR/"

    # Initialize a fake git repo (required for sync to work)
    (cd "$WORK_DIR" && git init --quiet)

    log "Copied test fixture to working directory"
}

# Run Repository Manager sync to generate all configs
run_sync() {
    log ""
    log "${YELLOW}>>> Running 'repo sync' to generate configs...${NC}"

    local log_file="$RESULTS_DIR/sync.log"

    # Run repo sync command
    if docker run --rm \
        -v "$WORK_DIR:/workspace/test-project" \
        repo-test/repo-manager:latest \
        sync 2>&1 | tee "$log_file"; then
        log "${GREEN}    Sync command completed${NC}"
        return 0
    else
        log "${RED}    Sync command failed${NC}"
        return 1
    fi
}

# Test if a config file was created
test_config_file() {
    local tool="$1"
    local config_file="$2"
    local expected_file="$3"

    log ""
    log "${YELLOW}>>> Testing $tool config generation...${NC}"

    local full_path="$WORK_DIR/$config_file"
    local log_file="$RESULTS_DIR/${tool}.log"

    # Check if config exists
    if [ -f "$full_path" ]; then
        log "    Config file: ${GREEN}created${NC}"
        echo "Config file created: $config_file" > "$log_file"

        # Copy the generated config to results for inspection
        cp "$full_path" "$RESULTS_DIR/${tool}-generated$(basename "$config_file" | sed 's/\(.*\)/.\1/')" 2>/dev/null || \
        cp "$full_path" "$RESULTS_DIR/${tool}-generated.txt"

        # Optionally compare with expected output
        if [ -n "$expected_file" ] && [ -f "$expected_file" ]; then
            if diff -q "$full_path" "$expected_file" > /dev/null 2>&1; then
                log "    Expected match: ${GREEN}yes${NC}"
                echo "Matches expected output" >> "$log_file"
            else
                log "    Expected match: ${YELLOW}differs (see diff below)${NC}"
                echo "Differs from expected output:" >> "$log_file"
                diff "$full_path" "$expected_file" >> "$log_file" 2>&1 || true

                # Show abbreviated diff
                log "    Diff:"
                diff "$full_path" "$expected_file" 2>&1 | head -10 | while read -r line; do
                    log "      $line"
                done
            fi
        else
            log "    Expected file: ${YELLOW}not found (skipping comparison)${NC}"
            echo "No expected file for comparison" >> "$log_file"
        fi

        PASSED_TESTS+=("$tool")
        echo "PASS: $tool" >> "$RESULTS_DIR/summary.txt"
        log "    Result: ${GREEN}PASS${NC}"
        return 0
    else
        log "    Config file: ${RED}NOT created${NC}"
        echo "Config file NOT created: $config_file" > "$log_file"

        FAILED_TESTS+=("$tool")
        echo "FAIL: $tool" >> "$RESULTS_DIR/summary.txt"
        log "    Result: ${RED}FAIL${NC}"
        return 1
    fi
}

# Print summary
print_summary() {
    local total=$((${#PASSED_TESTS[@]} + ${#FAILED_TESTS[@]}))

    log ""
    log "========================================"
    log "     CONFIG GENERATION TEST SUMMARY"
    log "========================================"
    log ""
    log "Total tests: $total"
    log "Passed: ${GREEN}${#PASSED_TESTS[@]}${NC}"
    log "Failed: ${RED}${#FAILED_TESTS[@]}${NC}"
    log ""

    # Append to summary file
    echo "" >> "$RESULTS_DIR/summary.txt"
    echo "Total: $total | Passed: ${#PASSED_TESTS[@]} | Failed: ${#FAILED_TESTS[@]}" >> "$RESULTS_DIR/summary.txt"

    if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
        log "${RED}Failed tests:${NC}"
        for test in "${FAILED_TESTS[@]}"; do
            log "  - $test"
            log "    Log: $RESULTS_DIR/${test}.log"
        done
        log ""
        echo "FAILED: ${FAILED_TESTS[*]}" >> "$RESULTS_DIR/summary.txt"
    fi

    # List generated files
    log "Generated files in working directory:"
    ls -la "$WORK_DIR" 2>/dev/null | while read -r line; do
        log "  $line"
    done

    log ""
    log "Results saved to: $RESULTS_DIR"
}

# Main execution
main() {
    log "${YELLOW}Starting config generation tests...${NC}"
    log ""

    setup_results_dir
    setup_work_dir

    # Run sync to generate all configs
    run_sync

    # Test each tool config
    test_config_file "cursor" ".cursorrules" "$FIXTURES_DIR/expected/cursor/.cursorrules"
    test_config_file "claude" "CLAUDE.md" "$FIXTURES_DIR/expected/claude/CLAUDE.md"
    test_config_file "aider" ".aider.conf.yml" "$FIXTURES_DIR/expected/aider/.aider.conf.yml"

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
