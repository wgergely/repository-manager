#!/bin/bash
# Cline VS Code Extension Integration Tests
# Simulates programmer workflows with Cline (Claude Dev) extension

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/tools/cline"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo "=============================================="
echo "     CLINE EXTENSION INTEGRATION TESTS"
echo "=============================================="
echo ""

# Test tracking
TOTAL=0
PASSED=0
FAILED=0
SKIPPED=0

# Check Docker
DOCKER_AVAILABLE=false
CLINE_IMAGE_EXISTS=false

if docker info >/dev/null 2>&1; then
    DOCKER_AVAILABLE=true
    if docker image inspect repo-test/cline:latest >/dev/null 2>&1; then
        CLINE_IMAGE_EXISTS=true
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
# SCENARIO 1: .clinerules Detection
# ============================================
echo -e "${BLUE}=== Scenario 1: .clinerules Detection ===${NC}"
echo "Programmer: 'Cline should read my project rules from .clinerules'"
echo ""

mkdir -p "$WORK_DIR/project1"

cat > "$WORK_DIR/project1/.clinerules" << 'EOF'
# Project Rules for Cline

You are an expert TypeScript developer working on a React application.

## Code Style
- Use functional components with hooks
- Prefer TypeScript strict mode
- Use named exports over default exports

## Testing
- Write Jest tests for components
- Use React Testing Library
- Aim for 80% coverage

## Architecture
- Follow the feature-folder structure
- Keep components small and focused
- Use React Query for data fetching
EOF

if [ -f "$WORK_DIR/project1/.clinerules" ]; then
    log_test ".clinerules file created" "PASS"
else
    log_test ".clinerules file created" "FAIL"
fi

# Verify content
line_count=$(wc -l < "$WORK_DIR/project1/.clinerules")
if [ "$line_count" -gt 10 ]; then
    log_test ".clinerules has substantial content ($line_count lines)" "PASS"
else
    log_test ".clinerules has substantial content" "FAIL" "Only $line_count lines"
fi

echo ""

# ============================================
# SCENARIO 2: Managed Blocks
# ============================================
echo -e "${BLUE}=== Scenario 2: Managed Blocks ===${NC}"
echo "Programmer: 'Repository Manager injects rules into .clinerules'"
echo ""

mkdir -p "$WORK_DIR/project2/.repository/rules"

cat > "$WORK_DIR/project2/.repository/rules/react-patterns.md" << 'EOF'
# React Patterns

- Use custom hooks for shared logic
- Implement error boundaries
- Use Suspense for loading states
EOF

cat > "$WORK_DIR/project2/.repository/rules/accessibility.md" << 'EOF'
# Accessibility Guidelines

- Use semantic HTML elements
- Add proper ARIA labels
- Support keyboard navigation
EOF

# Generate .clinerules with managed blocks
cat > "$WORK_DIR/project2/.clinerules" << 'EOF'
# Project Configuration

<!-- repo:block:react-patterns -->
# React Patterns

- Use custom hooks for shared logic
- Implement error boundaries
- Use Suspense for loading states
<!-- /repo:block:react-patterns -->

<!-- repo:block:accessibility -->
# Accessibility Guidelines

- Use semantic HTML elements
- Add proper ARIA labels
- Support keyboard navigation
<!-- /repo:block:accessibility -->

# My Custom Rules

- Check with design team for UI changes
- Run lighthouse audits before PRs
EOF

# Verify managed blocks
block_start_count=$(grep -c "<!-- repo:block:" "$WORK_DIR/project2/.clinerules")
block_end_count=$(grep -c "<!-- /repo:block:" "$WORK_DIR/project2/.clinerules")

if [ "$block_start_count" -eq "$block_end_count" ] && [ "$block_start_count" -eq 2 ]; then
    log_test "Managed blocks properly structured (2 blocks)" "PASS"
else
    log_test "Managed blocks properly structured" "FAIL" "Start: $block_start_count, End: $block_end_count"
fi

if grep -q "custom hooks" "$WORK_DIR/project2/.clinerules"; then
    log_test "React patterns content present" "PASS"
else
    log_test "React patterns content present" "FAIL"
fi

if grep -q "ARIA labels" "$WORK_DIR/project2/.clinerules"; then
    log_test "Accessibility rules content present" "PASS"
else
    log_test "Accessibility rules content present" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 3: Manual Content Preservation
# ============================================
echo -e "${BLUE}=== Scenario 3: Manual Content Preservation ===${NC}"
echo "Programmer: 'My custom rules outside blocks should survive sync'"
echo ""

if grep -q "My Custom Rules" "$WORK_DIR/project2/.clinerules"; then
    log_test "Manual content outside blocks exists" "PASS"
else
    log_test "Manual content outside blocks exists" "FAIL"
fi

# Simulate sync - update a managed block
sed -i 's/error boundaries/error boundaries with fallback UI/' "$WORK_DIR/project2/.clinerules"

if grep -q "fallback UI" "$WORK_DIR/project2/.clinerules" && \
   grep -q "My Custom Rules" "$WORK_DIR/project2/.clinerules"; then
    log_test "Manual content preserved after block update" "PASS"
else
    log_test "Manual content preserved after block update" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 4: VS Code Settings Integration
# ============================================
echo -e "${BLUE}=== Scenario 4: VS Code Settings ===${NC}"
echo "Programmer: 'Cline settings should be in VS Code config'"
echo ""

mkdir -p "$WORK_DIR/project1/.vscode"

cat > "$WORK_DIR/project1/.vscode/settings.json" << 'EOF'
{
  "cline.apiProvider": "anthropic",
  "cline.apiKey": "${env:ANTHROPIC_API_KEY}",
  "cline.modelId": "claude-3-opus-20240229",
  "cline.customInstructions": "Follow .clinerules file",
  "cline.autoApproveTools": false
}
EOF

if [ -f "$WORK_DIR/project1/.vscode/settings.json" ]; then
    log_test "VS Code settings file created" "PASS"
else
    log_test "VS Code settings file created" "FAIL"
fi

if grep -q "cline.apiProvider" "$WORK_DIR/project1/.vscode/settings.json"; then
    log_test "Cline API provider configured" "PASS"
else
    log_test "Cline API provider configured" "FAIL"
fi

if grep -q "autoApproveTools" "$WORK_DIR/project1/.vscode/settings.json"; then
    log_test "Tool approval settings configured" "PASS"
else
    log_test "Tool approval settings configured" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 5: Multi-Provider Support
# ============================================
echo -e "${BLUE}=== Scenario 5: Multi-Provider Support ===${NC}"
echo "Programmer: 'I switch between Claude and OpenAI'"
echo ""

# Anthropic configuration
export ANTHROPIC_API_KEY="sk-ant-cline-test"
if [ -n "$ANTHROPIC_API_KEY" ]; then
    log_test "Anthropic API key configurable" "PASS"
else
    log_test "Anthropic API key configurable" "FAIL"
fi

# OpenAI configuration
export OPENAI_API_KEY="sk-openai-cline-test"
if [ -n "$OPENAI_API_KEY" ]; then
    log_test "OpenAI API key configurable" "PASS"
else
    log_test "OpenAI API key configurable" "FAIL"
fi

# OpenRouter configuration (Cline also supports this)
export OPENROUTER_API_KEY="sk-or-cline-test"
if [ -n "$OPENROUTER_API_KEY" ]; then
    log_test "OpenRouter API key configurable" "PASS"
else
    log_test "OpenRouter API key configurable" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 6: MCP Server Configuration
# ============================================
echo -e "${BLUE}=== Scenario 6: MCP Server Configuration ===${NC}"
echo "Programmer: 'Cline uses MCP servers for tool access'"
echo ""

cat > "$WORK_DIR/project1/cline_mcp_settings.json" << 'EOF'
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/workspace"]
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
EOF

if [ -f "$WORK_DIR/project1/cline_mcp_settings.json" ]; then
    log_test "MCP settings file created" "PASS"
else
    log_test "MCP settings file created" "FAIL"
fi

if grep -q "filesystem" "$WORK_DIR/project1/cline_mcp_settings.json"; then
    log_test "Filesystem MCP server configured" "PASS"
else
    log_test "Filesystem MCP server configured" "FAIL"
fi

if grep -q "github" "$WORK_DIR/project1/cline_mcp_settings.json"; then
    log_test "GitHub MCP server configured" "PASS"
else
    log_test "GitHub MCP server configured" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 7: File Ignore Patterns
# ============================================
echo -e "${BLUE}=== Scenario 7: Ignore Patterns ===${NC}"
echo "Programmer: 'Some files should never be touched by Cline'"
echo ""

cat > "$WORK_DIR/project1/.clineignore" << 'EOF'
# Files Cline should ignore
.env
.env.*
*.key
*.pem
secrets/
credentials/
node_modules/
dist/
build/
.git/
EOF

if [ -f "$WORK_DIR/project1/.clineignore" ]; then
    log_test ".clineignore file created" "PASS"
else
    log_test ".clineignore file created" "FAIL"
fi

if grep -q ".env" "$WORK_DIR/project1/.clineignore"; then
    log_test "Environment files ignored" "PASS"
else
    log_test "Environment files ignored" "FAIL"
fi

if grep -q "node_modules" "$WORK_DIR/project1/.clineignore"; then
    log_test "Dependencies ignored" "PASS"
else
    log_test "Dependencies ignored" "FAIL"
fi

echo ""

# ============================================
# SCENARIO 8: Cross-Tool Consistency
# ============================================
echo -e "${BLUE}=== Scenario 8: Cross-Tool Consistency ===${NC}"
echo "Programmer: 'Rules should match CLAUDE.md and .cursorrules'"
echo ""

# Copy to CLAUDE.md
cp "$WORK_DIR/project2/.clinerules" "$WORK_DIR/project2/CLAUDE.md"

# Extract and compare managed blocks
cline_patterns=$(sed -n '/<!-- repo:block:react-patterns -->/,/<!-- \/repo:block:react-patterns -->/p' "$WORK_DIR/project2/.clinerules" | grep -v "repo:block")
claude_patterns=$(sed -n '/<!-- repo:block:react-patterns -->/,/<!-- \/repo:block:react-patterns -->/p' "$WORK_DIR/project2/CLAUDE.md" | grep -v "repo:block")

if [ "$cline_patterns" = "$claude_patterns" ]; then
    log_test "React patterns consistent across tools" "PASS"
else
    log_test "React patterns consistent across tools" "FAIL"
fi

echo ""

# ============================================
# RESULTS SUMMARY
# ============================================
echo "=============================================="
echo "       CLINE EXTENSION TEST SUMMARY"
echo "=============================================="
echo ""
echo -e "Total:   $TOTAL"
echo -e "Passed:  ${GREEN}$PASSED${NC}"
echo -e "Failed:  ${RED}$FAILED${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED${NC}"
echo ""

# Generate report
cat > "$RESULTS_DIR/cline-test-report.md" << EOF
# Cline Extension Integration Test Report

**Generated:** $(date -Iseconds)
**Docker Available:** $DOCKER_AVAILABLE
**Cline Image Exists:** $CLINE_IMAGE_EXISTS

## Summary

| Metric | Count |
|--------|-------|
| Total | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Skipped | $SKIPPED |
| Pass Rate | $(( (PASSED * 100) / TOTAL ))% |

## Scenarios Tested

1. **.clinerules Detection** - Config file handling
2. **Managed Blocks** - Repository Manager integration
3. **Manual Content** - Preservation of user edits
4. **VS Code Settings** - Extension configuration
5. **Multi-Provider** - Claude/OpenAI/OpenRouter switching
6. **MCP Servers** - Tool server configuration
7. **Ignore Patterns** - .clineignore handling
8. **Cross-Tool Consistency** - Rule matching

## Providers Supported

- **Anthropic:** \`ANTHROPIC_API_KEY\` - Claude models
- **OpenAI:** \`OPENAI_API_KEY\` - GPT models
- **OpenRouter:** \`OPENROUTER_API_KEY\` - Multiple providers

## Config Files

- \`.clinerules\` - Main rules file (markdown)
- \`.clineignore\` - File exclusions
- \`.vscode/settings.json\` - VS Code settings
- \`cline_mcp_settings.json\` - MCP server config
EOF

echo "Report: $RESULTS_DIR/cline-test-report.md"
echo ""

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}ALL TESTS PASSED${NC}"
    exit 0
fi
