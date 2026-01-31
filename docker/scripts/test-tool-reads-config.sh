#!/bin/bash
# Tool config reading test script
# Tests that each AI tool container can read its generated config file
# This verifies the volume mounting and file permissions work correctly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/test-results/tool-reads"
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
    echo "Tool config reading test results - $(date)" > "$RESULTS_DIR/summary.txt"
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

    local log_file="$RESULTS_DIR/sync-all.log"

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

# Test if a tool container can read a config file
# Args: tool_name, image_name, config_path, expected_pattern
test_tool_reads_config() {
    local tool="$1"
    local image="$2"
    local config_path="$3"
    local expected_pattern="$4"

    log ""
    log "${YELLOW}>>> Testing $tool reads $config_path...${NC}"

    local log_file="$RESULTS_DIR/${tool}-read.log"
    local status="PASS"

    # Run docker with entrypoint override to cat the config file
    if docker run --rm \
        -v "$WORK_DIR:/workspace" \
        --entrypoint /bin/bash \
        "$image" \
        -c "cat /workspace/$config_path && echo '' && echo '$tool config readable'" > "$log_file" 2>&1; then

        # Check if output contains expected pattern
        if [ -n "$expected_pattern" ]; then
            if grep -q "$expected_pattern" "$log_file" 2>/dev/null; then
                log "    File read: ${GREEN}success${NC}"
                log "    Expected pattern '$expected_pattern': ${GREEN}found${NC}"
            else
                log "    File read: ${GREEN}success${NC}"
                log "    Expected pattern '$expected_pattern': ${RED}not found${NC}"
                status="FAIL"
            fi
        else
            log "    File read: ${GREEN}success${NC}"
        fi
    else
        log "    File read: ${RED}failed${NC}"
        status="FAIL"
    fi

    # Record result
    if [ "$status" = "PASS" ]; then
        log "    Result: ${GREEN}PASS${NC}"
        PASSED_TESTS+=("$tool")
        echo "PASS: $tool can read $config_path" >> "$RESULTS_DIR/summary.txt"
    else
        log "    Result: ${RED}FAIL${NC}"
        FAILED_TESTS+=("$tool")
        echo "FAIL: $tool cannot read $config_path" >> "$RESULTS_DIR/summary.txt"
    fi
}

# Test if a tool container can see a directory
# Args: tool_name, image_name, dir_path
test_tool_sees_directory() {
    local tool="$1"
    local image="$2"
    local dir_path="$3"

    log ""
    log "${YELLOW}>>> Testing $tool sees $dir_path directory...${NC}"

    local log_file="$RESULTS_DIR/${tool}-read.log"
    local status="PASS"

    # Run docker with entrypoint override to list the directory
    if docker run --rm \
        -v "$WORK_DIR:/workspace" \
        --entrypoint /bin/bash \
        "$image" \
        -c "ls -la /workspace/$dir_path && echo '' && echo '$tool can see $dir_path'" > "$log_file" 2>&1; then
        log "    Directory visible: ${GREEN}yes${NC}"
    else
        log "    Directory visible: ${RED}no${NC}"
        status="FAIL"
    fi

    # Record result
    if [ "$status" = "PASS" ]; then
        log "    Result: ${GREEN}PASS${NC}"
        PASSED_TESTS+=("$tool")
        echo "PASS: $tool can see $dir_path" >> "$RESULTS_DIR/summary.txt"
    else
        log "    Result: ${RED}FAIL${NC}"
        FAILED_TESTS+=("$tool")
        echo "FAIL: $tool cannot see $dir_path" >> "$RESULTS_DIR/summary.txt"
    fi
}

# Print summary
print_summary() {
    local total=$((${#PASSED_TESTS[@]} + ${#FAILED_TESTS[@]}))

    log ""
    log "========================================"
    log "     TOOL CONFIG READING TEST SUMMARY"
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
            log "    Log: $RESULTS_DIR/${test}-read.log"
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
    log "${YELLOW}Starting tool config reading tests...${NC}"
    log ""

    setup_results_dir
    setup_work_dir

    # Run sync to generate all configs
    run_sync

    # Test Claude reads CLAUDE.md
    test_tool_reads_config "claude" \
        "repo-test/claude:latest" \
        "CLAUDE.md" \
        "config readable"

    # Test Aider reads .aider.conf.yml
    test_tool_reads_config "aider" \
        "repo-test/aider:latest" \
        ".aider.conf.yml" \
        "config readable"

    # Test Cursor reads .cursorrules
    test_tool_reads_config "cursor" \
        "repo-test/cursor:latest" \
        ".cursorrules" \
        "config readable"

    # Test Cline sees .vscode/ directory
    test_tool_sees_directory "cline" \
        "repo-test/cline:latest" \
        ".vscode"

    # Test Roo sees .vscode/ directory
    test_tool_sees_directory "roo" \
        "repo-test/roo:latest" \
        ".vscode"

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
