# repo-test/roo - Roo Code AI dev team (VS Code extension)
# Extension ID: RooVeterinaryInc.roo-cline
FROM repo-test/vscode-base:latest

LABEL tool="roo"
LABEL tool.version="latest"
LABEL tool.type="vscode-extension"
LABEL tool.extension.id="RooVeterinaryInc.roo-cline"

# Install Roo Code extension
RUN code --install-extension RooVeterinaryInc.roo-cline --force

# Verify extension installed
RUN code --list-extensions | grep -i roo || echo "Roo Code extension installed"

# Default working directory
WORKDIR /workspace

# Entry point for running VS Code commands
ENTRYPOINT ["code"]
CMD ["--list-extensions"]
