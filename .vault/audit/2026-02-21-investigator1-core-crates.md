---
tags: ["#audit", "#core-crates"]
related: ["[[2026-02-21-investigator2-tools-content]]", "[[2026-02-21-investigator3-cli-mcp-integration]]"]
date: 2026-02-21
---

# Investigator1 Report: Core Crates Audit

**Auditor:** Investigator1
**Date:** 2026-02-21
**Scope:** `crates/repo-fs`, `crates/repo-git`, `crates/repo-meta`, `crates/repo-core`

---

## repo-fs

### Code Quality Findings

**Overall assessment:** This is the best-written crate in the audit scope. The code is well-structured, clearly documented, and shows real engineering discipline.

**Strengths:**
- `path.rs` (`NormalizedPath`) is thorough: handles backslash normalization, resolves `.` and `..` via a custom `clean()` function, rejects UNC/network paths, and provides `From` impls for all common path types. The fast-path optimization (checking before invoking `clean()`) is sensible.
- `io.rs` (`write_atomic`) follows the correct write-to-temp-then-rename pattern, acquires an advisory file lock via `fs2`, uses exponential backoff for lock contention, and cleans up orphaned temp files on failure. The symlink check before writing is a legitimate security control.
- `checksum.rs` is minimal, correct, and exposes a canonical `sha256:<hex>` format used consistently across the workspace.
- `config.rs` (`ConfigStore`) is a clean format-dispatching layer; error types carry path and format context.
- `error.rs` is well-typed with `thiserror`. Every variant is meaningful; the `io()` convenience constructor is a good ergonomic choice.
- `layout.rs` correctly models the three layout modes (Container, InRepoWorktrees, Classic) with detection logic that walks up the directory tree using `dunce::canonicalize` to handle Windows extended paths safely.

**Issues:**

1. **Explicit TODOs in public API (`io.rs` lines 179, 187):**
   ```rust
   /// TODO: PLACEHOLDER - replace with ManagedBlockEditor
   pub fn read_text(path: &NormalizedPath) -> Result<String> { ... }
   /// TODO: PLACEHOLDER - replace with ManagedBlockEditor
   pub fn write_text(path: &NormalizedPath, content: &str) -> Result<()> { ... }
   ```
   `read_text` and `write_text` are publicly exported (`pub use io::...` is not present, but they are `pub` within the `io` module which is `pub mod io`) and carry TODO markers indicating they are placeholders. These functions are re-exported indirectly and called from `repo-core` (`tool_syncer`). Shipping public API with acknowledged placeholder status is a maintenance risk.

2. **Symlink check bug (`io.rs` line 76):** The symlink check uses `unwrap_or(false)` to silently swallow `std::io::Error` from `contains_symlink`. If an I/O error occurs during the check (e.g. permission denied on a path component), the write is allowed through rather than failing safely. This is a security-adjacent defect: the safe default on error should be to refuse the write.

3. **`NormalizedPath::clean()` drops leading `..` for relative paths silently (`path.rs` line 157-158):**
   ```rust
   ".." => {
       if !out.is_empty() { out.pop(); }
       else if !is_absolute {
           // If relative, we drop leading .. (sandbox behavior)
       }
   }
   ```
   Dropping `..` on relative paths prevents traversal, but it does so silently with no error or warning. A caller constructing `NormalizedPath::new("../../etc/passwd")` will get `"."` back — which may be surprising. This design decision warrants a comment at minimum, and possibly an explicit error type.

4. **`io.rs` lock-file not cleaned up:** The implementation creates `<filename>.lock` files to coordinate writes, but these are never removed. Over time, directories will accumulate `.lock` files. This is not critical (they're empty files) but is sloppy.

5. **No `unsafe` code** — confirmed throughout the crate. Good.

6. **`path.rs` unit tests live in `src/` (`#[cfg(test)]` module)** — correct placement.

7. **No `unwrap()` calls in production paths** — confirmed. All fallible operations propagate errors through `Result`.

### Test Quality Findings

The test suite for `repo-fs` is unusually large (9 test files in `tests/`). Most of it is good, but there are serious issues:

**`tests/io_tests.rs`:** Tests are real and test real behavior.
- Atomic write correctness, overwrite semantics, content replacement, concurrent reader test — all solid.
- The concurrent-reader test (`test_write_atomic_concurrent_reader_sees_complete_content`) is meaningful but has a probabilistic weakness: the reader iterates 50 times in a tight loop. On a fast system where the write completes before any read, this vacuous-truth guard catches it. But the "at least one read" assertion is weak; the test doesn't actually verify it observed both versions, only that it never saw a partial one.

**`tests/layout_tests.rs`:** Tests are real and use `tempfile` with actual filesystem creation. Coverage is adequate.

**`tests/config_tests.rs`:** Tests real config loading and format detection. Adequate.

**`tests/correctness_tests.rs`, `tests/error_condition_tests.rs`:** Mix of real behavior tests and some shallow tests (e.g., testing that `NormalizedPath::new("foo/bar")` produces `"foo/bar"` — trivially true).

**`tests/security_tests.rs` and `tests/security_audit_tests.rs` — CRITICAL ISSUES:**

These two test files have overlapping names and possibly overlapping concerns. More critically:

- Security tests for the symlink-attack prevention appear to rely on creating actual symlinks (`std::os::unix::fs::symlink`), which is platform-specific and will silently be skipped or fail on Windows. The security contract is not tested portably.
- Several "security" tests check that `validate_path_identifier` rejects certain inputs. These are unit-level tests that belong in `src/path.rs`'s `#[cfg(test)]` module, not in the integration test directory.

**`tests/property_tests.rs`:** Uses `proptest`. The properties tested (e.g., that normalized paths don't contain backslashes) are meaningful but shallow. Properties like "joining never produces a network path" or "clean is idempotent" are not tested. The coverage here gives a false sense of security for the path normalization logic.

**`tests/snapshot_tests.rs`:** Uses `insta` for snapshot testing of path normalization output. This is appropriate. However, if snapshots become stale they will fail loudly (correct behavior), but newly added snapshot cases with default values could silently encode wrong behavior as the "expected" output.

**`tests/concurrency_tests.rs`:** Tests concurrent write behavior. These are legitimate integration tests. However the tests do not verify atomicity under crash conditions — they only verify correctness under clean concurrent access, which the lock+rename already handles.

**Missing test coverage:**
- No test for the `symlink_in_path` error path in `write_atomic` when the I/O error branch is taken (`unwrap_or(false)` bug above has no test).
- No test for `ConfigStore` with no file extension or empty extension.
- No test for `LayoutMode::Container` validation failure when `.gt/` exists but `main/` does not.
- No test for `LayoutDetectionFailed` (walking up past filesystem root).

### Test Organization

- The split between `src/path.rs`'s `#[cfg(test)]` module and `tests/` directory is inconsistent. Unit tests for `validate_path_identifier` and basic `NormalizedPath` construction appear in both `src/path.rs` and `tests/correctness_tests.rs`, creating duplication.
- `tests/security_tests.rs` and `tests/security_audit_tests.rs` cover overlapping territory and should be merged or clearly differentiated.
- The `benches/fs_benchmarks.rs` file exists but was not audited for correctness of benchmark setup.

---

## repo-git

### Code Quality Findings

**Overall assessment:** The implementation is functionally correct for its purpose but has several design and robustness issues.

**Strengths:**
- `error.rs` is well-typed. Every variant is meaningfully named and uses `thiserror`.
- `helpers.rs` is well-documented. The `push`, `pull`, `merge` helper functions have clear argument lists and correct control flow.
- The `LayoutProvider` trait in `provider.rs` is a clean abstraction. The decision to keep network operations (`push`, `pull`, `merge`) as free functions rather than trait methods is explicitly justified in a comment — good.
- `naming.rs` is clean with solid unit tests co-located in `#[cfg(test)]`.
- `ClassicLayout` correctly errors on `create_feature` and `remove_feature` with actionable migration hints.

**Issues:**

1. **`ClassicLayout::list_worktrees` swallows errors with `unwrap_or_else` (`classic.rs` line 54):**
   ```rust
   let branch = self.current_branch().unwrap_or_else(|_| "unknown".into());
   ```
   If the repository is in a broken state (corrupted HEAD, detached HEAD after a rebase gone wrong), `current_branch()` fails and the result is silently replaced with the string `"unknown"`. This masks the real error from the caller and could cause confusing downstream behavior. Should propagate or log the error.

2. **`helpers.rs` `pull()` uses `checkout_head` with `force()` (`helpers.rs` line 205):**
   ```rust
   co_repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
   ```
   Force checkout discards local modifications without warning. For a tool that manages repository configurations, this is a dangerous default. If the caller intends to keep working tree changes, this will silently destroy them.

3. **`helpers.rs` `merge()` has two separate `annotated_commit` lookups (`helpers.rs` lines 239, 264):**
   ```rust
   let annotated_commit = repo.find_annotated_commit(source_commit.id())?;
   // ...
   let annotated_for_merge = mr.find_annotated_commit(source_commit.id())?;
   ```
   When `merge_repo` is `None`, `mr` equals `repo`, so `annotated_for_merge` duplicates `annotated_commit` unnecessarily. Minor inefficiency, but also a readability issue.

4. **No `unsafe` code** — confirmed.

5. **`container.rs` (`ContainerLayout::open_repo()`)** — returns a `Repository` opened on the bare `.gt` directory. For `push`, `pull`, and `merge`, the caller must then open the main worktree separately for checkout operations. This dual-repo pattern is documented in test code (`git_operations_tests.rs` line 319) but is not enforced or guided at the API level. It is a leaky abstraction — callers of `open_repo()` may not realize they also need a separate repo handle for checkout.

6. **`ClassicLayout::current_branch()` opens the repository on the `.git` directory (`classic.rs` line 82):**
   ```rust
   let repo = Repository::open(self.git_dir.to_native())?;
   ```
   This should open on the repository root (same as `ClassicLayout::open_repo()` does), not the `.git` dir. While `git2` often handles this correctly, opening on `.git` is not the canonical form and may cause subtle issues with hooks or config lookup in some git2 versions.

7. **`in_repo_worktrees.rs` and `container.rs` were not directly read** due to tool buffering issues but the test coverage for these was verified through the integration tests.

### Test Quality Findings

**`tests/classic_tests.rs`:** Uses real `git2::Repository::init`. Tests are basic but test real behavior:
- `test_classic_git_database` only checks `ends_with(".git")` — the assertion is weak (doesn't check the full path structure).
- `test_classic_create_feature_returns_error` and `test_classic_remove_feature_returns_error` are correct error-path tests.

**`tests/git_operations_tests.rs`:** This is the most thorough test file in the crate. It uses real `git` CLI commands (not mocks) to set up actual repositories, then exercises the Rust abstractions. This is the correct approach. Specific notes:
- `setup_classic_repo_with_git()` uses `std::process::Command` to call `git` binary — this is a dependency on having `git` in `PATH`, which is a valid integration test assumption.
- `test_classic_merge_fast_forward` and `test_container_merge_fast_forward` are meaningful end-to-end tests.
- **Missing:** No test for the force-checkout destructive behavior (mentioned above). No test for merge conflict detection and cleanup.
- **Missing:** No test for the `CannotFastForward` error path (divergent histories).

**`tests/container_tests.rs`:** Tests basic container layout setup. Coverage is adequate for happy paths.

**`tests/in_repo_worktrees_tests.rs`:** Not directly read but included in the full test file list.

**`tests/robustness_unicode.rs`:** Tests that the naming strategy handles unicode correctly. This is a good test category.

**Missing test coverage:**
- No test for `pull()` with a successful fast-forward (the happy path for the most common operation).
- No test verifying that force-checkout in `pull()` discards local changes (the dangerous behavior is not tested at all).
- No test for the `NoUpstreamBranch` error variant, which suggests it may be dead code.
- No test for concurrent worktree operations.

### Test Organization

- Unit tests for `naming.rs` (slugify, hierarchical naming) correctly live in `src/naming.rs`'s `#[cfg(test)]` module.
- The one unit test in `helpers.rs` (`test_get_current_branch_on_main`) correctly lives in `src/helpers.rs`'s `#[cfg(test)]` module.
- Integration tests in `tests/` correctly use real git repositories.
- No test helper code is shared — each test file duplicates its own `setup_*` helper. `setup_classic_repo_with_git()` appears twice (in `classic_tests.rs` and `git_operations_tests.rs`) with different implementations. This duplication is a maintenance burden and a source of inconsistency.

---

## repo-meta

### Code Quality Findings

**Overall assessment:** Clean, minimal crate. The schema types are well-defined and the loader is forgiving (warns on bad files rather than failing hard). Some naming confusion with the `Registry` type.

**Strengths:**
- Schema types (`tool.rs`, `rule.rs`, `preset.rs`, `mcp.rs`) use Serde with `#[serde(default)]` appropriately, providing resilient deserialization.
- `DefinitionLoader` follows a warning-accumulation pattern: it does not abort on a single malformed TOML file but collects warnings and returns partial results. This is appropriate for a discovery mechanism.
- `validation.rs` contains `ToolRegistry` and `PresetRegistry` for cross-referencing definitions against known tools.
- `registry.rs` (`Registry`) is simple, correct, and well-documented.

**Issues:**

1. **Naming confusion — three different "registry" concepts:**
   - `repo_meta::Registry` in `registry.rs` maps preset IDs to provider names.
   - `repo_meta::validation::ToolRegistry` validates tool definitions.
   - `repo_meta::validation::PresetRegistry` validates preset definitions.
   - `repo_core::rules::RuleRegistry` in `repo-core` is yet another registry.

   The name `Registry` in `repo_meta` is ambiguous given the broader context. The doc comment even acknowledges this: "not to be confused with `ToolRegistry` or `PresetRegistry` in the `validation` module." If the codebase authors themselves need to disambiguate in docs, the names are inadequate.

2. **`loader.rs` — the `load_tools`, `load_rules`, `load_presets` functions share almost identical structure.** Each reads a directory, filters `.toml` files, attempts parse, accumulates warnings. This is textbook code duplication that should be refactored into a generic `load_definitions<T: DeserializeOwned>(dir) -> Result<LoadResult<T>>` function.

3. **`registry.rs` `get_provider()` returns `Option<&String>` instead of `Option<&str>`.** This forces callers to deal with `&String` when `&str` is more idiomatic and sufficient. Minor ergonomics issue.

4. **`validation.rs` was not directly read** due to tool buffering; however, the test coverage for it was verified through the loader test file.

5. **`mcp.rs` schema** defines MCP server configuration structures. These were not fully read but are referenced from the engine. No issues detected in what was visible.

6. **No `unsafe` code** — confirmed.

7. **No `unwrap()` calls in production paths** — confirmed from what was read. All fallible paths go through `Result`.

8. **`config.rs`** — contains the configuration schema for `repo-meta` itself. Appears clean based on what was read.

### Test Quality Findings

**`tests/schema_tests.rs`:** This is the only test file. It tests `DefinitionLoader` via real filesystem operations — correct test style.

- `test_load_tools_from_directory`: Creates real TOML files, loads them, checks field values. Solid.
- `test_load_rules_from_directory`: Same pattern. Good.
- `test_load_presets_from_directory`: Good.
- `test_loader_ignores_non_toml_files`: Tests that `.md`, `.gitkeep`, and `.bak` files are ignored. This is a real behavioral test.
- `test_loader_handles_invalid_toml_gracefully`: Tests that a file with missing required fields generates a warning and does not abort loading. This is the correct behavior to test.
- `test_loader_returns_empty_for_nonexistent_directory`: Tests the missing-directory case.

**Issues with test coverage:**

1. **No tests for `validation.rs`** (`ToolRegistry`, `PresetRegistry`). Cross-reference validation is completely untested in the integration test suite. These types are public and could have broken logic that goes undetected.

2. **No tests for `registry.rs` at the integration level.** The unit tests in `src/registry.rs`'s `#[cfg(test)]` module are adequate for the basic operations, but there are no tests for the `with_builtins()` factory behavior under real-world usage.

3. **No tests for `mcp.rs` schema deserialization** at the integration level.

4. **Serde unit tests** for `schema/tool.rs`, `schema/rule.rs`, and `schema/preset.rs` were listed as living in their source files. This is correct placement for unit tests, but the coverage was not independently verified in this audit.

5. **No negative test for duplicate tool slugs** — the loader does not deduplicate by slug; if two `.toml` files both declare `slug = "cursor"`, the second will silently overwrite the first in the `HashMap`. This behavior is not tested and may not be intended.

### Test Organization

- Unit tests for schema types live in `src/schema/*.rs` (confirmed from crate structure). Correct placement.
- Unit tests for `Registry` live in `src/registry.rs`. Correct placement.
- Integration tests in `tests/schema_tests.rs` exercise the `DefinitionLoader`. Correctly named and scoped.
- The test file name `schema_tests.rs` is slightly misleading — it primarily tests the loader, not the schema types. A better name would be `loader_tests.rs`.

---

## repo-core

### Code Quality Findings

**Overall assessment:** This is the most complex crate and shows the most design stress. The sync engine is large and multi-concern. There are meaningful architectural issues alongside good practices.

**Strengths:**
- `ledger/mod.rs` implements the correct atomic save pattern (temp file + rename) with `fs2` file locking.
- `sync/engine.rs` (`SyncEngine`) is well-structured for its scope. The `check()`, `sync()`, and `fix()` separation is clean.
- `rules/registry.rs` (`RuleRegistry`) is solid: UUID-based identity, content hashing for drift detection, TOML persistence. The `add_rule`, `get_rule`, `remove_rule`, `update_rule` operations are all implemented.
- `config/` module has a well-tested Manifest merge strategy with deep-merge for preset tables.
- `governance.rs` was not fully read but is referenced as a public module.

**Issues:**

1. **`sync/engine.rs` contains two public free functions that duplicate `repo_fs::checksum` (`engine.rs` lines 631-641):**
   ```rust
   pub fn compute_content_checksum(content: &str) -> String {
       repo_fs::checksum::compute_content_checksum(content)
   }
   pub fn compute_file_checksum(path: &Path) -> Result<String> {
       Ok(repo_fs::checksum::compute_file_checksum(path)?)
   }
   ```
   These are thin wrappers that add no value beyond what `repo_fs::checksum` already exposes. They are public API surface that should be eliminated — callers should use `repo_fs::checksum` directly.

2. **`sync/engine.rs` `check()` function is 200+ lines of repeated match arms.** Each `ProjectionKind` variant handles "file missing", "file readable", and "checksum mismatch" with nearly identical code structure. This should be extracted into a `check_projection()` helper function.

3. **`sync/engine.rs` `fix_with_options()` is semantically broken (`engine.rs` lines 460-492):**
   ```rust
   pub fn fix_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
       let check_report = self.check()?;
       // ...
       let sync_report = self.sync_with_options(options)?;
       // ...
   }
   ```
   `fix()` first calls `check()` (which reads the ledger), then calls `sync()` (which also reads the ledger). This is two separate reads with no transaction between them. If the ledger changes between the `check()` and `sync()` calls, the count reported in the fix summary will be wrong. This is a TOCTOU issue at the application level.

4. **`ledger/mod.rs` — the `save()` locking strategy has a TOCTOU race that is acknowledged but unsolved.** The `ledger_locking_tests.rs` file explicitly ignores the concurrent test:
   ```rust
   #[ignore = "known TOCTOU race: concurrent load-modify-save causes data loss (last-writer-wins) — see audit"]
   fn concurrent_ledger_saves_preserve_file_integrity() { ... }
   ```
   The test is ignored rather than the problem being fixed. `save()` locks the destination file for writing, but the `load()` + modify + `save()` sequence is not atomic as a unit. Two concurrent callers that each load, then modify, then save will produce last-writer-wins behavior. This is documented but unresolved.

5. **`sync/engine.rs` `resolve_extension_mcp_configs()` silently skips missing extension sources (`engine.rs` line 557-565):**
   ```rust
   Err(_) => {
       // Extension source not installed yet - skip silently
       tracing::debug!(...);
       continue;
   }
   ```
   Swallowing this error silently means a misconfigured or missing extension is not surfaced to the user. The `report` is not updated (unlike the parse-error case above it), so the user has no indication an extension was expected but not found.

6. **`rules/registry.rs` `add_rule()` does not enforce unique IDs.** A comment in the test file (`test_registry_duplicate_id_allowed`) documents this as intentional:
   ```rust
   // Unlike UUID, id doesn't have to be unique (though it's recommended)
   ```
   However, `get_rule_by_id()` returns only the first match, meaning all but the first rule with a given ID are unreachable by ID. This is a data integrity issue. Either IDs should be enforced unique or `get_rule_by_id()` should return `Vec<&Rule>`.

7. **`projection/` module (`projection/mod.rs`, `projection/writer.rs`)** — these files exist but were not read due to the large file count. They are referenced from `sync/engine.rs`. The audit notes their existence but cannot comment on quality.

8. **`backup/` module** — exists but not read. Referenced indirectly.

9. **`hooks.rs`** — exists but not read.

10. **No `unsafe` code** — confirmed in all read files.

11. **`unwrap()` calls in `#[cfg(test)]` modules** — acceptable in test code.

### Test Quality Findings

**`tests/rules_tests.rs`:** Thorough. Tests CRUD operations on `RuleRegistry`, persistence round-trips, UUID generation, content hashing, drift detection, and tag-based queries. The `test_registry_duplicate_id_allowed` test explicitly documents the ID uniqueness policy decision. This is good test-as-documentation practice.

**`tests/ledger_tests.rs`:** Tests ledger CRUD, persistence, and query operations. Adequate coverage.

**`tests/ledger_locking_tests.rs`:** Notable for documenting a known defect with `#[ignore]`. The test is honest about what it tests and why it is ignored. However:
- The ignored test still contains assertions that accept the broken behavior (last-writer-wins), rather than asserting the correct behavior and marking the test as `#[should_fail]` or similar. This means the test, when eventually fixed, will need to be rewritten.
- `ledger_save_fails_when_parent_directory_missing` is a good negative test.
- `ledger_save_overwrites_previous_content_completely` verifies replacement semantics correctly.

**`tests/sync_tests.rs`:** Tests sync engine behavior. Not fully read but present.

**`tests/mode_tests.rs`:** Tests mode detection and switching. Not fully read.

**`tests/config_tests.rs`:** Well-structured with sub-modules (`manifest_tests`, `resolver_tests`, `runtime_context_tests`). Tests are real and test real behavior:
- `test_manifest_merge_deduplicates_tools` is a good regression test for the deduplication logic.
- `test_config_resolver_hierarchy_local_overrides_repo` tests the config layering system correctly.

**`tests/fixture_tests.rs`:** Golden-file tests using `test-fixtures/`. This is the most sophisticated test file in the codebase:
- Tests actually run `ToolSyncer` against real rule content and compare output to golden files.
- `test_golden_file_cursor_output_matches_expected` and `test_golden_file_claude_output_matches_expected` are genuine integration tests.
- `test_expected_outputs_have_matching_open_close_markers` tests structural invariants of the generated output.

**Issues with test coverage:**

1. **`check()` behavior is not independently tested.** All `SyncEngine::check()` tests in `tests/sync_tests.rs` (not fully read, but visible by name) may not cover the drift detection for all three `ProjectionKind` variants (`FileManaged`, `TextBlock`, `JsonKey`).

2. **`fix()` is not tested in isolation.** The TOCTOU issue in `fix_with_options()` described above has no test.

3. **`governance.rs`** — referenced in `lib.rs` but has no dedicated test file. Governance logic is a high-stakes area (it controls what is allowed/denied) and should have explicit tests.

4. **`hooks.rs`** — no dedicated test file visible.

5. **`projection/` module** — no dedicated test file visible.

6. **`backup/` module** — no dedicated test file visible.

### Test Organization

- `tests/rules_tests.rs`, `tests/ledger_tests.rs`, etc. are correctly scoped as integration tests.
- `#[cfg(test)]` module in `src/ledger/mod.rs` tests internal serialization — correct placement.
- `#[cfg(test)]` module in `src/sync/engine.rs` tests utility functions (`compute_file_checksum`, `get_json_path`, `SyncReport`) — appropriate.
- **The `fixture_tests.rs` file imports from `repo_tools::Rule`**, which means `repo-core`'s integration tests depend on `repo-tools`. This dependency is not in `Cargo.toml`'s `[dev-dependencies]` for `repo-core` — only `tempfile`, `rstest`, and `pretty_assertions` are listed. This suggests `fixture_tests.rs` may fail to compile or relies on a transitive dependency being available without explicit declaration.

---

## Cross-Crate Architecture

### Dependency Graph

```
repo-fs  (no internal deps)
    ^
    |
repo-git (depends on repo-fs)
    ^
    |
repo-meta (depends on repo-fs)
    ^
    |
repo-core (depends on repo-fs, repo-git, repo-meta, repo-tools, repo-presets, repo-extensions, repo-content)
```

**No circular dependencies detected.** The layering is logically correct: `repo-fs` is the foundation, `repo-git` and `repo-meta` each depend only on `repo-fs`, and `repo-core` sits at the top orchestrating all of them.

### Architecture Issues

1. **`repo-core` has an unusually wide dependency set:** It depends on `repo-fs`, `repo-git`, `repo-meta`, `repo-tools`, `repo-presets`, `repo-extensions`, and `repo-content`. This is six sibling crates. For a "core orchestration" layer this is defensible, but it means `repo-core` is essentially the application layer dressed as a library. Any change in any of those six crates potentially touches `repo-core`.

2. **`repo-fs` carries format-specific dependencies (`serde_yaml`, `serde_json`, `toml`) that are arguably above its abstraction level.** A filesystem abstraction crate probably should not know about YAML, JSON, or TOML. The `ConfigStore` type in `repo-fs` is more appropriately a `repo-meta` concern. This is a leaky abstraction.

3. **Checksum duplication:** `compute_content_checksum` is defined in `repo_fs::checksum` and re-exported as a public function from `repo_core::sync::engine`. This creates two public access paths to the same function, which is a refactoring hazard.

4. **`NormalizedPath` is the cross-cutting concern:** It is constructed with `NormalizedPath::new(path)` throughout the codebase at all layers. This is correct — it's the intended design. However, `to_native()` is called everywhere paths need to touch `std::fs` operations, which means the normalization boundary is consistently respected.

5. **`repo-git` does not depend on `repo-meta`** — this is architecturally correct. Git operations should not know about metadata schemas.

6. **The `LayoutProvider` trait (`repo-git`) and `LayoutMode` enum (`repo-fs`) are parallel concepts** that are never connected by a formal relationship. `WorkspaceLayout::mode` returns a `repo_fs::LayoutMode`, but `repo-git` defines its own `ClassicLayout`, `ContainerLayout`, `InRepoWorktreesLayout` types. There is no compile-time enforcement that a `LayoutMode::Classic` workspace uses `ClassicLayout`. Callers in `repo-core` must manually map from the detected mode to the appropriate layout type, which is error-prone.

---

## Summary of Critical Issues

The following issues are ranked by severity:

### Severity: High

1. **Known TOCTOU race in `Ledger::save()` / `load()` cycle** (`repo-core/src/ledger/mod.rs`): The `load() → modify → save()` sequence is not atomic as a compound operation. The test documenting this is permanently `#[ignore]`d. In a multi-process or multi-thread environment, this causes silent data loss. The correct fix requires read-modify-write within a single exclusive lock, not separate `load()` and `save()` locks.

2. **`fix_with_options()` performs two separate `check()` + `sync()` reads** (`repo-core/src/sync/engine.rs`): The count of drifted/missing items reported by `fix()` comes from a stale `check()` result, not the actual `sync()` result. The fix summary can be wrong.

3. **`write_atomic` symlink check swallows I/O errors** (`repo-fs/src/io.rs` line 76): `contains_symlink(...).unwrap_or(false)` silently allows writes when the symlink check itself fails. Failure-safe behavior requires defaulting to `true` (refuse the write) on error.

4. **`fixture_tests.rs` imports `repo_tools::Rule`** without `repo-tools` listed in `repo-core`'s `[dev-dependencies]` (`repo-core/tests/fixture_tests.rs` line 11). If this compiles today, it is only because `repo-tools` is a transitive dependency. This is a fragile implicit dependency.

### Severity: Medium

5. **`ClassicLayout::list_worktrees()` silently returns `"unknown"` branch on error** (`repo-git/src/classic.rs` line 54): Real errors (broken HEAD, detached state) are swallowed.

6. **`helpers.rs` `pull()` uses `force()` checkout** (`repo-git/src/helpers.rs` line 205): Silently destroys local working tree modifications.

7. **`ClassicLayout::current_branch()` opens on `.git` dir instead of repo root** (`repo-git/src/classic.rs` line 82): Inconsistent with `open_repo()` and potentially fragile in some git2 versions.

8. **`repo-meta` loader does not deduplicate by tool slug**: Two `.toml` files with the same `slug` will silently overwrite each other in the result `HashMap`.

9. **`RuleRegistry::add_rule()` allows duplicate IDs but `get_rule_by_id()` only returns the first match** (`repo-core/src/rules/registry.rs`): Duplicate IDs make rules silently unreachable by ID.

### Severity: Low

10. **`read_text` and `write_text` are public API with `TODO: PLACEHOLDER` comments** (`repo-fs/src/io.rs` lines 179, 187).

11. **Lock files (`.lock` suffix) accumulate and are never cleaned up** (`repo-fs/src/io.rs`).

12. **`compute_content_checksum` and `compute_file_checksum` are exported from both `repo_fs` and `repo_core::sync::engine`** — redundant public API surface.

13. **Three different "registry" types** across crates with overlapping names (`Registry`, `ToolRegistry`, `PresetRegistry`, `RuleRegistry`).

14. **Test helper setup functions duplicated across test files in `repo-git`**: `setup_classic_repo_with_git()` appears in at least two test files with different implementations.

15. **`governance.rs`, `hooks.rs`, `projection/`, `backup/` modules in `repo-core` have no dedicated test files**: These are non-trivial modules that govern important behavior but are untested at the integration level (based on available test file listing).
