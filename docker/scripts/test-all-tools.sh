#!/bin/bash
# Master Tool Test Runner
# Runs all tool-specific integration tests and generates comprehensive matrix

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
RESULTS_DIR="$PROJECT_ROOT/test-results/tools"
MATRIX_FILE="$RESULTS_DIR/TEST-MATRIX.md"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

mkdir -p "$RESULTS_DIR"

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║          REPOSITORY MANAGER - TOOL INTEGRATION TESTS         ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Testing configuration management across all supported AI tools"
echo ""

# Tool definitions
declare -A TOOLS
TOOLS["claude"]="Claude CLI|test-tool-claude.sh|CLAUDE.md|Anthropic"
TOOLS["cursor"]="Cursor IDE|test-tool-cursor.sh|.cursorrules|Anthropic,OpenAI"
TOOLS["aider"]="Aider|test-tool-aider.sh|.aider.conf.yml|Anthropic,OpenAI"
TOOLS["gemini"]="Gemini CLI|test-tool-gemini.sh|GEMINI.md|Google"
TOOLS["cline"]="Cline Extension|test-tool-cline.sh|.clinerules|Anthropic,OpenAI,OpenRouter"
TOOLS["roo"]="Roo Code|test-tool-roo.sh|.roorules|Anthropic,OpenAI,Ollama,Bedrock"

# Results tracking
declare -A TOOL_RESULTS
TOTAL_TOOLS=0
PASSED_TOOLS=0
FAILED_TOOLS=0
TOTAL_TESTS=0
TOTAL_PASSED=0
TOTAL_FAILED=0

# Run a tool test and capture results
run_tool_test() {
    local tool_key="$1"
    local tool_info="${TOOLS[$tool_key]}"

    IFS='|' read -r tool_name script_name config_file providers <<< "$tool_info"

    TOTAL_TOOLS=$((TOTAL_TOOLS + 1))

    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Testing: $tool_name${NC}"
    echo -e "Config: $config_file | Providers: $providers"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    local start_time=$(date +%s)
    local output_file="$RESULTS_DIR/${tool_key}/test-output.log"
    mkdir -p "$(dirname "$output_file")"

    if bash "$SCRIPT_DIR/$script_name" > "$output_file" 2>&1; then
        local status="PASS"
        PASSED_TOOLS=$((PASSED_TOOLS + 1))
    else
        local status="FAIL"
        FAILED_TOOLS=$((FAILED_TOOLS + 1))
    fi

    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Extract test counts from output (strip ANSI codes first)
    local clean_output=$(sed 's/\x1b\[[0-9;]*m//g' "$output_file")
    local total=$(echo "$clean_output" | grep -o "Total:[[:space:]]*[0-9]*" | grep -o "[0-9]*" | head -1 || echo "0")
    local passed=$(echo "$clean_output" | grep -o "Passed:[[:space:]]*[0-9]*" | grep -o "[0-9]*" | head -1 || echo "0")
    local failed=$(echo "$clean_output" | grep -o "Failed:[[:space:]]*[0-9]*" | grep -o "[0-9]*" | head -1 || echo "0")

    TOTAL_TESTS=$((TOTAL_TESTS + total))
    TOTAL_PASSED=$((TOTAL_PASSED + passed))
    TOTAL_FAILED=$((TOTAL_FAILED + failed))

    # Store result
    TOOL_RESULTS[$tool_key]="$status|$total|$passed|$failed|$duration"

    # Display summary
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✓ $tool_name: $passed/$total tests passed (${duration}s)${NC}"
    else
        echo -e "${RED}✗ $tool_name: $passed/$total tests passed, $failed failed (${duration}s)${NC}"
        # Show last few lines of error
        echo ""
        echo "Last output:"
        tail -5 "$output_file"
    fi
    echo ""
}

# Run all tool tests
echo -e "${BOLD}Running Tool-Specific Integration Tests${NC}"
echo ""

for tool_key in "${!TOOLS[@]}"; do
    run_tool_test "$tool_key"
done

# Generate test matrix
echo ""
echo -e "${BOLD}Generating Test Matrix...${NC}"
echo ""

cat > "$MATRIX_FILE" << EOF
# Tool Integration Test Matrix

**Generated:** $(date -Iseconds)
**Platform:** $(uname -s)

## Summary

| Metric | Count |
|--------|-------|
| Tools Tested | $TOTAL_TOOLS |
| Tools Passing | $PASSED_TOOLS |
| Tools Failing | $FAILED_TOOLS |
| Total Tests | $TOTAL_TESTS |
| Tests Passed | $TOTAL_PASSED |
| Tests Failed | $TOTAL_FAILED |
| Pass Rate | $(( (TOTAL_PASSED * 100) / (TOTAL_TESTS > 0 ? TOTAL_TESTS : 1) ))% |

## Tool Results

| Tool | Config File | Providers | Status | Tests | Passed | Failed | Time |
|------|-------------|-----------|--------|-------|--------|--------|------|
EOF

for tool_key in claude cursor aider gemini cline roo; do
    tool_info="${TOOLS[$tool_key]}"
    IFS='|' read -r tool_name script_name config_file providers <<< "$tool_info"

    result="${TOOL_RESULTS[$tool_key]}"
    IFS='|' read -r status total passed failed duration <<< "$result"

    status_icon=$([ "$status" = "PASS" ] && echo "✅" || echo "❌")

    echo "| $tool_name | \`$config_file\` | $providers | $status_icon | $total | $passed | $failed | ${duration}s |" >> "$MATRIX_FILE"
done

cat >> "$MATRIX_FILE" << 'EOF'

## Scenarios Tested Per Tool

### CLI Tools

| Scenario | Claude | Aider | Gemini | Cursor |
|----------|--------|-------|--------|--------|
| Config Detection | ✓ | ✓ | ✓ | ✓ |
| Managed Blocks | ✓ | ✓ | ✓ | ✓ |
| Manual Content Preservation | ✓ | ✓ | ✓ | ✓ |
| Multi-Language Projects | ✓ | - | ✓ | ✓ |
| Provider Configuration | ✓ | ✓ | ✓ | ✓ |
| Ignore Patterns | - | ✓ | ✓ | ✓ |
| Cross-Tool Consistency | ✓ | - | ✓ | ✓ |
| Error Handling | ✓ | - | - | - |

### VS Code Extensions

| Scenario | Cline | Roo Code |
|----------|-------|----------|
| Rules File Detection | ✓ | ✓ |
| Managed Blocks | ✓ | ✓ |
| Manual Content Preservation | ✓ | ✓ |
| VS Code Settings | ✓ | ✓ |
| Multi-Provider Support | ✓ | ✓ |
| MCP Server Configuration | ✓ | ✓ |
| Ignore Patterns | ✓ | ✓ |
| Cross-Tool Consistency | ✓ | ✓ |
| Custom Modes | - | ✓ |

## Provider Support Matrix

| Provider | API Key Env | Tools Supporting |
|----------|-------------|------------------|
| Anthropic | `ANTHROPIC_API_KEY` | Claude, Cursor, Aider, Cline, Roo |
| OpenAI | `OPENAI_API_KEY` | Cursor, Aider, Cline, Roo |
| Google | `GOOGLE_API_KEY` | Gemini |
| OpenRouter | `OPENROUTER_API_KEY` | Cline |
| Ollama | `OLLAMA_BASE_URL` | Roo |
| AWS Bedrock | `AWS_REGION`, `AWS_ACCESS_KEY_ID` | Roo |

## Config File Formats

| Tool | Config File | Format | Managed Block Syntax |
|------|-------------|--------|---------------------|
| Claude CLI | `CLAUDE.md` | Markdown | `<!-- repo:block:name -->` |
| Cursor | `.cursorrules` | Markdown | `<!-- repo:block:name -->` |
| Aider | `.aider.conf.yml` | YAML | `# repo:block:name` |
| Gemini | `GEMINI.md` | Markdown | `<!-- repo:block:name -->` |
| Cline | `.clinerules` | Markdown | `<!-- repo:block:name -->` |
| Roo Code | `.roorules` | Markdown | `<!-- repo:block:name -->` |

## Test Reports

Individual tool reports are available in:
EOF

for tool_key in claude cursor aider gemini cline roo; do
    tool_info="${TOOLS[$tool_key]}"
    IFS='|' read -r tool_name script_name config_file providers <<< "$tool_info"
    echo "- \`test-results/tools/${tool_key}/${tool_key}-test-report.md\`" >> "$MATRIX_FILE"
done

echo "" >> "$MATRIX_FILE"
echo "---" >> "$MATRIX_FILE"
echo "*Generated by Repository Manager Integration Tests*" >> "$MATRIX_FILE"

echo -e "${GREEN}Test matrix generated: $MATRIX_FILE${NC}"
echo ""

# Final summary
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║                    FINAL TEST SUMMARY                        ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Tools Tested:  $TOTAL_TOOLS"
echo -e "  Tools Passing: ${GREEN}$PASSED_TOOLS${NC}"
echo -e "  Tools Failing: ${RED}$FAILED_TOOLS${NC}"
echo ""
echo -e "  Total Tests:   $TOTAL_TESTS"
echo -e "  Tests Passed:  ${GREEN}$TOTAL_PASSED${NC}"
echo -e "  Tests Failed:  ${RED}$TOTAL_FAILED${NC}"
echo ""

if [ $FAILED_TOOLS -gt 0 ]; then
    echo -e "${RED}${BOLD}SOME TOOL TESTS FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}${BOLD}ALL TOOL TESTS PASSED${NC}"
    exit 0
fi
