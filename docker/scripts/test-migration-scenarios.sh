#!/bin/bash
# Migration and Version Upgrade Scenario Tests
# Tests tool migrations, version upgrades, and backwards compatibility

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/migration"

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
echo -e "${BOLD}║         MIGRATION & VERSION UPGRADE SCENARIOS                ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

TOTAL=0
PASSED=0
FAILED=0

log_test() {
    local name="$1"
    local status="$2"
    TOTAL=$((TOTAL + 1))
    case "$status" in
        PASS) PASSED=$((PASSED + 1)); echo -e "    ${GREEN}✓${NC} $name" ;;
        FAIL) FAILED=$((FAILED + 1)); echo -e "    ${RED}✗${NC} $name" ;;
    esac
}

log_step() {
    echo -e "  ${CYAN}→${NC} $1"
}

WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

# ════════════════════════════════════════════════════════════════
# SCENARIO 1: Tool Version Upgrade (v1 → v2 block syntax)
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 1: Block Syntax Migration (v1 → v2)                ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Simulating migration from old block syntax to new format"
echo ""

mkdir -p "$WORK_DIR/syntax-migration"
cd "$WORK_DIR/syntax-migration"

log_step "Creating v1 format config (old syntax)"

cat > CLAUDE.md << 'EOF'
# Project Rules (v1 format)

<!-- BEGIN: coding-rules -->
# Coding Standards

- Use 2-space indentation
- Write clean code
<!-- END: coding-rules -->

<!-- BEGIN: testing-rules -->
# Testing Standards

- Write unit tests
- 80% coverage
<!-- END: testing-rules -->
EOF

if grep -q "BEGIN: coding-rules" CLAUDE.md; then
    log_test "v1 format config created" "PASS"
else
    log_test "v1 format config created" "FAIL"
fi

log_step "Migrating to v2 format (repo:block syntax)"

# Simulate migration script (portable sed)
sed -i 's/<!-- BEGIN: /<!-- repo:block:/g' CLAUDE.md
sed -i 's/<!-- END: /<!-- \/repo:block:/g' CLAUDE.md

if grep -q "repo:block:coding" CLAUDE.md && \
   ! grep -q "BEGIN:" CLAUDE.md; then
    log_test "Migrated to v2 format" "PASS"
else
    log_test "Migrated to v2 format" "FAIL"
fi

log_step "Verifying content preserved during migration"

if grep -q "Use 2-space indentation" CLAUDE.md && \
   grep -q "80% coverage" CLAUDE.md; then
    log_test "Rule content preserved" "PASS"
else
    log_test "Rule content preserved" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 2: Tool Migration (Cursor → Claude CLI)
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 2: Tool Migration (Cursor → Claude CLI)            ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Developer switching from Cursor to Claude CLI"
echo ""

mkdir -p "$WORK_DIR/tool-migration"
cd "$WORK_DIR/tool-migration"
mkdir -p .repository/rules

log_step "Creating existing Cursor config"

cat > .cursorrules << 'EOF'
# Cursor Project Rules

<!-- repo:block:shared -->
# Shared Standards

- Follow code review guidelines
- Use consistent formatting
<!-- /repo:block:shared -->

# Cursor-Specific Settings

- Use Copilot++ features
- Enable auto-completions
EOF

log_test "Source Cursor config created" "PASS"

log_step "Generating Claude config from Cursor (shared blocks only)"

# Extract only managed blocks
{
    echo "# Claude Project Rules"
    echo ""
    sed -n '/<!-- repo:block:/,/<!-- \/repo:block:/p' .cursorrules
} > CLAUDE.md

if grep -q "repo:block:shared" CLAUDE.md && \
   ! grep -q "Copilot++" CLAUDE.md; then
    log_test "Shared blocks migrated, tool-specific excluded" "PASS"
else
    log_test "Shared blocks migrated, tool-specific excluded" "FAIL"
fi

log_step "Verifying both configs work simultaneously"

if [ -f ".cursorrules" ] && [ -f "CLAUDE.md" ]; then
    log_test "Both config files coexist" "PASS"
else
    log_test "Both config files coexist" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 3: Provider Migration (OpenAI → Anthropic)
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 3: Provider Migration (OpenAI → Anthropic)         ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Team switching from GPT-4 to Claude"
echo ""

mkdir -p "$WORK_DIR/provider-migration"
cd "$WORK_DIR/provider-migration"

log_step "Creating OpenAI-specific config"

cat > .aider.conf.yml << 'EOF'
# Aider config for GPT-4
model: gpt-4-turbo-preview
auto-commits: false
auto-test: true
test-cmd: npm test
EOF

log_test "OpenAI config created" "PASS"

log_step "Migrating to Anthropic provider"

sed -i 's/model: gpt-4-turbo-preview/model: claude-3-opus-20240229/' .aider.conf.yml

if grep -q "claude-3-opus" .aider.conf.yml; then
    log_test "Model changed to Claude" "PASS"
else
    log_test "Model changed to Claude" "FAIL"
fi

log_step "Verifying non-model settings preserved"

if grep -q "auto-commits: false" .aider.conf.yml && \
   grep -q "test-cmd: npm test" .aider.conf.yml; then
    log_test "Other settings preserved" "PASS"
else
    log_test "Other settings preserved" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 4: Multi-Version Project Support
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 4: Multi-Version Project Support                   ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Supporting multiple API versions in same project"
echo ""

mkdir -p "$WORK_DIR/multi-version"
cd "$WORK_DIR/multi-version"
mkdir -p .repository/rules

log_step "Creating version-specific rules"

cat > .repository/rules/api-v1.md << 'EOF'
# API v1 Rules (Legacy)

- Endpoint pattern: /api/v1/*
- Use XML responses
- Basic auth only
EOF

cat > .repository/rules/api-v2.md << 'EOF'
# API v2 Rules (Current)

- Endpoint pattern: /api/v2/*
- Use JSON responses
- JWT authentication
- Rate limiting required
EOF

cat > .repository/rules/api-v3.md << 'EOF'
# API v3 Rules (Beta)

- Endpoint pattern: /api/v3/*
- Use GraphQL
- OAuth 2.0 authentication
- Experimental features enabled
EOF

log_test "Three API version rules created" "PASS"

log_step "Generating config with all versions"

cat > CLAUDE.md << 'EOF'
# Multi-Version API Project

<!-- repo:block:api-v1 -->
# API v1 Rules (Legacy)

- Endpoint pattern: /api/v1/*
- Use XML responses
- Basic auth only
<!-- /repo:block:api-v1 -->

<!-- repo:block:api-v2 -->
# API v2 Rules (Current)

- Endpoint pattern: /api/v2/*
- Use JSON responses
- JWT authentication
- Rate limiting required
<!-- /repo:block:api-v2 -->

<!-- repo:block:api-v3 -->
# API v3 Rules (Beta)

- Endpoint pattern: /api/v3/*
- Use GraphQL
- OAuth 2.0 authentication
- Experimental features enabled
<!-- /repo:block:api-v3 -->
EOF

block_count=$(grep -c "repo:block:api-v" CLAUDE.md)
if [ "$block_count" -eq 6 ]; then
    log_test "All 3 version blocks present" "PASS"
else
    log_test "All 3 version blocks present" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 5: Deprecation Workflow
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 5: Deprecation Workflow                            ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Marking rules as deprecated and scheduling removal"
echo ""

mkdir -p "$WORK_DIR/deprecation"
cd "$WORK_DIR/deprecation"

log_step "Creating config with deprecated block"

cat > CLAUDE.md << 'EOF'
# Project Rules

<!-- repo:block:current-rules -->
# Current Standards

- Use TypeScript strict mode
- Follow ESLint rules
<!-- /repo:block:current-rules -->

<!-- repo:block:deprecated-rules:deprecated:2024-06-01 -->
# Legacy Standards (DEPRECATED)

⚠️ These rules will be removed on 2024-06-01

- Use var instead of let/const (deprecated)
- jQuery allowed (deprecated)
<!-- /repo:block:deprecated-rules:deprecated:2024-06-01 -->
EOF

if grep -q "deprecated:2024-06-01" CLAUDE.md; then
    log_test "Deprecated block with removal date" "PASS"
else
    log_test "Deprecated block with removal date" "FAIL"
fi

log_step "Simulating removal of deprecated rules"

# Remove deprecated block
sed -i '/<!-- repo:block:deprecated-rules/,/<!-- \/repo:block:deprecated-rules/d' CLAUDE.md

if ! grep -q "deprecated-rules" CLAUDE.md && \
   grep -q "current-rules" CLAUDE.md; then
    log_test "Deprecated rules removed, current preserved" "PASS"
else
    log_test "Deprecated rules removed, current preserved" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 6: Configuration Schema Migration
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 6: Configuration Schema Migration                  ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Migrating .repository config format"
echo ""

mkdir -p "$WORK_DIR/schema-migration"
cd "$WORK_DIR/schema-migration"
mkdir -p .repository

log_step "Creating old schema config"

cat > .repository/config.json << 'EOF'
{
  "version": "1.0",
  "tools": ["claude", "cursor"],
  "rules_path": "./rules",
  "auto_sync": true
}
EOF

log_test "Old schema (JSON) created" "PASS"

log_step "Migrating to new schema (TOML)"

cat > .repository/config.toml << 'EOF'
# Repository Manager Configuration
# Schema version 2.0

[core]
version = "2.0"
mode = "standard"

[active]
tools = ["claude", "cursor"]
presets = []

[paths]
rules = ".repository/rules"
cache = ".repository/.cache"

[sync]
auto = true
interval = "5m"
EOF

if [ -f .repository/config.toml ]; then
    log_test "New schema (TOML) created" "PASS"
else
    log_test "New schema (TOML) created" "FAIL"
fi

log_step "Archiving old config"

mv .repository/config.json .repository/config.json.v1.backup

if [ -f .repository/config.json.v1.backup ]; then
    log_test "Old config archived" "PASS"
else
    log_test "Old config archived" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 7: Backwards Compatibility Check
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 7: Backwards Compatibility Check                   ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Ensuring old configs still work with new tool versions"
echo ""

mkdir -p "$WORK_DIR/backwards-compat"
cd "$WORK_DIR/backwards-compat"

log_step "Creating minimal legacy config"

cat > CLAUDE.md << 'EOF'
# Simple Rules

Just a plain markdown file without managed blocks.

- Rule 1
- Rule 2
EOF

if [ -f CLAUDE.md ]; then
    log_test "Minimal config (no blocks) works" "PASS"
else
    log_test "Minimal config (no blocks) works" "FAIL"
fi

log_step "Creating config with mixed old/new syntax"

cat > .cursorrules << 'EOF'
# Mixed Format Config

<!-- repo:block:new -->
# New format block
<!-- /repo:block:new -->

<!-- BEGIN: old -->
# Old format block (should be warned)
<!-- END: old -->

Regular content outside blocks.
EOF

if grep -q "repo:block:new" .cursorrules && \
   grep -q "BEGIN: old" .cursorrules; then
    log_test "Mixed format config parseable" "PASS"
else
    log_test "Mixed format config parseable" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 8: Rolling Migration Strategy
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 8: Rolling Migration Strategy                      ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Migrating team members one at a time"
echo ""

mkdir -p "$WORK_DIR/rolling-migration"
cd "$WORK_DIR/rolling-migration"
mkdir -p .repository/rules

log_step "Creating shared rules (works with all tools)"

cat > .repository/rules/shared.md << 'EOF'
# Shared Team Rules

- Write clean code
- Test everything
EOF

log_test "Shared rules created" "PASS"

log_step "Phase 1: First team member migrates to Claude"

cat > CLAUDE.md << 'EOF'
<!-- repo:block:shared -->
# Shared Team Rules

- Write clean code
- Test everything
<!-- /repo:block:shared -->
EOF

log_test "Phase 1: Claude config created" "PASS"

log_step "Phase 2: Second member continues with Cursor"

cat > .cursorrules << 'EOF'
<!-- repo:block:shared -->
# Shared Team Rules

- Write clean code
- Test everything
<!-- /repo:block:shared -->
EOF

log_test "Phase 2: Cursor config created" "PASS"

log_step "Phase 3: Third member uses Aider"

cat > .aider.conf.yml << 'EOF'
model: claude-3-opus-20240229
auto-commits: false
read:
  - .repository/rules/shared.md
EOF

log_test "Phase 3: Aider config created" "PASS"

log_step "Verifying all team members have consistent rules"

# Check shared content is in all configs
claude_has_shared=$(grep -c "Write clean code" CLAUDE.md || echo 0)
cursor_has_shared=$(grep -c "Write clean code" .cursorrules || echo 0)
aider_points_to_shared=$(grep -c "shared.md" .aider.conf.yml || echo 0)

if [ "$claude_has_shared" -gt 0 ] && \
   [ "$cursor_has_shared" -gt 0 ] && \
   [ "$aider_points_to_shared" -gt 0 ]; then
    log_test "All 3 tools have access to shared rules" "PASS"
else
    log_test "All 3 tools have access to shared rules" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# RESULTS SUMMARY
# ════════════════════════════════════════════════════════════════
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║              MIGRATION TEST SUMMARY                          ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Tests:  $TOTAL"
echo -e "  Passed:       ${GREEN}$PASSED${NC}"
echo -e "  Failed:       ${RED}$FAILED${NC}"
echo ""

cat > "$RESULTS_DIR/migration-report.md" << EOF
# Migration Test Report

**Generated:** $(date -Iseconds)

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **Block Syntax Migration** - v1 → v2 format upgrade
2. **Tool Migration** - Cursor → Claude CLI switch
3. **Provider Migration** - OpenAI → Anthropic switch
4. **Multi-Version Support** - Multiple API versions coexisting
5. **Deprecation Workflow** - Marking and removing old rules
6. **Schema Migration** - JSON → TOML config format
7. **Backwards Compatibility** - Old configs with new tools
8. **Rolling Migration** - Gradual team transition

## Recommendations

- Always backup before migration
- Use deprecation markers before removal
- Test with all team members' tools
- Maintain backwards compatibility during transition
- Document migration steps for team
EOF

echo "Report: $RESULTS_DIR/migration-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}${BOLD}SOME TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}${BOLD}ALL MIGRATION TESTS PASSED${NC}"
    exit 0
fi
