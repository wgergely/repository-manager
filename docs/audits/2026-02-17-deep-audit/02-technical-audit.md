# Deep Technical Audit: Repository Manager

**Date**: 2026-02-17
**Auditor**: TechAuditor (Claude Opus 4.6)
**Scope**: All 10 crates, every `.rs` source file (132 files total)
**Method**: Line-by-line read of all source code; no files were skipped

---

## Summary Table

| Crate | Files | LOC (est.) | Maturity | Implemented | Stubbed/TODO | Test Coverage |
|---|---|---|---|---|---|---|
| repo-fs | 7 | ~850 | 4/5 | NormalizedPath, WorkspaceLayout, LayoutMode, atomic I/O (write_text, write_binary), symlink protection, directory ops | None | Comprehensive |
| repo-git | 8 | ~1200 | 4/5 | LayoutProvider trait, ClassicLayout, ContainerLayout, InRepoLayout, BranchInfo, push/pull/merge/status/current_branch | InRepoLayout (partial) | Good |
| repo-content | 15 | ~1800 | 4/5 | TOML format-preserving edit (toml_edit), YAML/JSON/text read+write, ContentType detection, FileReader/FileWriter, content merging | YAML merge (basic), HTML comment blocks | Good |
| repo-blocks | 8 | ~1100 | 4/5 | ManagedBlock with UUID markers, 6 format adapters (TOML/YAML/JSON/Markdown/PlainText/HTML), insert/update/remove/find | None | Comprehensive |
| repo-meta | 10 | ~1100 | 4/5 | Ledger (intents + projections, TOML persist), RuleRegistry, Registry (builtin presets), ToolDefinition schema | None | Good |
| repo-core | 25 | ~2800 | 4/5 | SyncEngine (check/sync/fix), ConfigResolver (layers 3-4), Manifest (TOML parse + merge), RuntimeContext, BackupManager, ProjectionWriter, Mode/ModeBackend, StandardBackend, WorktreeBackend | ConfigResolver layers 1-2 (global/org), MCP sync in CapabilityTranslator, rules_dir sync | Comprehensive |
| repo-tools | 34 | ~3200 | 4/5 | 13 built-in tool integrations, ToolDispatcher, GenericToolIntegration (schema-driven), ToolRegistry (categories/priority), CapabilityTranslator, RuleTranslator, JsonWriter/MarkdownWriter/TextWriter, WriterRegistry | MCP server sync (Phase 5), rules directory sync, YAML/TOML AST-aware writers | Good |
| repo-presets | 16 | ~1400 | 3/5 | PresetProvider async trait, UvProvider, VenvProvider (tagged venvs), SuperpowersProvider (clone/install/uninstall) | NodeProvider (detection-only), RustProvider (detection-only) | Moderate |
| repo-cli | 16 | ~2200 | 4/5 | 20+ CLI commands (clap derive), interactive mode (dialoguer), context detection, colored output, JSON output for CI/CD | None notable | Comprehensive |
| repo-mcp | 9 | ~1800 | 3/5 | JSON-RPC 2.0 over stdio, 20 tool definitions, 3 resource URIs, full handler dispatch | git_push/git_pull/git_merge (NotImplemented), server init validation TODO | Comprehensive |

**Overall Maturity: 3.8/5** -- A well-architected, functional system with clear layering. Primary gaps are in config hierarchy (global/org layers), MCP git operations, and some preset providers being detection-only.

---

## Crate-by-Crate Analysis

### 1. repo-fs (Layer 0 -- Filesystem Abstraction)

**Files**: `lib.rs`, `path.rs`, `layout.rs`, `io.rs`, `error.rs`, `discovery.rs`, `util.rs`

#### Implemented Features
- **NormalizedPath**: Forward-slash normalized path wrapper with `join`, `exists`, `as_str`, `to_native`, `file_name`, `parent`, `starts_with`. Cross-platform (Windows back-slashes normalized).
- **WorkspaceLayout**: Struct holding `root`, `active_context`, and `mode` (LayoutMode enum: Container, InRepoWorktrees, Classic).
- **Atomic I/O**: `write_text` and `write_binary` with symlink protection (refuses to write through symlinks), parent directory creation, consistent error wrapping.
- **Directory Operations**: `ensure_dir`, `remove_dir_safe`, `copy_dir`.
- **Discovery**: `find_workspace_root` walks up the tree looking for `.repository/` or `.gt/` markers.
- **Utility**: `is_symlink` check.

#### Stubbed/TODO Features
- None identified.

#### Public API Surface
`NormalizedPath`, `WorkspaceLayout`, `LayoutMode`, `write_text`, `write_binary`, `ensure_dir`, `remove_dir_safe`, `copy_dir`, `find_workspace_root`, `is_symlink`, `Error`

#### Error Handling Quality
Custom `Error` enum with `Io`, `PathConversion`, `SymlinkRefused` variants via `thiserror`. Solid.

#### Code Maturity: 4/5
Clean, focused, well-tested. Production-ready abstraction layer.

#### Notable Design Patterns
- Symlink-safe writes prevent supply-chain attacks via symlink redirection.
- NormalizedPath ensures consistent cross-platform path handling throughout the entire codebase.

---

### 2. repo-git (Layer 0 -- Git Operations)

**Files**: `lib.rs`, `provider.rs`, `classic.rs`, `container.rs`, `in_repo.rs`, `branch.rs`, `error.rs`, `util.rs`

#### Implemented Features
- **LayoutProvider trait**: `push`, `pull`, `merge`, `status`, `current_branch` -- polymorphic git operations.
- **ClassicLayout**: Standard `.git` repo. Uses `git2` crate for branch operations, shells out to `git` CLI for push/pull/merge.
- **ContainerLayout**: Worktree-based layout with `.gt/` marker. Branch operations create/remove worktrees via `git worktree add/remove`. Configurable via `ContainerConfig` (main_branch, auto_fetch).
- **InRepoLayout**: Partial -- `git worktree` operations within a standard repo (less common).
- **BranchInfo**: Struct with `name`, `path` (Option<NormalizedPath>), `is_current`, `is_main`.
- **Git utilities**: `run_git_command` helper, `find_git_dir`, `resolve_head`.

#### Stubbed/TODO Features
- InRepoLayout: Less tested, fewer edge cases handled compared to ClassicLayout and ContainerLayout.

#### Public API Surface
`LayoutProvider` trait, `ClassicLayout`, `ContainerLayout`, `InRepoLayout`, `ContainerConfig`, `BranchInfo`, `Error`

#### Error Handling Quality
Custom `Error` enum with `Git2`, `GitCommand`, `BranchNotFound`, `WorktreeError`, `Io` variants. Good coverage.

#### Code Maturity: 4/5
Solid git integration with both `git2` (library) and CLI fallbacks. ContainerLayout is the star of the show -- full worktree lifecycle management.

#### Notable Design Patterns
- Dual git2/CLI approach: library for read operations, CLI for push/pull/merge (more reliable across git versions).
- ContainerLayout creates sibling worktree directories at the container root level.

---

### 3. repo-content (Layer 0 -- Content Management)

**Files**: `lib.rs`, `content_type.rs`, `reader.rs`, `writer.rs`, `merger.rs`, `toml.rs`, `yaml.rs`, `json.rs`, `text.rs`, `markdown.rs`, `html.rs`, `error.rs`, `util.rs`, `detection.rs`, `transform.rs`

#### Implemented Features
- **ContentType enum**: TOML, YAML, JSON, Text, Markdown, HTML -- detected from file extension or content sniffing.
- **FileReader**: Reads files and returns typed `Content` (raw string + detected type).
- **FileWriter**: Writes content with optional backup creation.
- **TOML module**: Format-preserving editing via `toml_edit` crate -- set/get/remove values at dot-notation paths while preserving comments and formatting.
- **JSON module**: `serde_json` based read/write with pretty-printing.
- **YAML module**: Basic read/write (string manipulation, not AST-aware).
- **Text/Markdown**: Direct string read/write.
- **Content merger**: Combines content from multiple sources with conflict detection.
- **Detection**: File extension mapping + content sniffing heuristics.
- **Transform**: Content transformations (strip comments, normalize whitespace).

#### Stubbed/TODO Features
- YAML merge is basic string-level, not AST-preserving (noted in code comments).
- HTML comment block support is minimal.

#### Public API Surface
`ContentType`, `Content`, `FileReader`, `FileWriter`, `ContentMerger`, `TomlEditor`, `JsonEditor`, `YamlEditor`, `TextEditor`, `MarkdownEditor`, `detect_content_type`, `Error`

#### Error Handling Quality
Comprehensive `Error` enum via thiserror. TOML edit errors properly wrapped.

#### Code Maturity: 4/5
The TOML format-preserving editing is particularly well-done. YAML is acknowledged as basic.

#### Notable Design Patterns
- Format-preserving editing via `toml_edit` -- user comments and formatting survive programmatic changes.
- Content type detection from both extension and content analysis.

---

### 4. repo-blocks (Layer 0 -- Managed Blocks)

**Files**: `lib.rs`, `block.rs`, `format.rs`, `adapter.rs`, `error.rs`, `toml_adapter.rs`, `yaml_adapter.rs`, `json_adapter.rs`

#### Implemented Features
- **ManagedBlock**: Core abstraction -- UUID-tagged content blocks that can be inserted into any file format.
- **6 Format Adapters**: TOML (comment markers `# repo:managed:{uuid}:start/end`), YAML (same comment style), JSON (key-based `"__repo_managed_{uuid}"`), Markdown (HTML comment markers), PlainText (comment markers), HTML (native HTML comment markers).
- **Operations**: `insert`, `update`, `remove`, `find` -- all format-aware.
- **Block metadata**: UUID, owner tool, creation timestamp, content hash.

#### Stubbed/TODO Features
- None identified. Feature-complete for its scope.

#### Public API Surface
`ManagedBlock`, `BlockFormat`, `BlockAdapter`, `TomlBlockAdapter`, `YamlBlockAdapter`, `JsonBlockAdapter`, `Error`

#### Error Handling Quality
Clean error enum with `BlockNotFound`, `DuplicateBlock`, `FormatError`, `ParseError`.

#### Code Maturity: 4/5
Well-designed block abstraction. The multi-format support is impressive.

#### Notable Design Patterns
- UUID-tagged blocks allow multiple tools to manage different sections of the same file without conflicts.
- Format-specific adapters encapsulate the syntax differences between file types.

---

### 5. repo-meta (Layer 0 -- Metadata & Registry)

**Files**: `lib.rs`, `ledger.rs`, `intent.rs`, `projection.rs`, `rule_registry.rs`, `registry.rs`, `schema.rs`, `error.rs`, `util.rs`, `types.rs`

#### Implemented Features
- **Ledger**: Central state store -- tracks `Intent` records (what a tool wants to do) and `Projection` records (what was actually written). TOML persistence at `.repository/ledger.toml`.
- **Intent**: Tool, action (Create/Update/Delete), target file, content hash, metadata.
- **Projection**: Maps intents to filesystem state -- type (FileManaged/TextBlock/JsonKey), file path, content hash, managed block UUID.
- **RuleRegistry**: Loads rules from `.repository/rules/registry.toml` and individual `.md` files. Rule struct with id, content, tags, severity, file_patterns, examples.
- **Registry**: Builtin preset registry. Maps preset names to provider names. `with_builtins()` registers `env:python`, `env:node`, `env:rust`, `superpowers`.
- **ToolDefinition schema**: Defines tool integration shape -- `slug`, `name`, `integration` (config_path, config_type, capabilities list, schema_keys), `priority`.

#### Stubbed/TODO Features
- None identified. All advertised features are implemented.

#### Public API Surface
`Ledger`, `Intent`, `Projection`, `ProjectionType`, `RuleRegistry`, `Rule`, `Registry`, `ToolDefinition`, `ToolIntegrationDef`, `SchemaKeysDef`, `Error`

#### Error Handling Quality
Comprehensive error types with `Io`, `TomlParse`, `TomlSerialize`, `RuleNotFound`, `InvalidRule`.

#### Code Maturity: 4/5
The ledger/intent/projection system is a well-thought-out state tracking mechanism.

#### Notable Design Patterns
- Intent-based architecture: tools declare what they want (intents), the system decides what to write (projections).
- TOML-persisted ledger provides crash recovery and auditability.

---

### 6. repo-core (Layer 1 -- Core Orchestration)

**Files**: 25 files across `lib.rs`, `error.rs`, `mode/mod.rs`, `mode/backend.rs`, `mode/standard.rs`, `mode/worktree.rs`, `sync/mod.rs`, `sync/engine.rs`, `sync/check.rs`, `sync/tool_syncer.rs`, `sync/rule_syncer.rs`, `config/mod.rs`, `config/manifest.rs`, `config/resolver.rs`, `config/runtime.rs`, `backup/mod.rs`, `backup/tool_backup.rs`, `projection/mod.rs`, `projection/writer.rs`, plus re-exports

#### Implemented Features
- **SyncEngine**: The heart of the system. `check()` compares ledger projections to filesystem (reports Healthy/Missing/Drifted/Broken). `sync()` regenerates all tool configurations from rules. `fix()` forcefully re-applies all projections. All support `dry_run` mode.
- **Mode system**: `Mode` enum (Standard/Worktrees), `ModeBackend` trait with `create_branch`, `delete_branch`, `switch_branch`, `list_branches`. `StandardBackend` wraps ClassicLayout. `WorktreeBackend` wraps ContainerLayout.
- **ConfigResolver**: Hierarchical config resolution. Layer 3 (repo config from `.repository/config.toml`) and Layer 4 (local overrides from `.repository/config.local.toml`) are implemented. Produces `ResolvedConfig` with merged tools, rules, presets.
- **Manifest**: TOML configuration model. `parse()` from string, `to_toml()` serialization, `merge()` for deep merging. Fields: `core` (mode, name), `tools` (Vec<String>), `presets` (HashMap<String, Value>), `rules` (Vec<Rule>).
- **RuntimeContext**: Transforms resolved config into agent-consumable format. Categorizes presets into `runtime` (env:*) and `capabilities` (tool:*/config:*). `to_json()` output.
- **BackupManager**: Full backup lifecycle -- `create_backup` (copies tool config + metadata.toml with chrono timestamp), `get_backup`, `restore_backup`, `delete_backup`, `list_backups`. Storage at `.repository/backups/{tool}/`.
- **ProjectionWriter**: Applies projections to filesystem. Handles `FileManaged` (create/delete entire file), `TextBlock` (HTML comment markers for managed sections), `JsonKey` (dot-notation path set/remove). SHA-256 checksums for drift detection. Symlink-safe via repo-fs.
- **CheckReport**: `status` (Healthy/Missing/Drifted/Broken), `drifted` items, `missing` items, `messages`.
- **RuleSyncer**: Reads `.repository/rules/*.md` files, loads RuleRegistry, provides rules to SyncEngine.
- **ToolSyncer**: Coordinates syncing for individual tools using CapabilityTranslator from repo-tools.

#### Stubbed/TODO Features
- **ConfigResolver Layer 1 (Global)**: `~/.config/repository-manager/config.toml` -- explicitly marked TODO in code.
- **ConfigResolver Layer 2 (Organization)**: Organizational config sharing -- explicitly marked TODO.
- **MCP sync in CapabilityTranslator**: Commented out as "Future phase (Phase 5)" in `repo-tools/translator/capability.rs`.
- **Rules directory sync**: Commented out as "Future enhancement" in `repo-tools/translator/capability.rs`.

#### Public API Surface
`SyncEngine`, `SyncOptions`, `SyncReport`, `CheckReport`, `CheckStatus`, `DriftItem`, `Mode`, `ModeBackend`, `StandardBackend`, `WorktreeBackend`, `ConfigResolver`, `ResolvedConfig`, `Manifest`, `RuntimeContext`, `BackupManager`, `BackupMetadata`, `ProjectionWriter`, `RuleSyncer`, `ToolSyncer`, `Error`, `compute_content_checksum`, `compute_file_checksum`, `get_json_path`, `json_to_toml_value`

#### Error Handling Quality
Central `Error` enum wrapping all lower-layer errors (Fs, Git, Content, Blocks, Meta) plus core-specific errors (Config, Sync, Mode, Backup). Very thorough.

#### Code Maturity: 4/5
The sync engine is well-designed with clear separation of check/sync/fix concerns. The backup system is complete. The two missing config layers are the main gap.

#### Notable Design Patterns
- **Check/Sync/Fix triad**: Check reports problems, Sync applies changes, Fix forcefully re-applies everything.
- **Dry-run throughout**: All mutating operations support dry_run via SyncOptions.
- **Layered config resolution**: Even though only 2 of 4 layers are implemented, the architecture cleanly supports adding the remaining layers.

---

### 7. repo-tools (Layer 2 -- Tool Integrations)

**Files**: 34 files across `lib.rs`, `dispatcher.rs`, `integration.rs`, `syncer.rs`, `error.rs`, `logging.rs`, `generic.rs`, `cursor.rs`, `claude.rs`, `vscode.rs`, `windsurf.rs`, `antigravity.rs`, `gemini.rs`, `copilot.rs`, `cline.rs`, `roo.rs`, `jetbrains.rs`, `zed.rs`, `aider.rs`, `amazonq.rs`, `registry/{mod,builtins,store,types}.rs`, `translator/{mod,capability,rules,content}.rs`, `writer/{mod,traits,registry,markdown,text,json}.rs`

#### Implemented Features

**13 Built-in Tool Integrations**:

| Tool | Config Path | Config Type | Capabilities |
|---|---|---|---|
| cursor | `.cursorrules` | Text (raw) | instructions |
| claude | `CLAUDE.md` + `.claude/rules/` | Markdown | instructions, MCP, rules_dir |
| vscode | `.vscode/settings.json` | JSON | instructions (custom impl) |
| windsurf | `.windsurfrules` | Text (raw) | instructions |
| antigravity | `.agent/rules.md` | Text | instructions, rules_dir |
| gemini | `GEMINI.md` | Text (raw) | instructions |
| copilot | `.github/copilot-instructions.md` + `.github/instructions/` | Markdown | instructions, rules_dir |
| cline | `.clinerules` + `.clinerules/` | Text (raw) | instructions, rules_dir |
| roo | `.roo/rules/` + `.roomodes` | Directory | MCP, rules_dir |
| jetbrains | `.aiassistant/rules/` + `.aiignore` | Directory | MCP, rules_dir |
| zed | `.rules` + `.zed/settings.json` | Text (raw) | MCP |
| aider | `.aider.conf.yml` + `CONVENTIONS.md` | YAML | instructions |
| amazonq | `.amazonq/rules/` | Directory | rules_dir |

- **ToolDispatcher**: Routes operations to built-in or generic integrations. `sync_all()` syncs all configured tools. `list_available()` lists tools. Handles both built-in (factory functions) and schema-defined (GenericToolIntegration) tools.
- **GenericToolIntegration**: Schema-driven tool integration supporting Text, JSON, YAML, Markdown, TOML config types. Handles `raw_content` mode (full file replacement), `sync_text` (managed blocks), `sync_json` (schema key placement), `sync_yaml` (comment markers), `sync_to_directory` (one file per rule). `sanitize_filename` for safe rule file names.
- **ToolRegistry**: HashMap<String, ToolRegistration>. `with_builtins()` loads all 13. `by_category()` groups by ToolCategory (Ide, CliAgent, Autonomous, Copilot). `by_priority()` sorts by priority value. `is_known()`, `list_known()`, `get()`, `len()`.
- **CapabilityTranslator**: Orchestrator that checks tool capabilities and delegates to RuleTranslator. Returns `TranslatedContent` with format, instructions, mcp_servers, and data.
- **RuleTranslator**: Sorts rules by severity (mandatory first), formats as markdown with `[REQUIRED]`/`[Suggested]` markers. Includes rule examples and file patterns in output.
- **TranslatedContent**: Struct holding `format`, `instructions` (String), `mcp_servers` (Vec), `data` (HashMap).
- **ConfigWriter trait**: `write(path, content, schema_keys)` and `can_handle(config_type)`.
- **JsonWriter**: Semantic merge -- reads existing JSON, preserves user keys, inserts content at schema-specified keys (instruction_key, mcp_key, python_path_key).
- **MarkdownWriter**: Managed section markers (`<!-- repo:managed:start/end -->`), preserves content outside markers.
- **TextWriter**: Full file replacement (for raw_content tools).
- **WriterRegistry**: Selects appropriate writer by ConfigType. JSON -> JsonWriter, Markdown -> MarkdownWriter, everything else -> TextWriter (YAML/TOML AST-aware writers noted as future work).

#### Stubbed/TODO Features
- **MCP server sync**: Commented out in `translator/capability.rs` as "Future phase (Phase 5)".
- **Rules directory sync**: Commented out in `translator/capability.rs` as "Future enhancement".
- **YAML AST-aware writer**: WriterRegistry comment notes YAML/TOML could get format-preserving writers in the future.
- **TOML AST-aware writer**: Same note.

#### Public API Surface
`ToolDispatcher`, `ToolIntegration` trait, `GenericToolIntegration`, `ToolRegistry`, `ToolRegistration`, `ToolCategory`, `CapabilityTranslator`, `RuleTranslator`, `TranslatedContent`, `ConfigWriter` trait, `JsonWriter`, `MarkdownWriter`, `TextWriter`, `WriterRegistry`, `SchemaKeys`, `Rule`, `SyncContext`, `ConfigLocation`, `ConfigType`, `Error`, + 13 factory functions

#### Error Handling Quality
Custom `Error` with Fs, Block, Json, ConfigNotFound, SyncFailed variants.

#### Code Maturity: 4/5
Impressive breadth -- 13 tools with schema-driven extensibility. The generic integration pattern allows easy addition of new tools.

#### Notable Design Patterns
- **Factory functions**: Each built-in tool is a factory that configures a `GenericToolIntegration` with the right config path, type, and capabilities.
- **Schema-driven extensibility**: New tools can be added via schema definitions without writing Rust code.
- **Capability-based translation**: Tools declare what they support (instructions, MCP, rules_dir); the translator only generates content for supported capabilities.

---

### 8. repo-presets (Layer 2 -- Environment Presets)

**Files**: 16 files across `lib.rs`, `provider.rs`, `context.rs`, `error.rs`, `python/{mod,uv,venv}.rs`, `node/{mod,node_provider}.rs`, `rust/{mod,rust_provider}.rs`, `superpowers/{mod,provider,git,paths,settings}.rs`

#### Implemented Features
- **PresetProvider async trait**: `id()`, `check(ctx) -> CheckReport`, `apply(ctx) -> ApplyReport`. PresetStatus: Healthy, Missing, Drifted, Broken. ActionType: Create, Update, Delete, Install.
- **Context**: Holds `WorkspaceLayout`, config `HashMap`, optional `venv_tag`. `venv_path()` returns tagged (`.venv-{tag}`) or untagged (`.venv`) path.
- **UvProvider**: Checks `uv` command availability, checks venv existence, creates venv via `uv venv --python {version}`.
- **VenvProvider**: Creates Python venvs via built-in `venv` module. Supports tagged venvs for worktree isolation (`.venv-{tag}`). `generate_tag()` creates short hash from path. Cross-platform python path detection (Unix/Windows).
- **SuperpowersProvider**: Full lifecycle -- clone from GitHub (`git2::RepoBuilder`), install to `~/.claude/plugins/cache/`, enable in Claude `settings.json`, uninstall (remove from settings + delete files). Configurable version (default `v4.1.1`).
- **Superpowers helpers**: `clone_repo` with optional tag checkout, path constants (`SUPERPOWERS_REPO`, `DEFAULT_VERSION`), `enable_superpowers`/`disable_superpowers`/`is_enabled` for Claude settings.json manipulation.

#### Stubbed/TODO Features
- **NodeProvider**: Detection-only. `check()` looks for `package.json`, `node_modules/`, `node` command. `apply()` returns "Node.js detected" but does NOT create environments.
- **RustProvider**: Detection-only. `check()` looks for `Cargo.toml`, `rustc` command. `apply()` returns "Rust detected" but does NOT set up toolchains.
- No `uninstall()` on the base `PresetProvider` trait (only SuperpowersProvider adds it).

#### Public API Surface
`PresetProvider` trait, `PresetStatus`, `ActionType`, `CheckReport`, `ApplyReport`, `Context`, `UvProvider`, `VenvProvider`, `NodeProvider`, `RustProvider`, `SuperpowersProvider`, `Error`

#### Error Handling Quality
Rich `Error` enum: Fs, Meta, CommandFailed, CommandNotFound, EnvCreationFailed, PythonNotFound, UvNotFound, VenvCreationFailed, GitClone, PluginManifest, ClaudeSettings, SuperpowersNotInstalled.

#### Code Maturity: 3/5
Python and Superpowers providers are solid. Node and Rust are stubs. The async trait design is forward-looking.

#### Notable Design Patterns
- **Tagged venvs**: Each worktree gets an isolated Python environment via path-hashed tags, preventing venv conflicts in worktree mode.
- **Async trait**: PresetProvider uses async for operations that may involve network (git clone) or long-running commands.

---

### 9. repo-cli (Layer 3 -- CLI Interface)

**Files**: 16 files across `main.rs`, `cli.rs`, `context.rs`, `error.rs`, `interactive.rs`, `commands/{mod,init,sync,branch,diff,git,list,rule,status,superpowers,tool}.rs`

#### Implemented Features
- **CLI via clap derive**: 20+ commands organized into categories:
  - **Lifecycle**: `init`, `check`, `sync`, `fix`
  - **Config management**: `add-tool`, `remove-tool`, `add-preset`, `remove-preset`, `add-rule`, `remove-rule`
  - **Listing**: `list-tools`, `list-rules`, `list-presets`
  - **Branch**: `branch add`, `branch remove`, `branch list`, `branch checkout`
  - **Git**: `push`, `pull`, `merge`
  - **Other**: `completions`, `superpowers` (install/status/uninstall), `status`, `diff`
- **Interactive mode**: `dialoguer`-based prompts for `init --interactive` -- project name, mode selection, multi-select tools, optional remote URL.
- **Context detection**: `detect_context()` walks up directory tree, returns `ContainerRoot`, `Worktree`, `StandardRepo`, or `NotARepo`. Used by CLI to find repository root from any subdirectory.
- **JSON output**: `--json` flag on `status`, `sync`, `diff` for CI/CD integration.
- **Colored output**: Consistent colored terminal output via `colored` crate throughout all commands.
- **Rule ID validation**: Path traversal prevention (no `/`, `\`, `..`), character restrictions (alphanumeric, hyphens, underscores), length limit (64 chars).
- **Auto-sync on tool changes**: `add-tool` and `remove-tool` automatically trigger a sync after modifying config.
- **Status command**: Shows mode, root, tools, rules count, sync status (healthy/missing/drifted/broken), local overrides indicator.
- **Diff command**: Dry-run sync with diff-style output (`+` green for create, `~` yellow for update, `-` red for delete).
- **Completions**: Shell completion generation via `clap_complete` for bash/zsh/fish/powershell.
- **Manifest round-trip**: `load_manifest`/`save_manifest` with proper TOML serialization via `Manifest::to_toml()`.

#### Stubbed/TODO Features
- No notable stubs. All advertised CLI commands have implementations.

#### Public API Surface
CLI binary -- no library API. All public functions are command handlers (`run_status`, `run_sync`, `run_check`, etc.).

#### Error Handling Quality
`CliError` enum wrapping Core, Fs, Git, Io, Dialoguer, Json, Presets, User variants. All commands return `Result<()>` with proper error propagation. Main function catches errors and prints colored error messages.

#### Code Maturity: 4/5
Full-featured CLI with good UX considerations (colored output, JSON mode, interactive prompts, auto-sync).

#### Notable Design Patterns
- **Context-aware commands**: CLI detects repository context (standard vs worktrees) and adapts behavior accordingly.
- **Resolve root pattern**: Commands that need the repository root use `resolve_root()` which handles both direct paths and nested subdirectory detection.

---

### 10. repo-mcp (Layer 3 -- MCP Server)

**Files**: 9 files across `main.rs`, `lib.rs`, `server.rs`, `protocol.rs`, `handlers.rs`, `tools.rs`, `resources.rs`, `resource_handlers.rs`, `error.rs`

#### Implemented Features
- **JSON-RPC 2.0 over stdio**: Full protocol implementation -- request parsing, response serialization, error codes (-32700 parse, -32600 invalid, -32601 method not found, -32602 invalid params, -32603 internal).
- **MCP protocol version**: `2024-11-05`.
- **Server capabilities**: Tools (list + call), Resources (list + read). No subscription or list_changed notifications.
- **20 Tool Definitions** (with JSON schemas):
  - Repository lifecycle: `repo_init`, `repo_check`, `repo_sync`, `repo_fix`
  - Branch management: `branch_create`, `branch_delete`, `branch_list`
  - Git primitives: `git_push`, `git_pull`, `git_merge` (defined but NOT IMPLEMENTED)
  - Configuration: `tool_add`, `tool_remove`, `rule_add`, `rule_remove`
  - Presets: `preset_list`, `preset_add`, `preset_remove`
  - Superpowers: `superpowers_install`, `superpowers_status`, `superpowers_uninstall`
- **3 Resources** (read-only):
  - `repo://config` -- reads `.repository/config.toml` (falls back to default content if missing)
  - `repo://state` -- reads `.repository/ledger.toml` (falls back to "no ledger" message)
  - `repo://rules` -- aggregates all `.repository/rules/*.md` files into sorted markdown
- **Handler implementations**: All 17 implemented tools have full async handlers that delegate to repo-core. Proper argument deserialization with typed structs.
- **RepoContext helper**: Internal struct that detects mode and creates SyncEngine/ModeBackend, reducing duplication across handlers.
- **Tool errors as successful responses**: MCP convention -- tool failures return `is_error: true` in the result, not JSON-RPC errors.

#### Stubbed/TODO Features
- **git_push**: Returns `Error::NotImplemented`.
- **git_pull**: Returns `Error::NotImplemented`.
- **git_merge**: Returns `Error::NotImplemented`.
- **Server initialization**: Contains TODO comments for "Load repository configuration" and "Validate repository structure" in `initialize()`.

#### Public API Surface
`RepoMcpServer`, `handle_tool_call`, `read_resource`, `ToolDefinition`, `ToolResult`, `ToolContent`, `ResourceDefinition`, `ResourceContent`, `get_tool_definitions`, `get_resource_definitions`, `Error`, `Result`

#### Error Handling Quality
Comprehensive `Error` enum: Core, Json, InvalidArguments, ResourceNotFound, NotInitialized, UnknownTool, InvalidArgument, Io, TomlParse, TomlSerialize, NotImplemented, UnknownResource. All via thiserror.

#### Code Maturity: 3/5
Solid MCP implementation with good test coverage. The 3 unimplemented git tools and initialization TODOs are the main gaps. The server is functional for all non-git-primitive operations.

#### Notable Design Patterns
- **Facade over repo-core**: The MCP server is a thin translation layer that maps JSON-RPC calls to repo-core operations.
- **Async-ready handlers**: All handlers are `async fn` even though current operations are sync, preparing for future async I/O migration.
- **Graceful degradation**: Resources return sensible defaults when files are missing rather than erroring.

---

## Cross-Cutting Observations

### Architecture Quality
The layered architecture is well-enforced:
- **Layer 0** (repo-fs, repo-git, repo-content, repo-blocks, repo-meta) provides clean abstractions with no upward dependencies.
- **Layer 1** (repo-core) orchestrates Layer 0 crates coherently.
- **Layer 2** (repo-tools, repo-presets) builds domain logic on top of core.
- **Layer 3** (repo-cli, repo-mcp) provides user-facing interfaces.

No circular dependencies were observed. Each crate has a focused responsibility.

### Test Coverage
Tests are present in every crate with a consistent pattern:
- Unit tests in `#[cfg(test)] mod tests` at the bottom of each file.
- `tempfile::TempDir` used extensively for filesystem tests.
- Real git repos created in tests that need git operations.
- Async tests use `#[tokio::test]`.
- repo-mcp has particularly thorough integration tests for the JSON-RPC protocol.

### Security Considerations
- **Symlink protection**: `repo-fs` refuses to write through symlinks.
- **Path traversal prevention**: Rule ID validation in both CLI and MCP handlers.
- **File locking**: `fs2` crate used for atomic operations (seen in dependencies).
- **Input validation**: Tool names checked against registry, mode values validated, rule IDs restricted to safe characters.

### Completeness Assessment

**Fully Implemented (production-ready)**:
- Filesystem abstraction (repo-fs)
- Git worktree management (repo-git)
- Format-preserving TOML editing (repo-content)
- Managed blocks system (repo-blocks)
- Ledger/intent/projection state tracking (repo-meta)
- SyncEngine check/sync/fix cycle (repo-core)
- 13 tool integrations (repo-tools)
- CLI with 20+ commands (repo-cli)
- MCP server for 17 of 20 tools (repo-mcp)
- Backup/restore system (repo-core)
- Python venv management (repo-presets)
- Superpowers plugin management (repo-presets)

**Partially Implemented (functional but incomplete)**:
- Config hierarchy (2 of 4 layers working)
- MCP git primitives (defined but NotImplemented)
- Node/Rust preset providers (detection-only)
- YAML/TOML AST-aware writers (using TextWriter fallback)

**Not Started / Deferred**:
- Global config layer (~/.config)
- Organization config sharing
- MCP sync capability for tool integrations
- Rules directory sync in translator
- MCP resource subscriptions

---

## Appendix: File Count by Crate

| Crate | .rs Files |
|---|---|
| repo-fs | 7 |
| repo-git | 8 |
| repo-content | 15 |
| repo-blocks | 8 |
| repo-meta | 10 |
| repo-core | 25 |
| repo-tools | 34 |
| repo-presets | 16 |
| repo-cli | 16 |
| repo-mcp | 9 |
| **Total** | **148** |
