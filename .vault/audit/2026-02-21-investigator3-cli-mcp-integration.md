---
tags: ["#audit", "#cli-integration"]
related: ["[[2026-02-21-investigator1-core-crates]]", "[[2026-02-21-investigator2-tools-content]]"]
date: 2026-02-21
---

# Investigator3 Report: CLI, MCP, Integration & Infrastructure

## repo-cli

### Code Quality Findings

**Overall Assessment: Good quality with some notable gaps.**

The CLI (`crates/repo-cli/`) is well-structured, split into focused command modules under `src/commands/`. Error handling uses a proper `CliError` enum with `thiserror`, user-facing messages are consistently colored using `colored`, and the overall command dispatch is clean. However, several specific concerns are worth documenting.

#### Error Handling

The `CliError` type is properly defined in `src/error.rs`:
```rust
pub enum CliError {
    Core(#[from] repo_core::Error),
    Fs(#[from] repo_fs::Error),
    Git(#[from] repo_git::Error),
    Io(#[from] std::io::Error),
    Dialoguer(#[from] dialoguer::Error),
    Json(#[from] serde_json::Error),
    Presets(#[from] repo_presets::Error),
    User { message: String },
}
```

The `main.rs` top-level handler correctly catches errors from `run()` and prints them to stderr before exiting with code 1. There are no bare `unwrap()` calls in production code paths. All `unwrap()` calls observed were inside `#[cfg(test)]` blocks or inline unit test helpers — this is acceptable.

One issue: `src/commands/rule.rs` line 106 uses `unwrap()` in a non-test context:
```rust
let id = path.file_stem().unwrap().to_string_lossy();
```
This is in `run_list_rules()`, inside a `read_dir()` loop. The `file_stem()` can only return `None` if the path ends with `..`, which cannot happen for a path produced by `read_dir()` iteration. However, the code is a silent logic trap — a future refactor that changes how `path` is produced could silently introduce a panic. The path value here is always a real file path from directory iteration, so the unwrap is safe in practice, but it should be replaced with a proper `unwrap_or_default()` or a skip pattern.

#### Command Consistency

Commands are well-organized and follow a consistent pattern: validate input, check preconditions, mutate state, report outcome. The `add-tool`, `remove-tool`, `add-preset`, `remove-preset`, `add-rule`, `remove-rule`, and `list-rules` commands all follow this pattern faithfully.

One consistency gap: `add-tool` triggers a `trigger_sync_and_report()` after the config change, but `add-preset` and `remove-preset` do **not**. This means adding a preset does not immediately apply it to tool configs. The behavior is asymmetric. The `trigger_sync_and_report()` call is present in `run_add_tool()` and `run_remove_tool()` but absent from `run_add_preset()` and `run_remove_preset()`. Users who expect presets to take effect immediately after `add-preset` will be silently confused.

#### Extension Commands Are Entirely Stubs

The `src/commands/extension.rs` module contains 5 commands (`install`, `add`, `init`, `remove`, `list`) that all print "stub - not yet implemented" and return `Ok(())`. The CLI reports success for operations that do nothing. This is misleading:

```rust
// TODO: Implement actual install logic (clone, validate manifest, activate)
println!("{} Extension install from {} (stub - not yet implemented)", ...);
```

The tests for these stubs simply assert `result.is_ok()` — they test that the stub does not crash, not that it does anything useful. This is dead code advertised as a feature.

#### TOML Injection Defense

`src/commands/init.rs` includes a `escape_toml_value()` function that sanitizes user-supplied strings before interpolating them into generated TOML. This is present in both the CLI (`generate_config()`) and the MCP handler (`handle_repo_init()`). The escaping is correct for quotes, backslashes, newlines, and control characters. This is a positive finding.

#### Mode String Aliasing

`init_repository()` accepts both `"worktree"` and `"worktrees"` as synonyms for worktree mode:
```rust
let is_worktree_mode = mode == "worktree" || mode == "worktrees";
```

However, when the config is written, the literal string provided by the user is written verbatim. If you pass `"worktrees"`, the config says `mode = "worktrees"`. If you pass `"worktree"`, it says `mode = "worktree"`. The sync engine and mode parser must then handle both spellings, which creates fragility. A normalizing step to always canonicalize to one form is absent.

#### User Messages

User-facing output is consistently colored and meaningful. Commands print a clear action prefix, the relevant name, and a success or warning indicator. Error messages consistently say "Run 'repo init' first" when a config is missing, which is actionable guidance.

One oddity: `run_check()` with status `Missing` says "Run `repo fix` to repair." but `run_fix()` with a dry-run result of no actions says "No actions needed." — a healthy check followed by a fix of a missing-file state does a redundant health check internally and then calls `fix_with_options`. The double check (check then fix) inside `run_fix` is slightly wasteful but not incorrect.

### Test Quality Findings

**Overall: Good, thorough unit + integration test coverage for the CLI layer.**

The CLI has three tiers of tests:
1. Inline `#[cfg(test)]` modules within each command file (unit tests on the functions directly)
2. `crates/repo-cli/tests/integration_tests.rs` (binary tests via `assert_cmd`)

The unit tests are comprehensive and cover: happy paths, duplicate-prevention, error paths, dry-run behavior, and idempotency for tool and preset management. The integration tests exercise the compiled binary via `assert_cmd` and are genuinely end-to-end within the Rust test harness.

A concrete negative finding: the `test_branch_list_on_fresh_repo` test at line 680 of `integration_tests.rs` does this:

```rust
let _ = cmd
    .current_dir(dir.path())
    .args(["branch", "list"])
    .assert();
```

The assertion result is discarded entirely. This test does not verify any behavior; it merely checks the command does not crash during the binary invocation. It adds no regression value for branch listing.

Another concern: the `test_e2e_backup_on_tool_removal` test explicitly documents its own gap:
```rust
// Note: backup is only created if ToolSyncer.remove_tool is called
// CLI's remove-tool updates config but doesn't call ToolSyncer backup
// This documents the gap for future implementation
```
This is an honest comment but means the feature the test is named after is not tested — the test name is misleading.

The `test_sync_with_cursor_tool_creates_config_with_content` test is particularly strong: it verifies that after `init + sync`, the ledger contains a properly structured cursor intent with a `projections` array, and each projection has `tool` and `file` fields. This validates real serialized output structure, not just file existence.

The `test_sync_json_output_contains_structured_data` and `test_sync_dry_run_json_does_not_modify_filesystem` tests validate the JSON output contract of the sync command, which is critical for CI/CD integration. These are high-quality tests.

Missing coverage areas:
- No tests for the `governance` commands (`rules-lint`, `rules-diff`, `rules-export`, `rules-import`) at the binary level. The governance.rs file has its own unit tests, but no `assert_cmd`-level binary tests for these subcommands.
- No tests verifying that `add-tool` produces specific generated config file content (only ledger structure is verified, not e.g. `.cursorrules` content after sync).
- No tests for the `extension` commands at the binary level beyond confirming they do not crash.

---

## repo-mcp

### Code Quality Findings

**Overall: Well-architected for what is implemented, with clear scope limitations.**

The MCP server (`crates/repo-mcp/`) follows a clean layered architecture:
- `server.rs`: JSON-RPC parsing and dispatch
- `handlers.rs`: Tool call implementations
- `tools.rs`: Tool schema definitions
- `resources.rs` / `resource_handlers.rs`: Resource definitions and readers
- `protocol.rs`: Protocol type definitions
- `error.rs`: Error types

The protocol layer correctly handles JSON-RPC 2.0 semantics: ID preservation, error vs. result separation, notification handling (returns empty string), and the MCP `is_error` field for tool-level errors vs. protocol-level errors.

#### Server Initialization Warning

`server.rs` lines 76-79 log two explicit warnings at startup:
```rust
tracing::warn!("Repository configuration loading not yet implemented");
tracing::warn!("Repository structure validation not yet implemented");
```

This means the MCP server starts without verifying the repository is in a valid state. An AI agent connecting to the MCP server on an uninitialized directory will get no early warning — subsequent tool calls will fail with individual errors, but there is no upfront `NotInitialized` gate. The `initialized` flag on the server struct tracks whether `initialize()` was called, but there is no check on `is_initialized()` before handling requests.

#### Extension Handlers Are Stubs Returning `success: true`

Five extension-management tools (`extension_install`, `extension_add`, `extension_init`, `extension_remove`, `extension_list`) are fully stubbed:
```rust
Ok(json!({
    "success": true,
    "source": args.source,
    "message": format!("Extension install from '{}' (stub - not yet implemented)", args.source),
}))
```

An AI agent calling `extension_install` will receive `"success": true` even though nothing was installed. This is a semantic lie that will cause agents to believe operations succeeded when they did not.

#### Git Primitives Return `NotImplemented` Error

Three tools (`git_push`, `git_pull`, `git_merge`) are explicitly declared with `Err(Error::NotImplemented(...))`. The `handle_tool_call` dispatcher propagates these as tool errors with `is_error: true` in the MCP result. This is the correct approach — the tools are advertised via `tools/list` with their schemas, but invocations produce a clear error. The `test_tool_call_not_implemented_returns_is_error` test in the protocol compliance suite verifies this behavior.

#### Security: Branch Name Validation

`handlers.rs` contains a thorough `validate_branch_name()` function that rejects:
- Empty strings
- Names starting with `-` (git flag injection)
- Null bytes
- `..` sequences (path traversal)
- Names longer than 255 characters
- Invalid git ref characters (`space`, `~`, `^`, `:`, `?`, `*`, `[`, `\`)
- Names ending with `/`, `.`, or `.lock`

Similarly, `rule_add` and `rule_remove` validate rule IDs to alphanumeric + hyphens + underscores only. These are positive security findings. The MCP server is exposed to AI-generated input, so input validation is critical and has been implemented carefully.

#### Rule ID Validation Duplication

The same rule ID validation regex pattern appears identically in three places:
- `crates/repo-cli/src/commands/rule.rs` (`validate_rule_id()`)
- `crates/repo-mcp/src/handlers.rs` (`handle_rule_add()`, `handle_rule_remove()`)
- `crates/repo-cli/src/commands/governance.rs` (`run_rules_import()`)

This is a DRY violation. If the validation logic needs to change (e.g., to allow dots in IDs), it must be updated in multiple places. A shared `repo_core` validation utility should be extracted.

#### detect_mode() Duplication

`detect_mode()` is implemented separately in `crates/repo-cli/src/commands/sync.rs` and `crates/repo-mcp/src/handlers.rs`. The two implementations use different heuristics:
- CLI version uses `ConfigResolver` from `repo_core`
- MCP version checks for `.gt`, `.git`, parent `.gt`, then falls back to config parsing

This behavioral divergence means the MCP and CLI may disagree on mode detection for edge-case directory layouts.

### Test Quality Findings

**Overall: Excellent protocol compliance tests; functional tests are narrowly scoped.**

The `crates/repo-mcp/tests/protocol_compliance_tests.rs` test file contains 20 tests covering:
- JSON-RPC ID preservation (numeric, string, large numeric)
- Error code correctness (-32601 for method not found, -32602 for invalid params)
- Protocol version negotiation
- Notification handling (must return empty string)
- Response structure (result vs. error mutual exclusivity)
- Tools list content validation
- Resources list content validation
- End-to-end tool invocations (repo_init, rule_add + rules resource read)
- Sequential request isolation

These tests are well-written and test real protocol semantics, not just "does it return 200."

The `test_tool_call_rule_add_then_read_rules_resource` test is a genuine multi-step integration test that:
1. Calls `rule_add` tool
2. Reads `repo://rules` resource
3. Verifies the added rule appears in the resource output

This catches regressions in the tool-to-resource consistency path.

Missing test coverage:
- `repo_sync` tool is not tested end-to-end
- `repo_fix` tool is not tested
- `branch_create` and `branch_delete` are not tested
- `tool_add`, `tool_remove` tools are not tested
- `preset_add`, `preset_remove`, `preset_list` are not tested
- Error cases for branch name validation (e.g., names starting with `-`, containing `..`) have no test
- The `server.rs` module has additional unit tests (lines 300-540) testing initialization, ID round-trips, tools/list, resources/list at the server level, providing some coverage for the above gaps

The `crates/repo-mcp/src/server.rs` has inline tests that use `TempDir` and build actual server instances. These complement the protocol compliance tests but are in the same binary rather than the dedicated test file.

---

## Integration Tests

### Coverage Assessment

The workspace-level integration tests live in two files:
- `tests/integration/src/integration_test.rs` (6 tests: vertical slice, config loading, tool sync, registry)
- `tests/integration/src/mission_tests.rs` (structured as modules m1_init, m2_branch, m3_sync, etc.)

**integration_test.rs** is substantive: `test_full_vertical_slice` exercises the complete chain from config parsing through tool sync to file verification, including checking that managed blocks (`<!-- repo:block:X -->` ... `<!-- /repo:block:X -->`) appear correctly in cursor and Claude output files. `test_tool_sync_creates_files` exercises the API layer directly. These are genuine integration tests.

**mission_tests.rs** provides a `TestRepo` struct with helper methods and organizes tests by mission category. However, the mission test suite exercises library APIs directly, not the CLI binary. The tests are validating that library units compose correctly, not that the CLI composes them correctly.

### Test Quality Issues

**Issue 1: Workspace-level tests do not invoke the CLI binary.**

All `tests/integration/` tests import library crates directly (`use repo_core::...`, `use repo_tools::...`, etc.) and call Rust functions. They do not invoke the `repo` binary. The binary integration tests are in `crates/repo-cli/tests/integration_tests.rs`, which does use `assert_cmd`. The workspace-level tests are effectively API-layer tests, not CLI integration tests.

**Issue 2: mission_tests.rs M3 (sync) is sparse.**

The mission sync tests only verify `SyncEngine` can be constructed and that basic sync/check operations return without error. They do not verify the actual config files generated on disk. The comment at M3.1 says:
```
/// M3.1: SyncEngine can be created
```
This is a smoke test, not a functional validation of sync output.

**Issue 3: No tests for the governance commands at the integration level.**

`rules-lint`, `rules-diff`, `rules-export`, `rules-import` have no workspace-level integration tests.

**Issue 4: No mock usage; no stubs.**

This is a positive finding. The integration tests use real filesystem operations (`tempfile::TempDir`), real library calls, and do not stub out any dependencies. They are genuine integration tests within their scope.

**Issue 5: `test_python_provider_check` has environment-dependent behavior.**

```rust
// Should be Missing or Broken (depending on whether uv is installed)
// Either way, it won't be Healthy since no venv exists
assert!(report.status != PresetStatus::Healthy);
```

The test is asserting a negative (not Healthy) rather than a specific expected state. While this works in CI where uv is unlikely to be installed, it is fragile if the test environment has uv installed and a venv somehow exists. The comment acknowledges the environment dependency, but this is still a test quality issue.

**Issue 6: Generated tool config content verification is limited.**

`test_full_vertical_slice` does verify block markers in `.cursorrules` and `CLAUDE.md`. However, it does not test `.vscode/settings.json` content (only that it exists), and does not test any of the newer tool integrations (windsurf, aider, gemini, etc.).

---

## Test Fixtures

### Assessment

The `test-fixtures/` directory contains:
```
test-fixtures/
  expected/
    aider/.aider.conf.yml
    claude/CLAUDE.md
    cursor/.cursorrules
  repos/
    config-test/
      .repository/config.toml
      .repository/rules/coding-standards.md
      Cargo.toml
      src/main.rs
    simple-project/
      .aider.conf.yml
      CLAUDE.md
      Cargo.toml
      GEMINI.md
      src/main.rs
```

**Fixtures are minimal and only cover 3 of the 13+ supported tools.**

The `config-test` fixture's `config.toml` lists only `["cursor", "claude"]`, omitting aider despite aider being one of three expected output files. This is inconsistent — the fixture used for config generation testing references aider expectations but doesn't declare aider as a tool:

```toml
tools = ["cursor", "claude"]
```

Yet `test-config-generation.sh` tests:
```bash
test_config_file "aider" ".aider.conf.yml" ...
```

If aider requires being in the `tools` list to have config generated, this test will always fail for aider (unless aider config is generated unconditionally). If it passes, then tool selection is not properly enforced.

**Expected output files are identical to the rules file content.**

`expected/claude/CLAUDE.md`, `expected/cursor/.cursorrules`, and `expected/aider/.aider.conf.yml` contain content derived directly from `coding-standards.md`. This is appropriate for snapshot testing, but the expected outputs have not been auto-generated from the code — they are handwritten and could drift from what the system actually generates. There is no mechanism to regenerate them.

**The `simple-project` fixture is pre-configured with outputs but has no `.repository/` directory.**

`test-fixtures/repos/simple-project/` contains `CLAUDE.md`, `.aider.conf.yml`, and `GEMINI.md` but no `.repository/config.toml`. It represents a snapshot of what a configured project looks like, not a fixture for initializing and syncing. Its purpose is not documented anywhere visible.

**No README or documentation for test fixtures.**

There is no `test-fixtures/README.md` explaining the purpose of each fixture, when to update expected files, or how to add new fixtures for new tools.

**Fixtures do not cover realistic multi-rule scenarios.**

A real project might have 5-20 rules with various tags. The sole rule in `config-test` is a single `coding-standards.md` with no tags. Multi-rule conflict resolution, ordering, and tag filtering are not covered by fixtures.

---

## CI/CD & Docker

### Findings

**There is only one workflow file: `.github/workflows/docker-integration.yml`.**

There is no separate workflow for:
- Rust unit tests (`cargo test --workspace`)
- Clippy linting (`cargo clippy -- -D warnings`)
- Format checking (`cargo fmt -- --check`)
- Security auditing (`cargo audit`)

The only CI pipeline is the Docker integration pipeline. This means the CI pipeline as written does NOT run the Rust test suite. A regression in core library code (repo-core, repo-tools, etc.) would not be caught by CI unless it manifests in the Docker integration tests.

This is a critical CI gap. Standard Rust projects should have at minimum:
```yaml
- run: cargo test --workspace
- run: cargo clippy -- -D warnings
- run: cargo fmt -- --check
```

None of these exist in any workflow file.

**The Docker integration workflow uses path filters.**

The workflow only triggers on changes to `docker/`, `crates/`, `test-fixtures/`, or `docker-compose*.yml`. Changes to `tests/integration/` do not trigger the workflow:
```yaml
paths:
  - 'docker/**'
  - 'crates/**'
  - 'test-fixtures/**'
  - 'docker-compose*.yml'
```

Changes to `tests/integration/src/` would not automatically trigger CI.

**Integration test failure is soft-silenced.**

In the integration-tests job:
```bash
./docker/scripts/test-config-generation.sh || echo "Config gen tests completed"
./docker/scripts/test-tool-reads-config.sh || echo "Tool read tests completed"
```

Both test scripts have their failure exit codes suppressed with `|| echo "..."`. A failing config generation test will not fail the CI job. This means the pipeline can pass even when core config generation is broken.

**The docker-compose.ci.yml override does not wire Gemini to the mock API.**

`docker-compose.ci.yml` sets `GOOGLE_API_KEY: "mock-key-for-ci"` for the Gemini container but does not set an override base URL. The mock WireMock server likely does not implement the Gemini API endpoint schema. Gemini tests in CI may silently make real API calls or fail silently without meaningful verification.

**Smoke tests do not have a `needs: build-repo-manager` dependency.**

The `smoke-tests` job depends on `build-tool-images` and `build-vscode-images` but not on `build-repo-manager`. If the repo-manager image fails to build, smoke tests run with a potentially missing or stale image. The `integration-tests` job does depend on `smoke-tests` and `build-repo-manager`, so the sequencing is correct for the full integration test, but the smoke tests could theoretically run against an outdated repo-manager image.

**Dockerfiles for individual tools exist and are logically organized.**

The `docker/cli/` and `docker/vscode/` directories contain per-tool Dockerfiles. The build matrix covers `[claude, aider, gemini, cursor]` for CLI and `[cline, roo]` for VS Code extensions. These match the tools documented in the compose file.

**Mock API stubs are minimal.**

The WireMock stubs cover:
- `POST /v1/messages` (Anthropic)
- `GET /health`
- `POST /openai/completions` (OpenAI)

There are no stubs for:
- Gemini API (`/v1beta/models/...`)
- Cursor API
- Any streaming endpoints

The integration tests that test "mock API connectivity" only send a single POST to `/v1/messages` and check the response. They do not verify that generated config files actually cause tool containers to authenticate and use the mock.

---

## Documentation

### Assessment

The `docs/` directory contains a structured testing framework documentation in `docs/testing/`:

**`docs/testing/README.md`**
Accurately describes the gap tracking system and test organization. The test result summary (`42 total, 32 passed, 10 ignored`) appears to reflect a snapshot from 2026-01-27 that may not match the current state after subsequent gap closures. The "Key Discoveries" section correctly documents known behavioral nuances (e.g., `tools` vs. `active.tools` config key).

**`docs/testing/GAP_TRACKING.md`**
Well-maintained and honest. The gap tracking system shows clear before/after states and marks closed gaps with strikethrough. Remaining open gaps are accurately described. The dashboard shows "Production Readiness: 95%" which reflects genuine progress.

**Areas where documentation is missing:**
- No documentation for the `docker/` directory itself (only `docker/README.md` exists but was not found in scope). The `docker/scripts/` directory has 15+ shell scripts with no index or documentation of which scripts do what and in what order.
- No `test-fixtures/README.md` explaining fixture structure or update procedures.
- No documentation for the MCP server protocol beyond the code's own doc comments.
- `docs/testing/framework.md` shows a coverage matrix with some entries listed as "CRITICAL" gap score (Configuration Sync 20%, Git Operations 0%, MCP Server 0%) that appears stale — the MCP server is now substantially implemented and sync works. The framework document was last updated 2026-01-27 and has not been revised to reflect the post-background-agent implementation state.

**The framework.md coverage matrix is misleading.**

```
| Configuration Sync | 100% | 30% | 20% | CRITICAL |
| Git Operations     | 100% | 0%  | 0%  | CRITICAL |
| MCP Server         | 100% | 0%  | 0%  | CRITICAL |
```

These percentages do not reflect the current codebase. Sync is substantially implemented. Git operations have `LayoutProvider` implementations. The MCP server is a working implementation, not 0% implemented. This stale documentation creates a false picture of the project status.

---

## Summary of Critical Issues

### Priority 1: No Rust CI Workflow

The repository has no GitHub Actions workflow that runs `cargo test`, `cargo clippy`, or `cargo fmt`. The only CI workflow is Docker-based integration testing. A regression in any Rust library crate (wrong return type, removed method, panic in core logic) will not be caught by CI unless it happens to surface in a Docker integration test. This is the most severe infrastructure gap.

**Recommended fix:** Add a `.github/workflows/rust.yml` that runs `cargo test --workspace`, `cargo clippy -- -D warnings`, and `cargo fmt -- --check` on every push and PR.

### Priority 2: Integration Test Failure Suppression

In `docker-integration.yml`, the config generation and tool read test scripts are invoked with `|| echo "..."` which suppresses failure. A broken config generation pipeline will produce a green CI job. This makes the integration tests meaningless as a regression gate.

**Recommended fix:** Remove the `|| echo "..."` silencing and let test failures propagate to the job exit code.

### Priority 3: Extension Commands Are Deceptive Stubs

Both the CLI (`crates/repo-cli/src/commands/extension.rs`) and MCP server (`crates/repo-mcp/src/handlers.rs`, lines 700-810) implement extension management commands that report `success: true` or print "OK" while doing nothing. The CLI tests for these commands only assert `result.is_ok()`. This creates false assurance that the extension system works.

**Recommended fix:** Either implement the functionality or change the stub responses to `success: false` with a clear "not yet implemented" message, so callers know not to depend on them.

### Priority 4: Asymmetric Sync Triggering

`run_add_tool()` calls `trigger_sync_and_report()` after config modification, but `run_add_preset()` does not. Users adding presets will not see their presets applied until they manually run `repo sync`. This behavioral inconsistency is not documented in user-facing output.

### Priority 5: detect_mode() Divergence Between CLI and MCP

The CLI uses `ConfigResolver` (from `repo_core`) to detect mode, while the MCP server uses a custom filesystem heuristic (checking `.gt`, `.git`, parent `.gt`). These two implementations can produce different results for the same directory. Mode detection should be centralized in `repo_core` or `repo_fs`.

### Priority 6: Test Fixture Inconsistency

The `config-test` fixture declares `tools = ["cursor", "claude"]` but the config generation test script tests for aider config output. This mismatch will cause the aider config generation test to fail unless aider output is generated unconditionally (independent of the tools list), in which case the tools list is not being respected by the sync engine.

### Priority 7: Stale Documentation

`docs/testing/framework.md` shows coverage percentages and gap scores that predate the January 2026 implementation sprint. The displayed "CRITICAL" gaps for MCP Server and Git Operations are no longer accurate. Stale documentation misleads new contributors about the actual state of the project.

### Non-Critical Observations

- The `file_stem().unwrap()` in `run_list_rules()` is safe today but should be replaced for future-proofing.
- Rule ID validation logic is duplicated in 3+ locations and should be centralized.
- Mode string aliasing (`"worktree"` vs. `"worktrees"`) is not normalized on write.
- The `test_branch_list_on_fresh_repo` test discards its assertion result and provides zero regression value.
- The `test_e2e_backup_on_tool_removal` test name is misleading — it does not test backup behavior.
- No tests exist for `rules-lint`, `rules-diff`, `rules-export`, `rules-import` at the binary level.
- The `simple-project` fixture has no documentation explaining its purpose.
- The `docs/testing/README.md` test summary is dated 2026-01-27 and has not been updated.
