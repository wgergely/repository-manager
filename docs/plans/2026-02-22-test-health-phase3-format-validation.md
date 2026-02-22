# Phase 3: Format Validation Tests

**Priority**: P3 (test hygiene) — **highest-value phase in this chain**
**Audit ID**: TH-3
**Domain**: `crates/repo-tools/tests/`, new format validation test files
**Status**: Not started
**Prerequisite**: [Phase 2](2026-02-22-test-health-phase2-fixtures.md) (uses shared `TestRepo` fixture)
**Estimated scope**: 5 new test files, ~400 lines new code

---

## Testing Mandate

> **Inherited from [tasks/_index.md](../tasks/_index.md).** Read and internalize the
> full Testing Mandate before writing any code or any test.

**Domain-specific enforcement:**
- Every test in this phase must **fail** when the generated output is malformed. If
  a test passes on garbage output, the test is invalid.
- Tests must validate format structure (parseable, expected keys present) — NOT content
  strings. Content tests already exist and are sufficient for their purpose.
- Do NOT test against live tool installations (Cursor, VSCode, etc) — these tests must
  run in CI without those tools installed. Validate against format specifications.

---

## Problem Statement

No test validates that generated config files actually work with their target tools.
Every existing tool test only checks that written content contains the expected string —
proving `fs::write` works, not that the output is correct.

See: [Test Health Audit TH-3](../audits/2026-02-22-test-health-audit.md#finding-th-3-zero-format-validation-against-tool-schemas)

---

## Implementation Plan

### Step 1: Add format validation for VSCode settings

**File:** `crates/repo-tools/tests/format_vscode_tests.rs` (new)

VSCode settings.json is the highest-risk format — it's JSON, and a single syntax error
makes the entire file unparseable.

```rust
//! Format validation tests for VSCode settings.json output.
//!
//! Category: format-validation
//! These tests verify that generated .vscode/settings.json is valid JSON
//! and contains expected top-level structure.

use repo_test_utils::repo::TestRepo;
use serde_json::Value;

#[test]
fn vscode_settings_is_valid_json() {
    // Generate settings.json via sync
    // Parse it back as serde_json::Value
    // Assert it's a JSON object (not array, not primitive)
}

#[test]
fn vscode_settings_preserves_existing_user_keys() {
    // Write a settings.json with user-defined keys
    // Sync
    // Parse back
    // Assert user keys still present alongside managed keys
}

#[test]
fn vscode_python_path_is_valid_path_format() {
    // When python context is set, verify python.defaultInterpreterPath
    // is a string (not null, not number, not nested object)
}

#[test]
fn vscode_settings_has_no_duplicate_keys() {
    // Parse raw JSON text, check for duplicate keys
    // (serde_json silently deduplicates — need raw parse)
}
```

### Step 2: Add format validation for YAML tools (Aider)

**File:** `crates/repo-tools/tests/format_aider_tests.rs` (new)

Aider's `.aider.conf.yml` is YAML — currently never parsed back in tests.

```rust
//! Format validation tests for Aider .aider.conf.yml output.
//!
//! Category: format-validation

use serde_yaml::Value;

#[test]
fn aider_config_is_valid_yaml() {
    // Generate .aider.conf.yml via sync
    // Parse it back as serde_yaml::Value
    // Assert it's a YAML mapping (not sequence, not scalar)
}

#[test]
fn aider_config_preserves_existing_user_keys() {
    // Write a .aider.conf.yml with user-defined keys
    // Sync with rules
    // Parse back
    // Assert user keys still present
}

#[test]
fn aider_managed_blocks_are_valid_yaml_comments() {
    // Verify block markers use YAML comment syntax (# prefix)
    // not HTML comments (<!-- --> would break YAML)
}
```

### Step 3: Add format validation for Markdown tools (Cursor, Claude, Copilot, Windsurf, Gemini)

**File:** `crates/repo-tools/tests/format_markdown_tests.rs` (new)

All Markdown-based tools share the same format concern: managed block markers must be
valid HTML comments that don't break Markdown rendering.

```rust
//! Format validation tests for Markdown-based tool outputs.
//!
//! Category: format-validation
//! Covers: Cursor (.cursorrules), Claude (CLAUDE.md), Copilot
//! (.github/copilot-instructions.md), Windsurf (.windsurfrules),
//! Gemini (GEMINI.md), Cline (.clinerules), Roo (.roorules)

#[test]
fn managed_block_markers_are_valid_html_comments() {
    // For each Markdown tool:
    // Generate output with rules
    // Verify every <!-- repo:block:X --> has matching <!-- /repo:block:X -->
    // Verify no unclosed comments
}

#[test]
fn managed_block_markers_use_consistent_format() {
    // Verify markers match regex: <!-- repo:block:[\w.-]+ -->
    // Verify closing markers match regex: <!-- /repo:block:[\w.-]+ -->
}

#[test]
fn markdown_output_has_no_nested_html_comments() {
    // HTML comments cannot be nested (<!-- <!-- --> --> is invalid)
    // If rule content contains "<!--", verify it's escaped or excluded
}

#[test]
fn user_content_outside_blocks_is_byte_identical_after_sync() {
    // Write file with user content
    // Sync
    // Extract text outside managed blocks
    // Assert byte-identical to original user content
}
```

### Step 4: Add format validation for Antigravity (directory-based tool)

**File:** `crates/repo-tools/tests/format_antigravity_tests.rs` (new)

Antigravity uses `.agent/rules/<NN>-<id>.md` — directory structure matters.

```rust
//! Format validation tests for Antigravity .agent/rules/ output.
//!
//! Category: format-validation

#[test]
fn antigravity_creates_rules_directory_not_file() {
    // Verify .agent/rules/ is a directory
    // Not a file (which would happen if path handling is wrong)
}

#[test]
fn antigravity_rule_files_follow_naming_convention() {
    // Verify files are named <NN>-<id>.md
    // Verify NN is zero-padded two digits
    // Verify id matches the rule ID
}

#[test]
fn antigravity_rule_files_are_valid_markdown() {
    // Each .md file should be non-empty
    // Should contain the rule content
    // Should NOT contain managed block markers (Antigravity uses
    // one-file-per-rule, not managed blocks)
}
```

### Step 5: Add format validation for JSON tools (JetBrains, Zed)

**File:** `crates/repo-tools/tests/format_json_tests.rs` (new)

```rust
//! Format validation tests for JSON-based tool outputs.
//!
//! Category: format-validation
//! Covers: JetBrains, Zed

#[test]
fn json_tool_output_is_valid_json() {
    // For each JSON tool:
    // Generate output via sync
    // Parse back as serde_json::Value
    // Assert top-level structure matches expected shape
}

#[test]
fn json_tool_output_preserves_existing_structure() {
    // Write a config with existing keys
    // Sync
    // Parse back
    // Assert existing keys preserved
}
```

### Step 6: Add dev-dependencies to `repo-tools`

Update `crates/repo-tools/Cargo.toml`:
```toml
[dev-dependencies]
repo-test-utils = { workspace = true }
serde_yaml = "0.9"  # For YAML parse-back validation
regex = "1"          # For marker format validation
```

---

## Acceptance Criteria

- [ ] `format_vscode_tests.rs` — 4 tests, all pass
- [ ] `format_aider_tests.rs` — 3 tests, all pass
- [ ] `format_markdown_tests.rs` — 4 tests covering all 7 Markdown tools
- [ ] `format_antigravity_tests.rs` — 3 tests, all pass
- [ ] `format_json_tests.rs` — 2 tests covering JetBrains, Zed
- [ ] Every format test **fails** when output is deliberately malformed (verified during development)
- [ ] `cargo test -p repo-tools` passes with new tests included
- [ ] `cargo clippy` clean

---

## Files to Create

| File | Tests | Tools Covered |
|------|-------|--------------|
| `crates/repo-tools/tests/format_vscode_tests.rs` | 4 | VSCode |
| `crates/repo-tools/tests/format_aider_tests.rs` | 3 | Aider |
| `crates/repo-tools/tests/format_markdown_tests.rs` | 4 | Cursor, Claude, Copilot, Windsurf, Gemini, Cline, Roo |
| `crates/repo-tools/tests/format_antigravity_tests.rs` | 3 | Antigravity |
| `crates/repo-tools/tests/format_json_tests.rs` | 2 | JetBrains, Zed |

## Files to Modify

| File | Change |
|------|--------|
| `crates/repo-tools/Cargo.toml` | Add dev-deps: serde_yaml, regex, repo-test-utils |

---

## Dependencies

- **Depends on**: [Phase 2](2026-02-22-test-health-phase2-fixtures.md) (shared `TestRepo` fixture)
- **Blocks**: [Phase 4](2026-02-22-test-health-phase4-golden-files.md), [Phase 5](2026-02-22-test-health-phase5-tautological-tests.md)
- **Cannot parallelize with**: Phase 4 or 5 (they depend on format validators)

---

## Cross-References

- **Source finding**: [Test Health Audit TH-3](../audits/2026-02-22-test-health-audit.md#finding-th-3-zero-format-validation-against-tool-schemas)
- **Tool research**: [Cursor research](../research/), [Claude Code research](../research/)
- **Design specs**: [spec-tools.md](../design/) — tool output format specifications
- **Related task**: [P0 Sync Engine & Tool Paths](../tasks/P0-sync-engine-and-tool-paths.md) — secondary paths
- **ADR**: [ADR-001 Extension System](../adr/02-decisions/ADR-001-extension-system-architecture.md)
- **Chain**: Phase 3 of 5 — prev: [Phase 2](2026-02-22-test-health-phase2-fixtures.md), next: [Phase 4](2026-02-22-test-health-phase4-golden-files.md)

---

*Plan created: 2026-02-22*
