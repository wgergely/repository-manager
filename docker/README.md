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
- `build-verify.sh` - Build with logging and verification
- `verify-tools.sh` - Verify tool installations
- `smoke-test.sh` - Basic tool functionality tests
- `test-config-generation.sh` - Repository Manager config tests
- `test-tool-reads-config.sh` - Tool config reading tests
- `test-e2e.sh` - End-to-end tests with mock API
- `test-all.sh` - Run all test suites
- `run-tests.sh <mode> <profile>` - Legacy test runner

## Running Tests

### Full Test Suite

Run all integration tests:

```bash
./docker/scripts/test-all.sh
```

### Individual Test Suites

1. **Build Verification** - Ensure all images build:
   ```bash
   ./docker/scripts/build-verify.sh
   ```

2. **Smoke Tests** - Basic tool functionality:
   ```bash
   ./docker/scripts/smoke-test.sh
   ```

3. **Config Generation** - Repository Manager generates valid configs:
   ```bash
   ./docker/scripts/test-config-generation.sh
   ```

4. **Tool Config Reading** - Tools can read generated configs:
   ```bash
   ./docker/scripts/test-tool-reads-config.sh
   ```

5. **End-to-End** - Full workflow with mock API:
   ```bash
   ./docker/scripts/test-e2e.sh
   ```

## Test Results

Results are saved to `test-results/`:

```
test-results/
├── builds/          # Build logs per image
├── smoke/           # Smoke test output
├── config-gen/      # Generated config files
├── tool-reads/      # Tool reading verification
└── e2e/             # End-to-end test logs
```

## Test Fixtures

Test fixtures are in `test-fixtures/`:

```
test-fixtures/
├── repos/
│   ├── simple-project/     # Basic test project
│   └── config-test/        # Config generation test
│       └── .repository/    # Repository Manager config
└── expected/               # Expected output for validation
    ├── cursor/
    ├── claude/
    └── aider/
```

## CI/CD

GitHub Actions runs tests on:
- Push to `main` or `registry-architecture`
- Pull requests affecting `docker/`, `crates/`, or `test-fixtures/`

Pipeline stages:
1. Build base images
2. Build tool images (parallel matrix)
3. Build Repository Manager image
4. Run smoke tests
5. Run integration tests with mock API

## Architecture

Based on ADRs in `docs/research/audit/decisions.md`:
- ADR-001: Docker Compose for orchestration
- ADR-002: Layered base images
- ADR-003: Hybrid API testing strategy
- ADR-007: WireMock for API mocking
- ADR-008: Ubuntu 22.04 as base image
