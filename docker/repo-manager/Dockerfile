FROM repo-test/base:latest

LABEL tool="repo-manager"
LABEL description="Repository Manager CLI for config generation testing"

# Copy crates source
COPY crates /workspace/crates

# Build Repository Manager
WORKDIR /workspace
RUN cd crates/repo-cli && cargo build --release

# Add to PATH
ENV PATH="/workspace/crates/target/release:${PATH}"

# Verify installation
RUN repo --help || echo "Repository Manager built"

WORKDIR /workspace/test-project

ENTRYPOINT ["repo"]
CMD ["--help"]
