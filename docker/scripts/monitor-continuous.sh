#!/bin/bash
# Continuous Integration Monitor
# Tracks configuration drift and version changes over time
# Designed to run on schedule (cron/GitHub Actions)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
MONITOR_DIR="$PROJECT_ROOT/.monitoring"
HISTORY_FILE="$MONITOR_DIR/drift-history.jsonl"
ALERT_FILE="$MONITOR_DIR/alerts.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

mkdir -p "$MONITOR_DIR"

echo "=============================================="
echo "   CONTINUOUS INTEGRATION MONITOR"
echo "=============================================="
echo ""
echo "Timestamp: $(date -Iseconds)"
echo "Project: $PROJECT_ROOT"
echo ""

# ============================================
# MONITORING FUNCTIONS
# ============================================

record_event() {
    local event_type="$1"
    local message="$2"
    local severity="${3:-info}"
    local timestamp=$(date -Iseconds)

    local json="{\"timestamp\":\"$timestamp\",\"type\":\"$event_type\",\"severity\":\"$severity\",\"message\":\"$message\"}"
    echo "$json" >> "$HISTORY_FILE"

    if [ "$severity" = "warning" ] || [ "$severity" = "error" ]; then
        echo "[$timestamp] [$severity] $event_type: $message" >> "$ALERT_FILE"
    fi
}

check_dockerfile_changes() {
    echo -e "${BLUE}=== Checking Dockerfile Changes ===${NC}"

    local dockerfile_hash_file="$MONITOR_DIR/dockerfile-hashes.txt"
    local changes_detected=false

    # Calculate current hashes
    local current_hashes=""
    for df in "$PROJECT_ROOT"/docker/*/Dockerfile* "$PROJECT_ROOT"/docker/*/*/Dockerfile*; do
        if [ -f "$df" ]; then
            local rel_path="${df#$PROJECT_ROOT/}"
            local hash=$(sha256sum "$df" | cut -d' ' -f1)
            current_hashes+="$hash $rel_path\n"
        fi
    done

    if [ -f "$dockerfile_hash_file" ]; then
        local old_hashes=$(cat "$dockerfile_hash_file")
        if [ "$current_hashes" != "$old_hashes" ]; then
            changes_detected=true
            echo -e "  ${YELLOW}⚠ Dockerfile changes detected${NC}"
            record_event "dockerfile_change" "One or more Dockerfiles modified" "warning"

            # Show which files changed
            echo "$current_hashes" | while read -r line; do
                local hash=$(echo "$line" | cut -d' ' -f1)
                local file=$(echo "$line" | cut -d' ' -f2-)
                if ! echo "$old_hashes" | grep -q "$hash"; then
                    echo "    Changed: $file"
                fi
            done
        else
            echo -e "  ${GREEN}✓ No Dockerfile changes${NC}"
        fi
    else
        echo -e "  ${BLUE}ℹ First run - establishing baseline${NC}"
        record_event "baseline" "Dockerfile hash baseline established" "info"
    fi

    # Save current hashes
    echo -e "$current_hashes" > "$dockerfile_hash_file"
}

check_tool_versions() {
    echo ""
    echo -e "${BLUE}=== Checking Tool Version Manifest ===${NC}"

    local version_file="$MONITOR_DIR/tool-versions.json"

    # Check if we can get versions (requires Docker)
    if docker info >/dev/null 2>&1; then
        echo "  Docker available - checking tool versions..."

        local versions="{\"timestamp\":\"$(date -Iseconds)\",\"tools\":{"

        # Check each tool if image exists
        for tool in claude aider gemini cursor; do
            if docker image inspect "repo-test/$tool:latest" >/dev/null 2>&1; then
                local version_info="unknown"
                case $tool in
                    claude)
                        version_info=$(docker run --rm repo-test/claude:latest --version 2>/dev/null | head -1 || echo "error")
                        ;;
                    aider)
                        version_info=$(docker run --rm repo-test/aider:latest --version 2>/dev/null | head -1 || echo "error")
                        ;;
                    *)
                        version_info="check_required"
                        ;;
                esac
                versions+="\"$tool\":\"$version_info\","
            fi
        done

        versions="${versions%,}}}"
        echo "$versions" > "$version_file"
        echo -e "  ${GREEN}✓ Version manifest updated${NC}"
    else
        echo -e "  ${YELLOW}⏭ Skipped (Docker not available)${NC}"
    fi
}

check_config_drift() {
    echo ""
    echo -e "${BLUE}=== Running Drift Detection ===${NC}"

    local drift_result_file="$MONITOR_DIR/last-drift-check.json"

    # Run drift detection test
    if "$SCRIPT_DIR/test-drift-detection.sh" > "$MONITOR_DIR/drift-output.log" 2>&1; then
        local drift_count=$(grep -c "DRIFT DETECTED" "$MONITOR_DIR/drift-output.log" || echo "0")
        local pass_count=$(grep -c "✓" "$MONITOR_DIR/drift-output.log" || echo "0")

        echo -e "  Passed: ${GREEN}$pass_count${NC}"
        echo -e "  Drift: ${YELLOW}$drift_count${NC}"

        cat > "$drift_result_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "passed": $pass_count,
    "drift_detected": $drift_count,
    "status": "completed"
}
EOF

        if [ "$drift_count" -gt 0 ]; then
            record_event "drift_detected" "$drift_count drift scenarios found" "warning"
        else
            record_event "drift_check" "No new drift detected" "info"
        fi
    else
        echo -e "  ${RED}✗ Drift detection failed${NC}"
        record_event "drift_check_failed" "Drift detection test suite failed" "error"
    fi
}

check_fixture_integrity() {
    echo ""
    echo -e "${BLUE}=== Checking Test Fixture Integrity ===${NC}"

    local fixtures_dir="$PROJECT_ROOT/test-fixtures"
    local fixture_hash_file="$MONITOR_DIR/fixture-hashes.txt"

    if [ -d "$fixtures_dir" ]; then
        local current_hash=$(find "$fixtures_dir" -type f -exec sha256sum {} \; | sort | sha256sum | cut -d' ' -f1)

        if [ -f "$fixture_hash_file" ]; then
            local old_hash=$(cat "$fixture_hash_file")
            if [ "$current_hash" != "$old_hash" ]; then
                echo -e "  ${YELLOW}⚠ Test fixtures modified${NC}"
                record_event "fixture_change" "Test fixtures have been modified" "warning"
            else
                echo -e "  ${GREEN}✓ Fixtures unchanged${NC}"
            fi
        else
            echo -e "  ${BLUE}ℹ Baseline established${NC}"
        fi

        echo "$current_hash" > "$fixture_hash_file"
    fi
}

generate_report() {
    echo ""
    echo -e "${BLUE}=== Generating Monitor Report ===${NC}"

    local report_file="$MONITOR_DIR/report-$(date +%Y%m%d-%H%M%S).md"

    cat > "$report_file" << EOF
# Continuous Integration Monitor Report

**Generated:** $(date -Iseconds)

## Summary

### Recent Events
\`\`\`
$(tail -20 "$HISTORY_FILE" 2>/dev/null || echo "No history yet")
\`\`\`

### Alerts
\`\`\`
$(tail -10 "$ALERT_FILE" 2>/dev/null || echo "No alerts")
\`\`\`

## Recommendations

1. Review any drift warnings and decide if sync is needed
2. Check Dockerfile changes for security implications
3. Verify tool versions are compatible
4. Update baselines after intentional changes

## Next Scheduled Check

Configure via cron or GitHub Actions schedule.
Example: \`0 */6 * * * /path/to/monitor-continuous.sh\`
EOF

    echo "  Report: $report_file"
}

# ============================================
# MAIN EXECUTION
# ============================================

echo -e "${BLUE}Starting continuous monitoring...${NC}"
echo ""

check_dockerfile_changes
check_config_drift
check_fixture_integrity
check_tool_versions
generate_report

echo ""
echo "=============================================="
echo "   MONITORING COMPLETE"
echo "=============================================="
echo ""

# Check for any warnings/errors
warning_count=$(grep -c '"severity":"warning"' "$HISTORY_FILE" 2>/dev/null | tail -1 || echo "0")
error_count=$(grep -c '"severity":"error"' "$HISTORY_FILE" 2>/dev/null | tail -1 || echo "0")

echo "History entries: $(wc -l < "$HISTORY_FILE" 2>/dev/null || echo "0")"
echo "Total warnings: $warning_count"
echo "Total errors: $error_count"
echo ""

if [ "$error_count" -gt 0 ]; then
    echo -e "${RED}Errors detected - review alerts${NC}"
    exit 1
elif [ "$warning_count" -gt 0 ]; then
    echo -e "${YELLOW}Warnings present - review recommended${NC}"
    exit 0
else
    echo -e "${GREEN}All checks passed${NC}"
    exit 0
fi
