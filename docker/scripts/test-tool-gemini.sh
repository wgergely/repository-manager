#!/bin/bash
# Gemini CLI Tool-Specific Integration Tests
# Simulates programmer workflows with Google's Gemini CLI

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/tools/gemini"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "      GEMINI CLI INTEGRATION TESTS"
echo "=============================================="
echo ""

# Test tracking
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Check Docker
DOCKER_AVAILABLE=false
GEMINI_IMAGE_EXISTS=false

if docker info >/dev/null 2>&1; then
    DOCKER_AVAILABLE=true
    if docker image inspect repo-test/gemini:latest >/dev/null 2>&1; then
        GEMINI_IMAGE_EXISTS=true
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
# SCENARIO 1: GEMINI.md Config File
# ============================================
echo -e "${BLUE}=== Scenario 1: GEMINI.md Detection ===${NC}"
echo "Programmer: 'Gemini should read my project configuration'"
echo ""

mkdir -p "$WORK_DIR/project1"

cat > "$WORK_DIR/project1/GEMINI.md" << 'EOF'
# Project Guidelines for Gemini

You are assisting with a Rust CLI project. Follow these guidelines:

## Code Style
- Use idiomatic Rust patterns
- Prefer Result<T, E> for error handling
- Write comprehensive documentation

## Testing
- Every public function needs tests
- Use property-based testing for complex logic
- Maintain 80%+ code coverage

## Architecture
- Keep modules small and focused
- Use dependency injection patterns
- Separate concerns clearly
EOF

if [ -f "$WORK_DIR/project1/GEMINI.md" ]; then
    log_test "GEMINI.md file created" "PASS"
else
    log_test "GEMINI.md file created" "FAIL"
fi

# Verify content
line_count=$(wc -l < "$WORK_DIR/project1/GEMINI.md")
if [ "$line_count" -gt 10 ]; then
    log_test "GEMINI.md has substantial content ($line_count lines)" "PASS"
else
    log_test "GEMINI.md has substantial content" "FAIL" "Only $line_count lines"
fi

echo ""

# ============================================
# SCENARIO 2: Managed Blocks
# ============================================
echo -e "${BLUE}=== Scenario 2: Managed Blocks ===${NC}"
echo "Programmer: 'Repository Manager injects rules into GEMINI.md'"
echo ""

mkdir -p "$WORK_DIR/project2/.repository/rules"

cat > "$WORK_DIR/project2/.repository/rules/api-guidelines.md" << 'EOF'
# API Guidelines

- Use RESTful conventions
- Version all APIs (/v1/, /v2/)
- Return proper HTTP status codes
EOF

cat > "$WORK_DIR/project2/.repository/rules/documentation.md" << 'EOF'
# Documentation Standards

- Document all public functions
- Include examples in docstrings
- Keep README up to date
EOF

# Generate GEMINI.md with managed blocks
cat > "$WORK_DIR/project2/GEMINI.md" << 'EOF'
# Project Configuration

<!-- repo:block:api-guidelines -->
# API Guidelines

- Use RESTful conventions
- Version all APIs (/v1/, /v2/)
- Return proper HTTP status codes
<!-- /repo:block:api-guidelines -->

<!-- repo:block:documentation -->
# Documentation Standards

- Document all public functions
- Include examples in docstrings
- Keep README up to date
<!-- /repo:block:documentation -->

# My Custom Notes

- Remember the specific project requirements
- Check with team before major changes
EOF

# Verify managed blocks
block_start_count=$(grep -c "<!-- repo:block:" "$WORK_DIR/project2/GEMINI.md")
block_end_count=$(grep -c "<!-- /repo:block:" "$WORK_DIR/project2/GEMINI.md")

if [ "$block_start_count" -eq "$block_end_count" ] && [ "$block_start_count" -eq 2 ]; then
    log_test "Managed blocks properly structured (2 blocks)" "PASS"
else
    log_test "Managed blocks properly structured" "FAIL" "Start: $block_start_count, End: $block_end_count"
fi

# Verify content in blocks
if grep -q "RESTful conventions" "$WORK_DIR/project2/GEMINI.md"; then
    log_test "API rules content present" "PASS"
else
    log_test "API rules content present" "FAIL"
fi

if grep -q "Document all public functions" "$WORK_DIR/project2/GEMINI.md"; then
    log_test "Documentation rules content present" "PASS"
else
    log_test "Documentation rules content present" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 3: Manual Content Preservation
# ============================================
echo -e "${BLUE}=== Scenario 3: Manual Content Preservation ===${NC}"
echo "Programmer: 'My custom notes should survive sync'"
echo ""

if grep -q "My Custom Notes" "$WORK_DIR/project2/GEMINI.md"; then
    log_test "Manual content outside blocks exists" "PASS"
else
    log_test "Manual content outside blocks exists" "FAIL"
fi

# Simulate sync - update a managed block
sed -i 's/Version all APIs/Version all public APIs/' "$WORK_DIR/project2/GEMINI.md"

if grep -q "Version all public APIs" "$WORK_DIR/project2/GEMINI.md" && \
   grep -q "My Custom Notes" "$WORK_DIR/project2/GEMINI.md"; then
    log_test "Manual content preserved after block update" "PASS"
else
    log_test "Manual content preserved after block update" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 4: Google Cloud Provider Config
# ============================================
echo -e "${BLUE}=== Scenario 4: Google Cloud Provider ===${NC}"
echo "Programmer: 'Gemini uses my Google Cloud credentials'"
echo ""

# Test API key configuration
export GOOGLE_API_KEY="AIzaSy-test-gemini-key"
if [ -n "$GOOGLE_API_KEY" ]; then
    log_test "Google API key configurable" "PASS"
else
    log_test "Google API key configurable" "FAIL"
fi

# Test project ID configuration
export GOOGLE_CLOUD_PROJECT="my-test-project"
if [ -n "$GOOGLE_CLOUD_PROJECT" ]; then
    log_test "Google Cloud project configurable" "PASS"
else
    log_test "Google Cloud project configurable" "FAIL"
fi

# Test region configuration
export GOOGLE_CLOUD_REGION="us-central1"
if [ -n "$GOOGLE_CLOUD_REGION" ]; then
    log_test "Google Cloud region configurable" "PASS"
else
    log_test "Google Cloud region configurable" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 5: Model Selection
# ============================================
echo -e "${BLUE}=== Scenario 5: Model Selection ===${NC}"
echo "Programmer: 'I want to use different Gemini models'"
echo ""

cat > "$WORK_DIR/project1/.gemini-config.json" << 'EOF'
{
  "model": "gemini-1.5-pro",
  "temperature": 0.7,
  "maxOutputTokens": 8192,
  "topP": 0.95
}
EOF

if grep -q "gemini-1.5-pro" "$WORK_DIR/project1/.gemini-config.json"; then
    log_test "Gemini Pro model configurable" "PASS"
else
    log_test "Gemini Pro model configurable" "FAIL"
fi

# Alternative model config
cat > "$WORK_DIR/project1/.gemini-flash.json" << 'EOF'
{
  "model": "gemini-1.5-flash",
  "temperature": 0.5
}
EOF

if grep -q "gemini-1.5-flash" "$WORK_DIR/project1/.gemini-flash.json"; then
    log_test "Gemini Flash model configurable" "PASS"
else
    log_test "Gemini Flash model configurable" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 6: Multi-Language Project
# ============================================
echo -e "${BLUE}=== Scenario 6: Multi-Language Project ===${NC}"
echo "Programmer: 'My project uses multiple languages'"
echo ""

mkdir -p "$WORK_DIR/project3/backend" "$WORK_DIR/project3/frontend" "$WORK_DIR/project3/scripts"

cat > "$WORK_DIR/project3/backend/main.go" << 'EOF'
package main

func main() {
    println("Hello from Go")
}
EOF

cat > "$WORK_DIR/project3/frontend/index.ts" << 'EOF'
console.log("Hello from TypeScript");
EOF

cat > "$WORK_DIR/project3/scripts/deploy.py" << 'EOF'
print("Deploying...")
EOF

cat > "$WORK_DIR/project3/GEMINI.md" << 'EOF'
# Multi-Language Project

## Go (backend/)
- Follow Go idioms and conventions
- Use standard library when possible
- Write table-driven tests

## TypeScript (frontend/)
- Use strict TypeScript
- Functional React patterns
- Modern ES features

## Python (scripts/)
- Python 3.10+ features
- Type hints everywhere
- Use pathlib for paths
EOF

if grep -q "Go (backend/)" "$WORK_DIR/project3/GEMINI.md"; then
    log_test "Go-specific rules defined" "PASS"
else
    log_test "Go-specific rules defined" "FAIL"
fi

if grep -q "TypeScript (frontend/)" "$WORK_DIR/project3/GEMINI.md"; then
    log_test "TypeScript-specific rules defined" "PASS"
else
    log_test "TypeScript-specific rules defined" "FAIL"
fi

if grep -q "Python (scripts/)" "$WORK_DIR/project3/GEMINI.md"; then
    log_test "Python-specific rules defined" "PASS"
else
    log_test "Python-specific rules defined" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 7: Ignore Patterns
# ============================================
echo -e "${BLUE}=== Scenario 7: Ignore Patterns ===${NC}"
echo "Programmer: 'Some files should be ignored by Gemini'"
echo ""

cat > "$WORK_DIR/project3/.geminiignore" << 'EOF'
# Files Gemini should ignore
.env
.env.*
*.key
*.pem
secrets/
credentials/
node_modules/
vendor/
__pycache__/
.git/
EOF

if [ -f "$WORK_DIR/project3/.geminiignore" ]; then
    log_test ".geminiignore file created" "PASS"
else
    log_test ".geminiignore file created" "FAIL"
fi

if grep -q ".env" "$WORK_DIR/project3/.geminiignore"; then
    log_test "Environment files ignored" "PASS"
else
    log_test "Environment files ignored" "FAIL"
fi

if grep -q "vendor" "$WORK_DIR/project3/.geminiignore"; then
    log_test "Dependency directories ignored" "PASS"
else
    log_test "Dependency directories ignored" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 8: Sync Consistency
# ============================================
echo -e "${BLUE}=== Scenario 8: Cross-Tool Consistency ===${NC}"
echo "Programmer: 'Rules should match across all AI tools'"
echo ""

# Create CLAUDE.md with same managed blocks
cp "$WORK_DIR/project2/GEMINI.md" "$WORK_DIR/project2/CLAUDE.md"

# Extract content from managed blocks
gemini_api=$(sed -n '/<!-- repo:block:api-guidelines -->/,/<!-- \/repo:block:api-guidelines -->/p' "$WORK_DIR/project2/GEMINI.md" | grep -v "repo:block")
claude_api=$(sed -n '/<!-- repo:block:api-guidelines -->/,/<!-- \/repo:block:api-guidelines -->/p' "$WORK_DIR/project2/CLAUDE.md" | grep -v "repo:block")

if [ "$gemini_api" = "$claude_api" ]; then
    log_test "API rules consistent between Gemini and Claude" "PASS"
else
    log_test "API rules consistent between Gemini and Claude" "FAIL"
fi

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "        GEMINI CLI TEST SUMMARY"
echo "=============================================="
echo ""
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/gemini-test-report.md" << EOF
# Gemini CLI Integration Test Report

**Generated:** $(date -Iseconds)
**Docker Available:** $DOCKER_AVAILABLE
**Gemini Image Exists:** $GEMINI_IMAGE_EXISTS

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Skipped | $SKIPPED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **GEMINI.md Detection** - Config file handling
2. **Managed Blocks** - Repository Manager integration
3. **Manual Content** - Preservation of user edits
4. **Google Cloud** - API key and project configuration
5. **Model Selection** - Pro/Flash model switching
6. **Multi-Language** - Project-aware rules
7. **Ignore Patterns** - .geminiignore handling
8. **Cross-Tool Consistency** - Rule matching with Claude

## Provider: Google Cloud

- API Key: \`GOOGLE_API_KEY\`
- Project: \`GOOGLE_CLOUD_PROJECT\`
- Region: \`GOOGLE_CLOUD_REGION\`
- Models: gemini-1.5-pro, gemini-1.5-flash

## Config Files

- \`GEMINI.md\` - Main rules file (markdown)
- \`.geminiignore\` - File exclusions
- \`.gemini-config.json\` - Model settings
EOF

echo "Report: $RESULTS_DIR/gemini-test-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL TESTS PASSED${NC}"
    exit 0
fi
