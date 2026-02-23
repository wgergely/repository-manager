---
tags: ["#audit", "#test-results"]
related: ["[[2026-02-21-investigator1-core-crates]]", "[[2026-02-21-investigator2-tools-content]]", "[[2026-02-21-investigator3-cli-mcp-integration]]"]
date: 2026-02-21
---

# Test Runner 1: Rust Workspace Test Audit Results

## Executive Summary

**Overall Status: FAILING** - One test failure detected in the CLI test suite. All codebase compiles successfully. Clippy identifies 14 style warnings across two crates that should be addressed.

**Test Results:**
- **Total Test Count:** 348 tests run
- **Passed:** 347 tests (99.7%)
- **Failed:** 1 test (0.3%)
- **Ignored:** 1 test
- **Build Status:** All crates compile successfully
- **Clippy Warnings:** 14 warnings (10 in repo-tools, 4 in repo-core)

---

## Detailed Test Results by Crate

### 1. integration-tests (Integration Test Suite)

**Status:** PASSED

**Test Results:**
- Passed: 7 tests
- Failed: 0 tests
- Time: 0.84s

**Tests:**
- test_registry_builtin_providers ... ok
- test_tool_integration_names_and_locations ... ok
- test_load_config_and_registry ... ok
- test_config_with_preset_options ... ok
- test_python_provider_check ... ok
- test_full_vertical_slice ... ok
- test_tool_sync_creates_files ... ok

### 2. mission_tests (Mission/Integration Tests)

**Status:** PASSED

**Test Results:**
- Passed: 51 tests
- Failed: 0 tests
- Ignored: 1 test
- Time: 0.59s

**Ignored Test:**
- gap_019_add_tool_triggers_sync (GAP-019: add-tool does not yet trigger automatic sync)

**Test Coverage:**
- M1 Init Tests (5 tests): All passing
  - m1_1_init_standard_mode_creates_config
  - m1_2_init_worktrees_mode_creates_config
  - m1_3_init_with_tools_records_in_config
  - m1_4_init_with_presets_records_in_config
  - (init-related tests)

- M2 Branch Tests (5 tests): All passing
  - m2_1_classic_layout_current_branch
  - m2_2_feature_worktree_path_computation
  - m2_3_branch_name_sanitization
  - m2_4_container_git_database_path
  - m2_5_main_worktree_path

- M3 Sync Tests (6 tests): All passing
  - m3_1_sync_engine_creation
  - m3_2_check_empty_repo_healthy
  - m3_3_sync_creates_ledger
  - m3_4_tool_sync_creates_files
  - m3_5_managed_blocks_contain_rules
  - m3_6_multiple_rules_multiple_blocks
  - m3_7_check_detects_missing

- M4 Tools Tests (4 tests): All passing
  - m4_1_vscode_integration_info
  - m4_2_cursor_integration_info
  - m4_3_claude_integration_info
  - m4_4_vscode_python_path

- M5 Presets Tests (4 tests): All passing
  - m5_1_uv_provider_id
  - m5_2_uv_provider_check_missing
  - m5_3_registry_has_python_provider
  - m5_4_registry_unknown_preset

- M6 Git Ops Tests (3 tests): All passing
  - m6_1_push_command
  - m6_2_pull_command
  - m6_3_merge_command

- Consumer Verification Tests (8 tests): All passing
  - cv_vscode_settings_valid_json
  - cv_cursorrules_valid_markdown
  - cv_claude_md_valid_markdown
  - cv_multiple_rules_separate_blocks
  - cv_user_content_preserved_outside_blocks
  - cv_block_marker_format_consistent
  - cv_removed_rule_preserves_user_content
  - cv_user_content_between_blocks_preserved
  - cv_concurrent_edit_preservation_across_syncs

- Robustness Tests (4 tests): All passing
  - unicode_branch_name
  - empty_rules_sync
  - long_rule_content
  - special_chars_in_rule_id

- Sync Integration Tests (12 tests): All passing
  - gap_004_sync_applies_projections
  - gap_005_fix_is_stub
  - gap_012_node_provider
  - gap_013_rust_provider
  - gap_018_mcp_server
  - gap_019_add_tool_triggers_sync (IGNORED)
  - test_python_venv_provider
  - test_gemini_tool_integration
  - test_antigravity_tool_integration
  - test_windsurf_tool_integration
  - sync_creates_config_for_single_tool
  - sync_with_rules_includes_rule_content

### 3. repo-blocks (Block Management Library)

**Status:** PASSED

**Test Results:**
- Passed: 90 tests
- Failed: 0 tests
- Time: 0.05s

**Format Tests:**
- JSON format (12 tests): All passing
- TOML format (10 tests): All passing
- YAML format (10 tests): All passing
- Parser tests (18 tests): All passing
- Writer tests (30 tests): All passing
- Additional format and utility tests (10 tests): All passing

### 4. repo-cli (CLI Binary)

**Status:** FAILED

**Test Results:**
- Passed: 189 tests
- Failed: 1 test
- Time: 2.75s

**Failed Test:**

```
test commands::branch::tests::test_list_branches ... FAILED

---- commands::branch::tests::test_list_branches stdout ----

thread 'commands::branch::tests::test_list_branches' (50459) panicked at crates/repo-cli/src/commands/branch.rs:403:9:
assertion failed: result.is_ok()
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf/library/std/src/panicking.rs:689:5
   1: core::panicking::panic_fmt
             at /rustc/01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf/library/core/src/panicking.rs:80:14
   2: core::panicking::panic
             at /rustc/01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf/library/core/src/panicking.rs:150:5
   3: repo::commands::branch::tests::test_list_branches
             at ./src/commands/branch.rs:403:9
   4: repo::commands::branch::tests::test_list_branches::{{closure}}
             at ./src/commands/branch.rs:389:28
   5: core::ops::function::FnOnce::call_once
             at /rustc/01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf/library/core/src/ops/function.rs:250:5
   6: core::ops::function::FnOnce::call_once
             at /rustc/01f6ddf7588f42ae2d7eb0a2f21d44e8e96674cf/library/core/src/ops/function.rs:250:5
```

**Error Summary:** The test assertion `result.is_ok()` failed at line 403 in `crates/repo-cli/src/commands/branch.rs`. The test expects an operation to succeed but received an error result instead. The exact error being returned is not visible in the panic, indicating we need to examine the test code to understand what operation is failing.

**Passing Test Categories:**
- CLI Argument Parsing (55+ tests): All passing
- CLI Commands - Init (8 tests): All passing
- CLI Commands - Branch (3 tests): 2 passing, 1 failing
- CLI Commands - Config (5 tests): All passing
- CLI Commands - Extension (8 tests): All passing
- CLI Commands - Governance/Rules (7 tests): All passing
- CLI Commands - Diff (2 tests): All passing
- CLI Commands - Hooks (4 tests): All passing
- CLI Commands - List (3 tests): All passing
- CLI Commands - Open (7 tests): All passing
- CLI Commands - Rule Management (10 tests): All passing
- CLI Commands - Status (2 tests): All passing
- CLI Commands - Sync (10 tests): All passing
- CLI Commands - Tool Management (13 tests): All passing
- Context Detection (8 tests): All passing
- Interactive Mode (3 tests): All passing
- Integration Tests (20+ tests): All passing

---

## Clippy Lint Analysis

### Overall Warning Count: 14 warnings across 2 crates

### repo-tools (10 warnings)

**File:** crates/repo-tools/src/mcp_installer.rs
- **Line 277** - `unnecessary_map_or`: `map_or(false, |f| obj.contains_key(f))` should use `is_some_and` instead
  ```rust
  self.spec.field_mappings.sse_url_field.map_or(false, |f| obj.contains_key(f));
  // Should be:
  self.spec.field_mappings.sse_url_field.is_some_and(|f| obj.contains_key(f));
  ```

**File:** crates/repo-tools/src/mcp_translate.rs
- **Line 26** - `collapsible_if`: Nested if statements should use `&&` with let binding
  - Line 40: Similar collapsible if pattern
  - Line 51: Similar collapsible if pattern
- **Line 65** - `collapsible_if`: `if let Some(env)` with inner `if !env.is_empty()` should be combined
- **Line 109** - `unnecessary_map_or`: `.map_or(false, |sv| t == sv)` should use direct comparison
  ```rust
  fm.type_values.sse.map_or(false, |sv| t == sv)
  // Should be:
  fm.type_values.sse == Some(t)
  ```
- **Line 178** - `unnecessary_map_or`: Should use direct comparison instead
  ```rust
  fm.type_values.stdio.map_or(false, |v| v == type_str)
  // Should be:
  fm.type_values.stdio == Some(type_str)
  ```
- **Line 194** - `unnecessary_map_or`: Similar issue with http type values
- **Line 201** - `unnecessary_map_or`: Similar issue with sse type values

**File:** crates/repo-tools/src/translator/capability.rs
- **Line 47** - `collapsible_if`: `if tool.capabilities.supports_mcp` with inner `if let Some(servers)` should be combined

### repo-core (4 warnings)

**File:** crates/repo-core/src/hooks.rs
- **Line 198** - `collapsible_if`: Multiple nested if statements should be collapsed using `&&` with let binding
  ```rust
  if let Some(ref custom_dir) = hook.working_dir {
      if let (Ok(canon_custom), Ok(canon_default)) = ...
  ```
  Should combine with `&&` operator

- **Line 199** - `collapsible_if`: Inner if statements for tuple pattern matching should be collapsed

**File:** crates/repo-core/src/rules/registry.rs
- **Line 79** - `manual_inspect`: Using `map_err` where `inspect_err` is more appropriate
  ```rust
  std::fs::rename(&temp_path, &self.path).map_err(|e| {
  // Should be:
  std::fs::rename(&temp_path, &self.path).inspect_err(|e| {
  ```

**File:** crates/repo-core/src/sync/engine.rs
- **Line 536** - `for_kv_map`: Iterating over map with `(key, _value)` should use `.keys()`
  ```rust
  for (ext_name, _ext_config) in &manifest.extensions {
  // Should be:
  for ext_name in manifest.extensions.keys() {
  ```

---

## Compilation Check

**Status:** PASSED

All workspace crates compile successfully without warnings or errors in the check profile.

**Compiled Crates:**
- repo-fs v0.1.0
- repo-meta v0.1.0
- repo-blocks v0.1.0
- repo-git v0.1.0
- repo-content v0.1.0
- repo-extensions v0.1.0
- repo-tools v0.1.0
- repo-presets v0.1.0
- repo-core v0.1.0
- repo-mcp v0.1.0
- repo-cli v0.1.0
- integration-tests v0.1.0

**Compilation Time:** 6.44s

---

## Summary and Recommendations

### Critical Issues
1. **test_list_branches FAILURE** - One test in repo-cli is failing. This requires investigation into the branch listing functionality. The test fails on line 403 with `assertion failed: result.is_ok()`, indicating an unexpected error condition when listing branches.

### Medium Priority Issues
2. **14 Clippy Warnings** - While not breaking functionality, these style warnings should be addressed:
   - 7 warnings about unnecessarily complex `if` statements (collapsible_if)
   - 4 warnings about unnecessary `map_or` patterns that can be simplified
   - 1 warning about using `map_err` instead of `inspect_err`
   - 1 warning about suboptimal map iteration
   - 1 warning about simplifying Option checks

### Pass Rate
- **Functionality Tests:** 347/348 passing (99.7%)
- **Build Status:** 100% successful
- **Code Quality (Clippy):** 14 style warnings to address

### Next Steps
1. Investigate the `test_list_branches` failure in `crates/repo-cli/src/commands/branch.rs:403`
2. Address the clippy warnings in `repo-tools` (10 warnings) and `repo-core` (4 warnings)
3. Verify that the test suite passes after fixes

---

**Audit Generated:** 2026-02-21
**Test Runner:** TestRunner1
**Total Workspace Tests:** 348
**Final Status:** FAILING (due to 1 critical test failure)
