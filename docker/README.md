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
repo-test/base              Ubuntu 22.04 + Node.js + Python + Rust
  ├── repo-test/cli-base         Base for CLI tools
  │     ├── repo-test/claude         Claude Code CLI
  │     ├── repo-test/aider          Aider
  │     ├── repo-test/gemini         Gemini CLI
  │     └── repo-test/cursor         Cursor CLI (agent command)
  │
  └── repo-test/vscode-base      VS Code headless + Xvfb
        ├── repo-test/cline          Cline extension
        └── repo-test/roo            Roo Code extension
```

## Profiles

- `cli` - CLI tools (Claude, Aider, Gemini, Cursor)
- `vscode` - VS Code extensions (Cline, Roo)
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
