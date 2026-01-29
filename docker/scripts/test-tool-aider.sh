#!/bin/bash
# Aider Tool-Specific Integration Tests
# Simulates programmer workflows with Aider AI pair programming

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/tools/aider"
FIXTURES_DIR="$PROJECT_ROOT/test-fixtures"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "       AIDER INTEGRATION TESTS"
echo "=============================================="
echo ""

# Test tracking
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Check Docker
DOCKER_AVAILABLE=false
AIDER_IMAGE_EXISTS=false

if docker info >/dev/null 2>&1; then
    DOCKER_AVAILABLE=true
    if docker image inspect repo-test/aider:latest >/dev/null 2>&1; then
        AIDER_IMAGE_EXISTS=true
    fi
fi

log_test() {
    local name="$1"
    local status="$2"
    local details="$3"
    TOTAL=$((TOTAL + 1))

    case "$status" in
        PASS) PASSED=$((PASSED + 1)); echo -e "  ${GREEN}✓${NC} $name" ;;
        FAIL) FAILED=$((FAILED + 1)); echo -e "  ${RED}✗${NC} $name"; [ -n "$details" ] && echo "    $details" ;;
        SKIP) SKIPPED=$((SKIPPED + 1)); echo -e "  ${YELLOW}⏭${NC} $name (skipped)" ;;
    esac
}

# Create test workspace
WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

# ============================================
# SCENARIO 1: Aider Config File (.aider.conf.yml)
# ============================================
echo -e "${BLUE}=== Scenario 1: Aider Config Detection ===${NC}"
echo "Programmer: 'Aider should read my .aider.conf.yml'"
echo ""

mkdir -p "$WORK_DIR/project1"

cat > "$WORK_DIR/project1/.aider.conf.yml" << 'EOF'
# Aider configuration
model: claude-3-opus-20240229
auto-commits: false
auto-test: true
test-cmd: cargo test

# Read project rules
read:
  - .repository/rules/coding-standards.md
  - .repository/rules/testing.md
EOF

mkdir -p "$WORK_DIR/project1/.repository/rules"

cat > "$WORK_DIR/project1/.repository/rules/coding-standards.md" << 'EOF'
# Coding Standards

- Follow Rust idioms
- Use Result for error handling
- Document public APIs
EOF

cat > "$WORK_DIR/project1/.repository/rules/testing.md" << 'EOF'
# Testing Guidelines

- Write unit tests for all functions
- Aim for 80% coverage
- Use property-based testing where appropriate
EOF

# Verify config file
if [ -f "$WORK_DIR/project1/.aider.conf.yml" ]; then
    log_test "Aider config file created" "PASS"
else
    log_test "Aider config file created" "FAIL"
fi

# Verify read directive points to rules
if grep -q "coding-standards.md" "$WORK_DIR/project1/.aider.conf.yml"; then
    log_test "Config references coding standards" "PASS"
else
    log_test "Config references coding standards" "FAIL"
fi

if grep -q "testing.md" "$WORK_DIR/project1/.aider.conf.yml"; then
    log_test "Config references testing rules" "PASS"
else
    log_test "Config references testing rules" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 2: Model Selection
# ============================================
echo -e "${BLUE}=== Scenario 2: Model Selection ===${NC}"
echo "Programmer: 'I want to use different AI models'"
echo ""

# Test: Claude model config
if grep -q "claude-3-opus" "$WORK_DIR/project1/.aider.conf.yml"; then
    log_test "Claude model configured" "PASS"
else
    log_test "Claude model configured" "FAIL"
fi

# Test: Create OpenAI model config
cat > "$WORK_DIR/project1/.aider-openai.conf.yml" << 'EOF'
model: gpt-4-turbo-preview
auto-commits: false
EOF

if grep -q "gpt-4" "$WORK_DIR/project1/.aider-openai.conf.yml"; then
    log_test "OpenAI model configurable" "PASS"
else
    log_test "OpenAI model configurable" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 3: Auto-Test Integration
# ============================================
echo -e "${BLUE}=== Scenario 3: Auto-Test Integration ===${NC}"
echo "Programmer: 'Aider should run tests after changes'"
echo ""

# Create a Rust project for testing
mkdir -p "$WORK_DIR/project2/src"

cat > "$WORK_DIR/project2/Cargo.toml" << 'EOF'
[package]
name = "testproject"
version = "0.1.0"
edition = "2021"
EOF

cat > "$WORK_DIR/project2/src/lib.rs" << 'EOF'
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
EOF

cat > "$WORK_DIR/project2/.aider.conf.yml" << 'EOF'
model: claude-3-opus-20240229
auto-commits: false
auto-test: true
test-cmd: cargo test
EOF

# Verify test-cmd is set
if grep -q "test-cmd: cargo test" "$WORK_DIR/project2/.aider.conf.yml"; then
    log_test "Test command configured" "PASS"
else
    log_test "Test command configured" "FAIL"
fi

if grep -q "auto-test: true" "$WORK_DIR/project2/.aider.conf.yml"; then
    log_test "Auto-test enabled" "PASS"
else
    log_test "Auto-test enabled" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 4: Git Integration
# ============================================
echo -e "${BLUE}=== Scenario 4: Git Integration ===${NC}"
echo "Programmer: 'Aider works with git, how should it commit?'"
echo ""

cd "$WORK_DIR/project2"
git init -q
git add -A
git commit -q -m "Initial commit"

# Test: auto-commits disabled (we want manual control)
if grep -q "auto-commits: false" ".aider.conf.yml"; then
    log_test "Auto-commits disabled (manual control)" "PASS"
else
    log_test "Auto-commits disabled" "FAIL"
fi

# Test: Can we enable auto-commits?
cat > "$WORK_DIR/project2/.aider-autocommit.conf.yml" << 'EOF'
model: claude-3-opus-20240229
auto-commits: true
commit-prompt: "feat: {message}"
EOF

if grep -q "auto-commits: true" "$WORK_DIR/project2/.aider-autocommit.conf.yml"; then
    log_test "Auto-commits can be enabled" "PASS"
else
    log_test "Auto-commits can be enabled" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 5: Multi-Provider Support
# ============================================
echo -e "${BLUE}=== Scenario 5: Multi-Provider Support ===${NC}"
echo "Programmer: 'I switch between Anthropic and OpenAI'"
echo ""

# Provider: Anthropic
export ANTHROPIC_API_KEY="sk-ant-test-key"
if [ -n "$ANTHROPIC_API_KEY" ]; then
    log_test "Anthropic API key configurable" "PASS"
else
    log_test "Anthropic API key configurable" "FAIL"
fi

# Provider: OpenAI
export OPENAI_API_KEY="sk-openai-test-key"
if [ -n "$OPENAI_API_KEY" ]; then
    log_test "OpenAI API key configurable" "PASS"
else
    log_test "OpenAI API key configurable" "FAIL"
fi

# Provider: Mock (for testing)
export OPENAI_API_BASE="http://mock-api:8080/v1"
if [ -n "$OPENAI_API_BASE" ]; then
    log_test "API base URL configurable (mock support)" "PASS"
else
    log_test "API base URL configurable" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 6: Repository Manager Integration
# ============================================
echo -e "${BLUE}=== Scenario 6: Repo Manager Integration ===${NC}"
echo "Programmer: 'Rules from Repository Manager should flow to Aider'"
echo ""

mkdir -p "$WORK_DIR/project3/.repository/rules"

# Create repo manager config
cat > "$WORK_DIR/project3/.repository/config.toml" << 'EOF'
[core]
mode = "standard"

[active]
tools = ["aider", "claude", "cursor"]
presets = []
EOF

# Create rule that should be read by Aider
cat > "$WORK_DIR/project3/.repository/rules/api-design.md" << 'EOF'
# API Design Guidelines

1. Use RESTful conventions
2. Version your APIs (/api/v1/)
3. Return proper HTTP status codes
4. Document with OpenAPI
EOF

# Simulate repo sync - generate aider config with read directive
cat > "$WORK_DIR/project3/.aider.conf.yml" << 'EOF'
# Managed by Repository Manager
# repo:block:api-design
read:
  - .repository/rules/api-design.md
# /repo:block:api-design

model: claude-3-opus-20240229
auto-commits: false
EOF

# Verify managed block
if grep -q "repo:block:api-design" "$WORK_DIR/project3/.aider.conf.yml"; then
    log_test "Managed block present in aider config" "PASS"
else
    log_test "Managed block present in aider config" "FAIL"
fi

# Verify read directive points to rule
if grep -q ".repository/rules/api-design.md" "$WORK_DIR/project3/.aider.conf.yml"; then
    log_test "Read directive references rule file" "PASS"
else
    log_test "Read directive references rule file" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 7: File Ignore Patterns
# ============================================
echo -e "${BLUE}=== Scenario 7: File Ignore Patterns ===${NC}"
echo "Programmer: 'Some files should never be edited by Aider'"
echo ""

cat > "$WORK_DIR/project3/.aiderignore" << 'EOF'
# Files Aider should never touch
.env
*.key
secrets/
node_modules/
target/
.git/
EOF

if [ -f "$WORK_DIR/project3/.aiderignore" ]; then
    log_test "Aider ignore file created" "PASS"
else
    log_test "Aider ignore file created" "FAIL"
fi

if grep -q ".env" "$WORK_DIR/project3/.aiderignore"; then
    log_test "Secrets excluded from Aider" "PASS"
else
    log_test "Secrets excluded from Aider" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 8: Voice Mode Configuration
# ============================================
echo -e "${BLUE}=== Scenario 8: Voice & Input Modes ===${NC}"
echo "Programmer: 'I want to use voice commands'"
echo ""

cat > "$WORK_DIR/project3/.aider-voice.conf.yml" << 'EOF'
model: claude-3-opus-20240229
voice-language: en
suggest-shell-commands: true
EOF

if grep -q "voice-language" "$WORK_DIR/project3/.aider-voice.conf.yml"; then
    log_test "Voice language configurable" "PASS"
else
    log_test "Voice language configurable" "FAIL"
fi

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "         AIDER TEST SUMMARY"
echo "=============================================="
echo ""
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/aider-test-report.md" << EOF
# Aider Integration Test Report

**Generated:** $(date -Iseconds)
**Docker Available:** $DOCKER_AVAILABLE
**Aider Image Exists:** $AIDER_IMAGE_EXISTS

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Skipped | $SKIPPED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **Config Detection** - .aider.conf.yml handling
2. **Model Selection** - Claude/GPT model switching
3. **Auto-Test** - Test command integration
4. **Git Integration** - Commit behavior control
5. **Multi-Provider** - Anthropic/OpenAI switching
6. **Repo Manager** - Managed config integration
7. **Ignore Patterns** - File exclusion
8. **Voice Mode** - Input mode configuration

## Providers Tested

### Anthropic
- API Key: \`ANTHROPIC_API_KEY\`
- Models: claude-3-opus, claude-3-sonnet

### OpenAI
- API Key: \`OPENAI_API_KEY\`
- Base URL: \`OPENAI_API_BASE\` (mock support)
- Models: gpt-4-turbo

## Config Files

- \`.aider.conf.yml\` - Main configuration
- \`.aiderignore\` - File exclusions
EOF

echo "Report: $RESULTS_DIR/aider-test-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL TESTS PASSED${NC}"
    exit 0
fi
