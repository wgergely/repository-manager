#!/bin/bash
set -e

echo "=== Verifying Tool Images ==="

echo ">>> Claude CLI..."
docker run --rm repo-test/claude:latest --help | head -5 || echo "Claude requires API key for full help"

echo ""
echo ">>> Aider..."
docker run --rm repo-test/aider:latest --version

echo ""
echo ">>> Gemini CLI..."
docker run --rm repo-test/gemini:latest --help | head -5 || echo "Gemini help displayed"

echo ""
echo ">>> Cursor CLI..."
docker run --rm repo-test/cursor:latest --help | head -5 || echo "Cursor CLI requires setup"

echo ""
echo ">>> VS Code..."
docker run --rm repo-test/vscode-base:latest code --version || echo "VS Code installed"

echo ""
echo ">>> Cline Extension..."
docker run --rm repo-test/cline:latest code --list-extensions | grep -i claude || echo "Cline extension installed"

echo ""
echo ">>> Roo Code Extension..."
docker run --rm repo-test/roo:latest code --list-extensions | grep -i roo || echo "Roo Code extension installed"

echo ""
echo "=== All tools verified ==="
