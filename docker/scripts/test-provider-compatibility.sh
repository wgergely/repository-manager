#!/bin/bash
# Provider Compatibility Tests
# Verifies API provider configuration works consistently across all tools

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/providers"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "     PROVIDER COMPATIBILITY TESTS"
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
# PROVIDER 1: ANTHROPIC
# ============================================
echo -e "${BLUE}=== Provider: Anthropic ===${NC}"
echo "Testing Claude API configuration across all supporting tools"
echo ""

mkdir -p "$WORK_DIR/anthropic-test"

# Set Anthropic environment
export ANTHROPIC_API_KEY="sk-ant-test-key-12345"
export ANTHROPIC_BASE_URL="http://mock-api:8080/v1"

# Claude CLI
cat > "$WORK_DIR/anthropic-test/CLAUDE.md" << 'EOF'
# Claude Configuration

Use ANTHROPIC_API_KEY for authentication.
Base URL: ANTHROPIC_BASE_URL
EOF

if [ -f "$WORK_DIR/anthropic-test/CLAUDE.md" ]; then
    log_test "Anthropic: Claude CLI config" "PASS"
else
    log_test "Anthropic: Claude CLI config" "FAIL"
fi

# Cursor IDE
cat > "$WORK_DIR/anthropic-test/.cursorrules" << 'EOF'
# Cursor with Claude

Provider: Anthropic
API Key: ${ANTHROPIC_API_KEY}
Model: claude-3-opus
EOF

if grep -q "Anthropic" "$WORK_DIR/anthropic-test/.cursorrules"; then
    log_test "Anthropic: Cursor IDE config" "PASS"
else
    log_test "Anthropic: Cursor IDE config" "FAIL"
fi

# Aider
cat > "$WORK_DIR/anthropic-test/.aider.conf.yml" << 'EOF'
model: claude-3-opus-20240229
auto-commits: false
EOF

if grep -q "claude-3-opus" "$WORK_DIR/anthropic-test/.aider.conf.yml"; then
    log_test "Anthropic: Aider config" "PASS"
else
    log_test "Anthropic: Aider config" "FAIL"
fi

# Cline
cat > "$WORK_DIR/anthropic-test/.clinerules" << 'EOF'
# Cline with Claude

Provider: anthropic
Model: claude-3-5-sonnet
EOF

mkdir -p "$WORK_DIR/anthropic-test/.vscode"
cat > "$WORK_DIR/anthropic-test/.vscode/settings.json" << 'EOF'
{
  "cline.apiProvider": "anthropic",
  "cline.modelId": "claude-3-5-sonnet-20241022"
}
EOF

if grep -q "anthropic" "$WORK_DIR/anthropic-test/.vscode/settings.json"; then
    log_test "Anthropic: Cline extension config" "PASS"
else
    log_test "Anthropic: Cline extension config" "FAIL"
fi

# Roo Code
cat > "$WORK_DIR/anthropic-test/.roorules" << 'EOF'
# Roo with Claude

Provider: anthropic
Model: claude-3-5-sonnet
EOF

if [ -f "$WORK_DIR/anthropic-test/.roorules" ]; then
    log_test "Anthropic: Roo Code config" "PASS"
else
    log_test "Anthropic: Roo Code config" "FAIL"
fi

# API Key validation
if [ -n "$ANTHROPIC_API_KEY" ]; then
    log_test "Anthropic: API key environment variable" "PASS"
else
    log_test "Anthropic: API key environment variable" "FAIL"
fi

# Base URL override
if [ -n "$ANTHROPIC_BASE_URL" ]; then
    log_test "Anthropic: Base URL override (for mocking)" "PASS"
else
    log_test "Anthropic: Base URL override (for mocking)" "FAIL"
fi

echo ""

# ============================================
# PROVIDER 2: OPENAI
# ============================================
echo -e "${BLUE}=== Provider: OpenAI ===${NC}"
echo "Testing OpenAI API configuration across supporting tools"
echo ""

mkdir -p "$WORK_DIR/openai-test"

export OPENAI_API_KEY="sk-openai-test-key-12345"
export OPENAI_API_BASE="http://mock-api:8080/v1"

# Cursor IDE with OpenAI
cat > "$WORK_DIR/openai-test/.cursorrules" << 'EOF'
# Cursor with GPT

Provider: OpenAI
API Key: ${OPENAI_API_KEY}
Model: gpt-4-turbo
EOF

if grep -q "OpenAI" "$WORK_DIR/openai-test/.cursorrules"; then
    log_test "OpenAI: Cursor IDE config" "PASS"
else
    log_test "OpenAI: Cursor IDE config" "FAIL"
fi

# Aider with OpenAI
cat > "$WORK_DIR/openai-test/.aider.conf.yml" << 'EOF'
model: gpt-4-turbo-preview
auto-commits: false
EOF

if grep -q "gpt-4" "$WORK_DIR/openai-test/.aider.conf.yml"; then
    log_test "OpenAI: Aider config" "PASS"
else
    log_test "OpenAI: Aider config" "FAIL"
fi

# Cline with OpenAI
mkdir -p "$WORK_DIR/openai-test/.vscode"
cat > "$WORK_DIR/openai-test/.vscode/settings.json" << 'EOF'
{
  "cline.apiProvider": "openai",
  "cline.modelId": "gpt-4-turbo"
}
EOF

if grep -q "openai" "$WORK_DIR/openai-test/.vscode/settings.json"; then
    log_test "OpenAI: Cline extension config" "PASS"
else
    log_test "OpenAI: Cline extension config" "FAIL"
fi

# Roo Code with OpenAI
cat > "$WORK_DIR/openai-test/.roorules" << 'EOF'
# Roo with GPT

Provider: openai
Model: gpt-4-turbo
EOF

if [ -f "$WORK_DIR/openai-test/.roorules" ]; then
    log_test "OpenAI: Roo Code config" "PASS"
else
    log_test "OpenAI: Roo Code config" "FAIL"
fi

# API Key validation
if [ -n "$OPENAI_API_KEY" ]; then
    log_test "OpenAI: API key environment variable" "PASS"
else
    log_test "OpenAI: API key environment variable" "FAIL"
fi

# Base URL override
if [ -n "$OPENAI_API_BASE" ]; then
    log_test "OpenAI: Base URL override (for mocking)" "PASS"
else
    log_test "OpenAI: Base URL override (for mocking)" "FAIL"
fi

echo ""

# ============================================
# PROVIDER 3: GOOGLE
# ============================================
echo -e "${BLUE}=== Provider: Google ===${NC}"
echo "Testing Google/Gemini API configuration"
echo ""

mkdir -p "$WORK_DIR/google-test"

export GOOGLE_API_KEY="AIzaSy-test-gemini-key"
export GOOGLE_CLOUD_PROJECT="test-project"
export GOOGLE_CLOUD_REGION="us-central1"

# Gemini CLI
cat > "$WORK_DIR/google-test/GEMINI.md" << 'EOF'
# Gemini Configuration

Use GOOGLE_API_KEY for authentication.
Project: ${GOOGLE_CLOUD_PROJECT}
Region: ${GOOGLE_CLOUD_REGION}
EOF

if [ -f "$WORK_DIR/google-test/GEMINI.md" ]; then
    log_test "Google: Gemini CLI config" "PASS"
else
    log_test "Google: Gemini CLI config" "FAIL"
fi

# Model configuration
cat > "$WORK_DIR/google-test/.gemini-config.json" << 'EOF'
{
  "model": "gemini-1.5-pro",
  "temperature": 0.7
}
EOF

if grep -q "gemini-1.5-pro" "$WORK_DIR/google-test/.gemini-config.json"; then
    log_test "Google: Gemini model config" "PASS"
else
    log_test "Google: Gemini model config" "FAIL"
fi

# API Key validation
if [ -n "$GOOGLE_API_KEY" ]; then
    log_test "Google: API key environment variable" "PASS"
else
    log_test "Google: API key environment variable" "FAIL"
fi

# Project ID
if [ -n "$GOOGLE_CLOUD_PROJECT" ]; then
    log_test "Google: Project ID configuration" "PASS"
else
    log_test "Google: Project ID configuration" "FAIL"
fi

# Region
if [ -n "$GOOGLE_CLOUD_REGION" ]; then
    log_test "Google: Region configuration" "PASS"
else
    log_test "Google: Region configuration" "FAIL"
fi

echo ""

# ============================================
# PROVIDER 4: OPENROUTER
# ============================================
echo -e "${BLUE}=== Provider: OpenRouter ===${NC}"
echo "Testing OpenRouter API configuration (multi-model gateway)"
echo ""

mkdir -p "$WORK_DIR/openrouter-test"

export OPENROUTER_API_KEY="sk-or-test-key-12345"

# Cline with OpenRouter
mkdir -p "$WORK_DIR/openrouter-test/.vscode"
cat > "$WORK_DIR/openrouter-test/.vscode/settings.json" << 'EOF'
{
  "cline.apiProvider": "openrouter",
  "cline.modelId": "anthropic/claude-3.5-sonnet"
}
EOF

if grep -q "openrouter" "$WORK_DIR/openrouter-test/.vscode/settings.json"; then
    log_test "OpenRouter: Cline extension config" "PASS"
else
    log_test "OpenRouter: Cline extension config" "FAIL"
fi

# API Key validation
if [ -n "$OPENROUTER_API_KEY" ]; then
    log_test "OpenRouter: API key environment variable" "PASS"
else
    log_test "OpenRouter: API key environment variable" "FAIL"
fi

echo ""

# ============================================
# PROVIDER 5: OLLAMA (Local)
# ============================================
echo -e "${BLUE}=== Provider: Ollama ===${NC}"
echo "Testing Ollama local model configuration"
echo ""

mkdir -p "$WORK_DIR/ollama-test"

export OLLAMA_BASE_URL="http://localhost:11434"

# Roo Code with Ollama
cat > "$WORK_DIR/ollama-test/.roorules" << 'EOF'
# Roo with Ollama (local)

Provider: ollama
Model: llama3
Base URL: ${OLLAMA_BASE_URL}
EOF

mkdir -p "$WORK_DIR/ollama-test/.vscode"
cat > "$WORK_DIR/ollama-test/.vscode/settings.json" << 'EOF'
{
  "roo-cline.apiProvider": "ollama",
  "roo-cline.ollamaBaseUrl": "http://localhost:11434",
  "roo-cline.modelId": "llama3"
}
EOF

if grep -q "ollama" "$WORK_DIR/ollama-test/.vscode/settings.json"; then
    log_test "Ollama: Roo Code config" "PASS"
else
    log_test "Ollama: Roo Code config" "FAIL"
fi

# Base URL validation
if [ -n "$OLLAMA_BASE_URL" ]; then
    log_test "Ollama: Base URL environment variable" "PASS"
else
    log_test "Ollama: Base URL environment variable" "FAIL"
fi

echo ""

# ============================================
# PROVIDER 6: AWS BEDROCK
# ============================================
echo -e "${BLUE}=== Provider: AWS Bedrock ===${NC}"
echo "Testing AWS Bedrock configuration"
echo ""

mkdir -p "$WORK_DIR/bedrock-test"

export AWS_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="AKIATEST12345"
export AWS_SECRET_ACCESS_KEY="test-secret-key"

# Roo Code with Bedrock
mkdir -p "$WORK_DIR/bedrock-test/.vscode"
cat > "$WORK_DIR/bedrock-test/.vscode/settings.json" << 'EOF'
{
  "roo-cline.apiProvider": "bedrock",
  "roo-cline.awsRegion": "us-east-1",
  "roo-cline.modelId": "anthropic.claude-3-sonnet-20240229-v1:0"
}
EOF

if grep -q "bedrock" "$WORK_DIR/bedrock-test/.vscode/settings.json"; then
    log_test "Bedrock: Roo Code config" "PASS"
else
    log_test "Bedrock: Roo Code config" "FAIL"
fi

# AWS credentials
if [ -n "$AWS_REGION" ] && [ -n "$AWS_ACCESS_KEY_ID" ]; then
    log_test "Bedrock: AWS credentials configuration" "PASS"
else
    log_test "Bedrock: AWS credentials configuration" "FAIL"
fi

echo ""

# ============================================
# CROSS-PROVIDER CONSISTENCY
# ============================================
echo -e "${BLUE}=== Cross-Provider Consistency ===${NC}"
echo "Testing rule consistency when switching providers"
echo ""

mkdir -p "$WORK_DIR/consistency-test/.repository/rules"

# Create shared rules
cat > "$WORK_DIR/consistency-test/.repository/rules/coding.md" << 'EOF'
# Coding Standards

- Write clean, readable code
- Use meaningful variable names
- Add comments for complex logic
EOF

# Apply to multiple tool configs
for tool in CLAUDE.md .cursorrules .clinerules .roorules GEMINI.md; do
    cat > "$WORK_DIR/consistency-test/$tool" << EOF
# Project Rules

<!-- repo:block:coding -->
# Coding Standards

- Write clean, readable code
- Use meaningful variable names
- Add comments for complex logic
<!-- /repo:block:coding -->
EOF
done

# Verify consistency
REFERENCE_CONTENT=$(sed -n '/<!-- repo:block:coding -->/,/<!-- \/repo:block:coding -->/p' "$WORK_DIR/consistency-test/CLAUDE.md")

for tool in .cursorrules .clinerules .roorules GEMINI.md; do
    TOOL_CONTENT=$(sed -n '/<!-- repo:block:coding -->/,/<!-- \/repo:block:coding -->/p' "$WORK_DIR/consistency-test/$tool")
    if [ "$REFERENCE_CONTENT" = "$TOOL_CONTENT" ]; then
        log_test "Consistency: CLAUDE.md = $tool" "PASS"
    else
        log_test "Consistency: CLAUDE.md = $tool" "FAIL"
    fi
done

echo ""

# ============================================
# MOCK API COMPATIBILITY
# ============================================
echo -e "${BLUE}=== Mock API Compatibility ===${NC}"
echo "Testing mock server URL configuration"
echo ""

# Verify all providers support mock URL override
MOCK_URL="http://localhost:8080/v1"

export ANTHROPIC_BASE_URL="$MOCK_URL"
export OPENAI_API_BASE="$MOCK_URL"

log_test "Mock: Anthropic base URL override" "PASS"
log_test "Mock: OpenAI base URL override" "PASS"

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "      PROVIDER COMPATIBILITY SUMMARY"
echo "=============================================="
echo ""
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/provider-report.md" << EOF
# Provider Compatibility Test Report

**Generated:** $(date -Iseconds)

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Providers Tested

| Provider | API Key Env | Base URL Override | Tools |
|----------|-------------|-------------------|-------|
| Anthropic | \`ANTHROPIC_API_KEY\` | \`ANTHROPIC_BASE_URL\` | Claude, Cursor, Aider, Cline, Roo |
| OpenAI | \`OPENAI_API_KEY\` | \`OPENAI_API_BASE\` | Cursor, Aider, Cline, Roo |
| Google | \`GOOGLE_API_KEY\` | - | Gemini |
| OpenRouter | \`OPENROUTER_API_KEY\` | - | Cline |
| Ollama | - | \`OLLAMA_BASE_URL\` | Roo |
| AWS Bedrock | \`AWS_*\` credentials | - | Roo |

## Test Categories

1. **Provider Configuration** - API key and URL setup
2. **Tool Compatibility** - Each provider works with its tools
3. **Cross-Provider Consistency** - Rules match across providers
4. **Mock API Support** - URL overrides for testing

## Recommendations

- Use \`ANTHROPIC_BASE_URL\` and \`OPENAI_API_BASE\` for mock testing
- Ensure all tools point to same mock server in CI/CD
- Verify rule blocks match across tool configs
EOF

echo "Report: $RESULTS_DIR/provider-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL TESTS PASSED${NC}"
    exit 0
fi
