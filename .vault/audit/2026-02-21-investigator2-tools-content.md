---
tags: ["#audit", "#tools-content"]
related: ["[[2026-02-21-investigator1-core-crates]]", "[[2026-02-21-investigator3-cli-mcp-integration]]"]
date: 2026-02-21
---

# Investigator2 Report: Tools, Content & Supporting Crates

## repo-tools

### Code Quality Findings

**Architecture**

The crate implements 13 tool integrations (Cursor, Claude, VS Code, Windsurf, Copilot, Cline, Gemini, Antigravity, JetBrains, Zed, Aider, Amazon Q, Roo) via a shared `ToolIntegration` trait and a `ToolDispatcher` registry. The `registry/` module is described as a "single source of truth" that eliminated a previous three-location duplication — this is good history but means the documentation implies there was a prior structural debt.

The `mcp_translate.rs` module (`to_tool_json` / `from_tool_json`) is well-structured and handles the substantial variation in MCP field naming across tools (e.g., `url` vs `serverUrl` vs `httpUrl`, `type` vs absent). The `from_tool_json` transport-detection logic is non-trivial — SSE/HTTP disambiguation when both transports share the same URL field requires checking the `type` field value, and the code correctly handles this with explicit precedence ordering. However the detection path has multiple fallback branches (`detect_transport_by_type`) that are reached only in edge cases, making the control flow difficult to follow without the comments.

**Specific Issues**

1. **`update_block` regex compiled at call time (writer.rs, line 96-101):** `Regex::new(&pattern).expect(...)` — every call to `update_block` and `remove_block` compiles a fresh regex. The same applies to `remove_block` (line 143-148). For a library called in hot sync loops this is wasteful. The open-marker regex in `parser.rs` correctly uses `LazyLock`, but the writer does not follow the same pattern. The fallback `.expect("UUID should produce valid regex pattern")` is also a production-panic risk if a UUID somehow contains characters that break regex syntax (unlikely given the character set, but the OPEN_MARKER_REGEX explicitly constrains UUID characters only at parse time, not at write time, so a UUID injected from outside can contain arbitrary characters).

2. **`from_tool_json` silently swallows unknown inputs:** Returns `None` for unrecognizable JSON. This is acceptable for parsing, but callers have no way to distinguish "this is not an MCP entry" from "this is a malformed MCP entry". A `Result<Option<...>>` or an error enum would give callers better diagnostic information.

3. **`mcp_installer.rs` and `syncer.rs`** — these were not directly readable in full from the persisted results. However, given the registry/dispatcher architecture, if either file contains `unwrap()` calls on I/O operations, those are potential panics in production. Based on what was visible in the integration tests, file writes go through the integration trait's `sync` method, which does propagate `Result`.

4. **`translator/capability.rs` and `registry/builtins.rs`** — the translator capability module was small and focused; the builtins registry is a correct single-source declaration.  No unsafe code observed.

5. **Public API documentation:** `lib.rs` re-exports are not individually documented — the module-level doc comment explains the architecture at a high level, but individual public types in tool integration files lack `///` doc comments in many places (e.g., `SyncContext`, `Rule`). This is a minor maintainability concern.

6. **No `unsafe` code** observed across the crate. No dead code was flagged in reviewed files.

**Complexity Hotspots**

- `from_tool_json` in `mcp_translate.rs`: ~60 lines of nested conditional transport detection. The logic is correct but dense. A named helper extracting "what transport is this?" as a separate function would improve clarity.
- The 13-tool integration: each integration file (`claude.rs`, `cursor.rs`, `vscode.rs`, `windsurf.rs`, `generic.rs`, etc.) likely has repetitive patterns. The `generic.rs` file suggests an abstraction exists for text-file rules integrations, but VS Code has unique behavior (JSON config), so the abstraction is appropriately bounded.

### Test Quality Findings

**What exists:**
- `tests/integration_tests.rs` — a very large file with end-to-end sync tests per tool (Cursor, Claude, Copilot, VS Code, Windsurf) plus cross-tool and dispatcher tests. Tests write real files to `TempDir` and read them back.
- `tests/dispatcher_tests.rs` — dispatcher-level tests.
- `tests/claude_tests.rs` — additional Claude-specific tests.
- `src/mcp_translate.rs` contains an inline `#[cfg(test)]` module with extensive per-tool roundtrip tests.

**Strengths:**
- Integration tests verify actual file content, not just "did it not panic". They check for block markers, rule content, idempotency, cross-tool isolation, and `sync_all` behavior.
- The idempotency test (`test_tool_idempotency`) syncs three times and compares outputs — this is a meaningful property test.
- `test_cross_tool_sync_does_not_interfere` tests isolation between tools operating on separate files.
- `test_tools_write_to_separate_files` uses distinct content strings per tool to assert no leakage.
- `mcp_translate.rs` unit tests cover all 13 MCP-capable tools for stdio translation and include roundtrip tests for all major transports.

**Weaknesses:**
1. **MCP config write tests missing from integration suite.** The `mcp_installer.rs` MCP installation path is not covered by the visible integration tests. Tests verify translation logic (in `mcp_translate.rs`) but not the end-to-end flow of reading a canonical config, translating it, and writing it to the tool-specific file.

2. **No error-path integration tests.** None of the `tests/integration_tests.rs` tests verify error conditions: what happens if the target directory is not writable, if an existing file is malformed, or if `sync` is called on a path that doesn't exist. All tests use `TempDir::new().unwrap()` on a writable temp dir.

3. **VS Code test is shallow.** `test_vscode_settings_json_has_python_path` only checks that `python.defaultInterpreterPath` is set. It does not verify that repeated syncs don't duplicate or corrupt existing settings, that other settings keys are preserved, or that invalid JSON in an existing file is handled gracefully.

4. **Copilot test preconditions baked in.** `test_copilot_creates_instructions_with_content` pre-creates `.github/` — the test does not verify what happens if `.github/` doesn't exist (the integration may or may not create it).

5. **`test_dispatcher_has_all_builtin_tools`** is a presence check, not a behavior check. It passes as long as the tool name is registered, regardless of whether the integration actually does anything useful.

6. **`test_sync_all_skips_unknown_tools_without_affecting_others`** tests "unknown tool silently skipped" which is the documented behavior, but there's no test for whether a failure in one tool's sync propagates or is isolated.

### Test Organization

- The integration test file (`tests/integration_tests.rs`) is appropriately placed in the `tests/` directory and uses tempdir-based I/O, making it a genuine integration test.
- The `mcp_translate.rs` module has inline `#[cfg(test)]` unit tests, which is appropriate for pure-function unit testing of translation logic.
- Some overlap exists: `tests/claude_tests.rs` and the claude-specific section of `tests/integration_tests.rs` likely cover similar ground. This is minor duplication.
- No shared test helper infrastructure is visible — each test file re-implements `create_rule()` independently.

---

## repo-content

### Code Quality Findings

**Architecture**

`repo-content` provides a `Document` facade over format-specific handlers (TOML, JSON, YAML, Markdown, PlainText). It supports managed blocks (insert/update/remove by UUID), path-based structured data access, semantic diffing, and edit tracking with inverse/rollback support.

The design is clean: a `FormatHandler` trait dispatches to the correct backend, and `Document` holds the source string as the mutable state, recomputing via handler methods on each operation. This avoids storing parsed ASTs in the Document, which simplifies mutability but means repeated operations re-parse the source string.

**Specific Issues**

1. **`set_path` / `remove_path` lose TOML formatting.** The round-trip for TOML path mutations goes: source -> normalize to `serde_json::Value` -> convert back to `toml::Value` -> re-render with `toml::to_string_pretty`. This destroys toml_edit's format-preserving semantics (comments, ordering, inline tables). The `handlers/` presumably uses `toml_edit` for block operations, but path set/remove bypasses it entirely. This is a correctness regression for TOML files — a caller who sets `package.name` will have their TOML reformatted, which is surprising and potentially disruptive.

2. **`render()` uses a silent fallback.** In the non-text path, if `self.handler.parse(&self.source)` fails or `render()` fails, `render()` silently returns `self.source.clone()`. This means a partially-corrupted document will silently produce the old source instead of signaling an error. The caller has no way to know whether the rendered output reflects the current state or a stale snapshot.

3. **`semantic_eq` is asymmetric by design flaw.** `semantic_eq` uses `self.handler.normalize()` for both documents, not `other.handler.normalize()`. Two documents of different formats are compared with the wrong handler for the second document (since `other.handler` is ignored). For example, comparing a JSON document to a TOML document will normalize both with the JSON handler, potentially producing incorrect results for the TOML side.

4. **`diff()` has a format matching condition that is overly broad.** The match arms `(Format::Json, _) | (Format::Toml, _) | (Format::Yaml, _) | (_, Format::Json) | (_, Format::Toml) | (_, Format::Yaml)` catch all cross-format combinations involving at least one structured format. This means comparing a Markdown document against a JSON document goes through the structured diff path, which tries to `normalize` the Markdown document as JSON (it isn't). The `Ok(norm)` guard prevents a panic, but a Markdown-vs-JSON diff falls into `SemanticDiff::with_changes(Vec::new(), 0.0)`, which is misleading: it says "0% similarity with no changes", when in reality it just failed to normalize.

5. **`get_block` calls `find_blocks()` and re-scans** on every call. `find_blocks()` itself reparsing the full source is O(n). A `get_block` by UUID calling `find_blocks()` is O(n) per call. For large files with many blocks, this adds up. This is a design-level performance issue, though not a correctness issue.

6. **`json_to_toml` cannot represent null values.** TOML does not have null, and `json_to_toml` returns `Error::parse("TOML", "TOML does not support null values")`. If the JSON normalized form contains a null (e.g., from a JSON source document), `set_path`/`remove_path` will fail on TOML documents with a confusing error. This should be documented at the API level.

7. **No `unsafe` code observed.** No panics via `unwrap()` in public API surface (errors are properly propagated via `Result`).

**Complexity Hotspots**

- `document.rs` `diff()` method: the format-dispatch logic has subtle ordering dependencies and the asymmetric-handler issue for `semantic_eq`.
- `diff.rs` `diff_values_with_depth()`: correctly depth-limited, well-structured recursive diff. The `MAX_DIFF_DEPTH = 128` is appropriate.
- `edit.rs` `inverse()`: comprehensive, covers all 8 edit kinds. Logic is slightly confusing for `PathRemove` -> `PathSet` (the kind changes), which is correct but worth a comment.

### Test Quality Findings

**What exists:**
- `tests/document_tests.rs` — document lifecycle tests (parse, insert block, update block, remove block, semantic_eq, diff, is_modified).
- `tests/path_tests.rs` — path-based get/set/remove tests, including Unicode, edge cases, nested arrays, TOML paths.
- `tests/diff_integration_tests.rs` — semantic diff tests.
- `tests/integration_tests.rs` — broader integration tests.
- Inline `#[cfg(test)]` modules in `diff.rs` and `edit.rs`.

**Strengths:**
- `path_tests.rs` is thorough: Unicode keys/values, empty string keys, numeric string keys, deeply nested paths, hyphenated keys, boolean/null values, array-of-arrays, TOML dotted keys, error paths (`set_path` for nonexistent key, `remove_path` for nonexistent key).
- `diff.rs` inline tests cover depth-limiting (200-level nesting, 150-level with differences), text diff, all change types.
- `edit.rs` inline tests cover all 8 edit kinds with roundtrip apply/inverse verification.
- The block lifecycle test in `document_tests.rs` covers insert/find/update/remove in sequence.

**Weaknesses:**
1. **`semantic_eq` asymmetry is not tested.** There is no test that calls `doc1.semantic_eq(&doc2)` where `doc1` and `doc2` have different formats. The existing test only compares two JSON documents. The cross-format comparison bug described in the code quality section is undetected.

2. **TOML format loss is not tested.** No test verifies what happens to TOML comments, ordering, or inline tables after `set_path` or `remove_path`. A test that sets a path in a TOML document with comments and checks that the comments survive (or asserts they don't, documenting the limitation) is absent.

3. **`render()` silent fallback is not tested.** There is no test that puts a document in a state where `parse()` would fail on its source, to verify the fallback behavior.

4. **`diff()` cross-format behavior is not tested.** No test compares documents of different formats (e.g., JSON vs Markdown) to verify the behavior in the degraded `with_changes(Vec::new(), 0.0)` path.

5. **`test_document_parse_auto_detect`** tests a YAML document `"host: localhost\nport: 8080\n"` and a Markdown document `"# My Document\n\nSome content here.\n"`. The auto-detection heuristics are tested, but edge cases (e.g., a YAML document that looks like Markdown, a TOML document without `[section]`) are not.

6. **Block operations on JSON** are not tested (the main test file uses PlainText for block lifecycle). JSON has a special `_repo_managed` block representation; there are no tests verifying that the JSON format handler's block insert/update/remove work correctly.

### Test Organization

- Tests in `tests/` are integration-style, operating on the public `Document` API. Appropriate.
- Inline `#[cfg(test)]` modules in `diff.rs` and `edit.rs` are unit tests of private implementation details. Appropriate.
- `tests/document_tests.rs` and `tests/integration_tests.rs` have some functional overlap.
- No shared test helpers; each test file constructs its own `Document` instances directly.

---

## repo-blocks

### Code Quality Findings

**Architecture**

`repo-blocks` contains two independent block-marker systems, clearly documented in `lib.rs`:
1. `parser` + `writer` modules: HTML comment markers (`<!-- repo:block:UUID -->`) for tool config files.
2. `formats` module: format-specific markers (TOML/YAML `#` comments, JSON `_repo_managed` key, Markdown/PlainText HTML comments) for content-level block management.

The separation is intentional and well-documented, though having two block systems in one crate with different UUID formats (short alphanumeric vs full UUID-v4) is a source of confusion for contributors.

**Specific Issues**

1. **`update_block` and `remove_block` compile regex at runtime (writer.rs).** As noted for repo-tools (which depends on this), `Regex::new(&pattern).expect(...)` is called on every invocation. The `.expect("UUID should produce valid regex pattern")` is dangerous: if a UUID arrives from user input containing regex metacharacters (e.g., `.`, `+`, `(`, `)`) that survive the parser's character class, the compiled pattern could match unintended content. The `regex::escape(uuid)` call is present and mitigates the worst cases, but the `.expect()` on `Regex::new` makes any regex compilation failure a production panic rather than a graceable error.

2. **`remove_block` leaves behind whitespace artifacts.** The regex for removal uses `\n?\n?` prefix/suffix and then calls `trim_start_matches('\n')`. The test `remove_block_cleans_up_whitespace` passes, but the whitespace normalization is fragile: removing a block in the middle of a file followed by removing another may produce different whitespace than expected. The behavior is documented only implicitly by tests, not by the API contract.

3. **Nested same-UUID blocks are silently corrupted.** The parser test `nested_blocks_with_same_uuid_uses_first_close` explicitly documents that nesting the same UUID produces unexpected behavior (the first close marker terminates the outer block). This is a known limitation, not a silent bug, but it is not reflected in any API-level documentation or error.

4. **`update_block` regex uses `(?s)` (DOTALL) mode with `.*?`.** The pattern `(?s)<!-- repo:block:{uuid} -->\n.*?\n<!-- /repo:block:{uuid} -->` matches lazily between markers. If block content itself contains a closing marker for the same UUID (block injection), the test `cross_block_marker_injection_does_not_corrupt_other_blocks` reveals that `update_block` updates the first match — which is `block-A`'s real marker — correctly. However the test also reveals that `parse_blocks` finds 3 blocks in that scenario (including the injected fake marker inside block-B's content), which means the data model allows an inconsistent state.

5. **`find_block` is O(n) via `parse_blocks().into_iter().find(...)`** — reparsing the entire document for every lookup. For large files with many blocks, callers should use `parse_blocks()` once and then query the result.

6. **No `unsafe` code.** No panics via public API `unwrap()` (errors propagate as `Result`). The `LazyLock` for `OPEN_MARKER_REGEX` is correctly initialized.

7. **`formats/` module** (TOML, JSON, YAML format handlers) handles format-specific block embedding with proper per-format comment styles. The JSON handler uses a `_repo_managed` key, which is unusual and may conflict with user data if the JSON document happens to have such a key. There is no protection against this collision.

### Test Quality Findings

**What exists:**
- `src/parser.rs` and `src/writer.rs` both have large inline `#[cfg(test)]` modules covering many edge cases.

**Strengths:**
- Parser tests cover: empty content, single/multiple blocks, line position tracking, multiline content, special characters in content, case-sensitive UUIDs, unclosed blocks (silently skipped), mismatched UUID open/close (not paired), closing-without-opening (ignored), duplicate UUIDs (both parsed), nested same-UUID blocks, empty content blocks, markers inside code blocks, UUID character class enforcement, cross-block fake-marker injection.
- Writer tests cover: insert to empty, insert to existing, update/replace, update preserves surrounding content, remove with whitespace cleanup, upsert insert/update, multiline content, content with comment-like text, cross-block marker injection with update and remove, roundtrip (upsert/remove), update specific block among three.
- Both test suites are genuine behavioral tests, not just "does it not crash".

**Weaknesses:**
1. **`formats/` module lacks its own test coverage** in the visible files. The TOML, JSON, and YAML format handlers in `formats/mod.rs`, `formats/toml.rs`, `formats/json.rs`, and `formats/yaml.rs` are relied upon by `repo-content`, but their own unit tests (if any) are in those files' inline `#[cfg(test)]` modules, which were not fully visible. Given the complexity of the JSON `_repo_managed` approach, full round-trip tests are needed.

2. **No tests for concurrent modification.** If two callers simultaneously `update_block` on the same content, the behavior is undefined (pure functions, so this is safe at the Rust level, but semantically one update will be lost). This is acceptable but undocumented.

3. **`remove_block` whitespace tests are weak.** The `remove_block_cleans_up_whitespace` test checks for absence of triple newlines, but does not verify the exact output. A stricter assertion about the resulting string (e.g., verifying `Header\n\nFooter` vs `Header\nFooter`) would be more robust.

4. **No test for `update_block` with a block that has no trailing newline after the closing marker.** Edge cases around end-of-file block positions are not covered.

### Test Organization

- All tests are in inline `#[cfg(test)]` modules within `src/parser.rs` and `src/writer.rs`. These are unit tests of non-public functions and appropriate for that placement.
- There is no `tests/` directory for `repo-blocks` integration tests. Given that `repo-blocks` is consumed by both `repo-tools` and `repo-content`, integration-level tests in those crates provide some coverage, but format-handler level tests (the `formats/` module) are not exercised from the outside.
- No test helpers are shared between parser and writer tests — each sets up content inline. This is fine for the current scale.

---

## repo-presets

### Code Quality Findings

**Architecture**

`repo-presets` implements three environment preset providers: `UvProvider` (Python/uv), `NodeProvider` (Node.js), and `RustProvider` (Rust/Cargo). Each implements the `PresetProvider` trait with `check()` and `apply()` async methods returning structured `CheckReport` and `ApplyReport` types.

`RustProvider` and `NodeProvider` are "detection-only" — they report status but do not modify the environment. `UvProvider` is the active provider that can create a virtual environment.

**Specific Issues**

1. **`check_rustc_available_sync()` and `check_node_available_sync()` are exposed as public methods** on the provider structs. These are used only in tests (to branch on whether the binary is available in the CI environment). Exposing sync shims of async operations as public API is a design smell — they exist to work around the test environment, not for production use. These should be `#[cfg(test)]` or `pub(crate)`.

2. **`UvProvider` venv path defaults.** The default venv path `.venv` and default Python version `3.12` are hardcoded in `context.rs`. If the project's `pyproject.toml` specifies a different Python version, the preset silently uses `3.12`. There is no detection of `pyproject.toml`'s `requires-python` field.

3. **`apply()` for Rust and Node returns a trivial "detection-only" report.** The `apply()` method on detection-only providers returns `Ok(ApplyReport { success: true, actions_taken: vec!["No action taken (detection-only preset)".to_string()], ... })`. This is correct behavior but could mislead callers into thinking `apply()` is a safe no-op, when in reality the contract is that `apply()` should configure the environment. If Node.js dependencies need installing, the provider gives no path to do so.

4. **No error path for missing `uv` binary in `UvProvider::apply()`.** If `uv` is not installed and `apply()` is called, the behavior depends on how the subprocess invocation handles `CommandNotFound`. If it panics or propagates an OS error, the user sees an unformatted error. The `check()` method correctly reports `Broken` when `uv` is missing, but `apply()` may still be called without checking first.

5. **`context.rs` `with_venv_tag` method** — the venv tag feature allows platform-specific venv naming (e.g., `.venv-main-win-py312`). This is good for multi-platform repos but there is no validation that the tag contains only safe filesystem characters.

6. **No `unsafe` code.** No panics via `unwrap()` in production paths.

### Test Quality Findings

**What exists:**
- `tests/detection_tests.rs` — integration tests for Rust, Node, and UvProvider detection against real temp directories.
- `tests/python_tests.rs` — more detailed UvProvider tests.

**Strengths:**
- Tests create realistic project file structures (realistic `Cargo.toml`, realistic `package.json` with dependencies and scripts, real Python files).
- Tests correctly branch on binary availability (`if provider.check_rustc_available_sync()`) rather than hardcoding expected status.
- `test_rust_not_confused_by_non_rust_files` verifies no false positives.
- `test_uv_check_with_complete_venv_structure` handles platform differences (Windows vs Unix venv layout).
- `test_uv_check_report_details_are_actionable` checks that error strings are non-trivial (length > 5).
- `test_polyglot_project_detects_multiple_presets` verifies both Rust and Node detect independently in the same directory.
- `test_context_venv_path_with_tag` verifies tagged venv path construction.

**Weaknesses:**
1. **`test_uv_provider_default`** tests `UvProvider` (the unit struct) as well as `UvProvider::new()`. Testing that both produce the same `id()` is trivial — it tests a getter, not behavior.

2. **`test_uv_provider_id`** and **`test_uv_provider_default`** are pure property/getter tests. They pass regardless of whether the provider does anything correct. They would pass even if `check()` panicked.

3. **No tests for `apply()` behavior.** `UvProvider::apply()` is the active mutation path. There are no tests for: successfully creating a venv, failing gracefully when `uv` is missing, handling an already-existing venv, or the Python version selection. The `test_rust_apply_is_detection_only` and `test_node_apply_is_detection_only` only check the return message, not actual filesystem state.

4. **`test_uv_check_with_complete_venv_structure`** writes a fake binary (just the string `"fake"`) and expects it to be treated as a valid Python interpreter. If the check actually invokes the binary (e.g., `python --version`), this test would fail or produce a false result. The test's validity depends on whether the implementation only checks file existence or also validates the binary. This is a potential false-positive test.

5. **No tests for the config override path.** `test_context_custom_python_version` tests that a config map overrides the Python version, but there are no tests for invalid version strings, version strings that don't match available Python binaries, or the `provider` config key.

6. **Detection tests are environment-coupled.** Tests fork on `check_rustc_available_sync()` at runtime. In a CI environment without Rust, the test still passes but provides weaker coverage. This is acceptable but means the test suite has different assertion strength on different machines.

### Test Organization

- Tests are in `tests/` as integration-level tests against the public API. Appropriate.
- `create_test_context` helper is duplicated between `detection_tests.rs` and `python_tests.rs`. This is a minor duplication; a shared `test_helpers.rs` would clean this up.
- No inline `#[cfg(test)]` modules in source files observed. Good — the source files focus on implementation.
- `tests/python_tests.rs` and `tests/detection_tests.rs` have some overlap (both test `UvProvider::check` on empty dirs). Not severe.

---

## repo-extensions

### Code Quality Findings

**Architecture**

`repo-extensions` provides: manifest parsing (`repo_extension.toml`), extension config (`ExtensionConfig`), MCP config resolution with template variable substitution (`mcp.rs`), and an `ExtensionRegistry` of known extensions.

The crate is well-structured. Each module has a focused purpose with clear documentation.

**Specific Issues**

1. **`mcp.rs` path traversal check has a TOCTOU caveat, acknowledged but unresolved.** The code reads the file first, then calls `canonicalize()` after reading (line 90). The comment says this avoids a TOCTOU race on the pre-read canonicalize. However, the post-read canonicalize is itself racy: between reading the file and calling `canonicalize()`, a symlink could be added or changed. The comment acknowledges this is better than pre-check, but the real fix would be a chroot-style approach (operating on a file descriptor obtained via `openat` within the extension source directory). For practical purposes this is probably acceptable, but the security comment should be more explicit that the window remains.

2. **`canonicalize()` failure is treated as a security-critical error.** If `canonicalize()` fails for a non-security reason (e.g., a valid file in a directory with unusual permissions, or a race where the file was just created), the entire `resolve_mcp_config` call fails with a "refusing to load" error. This is fail-safe but may produce confusing errors in legitimate edge cases.

3. **`EntryPoints::resolve_one` security mitigation is incomplete.** Absolute paths are "forced relative" by stripping leading `/` and resolving against `source_dir`. This is documented as a security measure, but stripping `/` from `/etc/passwd` gives `etc/passwd` relative to `source_dir`, which would resolve to `source_dir/etc/passwd`. This silently redirects rather than rejecting — a warning is logged but execution continues. The right behavior for a security control is to reject the manifest, not to silently redirect it. The `tracing::warn!` is insufficient.

4. **`merge_mcp_configs` last-write-wins with only a tracing warning.** If two extensions define the same MCP server name, the later one silently wins. For security-sensitive MCP server configurations, this is a risk — a malicious extension could shadow a legitimate one. The warning is not actionable from the caller's perspective (no way to detect or reject duplicates).

5. **`ExtensionRegistry` contains only one built-in extension** (`vaultspec`). The `with_known()` constructor hard-codes a single entry. This is fine for the current scope but the "known extensions" concept implies a catalog that could grow, and there is no mechanism (e.g., a TOML file) to declare known extensions without recompiling.

6. **No `unsafe` code.** No panics via `unwrap()` in public API. `ExtensionManifest::from_toml` uses `toml::from_str` which returns `Result`. Validation is thorough (name character set, semver version).

7. **`ExtensionMeta` uses `#[serde(deny_unknown_fields)]`** but `ExtensionManifest` does not. This asymmetry means unknown fields in top-level sections (e.g., `[unknown_section]`) are silently ignored, which is tested and intentional. The `ExtensionMeta` strictness is appropriate for the tightly-specified section.

### Test Quality Findings

**What exists:**
- `src/manifest.rs` has a large inline `#[cfg(test)]` module (30+ tests).
- `src/mcp.rs` has a large inline `#[cfg(test)]` module including security tests.
- `src/registry.rs` has a small inline `#[cfg(test)]` module.
- `src/config.rs` has a minimal inline `#[cfg(test)]` module.

**Strengths:**
- `manifest.rs` tests cover: full manifest parse, minimal manifest, invalid version, missing name, missing version, missing extension section, empty name, name with spaces, name with hyphens/underscores, unknown top-level section accepted, unknown field in extension section rejected, empty provides vectors, TOML round-trip, `from_path` reads file, `from_path` not found, error messages contain actionable context (the invalid value), entry point resolution for CLI-only / MCP-with-args / both / empty, manifest with `mcp_config` field.
- `mcp.rs` tests include **security tests**: absolute path rejection, parent traversal rejection (`../../secret.json`), template chaining does not re-expand context values, unknown template variable preserved, unclosed template preserved.
- The `test_parent_traversal_rejected` test writes a real file outside the extension directory and verifies it cannot be loaded — this is a true security regression test.
- `test_template_chaining_does_not_expand` is a subtle and important test: verifies that if `ctx.root` itself contains `{{extension.source}}`, it does NOT get re-expanded. This is correct behavior that prevents injection via context values.

**Weaknesses:**
1. **`test_reads_and_resolves_mcp_json`** creates a `TempDir` and writes a file, then calls `resolve_mcp_config`. The canonicalize check inside `resolve_mcp_config` runs on `tmp.path()` (a real temp dir), which should pass. But the test hardcodes `ctx.extension_source = "/repo/.repository/extensions/test-ext"` while the actual `source_dir` passed is `tmp.path()`. The canonicalize check compares `canon_full` (relative to `tmp.path()`) against `canon_source` (relative to `tmp.path()`), which succeeds — but the `ctx.extension_source` value used in template substitution is unrelated to `tmp.path()`. This means template variables in the test are not verified against a real extension directory.

2. **The `test_parent_traversal_rejected` test verifies the error occurs but does not verify the extension source directory itself is unchanged.** A more robust test would verify that the file outside the directory was never read (e.g., by checking that the returned error type is `McpConfigParse`, not an I/O error from reading).

3. **No test for `merge_mcp_configs` with a non-object input.** The `merge_mcp_configs` function iterates `configs.as_object()` and silently skips non-objects. There is no test that passes a non-object `Value` in the `configs` slice.

4. **`registry.rs` tests** are purely structural: does it contain entries, is sorting correct, does registration replace existing entries. No test exercises the `with_known()` extensions in a meaningful workflow.

5. **No integration test** combining `ExtensionManifest`, `resolve_mcp_config`, and `merge_mcp_configs` in a single end-to-end scenario (parse manifest -> resolve MCP -> merge with another extension's config). The three modules are tested in isolation.

### Test Organization

- All tests are in inline `#[cfg(test)]` modules. For a library without complex I/O side effects, this is appropriate — the MCP tests use `TempDir` for file operations but otherwise operate on in-memory data.
- No `tests/` directory exists for `repo-extensions`. Given the security-sensitivity of the MCP resolution path, an integration-level test in `tests/` (e.g., testing the full extension loading pipeline against a real extension directory structure) would add confidence.
- Test helpers (`make_manifest`, `make_ctx`) in `mcp.rs` are scoped to that module's `#[cfg(test)]` block, which is appropriate.

---

## Cross-Crate Observations

### Two Block Systems in `repo-blocks`

`repo-blocks` houses two independent block-marker systems with different UUID formats and different use cases. Both systems are consumed by other crates in the workspace. This creates a learning-curve problem for contributors: which system do I use? The `lib.rs` documentation explains the distinction well, but the crate structure (one crate, two systems) means there is no compile-time enforcement of which system is appropriate in which context. A contributor calling `repo_blocks::insert_block` (HTML comment, short UUID) instead of the `formats` module (format-specific markers, UUID-v4) will produce incorrect output silently.

**Recommendation:** Consider splitting into `repo-blocks-raw` (HTML comment markers) and `repo-blocks-format` (format-specific), or at minimum adding `#[deprecated(note = "Use formats module for content-level blocks")]` markers on the parser/writer exports when called from a `repo-content` context.

### `repo-content` Re-implements Block Detection

`repo-content`'s `handlers/` module implements its own block detection (via `FormatHandler::find_blocks`) independently of `repo-blocks::parser`. While the two systems serve different purposes (different marker formats, different UUID types), the conceptual overlap means bug fixes in one do not automatically apply to the other. The diffing logic for detecting block-level changes (`SemanticChange::BlockAdded` / `BlockRemoved`) in `diff.rs` uses generic change types that apply to both structured and text diffs, but the block-aware diff variant is not wired to UUID-specific tracking.

### Test Helper Duplication

`create_test_context` (in `repo-presets`) and `create_rule` (in `repo-tools` test files) are both duplicated across test files within their respective crates. Neither crate has a `tests/common/mod.rs` or similar shared helper module. This is minor but will compound as test coverage grows.

### Security Boundary: Extension MCP Paths

The path traversal prevention in `repo-extensions/src/mcp.rs` is the most security-sensitive code in the audited crates. The post-read canonicalize approach is a practical compromise, but the TOCTOU window (between `read_to_string` and `canonicalize`) is real. In practice this window is very small and exploitation requires write access to the extension directory, which would already imply compromise. The absolute-path rejection in `EntryPoints::resolve_one` uses silent redirection rather than rejection, which is a weak security control.

### Missing `tests/` for `repo-blocks` and `repo-extensions`

Both crates place all tests in inline `#[cfg(test)]` modules. For `repo-blocks`, the format-handler modules (`formats/toml.rs`, `formats/json.rs`, `formats/yaml.rs`) are tested indirectly through `repo-content`, but not directly. For `repo-extensions`, the security-critical `mcp.rs` tests are in-module but could benefit from integration-level tests in `tests/`.

---

## Summary of Critical Issues

### Critical (would cause incorrect or dangerous behavior)

1. **`semantic_eq` uses wrong handler for second document** (`repo-content/src/document.rs`). Comparing documents of different formats uses only `self.handler` for normalization. Cross-format comparison produces incorrect equality results.

2. **`set_path` / `remove_path` destroy TOML formatting** (`repo-content/src/document.rs`). Path mutations on TOML documents re-serialize via `serde_json::Value -> toml::Value -> toml::to_string_pretty`, losing all comments, ordering, and inline table formatting. This is a silent regression for TOML file management.

3. **`EntryPoints::resolve_one` silently redirects absolute paths** (`repo-extensions/src/manifest.rs`, line ~167-174). A manifest with an absolute entry point path (e.g., `/usr/bin/bash`) is redirected to `source_dir/usr/bin/bash` with only a warning log. For a security boundary, this should be a hard error that rejects the manifest.

4. **`update_block` and `remove_block` in `repo-blocks/src/writer.rs` compile regex at call time with `.expect()`.** If a UUID (arriving from user input) contains characters that produce an invalid regex after `regex::escape` (which handles most cases but not all), this panics in production. The fix is to return a `Result` from regex compilation.

### High (significant quality or correctness issue)

5. **`diff()` cross-format degraded path produces misleading `SemanticDiff`** (`repo-content/src/document.rs`). Comparing Markdown vs JSON returns `is_equivalent: false, changes: [], similarity: 0.0` — looks like "completely different with no changes", which is logically contradictory and unhelpful.

6. **`render()` has silent fallback** (`repo-content/src/document.rs`). Parsing/rendering failure silently returns the old source, making it impossible for callers to detect that the rendered output does not reflect current document state.

7. **MCP config end-to-end not tested** in `repo-tools`. The `mcp_translate.rs` translation logic is tested, but the full pipeline (read canonical MCP config -> translate per-tool -> write to tool config file) is not covered by integration tests.

8. **No error-path tests for `repo-tools` integration.** Tool sync failures (unwritable directories, malformed existing files) are not tested.

9. **`UvProvider::apply()` is not tested** (`repo-presets`). The active environment-modification path has zero test coverage for its actual effect.

10. **`_repo_managed` JSON key collision** in `repo-blocks/src/formats/json.rs`. If a user's JSON document contains a `_repo_managed` key, the block system will misinterpret user data as managed blocks, potentially causing data corruption. No collision detection or error is emitted.

### Medium (maintainability or correctness debt)

11. **`check_rustc_available_sync()` and `check_node_available_sync()` are public** (`repo-presets`). Test-only helpers exposed as public API.

12. **`find_block` is O(n) per call** (`repo-blocks/src/parser.rs`). Fine for current usage, but callers doing multiple lookups should be guided to call `parse_blocks()` once.

13. **No shared test helpers** across test files within each crate. `create_test_context`, `create_rule` etc. are duplicated.

14. **No tests for `formats/` module in `repo-blocks`** directly. TOML, JSON, and YAML format handlers are tested only indirectly via `repo-content`.

15. **`merge_mcp_configs` last-write-wins is silent** (`repo-extensions/src/mcp.rs`). Callers cannot detect or reject duplicate MCP server names across extensions.
