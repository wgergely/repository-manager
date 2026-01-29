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

echo ">>> Building Cursor CLI image..."
docker build -f cli/cursor/Dockerfile -t repo-test/cursor:latest .

echo ">>> Building VS Code base image..."
docker build -f base/Dockerfile.vscode -t repo-test/vscode-base:latest .

echo ">>> Building Cline extension image..."
docker build -f vscode/cline/Dockerfile -t repo-test/cline:latest .

echo ">>> Building Roo Code extension image..."
docker build -f vscode/roo/Dockerfile -t repo-test/roo:latest .

echo "=== All images built successfully ==="
docker images | grep repo-test
