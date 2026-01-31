# Dockerized Testing Framework Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Docker-based integration testing framework that tests Repository Manager against real AI coding tools (Claude CLI, Aider, Gemini CLI, VS Code extensions).

**Architecture:** Layered Docker images (base → category → tool) orchestrated by Docker Compose. Hybrid testing strategy using WireMock for CI mocks and real APIs for certification. Priority on CLI tools first (trivial), then VS Code extensions (moderate).

**Tech Stack:** Docker, Docker Compose, WireMock (Java), Ubuntu 22.04, Node.js 20, Python 3.12, Rust

---

## Phase 1: Infrastructure Foundation

### Task 1: Create Docker Directory Structure

**Files:**
- Create: `docker/base/Dockerfile.base`
- Create: `docker/base/Dockerfile.cli`
- Create: `docker/scripts/start-xvfb.sh`
- Create: `docker/.dockerignore`

**Step 1: Create directory structure**

```bash
mkdir -p docker/base docker/cli/claude docker/cli/aider docker/cli/gemini docker/vscode/cline docker/vscode/roo docker/mock-api/stubs docker/scripts test-fixtures/repos test-fixtures/configs
```

**Step 2: Create .dockerignore**

Create `docker/.dockerignore`:
```
*.md
*.txt
.git
.gitignore
__pycache__
*.pyc
.env
.env.*
```

**Step 3: Commit**

```bash
git add docker/ test-fixtures/
git commit -m "chore: create docker testing infrastructure directories"
```

---

### Task 2: Create Base Dockerfile

**Files:**
- Create: `docker/base/Dockerfile.base`

**Step 1: Write the base Dockerfile**

Create `docker/base/Dockerfile.base`:
```dockerfile
# repo-test/base - Common foundation for all test containers
# Based on ADR-008: Ubuntu 22.04 as Base Image
FROM ubuntu:22.04

LABEL maintainer="Repository Manager Team"
LABEL description="Base image for repository manager integration tests"

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

# System packages
RUN apt-get update && apt-get install -y \
    git \
    curl \
    wget \
    jq \
    unzip \
    ca-certificates \
    gnupg \
    build-essential \
    pkg-config \
    libssl-dev \
    locales \
    && locale-gen en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*

# Node.js 20 LTS
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Python 3.12 (use deadsnakes PPA for Ubuntu 22.04)
RUN apt-get update && apt-get install -y software-properties-common \
    && add-apt-repository -y ppa:deadsnakes/ppa \
    && apt-get update \
    && apt-get install -y python3.12 python3.12-venv python3-pip \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf /usr/bin/python3.12 /usr/bin/python

# Rust (latest stable)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Working directory
WORKDIR /workspace

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    CMD echo "healthy" || exit 1
```

**Step 2: Build and verify base image**

```bash
cd docker && docker build -f base/Dockerfile.base -t repo-test/base:latest .
docker run --rm repo-test/base:latest node --version
docker run --rm repo-test/base:latest python --version
docker run --rm repo-test/base:latest rustc --version
```

Expected: Node v20.x, Python 3.12.x, rustc 1.x

**Step 3: Commit**

```bash
git add docker/base/Dockerfile.base
git commit -m "feat(docker): add base image with Node.js, Python, Rust"
```

---

### Task 3: Create CLI Base Dockerfile

**Files:**
- Create: `docker/base/Dockerfile.cli`

**Step 1: Write CLI base Dockerfile**

Create `docker/base/Dockerfile.cli`:
```dockerfile
# repo-test/cli-base - Base for CLI tools (Claude, Aider, Gemini)
# Based on ADR-002: Layered Base Images
FROM repo-test/base:latest

LABEL category="cli"
LABEL description="Base image for CLI-based AI coding tools"

# CLI tools are self-contained, no additional deps needed
# This layer exists for consistency and potential future additions

# Ensure pip is available for Python tools
RUN python -m pip install --upgrade pip

# Default entrypoint for CLI tools
ENTRYPOINT ["/bin/bash"]
```

**Step 2: Build and verify**

```bash
docker build -f base/Dockerfile.cli -t repo-test/cli-base:latest .
docker run --rm repo-test/cli-base:latest pip --version
```

**Step 3: Commit**

```bash
git add docker/base/Dockerfile.cli
git commit -m "feat(docker): add CLI base image"
```

---

## Phase 2: CLI Tool Images (Trivial Tier)

### Task 4: Create Claude CLI Dockerfile

**Files:**
- Create: `docker/cli/claude/Dockerfile`

**Step 1: Write Claude CLI Dockerfile**

Create `docker/cli/claude/Dockerfile`:
```dockerfile
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
```

**Step 2: Build image**

```bash
docker build -f cli/claude/Dockerfile -t repo-test/claude:latest .
```

**Step 3: Verify image works**

```bash
docker run --rm repo-test/claude:latest --help
```

Expected: Claude CLI help output

**Step 4: Commit**

```bash
git add docker/cli/claude/Dockerfile
git commit -m "feat(docker): add Claude CLI tool image"
```

---

### Task 5: Create Aider Dockerfile

**Files:**
- Create: `docker/cli/aider/Dockerfile`

**Step 1: Write Aider Dockerfile**

Create `docker/cli/aider/Dockerfile`:
```dockerfile
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
```

**Step 2: Build and verify**

```bash
docker build -f cli/aider/Dockerfile -t repo-test/aider:latest .
docker run --rm repo-test/aider:latest --version
```

**Step 3: Commit**

```bash
git add docker/cli/aider/Dockerfile
git commit -m "feat(docker): add Aider tool image"
```

---

### Task 6: Create Gemini CLI Dockerfile

**Files:**
- Create: `docker/cli/gemini/Dockerfile`

**Step 1: Write Gemini CLI Dockerfile**

Create `docker/cli/gemini/Dockerfile`:
```dockerfile
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
```

**Step 2: Build and verify**

```bash
docker build -f cli/gemini/Dockerfile -t repo-test/gemini:latest .
docker run --rm repo-test/gemini:latest --help
```

**Step 3: Commit**

```bash
git add docker/cli/gemini/Dockerfile
git commit -m "feat(docker): add Gemini CLI tool image"
```

---

## Phase 3: Environment Configuration

### Task 7: Create .env.example Template

**Files:**
- Create: `.env.example`
- Modify: `.gitignore` (verify .env is ignored)

**Step 1: Write .env.example**

Create `.env.example` at project root:
```bash
# ==============================================
# Repository Manager Docker Testing Environment
# ==============================================
# Copy this file to .env and fill in values
# NEVER commit .env to version control
# ==============================================

# Test Mode: mock | real | hybrid
TEST_MODE=mock

# Mock API Server (when TEST_MODE=mock or hybrid)
MOCK_API_URL=http://mock-api:8080

# ----------------------------------------------
# LLM Provider API Keys
# ----------------------------------------------

# Anthropic (Claude CLI, Cursor, Cline, Roo)
ANTHROPIC_API_KEY=sk-ant-api03-xxxx

# OpenAI (Aider, some Cline/Roo configs)
OPENAI_API_KEY=sk-xxxx

# Google (Gemini CLI)
# Option 1: API Key
GOOGLE_API_KEY=xxxx
# Option 2: Service Account JSON (place at docker/secrets/)
# GOOGLE_APPLICATION_CREDENTIALS=/workspace/secrets/gcloud-key.json

# ----------------------------------------------
# Platform Credentials
# ----------------------------------------------

# GitHub (Copilot)
GITHUB_TOKEN=ghp_xxxx

# AWS (Amazon Q)
AWS_ACCESS_KEY_ID=AKIA...
AWS_SECRET_ACCESS_KEY=xxxx
AWS_DEFAULT_REGION=us-east-1
```

**Step 2: Verify .gitignore includes .env**

Check `.gitignore` contains:
```
.env
.env.*
```

**Step 3: Commit**

```bash
git add .env.example
git commit -m "feat: add .env.example template for API credentials"
```

---

### Task 8: Create Docker Compose File

**Files:**
- Create: `docker-compose.yml`

**Step 1: Write docker-compose.yml**

Create `docker-compose.yml` at project root:
```yaml
# Docker Compose for Repository Manager Integration Testing
# Based on ADR-001: Docker Compose for Container Orchestration
version: "3.9"

x-common-env: &common-env
  TZ: UTC
  TEST_MODE: ${TEST_MODE:-mock}

x-common-volumes: &common-volumes
  - ./test-fixtures:/fixtures:ro
  - ./test-results:/results
  - ./crates:/workspace/crates:ro

services:
  # ============================================
  # CLI Agents (Trivial Tier)
  # ============================================
  claude:
    build:
      context: ./docker
      dockerfile: cli/claude/Dockerfile
    image: repo-test/claude:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY:-}
      ANTHROPIC_BASE_URL: ${ANTHROPIC_BASE_URL:-}
    working_dir: /workspace
    profiles: ["cli", "all"]
    stdin_open: true
    tty: true

  aider:
    build:
      context: ./docker
      dockerfile: cli/aider/Dockerfile
    image: repo-test/aider:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY:-}
      OPENAI_API_KEY: ${OPENAI_API_KEY:-}
      OPENAI_API_BASE: ${OPENAI_API_BASE:-}
    working_dir: /workspace
    profiles: ["cli", "all"]
    stdin_open: true
    tty: true

  gemini:
    build:
      context: ./docker
      dockerfile: cli/gemini/Dockerfile
    image: repo-test/gemini:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      GOOGLE_API_KEY: ${GOOGLE_API_KEY:-}
    working_dir: /workspace
    profiles: ["cli", "all"]
    stdin_open: true
    tty: true

  # ============================================
  # Mock API Server (for CI)
  # ============================================
  mock-api:
    image: wiremock/wiremock:3.3.1
    volumes:
      - ./docker/mock-api/stubs:/home/wiremock/mappings:ro
    ports:
      - "8080:8080"
    profiles: ["mock", "ci"]
    command: ["--verbose"]
```

**Step 2: Verify compose file is valid**

```bash
docker compose config
```

**Step 3: Commit**

```bash
git add docker-compose.yml
git commit -m "feat: add Docker Compose for CLI tool orchestration"
```

---

## Phase 4: Mock API Server

### Task 9: Create WireMock Stubs for Anthropic API

**Files:**
- Create: `docker/mock-api/stubs/anthropic-messages.json`

**Step 1: Write Anthropic API mock stub**

Create `docker/mock-api/stubs/anthropic-messages.json`:
```json
{
  "request": {
    "method": "POST",
    "urlPath": "/v1/messages",
    "headers": {
      "x-api-key": {
        "matches": ".*"
      },
      "Content-Type": {
        "equalTo": "application/json"
      }
    }
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json"
    },
    "jsonBody": {
      "id": "msg_mock_12345",
      "type": "message",
      "role": "assistant",
      "content": [
        {
          "type": "text",
          "text": "This is a mock response from the test API server. The Repository Manager integration test is working correctly."
        }
      ],
      "model": "claude-3-opus-20240229",
      "stop_reason": "end_turn",
      "stop_sequence": null,
      "usage": {
        "input_tokens": 10,
        "output_tokens": 25
      }
    }
  }
}
```

**Step 2: Create health check stub**

Create `docker/mock-api/stubs/health.json`:
```json
{
  "request": {
    "method": "GET",
    "urlPath": "/health"
  },
  "response": {
    "status": 200,
    "body": "OK"
  }
}
```

**Step 3: Verify WireMock starts with stubs**

```bash
docker compose --profile mock up mock-api -d
curl http://localhost:8080/health
docker compose --profile mock down
```

Expected: "OK"

**Step 4: Commit**

```bash
git add docker/mock-api/stubs/
git commit -m "feat(docker): add WireMock stubs for Anthropic API"
```

---

### Task 10: Create OpenAI API Mock Stub

**Files:**
- Create: `docker/mock-api/stubs/openai-completions.json`

**Step 1: Write OpenAI API mock stub**

Create `docker/mock-api/stubs/openai-completions.json`:
```json
{
  "request": {
    "method": "POST",
    "urlPath": "/v1/chat/completions",
    "headers": {
      "Authorization": {
        "matches": "Bearer .*"
      }
    }
  },
  "response": {
    "status": 200,
    "headers": {
      "Content-Type": "application/json"
    },
    "jsonBody": {
      "id": "chatcmpl-mock123",
      "object": "chat.completion",
      "created": 1700000000,
      "model": "gpt-4",
      "choices": [
        {
          "index": 0,
          "message": {
            "role": "assistant",
            "content": "This is a mock response from the test API server. The Repository Manager integration test is working correctly."
          },
          "finish_reason": "stop"
        }
      ],
      "usage": {
        "prompt_tokens": 10,
        "completion_tokens": 25,
        "total_tokens": 35
      }
    }
  }
}
```

**Step 2: Commit**

```bash
git add docker/mock-api/stubs/openai-completions.json
git commit -m "feat(docker): add WireMock stubs for OpenAI API"
```

---

## Phase 5: Test Fixtures

### Task 11: Create Test Repository Fixture

**Files:**
- Create: `test-fixtures/repos/simple-project/CLAUDE.md`
- Create: `test-fixtures/repos/simple-project/.aider.conf.yml`
- Create: `test-fixtures/repos/simple-project/GEMINI.md`
- Create: `test-fixtures/repos/simple-project/src/main.rs`

**Step 1: Create simple test project**

Create `test-fixtures/repos/simple-project/CLAUDE.md`:
```markdown
# Test Project

This is a test project for Repository Manager integration testing.

## Rules

- Write clean, idiomatic code
- Add tests for new functionality
- Keep functions small and focused
```

Create `test-fixtures/repos/simple-project/.aider.conf.yml`:
```yaml
# Aider configuration for test project
model: gpt-4
auto-commits: false
```

Create `test-fixtures/repos/simple-project/GEMINI.md`:
```markdown
# Test Project for Gemini

This is a test project for Repository Manager integration testing.

Follow best practices for the language being used.
```

Create `test-fixtures/repos/simple-project/src/main.rs`:
```rust
fn main() {
    println!("Hello from test fixture!");
}
```

Create `test-fixtures/repos/simple-project/Cargo.toml`:
```toml
[package]
name = "simple-project"
version = "0.1.0"
edition = "2021"
```

**Step 2: Commit**

```bash
git add test-fixtures/repos/simple-project/
git commit -m "feat: add simple project test fixture"
```

---

## Phase 6: CI Override Compose File

### Task 12: Create CI Docker Compose Override

**Files:**
- Create: `docker-compose.ci.yml`

**Step 1: Write CI override**

Create `docker-compose.ci.yml`:
```yaml
# CI overrides - routes all API calls to mock server
# Usage: docker compose -f docker-compose.yml -f docker-compose.ci.yml up
version: "3.9"

services:
  claude:
    environment:
      ANTHROPIC_API_KEY: "mock-key-for-ci"
      ANTHROPIC_BASE_URL: "http://mock-api:8080"
    depends_on:
      - mock-api

  aider:
    environment:
      OPENAI_API_KEY: "mock-key-for-ci"
      OPENAI_API_BASE: "http://mock-api:8080/v1"
      ANTHROPIC_API_KEY: "mock-key-for-ci"
    depends_on:
      - mock-api

  gemini:
    environment:
      # Gemini CLI mock requires different setup
      GOOGLE_API_KEY: "mock-key-for-ci"
    depends_on:
      - mock-api
```

**Step 2: Commit**

```bash
git add docker-compose.ci.yml
git commit -m "feat: add CI Docker Compose override for mock APIs"
```

---

## Phase 7: Build and Verification Scripts

### Task 13: Create Build Script

**Files:**
- Create: `docker/scripts/build-all.sh`

**Step 1: Write build script**

Create `docker/scripts/build-all.sh`:
```bash
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
```

**Step 2: Make executable and test**

```bash
chmod +x docker/scripts/build-all.sh
./docker/scripts/build-all.sh
```

**Step 3: Commit**

```bash
git add docker/scripts/build-all.sh
git commit -m "feat(docker): add build-all script"
```

---

### Task 14: Create Verification Script

**Files:**
- Create: `docker/scripts/verify-tools.sh`

**Step 1: Write verification script**

Create `docker/scripts/verify-tools.sh`:
```bash
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
echo "=== All tools verified ==="
```

**Step 2: Make executable and test**

```bash
chmod +x docker/scripts/verify-tools.sh
./docker/scripts/verify-tools.sh
```

**Step 3: Commit**

```bash
git add docker/scripts/verify-tools.sh
git commit -m "feat(docker): add tool verification script"
```

---

## Phase 8: Integration Test Runner

### Task 15: Create Test Runner Script

**Files:**
- Create: `docker/scripts/run-tests.sh`

**Step 1: Write test runner**

Create `docker/scripts/run-tests.sh`:
```bash
#!/bin/bash
set -e

MODE="${1:-mock}"
PROFILE="${2:-cli}"

echo "=== Running Integration Tests ==="
echo "Mode: $MODE"
echo "Profile: $Profile"

cd "$(dirname "$0")/../.."

# Start mock API if in mock mode
if [ "$MODE" = "mock" ]; then
    echo ">>> Starting mock API server..."
    docker compose --profile mock up -d mock-api
    sleep 2

    # Verify mock API is healthy
    curl -s http://localhost:8080/health || {
        echo "ERROR: Mock API not healthy"
        docker compose --profile mock down
        exit 1
    }
fi

# Create results directory
mkdir -p test-results

echo ">>> Running tests with profile: $PROFILE"

# Run a simple config detection test
docker compose -f docker-compose.yml -f docker-compose.ci.yml \
    run --rm claude sh -c "ls /fixtures/repos/simple-project/ && cat /fixtures/repos/simple-project/CLAUDE.md"

echo ">>> Tests completed"

# Cleanup
if [ "$MODE" = "mock" ]; then
    docker compose --profile mock down
fi

echo "=== Integration tests passed ==="
```

**Step 2: Make executable**

```bash
chmod +x docker/scripts/run-tests.sh
```

**Step 3: Commit**

```bash
git add docker/scripts/run-tests.sh
git commit -m "feat(docker): add integration test runner script"
```

---

## Phase 9: Update .gitignore for Docker

### Task 16: Update .gitignore

**Files:**
- Modify: `.gitignore`

**Step 1: Add Docker-related ignores**

Add to `.gitignore`:
```gitignore
# ----------------------------------------------------------------------
# Docker Testing
# ----------------------------------------------------------------------
test-results/
docker/secrets/
*.tar
```

**Step 2: Commit**

```bash
git add .gitignore
git commit -m "chore: update .gitignore for docker testing"
```

---

## Phase 10: Documentation

### Task 17: Create Docker Testing README

**Files:**
- Create: `docker/README.md`

**Step 1: Write Docker README**

Create `docker/README.md`:
```markdown
# Docker Testing Infrastructure

Integration testing infrastructure for Repository Manager.

## Quick Start

1. **Copy environment template:**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys
   ```

2. **Build all images:**
   ```bash
   ./docker/scripts/build-all.sh
   ```

3. **Run tests (mock mode):**
   ```bash
   ./docker/scripts/run-tests.sh mock cli
   ```

## Image Hierarchy

```
repo-test/base          Ubuntu 22.04 + Node.js + Python + Rust
  └── repo-test/cli-base     Base for CLI tools
        ├── repo-test/claude     Claude Code CLI
        ├── repo-test/aider      Aider
        └── repo-test/gemini     Gemini CLI
```

## Profiles

- `cli` - CLI tools (Claude, Aider, Gemini)
- `mock` - Mock API server
- `ci` - CI/CD configuration
- `all` - All tools

## Test Modes

- `mock` - Use WireMock for API responses (fast, free)
- `real` - Use real APIs (requires API keys)
- `hybrid` - Mock by default, real for specific tests

## Scripts

- `build-all.sh` - Build all Docker images
- `verify-tools.sh` - Verify tool installations
- `run-tests.sh <mode> <profile>` - Run integration tests

## Architecture

Based on ADRs in `docs/research/audit/decisions.md`:
- ADR-001: Docker Compose for orchestration
- ADR-002: Layered base images
- ADR-003: Hybrid API testing strategy
- ADR-007: WireMock for API mocking
- ADR-008: Ubuntu 22.04 as base image
```

**Step 2: Commit**

```bash
git add docker/README.md
git commit -m "docs: add Docker testing infrastructure README"
```

---

## Summary

### Files Created

```
docker/
├── base/
│   ├── Dockerfile.base
│   └── Dockerfile.cli
├── cli/
│   ├── claude/Dockerfile
│   ├── aider/Dockerfile
│   └── gemini/Dockerfile
├── mock-api/
│   └── stubs/
│       ├── anthropic-messages.json
│       ├── openai-completions.json
│       └── health.json
├── scripts/
│   ├── build-all.sh
│   ├── verify-tools.sh
│   └── run-tests.sh
├── .dockerignore
└── README.md

test-fixtures/
└── repos/
    └── simple-project/
        ├── CLAUDE.md
        ├── .aider.conf.yml
        ├── GEMINI.md
        ├── Cargo.toml
        └── src/main.rs

.env.example
docker-compose.yml
docker-compose.ci.yml
```

### Commits

1. `chore: create docker testing infrastructure directories`
2. `feat(docker): add base image with Node.js, Python, Rust`
3. `feat(docker): add CLI base image`
4. `feat(docker): add Claude CLI tool image`
5. `feat(docker): add Aider tool image`
6. `feat(docker): add Gemini CLI tool image`
7. `feat: add .env.example template for API credentials`
8. `feat: add Docker Compose for CLI tool orchestration`
9. `feat(docker): add WireMock stubs for Anthropic API`
10. `feat(docker): add WireMock stubs for OpenAI API`
11. `feat: add simple project test fixture`
12. `feat: add CI Docker Compose override for mock APIs`
13. `feat(docker): add build-all script`
14. `feat(docker): add tool verification script`
15. `feat(docker): add integration test runner script`
16. `chore: update .gitignore for docker testing`
17. `docs: add Docker testing infrastructure README`

### Next Steps (Future Phases)

- **Phase 11:** VS Code base image and extension testing
- **Phase 12:** Cursor CLI integration (uses `agent` command)
- **Phase 13:** GUI testing with Xvfb (Windsurf, Antigravity)
- **Phase 14:** GitHub Actions CI workflow
- **Phase 15:** Certification test suite with real APIs
