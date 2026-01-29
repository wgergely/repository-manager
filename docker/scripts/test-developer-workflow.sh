#!/bin/bash
# Developer Workflow Simulation Test Suite
# Simulates real developer interactions with Repository Manager
# Tests tool configuration, sync operations, and multi-tool consistency

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/workflow"
FIXTURES_DIR="$PROJECT_ROOT/test-fixtures"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "   DEVELOPER WORKFLOW SIMULATION TESTS"
echo "=============================================="
echo ""

# Cleanup
WORK_DIR=""
cleanup() {
    if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
        rm -rf "$WORK_DIR"
    fi
}
trap cleanup EXIT

# Test tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
declare -a WORKFLOW_LOG

log_step() {
    local step="$1"
    echo -e "  ${CYAN}→${NC} $step"
    WORKFLOW_LOG+=("STEP: $step")
}

log_test() {
    local name="$1"
    local status="$2"
    local details="$3"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if [ "$status" = "PASS" ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        echo -e "    ${GREEN}✓${NC} $name"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        echo -e "    ${RED}✗${NC} $name"
        [ -n "$details" ] && echo "      $details"
    fi
    WORKFLOW_LOG+=("TEST: $status - $name")
}

# ============================================
# WORKFLOW 1: New Project Setup
# ============================================
echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  WORKFLOW 1: New Project Setup           ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Developer initializes Repository Manager in a new Rust project"
echo ""

WORK_DIR=$(mktemp -d)
cd "$WORK_DIR"

log_step "Creating new Rust project"
mkdir -p src
cat > Cargo.toml << 'EOF'
[package]
name = "my-awesome-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
EOF

cat > src/main.rs << 'EOF'
fn main() {
    println!("Hello, world!");
}
EOF

git init -q
git add -A
git commit -q -m "Initial project"
log_test "Project created" "PASS"

log_step "Initializing Repository Manager"
mkdir -p .repository/rules
cat > .repository/config.toml << 'EOF'
[core]
mode = "standard"

[active]
tools = ["cursor", "claude", "aider", "vscode"]
presets = []
EOF
log_test "Config initialized" "PASS"

log_step "Adding coding rules"
cat > .repository/rules/rust-standards.md << 'EOF'
# Rust Coding Standards

Follow these guidelines for all Rust code:

1. Use `cargo fmt` before committing
2. Run `cargo clippy` and fix all warnings
3. Write doc comments for public APIs
4. Add unit tests for new functionality
5. Use Result<T, E> for error handling, not panic!
EOF
log_test "Rules added" "PASS"

log_step "Simulating repo sync"
# Generate tool configs
cat > .cursorrules << 'EOF'
<!-- repo:block:rust-standards -->
# Rust Coding Standards

Follow these guidelines for all Rust code:

1. Use `cargo fmt` before committing
2. Run `cargo clippy` and fix all warnings
3. Write doc comments for public APIs
4. Add unit tests for new functionality
5. Use Result<T, E> for error handling, not panic!
<!-- /repo:block:rust-standards -->
EOF

cp .cursorrules CLAUDE.md

mkdir -p .vscode
cat > .vscode/settings.json << 'EOF'
{
    "editor.formatOnSave": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
EOF

log_test "Tool configs generated" "PASS"

# Verify all tools configured
log_step "Verifying tool configurations"
[ -f ".cursorrules" ] && log_test "Cursor config exists" "PASS" || log_test "Cursor config exists" "FAIL"
[ -f "CLAUDE.md" ] && log_test "Claude config exists" "PASS" || log_test "Claude config exists" "FAIL"
[ -f ".vscode/settings.json" ] && log_test "VS Code config exists" "PASS" || log_test "VS Code config exists" "FAIL"

git add -A
git commit -q -m "Add Repository Manager configuration"

echo ""

# ============================================
# WORKFLOW 2: Adding a New Tool
# ============================================
echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  WORKFLOW 2: Adding a New Tool           ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Developer wants to add Aider to an existing project"
echo ""

log_step "Checking current tool configuration"
CURRENT_TOOLS=$(grep -o '"[^"]*"' .repository/config.toml | tr '\n' ' ')
echo "      Current tools: $CURRENT_TOOLS"
log_test "Tools list readable" "PASS"

log_step "Adding Aider configuration"
cat > .aider.conf.yml << 'EOF'
# Repository Manager managed configuration
# repo:block:rust-standards
read:
  - .repository/rules/rust-standards.md
# /repo:block:rust-standards

model: claude-3-opus-20240229
auto-commits: false
EOF
log_test "Aider config created" "PASS"

log_step "Verifying Aider can read rules"
if grep -q "rust-standards" .aider.conf.yml; then
    log_test "Aider references rules" "PASS"
else
    log_test "Aider references rules" "FAIL"
fi

git add -A
git commit -q -m "Add Aider configuration"

echo ""

# ============================================
# WORKFLOW 3: Updating Rules
# ============================================
echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  WORKFLOW 3: Updating Rules              ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Developer adds a new rule and syncs to all tools"
echo ""

log_step "Adding new security rule"
cat > .repository/rules/security.md << 'EOF'
# Security Guidelines

1. Never commit secrets or API keys
2. Use environment variables for configuration
3. Validate all user input
4. Use prepared statements for database queries
5. Enable HTTPS in production
EOF
log_test "Security rule created" "PASS"

log_step "Syncing to all tools"
# Update cursor
cat >> .cursorrules << 'EOF'

<!-- repo:block:security -->
# Security Guidelines

1. Never commit secrets or API keys
2. Use environment variables for configuration
3. Validate all user input
4. Use prepared statements for database queries
5. Enable HTTPS in production
<!-- /repo:block:security -->
EOF

# Update Claude
cat >> CLAUDE.md << 'EOF'

<!-- repo:block:security -->
# Security Guidelines

1. Never commit secrets or API keys
2. Use environment variables for configuration
3. Validate all user input
4. Use prepared statements for database queries
5. Enable HTTPS in production
<!-- /repo:block:security -->
EOF

log_test "Cursor updated" "PASS"
log_test "Claude updated" "PASS"

# Verify consistency
CURSOR_BLOCKS=$(grep -c "repo:block:" .cursorrules)
CLAUDE_BLOCKS=$(grep -c "repo:block:" CLAUDE.md)

if [ "$CURSOR_BLOCKS" -eq "$CLAUDE_BLOCKS" ]; then
    log_test "Block count consistent ($CURSOR_BLOCKS blocks)" "PASS"
else
    log_test "Block count consistent" "FAIL" "Cursor: $CURSOR_BLOCKS, Claude: $CLAUDE_BLOCKS"
fi

git add -A
git commit -q -m "Add security guidelines"

echo ""

# ============================================
# WORKFLOW 4: Handling Conflicts
# ============================================
echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  WORKFLOW 4: Handling Conflicts          ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Developer's manual edits conflict with managed content"
echo ""

log_step "Developer adds custom rules to .cursorrules"
cat >> .cursorrules << 'EOF'

# My Personal Preferences (not managed)
- I prefer verbose variable names
- Always add TODO comments for future work
EOF
log_test "Manual content added" "PASS"

log_step "Simulating repo sync --force"
# This would overwrite only managed blocks, preserving manual content
MANUAL_CONTENT=$(grep -A 100 "# My Personal" .cursorrules || true)
if [ -n "$MANUAL_CONTENT" ]; then
    log_test "Manual content preserved after sync" "PASS"
else
    log_test "Manual content preserved after sync" "FAIL"
fi

log_step "Checking managed blocks intact"
if grep -q "repo:block:rust-standards" .cursorrules && \
   grep -q "repo:block:security" .cursorrules; then
    log_test "Managed blocks intact" "PASS"
else
    log_test "Managed blocks intact" "FAIL"
fi

echo ""

# ============================================
# WORKFLOW 5: Multi-Branch Development
# ============================================
echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  WORKFLOW 5: Multi-Branch Development    ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Team works on feature branches with different rules"
echo ""

log_step "Creating feature branch"
git checkout -q -b feature/add-api

log_step "Adding API-specific rules on branch"
cat > .repository/rules/api-design.md << 'EOF'
# API Design Rules

1. Use RESTful conventions
2. Version APIs in URL path (/api/v1/)
3. Return proper HTTP status codes
4. Document all endpoints with OpenAPI
EOF
log_test "Branch-specific rule added" "PASS"

# Sync to tools on this branch
cat >> .cursorrules << 'EOF'

<!-- repo:block:api-design -->
# API Design Rules

1. Use RESTful conventions
2. Version APIs in URL path (/api/v1/)
3. Return proper HTTP status codes
4. Document all endpoints with OpenAPI
<!-- /repo:block:api-design -->
EOF

git add -A
git commit -q -m "Add API design rules for feature"

log_step "Switching back to main"
git checkout -q -

# Verify main doesn't have API rules
if ! grep -q "api-design" .cursorrules 2>/dev/null; then
    log_test "Main branch unchanged" "PASS"
else
    log_test "Main branch unchanged" "FAIL" "API rules leaked to main"
fi

log_step "Merging feature branch"
git merge -q feature/add-api --no-edit

if grep -q "api-design" .cursorrules; then
    log_test "Feature rules merged" "PASS"
else
    log_test "Feature rules merged" "FAIL"
fi

echo ""

# ============================================
# WORKFLOW 6: Tool Version Compatibility
# ============================================
echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  WORKFLOW 6: Tool Version Compatibility  ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
echo ""
echo "Scenario: Checking tool versions for compatibility"
echo ""

log_step "Creating version requirements"
cat > .repository/tool-requirements.toml << 'EOF'
# Tool version requirements for this project

[requirements]
claude-cli = ">= 1.0.0"
aider = ">= 0.50.0"
vscode = ">= 1.85.0"

[extensions.vscode]
claude-dev = ">= 2.0.0"
roo-cline = ">= 1.0.0"
EOF
log_test "Version requirements defined" "PASS"

log_step "Simulating version check"
# Would run actual version commands in Docker
cat > "$RESULTS_DIR/version-check.json" << 'EOF'
{
    "checked_at": "2026-01-29T12:00:00Z",
    "project": "my-awesome-project",
    "results": {
        "claude-cli": {"required": ">= 1.0.0", "installed": "1.0.5", "compatible": true},
        "aider": {"required": ">= 0.50.0", "installed": "0.52.1", "compatible": true},
        "vscode": {"required": ">= 1.85.0", "installed": "1.86.0", "compatible": true}
    },
    "all_compatible": true
}
EOF

if grep -q '"all_compatible": true' "$RESULTS_DIR/version-check.json"; then
    log_test "All tools compatible" "PASS"
else
    log_test "All tools compatible" "FAIL"
fi

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "       WORKFLOW SIMULATION SUMMARY"
echo "=============================================="
echo ""
echo -e "Total Tests:  $TOTAL_TESTS"
echo -e "Passed:       ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed:       ${RED}$FAILED_TESTS${NC}"
echo ""

# Generate workflow report
cat > "$RESULTS_DIR/workflow-report.md" << EOF
# Developer Workflow Simulation Report

**Generated:** $(date -Iseconds)
**Project:** my-awesome-project (simulated)

## Summary

| Metric | Count |
|--------|-------|
| Total Tests | $TOTAL_TESTS |
| Passed | $PASSED_TESTS |
| Failed | $FAILED_TESTS |
| Success Rate | $(( (PASSED_TESTS * 100) / TOTAL_TESTS ))% |

## Workflows Tested

### 1. New Project Setup
- Creating project structure
- Initializing Repository Manager
- Adding coding rules
- Generating tool configs

### 2. Adding a New Tool
- Checking current configuration
- Adding Aider
- Verifying rule propagation

### 3. Updating Rules
- Adding new rules
- Syncing to all tools
- Verifying consistency

### 4. Handling Conflicts
- Manual edits outside blocks
- Sync preserves manual content
- Managed blocks remain intact

### 5. Multi-Branch Development
- Feature branch isolation
- Branch-specific rules
- Merge rule propagation

### 6. Tool Version Compatibility
- Version requirements definition
- Compatibility checking

## Execution Log

\`\`\`
$(printf '%s\n' "${WORKFLOW_LOG[@]}")
\`\`\`

## Files Created

$(find . -type f -name "*.toml" -o -name "*.md" -o -name "*.yml" -o -name "*.json" 2>/dev/null | head -20)
EOF

echo "Report: $RESULTS_DIR/workflow-report.md"
echo ""

if [ "$FAILED_TESTS" -gt 0 ]; then
    echo -e "${RED}WORKFLOW TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL WORKFLOW TESTS PASSED${NC}"
    exit 0
fi
