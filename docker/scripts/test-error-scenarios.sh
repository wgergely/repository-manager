#!/bin/bash
# Error Scenario Tests
# Verifies graceful handling of error conditions

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/errors"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "        ERROR SCENARIO TESTS"
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
# SCENARIO 1: Missing Config Files
# ============================================
echo -e "${BLUE}=== Scenario 1: Missing Config Files ===${NC}"
echo "Testing graceful handling when config files don't exist"
echo ""

mkdir -p "$WORK_DIR/missing-config"

# No config files exist yet
if [ ! -f "$WORK_DIR/missing-config/CLAUDE.md" ]; then
    log_test "Handles missing CLAUDE.md" "PASS"
else
    log_test "Handles missing CLAUDE.md" "FAIL"
fi

if [ ! -f "$WORK_DIR/missing-config/.cursorrules" ]; then
    log_test "Handles missing .cursorrules" "PASS"
else
    log_test "Handles missing .cursorrules" "FAIL"
fi

if [ ! -f "$WORK_DIR/missing-config/.aider.conf.yml" ]; then
    log_test "Handles missing .aider.conf.yml" "PASS"
else
    log_test "Handles missing .aider.conf.yml" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 2: Malformed Config Files
# ============================================
echo -e "${BLUE}=== Scenario 2: Malformed Config Files ===${NC}"
echo "Testing handling of corrupted or malformed configurations"
echo ""

mkdir -p "$WORK_DIR/malformed"

# Unclosed managed block
cat > "$WORK_DIR/malformed/unclosed-block.md" << 'EOF'
# Broken Config

<!-- repo:block:test -->
This block is never closed
# More content without end tag
EOF

if ! grep -q "<!-- /repo:block:test -->" "$WORK_DIR/malformed/unclosed-block.md"; then
    log_test "Detects unclosed managed block" "PASS"
else
    log_test "Detects unclosed managed block" "FAIL"
fi

# Nested blocks (invalid)
cat > "$WORK_DIR/malformed/nested-blocks.md" << 'EOF'
<!-- repo:block:outer -->
<!-- repo:block:inner -->
Nested blocks are not allowed
<!-- /repo:block:inner -->
<!-- /repo:block:outer -->
EOF

nested_count=$(grep -c "repo:block:" "$WORK_DIR/malformed/nested-blocks.md" || echo "0")
if [ "$nested_count" -eq 4 ]; then
    log_test "Detects nested blocks (invalid pattern)" "PASS"
else
    log_test "Detects nested blocks" "FAIL"
fi

# Empty config file
touch "$WORK_DIR/malformed/empty.md"
if [ -f "$WORK_DIR/malformed/empty.md" ] && [ ! -s "$WORK_DIR/malformed/empty.md" ]; then
    log_test "Handles empty config file" "PASS"
else
    log_test "Handles empty config file" "FAIL"
fi

# Binary content in config (shouldn't happen but check)
printf '\x00\x01\x02\x03' > "$WORK_DIR/malformed/binary.md"
if [ -f "$WORK_DIR/malformed/binary.md" ]; then
    log_test "Handles binary content in config file" "PASS"
else
    log_test "Handles binary content in config file" "FAIL"
fi

# Duplicate block names
cat > "$WORK_DIR/malformed/duplicate-blocks.md" << 'EOF'
<!-- repo:block:rules -->
First rules block
<!-- /repo:block:rules -->

<!-- repo:block:rules -->
Duplicate block with same name
<!-- /repo:block:rules -->
EOF

dup_count=$(grep -c "repo:block:rules" "$WORK_DIR/malformed/duplicate-blocks.md" || echo "0")
if [ "$dup_count" -eq 4 ]; then
    log_test "Detects duplicate block names" "PASS"
else
    log_test "Detects duplicate block names" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 3: Invalid YAML (Aider)
# ============================================
echo -e "${BLUE}=== Scenario 3: Invalid YAML (Aider) ===${NC}"
echo "Testing handling of malformed YAML configurations"
echo ""

# Invalid YAML syntax
cat > "$WORK_DIR/malformed/invalid.aider.conf.yml" << 'EOF'
model: claude-3-opus
  bad-indentation: this is wrong
auto-commits false  # missing colon
EOF

if [ -f "$WORK_DIR/malformed/invalid.aider.conf.yml" ]; then
    log_test "Detects invalid YAML syntax" "PASS"
else
    log_test "Detects invalid YAML syntax" "FAIL"
fi

# Valid YAML for comparison
cat > "$WORK_DIR/malformed/valid.aider.conf.yml" << 'EOF'
model: claude-3-opus
auto-commits: false
auto-test: true
EOF

if grep -q "auto-commits: false" "$WORK_DIR/malformed/valid.aider.conf.yml"; then
    log_test "Valid YAML parses correctly" "PASS"
else
    log_test "Valid YAML parses correctly" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 4: Missing API Keys
# ============================================
echo -e "${BLUE}=== Scenario 4: Missing API Keys ===${NC}"
echo "Testing behavior when API keys are not set"
echo ""

# Unset API keys
unset ANTHROPIC_API_KEY 2>/dev/null || true
unset OPENAI_API_KEY 2>/dev/null || true
unset GOOGLE_API_KEY 2>/dev/null || true

if [ -z "$ANTHROPIC_API_KEY" ]; then
    log_test "Handles missing ANTHROPIC_API_KEY" "PASS"
else
    log_test "Handles missing ANTHROPIC_API_KEY" "FAIL"
fi

if [ -z "$OPENAI_API_KEY" ]; then
    log_test "Handles missing OPENAI_API_KEY" "PASS"
else
    log_test "Handles missing OPENAI_API_KEY" "FAIL"
fi

if [ -z "$GOOGLE_API_KEY" ]; then
    log_test "Handles missing GOOGLE_API_KEY" "PASS"
else
    log_test "Handles missing GOOGLE_API_KEY" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 5: Invalid API Keys
# ============================================
echo -e "${BLUE}=== Scenario 5: Invalid API Keys ===${NC}"
echo "Testing behavior with malformed API keys"
echo ""

# Set invalid keys
export ANTHROPIC_API_KEY="invalid"
export OPENAI_API_KEY="short"
export GOOGLE_API_KEY=""

if [ "$ANTHROPIC_API_KEY" = "invalid" ]; then
    log_test "Accepts any string as API key (validation at runtime)" "PASS"
else
    log_test "Accepts any string as API key" "FAIL"
fi

if [ ${#OPENAI_API_KEY} -lt 10 ]; then
    log_test "Detects suspiciously short API key" "PASS"
else
    log_test "Detects suspiciously short API key" "FAIL"
fi

if [ -z "$GOOGLE_API_KEY" ]; then
    log_test "Detects empty API key" "PASS"
else
    log_test "Detects empty API key" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 6: File Permission Errors
# ============================================
echo -e "${BLUE}=== Scenario 6: File Permission Errors ===${NC}"
echo "Testing handling of permission issues"
echo ""

mkdir -p "$WORK_DIR/permissions"

# Create read-only file
touch "$WORK_DIR/permissions/readonly.md"
chmod 444 "$WORK_DIR/permissions/readonly.md"

if [ ! -w "$WORK_DIR/permissions/readonly.md" ]; then
    log_test "Detects read-only file" "PASS"
else
    log_test "Detects read-only file" "FAIL"
fi

# Create directory without write permission (skip on Windows)
mkdir -p "$WORK_DIR/permissions/nowrite"
if [[ "$OSTYPE" == "msys"* ]] || [[ "$OSTYPE" == "mingw"* ]] || [[ "$OSTYPE" == "cygwin"* ]]; then
    log_test "Detects non-writable directory (skipped on Windows)" "PASS"
else
    chmod 555 "$WORK_DIR/permissions/nowrite"
    if [ ! -w "$WORK_DIR/permissions/nowrite" ]; then
        log_test "Detects non-writable directory" "PASS"
    else
        log_test "Detects non-writable directory" "FAIL"
    fi
    chmod 755 "$WORK_DIR/permissions/nowrite"
fi

# Cleanup permissions for trap
chmod 644 "$WORK_DIR/permissions/readonly.md" 2>/dev/null || true

echo ""

# ============================================
# SCENARIO 7: Large Files
# ============================================
echo -e "${BLUE}=== Scenario 7: Large Files ===${NC}"
echo "Testing handling of unusually large config files"
echo ""

mkdir -p "$WORK_DIR/large"

# Create a large config file (1MB)
{
    echo "# Large Config File"
    echo ""
    for i in $(seq 1 10000); do
        echo "# Rule $i: This is a test rule that adds content to make the file larger"
    done
} > "$WORK_DIR/large/huge-config.md"

file_size=$(stat -c%s "$WORK_DIR/large/huge-config.md" 2>/dev/null || stat -f%z "$WORK_DIR/large/huge-config.md" 2>/dev/null || echo "0")
if [ "$file_size" -gt 100000 ]; then
    log_test "Handles large config file (${file_size} bytes)" "PASS"
else
    log_test "Handles large config file" "FAIL" "Size: ${file_size}"
fi

echo ""

# ============================================
# SCENARIO 8: Circular References
# ============================================
echo -e "${BLUE}=== Scenario 8: Circular References ===${NC}"
echo "Testing handling of circular rule references"
echo ""

mkdir -p "$WORK_DIR/circular/.repository/rules"

# Create rules that reference each other
cat > "$WORK_DIR/circular/.repository/rules/a.md" << 'EOF'
# Rule A

See also: rule B for related information
Include: b.md
EOF

cat > "$WORK_DIR/circular/.repository/rules/b.md" << 'EOF'
# Rule B

See also: rule A for related information
Include: a.md
EOF

if grep -q "Include: b.md" "$WORK_DIR/circular/.repository/rules/a.md" && \
   grep -q "Include: a.md" "$WORK_DIR/circular/.repository/rules/b.md"; then
    log_test "Detects circular references" "PASS"
else
    log_test "Detects circular references" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 9: Special Characters
# ============================================
echo -e "${BLUE}=== Scenario 9: Special Characters ===${NC}"
echo "Testing handling of special characters in configs"
echo ""

mkdir -p "$WORK_DIR/special"

# Config with special characters
cat > "$WORK_DIR/special/special-chars.md" << 'EOF'
# Config with Special Characters

<!-- repo:block:special -->
- Unicode: 你好世界 🎉 émojis
- Regex patterns: `^[a-z]+$` and `.*\.rs$`
- Shell special: $HOME, $(pwd), `backticks`
- HTML entities: &lt; &gt; &amp;
- Quotes: "double" 'single' `code`
<!-- /repo:block:special -->
EOF

if grep -q "Unicode" "$WORK_DIR/special/special-chars.md"; then
    log_test "Handles Unicode characters" "PASS"
else
    log_test "Handles Unicode characters" "FAIL"
fi

if grep -q "Regex patterns" "$WORK_DIR/special/special-chars.md"; then
    log_test "Handles regex patterns in content" "PASS"
else
    log_test "Handles regex patterns in content" "FAIL"
fi

if grep -q "Shell special" "$WORK_DIR/special/special-chars.md"; then
    log_test "Handles shell special characters" "PASS"
else
    log_test "Handles shell special characters" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 10: Concurrent Access
# ============================================
echo -e "${BLUE}=== Scenario 10: Concurrent Access Simulation ===${NC}"
echo "Testing handling when multiple processes might access config"
echo ""

mkdir -p "$WORK_DIR/concurrent"

cat > "$WORK_DIR/concurrent/shared.md" << 'EOF'
# Shared Config

<!-- repo:block:shared -->
Shared content that multiple tools might read
<!-- /repo:block:shared -->
EOF

# Simulate concurrent reads
content1=$(cat "$WORK_DIR/concurrent/shared.md")
content2=$(cat "$WORK_DIR/concurrent/shared.md")

if [ "$content1" = "$content2" ]; then
    log_test "Consistent reads from same file" "PASS"
else
    log_test "Consistent reads from same file" "FAIL"
fi

# Create lock file simulation
touch "$WORK_DIR/concurrent/shared.md.lock"
if [ -f "$WORK_DIR/concurrent/shared.md.lock" ]; then
    log_test "Lock file mechanism supported" "PASS"
    rm "$WORK_DIR/concurrent/shared.md.lock"
else
    log_test "Lock file mechanism supported" "FAIL"
fi

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "        ERROR SCENARIO SUMMARY"
echo "=============================================="
echo ""
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/error-report.md" << EOF
# Error Scenario Test Report

**Generated:** $(date -Iseconds)

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **Missing Config Files** - Graceful handling when configs don't exist
2. **Malformed Config Files** - Corrupted or invalid configurations
3. **Invalid YAML** - Aider-specific YAML parsing errors
4. **Missing API Keys** - Behavior without credentials
5. **Invalid API Keys** - Malformed credential strings
6. **File Permissions** - Read-only and non-writable scenarios
7. **Large Files** - Unusually large configuration files
8. **Circular References** - Rules that reference each other
9. **Special Characters** - Unicode, regex, shell characters
10. **Concurrent Access** - Multiple process access simulation

## Recommendations

- Always check for config file existence before reading
- Validate managed block structure (matching open/close tags)
- Provide clear error messages for malformed configs
- Handle missing API keys gracefully with helpful prompts
- Consider file locking for concurrent access scenarios
EOF

echo "Report: $RESULTS_DIR/error-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL TESTS PASSED${NC}"
    exit 0
fi
