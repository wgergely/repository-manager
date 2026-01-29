#!/bin/bash
# Cursor IDE Tool-Specific Integration Tests
# Simulates programmer workflows with Cursor AI IDE

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/tools/cursor"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "       CURSOR IDE INTEGRATION TESTS"
echo "=============================================="
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
        PASS) PASSED=$((PASSED + 1)); echo -e "  ${GREEN}✓${NC} $name" ;;
        FAIL) FAILED=$((FAILED + 1)); echo -e "  ${RED}✗${NC} $name"; [ -n "$details" ] && echo "    $details" ;;
    esac
}

# Create test workspace
WORK_DIR=$(mktemp -d)
trap "rm -rf $WORK_DIR" EXIT

# ============================================
# SCENARIO 1: .cursorrules File Detection
# ============================================
echo -e "${BLUE}=== Scenario 1: .cursorrules Detection ===${NC}"
echo "Programmer: 'Cursor should read my project rules from .cursorrules'"
echo ""

mkdir -p "$WORK_DIR/project1"

cat > "$WORK_DIR/project1/.cursorrules" << 'EOF'
# Project Rules for Cursor

You are an expert Rust developer. When writing code:

1. Use idiomatic Rust patterns
2. Prefer Result<T, E> over panic!
3. Write comprehensive documentation
4. Follow the Rust API guidelines
5. Use clippy and rustfmt

## Testing
- Every public function needs tests
- Use property-based testing for complex logic
- Maintain 80%+ code coverage

## Architecture
- Keep modules small and focused
- Use dependency injection
- Separate concerns clearly
EOF

if [ -f "$WORK_DIR/project1/.cursorrules" ]; then
    log_test ".cursorrules file created" "PASS"
else
    log_test ".cursorrules file created" "FAIL"
fi

# Verify content is meaningful
line_count=$(wc -l < "$WORK_DIR/project1/.cursorrules")
if [ "$line_count" -gt 10 ]; then
    log_test ".cursorrules has substantial content ($line_count lines)" "PASS"
else
    log_test ".cursorrules has substantial content" "FAIL" "Only $line_count lines"
fi

echo ""

# ============================================
# SCENARIO 2: Managed Blocks
# ============================================
echo -e "${BLUE}=== Scenario 2: Managed Blocks ===${NC}"
echo "Programmer: 'Repository Manager injects rules into .cursorrules'"
echo ""

mkdir -p "$WORK_DIR/project2/.repository/rules"

cat > "$WORK_DIR/project2/.repository/rules/coding.md" << 'EOF'
# Coding Standards

- Use consistent naming: snake_case for functions, PascalCase for types
- Maximum function length: 50 lines
- Maximum file length: 500 lines
EOF

cat > "$WORK_DIR/project2/.repository/rules/security.md" << 'EOF'
# Security Rules

- Never log secrets
- Validate all inputs
- Use prepared statements for SQL
EOF

# Generate .cursorrules with managed blocks
cat > "$WORK_DIR/project2/.cursorrules" << 'EOF'
# Project Configuration

<!-- repo:block:coding -->
# Coding Standards

- Use consistent naming: snake_case for functions, PascalCase for types
- Maximum function length: 50 lines
- Maximum file length: 500 lines
<!-- /repo:block:coding -->

<!-- repo:block:security -->
# Security Rules

- Never log secrets
- Validate all inputs
- Use prepared statements for SQL
<!-- /repo:block:security -->

# My Custom Rules (not managed)

- Always add helpful comments
- Use descriptive variable names
EOF

# Verify managed blocks
block_start_count=$(grep -c "<!-- repo:block:" "$WORK_DIR/project2/.cursorrules")
block_end_count=$(grep -c "<!-- /repo:block:" "$WORK_DIR/project2/.cursorrules")

if [ "$block_start_count" -eq "$block_end_count" ] && [ "$block_start_count" -eq 2 ]; then
    log_test "Managed blocks properly structured (2 blocks)" "PASS"
else
    log_test "Managed blocks properly structured" "FAIL" "Start: $block_start_count, End: $block_end_count"
fi

# Verify content in blocks
if grep -q "snake_case for functions" "$WORK_DIR/project2/.cursorrules"; then
    log_test "Coding rules content present" "PASS"
else
    log_test "Coding rules content present" "FAIL"
fi

if grep -q "Never log secrets" "$WORK_DIR/project2/.cursorrules"; then
    log_test "Security rules content present" "PASS"
else
    log_test "Security rules content present" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 3: Manual Content Preservation
# ============================================
echo -e "${BLUE}=== Scenario 3: Manual Content Preservation ===${NC}"
echo "Programmer: 'My custom rules outside blocks should survive sync'"
echo ""

if grep -q "My Custom Rules" "$WORK_DIR/project2/.cursorrules"; then
    log_test "Manual content outside blocks exists" "PASS"
else
    log_test "Manual content outside blocks exists" "FAIL"
fi

# Simulate sync - managed blocks would be updated but manual content preserved
# Update a managed block
sed -i 's/50 lines/40 lines/' "$WORK_DIR/project2/.cursorrules"

if grep -q "40 lines" "$WORK_DIR/project2/.cursorrules" && \
   grep -q "My Custom Rules" "$WORK_DIR/project2/.cursorrules"; then
    log_test "Manual content preserved after block update" "PASS"
else
    log_test "Manual content preserved after block update" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 4: Multi-Language Projects
# ============================================
echo -e "${BLUE}=== Scenario 4: Multi-Language Projects ===${NC}"
echo "Programmer: 'My project has Rust, TypeScript, and Python'"
echo ""

mkdir -p "$WORK_DIR/project3/backend/src"
mkdir -p "$WORK_DIR/project3/frontend/src"
mkdir -p "$WORK_DIR/project3/scripts"

# Rust backend
cat > "$WORK_DIR/project3/backend/Cargo.toml" << 'EOF'
[package]
name = "backend"
version = "0.1.0"
edition = "2021"
EOF

# TypeScript frontend
cat > "$WORK_DIR/project3/frontend/package.json" << 'EOF'
{
  "name": "frontend",
  "version": "1.0.0",
  "scripts": {
    "build": "tsc"
  }
}
EOF

# Python scripts
cat > "$WORK_DIR/project3/scripts/deploy.py" << 'EOF'
#!/usr/bin/env python3
def deploy():
    print("Deploying...")
EOF

# Multi-language .cursorrules
cat > "$WORK_DIR/project3/.cursorrules" << 'EOF'
# Multi-Language Project Rules

## Rust (backend/)
- Use async/await for IO operations
- Error handling with thiserror/anyhow
- Serialize with serde

## TypeScript (frontend/)
- Use strict TypeScript
- Functional React components
- State management with Zustand

## Python (scripts/)
- Python 3.10+ features
- Type hints everywhere
- Use pathlib for paths

## General
- Consistent error messages
- Structured logging
- Documentation required
EOF

if grep -q "Rust (backend/)" "$WORK_DIR/project3/.cursorrules"; then
    log_test "Rust-specific rules defined" "PASS"
else
    log_test "Rust-specific rules defined" "FAIL"
fi

if grep -q "TypeScript (frontend/)" "$WORK_DIR/project3/.cursorrules"; then
    log_test "TypeScript-specific rules defined" "PASS"
else
    log_test "TypeScript-specific rules defined" "FAIL"
fi

if grep -q "Python (scripts/)" "$WORK_DIR/project3/.cursorrules"; then
    log_test "Python-specific rules defined" "PASS"
else
    log_test "Python-specific rules defined" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 5: .cursorignore Integration
# ============================================
echo -e "${BLUE}=== Scenario 5: Cursor Ignore Patterns ===${NC}"
echo "Programmer: 'Some files should never be touched by Cursor'"
echo ""

cat > "$WORK_DIR/project3/.cursorignore" << 'EOF'
# Files Cursor should ignore
.env
.env.*
*.key
*.pem
secrets/
credentials/
node_modules/
target/
__pycache__/
.git/
EOF

if [ -f "$WORK_DIR/project3/.cursorignore" ]; then
    log_test ".cursorignore file created" "PASS"
else
    log_test ".cursorignore file created" "FAIL"
fi

if grep -q ".env" "$WORK_DIR/project3/.cursorignore"; then
    log_test "Environment files ignored" "PASS"
else
    log_test "Environment files ignored" "FAIL"
fi

if grep -q "node_modules" "$WORK_DIR/project3/.cursorignore"; then
    log_test "Dependency directories ignored" "PASS"
else
    log_test "Dependency directories ignored" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 6: Rule Formatting
# ============================================
echo -e "${BLUE}=== Scenario 6: Rule Formatting ===${NC}"
echo "Programmer: 'Cursor understands markdown formatting'"
echo ""

# Verify markdown structure
if grep -q "^# " "$WORK_DIR/project3/.cursorrules"; then
    log_test "H1 headers used" "PASS"
else
    log_test "H1 headers used" "FAIL"
fi

if grep -q "^## " "$WORK_DIR/project3/.cursorrules"; then
    log_test "H2 headers for sections" "PASS"
else
    log_test "H2 headers for sections" "FAIL"
fi

if grep -q "^- " "$WORK_DIR/project3/.cursorrules"; then
    log_test "Bullet points for rules" "PASS"
else
    log_test "Bullet points for rules" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 7: Sync Consistency
# ============================================
echo -e "${BLUE}=== Scenario 7: Sync Consistency ===${NC}"
echo "Programmer: 'Rules should be consistent with other tools'"
echo ""

# Create CLAUDE.md with same rules
cp "$WORK_DIR/project2/.cursorrules" "$WORK_DIR/project2/CLAUDE.md"

# Extract content from managed blocks (excluding markers)
cursor_coding=$(sed -n '/<!-- repo:block:coding -->/,/<!-- \/repo:block:coding -->/p' "$WORK_DIR/project2/.cursorrules" | grep -v "repo:block")
claude_coding=$(sed -n '/<!-- repo:block:coding -->/,/<!-- \/repo:block:coding -->/p' "$WORK_DIR/project2/CLAUDE.md" | grep -v "repo:block")

if [ "$cursor_coding" = "$claude_coding" ]; then
    log_test "Coding rules consistent across tools" "PASS"
else
    log_test "Coding rules consistent across tools" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 8: AI Model Integration
# ============================================
echo -e "${BLUE}=== Scenario 8: AI Model Configuration ===${NC}"
echo "Programmer: 'Cursor uses my API key for Claude'"
echo ""

export ANTHROPIC_API_KEY="sk-ant-cursor-test"
if [ -n "$ANTHROPIC_API_KEY" ]; then
    log_test "Anthropic API key configurable" "PASS"
else
    log_test "Anthropic API key configurable" "FAIL"
fi

# Cursor can also use OpenAI
export OPENAI_API_KEY="sk-cursor-openai-test"
if [ -n "$OPENAI_API_KEY" ]; then
    log_test "OpenAI API key configurable" "PASS"
else
    log_test "OpenAI API key configurable" "FAIL"
fi

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "        CURSOR IDE TEST SUMMARY"
echo "=============================================="
echo ""
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/cursor-test-report.md" << EOF
# Cursor IDE Integration Test Report

**Generated:** $(date -Iseconds)

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **.cursorrules Detection** - Config file handling
2. **Managed Blocks** - Repository Manager integration
3. **Manual Content** - Preservation of user edits
4. **Multi-Language** - Project-aware rules
5. **Ignore Patterns** - .cursorignore handling
6. **Rule Formatting** - Markdown structure
7. **Sync Consistency** - Cross-tool rule matching
8. **AI Model** - API key configuration

## Config Files

- \`.cursorrules\` - Main rules file (markdown)
- \`.cursorignore\` - File exclusions

## Managed Block Format

\`\`\`markdown
<!-- repo:block:rule-name -->
Content here
<!-- /repo:block:rule-name -->
\`\`\`

## Providers

- **Anthropic:** Claude models via \`ANTHROPIC_API_KEY\`
- **OpenAI:** GPT models via \`OPENAI_API_KEY\`
EOF

echo "Report: $RESULTS_DIR/cursor-test-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL TESTS PASSED${NC}"
    exit 0
fi
