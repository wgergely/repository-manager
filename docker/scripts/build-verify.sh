#!/bin/bash
# Build verification script with logging
# Builds all Docker images in dependency order and captures logs

set -e

# Setup directories
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCKER_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="${DOCKER_DIR}/test-results/builds"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Track builds
FAILED_BUILDS=()
TOTAL_BUILDS=0

# Color output (if terminal supports it)
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Build a single image with logging
# Args: $1=name, $2=dockerfile, $3=tag, $4=context (optional, defaults to DOCKER_DIR)
build_image() {
    local name="$1"
    local dockerfile="$2"
    local tag="$3"
    local context="${4:-$DOCKER_DIR}"
    local log_file="${RESULTS_DIR}/${name}.log"

    echo -n "Building ${name}... "
    ((TOTAL_BUILDS++)) || true

    if docker build -f "${DOCKER_DIR}/${dockerfile}" -t "${tag}" "${context}" > "$log_file" 2>&1; then
        echo -e "${GREEN}✓ SUCCESS${NC}"
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        FAILED_BUILDS+=("$name")
        return 1
    fi
}

echo "=== Docker Build Verification ==="
echo "Log directory: ${RESULTS_DIR}"
echo ""

# Phase 1: Base images (no dependencies)
echo "--- Phase 1: Base Images ---"
build_image "base" "base/Dockerfile.base" "repo-test/base:latest" || true
build_image "cli-base" "base/Dockerfile.cli" "repo-test/cli-base:latest" || true
build_image "vscode-base" "base/Dockerfile.vscode" "repo-test/vscode-base:latest" || true
echo ""

# Phase 2: CLI tools (depend on cli-base)
echo "--- Phase 2: CLI Tools ---"
build_image "claude" "cli/claude/Dockerfile" "repo-test/claude:latest" || true
build_image "aider" "cli/aider/Dockerfile" "repo-test/aider:latest" || true
build_image "gemini" "cli/gemini/Dockerfile" "repo-test/gemini:latest" || true
build_image "cursor" "cli/cursor/Dockerfile" "repo-test/cursor:latest" || true
echo ""

# Phase 3: VS Code extensions (depend on vscode-base)
echo "--- Phase 3: VS Code Extensions ---"
build_image "cline" "vscode/cline/Dockerfile" "repo-test/cline:latest" || true
build_image "roo" "vscode/roo/Dockerfile" "repo-test/roo:latest" || true
echo ""

# Phase 4: Repository Manager (needs project root as context for crates/)
PROJECT_ROOT="$(dirname "$DOCKER_DIR")"
echo "--- Phase 4: Repository Manager ---"
build_image "repo-manager" "repo-manager/Dockerfile" "repo-test/repo-manager:latest" "$PROJECT_ROOT" || true
echo ""

# Summary
echo "=== Build Summary ==="
failed_count=${#FAILED_BUILDS[@]}
passed_count=$((TOTAL_BUILDS - failed_count))

echo "Passed: ${passed_count}/${TOTAL_BUILDS}"
echo "Failed: ${failed_count}/${TOTAL_BUILDS}"

if [ ${#FAILED_BUILDS[@]} -gt 0 ]; then
    echo ""
    echo "Failed builds:"
    for build in "${FAILED_BUILDS[@]}"; do
        echo "  - ${build} (see ${RESULTS_DIR}/${build}.log)"
    done
    exit 1
fi

echo ""
echo "All builds completed successfully!"
exit 0
