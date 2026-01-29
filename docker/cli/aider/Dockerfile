# repo-test/aider - Aider AI pair programming
FROM repo-test/cli-base:latest

LABEL tool="aider"
LABEL tool.version="latest"

# Install Aider
RUN pip install aider-chat

# Verify installation
RUN aider --version

# Default working directory
WORKDIR /workspace

# Entry point
ENTRYPOINT ["aider"]
CMD ["--help"]
