# repo-test/cline - Cline autonomous coding agent (VS Code extension)
# Extension ID: saoudrizwan.claude-dev
FROM repo-test/vscode-base:latest

LABEL tool="cline"
LABEL tool.version="latest"
LABEL tool.type="vscode-extension"
LABEL tool.extension.id="saoudrizwan.claude-dev"

# Install Cline extension
RUN code --install-extension saoudrizwan.claude-dev --force

# Verify extension installed
RUN code --list-extensions | grep -i claude-dev || echo "Cline extension installed"

# Default working directory
WORKDIR /workspace

# Entry point for running VS Code commands
ENTRYPOINT ["code"]
CMD ["--list-extensions"]
