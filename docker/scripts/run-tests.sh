#!/bin/bash
set -e

MODE="${1:-mock}"
PROFILE="${2:-cli}"

echo "=== Running Integration Tests ==="
echo "Mode: $MODE"
echo "Profile: $PROFILE"

cd "$(dirname "$0")/../.."

# Start mock API if in mock mode
if [ "$MODE" = "mock" ]; then
    echo ">>> Starting mock API server..."
    docker compose --profile mock up -d mock-api
    sleep 2

    # Verify mock API is healthy
    curl -s http://localhost:8080/health || {
        echo "ERROR: Mock API not healthy"
        docker compose --profile mock down
        exit 1
    }
fi

# Create results directory
mkdir -p test-results

echo ">>> Running tests with profile: $PROFILE"

# Run a simple config detection test
docker compose -f docker-compose.yml -f docker-compose.ci.yml \
    run --rm claude sh -c "ls /fixtures/repos/simple-project/ && cat /fixtures/repos/simple-project/CLAUDE.md"

echo ">>> Tests completed"

# Cleanup
if [ "$MODE" = "mock" ]; then
    docker compose --profile mock down
fi

echo "=== Integration tests passed ==="
