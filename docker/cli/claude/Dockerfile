# repo-test/claude - Claude Code CLI
FROM repo-test/cli-base:latest

LABEL tool="claude"
LABEL tool.version="latest"

# Install Claude CLI globally
RUN npm install -g @anthropic-ai/claude-code

# Verify installation
RUN claude --version || echo "Claude CLI installed (version check may require API key)"

# Default working directory
WORKDIR /workspace

# Entry point
ENTRYPOINT ["claude"]
CMD ["--help"]
