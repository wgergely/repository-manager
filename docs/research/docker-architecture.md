# Docker Architecture Research

> **Purpose:** Define container architecture for the testing infrastructure.
> **Last Updated:** 2026-01-29
> **Status:** Research Draft

## Overview

This document defines the Docker architecture for testing Repository Manager against real tools. The design uses a **layered base image** strategy with **Docker Compose** for orchestration.

## Design Principles

1. **Minimize image size** - Shared base layers reduce storage and build time
2. **Isolate tool concerns** - Each tool gets its own container
3. **Reproducible builds** - Pin versions where possible, document snapshots where not
4. **Fast iteration** - Layer caching should make rebuilds quick
5. **CI-friendly** - Works in GitHub Actions, GitLab CI, etc.

---

## Image Hierarchy

```
┌─────────────────────────────────────────────────────────────────┐
│                      Tool Images (leaf)                          │
│  repo-test/claude  repo-test/aider  repo-test/cursor  ...       │
├─────────────────────────────────────────────────────────────────┤
│                     Category Images (mid)                        │
│  repo-test/cli-base    repo-test/vscode-base                    │
│  repo-test/gui-base    repo-test/jetbrains-base                 │
├─────────────────────────────────────────────────────────────────┤
│                      Base Image (root)                           │
│                    repo-test/base                                │
│              Ubuntu 22.04 + common dependencies                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Base Image: `repo-test/base`

**Purpose:** Common foundation for all test containers.

**Base:** `ubuntu:22.04`

**Included:**
- System utilities: `git`, `curl`, `wget`, `jq`, `unzip`, `ca-certificates`
- Language runtimes: Node.js 20 LTS, Python 3.12, Rust (latest stable)
- Build essentials: `build-essential`, `pkg-config`, `libssl-dev`
- Locales: `en_US.UTF-8`

```dockerfile
# docker/base/Dockerfile.base
FROM ubuntu:22.04

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

# Python 3.12
RUN apt-get update && apt-get install -y \
    python3.12 \
    python3.12-venv \
    python3-pip \
    && rm -rf /var/lib/apt/lists/* \
    && ln -sf /usr/bin/python3.12 /usr/bin/python

# Rust (latest stable)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Working directory
WORKDIR /workspace

# Health check placeholder
HEALTHCHECK --interval=30s --timeout=3s \
    CMD echo "healthy" || exit 1
```

**Estimated Size:** ~1.5GB (uncompressed)

---

## Category Image: `repo-test/cli-base`

**Purpose:** Base for CLI tools (Claude, Aider, Gemini).

**Inherits:** `repo-test/base`

**Additional:** Nothing - CLI tools are self-contained.

```dockerfile
# docker/base/Dockerfile.cli
FROM repo-test/base

# CLI tools need no additional system deps
# This layer exists for consistency and potential future additions

LABEL category="cli"
```

**Estimated Size:** Same as base (~1.5GB)

---

## Category Image: `repo-test/vscode-base`

**Purpose:** Base for VS Code and its extensions (Cline, Roo, Copilot, Amazon Q).

**Inherits:** `repo-test/base`

**Additional:**
- VS Code (stable)
- VS Code CLI tools
- Libraries for headless operation

```dockerfile
# docker/base/Dockerfile.vscode
FROM repo-test/base

# VS Code dependencies
RUN apt-get update && apt-get install -y \
    libasound2 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libxcomposite1 \
    libxdamage1 \
    libxfixes3 \
    libxkbcommon0 \
    libxrandr2 \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Install VS Code
RUN wget -qO- https://packages.microsoft.com/keys/microsoft.asc | gpg --dearmor > /usr/share/keyrings/packages.microsoft.gpg \
    && echo "deb [arch=amd64 signed-by=/usr/share/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main" > /etc/apt/sources.list.d/vscode.list \
    && apt-get update \
    && apt-get install -y code \
    && rm -rf /var/lib/apt/lists/*

# Configure for headless
ENV DISPLAY=:99
ENV DONT_PROMPT_WSL_INSTALL=1

LABEL category="vscode"
```

**Estimated Size:** ~2.5GB

---

## Category Image: `repo-test/gui-base`

**Purpose:** Base for GUI applications requiring virtual display (Cursor, Zed, Windsurf).

**Inherits:** `repo-test/base`

**Additional:**
- Xvfb (virtual framebuffer)
- x11vnc (VNC server for debugging)
- fluxbox (lightweight window manager)
- Basic X11 libraries

```dockerfile
# docker/base/Dockerfile.gui
FROM repo-test/base

# X11 and display dependencies
RUN apt-get update && apt-get install -y \
    xvfb \
    x11vnc \
    fluxbox \
    xterm \
    libx11-6 \
    libxext6 \
    libxrender1 \
    libxtst6 \
    libxi6 \
    libxrandr2 \
    libxcomposite1 \
    libxcursor1 \
    libxdamage1 \
    libxfixes3 \
    libxss1 \
    libgconf-2-4 \
    libnss3 \
    libasound2 \
    libatk1.0-0 \
    libgtk-3-0 \
    libgbm1 \
    libdrm2 \
    fonts-liberation \
    && rm -rf /var/lib/apt/lists/*

# Xvfb startup script
COPY scripts/start-xvfb.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/start-xvfb.sh

# VNC password (for debugging, not security)
RUN mkdir -p /root/.vnc && x11vnc -storepasswd "testpass" /root/.vnc/passwd

# Default display
ENV DISPLAY=:99
ENV VNC_PORT=5900

LABEL category="gui"

# Start Xvfb by default
CMD ["/usr/local/bin/start-xvfb.sh"]
```

**Supporting script - `scripts/start-xvfb.sh`:**
```bash
#!/bin/bash
set -e

# Start Xvfb
Xvfb :99 -screen 0 1920x1080x24 &
sleep 1

# Start window manager
fluxbox &
sleep 1

# Optionally start VNC for debugging
if [ "$ENABLE_VNC" = "true" ]; then
    x11vnc -display :99 -forever -usepw -rfbport $VNC_PORT &
fi

# Execute the passed command
exec "$@"
```

**Estimated Size:** ~2GB

---

## Category Image: `repo-test/jetbrains-base`

**Purpose:** Base for JetBrains IDEs.

**Inherits:** `repo-test/gui-base` (needs display capabilities)

**Additional:**
- JDK 17 (required for JetBrains)
- Headless IDE dependencies

```dockerfile
# docker/base/Dockerfile.jetbrains
FROM repo-test/gui-base

# JDK 17
RUN apt-get update && apt-get install -y \
    openjdk-17-jdk \
    && rm -rf /var/lib/apt/lists/*

ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
ENV PATH="${JAVA_HOME}/bin:${PATH}"

# JetBrains Toolbox directory
RUN mkdir -p /opt/jetbrains

LABEL category="jetbrains"
```

**Estimated Size:** ~2.5GB

---

## Tool Images

Each tool gets a thin layer on top of its category base.

### CLI Tools

```dockerfile
# docker/cli/claude/Dockerfile
FROM repo-test/cli-base
RUN npm install -g @anthropic-ai/claude-code
ENTRYPOINT ["claude"]
```

```dockerfile
# docker/cli/aider/Dockerfile
FROM repo-test/cli-base
RUN pip install aider-chat
ENTRYPOINT ["aider"]
```

```dockerfile
# docker/cli/gemini/Dockerfile
FROM repo-test/cli-base
# TODO: Verify package name
RUN npm install -g @google/gemini-cli
ENTRYPOINT ["gemini"]
```

### VS Code Extensions

```dockerfile
# docker/vscode/cline/Dockerfile
FROM repo-test/vscode-base
RUN code --install-extension saoudrizwan.claude-dev --force
```

```dockerfile
# docker/vscode/roo/Dockerfile
FROM repo-test/vscode-base
# TODO: Verify extension ID
RUN code --install-extension roo-code.roo --force
```

### GUI Tools

```dockerfile
# docker/gui/cursor/Dockerfile
FROM repo-test/gui-base

# TODO: Verify installation method
RUN wget https://download.cursor.sh/linux/appImage/x64 -O /opt/cursor.AppImage \
    && chmod +x /opt/cursor.AppImage

# AppImage extraction for containerized use
RUN cd /opt && ./cursor.AppImage --appimage-extract \
    && mv squashfs-root cursor

ENTRYPOINT ["/usr/local/bin/start-xvfb.sh", "/opt/cursor/cursor"]
```

---

## Docker Compose Structure

### Main Compose File

```yaml
# docker-compose.yml
version: "3.9"

x-common-env: &common-env
  TZ: UTC

x-common-volumes: &common-volumes
  - ./test-fixtures:/fixtures:ro
  - ./test-results:/results
  - ./.env:/workspace/.env:ro

x-healthcheck: &healthcheck
  interval: 30s
  timeout: 10s
  retries: 3

services:
  # ============================================
  # CLI Agents
  # ============================================
  claude:
    build:
      context: ./docker
      dockerfile: cli/claude/Dockerfile
    image: repo-test/claude:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
    working_dir: /workspace
    profiles: ["cli", "all"]

  aider:
    build:
      context: ./docker
      dockerfile: cli/aider/Dockerfile
    image: repo-test/aider:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      ANTHROPIC_API_KEY: ${ANTHROPIC_API_KEY}
      OPENAI_API_KEY: ${OPENAI_API_KEY}
    working_dir: /workspace
    profiles: ["cli", "all"]

  gemini:
    build:
      context: ./docker
      dockerfile: cli/gemini/Dockerfile
    image: repo-test/gemini:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      GOOGLE_APPLICATION_CREDENTIALS: /workspace/secrets/gcloud-key.json
    working_dir: /workspace
    profiles: ["cli", "all"]

  # ============================================
  # VS Code Extensions
  # ============================================
  vscode-cline:
    build:
      context: ./docker
      dockerfile: vscode/cline/Dockerfile
    image: repo-test/vscode-cline:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      DISPLAY: ":99"
    working_dir: /workspace
    profiles: ["vscode", "all"]

  vscode-roo:
    build:
      context: ./docker
      dockerfile: vscode/roo/Dockerfile
    image: repo-test/vscode-roo:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      DISPLAY: ":99"
    working_dir: /workspace
    profiles: ["vscode", "all"]

  vscode-copilot:
    build:
      context: ./docker
      dockerfile: vscode/copilot/Dockerfile
    image: repo-test/vscode-copilot:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      GITHUB_TOKEN: ${GITHUB_TOKEN}
    working_dir: /workspace
    profiles: ["vscode", "all"]

  vscode-amazonq:
    build:
      context: ./docker
      dockerfile: vscode/amazonq/Dockerfile
    image: repo-test/vscode-amazonq:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      AWS_ACCESS_KEY_ID: ${AWS_ACCESS_KEY_ID}
      AWS_SECRET_ACCESS_KEY: ${AWS_SECRET_ACCESS_KEY}
    working_dir: /workspace
    profiles: ["vscode", "all"]

  # ============================================
  # GUI Tools
  # ============================================
  cursor:
    build:
      context: ./docker
      dockerfile: gui/cursor/Dockerfile
    image: repo-test/cursor:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      DISPLAY: ":99"
      ENABLE_VNC: "true"
    ports:
      - "5901:5900"
    working_dir: /workspace
    profiles: ["gui", "all"]

  zed:
    build:
      context: ./docker
      dockerfile: gui/zed/Dockerfile
    image: repo-test/zed:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      DISPLAY: ":99"
      ENABLE_VNC: "true"
    ports:
      - "5902:5900"
    working_dir: /workspace
    profiles: ["gui", "all"]

  windsurf:
    build:
      context: ./docker
      dockerfile: gui/windsurf/Dockerfile
    image: repo-test/windsurf:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      DISPLAY: ":99"
      ENABLE_VNC: "true"
    ports:
      - "5903:5900"
    working_dir: /workspace
    profiles: ["gui", "all"]

  # ============================================
  # JetBrains
  # ============================================
  jetbrains-intellij:
    build:
      context: ./docker
      dockerfile: jetbrains/intellij/Dockerfile
    image: repo-test/jetbrains-intellij:latest
    volumes: *common-volumes
    environment:
      <<: *common-env
      DISPLAY: ":99"
    working_dir: /workspace
    profiles: ["jetbrains", "all"]

  # ============================================
  # Infrastructure
  # ============================================
  mock-api:
    build:
      context: ./docker
      dockerfile: mock-api/Dockerfile
    image: repo-test/mock-api:latest
    ports:
      - "8080:8080"
    profiles: ["mock", "ci"]
```

### CI Override

```yaml
# docker-compose.ci.yml
version: "3.9"

services:
  claude:
    environment:
      ANTHROPIC_API_KEY: "mock"
      ANTHROPIC_BASE_URL: "http://mock-api:8080"
    depends_on:
      - mock-api

  aider:
    environment:
      OPENAI_API_KEY: "mock"
      OPENAI_API_BASE: "http://mock-api:8080"
    depends_on:
      - mock-api
```

### Certification Override

```yaml
# docker-compose.cert.yml
version: "3.9"

services:
  # All services use real APIs (from .env)
  # Add resource limits for certification runs
  claude:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
```

---

## Usage Patterns

### Build All Images

```bash
# Build base images first
docker compose build repo-test-base
docker compose build cli-base vscode-base gui-base jetbrains-base

# Build tool images
docker compose build
```

### Run Tests by Profile

```bash
# CLI tools only
docker compose --profile cli up

# VS Code extensions only
docker compose --profile vscode up

# All tools
docker compose --profile all up

# CI mode (with mock APIs)
docker compose -f docker-compose.yml -f docker-compose.ci.yml --profile ci up
```

### Run Single Tool Test

```bash
# Interactive shell in claude container
docker compose run --rm claude bash

# Run specific test
docker compose run --rm claude pytest /fixtures/tests/claude/
```

### Debug GUI Tool

```bash
# Start with VNC enabled
docker compose up cursor

# Connect VNC client to localhost:5901
# Password: testpass
```

---

## Build Optimization

### Multi-stage Builds

For tools requiring compilation:

```dockerfile
# Example: building a Rust tool
FROM repo-test/base AS builder
RUN cargo build --release

FROM repo-test/cli-base AS runtime
COPY --from=builder /workspace/target/release/tool /usr/local/bin/
```

### Layer Caching

Structure Dockerfiles to maximize cache hits:

```dockerfile
# Good: Dependencies first, code last
COPY package.json ./
RUN npm install
COPY . .

# Bad: Busts cache on every code change
COPY . .
RUN npm install
```

### BuildKit

Enable BuildKit for better caching and parallel builds:

```bash
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1
```

---

## Storage Estimates

| Image | Estimated Size | Shared Layers |
|-------|---------------|---------------|
| repo-test/base | 1.5 GB | - |
| repo-test/cli-base | +50 MB | base |
| repo-test/vscode-base | +1 GB | base |
| repo-test/gui-base | +500 MB | base |
| repo-test/jetbrains-base | +500 MB | gui-base |
| Tool images | +100-500 MB each | category base |

**Total unique storage:** ~5-8 GB for all images (due to layer sharing)

---

## CI Integration

### GitHub Actions Example

```yaml
# .github/workflows/integration-test.yml
name: Integration Tests

on:
  push:
    branches: [main]
  pull_request:

jobs:
  cli-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build CLI images
        run: docker compose --profile cli build

      - name: Run CLI tests (mock mode)
        run: |
          docker compose -f docker-compose.yml -f docker-compose.ci.yml \
            --profile cli --profile mock up --abort-on-container-exit

  vscode-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build VS Code images
        run: docker compose --profile vscode build

      - name: Run VS Code extension tests
        run: |
          docker compose -f docker-compose.yml -f docker-compose.ci.yml \
            --profile vscode --profile mock up --abort-on-container-exit

  certification:
    runs-on: ubuntu-latest
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    needs: [cli-tests, vscode-tests]
    steps:
      - uses: actions/checkout@v4

      - name: Run certification (real APIs)
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
          OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
        run: |
          docker compose -f docker-compose.yml -f docker-compose.cert.yml \
            --profile all up --abort-on-container-exit
```

---

## Research TODOs

- [ ] Verify VS Code headless mode works for extension testing
- [ ] Test Xvfb + VNC setup with actual GUI apps
- [ ] Measure actual image sizes after building
- [ ] Benchmark build times with/without cache
- [ ] Test on GitHub Actions runners (resource limits)
- [ ] Evaluate Testcontainers for programmatic Rust integration
