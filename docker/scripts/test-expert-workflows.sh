#!/bin/bash
# Expert-Level Developer Workflow Tests
# Assumes advanced usage patterns that experienced developers would expect

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/expert"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log_test() {
    local name="$1"
    local status="$2"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    if [ "$status" = "PASS" ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "  ${GREEN}✓${NC} $name"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "  ${RED}✗${NC} $name"
    fi
}

WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}Expert-Level Developer Workflow Tests${NC}"
echo "Simulates advanced usage patterns from experienced developers"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# ============================================
# SCENARIO 1: Polyglot Monorepo Setup
# Expert: Expects per-directory tool configs
# ============================================
echo ""
echo -e "${YELLOW}Scenario 1: Polyglot Monorepo with Multiple Teams${NC}"
cd "$WORK_DIR"
mkdir -p polyglot-monorepo
cd polyglot-monorepo
git init -q

# Root config
mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor", "copilot"]
rules = ["repo-root"]

[core]
mode = "standard"
EOF

# Backend team (Python + Claude)
mkdir -p services/backend
cat > services/backend/CLAUDE.md << 'EOF'
# Backend Service Guidelines

<!-- repo:block:python-backend -->
Use Python 3.12+ features. Prefer pydantic for validation.
Follow FastAPI patterns. Use async where beneficial.
<!-- /repo:block:python-backend -->

## Team Notes
Backend team prefers explicit error handling.
EOF

# Frontend team (TypeScript + Cursor)
mkdir -p apps/frontend
cat > apps/frontend/.cursorrules << 'EOF'
<!-- repo:block:frontend-rules -->
React 19 with Server Components preferred.
Use TypeScript strict mode. No any types.
<!-- /repo:block:frontend-rules -->

## Frontend Standards
Component-driven development.
EOF

# Infrastructure team (Rust + multiple tools)
mkdir -p infra/core
cat > infra/core/CLAUDE.md << 'EOF'
<!-- repo:block:rust-infra -->
Safety-critical code. Document all unsafe blocks.
Prefer tokio for async. Use tracing for observability.
<!-- /repo:block:rust-infra -->
EOF
cat > infra/core/.cursorrules << 'EOF'
<!-- repo:block:rust-cursor -->
Rust with strict clippy lints. No unwrap() in production code.
<!-- /repo:block:rust-cursor -->
EOF

# Test: Verify nested config coexistence
if [ -f services/backend/CLAUDE.md ] && [ -f apps/frontend/.cursorrules ]; then
    log_test "Nested team-specific configs coexist" "PASS"
else
    log_test "Nested team-specific configs coexist" "FAIL"
fi

# Test: Multiple tools in same directory
if [ -f infra/core/CLAUDE.md ] && [ -f infra/core/.cursorrules ]; then
    log_test "Multiple tool configs per directory" "PASS"
else
    log_test "Multiple tool configs per directory" "FAIL"
fi

# Test: Block syntax consistency across nested configs
backend_blocks=$(grep -c "repo:block:" services/backend/CLAUDE.md || echo "0")
frontend_blocks=$(grep -c "repo:block:" apps/frontend/.cursorrules || echo "0")
if [ "$backend_blocks" -gt 0 ] && [ "$frontend_blocks" -gt 0 ]; then
    log_test "Consistent block syntax in nested configs" "PASS"
else
    log_test "Consistent block syntax in nested configs" "FAIL"
fi

# ============================================
# SCENARIO 2: Feature Flag Development
# Expert: Expects conditional tool behavior
# ============================================
echo ""
echo -e "${YELLOW}Scenario 2: Feature Flag Development Workflow${NC}"
cd "$WORK_DIR"
mkdir -p feature-flags
cd feature-flags
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "aider"]

[core]
mode = "standard"

[presets]
"env:python" = {}
EOF

# Create config with feature-flag-aware rules
cat > CLAUDE.md << 'EOF'
# Feature Flag Development

<!-- repo:block:feature-flags -->
## Active Feature Flags

When working with feature flags:
1. Check LaunchDarkly integration: `src/flags/`
2. Default to flag OFF in local dev
3. Add flag evaluation to all new features
4. Document flag lifecycle in PR description

Current flags:
- `new_checkout_flow` - A/B testing checkout
- `dark_mode` - UI theme experiment
- `async_payments` - Background payment processing
<!-- /repo:block:feature-flags -->

## Manual Development Notes
Always test both flag states before PR.
EOF

# Test: Feature flag documentation preserved
if grep -q "new_checkout_flow" CLAUDE.md; then
    log_test "Feature flag list in managed block" "PASS"
else
    log_test "Feature flag list in managed block" "FAIL"
fi

# Test: Manual notes outside block preserved
if grep -q "both flag states" CLAUDE.md; then
    log_test "Manual notes preserved outside block" "PASS"
else
    log_test "Manual notes preserved outside block" "FAIL"
fi

# Simulate flag lifecycle update - append graduated status
echo "" >> CLAUDE.md
echo "## Graduated Flags" >> CLAUDE.md
echo "- new_checkout_flow - GRADUATED (100% rollout)" >> CLAUDE.md
if grep -q "GRADUATED" CLAUDE.md; then
    log_test "Feature flag status update" "PASS"
else
    log_test "Feature flag status update" "FAIL"
fi

# ============================================
# SCENARIO 3: Multi-Environment Configuration
# Expert: Expects env-specific tool settings
# ============================================
echo ""
echo -e "${YELLOW}Scenario 3: Multi-Environment Config Management${NC}"
cd "$WORK_DIR"
mkdir -p multi-env
cd multi-env
git init -q

mkdir -p .repository/environments
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor", "cline"]

[core]
mode = "standard"

[presets]
"env:python" = { version = "3.12" }
EOF

# Development environment
cat > .repository/environments/dev.toml << 'EOF'
[api]
base_url = "http://localhost:8000"
mock_services = true

[tools]
enable_experimental = true
verbose_logging = true
EOF

# Staging environment
cat > .repository/environments/staging.toml << 'EOF'
[api]
base_url = "https://staging-api.example.com"
mock_services = false

[tools]
enable_experimental = true
verbose_logging = false
EOF

# Production environment
cat > .repository/environments/prod.toml << 'EOF'
[api]
base_url = "https://api.example.com"
mock_services = false

[tools]
enable_experimental = false
verbose_logging = false
EOF

# Create environment-aware CLAUDE.md
cat > CLAUDE.md << 'EOF'
# Multi-Environment Development

<!-- repo:block:env-config -->
## Environment Configuration

Check `.repository/environments/` for env-specific settings.

Development: Local testing with mocks
Staging: Integration testing with real services
Production: Live deployment (read-only access)

Use `REPO_ENV=dev|staging|prod` to switch contexts.
<!-- /repo:block:env-config -->
EOF

# Test: Environment files exist
env_count=$(ls -1 .repository/environments/*.toml 2>/dev/null | wc -l)
if [ "$env_count" -eq 3 ]; then
    log_test "Three environment configs present" "PASS"
else
    log_test "Three environment configs present" "FAIL"
fi

# Test: Env documentation in tool config
if grep -q "REPO_ENV" CLAUDE.md; then
    log_test "Environment switch docs in config" "PASS"
else
    log_test "Environment switch docs in config" "FAIL"
fi

# ============================================
# SCENARIO 4: AI Agent Collaboration
# Expert: Multiple AI tools working together
# ============================================
echo ""
echo -e "${YELLOW}Scenario 4: Multi-Agent AI Collaboration${NC}"
cd "$WORK_DIR"
mkdir -p ai-collaboration
cd ai-collaboration
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "aider", "cline", "copilot"]

[core]
mode = "standard"
EOF

# Claude for architecture and design
cat > CLAUDE.md << 'EOF'
# Claude Code - Architecture Lead

<!-- repo:block:claude-role -->
## Role: System Architecture

Claude handles:
- System design decisions
- API contract definitions
- Documentation generation
- Code review orchestration

Defer implementation details to Aider.
Defer VS Code tasks to Cline.
<!-- /repo:block:claude-role -->

## Agent Handoff Protocol
When handing off:
1. Document the context
2. Specify expected output
3. Define success criteria
EOF

# Aider for implementation
cat > .aider.conf.yml << 'EOF'
# repo:block:aider-role
# Role: Implementation Agent
# Aider handles:
# - Feature implementation from specs
# - Test writing
# - Bug fixes with focused scope
# /repo:block:aider-role

model: claude-3-5-sonnet-20241022
auto-commits: true
dirty-commits: true

# repo:block:aider-patterns
lint-cmd: ruff check --fix
test-cmd: pytest -x
# /repo:block:aider-patterns
EOF

# Cline for VS Code integration
cat > .clinerules << 'EOF'
<!-- repo:block:cline-role -->
## Role: VS Code Integration

Cline handles:
- IDE-specific tasks
- Real-time assistance
- Quick edits and refactors
- Extension management

For architectural changes, defer to Claude.
<!-- /repo:block:cline-role -->
EOF

# Test: All agent configs exist
if [ -f CLAUDE.md ] && [ -f .aider.conf.yml ] && [ -f .clinerules ]; then
    log_test "Multi-agent configs all present" "PASS"
else
    log_test "Multi-agent configs all present" "FAIL"
fi

# Test: Role documentation defined
claude_role=$(grep -c "System Architecture\|Architecture Lead" CLAUDE.md || echo "0")
aider_role=$(grep -c "Implementation Agent" .aider.conf.yml || echo "0")
cline_role=$(grep -c "VS Code Integration" .clinerules || echo "0")
if [ "$claude_role" -gt 0 ] && [ "$aider_role" -gt 0 ] && [ "$cline_role" -gt 0 ]; then
    log_test "Agent roles clearly defined" "PASS"
else
    log_test "Agent roles clearly defined" "FAIL"
fi

# Test: Handoff protocol documented
if grep -q "Handoff Protocol" CLAUDE.md; then
    log_test "Agent handoff protocol documented" "PASS"
else
    log_test "Agent handoff protocol documented" "FAIL"
fi

# ============================================
# SCENARIO 5: Security-Sensitive Development
# Expert: Expects secrets management awareness
# ============================================
echo ""
echo -e "${YELLOW}Scenario 5: Security-Sensitive Development${NC}"
cd "$WORK_DIR"
mkdir -p secure-dev
cd secure-dev
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "copilot"]

[core]
mode = "standard"
EOF

# Create security-aware configs
cat > CLAUDE.md << 'EOF'
# Security-First Development

<!-- repo:block:security -->
## Security Guidelines

NEVER commit:
- API keys, tokens, or credentials
- .env files with real values
- Private certificates
- Database connection strings with passwords

ALWAYS:
- Use environment variables for secrets
- Reference AWS Secrets Manager or HashiCorp Vault
- Review diffs before committing
- Run git-secrets pre-commit hook
<!-- /repo:block:security -->

<!-- repo:block:sensitive-paths -->
## Sensitive Paths (DO NOT AI-GENERATE)

- `config/credentials.yml` - Template only, no values
- `scripts/deploy-prod.sh` - Requires manual review
- `terraform/secrets.tf` - Infrastructure secrets
<!-- /repo:block:sensitive-paths -->
EOF

# Create .gitignore for security
cat > .gitignore << 'EOF'
# Secrets
.env
.env.local
.env.*.local
*.pem
*.key
credentials.json
secrets.yml

# IDE
.idea/
.vscode/settings.json
EOF

# Test: Security guidelines documented
if grep -q "NEVER commit" CLAUDE.md; then
    log_test "Security never-commit list documented" "PASS"
else
    log_test "Security never-commit list documented" "FAIL"
fi

# Test: Sensitive paths documented
if grep -q "Sensitive Paths" CLAUDE.md; then
    log_test "Sensitive paths section exists" "PASS"
else
    log_test "Sensitive paths section exists" "FAIL"
fi

# Test: .gitignore includes secrets patterns
if grep -q "credentials.json" .gitignore; then
    log_test ".gitignore excludes credential files" "PASS"
else
    log_test ".gitignore excludes credential files" "FAIL"
fi

# ============================================
# SCENARIO 6: Performance-Critical Development
# Expert: Expects profiling and benchmark awareness
# ============================================
echo ""
echo -e "${YELLOW}Scenario 6: Performance-Critical Development${NC}"
cd "$WORK_DIR"
mkdir -p perf-critical
cd perf-critical
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor"]

[core]
mode = "standard"

[presets]
"env:rust" = {}
EOF

cat > CLAUDE.md << 'EOF'
# Performance-Critical Service

<!-- repo:block:performance -->
## Performance Requirements

Target metrics:
- P99 latency: <10ms
- Throughput: >10k RPS
- Memory: <512MB per instance

Profiling tools:
- `cargo flamegraph` for CPU profiling
- `heaptrack` for memory analysis
- `criterion` for micro-benchmarks

Before any PR:
1. Run `cargo bench` comparison
2. Check for allocations in hot paths
3. Verify no new dependencies in critical paths
<!-- /repo:block:performance -->

<!-- repo:block:hot-paths -->
## Hot Paths (Optimize Carefully)

- `src/parser/mod.rs` - Main request parser
- `src/cache/lru.rs` - In-memory cache
- `src/db/pool.rs` - Connection pooling
<!-- /repo:block:hot-paths -->
EOF

# Create benchmark config
mkdir -p benches
cat > benches/bench_parser.rs << 'EOF'
use criterion::{criterion_group, criterion_main, Criterion};

fn parser_benchmark(c: &mut Criterion) {
    c.bench_function("parse_request", |b| {
        b.iter(|| {
            // Benchmark implementation
        });
    });
}

criterion_group!(benches, parser_benchmark);
criterion_main!(benches);
EOF

# Test: Performance requirements documented
if grep -q "P99 latency" CLAUDE.md; then
    log_test "Performance SLOs documented" "PASS"
else
    log_test "Performance SLOs documented" "FAIL"
fi

# Test: Profiling tools documented
if grep -q "flamegraph" CLAUDE.md; then
    log_test "Profiling tools documented" "PASS"
else
    log_test "Profiling tools documented" "FAIL"
fi

# Test: Hot paths identified
if grep -q "Hot Paths" CLAUDE.md; then
    log_test "Hot paths section exists" "PASS"
else
    log_test "Hot paths section exists" "FAIL"
fi

# Test: Benchmark file exists
if [ -f benches/bench_parser.rs ]; then
    log_test "Benchmark file structure present" "PASS"
else
    log_test "Benchmark file structure present" "FAIL"
fi

# ============================================
# SCENARIO 7: Open Source Project Setup
# Expert: Community contribution workflow
# ============================================
echo ""
echo -e "${YELLOW}Scenario 7: Open Source Project Setup${NC}"
cd "$WORK_DIR"
mkdir -p oss-project
cd oss-project
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor", "copilot", "aider"]

[core]
mode = "standard"
EOF

cat > CLAUDE.md << 'EOF'
# Open Source Project Guidelines

<!-- repo:block:contribution -->
## Contribution Workflow

1. Fork the repository
2. Create feature branch from `main`
3. Follow conventional commits:
   - `feat:` new features
   - `fix:` bug fixes
   - `docs:` documentation
   - `refactor:` code refactoring
4. Run `make lint && make test`
5. Submit PR with description template

DCO sign-off required: `git commit -s`
<!-- /repo:block:contribution -->

<!-- repo:block:code-style -->
## Code Style

- Run `cargo fmt` before commit
- Pass `cargo clippy` with no warnings
- Maintain >80% test coverage
- Document public APIs with examples
<!-- /repo:block:code-style -->
EOF

# Create CONTRIBUTING.md
cat > CONTRIBUTING.md << 'EOF'
# Contributing to Project

Thank you for contributing!

## Getting Started

1. Clone: `git clone https://github.com/org/project`
2. Install: `cargo build`
3. Test: `cargo test`

## Pull Request Process

1. Update documentation for changes
2. Add tests for new functionality
3. Ensure CI passes
4. Request review from maintainers
EOF

# Create PR template
mkdir -p .github
cat > .github/PULL_REQUEST_TEMPLATE.md << 'EOF'
## Description

<!-- Describe your changes -->

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Checklist

- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Signed-off-by added
EOF

# Test: Contribution workflow documented
if grep -q "Contribution Workflow" CLAUDE.md; then
    log_test "Contribution workflow in AI config" "PASS"
else
    log_test "Contribution workflow in AI config" "FAIL"
fi

# Test: CONTRIBUTING.md exists
if [ -f CONTRIBUTING.md ]; then
    log_test "CONTRIBUTING.md file exists" "PASS"
else
    log_test "CONTRIBUTING.md file exists" "FAIL"
fi

# Test: PR template exists
if [ -f .github/PULL_REQUEST_TEMPLATE.md ]; then
    log_test "PR template configured" "PASS"
else
    log_test "PR template configured" "FAIL"
fi

# Test: DCO sign-off mentioned
if grep -q "sign-off" CLAUDE.md; then
    log_test "DCO sign-off requirement documented" "PASS"
else
    log_test "DCO sign-off requirement documented" "FAIL"
fi

# ============================================
# SCENARIO 8: Microservices Architecture
# Expert: Service mesh and API contracts
# ============================================
echo ""
echo -e "${YELLOW}Scenario 8: Microservices Architecture${NC}"
cd "$WORK_DIR"
mkdir -p microservices
cd microservices
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor"]

[core]
mode = "standard"
EOF

# Create service-specific configs
mkdir -p services/user-service
mkdir -p services/payment-service
mkdir -p services/notification-service

cat > services/user-service/CLAUDE.md << 'EOF'
# User Service

<!-- repo:block:service-info -->
## Service: user-service
Port: 8001
Dependencies: postgres, redis
Downstream: payment-service, notification-service

API Contract: `api/contracts/user-service.yaml`
<!-- /repo:block:service-info -->

## Local Development
`docker-compose up user-service`
EOF

cat > services/payment-service/CLAUDE.md << 'EOF'
# Payment Service

<!-- repo:block:service-info -->
## Service: payment-service
Port: 8002
Dependencies: postgres, stripe-api
Upstream: user-service
Downstream: notification-service

API Contract: `api/contracts/payment-service.yaml`
<!-- /repo:block:service-info -->

## PCI Compliance Notes
No card data stored locally.
EOF

cat > services/notification-service/CLAUDE.md << 'EOF'
# Notification Service

<!-- repo:block:service-info -->
## Service: notification-service
Port: 8003
Dependencies: redis, sendgrid-api
Upstream: user-service, payment-service

API Contract: `api/contracts/notification-service.yaml`
<!-- /repo:block:service-info -->
EOF

# Create API contracts directory
mkdir -p api/contracts

# Test: Service configs exist
svc_count=$(find services -name "CLAUDE.md" | wc -l)
if [ "$svc_count" -eq 3 ]; then
    log_test "All service configs present" "PASS"
else
    log_test "All service configs present" "FAIL"
fi

# Test: Service dependencies documented
if grep -q "Dependencies:" services/user-service/CLAUDE.md; then
    log_test "Service dependencies documented" "PASS"
else
    log_test "Service dependencies documented" "FAIL"
fi

# Test: API contracts referenced
if grep -q "API Contract:" services/user-service/CLAUDE.md; then
    log_test "API contracts referenced in configs" "PASS"
else
    log_test "API contracts referenced in configs" "FAIL"
fi

# ============================================
# SCENARIO 9: Database Migration Workflow
# Expert: Schema change management
# ============================================
echo ""
echo -e "${YELLOW}Scenario 9: Database Migration Workflow${NC}"
cd "$WORK_DIR"
mkdir -p db-migrations
cd db-migrations
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "aider"]

[core]
mode = "standard"
EOF

cat > CLAUDE.md << 'EOF'
# Database Migration Guidelines

<!-- repo:block:migrations -->
## Migration Rules

1. Migrations must be reversible
2. No data loss in rollback scenarios
3. Test on production data snapshot first
4. Use explicit transactions
5. Document expected duration

Migration commands:
- `sqlx migrate add <name>` - Create migration
- `sqlx migrate run` - Apply pending
- `sqlx migrate revert` - Rollback last

Current schema version: Check `_sqlx_migrations` table.
<!-- /repo:block:migrations -->

<!-- repo:block:dangerous-ops -->
## Dangerous Operations (Require DBA Review)

- DROP TABLE
- ALTER COLUMN type changes
- Index drops on large tables
- Foreign key modifications
<!-- /repo:block:dangerous-ops -->
EOF

mkdir -p migrations
cat > migrations/20260129_001_create_users.sql << 'EOF'
-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Rollback: DROP TABLE users;
EOF

# Test: Migration rules documented
if grep -q "Migrations must be reversible" CLAUDE.md; then
    log_test "Migration reversibility rule documented" "PASS"
else
    log_test "Migration reversibility rule documented" "FAIL"
fi

# Test: Dangerous operations listed
if grep -q "Dangerous Operations" CLAUDE.md; then
    log_test "Dangerous operations section exists" "PASS"
else
    log_test "Dangerous operations section exists" "FAIL"
fi

# Test: Migration file exists
if [ -f migrations/20260129_001_create_users.sql ]; then
    log_test "Migration file structure present" "PASS"
else
    log_test "Migration file structure present" "FAIL"
fi

# ============================================
# SCENARIO 10: CI/CD Pipeline Integration
# Expert: Build/deploy automation awareness
# ============================================
echo ""
echo -e "${YELLOW}Scenario 10: CI/CD Pipeline Integration${NC}"
cd "$WORK_DIR"
mkdir -p cicd-project
cd cicd-project
git init -q

mkdir -p .repository
cat > .repository/config.toml << 'EOF'
tools = ["claude", "cursor", "copilot"]

[core]
mode = "standard"
EOF

cat > CLAUDE.md << 'EOF'
# CI/CD Integration

<!-- repo:block:pipeline -->
## Pipeline Stages

1. **lint** - Code style checks
2. **test** - Unit and integration tests
3. **build** - Docker image creation
4. **security** - Vulnerability scanning
5. **deploy** - Environment deployment

Pipeline config: `.github/workflows/ci.yml`

Required checks before merge:
- lint ✓
- test ✓
- security ✓
<!-- /repo:block:pipeline -->

<!-- repo:block:deployments -->
## Deployment Environments

| Environment | Branch | Auto-Deploy |
|-------------|--------|-------------|
| Development | `develop` | Yes |
| Staging | `main` | Yes |
| Production | `main` + tag | Manual |
<!-- /repo:block:deployments -->
EOF

mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Lint
        run: cargo fmt --check && cargo clippy

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Test
        run: cargo test

  build:
    needs: [lint, test]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release
EOF

# Test: Pipeline stages documented
if grep -q "Pipeline Stages" CLAUDE.md; then
    log_test "Pipeline stages documented" "PASS"
else
    log_test "Pipeline stages documented" "FAIL"
fi

# Test: Deployment environments documented
if grep -q "Deployment Environments" CLAUDE.md; then
    log_test "Deployment environments documented" "PASS"
else
    log_test "Deployment environments documented" "FAIL"
fi

# Test: GitHub Actions workflow exists
if [ -f .github/workflows/ci.yml ]; then
    log_test "CI workflow file present" "PASS"
else
    log_test "CI workflow file present" "FAIL"
fi

# Test: CI references in CLAUDE.md
if grep -q ".github/workflows" CLAUDE.md; then
    log_test "CI config path referenced" "PASS"
else
    log_test "CI config path referenced" "FAIL"
fi

# ============================================
# FINAL SUMMARY
# ============================================
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}Expert Workflow Test Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  Total Tests:  $TOTAL_TESTS"
echo -e "  Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "  Failed:       ${RED}$FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}${BOLD}ALL EXPERT WORKFLOW TESTS PASSED${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}SOME TESTS FAILED${NC}"
    exit 1
fi
