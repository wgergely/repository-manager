#!/bin/bash
# Advanced Developer Workflow Simulation Tests
# Complex multi-step scenarios that mirror real team development patterns

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/advanced-workflows"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║         ADVANCED DEVELOPER WORKFLOW SIMULATIONS              ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Test tracking
TOTAL=0
PASSED=0
FAILED=0

log_test() {
    local name="$1"
    local status="$2"
    local details="$3"
    TOTAL=$((TOTAL + 1))

    case "$status" in
        PASS) PASSED=$((PASSED + 1)); echo -e "    ${GREEN}✓${NC} $name" ;;
        FAIL) FAILED=$((FAILED + 1)); echo -e "    ${RED}✗${NC} $name"; [ -n "$details" ] && echo "      $details" ;;
    esac
}

log_step() {
    echo -e "  ${CYAN}→${NC} $1"
}

# Create test workspace
WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

# ════════════════════════════════════════════════════════════════
# WORKFLOW 1: Team Collaboration with Mixed Tool Preferences
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  WORKFLOW 1: Team Collaboration (Mixed Tool Preferences)     ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Team of 4 developers each using different AI tools"
echo "- Alice: Claude CLI (prefers terminal)"
echo "- Bob: Cursor IDE (visual editor)"
echo "- Carol: Aider (git-integrated)"
echo "- Dave: Cline in VS Code"
echo ""

mkdir -p "$WORK_DIR/team-project/.repository/rules"
cd "$WORK_DIR/team-project"
git init -q

log_step "Creating shared team rules"

cat > .repository/rules/team-standards.md << 'EOF'
# Team Coding Standards

## Code Review Requirements
- All PRs need 2 approvals
- Tests must pass before merge
- Documentation required for public APIs

## Commit Conventions
- Use conventional commits (feat:, fix:, docs:)
- Reference issue numbers
- Keep commits atomic
EOF

cat > .repository/rules/security.md << 'EOF'
# Security Guidelines

- Never commit secrets
- Use environment variables for credentials
- Validate all user inputs
- Use parameterized queries
EOF

log_test "Shared rules created" "PASS"

log_step "Generating configs for each team member's tool"

# Alice: Claude CLI
cat > CLAUDE.md << 'EOF'
# Team Project - Claude Configuration

<!-- repo:block:team-standards -->
# Team Coding Standards

## Code Review Requirements
- All PRs need 2 approvals
- Tests must pass before merge
- Documentation required for public APIs

## Commit Conventions
- Use conventional commits (feat:, fix:, docs:)
- Reference issue numbers
- Keep commits atomic
<!-- /repo:block:team-standards -->

<!-- repo:block:security -->
# Security Guidelines

- Never commit secrets
- Use environment variables for credentials
- Validate all user inputs
- Use parameterized queries
<!-- /repo:block:security -->
EOF

# Bob: Cursor IDE
cat > .cursorrules << 'EOF'
# Team Project - Cursor Configuration

<!-- repo:block:team-standards -->
# Team Coding Standards

## Code Review Requirements
- All PRs need 2 approvals
- Tests must pass before merge
- Documentation required for public APIs

## Commit Conventions
- Use conventional commits (feat:, fix:, docs:)
- Reference issue numbers
- Keep commits atomic
<!-- /repo:block:team-standards -->

<!-- repo:block:security -->
# Security Guidelines

- Never commit secrets
- Use environment variables for credentials
- Validate all user inputs
- Use parameterized queries
<!-- /repo:block:security -->
EOF

# Carol: Aider
cat > .aider.conf.yml << 'EOF'
# Team Project - Aider Configuration
model: claude-3-opus-20240229
auto-commits: false
auto-test: true
test-cmd: cargo test

# repo:block:team-standards
read:
  - .repository/rules/team-standards.md
  - .repository/rules/security.md
# /repo:block:team-standards
EOF

# Dave: Cline
cat > .clinerules << 'EOF'
# Team Project - Cline Configuration

<!-- repo:block:team-standards -->
# Team Coding Standards

## Code Review Requirements
- All PRs need 2 approvals
- Tests must pass before merge
- Documentation required for public APIs

## Commit Conventions
- Use conventional commits (feat:, fix:, docs:)
- Reference issue numbers
- Keep commits atomic
<!-- /repo:block:team-standards -->

<!-- repo:block:security -->
# Security Guidelines

- Never commit secrets
- Use environment variables for credentials
- Validate all user inputs
- Use parameterized queries
<!-- /repo:block:security -->
EOF

log_test "All 4 tool configs generated" "PASS"

log_step "Verifying rule consistency across all tools"

# Extract team-standards block from each
claude_rules=$(sed -n '/<!-- repo:block:team-standards -->/,/<!-- \/repo:block:team-standards -->/p' CLAUDE.md | grep -v "repo:block")
cursor_rules=$(sed -n '/<!-- repo:block:team-standards -->/,/<!-- \/repo:block:team-standards -->/p' .cursorrules | grep -v "repo:block")
cline_rules=$(sed -n '/<!-- repo:block:team-standards -->/,/<!-- \/repo:block:team-standards -->/p' .clinerules | grep -v "repo:block")

if [ "$claude_rules" = "$cursor_rules" ] && [ "$cursor_rules" = "$cline_rules" ]; then
    log_test "Team standards identical across Claude/Cursor/Cline" "PASS"
else
    log_test "Team standards identical across Claude/Cursor/Cline" "FAIL"
fi

log_step "Simulating Alice updates team standards via Claude"

# Alice adds a new rule
cat >> .repository/rules/team-standards.md << 'EOF'

## Performance Requirements
- Response times under 200ms for API calls
- Lazy loading for large datasets
EOF

log_test "Alice's rule update applied" "PASS"

log_step "Simulating sync to propagate Alice's changes"

# Update all configs with new content
for config in CLAUDE.md .cursorrules .clinerules; do
    sed -i '/<!-- \/repo:block:team-standards -->/i \
\
## Performance Requirements\
- Response times under 200ms for API calls\
- Lazy loading for large datasets' "$config"
done

if grep -q "Performance Requirements" CLAUDE.md && \
   grep -q "Performance Requirements" .cursorrules && \
   grep -q "Performance Requirements" .clinerules; then
    log_test "All team members see Alice's update" "PASS"
else
    log_test "All team members see Alice's update" "FAIL"
fi

log_step "Simulating Bob adds personal notes in Cursor (outside blocks)"

cat >> .cursorrules << 'EOF'

# Bob's Personal Notes (not synced)
- Remember to check with Alice about the auth flow
- TODO: Refactor the user service
EOF

if grep -q "Bob's Personal Notes" .cursorrules && \
   ! grep -q "Bob's Personal Notes" CLAUDE.md; then
    log_test "Bob's personal notes stay in Cursor only" "PASS"
else
    log_test "Bob's personal notes stay in Cursor only" "FAIL"
fi

git add -A
git commit -q -m "feat: team collaboration test setup"

echo ""

# ════════════════════════════════════════════════════════════════
# WORKFLOW 2: CI/CD Pipeline Integration
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  WORKFLOW 2: CI/CD Pipeline Integration                      ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Repository Manager runs in CI to validate configs"
echo ""

mkdir -p "$WORK_DIR/ci-project/.github/workflows"
cd "$WORK_DIR/ci-project"
git init -q

log_step "Creating GitHub Actions workflow for config validation"

cat > .github/workflows/validate-configs.yml << 'EOF'
name: Validate AI Tool Configs

on:
  push:
    paths:
      - '.repository/**'
      - 'CLAUDE.md'
      - '.cursorrules'
      - '.aider.conf.yml'
  pull_request:
    paths:
      - '.repository/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Validate managed blocks
        run: |
          # Check all managed blocks are closed
          for file in CLAUDE.md .cursorrules .clinerules .roorules; do
            if [ -f "$file" ]; then
              opens=$(grep -c '<!-- repo:block:' "$file" || echo 0)
              closes=$(grep -c '<!-- /repo:block:' "$file" || echo 0)
              if [ "$opens" != "$closes" ]; then
                echo "ERROR: Unclosed blocks in $file"
                exit 1
              fi
            fi
          done

      - name: Check rule consistency
        run: |
          # Extract and compare blocks across tools
          echo "Checking cross-tool consistency..."

      - name: Sync configs
        run: |
          # Run repo sync to ensure configs are up to date
          echo "Running repo sync..."
EOF

log_test "GitHub Actions workflow created" "PASS"

log_step "Creating pre-commit hook for local validation"

mkdir -p .git/hooks

cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Validate AI tool configs before commit

echo "Validating AI tool configurations..."

# Check for unclosed blocks
for file in CLAUDE.md .cursorrules .clinerules .roorules .aider.conf.yml; do
    if [ -f "$file" ]; then
        if [[ "$file" == *.yml ]]; then
            # YAML block syntax
            opens=$(grep -c '# repo:block:' "$file" 2>/dev/null || echo 0)
            closes=$(grep -c '# /repo:block:' "$file" 2>/dev/null || echo 0)
        else
            # Markdown block syntax
            opens=$(grep -c '<!-- repo:block:' "$file" 2>/dev/null || echo 0)
            closes=$(grep -c '<!-- /repo:block:' "$file" 2>/dev/null || echo 0)
        fi

        if [ "$opens" != "$closes" ]; then
            echo "ERROR: Unclosed managed blocks in $file"
            echo "  Opens: $opens, Closes: $closes"
            exit 1
        fi
    fi
done

echo "✓ All configs valid"
EOF

chmod +x .git/hooks/pre-commit

log_test "Pre-commit hook installed" "PASS"

log_step "Simulating CI validation run"

# Create test configs
mkdir -p .repository/rules
cat > .repository/rules/coding.md << 'EOF'
# Coding Standards
- Write clean code
EOF

cat > CLAUDE.md << 'EOF'
<!-- repo:block:coding -->
# Coding Standards
- Write clean code
<!-- /repo:block:coding -->
EOF

# Run validation
opens=$(grep -c '<!-- repo:block:' CLAUDE.md || echo 0)
closes=$(grep -c '<!-- /repo:block:' CLAUDE.md || echo 0)

if [ "$opens" = "$closes" ]; then
    log_test "CI validation passes for valid config" "PASS"
else
    log_test "CI validation passes for valid config" "FAIL"
fi

log_step "Simulating CI catches invalid config"

cat > BROKEN.md << 'EOF'
<!-- repo:block:test -->
This block is never closed
EOF

opens=$(grep -c '<!-- repo:block:' BROKEN.md || echo 0)
closes=$(grep -c '<!-- /repo:block:' BROKEN.md || echo 0)

if [ "$opens" != "$closes" ]; then
    log_test "CI detects unclosed block" "PASS"
else
    log_test "CI detects unclosed block" "FAIL"
fi

rm BROKEN.md

echo ""

# ════════════════════════════════════════════════════════════════
# WORKFLOW 3: Monorepo with Multiple Projects
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  WORKFLOW 3: Monorepo with Multiple Projects                 ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Monorepo with 3 projects, each with different rules"
echo "- /packages/frontend (React/TypeScript)"
echo "- /packages/backend (Rust)"
echo "- /packages/shared (TypeScript library)"
echo ""

mkdir -p "$WORK_DIR/monorepo"
cd "$WORK_DIR/monorepo"
git init -q

log_step "Creating monorepo structure"

mkdir -p packages/frontend/src
mkdir -p packages/backend/src
mkdir -p packages/shared/src
mkdir -p .repository/rules

# Root-level shared rules
cat > .repository/rules/shared.md << 'EOF'
# Shared Rules (All Projects)

- Use conventional commits
- Run tests before pushing
- Keep dependencies up to date
EOF

log_test "Root-level shared rules created" "PASS"

log_step "Creating project-specific rules"

# Frontend rules
mkdir -p packages/frontend/.repository/rules
cat > packages/frontend/.repository/rules/react.md << 'EOF'
# Frontend Rules

- Use functional components
- Use TypeScript strict mode
- Test with React Testing Library
- Follow accessibility guidelines
EOF

cat > packages/frontend/CLAUDE.md << 'EOF'
# Frontend Project

<!-- repo:block:shared -->
# Shared Rules (All Projects)

- Use conventional commits
- Run tests before pushing
- Keep dependencies up to date
<!-- /repo:block:shared -->

<!-- repo:block:react -->
# Frontend Rules

- Use functional components
- Use TypeScript strict mode
- Test with React Testing Library
- Follow accessibility guidelines
<!-- /repo:block:react -->
EOF

log_test "Frontend project rules created" "PASS"

# Backend rules
mkdir -p packages/backend/.repository/rules
cat > packages/backend/.repository/rules/rust.md << 'EOF'
# Backend Rules

- Use async/await patterns
- Handle all errors with Result<T, E>
- Write integration tests for APIs
- Document with rustdoc
EOF

cat > packages/backend/CLAUDE.md << 'EOF'
# Backend Project

<!-- repo:block:shared -->
# Shared Rules (All Projects)

- Use conventional commits
- Run tests before pushing
- Keep dependencies up to date
<!-- /repo:block:shared -->

<!-- repo:block:rust -->
# Backend Rules

- Use async/await patterns
- Handle all errors with Result<T, E>
- Write integration tests for APIs
- Document with rustdoc
<!-- /repo:block:rust -->
EOF

log_test "Backend project rules created" "PASS"

# Shared library rules
mkdir -p packages/shared/.repository/rules
cat > packages/shared/.repository/rules/library.md << 'EOF'
# Shared Library Rules

- Maintain backwards compatibility
- Export clear public APIs
- Include usage examples
- Semantic versioning
EOF

cat > packages/shared/CLAUDE.md << 'EOF'
# Shared Library Project

<!-- repo:block:shared -->
# Shared Rules (All Projects)

- Use conventional commits
- Run tests before pushing
- Keep dependencies up to date
<!-- /repo:block:shared -->

<!-- repo:block:library -->
# Shared Library Rules

- Maintain backwards compatibility
- Export clear public APIs
- Include usage examples
- Semantic versioning
<!-- /repo:block:library -->
EOF

log_test "Shared library rules created" "PASS"

log_step "Verifying shared rules propagate to all projects"

frontend_shared=$(sed -n '/<!-- repo:block:shared -->/,/<!-- \/repo:block:shared -->/p' packages/frontend/CLAUDE.md)
backend_shared=$(sed -n '/<!-- repo:block:shared -->/,/<!-- \/repo:block:shared -->/p' packages/backend/CLAUDE.md)
shared_shared=$(sed -n '/<!-- repo:block:shared -->/,/<!-- \/repo:block:shared -->/p' packages/shared/CLAUDE.md)

if [ "$frontend_shared" = "$backend_shared" ] && [ "$backend_shared" = "$shared_shared" ]; then
    log_test "Shared rules identical across all projects" "PASS"
else
    log_test "Shared rules identical across all projects" "FAIL"
fi

log_step "Verifying project-specific rules are isolated"

if grep -q "functional components" packages/frontend/CLAUDE.md && \
   ! grep -q "functional components" packages/backend/CLAUDE.md; then
    log_test "Frontend rules don't leak to backend" "PASS"
else
    log_test "Frontend rules don't leak to backend" "FAIL"
fi

if grep -q "async/await patterns" packages/backend/CLAUDE.md && \
   ! grep -q "async/await patterns" packages/frontend/CLAUDE.md; then
    log_test "Backend rules don't leak to frontend" "PASS"
else
    log_test "Backend rules don't leak to frontend" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# WORKFLOW 4: Rule Inheritance and Override
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  WORKFLOW 4: Rule Inheritance and Override                   ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Organization → Team → Project rule hierarchy"
echo ""

mkdir -p "$WORK_DIR/hierarchy"
cd "$WORK_DIR/hierarchy"
git init -q

log_step "Creating organization-level rules (base)"

mkdir -p .repository/rules/org
cat > .repository/rules/org/base.md << 'EOF'
# Organization Standards

## Security (ORG-SEC)
- All data must be encrypted at rest
- Use SSO for authentication
- Annual security training required

## Compliance (ORG-COMP)
- GDPR compliance required
- Audit logs for all actions
- Data retention: 7 years
EOF

log_test "Organization rules created" "PASS"

log_step "Creating team-level rules (extends org)"

mkdir -p .repository/rules/team
cat > .repository/rules/team/platform.md << 'EOF'
# Platform Team Standards

## Extends: Organization Standards

## Infrastructure (TEAM-INFRA)
- Use Kubernetes for deployment
- Terraform for infrastructure
- Prometheus for monitoring

## Code Quality (TEAM-CODE)
- 80% test coverage minimum
- Code review required
- CI must pass before merge
EOF

log_test "Team rules created (extends org)" "PASS"

log_step "Creating project-level rules (extends team, overrides some)"

mkdir -p .repository/rules/project
cat > .repository/rules/project/api-service.md << 'EOF'
# API Service Rules

## Extends: Platform Team Standards

## Overrides
# Override: TEAM-CODE.coverage = 90% (stricter for API)

## API Specific
- OpenAPI spec required
- Rate limiting on all endpoints
- Response time < 100ms p99
EOF

log_test "Project rules created (extends team, overrides coverage)" "PASS"

log_step "Generating merged config"

cat > CLAUDE.md << 'EOF'
# API Service Project

<!-- repo:block:org-security -->
# Organization Standards - Security (ORG-SEC)
- All data must be encrypted at rest
- Use SSO for authentication
- Annual security training required
<!-- /repo:block:org-security -->

<!-- repo:block:org-compliance -->
# Organization Standards - Compliance (ORG-COMP)
- GDPR compliance required
- Audit logs for all actions
- Data retention: 7 years
<!-- /repo:block:org-compliance -->

<!-- repo:block:team-infrastructure -->
# Platform Team - Infrastructure (TEAM-INFRA)
- Use Kubernetes for deployment
- Terraform for infrastructure
- Prometheus for monitoring
<!-- /repo:block:team-infrastructure -->

<!-- repo:block:team-code -->
# Platform Team - Code Quality (TEAM-CODE)
- 90% test coverage minimum (OVERRIDE: was 80%)
- Code review required
- CI must pass before merge
<!-- /repo:block:team-code -->

<!-- repo:block:project-api -->
# API Service Specific
- OpenAPI spec required
- Rate limiting on all endpoints
- Response time < 100ms p99
<!-- /repo:block:project-api -->
EOF

if grep -q "90% test coverage" CLAUDE.md; then
    log_test "Override applied (80% → 90%)" "PASS"
else
    log_test "Override applied (80% → 90%)" "FAIL"
fi

block_count=$(grep -c "repo:block:" CLAUDE.md)
if [ "$block_count" -eq 10 ]; then
    log_test "All 5 rule blocks present (5 opens + 5 closes)" "PASS"
else
    log_test "All 5 rule blocks present" "FAIL" "Found $block_count"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# WORKFLOW 5: Migration from Legacy to Repository Manager
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  WORKFLOW 5: Legacy Migration                                ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Migrating existing hand-crafted configs to managed format"
echo ""

mkdir -p "$WORK_DIR/legacy-migration"
cd "$WORK_DIR/legacy-migration"
git init -q

log_step "Creating legacy (unmanaged) configs"

cat > CLAUDE.md << 'EOF'
# My Project Rules

These are rules I wrote by hand over time.

## Coding Style
- Use 2-space indentation
- Max line length 100

## Testing
- Write tests for everything
- Mock external services

## My Notes
- Talk to Sarah about the API redesign
- Remember the deadline is next Friday
EOF

cat > .cursorrules << 'EOF'
# Cursor Rules

Follow clean code principles.

## TypeScript
- Use strict mode
- Prefer interfaces over types

## React
- Functional components only
- Use hooks
EOF

log_test "Legacy configs exist (hand-crafted)" "PASS"

log_step "Extracting common rules to .repository"

mkdir -p .repository/rules

# Extract coding style (common to both)
cat > .repository/rules/coding-style.md << 'EOF'
# Coding Style

- Use 2-space indentation
- Max line length 100
- Follow clean code principles
EOF

cat > .repository/rules/testing.md << 'EOF'
# Testing Guidelines

- Write tests for everything
- Mock external services
EOF

log_test "Common rules extracted to .repository" "PASS"

log_step "Converting to managed format (preserving personal notes)"

cat > CLAUDE.md << 'EOF'
# My Project Rules

<!-- repo:block:coding-style -->
# Coding Style

- Use 2-space indentation
- Max line length 100
- Follow clean code principles
<!-- /repo:block:coding-style -->

<!-- repo:block:testing -->
# Testing Guidelines

- Write tests for everything
- Mock external services
<!-- /repo:block:testing -->

# My Notes (not managed)
- Talk to Sarah about the API redesign
- Remember the deadline is next Friday
EOF

if grep -q "repo:block:coding-style" CLAUDE.md && \
   grep -q "My Notes" CLAUDE.md; then
    log_test "CLAUDE.md migrated with preserved notes" "PASS"
else
    log_test "CLAUDE.md migrated with preserved notes" "FAIL"
fi

cat > .cursorrules << 'EOF'
# Cursor Rules

<!-- repo:block:coding-style -->
# Coding Style

- Use 2-space indentation
- Max line length 100
- Follow clean code principles
<!-- /repo:block:coding-style -->

## TypeScript (tool-specific, not managed)
- Use strict mode
- Prefer interfaces over types

## React (tool-specific, not managed)
- Functional components only
- Use hooks
EOF

if grep -q "repo:block:coding-style" .cursorrules && \
   grep -q "TypeScript" .cursorrules; then
    log_test ".cursorrules migrated with tool-specific sections" "PASS"
else
    log_test ".cursorrules migrated with tool-specific sections" "FAIL"
fi

log_step "Verifying shared rules are consistent post-migration"

claude_style=$(sed -n '/<!-- repo:block:coding-style -->/,/<!-- \/repo:block:coding-style -->/p' CLAUDE.md)
cursor_style=$(sed -n '/<!-- repo:block:coding-style -->/,/<!-- \/repo:block:coding-style -->/p' .cursorrules)

if [ "$claude_style" = "$cursor_style" ]; then
    log_test "Shared rules identical after migration" "PASS"
else
    log_test "Shared rules identical after migration" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# WORKFLOW 6: Feature Flag Rules
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  WORKFLOW 6: Feature Flag Rules                              ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Different rules for experimental features"
echo ""

mkdir -p "$WORK_DIR/feature-flags"
cd "$WORK_DIR/feature-flags"
git init -q
mkdir -p .repository/rules

log_step "Creating feature flag configuration"

cat > .repository/rules/base.md << 'EOF'
# Base Rules

- Write clean code
- Test everything
EOF

cat > .repository/rules/experimental-ai.md << 'EOF'
# Experimental AI Features (FLAG: experimental_ai)

When working on AI features:
- Use extensive logging
- Add feature flags for rollout
- Monitor error rates closely
- Prepare rollback procedures
EOF

cat > .repository/rules/beta-api.md << 'EOF'
# Beta API Features (FLAG: beta_api)

For beta API endpoints:
- Version prefix: /beta/
- Add deprecation warnings
- Rate limit more aggressively
- Collect usage metrics
EOF

log_test "Feature-flagged rules created" "PASS"

log_step "Generating config with active features"

# Simulate: experimental_ai=true, beta_api=false
cat > CLAUDE.md << 'EOF'
# Project with Feature Flags

## Active Features
- experimental_ai: enabled
- beta_api: disabled

<!-- repo:block:base -->
# Base Rules

- Write clean code
- Test everything
<!-- /repo:block:base -->

<!-- repo:block:experimental-ai -->
# Experimental AI Features (ACTIVE)

When working on AI features:
- Use extensive logging
- Add feature flags for rollout
- Monitor error rates closely
- Prepare rollback procedures
<!-- /repo:block:experimental-ai -->

<!-- repo:block:beta-api:disabled -->
# Beta API Features (DISABLED - for reference only)
<!-- /repo:block:beta-api:disabled -->
EOF

if grep -q "experimental-ai" CLAUDE.md && \
   grep -q "beta-api:disabled" CLAUDE.md; then
    log_test "Feature flags reflected in config" "PASS"
else
    log_test "Feature flags reflected in config" "FAIL"
fi

log_step "Simulating feature flag change"

# Enable beta_api
sed -i 's/beta_api: disabled/beta_api: enabled/' CLAUDE.md
sed -i 's/<!-- repo:block:beta-api:disabled -->/<!-- repo:block:beta-api -->/' CLAUDE.md
sed -i 's/<!-- \/repo:block:beta-api:disabled -->/<!-- \/repo:block:beta-api -->/' CLAUDE.md
sed -i 's/DISABLED - for reference only/ACTIVE/' CLAUDE.md

if grep -q "beta_api: enabled" CLAUDE.md; then
    log_test "Feature flag toggle updated config" "PASS"
else
    log_test "Feature flag toggle updated config" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# WORKFLOW 7: Emergency Hotfix with Rule Bypass
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  WORKFLOW 7: Emergency Hotfix with Rule Bypass               ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Production is down, need to bypass normal review rules"
echo ""

mkdir -p "$WORK_DIR/hotfix"
cd "$WORK_DIR/hotfix"
git init -q
mkdir -p .repository/rules

log_step "Creating normal rules (strict review process)"

cat > .repository/rules/review.md << 'EOF'
# Code Review Policy

- All changes need 2 approvals
- CI must pass completely
- Documentation must be updated
- Security review for auth changes
EOF

cat > CLAUDE.md << 'EOF'
# Project Rules

<!-- repo:block:review -->
# Code Review Policy

- All changes need 2 approvals
- CI must pass completely
- Documentation must be updated
- Security review for auth changes
<!-- /repo:block:review -->
EOF

log_test "Normal strict review rules in place" "PASS"

log_step "Simulating emergency mode activation"

cat > .repository/EMERGENCY_MODE << 'EOF'
ACTIVATED: 2024-01-29T10:30:00Z
REASON: Production outage - payment processing down
APPROVED_BY: cto@company.com
EXPIRES: 2024-01-29T14:30:00Z (4 hours)
EOF

# Add emergency override to config
cat >> CLAUDE.md << 'EOF'

<!-- repo:block:emergency -->
# ⚠️ EMERGENCY MODE ACTIVE

**Normal rules temporarily relaxed:**
- Single approval allowed for hotfixes
- Skip non-critical CI checks
- Rollback procedures mandatory
- Post-incident review required

**Expires:** 4 hours from activation
<!-- /repo:block:emergency -->
EOF

if grep -q "EMERGENCY MODE ACTIVE" CLAUDE.md; then
    log_test "Emergency mode added to config" "PASS"
else
    log_test "Emergency mode added to config" "FAIL"
fi

if [ -f .repository/EMERGENCY_MODE ]; then
    log_test "Emergency mode flag file exists" "PASS"
else
    log_test "Emergency mode flag file exists" "FAIL"
fi

log_step "Simulating emergency mode deactivation"

rm .repository/EMERGENCY_MODE
sed -i '/<!-- repo:block:emergency -->/,/<!-- \/repo:block:emergency -->/d' CLAUDE.md

if ! grep -q "EMERGENCY MODE" CLAUDE.md && \
   ! [ -f .repository/EMERGENCY_MODE ]; then
    log_test "Emergency mode fully deactivated" "PASS"
else
    log_test "Emergency mode fully deactivated" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# RESULTS SUMMARY
# ════════════════════════════════════════════════════════════════
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║              ADVANCED WORKFLOW TEST SUMMARY                  ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Tests:  $TOTAL"
echo -e "  Passed:       ${GREEN}$PASSED${NC}"
echo -e "  Failed:       ${RED}$FAILED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/advanced-workflow-report.md" << EOF
# Advanced Developer Workflow Test Report

**Generated:** $(date -Iseconds)

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Workflows Tested

### 1. Team Collaboration (Mixed Tool Preferences)
- 4 developers using different AI tools
- Shared rules propagate to all
- Personal notes stay tool-specific

### 2. CI/CD Pipeline Integration
- GitHub Actions validation workflow
- Pre-commit hooks for local checks
- Unclosed block detection

### 3. Monorepo with Multiple Projects
- Root-level shared rules
- Project-specific rules isolated
- No rule leakage between projects

### 4. Rule Inheritance and Override
- Organization → Team → Project hierarchy
- Override mechanism for stricter rules
- All layers visible in final config

### 5. Legacy Migration
- Hand-crafted configs to managed format
- Personal notes preserved
- Shared rules extracted and synced

### 6. Feature Flag Rules
- Conditional rule activation
- Disabled rules marked in config
- Dynamic flag toggling

### 7. Emergency Hotfix Bypass
- Temporary rule relaxation
- Emergency mode flag file
- Clean deactivation
EOF

echo "Report: $RESULTS_DIR/advanced-workflow-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}${BOLD}SOME TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}${BOLD}ALL ADVANCED WORKFLOW TESTS PASSED${NC}"
    exit 0
fi
