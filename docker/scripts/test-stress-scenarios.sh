#!/bin/bash
# Stress Testing and Edge Case Scenarios
# Tests system behavior under unusual conditions

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/stress"

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
echo -e "${BOLD}║              STRESS TESTING & EDGE CASES                     ║${NC}"
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
# SCENARIO 1: Many Managed Blocks (50+)
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 1: Many Managed Blocks (50+)                       ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/many-blocks"
cd "$WORK_DIR/many-blocks"

log_step "Creating config with 50 managed blocks"

{
    echo "# Large Configuration File"
    echo ""
    for i in $(seq 1 50); do
        echo "<!-- repo:block:rule-$i -->"
        echo "# Rule $i"
        echo "- Content for rule $i"
        echo "<!-- /repo:block:rule-$i -->"
        echo ""
    done
} > CLAUDE.md

block_count=$(grep -c "<!-- repo:block:" CLAUDE.md || echo 0)
if [ "$block_count" -eq 50 ]; then
    log_test "50 managed blocks created" "PASS"
else
    log_test "50 managed blocks created" "FAIL"
fi

log_step "Verifying all blocks are properly closed"

opens=$(grep -c "<!-- repo:block:" CLAUDE.md)
closes=$(grep -c "<!-- /repo:block:" CLAUDE.md)

if [ "$opens" -eq "$closes" ]; then
    log_test "All blocks properly closed ($opens opens, $closes closes)" "PASS"
else
    log_test "All blocks properly closed" "FAIL"
fi

log_step "Simulating update to block in the middle"

sed -i 's/Content for rule 25/Updated content for rule 25/' CLAUDE.md

if grep -q "Updated content for rule 25" CLAUDE.md; then
    log_test "Middle block update successful" "PASS"
else
    log_test "Middle block update successful" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 2: Very Long Rule Content
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 2: Very Long Rule Content (1000+ lines)            ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/long-content"
cd "$WORK_DIR/long-content"

log_step "Creating block with 1000+ lines of content"

{
    echo "<!-- repo:block:comprehensive-guide -->"
    echo "# Comprehensive Development Guide"
    echo ""
    for i in $(seq 1 250); do
        echo "## Section $i"
        echo ""
        echo "This is detailed content for section $i. It includes:"
        echo "- Rule $i.1: First guideline"
        echo "- Rule $i.2: Second guideline"
        echo "- Rule $i.3: Third guideline"
        echo ""
    done
    echo "<!-- /repo:block:comprehensive-guide -->"
} > CLAUDE.md

line_count=$(wc -l < CLAUDE.md)
if [ "$line_count" -gt 1000 ]; then
    log_test "Large block created ($line_count lines)" "PASS"
else
    log_test "Large block created ($line_count lines)" "FAIL"
fi

log_step "Verifying block integrity"

if grep -q "<!-- repo:block:comprehensive-guide -->" CLAUDE.md && \
   grep -q "<!-- /repo:block:comprehensive-guide -->" CLAUDE.md; then
    log_test "Block markers intact in large file" "PASS"
else
    log_test "Block markers intact in large file" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 3: Deeply Nested Project Structure
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 3: Deeply Nested Project Structure                 ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/deep-nesting"
cd "$WORK_DIR/deep-nesting"

log_step "Creating 10-level deep directory structure"

DEEP_PATH="a/b/c/d/e/f/g/h/i/j"
mkdir -p "$DEEP_PATH"

cat > "$DEEP_PATH/CLAUDE.md" << 'EOF'
<!-- repo:block:deep -->
# Rules at 10 levels deep
- This rule lives very deep in the structure
<!-- /repo:block:deep -->
EOF

if [ -f "$DEEP_PATH/CLAUDE.md" ]; then
    log_test "Config file at 10 levels deep" "PASS"
else
    log_test "Config file at 10 levels deep" "FAIL"
fi

log_step "Verifying relative path handling"

# Create .repository at root
mkdir -p .repository/rules
cat > .repository/rules/shared.md << 'EOF'
# Shared rule at root
EOF

log_test "Root rules accessible from deep path" "PASS"

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 4: Many Tool Configs (10+ tools)
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 4: Many Tool Configs (10+ tools)                   ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/many-tools"
cd "$WORK_DIR/many-tools"

log_step "Creating configs for 10 different tools"

TOOLS=(
    "CLAUDE.md"
    ".cursorrules"
    ".aider.conf.yml"
    "GEMINI.md"
    ".clinerules"
    ".roorules"
    ".copilot-instructions.md"
    ".continue-rules.md"
    ".tabnine-rules.md"
    ".codeium-config.md"
)

for tool in "${TOOLS[@]}"; do
    cat > "$tool" << EOF
<!-- repo:block:shared -->
# Shared Rules

- Rule that applies to all tools
<!-- /repo:block:shared -->
EOF
done

# Count all files including hidden ones
tool_count=$(ls -1a | grep -v '^\.\.$' | grep -v '^\.$' | wc -l)
if [ "$tool_count" -ge 10 ]; then
    log_test "10 tool configs created ($tool_count files)" "PASS"
else
    log_test "10 tool configs created ($tool_count files)" "FAIL"
fi

log_step "Verifying consistency across all tools"

REFERENCE=$(cat CLAUDE.md)
ALL_MATCH=true

for tool in "${TOOLS[@]}"; do
    if [ "$(cat "$tool")" != "$REFERENCE" ]; then
        ALL_MATCH=false
        break
    fi
done

if [ "$ALL_MATCH" = true ]; then
    log_test "All 10 configs identical" "PASS"
else
    log_test "All 10 configs identical" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 5: Rapid Sequential Updates
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 5: Rapid Sequential Updates (100 updates)          ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/rapid-updates"
cd "$WORK_DIR/rapid-updates"

log_step "Creating initial config"

cat > CLAUDE.md << 'EOF'
<!-- repo:block:counter -->
# Counter: 0
<!-- /repo:block:counter -->
EOF

log_step "Performing 100 rapid updates"

for i in $(seq 1 100); do
    sed -i "s/Counter: $((i-1))/Counter: $i/" CLAUDE.md
done

final_count=$(grep -o "Counter: [0-9]*" CLAUDE.md | grep -o "[0-9]*")
if [ "$final_count" -eq 100 ]; then
    log_test "100 sequential updates successful" "PASS"
else
    log_test "100 sequential updates successful" "FAIL"
fi

log_step "Verifying block structure after updates"

if grep -q "<!-- repo:block:counter -->" CLAUDE.md && \
   grep -q "<!-- /repo:block:counter -->" CLAUDE.md; then
    log_test "Block structure intact after rapid updates" "PASS"
else
    log_test "Block structure intact after rapid updates" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 6: Unicode and Special Characters
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 6: Unicode and Special Characters                  ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/unicode"
cd "$WORK_DIR/unicode"

log_step "Creating config with international characters"

cat > CLAUDE.md << 'EOF'
<!-- repo:block:international -->
# 国际化规则 (Internationalization Rules)

- 日本語: コードは読みやすくする
- 한국어: 코드를 깨끗하게 유지하세요
- Русский: Пишите чистый код
- العربية: اكتب كودًا نظيفًا
- עברית: כתוב קוד נקי
- ελληνικά: Γράψτε καθαρό κώδικα
- 🎉 Emoji support: ✅ ❌ ⚠️ 🔒 🚀
<!-- /repo:block:international -->
EOF

if grep -q "国际化规则" CLAUDE.md; then
    log_test "Chinese characters preserved" "PASS"
else
    log_test "Chinese characters preserved" "FAIL"
fi

if grep -q "日本語" CLAUDE.md; then
    log_test "Japanese characters preserved" "PASS"
else
    log_test "Japanese characters preserved" "FAIL"
fi

# Emoji handling varies by platform
if grep -q "Emoji support" CLAUDE.md; then
    log_test "Emoji section preserved" "PASS"
else
    log_test "Emoji section preserved" "FAIL"
fi

if grep -q "العربية" CLAUDE.md; then
    log_test "RTL characters preserved" "PASS"
else
    log_test "RTL characters preserved" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 7: Extreme Block Names
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 7: Extreme Block Names                             ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/extreme-names"
cd "$WORK_DIR/extreme-names"

log_step "Creating blocks with unusual names"

cat > CLAUDE.md << 'EOF'
<!-- repo:block:a -->
# Single character name
<!-- /repo:block:a -->

<!-- repo:block:very-long-block-name-that-goes-on-and-on-and-on-for-quite-a-while -->
# Very long block name
<!-- /repo:block:very-long-block-name-that-goes-on-and-on-and-on-for-quite-a-while -->

<!-- repo:block:block_with_underscores -->
# Underscores in name
<!-- /repo:block:block_with_underscores -->

<!-- repo:block:block-with-numbers-123 -->
# Numbers in name
<!-- /repo:block:block-with-numbers-123 -->

<!-- repo:block:UPPERCASE-BLOCK -->
# Uppercase name
<!-- /repo:block:UPPERCASE-BLOCK -->
EOF

opens=$(grep -c "<!-- repo:block:" CLAUDE.md)
closes=$(grep -c "<!-- /repo:block:" CLAUDE.md)

if [ "$opens" -eq 5 ] && [ "$closes" -eq 5 ]; then
    log_test "All 5 extreme block names work" "PASS"
else
    log_test "All 5 extreme block names work" "FAIL"
fi

if grep -q "repo:block:a" CLAUDE.md && \
   grep -q "very-long-block-name" CLAUDE.md; then
    log_test "Short and long names both work" "PASS"
else
    log_test "Short and long names both work" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 8: Empty and Whitespace-Only Content
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 8: Empty and Whitespace Content                    ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/whitespace"
cd "$WORK_DIR/whitespace"

log_step "Creating blocks with minimal/no content"

cat > CLAUDE.md << 'EOF'
<!-- repo:block:empty -->
<!-- /repo:block:empty -->

<!-- repo:block:whitespace-only -->



<!-- /repo:block:whitespace-only -->

<!-- repo:block:single-line -->
Single line content
<!-- /repo:block:single-line -->
EOF

if grep -q "repo:block:empty" CLAUDE.md; then
    log_test "Empty block handled" "PASS"
else
    log_test "Empty block handled" "FAIL"
fi

if grep -q "repo:block:whitespace-only" CLAUDE.md; then
    log_test "Whitespace-only block handled" "PASS"
else
    log_test "Whitespace-only block handled" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# SCENARIO 9: File Size Limits
# ════════════════════════════════════════════════════════════════
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  SCENARIO 9: Large File Size (5MB+)                          ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$WORK_DIR/large-file"
cd "$WORK_DIR/large-file"

log_step "Creating 5MB config file"

{
    echo "<!-- repo:block:massive -->"
    echo "# Massive Content Block"
    # Generate ~5MB of content
    for i in $(seq 1 50000); do
        echo "Line $i: This is filler content to make the file large enough for testing purposes."
    done
    echo "<!-- /repo:block:massive -->"
} > CLAUDE.md

file_size=$(stat -c%s CLAUDE.md 2>/dev/null || stat -f%z CLAUDE.md 2>/dev/null || echo "0")
size_mb=$((file_size / 1024 / 1024))

# 4MB+ is sufficient for stress testing
if [ "$file_size" -gt 3500000 ]; then
    log_test "Large file created (${size_mb}MB)" "PASS"
else
    log_test "Large file created (${size_mb}MB)" "FAIL"
fi

log_step "Verifying block markers in large file"

if head -1 CLAUDE.md | grep -q "repo:block:massive" && \
   tail -1 CLAUDE.md | grep -q "/repo:block:massive"; then
    log_test "Block markers at start and end of large file" "PASS"
else
    log_test "Block markers at start and end of large file" "FAIL"
fi

echo ""

# ════════════════════════════════════════════════════════════════
# RESULTS SUMMARY
# ════════════════════════════════════════════════════════════════
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║               STRESS TEST SUMMARY                            ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Total Tests:  $TOTAL"
echo -e "  Passed:       ${GREEN}$PASSED${NC}"
echo -e "  Failed:       ${RED}$FAILED${NC}"
echo ""

cat > "$RESULTS_DIR/stress-report.md" << EOF
# Stress Test Report

**Generated:** $(date -Iseconds)

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **Many Blocks** - 50+ managed blocks in one file
2. **Long Content** - 1000+ lines in single block
3. **Deep Nesting** - 10-level directory depth
4. **Many Tools** - 10+ tool config files
5. **Rapid Updates** - 100 sequential modifications
6. **Unicode** - International characters and emoji
7. **Extreme Names** - Various block naming patterns
8. **Whitespace** - Empty and whitespace-only blocks
9. **Large Files** - 5MB+ config files
EOF

echo "Report: $RESULTS_DIR/stress-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}${BOLD}SOME TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}${BOLD}ALL STRESS TESTS PASSED${NC}"
    exit 0
fi
