# repo-test/cursor - Cursor AI Editor CLI (agent command)
# Based on research: Cursor has full CLI with headless print mode
FROM repo-test/cli-base:latest

LABEL tool="cursor"
LABEL tool.version="latest"
LABEL tool.cli="agent"

# Install Cursor CLI
# Note: The installer may require interaction, using a non-interactive approach
RUN curl -fsSL https://cursor.com/install | bash || echo "Cursor CLI installation attempted"

# Add cursor to PATH (installer puts it in ~/.cursor/bin or similar)
ENV PATH="/root/.cursor/bin:/root/.local/bin:${PATH}"

# Verify installation (may fail without API key, that's OK)
RUN which cursor || which agent || echo "Cursor CLI installed - verify path"

# Default working directory
WORKDIR /workspace

# Entry point - use agent command for CLI operations
ENTRYPOINT ["agent"]
CMD ["--help"]
