# Code Quality Audit Report — Investigator 3

**Scope:** Code quality, readability, and adherence to best practices
**Crates audited:** repo-meta, repo-mcp, repo-presets, repo-tools, repo-agent, repo-blocks,
repo-content, repo-core, repo-cli, repo-fs, repo-git
**Date:** 2026-02-18

---

## Executive Summary

The codebase is in generally good shape. Error handling is consistent, unwrap() is absent
from production code, and the architecture is well-structured with clear separation of
concerns. There are, however, a cluster of actionable issues: silent error swallowing in
critical paths, a naming collision between two unrelated `ToolRegistry` types, checksum
format inconsistencies, and architectural leftovers from a migration that is mostly but
not fully complete.

**Severity key:** Critical > High > Medium > Low > Cosmetic

---

## Finding 1 — Silent save-error discard in `RuleRegistry::remove_rule`

**File:** `crates/repo-core/src/rules/registry.rs`
**Severity:** High

```rust
pub fn remove_rule(&mut self, id: &str) -> Option<Rule> {
    ...
    self.save().ok()?  // <-- silently discards save errors
}
```

The `.save().ok()?` idiom propagates `None` to the caller when `save()` fails, but the
caller receives the same `None` it would get if the rule simply did not exist. There is
no way to distinguish "rule not found" from "rule removed from memory but not persisted."
The in-memory state and the on-disk TOML file will silently diverge.

**Recommendation:** Return `Result<Option<Rule>>`. The caller can distinguish the failure
cases and the error is not swallowed.

---

## Finding 2 — `ToolRegistry` naming collision

**Files:**
- `crates/repo-meta/src/validation.rs` — defines `ToolRegistry` with a hardcoded list
  of known tool/preset slugs for validation purposes
- `crates/repo-tools/src/registry/store.rs` — defines a different `ToolRegistry` with
  the actual `HashMap`-backed registry and `get/list/by_category` API

These two types share the same name, live in different crates, and serve completely
different roles. Anyone `use repo_meta::*; use repo_tools::*;` gets a compilation error.
More importantly, they can diverge silently: the hardcoded list in repo-meta must be
manually kept in sync with the `builtin_registrations()` list in repo-tools.

**Recommendation:** Rename the repo-meta type to `KnownToolSlugs` or `ToolSlugList`, or
delete it entirely if repo-tools is the canonical source of truth and validation can
delegate to it at runtime.

---

## Finding 3 — Checksum format inconsistency

**Files:**
- `crates/repo-core/src/rules/rule.rs` — stores checksums with `"sha256:"` prefix:
  `format!("sha256:{:x}", result.finalize())`
- `crates/repo-core/src/projection/writer.rs` — stores checksums as bare hex:
  `format!("{:x}", result.finalize())`
- `crates/repo-content/src/block.rs` — also bare hex

The ledger compares checksums from projections against stored rule checksums. If these
two formats are ever compared directly (which is likely given the projections track
`TextBlock` content with a checksum), the comparison will silently fail: `"sha256:abc"`
never equals `"abc"`.

**Recommendation:** Establish a single canonical format (the `"sha256:"` prefix is the
more descriptive choice) and use a shared constant or helper function across all crates.

---

## Finding 4 — Silent IO error swallowing in MCP resource handlers

**File:** `crates/repo-mcp/src/resource_handlers.rs`
**Severity:** High

```rust
fn read_config(root: &Path) -> String {
    std::fs::read_to_string(config_path)
        .unwrap_or_else(|_| "# Config not found".to_string()) // error lost
}

fn read_rules() {
    for entry in entries {
        if let Ok(rule_content) = std::fs::read_to_string(...) {
            // silently skips unreadable files with no logging
        }
    }
}
```

IO errors (permissions, encoding issues, partial disk) are silently replaced with
default content. An MCP client asked for the config or rules will receive plausible-
looking but incorrect content with no indication that anything went wrong.

**Recommendation:** Return `Result<String>` from these functions and propagate the error
to the MCP protocol layer, which already has error response infrastructure
(`JsonRpcError`). At minimum, log the error before returning the default.

---

## Finding 5 — `VSCodeIntegration` architectural inconsistency

**File:** `crates/repo-tools/src/vscode.rs`
**Severity:** Medium

All 12 other tools use `GenericToolIntegration` via a factory function and are fully
schema-driven. `VSCodeIntegration` is the sole remaining bespoke implementation with
its own JSON-handling code. The same JSON update logic that exists in
`GenericToolIntegration::sync_json()` is partially duplicated here.

This creates two maintenance burdens: any schema-level change must be applied twice,
and `VSCodeIntegration` is an exception that new contributors must learn separately.

**Recommendation:** Migrate `VSCodeIntegration` to `GenericToolIntegration` with a
`vscode_integration()` factory function, consistent with all other tools. The
`SchemaKeys` mechanism already supports the custom JSON key paths VS Code needs.

---

## Finding 6 — `CapabilityTranslator` is a zero-sized struct with only static methods

**File:** `crates/repo-tools/src/translator/capability.rs`
**Severity:** Low / Cosmetic

```rust
pub struct CapabilityTranslator;

impl CapabilityTranslator {
    pub fn translate_mcp(def: &ToolDefinition) -> Option<McpConfig> { ... }
    pub fn translate_capabilities(def: &ToolDefinition) -> TranslatedCapabilities { ... }
}
```

The struct holds no state and is never instantiated. All callers use
`CapabilityTranslator::translate_mcp(...)` as a free-function call with a type prefix.
This is a patterns mismatch: Rust convention for namespace-like groupings of free
functions is a module, not an empty struct with associated functions.

**Recommendation:** Convert to module-level `pub fn` in `capability.rs` or inline into
`translator/mod.rs`. No behavioral change required.

---

## Finding 7 — `name()` returns slug, requires Clippy suppression

**File:** `crates/repo-tools/src/generic.rs`
**Severity:** Low**

```rust
#[allow(clippy::misnamed_getters)]
fn name(&self) -> &str {
    &self.definition.meta.slug  // returns slug, not "name"
}
```

The `ToolIntegration` trait defines `name()` but the implementation returns the slug.
The Clippy suppression annotation documents the inconsistency but does not resolve it.

**Recommendation:** Rename the trait method to `slug()` and add a separate `display_name()`
if needed, removing the suppression. This change would require updating all call sites
and the trait definition in `integration.rs`.

---

## Finding 8 — Deprecated `new()` functions still present in all tool modules

**Files:** `cursor.rs`, `claude.rs`, `windsurf.rs`, `gemini.rs`, `zed.rs`, `aider.rs`,
`copilot.rs` (all in `crates/repo-tools/src/`)
**Severity:** Low

Each file contains a `pub fn new()` with a `# Deprecated` doc comment:

```rust
/// # Deprecated
/// Use `cursor_integration()` instead.
pub fn new() -> GenericToolIntegration {
    cursor_integration()
}
```

These functions are not marked `#[deprecated]` (the Rust attribute), so they emit no
compiler warning when called. A user reading only the function signature will not see
the deprecation notice.

**Recommendation:** Either add `#[deprecated(since = "X.Y.Z", note = "use xxx_integration() instead")]`
attributes, or remove the deprecated functions entirely if they have no external callers.
Type aliases (`pub type CursorIntegration = GenericToolIntegration;`) are already
present and provide sufficient backward compatibility for type-level usage.

---

## Finding 9 — `sync_yaml` does full file replacement (no content preservation)

**File:** `crates/repo-tools/src/generic.rs`
**Severity:** Medium

```rust
fn sync_yaml(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
    // ...
    io::write_text(path, &content)?; // full replacement every sync
    Ok(())
}
```

Unlike `sync_json()` (which merges into existing JSON) and `sync_text()` (which uses
managed blocks), `sync_yaml()` replaces the entire YAML file on every sync. Any user-
added YAML content outside the managed section will be silently deleted.

**Recommendation:** Implement managed-block-based YAML update (using `repo-blocks`'s
YAML format handler) or at minimum document this destructive behavior in the function
and the `ConfigType::Yaml` variant.

---

## Finding 10 — `CheckReport` naming collision between repo-core and repo-presets

**Files:**
- `crates/repo-core/src/sync/check.rs` — `CheckReport` with `merge()`, `CheckStatus`,
  `DriftItem`
- `crates/repo-presets/src/provider.rs` — `CheckReport` with `PresetStatus`, `ActionType`

Two completely different types share the same name across crates. They cannot be used
together in the same file without explicit qualification. The repo-presets version
is not a superset of the repo-core version; they model different domains.

**Recommendation:** Rename the repo-presets type to `PresetCheckReport` or
`ProviderCheckReport` to differentiate it. Alternatively, unify the check-report
concept into a shared crate if they serve the same conceptual purpose.

---

## Finding 11 — Error type conversion loses type information in `ProjectionWriter`

**File:** `crates/repo-core/src/projection/writer.rs`
**Severity:** Medium

```rust
fn set_json_path(...) -> std::io::Result<()> {
    serde_json::from_str(...)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
}
```

JSON parse errors and IO errors are coerced to `io::Error` via string formatting. The
original error type, error code, and structured context are lost. Downstream error
handlers cannot distinguish a JSON parse failure from a file permission error.

**Recommendation:** Define a dedicated `ProjectionError` enum in this module (or reuse
an existing error from `repo-core`), wrapping both IO and JSON errors without string
conversion.

---

## Finding 12 — Hardcoded tool/preset list in repo-meta must be manually kept in sync

**File:** `crates/repo-meta/src/validation.rs`
**Severity:** Medium

```rust
pub fn with_builtins() -> Self {
    let mut registry = Self::new();
    registry.register("claude");
    registry.register("cursor");
    // ... 13 more hardcoded strings
}
```

This list must be manually synchronized with `crates/repo-tools/src/registry/builtins.rs`
where the actual tool definitions live. There is no compile-time or test-time enforcement
that these two lists agree.

**Recommendation:** Provide a `known_slugs()` function from repo-tools that repo-meta
can call, or add an integration test that asserts the validation slug list exactly matches
the builtin registrations.

---

## Finding 13 — `BackupManager` flattens directory structure in backup filenames

**File:** `crates/repo-core/src/backup/tool_backup.rs`
**Severity:** Medium

```rust
fn backup_filename(path: &str) -> String {
    path.replace(['/', '\\', '.'], "_")  // .cursor/settings.json -> _cursor_settings_json
}
```

Nested tool config files (`.zed/settings.json`, `.vscode/settings.json`) lose their
directory component in the backup filename. If two different tools have config files
with the same basename in different directories (e.g., `tool-a/config.json` and
`tool-b/config.json`), their backup filenames could collide and one would silently
overwrite the other.

**Recommendation:** Use a more collision-resistant scheme, such as hashing the full
path or preserving the directory separator as a different delimiter (`__`).

---

## Finding 14 — Context detection in `repo-cli/src/context.rs` uses raw `fs::read_to_string`

**File:** `crates/repo-cli/src/context.rs`
**Severity:** Low

```rust
if let Ok(content) = std::fs::read_to_string(&config_path) {
    let mode = parse_mode(&content);
    // ...
}
```

The context detection walks up the directory tree reading raw files with the standard
library, bypassing the `repo-fs` abstractions (atomic reads, robustness config, locked
reads). This is inconsistent with the rest of the codebase. For context detection this
is probably acceptable (reads only, no writes), but it means the robustness guarantees
of `repo-fs` do not apply here.

**Recommendation:** Document in a comment that raw reads are intentional here (for
simplicity and to avoid a dependency cycle), or refactor to use `repo-fs::io::read_text`.

---

## Finding 15 — `Format::from_content` heuristics are ambiguous

**File:** `crates/repo-content/src/format.rs`
**Severity:** Low

```rust
pub fn from_content(content: &str) -> Self {
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Self::Json;
    }
    if trimmed.contains("\n[") || trimmed.starts_with('[') {
        if trimmed.lines().any(|l| l.contains(" = ")) {
            return Self::Toml;
        }
    }
    // ...
}
```

JSON arrays (`["a", "b"]`) and TOML section headers (`[section]`) both start with `[`.
The heuristic disambiguates by checking for ` = ` lines, but a JSON object with a key
`"a = b"` would match the TOML branch. The YAML heuristic (`key: value` on line 1)
also matches many other text formats with colons.

**Recommendation:** The heuristic is documented nowhere and the failure mode (wrong
format detection) could silently corrupt documents. Add a comment explaining the known
ambiguities and consider requiring explicit format specification from callers when the
content is ambiguous.

---

## Finding 16 — `repo-agent` crate is empty

**Directory:** `crates/repo-agent/src/`
**Severity:** Low

The crate exists in the workspace but contains no source files (no `lib.rs`, no modules).
It compiles as an empty crate with no public API.

**Recommendation:** Either add a `// TODO:` stub `lib.rs` explaining the planned
purpose, or remove the crate from the workspace `members` list until it has content.
An empty crate in the workspace wastes compile time and creates a misleading impression
of implemented functionality.

---

## Finding 17 — Two parallel block-marker systems in `repo-blocks`

**File:** `crates/repo-blocks/src/lib.rs` (documented in module comment)
**Severity:** Low (documented but worth flagging)

The `repo-blocks` crate intentionally implements two different block-marker systems:

1. `parser` + `writer`: Uses short alphanumeric UUIDs (`abc-123`), HTML comment syntax
2. `formats`: Uses full UUID-v4 values, format-specific marker syntax

These are used at different architectural layers (tool integration vs. content document
management) and the module comment explains this clearly. However, having two systems
in one crate with similar-sounding types (`Block` vs `ManagedBlock`, `has_block()` vs
`FormatHandler::find_blocks()`) increases cognitive load and risk of accidental mixing.

**Recommendation:** The documentation in `lib.rs` is good. Consider strengthening
isolation by moving one system to a sub-module with a clear `#[doc(hidden)]` or
internal visibility, or splitting them into `repo-blocks-tools` and `repo-blocks-content`.
At minimum, ensure the integration tests verify that the two systems do not interfere.

---

## Finding 18 — Plugin version hardcoded as default in CLI struct

**File:** `crates/repo-cli/src/cli.rs`
**Severity:** Low

```rust
/// Install a plugin
Install {
    #[arg(long, default_value = "v4.1.1")]
    version: String,
},
```

The plugin version `"v4.1.1"` is hardcoded as a `default_value` in the CLI struct.
This will become stale as versions advance, and users relying on the default will
silently get an old version.

**Recommendation:** Source this from a `const` defined in `repo-presets/src/plugins/paths.rs`
(where `DEFAULT_VERSION` already exists), or omit the default and require explicit
version specification.

---

## Positive Observations

The following practices are consistently applied across the workspace and represent
genuine strengths:

1. **No `unwrap()` in production code.** A grep across all `src/**/*.rs` files confirmed
   zero `unwrap()` calls in non-test code. All error paths use `?` propagation or
   explicit `match`/`if let` handling.

2. **Consistent use of `thiserror`.** Every crate defines its own typed `Error` enum
   with `#[from]` derivations. Cross-crate errors are wrapped, not strung.

3. **Atomic file writes.** `repo-fs::io::write_atomic()` uses temp-file-rename with
   file locking, `fsync` support, and exponential backoff. This prevents partial writes
   from leaving files in a corrupt state.

4. **Symlink protection.** `io::write_atomic()` calls `contains_symlink()` on the
   target path, preventing symlink-based path traversal attacks on config file writes.

5. **`Document` facade pattern.** `repo-content` wraps all format backends behind a
   single `Document` type with a consistent API. Format handlers are cleanly injected
   and the `Box<dyn Any>` design rationale is documented in the trait definition.

6. **Schema-driven tool integrations.** The `GenericToolIntegration` + `ToolDefinition`
   design means adding a new tool requires only a factory function with a struct
   literal—no custom sync logic. The 12 non-VSCode tools demonstrate this cleanly.

7. **`NormalizedPath` cross-platform path handling.** The type maintains forward-slash
   internal representation with a fast-path for already-clean paths, avoiding redundant
   cleaning operations.

8. **Rich CLI with thorough test coverage.** `cli.rs` contains exhaustive parse tests
   for every command variant and every flag combination. `main.rs` tests exercise the
   full command pipeline against temp-dir repositories.

9. **MCP protocol types are well-typed.** `repo-mcp/src/protocol.rs` defines strongly-
   typed request/response structs for each JSON-RPC method rather than using raw
   `serde_json::Value` throughout.

10. **Drift detection via SHA-256.** The ledger/projection system uses content hashing
    to detect when managed files have been externally modified. The `has_drifted()`
    method on both `Rule` and `ManagedBlock` provides a consistent interface.

---

## Priority Summary

| # | Finding | Severity |
|---|---------|----------|
| 1 | Silent save-error discard in `remove_rule` | High |
| 4 | Silent IO error swallowing in MCP handlers | High |
| 2 | `ToolRegistry` naming collision | Medium |
| 3 | Checksum format inconsistency (`sha256:` vs bare hex) | Medium |
| 5 | `VSCodeIntegration` architectural inconsistency | Medium |
| 9 | `sync_yaml` full replacement (no content preservation) | Medium |
| 10 | `CheckReport` naming collision | Medium |
| 11 | Error type conversion loses type information | Medium |
| 12 | Hardcoded tool list must be manually synchronized | Medium |
| 13 | `BackupManager` filename collision risk | Medium |
| 6 | `CapabilityTranslator` should be module functions | Low |
| 7 | `name()` returns slug, requires Clippy suppression | Low |
| 8 | Deprecated `new()` lacks `#[deprecated]` attribute | Low |
| 14 | `context.rs` bypasses `repo-fs` abstractions | Low |
| 15 | `Format::from_content` heuristics undocumented/ambiguous | Low |
| 16 | `repo-agent` crate is empty | Low |
| 17 | Two parallel block-marker systems in one crate | Low |
| 18 | Plugin version hardcoded in CLI struct | Low |
