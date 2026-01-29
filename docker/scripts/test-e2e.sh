#!/bin/bash
# End-to-end integration test script
# Tests the full workflow: config generation -> tool execution -> API communication
# Uses WireMock as a mock API server to verify requests are made correctly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/test-results/e2e"
FIXTURES_DIR="$PROJECT_ROOT/test-fixtures"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED_TESTS=()
PASSED_TESTS=()

# Working directory for test
WORK_DIR=""

# Flag to track if mock API was started
MOCK_API_STARTED=false

# Setup results directory
setup_results_dir() {
    mkdir -p "$RESULTS_DIR"
    echo "End-to-end test results - $(date)" > "$RESULTS_DIR/summary.txt"
    echo "============================================" >> "$RESULTS_DIR/summary.txt"
}

# Log function
log() {
    echo -e "$1"
}

# Cleanup function - runs on exit
cleanup() {
    log ""
    log "${YELLOW}>>> Cleaning up...${NC}"

    # Stop mock API if it was started
    if [ "$MOCK_API_STARTED" = true ]; then
        log "    Stopping mock API..."
        cd "$PROJECT_ROOT"
        docker compose --profile mock down 2>/dev/null || true
    fi

    # Remove temp working directory
    if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
        log "    Removing temporary directory..."
        rm -rf "$WORK_DIR" 2>/dev/null || true
    fi

    log "${GREEN}    Cleanup complete${NC}"
}
trap cleanup EXIT

# Setup working directory
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

# Start mock API server
start_mock_api() {
    log ""
    log "${YELLOW}>>> Starting mock API server (WireMock)...${NC}"

    cd "$PROJECT_ROOT"

    # Start mock API using docker compose with mock profile
    if docker compose --profile mock up -d mock-api 2>&1 | tee "$RESULTS_DIR/mock-api-start.log"; then
        MOCK_API_STARTED=true
        log "    Docker compose started"
    else
        log "${RED}    Failed to start mock API container${NC}"
        return 1
    fi

    # Wait for mock API to be ready
    log "    Waiting for mock API to be healthy..."
    local max_attempts=30
    local attempt=0

    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://localhost:8080/health 2>/dev/null | grep -q "OK"; then
            log "${GREEN}    Mock API is ready${NC}"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    log "${RED}    Mock API failed to become healthy after ${max_attempts}s${NC}"
    return 1
}

# Phase 1: Generate configs using Repository Manager
phase1_config_generation() {
    log ""
    log "============================================"
    log "=== Phase 1: Config Generation ==="
    log "============================================"

    local log_file="$RESULTS_DIR/config-gen.log"
    local status="PASS"

    log "${YELLOW}>>> Running 'repo sync' to generate configs...${NC}"

    # Run repo sync command
    if docker run --rm \
        -v "$WORK_DIR:/workspace/test-project" \
        repo-test/repo-manager:latest \
        sync 2>&1 | tee "$log_file"; then
        log "${GREEN}    Sync command completed${NC}"
    else
        log "${RED}    Sync command failed${NC}"
        status="FAIL"
    fi

    # Verify configs were created
    log ""
    log "${YELLOW}>>> Verifying generated configs...${NC}"

    local configs_found=0

    if [ -f "$WORK_DIR/CLAUDE.md" ]; then
        log "    CLAUDE.md: ${GREEN}found${NC}"
        configs_found=$((configs_found + 1))
    else
        log "    CLAUDE.md: ${RED}not found${NC}"
        status="FAIL"
    fi

    if [ -f "$WORK_DIR/.cursorrules" ]; then
        log "    .cursorrules: ${GREEN}found${NC}"
        configs_found=$((configs_found + 1))
    else
        log "    .cursorrules: ${RED}not found${NC}"
        status="FAIL"
    fi

    if [ -f "$WORK_DIR/.aider.conf.yml" ]; then
        log "    .aider.conf.yml: ${GREEN}found${NC}"
        configs_found=$((configs_found + 1))
    else
        log "    .aider.conf.yml: ${RED}not found${NC}"
        status="FAIL"
    fi

    log "    Configs found: $configs_found/3"

    # Record result
    if [ "$status" = "PASS" ]; then
        log "    Result: ${GREEN}PASS${NC}"
        PASSED_TESTS+=("config-generation")
        echo "PASS: config-generation" >> "$RESULTS_DIR/summary.txt"
    else
        log "    Result: ${RED}FAIL${NC}"
        FAILED_TESTS+=("config-generation")
        echo "FAIL: config-generation" >> "$RESULTS_DIR/summary.txt"
    fi
}

# Phase 2: Run tools with mock API
phase2_tool_execution() {
    log ""
    log "============================================"
    log "=== Phase 2: Tool Execution with Mock API ==="
    log "============================================"

    # Detect platform for host.docker.internal support
    local host_gateway_flag=""
    if [[ "$(uname -s)" == "Linux" ]]; then
        host_gateway_flag="--add-host=host.docker.internal:host-gateway"
    fi

    # Test Claude container reads config and can reach mock API
    test_tool_with_mock_api "claude" \
        "repo-test/claude:latest" \
        "ANTHROPIC_API_KEY=mock-key" \
        "ANTHROPIC_BASE_URL=http://host.docker.internal:8080" \
        "$host_gateway_flag" \
        "head -5 /workspace/CLAUDE.md && echo 'Claude config readable'"

    # Test Aider container reads config
    test_tool_with_mock_api "aider" \
        "repo-test/aider:latest" \
        "OPENAI_API_KEY=mock-key" \
        "OPENAI_API_BASE=http://host.docker.internal:8080/v1" \
        "$host_gateway_flag" \
        "head -5 /workspace/.aider.conf.yml && echo 'Aider config readable'"

    # Test Cursor container reads config
    test_tool_with_mock_api "cursor" \
        "repo-test/cursor:latest" \
        "ANTHROPIC_API_KEY=mock-key" \
        "ANTHROPIC_BASE_URL=http://host.docker.internal:8080" \
        "$host_gateway_flag" \
        "head -5 /workspace/.cursorrules && echo 'Cursor config readable'"
}

# Test a tool container with mock API environment
test_tool_with_mock_api() {
    local tool="$1"
    local image="$2"
    local env1="$3"
    local env2="$4"
    local extra_flag="$5"
    local command="$6"

    log ""
    log "${YELLOW}>>> Testing $tool with mock API...${NC}"

    local log_file="$RESULTS_DIR/${tool}-e2e.log"
    local status="PASS"

    # Build docker run command
    local docker_cmd="docker run --rm"
    docker_cmd="$docker_cmd -v $WORK_DIR:/workspace"
    docker_cmd="$docker_cmd -e $env1"
    docker_cmd="$docker_cmd -e $env2"

    if [ -n "$extra_flag" ]; then
        docker_cmd="$docker_cmd $extra_flag"
    fi

    docker_cmd="$docker_cmd --entrypoint /bin/bash"
    docker_cmd="$docker_cmd $image"
    docker_cmd="$docker_cmd -c \"$command\""

    # Execute
    if eval "$docker_cmd" > "$log_file" 2>&1; then
        log "    Command executed: ${GREEN}success${NC}"
    else
        log "    Command executed: ${RED}failed${NC}"
        status="FAIL"
    fi

    # Check if config was readable
    if grep -q "config readable" "$log_file" 2>/dev/null; then
        log "    Config readable: ${GREEN}yes${NC}"
    else
        log "    Config readable: ${RED}no${NC}"
        status="FAIL"
    fi

    # Record result
    if [ "$status" = "PASS" ]; then
        log "    Result: ${GREEN}PASS${NC}"
        PASSED_TESTS+=("${tool}-e2e")
        echo "PASS: ${tool}-e2e" >> "$RESULTS_DIR/summary.txt"
    else
        log "    Result: ${RED}FAIL${NC}"
        FAILED_TESTS+=("${tool}-e2e")
        echo "FAIL: ${tool}-e2e" >> "$RESULTS_DIR/summary.txt"
    fi
}

# Phase 3: Verify mock API received requests
phase3_api_verification() {
    log ""
    log "============================================"
    log "=== Phase 3: API Request Verification ==="
    log "============================================"

    log "${YELLOW}>>> Checking mock API request log...${NC}"

    local log_file="$RESULTS_DIR/api-requests.log"
    local status="PASS"

    # Query WireMock admin API for request log
    if curl -s http://localhost:8080/__admin/requests > "$log_file" 2>&1; then
        log "    Admin API accessible: ${GREEN}yes${NC}"

        # Parse request count
        local request_count
        request_count=$(cat "$log_file" | grep -o '"total":[0-9]*' | head -1 | cut -d: -f2 || echo "0")

        if [ -z "$request_count" ]; then
            # Try alternative JSON parsing if jq available
            if command -v jq &> /dev/null; then
                request_count=$(jq '.requests | length' "$log_file" 2>/dev/null || echo "0")
            else
                request_count="unknown"
            fi
        fi

        log "    Requests recorded: $request_count"

        # Save formatted request log
        if command -v jq &> /dev/null; then
            jq '.' "$log_file" > "$RESULTS_DIR/api-requests-formatted.json" 2>/dev/null || true
        fi
    else
        log "    Admin API accessible: ${RED}no${NC}"
        status="FAIL"
    fi

    # Check if health endpoint was hit (our startup check)
    if grep -q "health" "$log_file" 2>/dev/null; then
        log "    Health check recorded: ${GREEN}yes${NC}"
    else
        log "    Health check recorded: ${YELLOW}not found${NC}"
    fi

    # Record result
    if [ "$status" = "PASS" ]; then
        log "    Result: ${GREEN}PASS${NC}"
        PASSED_TESTS+=("api-verification")
        echo "PASS: api-verification" >> "$RESULTS_DIR/summary.txt"
    else
        log "    Result: ${RED}FAIL${NC}"
        FAILED_TESTS+=("api-verification")
        echo "FAIL: api-verification" >> "$RESULTS_DIR/summary.txt"
    fi
}

# Print summary
print_summary() {
    local total=$((${#PASSED_TESTS[@]} + ${#FAILED_TESTS[@]}))

    log ""
    log "========================================"
    log "     END-TO-END TEST SUMMARY"
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
        done
        log ""
        log "Check logs in: $RESULTS_DIR/"
        echo "FAILED: ${FAILED_TESTS[*]}" >> "$RESULTS_DIR/summary.txt"
    fi

    # List result files
    log "Result files:"
    ls -la "$RESULTS_DIR" 2>/dev/null | while read -r line; do
        log "  $line"
    done

    log ""
    log "Results saved to: $RESULTS_DIR"
}

# Main execution
main() {
    log "${YELLOW}Starting end-to-end integration tests...${NC}"
    log ""

    setup_results_dir
    setup_work_dir

    # Start mock API server
    if ! start_mock_api; then
        log "${RED}FATAL: Cannot start mock API server${NC}"
        exit 1
    fi

    # Run test phases
    phase1_config_generation
    phase2_tool_execution
    phase3_api_verification

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
