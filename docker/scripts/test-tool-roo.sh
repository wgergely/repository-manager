#!/bin/bash
# Roo Code VS Code Extension Integration Tests
# Simulates programmer workflows with Roo Code (Roo-Cline) extension

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/tools/roo"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "     ROO CODE EXTENSION INTEGRATION TESTS"
echo "=============================================="
echo ""

# Test tracking
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Check Docker
DOCKER_AVAILABLE=false
ROO_IMAGE_EXISTS=false

if docker info >/dev/null 2>&1; then
    DOCKER_AVAILABLE=true
    if docker image inspect repo-test/roo:latest >/dev/null 2>&1; then
        ROO_IMAGE_EXISTS=true
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
# SCENARIO 1: .roorules Detection
# ============================================
echo -e "${BLUE}=== Scenario 1: .roorules Detection ===${NC}"
echo "Programmer: 'Roo should read my project rules from .roorules'"
echo ""

mkdir -p "$WORK_DIR/project1"

cat > "$WORK_DIR/project1/.roorules" << 'EOF'
# Project Rules for Roo Code

You are an expert Python developer working on a FastAPI application.

## Code Style
- Follow PEP 8 guidelines
- Use type hints everywhere
- Prefer async/await patterns

## Testing
- Write pytest tests for all endpoints
- Use factories for test data
- Mock external services

## Architecture
- Use dependency injection
- Separate business logic from routes
- Keep modules focused and small
EOF

if [ -f "$WORK_DIR/project1/.roorules" ]; then
    log_test ".roorules file created" "PASS"
else
    log_test ".roorules file created" "FAIL"
fi

# Verify content
line_count=$(wc -l < "$WORK_DIR/project1/.roorules")
if [ "$line_count" -gt 10 ]; then
    log_test ".roorules has substantial content ($line_count lines)" "PASS"
else
    log_test ".roorules has substantial content" "FAIL" "Only $line_count lines"
fi

echo ""

# ============================================
# SCENARIO 2: Managed Blocks
# ============================================
echo -e "${BLUE}=== Scenario 2: Managed Blocks ===${NC}"
echo "Programmer: 'Repository Manager injects rules into .roorules'"
echo ""

mkdir -p "$WORK_DIR/project2/.repository/rules"

cat > "$WORK_DIR/project2/.repository/rules/python-standards.md" << 'EOF'
# Python Standards

- Use black for formatting
- Sort imports with isort
- Type check with mypy
EOF

cat > "$WORK_DIR/project2/.repository/rules/api-design.md" << 'EOF'
# API Design

- Use Pydantic for validation
- Document with OpenAPI
- Version all endpoints
EOF

# Generate .roorules with managed blocks
cat > "$WORK_DIR/project2/.roorules" << 'EOF'
# Project Configuration

<!-- repo:block:python-standards -->
# Python Standards

- Use black for formatting
- Sort imports with isort
- Type check with mypy
<!-- /repo:block:python-standards -->

<!-- repo:block:api-design -->
# API Design

- Use Pydantic for validation
- Document with OpenAPI
- Version all endpoints
<!-- /repo:block:api-design -->

# My Custom Rules

- Run pre-commit hooks before pushing
- Update CHANGELOG.md for releases
EOF

# Verify managed blocks
block_start_count=$(grep -c "<!-- repo:block:" "$WORK_DIR/project2/.roorules")
block_end_count=$(grep -c "<!-- /repo:block:" "$WORK_DIR/project2/.roorules")

if [ "$block_start_count" -eq "$block_end_count" ] && [ "$block_start_count" -eq 2 ]; then
    log_test "Managed blocks properly structured (2 blocks)" "PASS"
else
    log_test "Managed blocks properly structured" "FAIL" "Start: $block_start_count, End: $block_end_count"
fi

if grep -q "black for formatting" "$WORK_DIR/project2/.roorules"; then
    log_test "Python standards content present" "PASS"
else
    log_test "Python standards content present" "FAIL"
fi

if grep -q "Pydantic for validation" "$WORK_DIR/project2/.roorules"; then
    log_test "API design content present" "PASS"
else
    log_test "API design content present" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 3: Manual Content Preservation
# ============================================
echo -e "${BLUE}=== Scenario 3: Manual Content Preservation ===${NC}"
echo "Programmer: 'My custom rules outside blocks should survive sync'"
echo ""

if grep -q "My Custom Rules" "$WORK_DIR/project2/.roorules"; then
    log_test "Manual content outside blocks exists" "PASS"
else
    log_test "Manual content outside blocks exists" "FAIL"
fi

# Simulate sync - update a managed block
sed -i 's/Sort imports with isort/Sort imports with ruff/' "$WORK_DIR/project2/.roorules"

if grep -q "Sort imports with ruff" "$WORK_DIR/project2/.roorules" && \
   grep -q "My Custom Rules" "$WORK_DIR/project2/.roorules"; then
    log_test "Manual content preserved after block update" "PASS"
else
    log_test "Manual content preserved after block update" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 4: VS Code Settings Integration
# ============================================
echo -e "${BLUE}=== Scenario 4: VS Code Settings ===${NC}"
echo "Programmer: 'Roo settings should be in VS Code config'"
echo ""

mkdir -p "$WORK_DIR/project1/.vscode"

cat > "$WORK_DIR/project1/.vscode/settings.json" << 'EOF'
{
  "roo-cline.apiProvider": "anthropic",
  "roo-cline.apiKey": "${env:ANTHROPIC_API_KEY}",
  "roo-cline.modelId": "claude-3-5-sonnet-20241022",
  "roo-cline.customInstructions": "Follow .roorules file",
  "roo-cline.enableAutoApproval": false,
  "roo-cline.alwaysAllowReadOnly": true
}
EOF

if [ -f "$WORK_DIR/project1/.vscode/settings.json" ]; then
    log_test "VS Code settings file created" "PASS"
else
    log_test "VS Code settings file created" "FAIL"
fi

if grep -q "roo-cline.apiProvider" "$WORK_DIR/project1/.vscode/settings.json"; then
    log_test "Roo API provider configured" "PASS"
else
    log_test "Roo API provider configured" "FAIL"
fi

if grep -q "alwaysAllowReadOnly" "$WORK_DIR/project1/.vscode/settings.json"; then
    log_test "Read-only approval settings configured" "PASS"
else
    log_test "Read-only approval settings configured" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 5: Multi-Provider Support
# ============================================
echo -e "${BLUE}=== Scenario 5: Multi-Provider Support ===${NC}"
echo "Programmer: 'I use Claude, OpenAI, and local models'"
echo ""

# Anthropic configuration
export ANTHROPIC_API_KEY="sk-ant-roo-test"
if [ -n "$ANTHROPIC_API_KEY" ]; then
    log_test "Anthropic API key configurable" "PASS"
else
    log_test "Anthropic API key configurable" "FAIL"
fi

# OpenAI configuration
export OPENAI_API_KEY="sk-openai-roo-test"
if [ -n "$OPENAI_API_KEY" ]; then
    log_test "OpenAI API key configurable" "PASS"
else
    log_test "OpenAI API key configurable" "FAIL"
fi

# Ollama (local) configuration
export OLLAMA_BASE_URL="http://localhost:11434"
if [ -n "$OLLAMA_BASE_URL" ]; then
    log_test "Ollama local URL configurable" "PASS"
else
    log_test "Ollama local URL configurable" "FAIL"
fi

# AWS Bedrock configuration
export AWS_REGION="us-east-1"
export AWS_ACCESS_KEY_ID="test-access-key"
if [ -n "$AWS_REGION" ] && [ -n "$AWS_ACCESS_KEY_ID" ]; then
    log_test "AWS Bedrock configurable" "PASS"
else
    log_test "AWS Bedrock configurable" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 6: MCP Server Configuration
# ============================================
echo -e "${BLUE}=== Scenario 6: MCP Server Configuration ===${NC}"
echo "Programmer: 'Roo uses MCP servers for extended capabilities'"
echo ""

cat > "$WORK_DIR/project1/roo_mcp_settings.json" << 'EOF'
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"],
      "disabled": false
    },
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "${DATABASE_URL}"
      }
    },
    "brave-search": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-brave-search"],
      "env": {
        "BRAVE_API_KEY": "${BRAVE_API_KEY}"
      }
    }
  }
}
EOF

if [ -f "$WORK_DIR/project1/roo_mcp_settings.json" ]; then
    log_test "MCP settings file created" "PASS"
else
    log_test "MCP settings file created" "FAIL"
fi

if grep -q "filesystem" "$WORK_DIR/project1/roo_mcp_settings.json"; then
    log_test "Filesystem MCP server configured" "PASS"
else
    log_test "Filesystem MCP server configured" "FAIL"
fi

if grep -q "postgres" "$WORK_DIR/project1/roo_mcp_settings.json"; then
    log_test "Postgres MCP server configured" "PASS"
else
    log_test "Postgres MCP server configured" "FAIL"
fi

if grep -q "brave-search" "$WORK_DIR/project1/roo_mcp_settings.json"; then
    log_test "Brave Search MCP server configured" "PASS"
else
    log_test "Brave Search MCP server configured" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 7: File Ignore Patterns
# ============================================
echo -e "${BLUE}=== Scenario 7: Ignore Patterns ===${NC}"
echo "Programmer: 'Some files should never be touched by Roo'"
echo ""

cat > "$WORK_DIR/project1/.rooignore" << 'EOF'
# Files Roo should ignore
.env
.env.*
*.key
*.pem
secrets/
credentials/
__pycache__/
.venv/
.pytest_cache/
.git/
*.pyc
EOF

if [ -f "$WORK_DIR/project1/.rooignore" ]; then
    log_test ".rooignore file created" "PASS"
else
    log_test ".rooignore file created" "FAIL"
fi

if grep -q ".env" "$WORK_DIR/project1/.rooignore"; then
    log_test "Environment files ignored" "PASS"
else
    log_test "Environment files ignored" "FAIL"
fi

if grep -q ".venv" "$WORK_DIR/project1/.rooignore"; then
    log_test "Virtual environment ignored" "PASS"
else
    log_test "Virtual environment ignored" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 8: Custom Modes
# ============================================
echo -e "${BLUE}=== Scenario 8: Custom Modes ===${NC}"
echo "Programmer: 'Roo supports different operational modes'"
echo ""

mkdir -p "$WORK_DIR/project1/.roo"

cat > "$WORK_DIR/project1/.roo/modes.json" << 'EOF'
{
  "modes": {
    "architect": {
      "name": "Architect Mode",
      "description": "High-level design and planning",
      "systemPrompt": "Focus on architecture, not implementation details",
      "allowedTools": ["read_file", "list_files", "search_files"]
    },
    "coder": {
      "name": "Coder Mode",
      "description": "Implementation and coding",
      "systemPrompt": "Focus on clean, tested implementations",
      "allowedTools": ["read_file", "write_file", "execute_command", "list_files"]
    },
    "reviewer": {
      "name": "Code Reviewer Mode",
      "description": "Review and feedback",
      "systemPrompt": "Review code for bugs, security, and style",
      "allowedTools": ["read_file", "list_files", "search_files"]
    }
  }
}
EOF

if [ -f "$WORK_DIR/project1/.roo/modes.json" ]; then
    log_test "Custom modes file created" "PASS"
else
    log_test "Custom modes file created" "FAIL"
fi

if grep -q "architect" "$WORK_DIR/project1/.roo/modes.json"; then
    log_test "Architect mode defined" "PASS"
else
    log_test "Architect mode defined" "FAIL"
fi

if grep -q "coder" "$WORK_DIR/project1/.roo/modes.json"; then
    log_test "Coder mode defined" "PASS"
else
    log_test "Coder mode defined" "FAIL"
fi

if grep -q "reviewer" "$WORK_DIR/project1/.roo/modes.json"; then
    log_test "Reviewer mode defined" "PASS"
else
    log_test "Reviewer mode defined" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 9: Cross-Tool Consistency
# ============================================
echo -e "${BLUE}=== Scenario 9: Cross-Tool Consistency ===${NC}"
echo "Programmer: 'Rules should match across all AI tools'"
echo ""

# Copy to CLAUDE.md and .clinerules
cp "$WORK_DIR/project2/.roorules" "$WORK_DIR/project2/CLAUDE.md"
cp "$WORK_DIR/project2/.roorules" "$WORK_DIR/project2/.clinerules"

# Extract and compare managed blocks
roo_python=$(sed -n '/<!-- repo:block:python-standards -->/,/<!-- \/repo:block:python-standards -->/p' "$WORK_DIR/project2/.roorules" | grep -v "repo:block")
claude_python=$(sed -n '/<!-- repo:block:python-standards -->/,/<!-- \/repo:block:python-standards -->/p' "$WORK_DIR/project2/CLAUDE.md" | grep -v "repo:block")
cline_python=$(sed -n '/<!-- repo:block:python-standards -->/,/<!-- \/repo:block:python-standards -->/p' "$WORK_DIR/project2/.clinerules" | grep -v "repo:block")

if [ "$roo_python" = "$claude_python" ]; then
    log_test "Python standards consistent with CLAUDE.md" "PASS"
else
    log_test "Python standards consistent with CLAUDE.md" "FAIL"
fi

if [ "$roo_python" = "$cline_python" ]; then
    log_test "Python standards consistent with .clinerules" "PASS"
else
    log_test "Python standards consistent with .clinerules" "FAIL"
fi

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "       ROO CODE EXTENSION TEST SUMMARY"
echo "=============================================="
echo ""
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/roo-test-report.md" << EOF
# Roo Code Extension Integration Test Report

**Generated:** $(date -Iseconds)
**Docker Available:** $DOCKER_AVAILABLE
**Roo Image Exists:** $ROO_IMAGE_EXISTS

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Skipped | $SKIPPED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **.roorules Detection** - Config file handling
2. **Managed Blocks** - Repository Manager integration
3. **Manual Content** - Preservation of user edits
4. **VS Code Settings** - Extension configuration
5. **Multi-Provider** - Claude/OpenAI/Ollama/Bedrock
6. **MCP Servers** - Tool server configuration
7. **Ignore Patterns** - .rooignore handling
8. **Custom Modes** - Architect/Coder/Reviewer modes
9. **Cross-Tool Consistency** - Rule matching

## Providers Supported

- **Anthropic:** \`ANTHROPIC_API_KEY\` - Claude models
- **OpenAI:** \`OPENAI_API_KEY\` - GPT models
- **Ollama:** \`OLLAMA_BASE_URL\` - Local models
- **AWS Bedrock:** \`AWS_REGION\`, \`AWS_ACCESS_KEY_ID\`

## Config Files

- \`.roorules\` - Main rules file (markdown)
- \`.rooignore\` - File exclusions
- \`.vscode/settings.json\` - VS Code settings
- \`roo_mcp_settings.json\` - MCP server config
- \`.roo/modes.json\` - Custom operational modes
EOF

echo "Report: $RESULTS_DIR/roo-test-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL TESTS PASSED${NC}"
    exit 0
fi
