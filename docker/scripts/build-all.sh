#!/bin/bash
set -e

echo "=== Building Repository Manager Test Images ==="

cd "$(dirname "$0")/.."

echo ">>> Building base image..."
docker build -f base/Dockerfile.base -t repo-test/base:latest .

echo ">>> Building CLI base image..."
docker build -f base/Dockerfile.cli -t repo-test/cli-base:latest .

echo ">>> Building Claude CLI image..."
docker build -f cli/claude/Dockerfile -t repo-test/claude:latest .

echo ">>> Building Aider image..."
docker build -f cli/aider/Dockerfile -t repo-test/aider:latest .

echo ">>> Building Gemini CLI image..."
docker build -f cli/gemini/Dockerfile -t repo-test/gemini:latest .

echo "=== All images built successfully ==="
docker images | grep repo-test
