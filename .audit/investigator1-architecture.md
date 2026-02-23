# Architecture & Dependency Audit Report
## Investigator 1 — Crate Structure, Dependencies, and Architecture

**Audit Date:** 2026-02-18
**Project:** repository-manager
**Workspace Root:** `Y:/code/repository-manager-worktrees/main/`

---

## Executive Summary

The repository-manager workspace is a well-structured Rust project with 11 library/binary crates and 1 integration test crate. The overall layering intent is sound — low-level infrastructure crates (repo-fs, repo-git) at the bottom, domain crates (repo-meta, repo-content, repo-blocks) in the middle, and orchestration/UI crates (repo-core, repo-cli, repo-mcp) at the top. However, several significant architectural issues undermine this structure:

1. **Duplicate block-marker systems:** `repo-blocks` and `repo-content` both implement managed block parsing/writing with different marker formats, and this is intentional but underdocumented and potentially confusing. The actual duplication concern is that `repo-blocks/formats` is nearly a subset of `repo-content/block.rs + handlers`.
2. **Non-workspace dependency versions:** `chrono` and `fs2` are declared with inline version strings in `repo-core`'s Cargo.toml rather than using workspace versions, creating potential version skew.
3. **`dirs` crate used only in `repo-presets` but not declared in workspace dependencies** — an inconsistency in dependency governance.
4. **`RepositoryConfig` is deprecated but still ships in `repo-meta`** — a dead-code smell that increases the public API surface unnecessarily.
5. **`repo-core::Mode` and `repo-meta::RepositoryMode` are duplicate enum types** representing the same concept, with different defaults (Mode defaults to Worktrees; RepositoryMode defaults to Standard).
6. **Manifest `core.mode` is a raw `String` instead of a typed enum**, losing compile-time safety at the most critical config boundary.
7. **`repo-agent` is an extremely thin crate** with no dependencies on other repo-* crates, raising the question of whether it needs to be a separate crate.
8. **Integration test crate imports `repo-mcp`** but only uses it as an indirect dependency — `repo-mcp` is not exercised in any of the integration test assertions.

---

## Workspace Structure Overview

```
repository-manager/
├── Cargo.toml                    (workspace root, edition 2024)
├── crates/
│   ├── repo-agent/               (binary helper, Python subprocess orchestration)
│   ├── repo-blocks/              (block parser/writer, two-system design)
│   ├── repo-cli/                 (bin: repo, CLI entry point)
│   ├── repo-content/             (document model, format-aware editing)
│   ├── repo-core/                (orchestration, sync engine, ledger)
│   ├── repo-fs/                  (filesystem abstractions, path normalization)
│   ├── repo-git/                 (git operations, worktree layouts)
│   ├── repo-meta/                (config types, schema, loader, registry)
│   ├── repo-mcp/                 (bin: repo-mcp, MCP server)
│   ├── repo-presets/             (environment preset providers)
│   └── repo-tools/               (tool integrations: cursor, vscode, claude, etc.)
└── tests/
    └── integration/              (end-to-end integration tests)
```

**Workspace uses Cargo edition 2024**, which requires Rust 1.85+. This is a recent choice that constrains the minimum toolchain but enables modern features (e.g., `let-chains` used in sync/engine.rs).

---

## Inter-crate Dependency Analysis

### Dependency Graph (text diagram)

```
                    repo-cli (bin)          repo-mcp (bin)
                   /   |   |   \  \           / |  |  \
                  /    |   |    \  \          /  |  |   \
        repo-agent  repo-core  repo-git  repo-agent  repo-core
                         |                           |
                    +----|----+-----+------+------+---+----+
                    |    |    |     |      |      |        |
                repo-fs repo-git repo-meta repo-tools repo-presets repo-content
                    ^        |         ^        |
                    |        +----repo-fs+-------+
                    |
              repo-blocks
                    |
                repo-fs

integration-tests --> repo-fs, repo-git, repo-core, repo-meta, repo-presets, repo-tools, repo-mcp
```

### Simplified Layers

```
Layer 2 (UI/API):    repo-cli, repo-mcp
                         |
Layer 1 (Orchestration): repo-core, repo-agent
                         |
Layer 0 (Domain):    repo-tools, repo-presets, repo-content, repo-meta, repo-blocks
                         |
Layer -1 (Infra):    repo-fs, repo-git
```

### Dependency Matrix

| Crate        | Depends on                                                                         |
|--------------|------------------------------------------------------------------------------------|
| repo-fs      | (no internal deps)                                                                 |
| repo-git     | repo-fs                                                                            |
| repo-blocks  | repo-fs                                                                            |
| repo-meta    | repo-fs                                                                            |
| repo-content | (no internal deps — pure domain logic)                                             |
| repo-tools   | repo-fs, repo-meta, repo-blocks                                                    |
| repo-presets | repo-fs, repo-meta                                                                 |
| repo-agent   | (no internal deps)                                                                 |
| repo-core    | repo-fs, repo-git, repo-meta, repo-tools, repo-presets, repo-content              |
| repo-cli     | repo-agent, repo-core, repo-fs, repo-git, repo-meta, repo-tools, repo-presets     |
| repo-mcp     | repo-agent, repo-core, repo-fs, repo-meta, repo-presets                           |

**No circular dependencies detected.** The layering is sound.

### Layering Violations

- **Minor:** `repo-core` imports `repo-content` for content-level operations, but the sync engine (tool_syncer.rs) doesn't directly use `repo-content` — it instead calls through `repo-tools`. This means `repo-content` is a declared dependency of `repo-core` but may only be used in a subset of modules. This is worth verifying with `cargo-machete`.
- **Minor:** `repo-cli` depends on `repo-git` directly (in addition to via `repo-core`). This bypasses the orchestration layer for some git operations and may create inconsistency.
- **Observation:** `repo-mcp` comment in its Cargo.toml says "repo-tools is not needed directly here; tool dispatch is handled via repo-core" — this is a documentation of a deliberate design, which is good.

---

## Per-crate Architecture Review

### repo-fs

**Purpose:** Filesystem abstraction layer. Provides path normalization (`NormalizedPath`), atomic I/O, file locking, and workspace layout detection.

**Public API:**
- `NormalizedPath` — cross-platform path wrapper with forward-slash normalization
- `WorkspaceLayout` / `LayoutMode` — detects Container/InRepoWorktrees/Classic layouts
- `RepoPath` — filesystem path constants enum
- `ConfigStore` — config file I/O
- `RobustnessConfig` — configurable atomic write behavior

**Dependencies:** serde, toml, serde_json, serde_yaml, fs2, dunce, thiserror, backoff, tracing

**Issues Found:**

1. **`read_text` and `write_text` in io.rs are marked as TODO placeholders** ("replace with ManagedBlockEditor"). These functions are being used in production code paths (e.g., via `tool_syncer.rs`) but are labeled as temporary. This is a documentation drift/risk.
2. **`NormalizedPath::clean` drops leading `..` components for relative paths** with a comment "sandbox behavior". This silently swallows path traversal attempts instead of erroring — acceptable for security but surprising and underdocumented in the function signature.
3. **`NormalizedPath::to_native` returns a `PathBuf` with forward-slash paths on Windows.** On Windows, `PathBuf::from("C:/foo/bar")` works fine in most contexts, but passing the result to external processes may behave unexpectedly. The struct is designed for this but the potential Windows edge case is not tested.
4. **`serde_yaml` is listed as a dependency** of `repo-fs` (for config format support), but YAML config is an edge case rarely used at the filesystem level. It pulls in a heavy dependency for a capability that could be feature-gated.

---

### repo-git

**Purpose:** Git abstraction supporting three layout styles: Container (`.gt/`), InRepoWorktrees (`.git` + `.worktrees/`), and Classic (`.git`).

**Public API:**
- `ClassicLayout`, `ContainerLayout`, `InRepoWorktreesLayout`
- `NamingStrategy`
- `LayoutProvider` trait + `WorktreeInfo`
- Helper functions: `create_worktree_with_branch`, `merge`, `pull`, `push`, etc.

**Dependencies:** repo-fs, git2, thiserror, tracing

**Issues Found:**

1. **`helpers.rs` exposes `merge`, `pull`, `push` as free functions.** These don't take a `LayoutProvider` — they operate directly on a `git2::Repository`. This means the layout abstraction leaks at the helper level.
2. **bench target declared (`git_benchmarks`) but benchmark file location not verified.** If the bench file is missing, `cargo build` will fail.
3. **`get_current_branch` is a helper function** exported in the public API but also duplicated in the `LayoutProvider` trait as `current_branch()`. Two entry points for the same concept.

---

### repo-blocks

**Purpose:** Block parsing and writing. Contains **two independent block-marker systems**:
1. `parser` + `writer`: HTML comment markers (`<!-- repo:block:UUID -->`) with short alphanumeric IDs — used by `repo-tools`
2. `formats` module: Format-specific markers with full UUID-v4 values — used by `repo-content`

**Public API:**
- `Block`, `parse_blocks`, `find_block`, `has_block` (system 1)
- `insert_block`, `remove_block`, `update_block`, `upsert_block` (system 1)
- `FormatHandler`, `JsonFormatHandler`, `ManagedBlock`, `TomlFormatHandler`, `YamlFormatHandler` (system 2)

**Dependencies:** repo-fs, uuid, regex, serde_json, toml, thiserror

**Issues Found:**

1. **The two-system design is explicitly documented in the module-level comment**, which is good. However, it raises a fundamental question: why do two systems with nearly identical purpose exist? System 1 uses `<!-- repo:block:abc-123 -->` for tool configs; System 2 uses format-specific markers for document-level management in `repo-content`. The distinction is real but subtle enough to cause confusion for new contributors.
2. **`repo-blocks` depends on `repo-fs`** but the `formats` module doesn't appear to need filesystem access — it's pure content manipulation. This suggests `formats` may belong in `repo-content` instead.
3. **`repo-content` re-implements its own `ManagedBlock` type** (`repo-content::block::ManagedBlock`) independently from `repo-blocks::formats::ManagedBlock`. These are distinct types with different fields and semantics, which is confusing given the same name.
4. **The `serde` feature is not used** in `repo-blocks` despite `uuid` and `serde_json` being dependencies — no serialization derive macros appear in the `parser.rs` or `writer.rs` files.

---

### repo-meta

**Purpose:** Metadata schema definitions, configuration types, definition loader for `.repository/` directory structure.

**Public API:**
- `DefinitionLoader`, `LoadResult`
- `ToolDefinition`, `RuleDefinition`, `PresetDefinition`
- `Registry` — maps preset IDs to provider names
- `RepositoryMode`, `CoreConfig`, `ActiveConfig`, `SyncConfig`, `SyncStrategy`
- `RepositoryConfig` — **deprecated**
- `ToolRegistry`, `PresetRegistry` — validation types

**Dependencies:** repo-fs, serde, toml, thiserror, tracing

**Issues Found:**

1. **`RepositoryConfig` is `#[deprecated]`** but still ships in the public API. The deprecation comment points to `repo_core::Manifest`. However, `repo-meta` is a lower-layer crate than `repo-core`, creating a problematic reference situation where a lower-layer crate's deprecation note references a higher-layer replacement that users of `repo-meta` alone may not have access to. The deprecated code should either be removed or isolated behind a feature flag.
2. **`RepositoryMode` in `repo-meta` and `Mode` in `repo-core` are duplicate concepts** with different defaults:
   - `repo_meta::RepositoryMode::default()` → `Standard`
   - `repo_core::Mode::default()` → `Worktrees`
   This inconsistency is a bug surface. A user reading the schema sees "Standard" as default; the core engine defaults to "Worktrees".
3. **`SyncConfig`/`SyncStrategy`** are defined in `repo-meta` but appear unused — there are no callers that check `SyncStrategy::OnCommit` or `SyncStrategy::Manual` and behave differently. These types may be aspirational future features shipped as dead public API.
4. **The schema types (`ToolDefinition`, `RuleDefinition`, `PresetDefinition`) in `repo-meta`** define a `ConfigType` enum, and `repo-tools::integration.rs` also defines a `ConfigType` enum. These appear to be different types for the same concept, creating a naming collision.

---

### repo-content

**Purpose:** Unified document model supporting multiple formats (TOML, JSON, YAML, Markdown, PlainText) with managed block insertion, semantic comparison, and path-based editing.

**Public API:**
- `Document` — main type with parse/edit/render/diff operations
- `Format`, `FormatHandler`, `CommentStyle`
- `ManagedBlock`, `BlockLocation`
- `SemanticDiff`, `SemanticChange`
- `Edit`, `EditKind`
- Format handlers: `TomlHandler`, `JsonHandler`, `YamlHandler`, `MarkdownHandler`, `PlainTextHandler`

**Dependencies:** serde, serde_json, serde_yaml, toml, toml_edit, similar, uuid, sha2, regex, thiserror

**Issues Found:**

1. **`Document::render_from_normalized` for TOML loses format preservation.** It converts JSON → TOML via a manual `json_to_toml` function, not via `toml_edit`. This means `set_path` and `remove_path` on TOML documents will reformat the file — defeating the purpose of `toml_edit` which is also a dependency. The `toml_edit` crate is imported but not used for round-trip editing.
2. **`repo-content` duplicates `compute_checksum` logic** using SHA-256, as does `repo-core::projection::writer` and `repo-core::sync::engine`. There are at least 3 independent SHA-256 hash helpers across the codebase.
3. **`repo-content::block::ManagedBlock` and `repo-blocks::formats::ManagedBlock`** share the same name but are distinct types. This is a naming collision that will confuse `use` statement imports.
4. **`Document::parse` uses auto-detection (`Format::from_content`) that falls back to PlainText**, meaning malformed TOML/JSON silently becomes PlainText instead of erroring. This may mask user errors.
5. **`path.rs` module** provides `get_at_path`, `set_at_path`, `remove_at_path`, `parse_path` — these are internal helpers but fully public (`pub mod path`). They should be `pub(crate)`.

---

### repo-tools

**Purpose:** Tool integration implementations for ~14 AI coding tools (cursor, vscode, claude, windsurf, gemini, aider, cline, copilot, roo, zed, jetbrains, amazonq, antigravity, gemini).

**Public API:**
- `ToolIntegration` trait
- `ToolDispatcher` — routes to built-in or generic integrations
- `ToolRegistry`, `ToolRegistration`, `builtin_registrations`
- Individual integration functions/types for all 14+ tools
- `GenericToolIntegration` — schema-driven fallback
- `CapabilityTranslator`, `RuleTranslator`, `TranslatedContent`
- Writer types: `ConfigWriter`, `JsonWriter`, `MarkdownWriter`, `TextWriter`, `WriterRegistry`
- `ToolCapabilitySyncer`

**Dependencies:** repo-fs, repo-meta, repo-blocks, serde, serde_json, thiserror, tracing, tracing-subscriber

**Issues Found:**

1. **`tracing-subscriber` is a dependency of `repo-tools`** — this is unusual for a library crate. `tracing-subscriber` is typically only needed in binary crates. Library crates should only depend on `tracing` (for emitting events), not the subscriber infrastructure. This likely appeared from early development and was never cleaned up.
2. **The `logging.rs` module in `repo-tools`** exports subscriber setup functions — further evidence that library-inappropriate code exists in this crate.
3. **`repo-tools::integration::ConfigType` and `repo-meta::schema::tool::ConfigType`** share the same name with similar but distinct semantics. The `repo-tools` version has `Rules`/`Settings`/`KeyValue` variants; the `repo-meta` version has `Markdown`/`Json`/`Toml`/`Text`/`KeyValue`. This naming collision is confusing.
4. **Stub implementations:** `aider`, `amazonq`, `cline`, `copilot`, `jetbrains`, `roo`, `zed` integration modules — it is not clear from reading lib.rs alone which of these are full implementations vs. stubs. The integration test only tests vscode/cursor/claude directly.
5. **`BUILTIN_COUNT` constant is exported as public API.** This is an internal implementation detail that leaks into the public surface.
6. **`repo-tools` does not use `repo-content`** — instead it relies on `repo-blocks` (system 1) directly for managed block operations. This means the richer `Document` model from `repo-content` is unavailable to tool integrations, which might limit future capabilities.

---

### repo-presets

**Purpose:** Environment preset providers for Python (uv/venv), Node, Rust. Each provider implements `PresetProvider` trait for `check` and `apply` operations.

**Public API:**
- `PresetProvider` trait
- `Context` — workspace context for providers
- `UvProvider`, `VenvProvider` (Python)
- `NodeProvider`
- `RustProvider`
- `PluginsProvider`
- `CheckReport`, `ApplyReport`, `PresetStatus`, `ActionType`

**Dependencies:** repo-fs, repo-meta, async-trait, dirs, git2, tokio, thiserror, serde_json, toml

**Issues Found:**

1. **`dirs` crate is not in workspace dependencies** — it's declared inline only in `repo-presets/Cargo.toml`. This breaks the workspace dependency governance model where all dependency versions should be centrally managed.
2. **`git2` is a dependency of `repo-presets`** — the reason is not obvious from the lib.rs. `repo-git` already provides git abstractions. Using `git2` directly in `repo-presets` bypasses the git abstraction layer, creating a layering violation.
3. **`tokio` is declared in both `[dependencies]` and `[dev-dependencies]`** with different features (`["rt", "process", "fs"]` vs `["rt-multi-thread", "macros"]`). The workspace version only declares `["rt", "process", "fs"]`. The dev-dependencies override is correct (tests need `rt-multi-thread` and `macros`), but the duplication of the version number `"1.42"` in dev-dependencies rather than using `workspace = true` is inconsistent.
4. **`PluginsProvider`** is exported but not well-documented. Its purpose relative to the other providers is unclear from the lib.rs comment alone.

---

### repo-agent

**Purpose:** Discovers and manages the vaultspec Python agent framework. Handles Python 3.13+ discovery, `.vaultspec/` directory validation, and health checking.

**Public API:**
- `AgentManager` — discovery and health checking
- `AgentError`, `Result`
- `AgentInfo`, `HealthReport`, `TaskStatus`

**Dependencies:** thiserror, serde, serde_json

**Issues Found:**

1. **`repo-agent` has no internal workspace dependencies** — it stands completely alone. This is fine in principle but raises the question of whether it needs to be a separate crate vs. a module within `repo-core` or `repo-cli`.
2. **The `subprocess` and `process` modules** are both exported from lib.rs but not re-exported at the top level. They are accessible via `repo_agent::subprocess` but are not in the `pub use` list. This means callers must use full paths which is inconsistent.
3. **`AgentManager` discovers Python 3.13+** — this is a very specific minimum version requirement that is not documented in the crate description beyond the lib.rs doc comment.

---

### repo-core

**Purpose:** Core orchestration layer. Implements SyncEngine (check/sync/fix), Ledger (intent+projection tracking), ConfigResolver, ModeBackend (Standard/Worktree), governance (lint/diff/export), and hooks.

**Public API:**
- `SyncEngine`, `SyncOptions`, `SyncReport`
- `CheckReport`, `CheckStatus`, `DriftItem`
- `Ledger`, `Intent`, `Projection`, `ProjectionKind`
- `Manifest`, `ConfigResolver`, `ResolvedConfig`, `RuntimeContext`
- `Mode` (enum)
- `ModeBackend` trait, `StandardBackend`, `WorktreeBackend`, `BranchInfo`
- `Rule`, `RuleRegistry`
- `ProjectionWriter`, `compute_checksum`
- `BackupManager`, `BackupMetadata`, `ToolBackup`
- `ConfigDrift`, `DriftType`, `LintWarning`, `WarnLevel`
- `HookConfig`, `HookContext`, `HookEvent`, `run_hooks`
- `RuleFile`, `RuleSyncer`

**Dependencies:** repo-fs, repo-git, repo-meta, repo-tools, repo-presets, repo-content (+ chrono, sha2, fs2, uuid, serde/serde_json/toml)

**Issues Found:**

1. **`chrono` is declared with an inline version `"0.4"` in repo-core's Cargo.toml** (`chrono = { version = "0.4", features = ["serde"] }`) instead of using `workspace = true`. The workspace Cargo.toml also declares `chrono = { version = "0.4", features = ["serde"] }`. This is a direct version duplication that bypasses workspace governance.
2. **`fs2` is declared inline in repo-core** (`fs2 = "0.4"`) instead of `fs2 = { workspace = true }`. The workspace already declares `fs2 = "0.4"`. Same issue as above.
3. **`repo-content` dependency may be partially unused.** The `sync/tool_syncer.rs` uses `repo-tools` for tool operations and `repo-fs::io::write_text` for file writing. The actual use of `repo-content` in `repo-core` needs verification — it's possible only a small surface is actually used.
4. **`compute_checksum` is defined in three places:**
   - `repo-core::sync::engine::compute_content_checksum`
   - `repo-core::sync::engine::compute_file_checksum`
   - `repo-core::projection::writer::compute_checksum` (re-exported at crate root)
   - `repo-content::block::ManagedBlock::compute_checksum` (private method)
   All implement SHA-256 hex encoding. This should be a single utility.
5. **`Manifest::core.mode` is a raw `String`** rather than `Mode` or `RepositoryMode`. This loses type safety at the most important config parsing boundary — `Mode::from_str` must be called manually by callers.
6. **`governance::diff_configs` reads the ledger file directly with `toml::from_str`** rather than using the `Ledger::load` method. This bypasses the file locking in `Ledger::load`, creating a TOCTOU race in the diff operation.
7. **`SyncEngine::sync_with_options` creates `ToolSyncer` and `RuleSyncer` on every call** — these are cheap to construct, but dispatchers and registries inside them are not pooled. For frequent sync calls this creates repeated initialization overhead.
8. **`get_json_path` in sync/engine.rs** is a utility function for JSON path traversal that duplicates functionality available through `repo-content::Document::get_path`. As `repo-content` is already a dependency, this is redundant code.
9. **`ToolSyncer` is a private module** (`sync::tool_syncer`) that is not exported from `repo-core`'s public API. However, `SyncEngine` uses it internally. This is appropriate encapsulation.

---

### repo-cli

**Purpose:** Command-line interface (`repo` binary). Implements all user-facing commands.

**Public API:** Binary only (no library).

**Dependencies:** repo-agent, repo-core, repo-fs, repo-git, repo-meta, repo-tools, repo-presets + clap, clap_complete, colored, dialoguer, git2, serde, serde_json, toml, tracing, tracing-subscriber, tokio

**Issues Found:**

1. **`repo-cli` depends on `repo-git` directly** as well as through `repo-core`. This bypasses the orchestration layer for some git operations, potentially leading to inconsistency between what the CLI does directly and what goes through `SyncEngine`.
2. **`git2` is a direct dependency of `repo-cli`** — again, bypassing the `repo-git` abstraction layer.
3. **`toml` is listed twice in dev-dependencies** (once as `toml.workspace = true` and once as `toml = { workspace = true }` under `[dev-dependencies]`). The `Cargo.toml` shows `toml.workspace = true` followed by `toml.workspace = true` — this is a duplicate dev-dependency declaration.
4. **`tokio` features `["rt", "macros"]`** are declared in `repo-cli`'s dependencies. The workspace declares `["rt", "process", "fs"]`. These don't overlap correctly — if `repo-cli` needs `macros` it should extend the workspace features, but instead it re-declares with different features. This may cause feature unification issues.
5. **`cmd_plugins` is the only command using async** (`rt.block_on`) while all others are synchronous. This inconsistency in the async boundary is notable and suggests plugins may have been added later with a different design.
6. **`interactive.rs` module** is defined but not exposed in the crate — it's a private module providing `interactive_init`. This is fine but could lead to dead code warnings if the interactive path is not well-tested.

---

### repo-mcp

**Purpose:** MCP (Model Context Protocol) server binary. Exposes repository management functionality via JSON-RPC over stdio.

**Public API:** Library (`RepoMcpServer`, `handle_tool_call`, `read_resource`, tool definitions) + Binary.

**Dependencies:** repo-agent, repo-core, repo-fs, repo-meta, repo-presets + tokio (full), serde, serde_json, toml, tracing, tracing-subscriber, clap, thiserror

**Issues Found:**

1. **`tokio = { workspace = true, features = ["full"] }`** — using `full` features of tokio in production is wasteful. The workspace declares only `["rt", "process", "fs"]`. The MCP server overrides this with `full`, which pulls in all tokio components including those not needed (e.g., `signal`, `sync`, `time`). This should be scoped to only the features actually used.
2. **`repo-mcp` does not depend on `repo-tools` or `repo-content`** — the comment in Cargo.toml says "tool dispatch is handled via repo-core". However, this means the MCP server cannot access tool-specific information (e.g., listing available tools with their metadata) without going through repo-core, limiting MCP tool description richness.
3. **`repo-mcp` is included in the integration test's dependencies** but the integration tests don't actually test MCP protocol behavior — there are no JSON-RPC request/response tests. The dependency seems unnecessary in the integration tests.

---

## Dependency Health

### Unused Workspace Dependencies (Declared but Potentially Underused)

| Dependency  | Status                                                                             |
|-------------|------------------------------------------------------------------------------------|
| `similar`   | Used only in `repo-content` for text diff — appropriate                            |
| `insta`     | Dev-only in repo-fs and repo-tools — snapshot testing, appropriate                 |
| `proptest`  | Dev-only in repo-fs — property testing, appropriate but only in one crate          |
| `criterion` | Dev-only in repo-fs and repo-git — benchmarking, appropriate                       |
| `backoff`   | Used only in repo-fs io.rs — appropriate for retry logic                           |
| `sha2`      | Used in repo-content and repo-core — appropriate but logic is duplicated           |
| `regex`     | Used in repo-blocks and repo-content — appropriate                                 |
| `chrono`    | Used in repo-core ledger (Intent timestamps) — but declared twice (workspace + inline) |

### Non-Workspace Inline Dependencies

These should be moved to workspace or removed:

| Crate/Dep    | File                          | Issue                                           |
|--------------|-------------------------------|-------------------------------------------------|
| `chrono`     | repo-core/Cargo.toml:24       | Inline `{ version = "0.4", features = ["serde"] }` — duplicates workspace |
| `fs2`        | repo-core/Cargo.toml:26       | Inline `"0.4"` — duplicates workspace            |
| `dirs`       | repo-presets/Cargo.toml:12    | Not in workspace at all — version unmanaged      |
| `tokio` (dev)| repo-presets/Cargo.toml:23    | Dev dep uses inline `"1.42"` instead of `workspace = true` |
| `tokio`      | repo-cli/Cargo.toml:32        | Features `["rt", "macros"]` different from workspace `["rt", "process", "fs"]` |
| `tokio`      | repo-mcp/Cargo.toml           | Features `["full"]` far exceeds workspace declaration |
| `regex`      | integration-tests/Cargo.toml  | Inline `"1"` not using workspace version         |

### Duplicate Code: SHA-256 Checksums

Three (arguably four) independent implementations of SHA-256 hex checksumming:
- `repo-core::sync::engine::compute_content_checksum(content: &str) -> String`
- `repo-core::sync::engine::compute_file_checksum(path: &Path) -> Result<String>`
- `repo-core::projection::writer::compute_checksum(content: &str) -> String` (re-exported)
- `repo-content::block::ManagedBlock::compute_checksum(content: &str) -> String` (private)

These should be consolidated into a single utility (e.g., in `repo-fs` or `repo-core`).

---

## Architectural Concerns

### 1. Mode Enum Duplication (High Priority)

`repo-core::Mode` and `repo-meta::RepositoryMode` represent the same concept with different defaults:
- `Mode::default()` = `Worktrees`
- `RepositoryMode::default()` = `Standard`

In `Manifest::core.mode` (repo-core), the field is stored as a raw `String`, not typed as `Mode`. This means the type system doesn't enforce valid mode values at parse time. Callers must manually call `Mode::from_str(&manifest.core.mode)` and handle errors. This is an unnecessary footgun at the most important configuration boundary.

**Recommendation:** Define `Mode` canonically in `repo-meta` (or `repo-fs`), use it in `Manifest`, and re-export it from `repo-core`. Remove `RepositoryMode`.

### 2. Two Block-Marker Systems (Medium Priority)

The two-system design in `repo-blocks` (HTML comment markers with short IDs for `repo-tools` vs. format-specific markers with UUID-v4 for `repo-content`) is documented but represents a significant conceptual split. The existence of two `ManagedBlock` types with the same name (one in `repo-blocks::formats` and one in `repo-content::block`) is a naming collision that will confuse contributors.

**Recommendation:** Rename `repo-blocks::formats::ManagedBlock` to something like `FormatManagedBlock` or `ContentBlock` to distinguish it from the `repo-content::block::ManagedBlock`. Alternatively, move system 2 entirely into `repo-content` and remove it from `repo-blocks`.

### 3. `governance::diff_configs` Bypasses File Locking (Medium Priority)

In `repo-core/src/governance.rs:165`, the ledger is loaded via direct `toml::from_str` on `fs::read_to_string` output, bypassing `Ledger::load` which acquires a shared file lock. This creates a potential race condition if sync and diff run concurrently.

**Recommendation:** Replace the inline ledger read in `diff_configs` with `Ledger::load()`.

### 4. `tracing-subscriber` in Library Crate (Medium Priority)

`repo-tools` depends on `tracing-subscriber`, which is a subscriber implementation crate. Library crates should only depend on `tracing` (the facade). Having `tracing-subscriber` in a library forces all consumers to link its implementation, which can conflict with their own subscriber setup.

**Recommendation:** Move the `logging.rs` module content to `repo-cli` and `repo-mcp` (the binaries). Remove `tracing-subscriber` from `repo-tools`' dependencies.

### 5. `RepositoryConfig` Deprecated But Not Removed (Low Priority)

`repo-meta::config::RepositoryConfig` is `#[deprecated]` with a migration note pointing to `repo_core::Manifest`. Since the deprecation note crosses crate boundaries (meta → core), users of `repo-meta` alone cannot use the recommended replacement without adding `repo-core` as a dependency. The deprecated type should be removed, not just flagged.

### 6. `toml_edit` Dependency Not Used for Round-Trip Editing (Medium Priority)

`repo-content` imports `toml_edit` for format-preserving TOML editing, but `Document::set_path` and `Document::remove_path` convert through JSON → `toml::Value` → string, losing format preservation. The `toml_edit` dependency is used only by the `TomlHandler` for block insertion, not for structured path editing. Either `set_path`/`remove_path` should be implemented with `toml_edit` for real format preservation, or the limitation should be prominently documented.

### 7. Integration Tests Import `repo-mcp` Without Testing It

The `integration-tests/Cargo.toml` lists `repo-mcp` as a dependency, but `integration_test.rs` doesn't import or use it. The `mission_tests.rs` may use it, but if not, this is unnecessary compilation overhead in CI.

---

## Recommendations (Prioritized)

### High Priority

1. **Fix `Manifest::core.mode` to use a typed enum.** Replace the raw `String` field with a proper enum type to catch invalid mode values at parse time. Define the enum in `repo-meta` and use it in `repo-core::Manifest`.

2. **Move non-workspace deps to workspace.** Move `chrono`, `fs2`, `dirs` (with proper version selection), and `regex` into `[workspace.dependencies]`. Fix `tokio` feature declarations across repo-cli, repo-mcp, and repo-presets to use workspace features correctly.

3. **Consolidate SHA-256 checksum helpers** into a single utility in `repo-core` or a new `repo-util` crate, and remove the three duplicates.

4. **Fix `governance::diff_configs`** to use `Ledger::load()` instead of manual file reading, to preserve the file-locking contract.

### Medium Priority

5. **Remove `tracing-subscriber` from `repo-tools`** (library crate). Move subscriber initialization to binary crates only.

6. **Rename `ManagedBlock` collision.** Rename either `repo-blocks::formats::ManagedBlock` or `repo-content::block::ManagedBlock` to eliminate the naming confusion.

7. **Remove or isolate `RepositoryConfig` deprecated type** from `repo-meta`. Since it has an impractical deprecation note (references a higher-layer crate), it should be deleted.

8. **Unify `Mode` / `RepositoryMode`.** One of these should be removed. Prefer defining in `repo-meta` (or `repo-fs`) so lower-layer crates can use it without the circular dependency.

9. **Implement `Document::set_path`/`remove_path` with `toml_edit`** for proper format preservation, or document the limitation prominently that these operations destroy TOML formatting.

10. **Verify integration test's `repo-mcp` dependency** is actually needed. If `mission_tests.rs` doesn't use it, remove it from the integration crate's dependencies.

### Low Priority

11. **Make `repo-content::path` module `pub(crate)`** instead of fully `pub`. These are internal utilities (`get_at_path`, `set_at_path`, etc.) that shouldn't be part of the public API.

12. **Evaluate whether `repo-agent` needs to be a separate crate** or could be a module within `repo-cli`. It has no internal crate dependencies and its functionality (Python process discovery) doesn't need to be shared across multiple crates currently.

13. **Fix `repo-cli` duplicate `toml` dev-dependency declaration** in its Cargo.toml.

14. **Clarify which `repo-tools` integrations are stubs vs. full implementations** (aider, amazonq, cline, copilot, jetbrains, roo, zed). Add `#[cfg(test)]` stub markers or documentation distinguishing these.

15. **Remove `BUILTIN_COUNT` from public API of `repo-tools`** — it's an internal implementation detail.

16. **Evaluate `repo-blocks` dependency on `repo-fs`** — the `formats` module (system 2) may not need filesystem access. If so, `repo-blocks::formats` could be split into a pure crate.

17. **Scope `tokio` features in `repo-mcp`** from `"full"` to only the features actually needed (likely `rt-multi-thread`, `macros`, `io-util`).
