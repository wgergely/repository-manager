#!/usr/bin/env bash
#
# Run tool integration tests for repo-tools crate
#
# Usage:
#   ./scripts/run-tool-integration-tests.sh           # Run all tests
#   ./scripts/run-tool-integration-tests.sh --update  # Update insta snapshots
#   ./scripts/run-tool-integration-tests.sh --review  # Review pending snapshots
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Parse arguments
UPDATE_SNAPSHOTS=false
REVIEW_SNAPSHOTS=false
FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --update|-u)
            UPDATE_SNAPSHOTS=true
            shift
            ;;
        --review|-r)
            REVIEW_SNAPSHOTS=true
            shift
            ;;
        --filter|-f)
            FILTER="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--update|-u] [--review|-r] [--filter|-f <pattern>]"
            exit 1
            ;;
    esac
done

echo "Running repo-tools integration tests..."
echo "========================================"

# Build test command
TEST_CMD="cargo test -p repo-tools --test integration_tests"

if [ -n "$FILTER" ]; then
    TEST_CMD="$TEST_CMD -- $FILTER"
fi

# Run tests with appropriate insta settings
if [ "$UPDATE_SNAPSHOTS" = true ]; then
    echo "Mode: Update snapshots"
    INSTA_UPDATE=always $TEST_CMD
elif [ "$REVIEW_SNAPSHOTS" = true ]; then
    echo "Mode: Review snapshots"
    cargo insta review -p repo-tools
else
    echo "Mode: Normal test run"
    $TEST_CMD
fi

echo ""
echo "Done!"
