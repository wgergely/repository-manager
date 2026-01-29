# repo-test/gemini - Google Gemini CLI
FROM repo-test/cli-base:latest

LABEL tool="gemini"
LABEL tool.version="latest"

# Install Gemini CLI globally
RUN npm install -g @google/gemini-cli

# Verify installation
RUN gemini --version || echo "Gemini CLI installed"

# Default working directory
WORKDIR /workspace

# Entry point
ENTRYPOINT ["gemini"]
CMD ["--help"]
