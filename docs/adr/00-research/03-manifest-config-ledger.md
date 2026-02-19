# Research: Manifest, Config Resolution, and Ledger System

**Date:** 2026-02-19
**Researcher:** core-researcher (Opus agent)
**Source:** `repo-core` and `repo-meta` crates

---

## 1. Manifest Struct

**File:** `crates/repo-core/src/config/manifest.rs` (lines 37-63)

```rust
pub struct Manifest {
    pub core: CoreSection,              // [core] mode: String
    pub presets: HashMap<String, Value>, // [presets."type:name"] sections
    pub tools: Vec<String>,             // tools = ["cursor", "vscode"]
    pub rules: Vec<String>,             // rules = ["no-unsafe", "no-unwrap"]
    pub hooks: Vec<HookConfig>,         // [[hooks]] array of tables
}
```

Methods: `parse()`, `empty()`, `to_toml()`, `merge()`.

Merge strategy:
- `core.mode`: other always wins
- `presets`: Deep merge (recursive object merge)
- `tools`: Extend with unique values
- `rules`: Extend with unique values
- `hooks`: Append all from other

## 2. ConfigResolver

**File:** `crates/repo-core/src/config/resolver.rs`

### ResolvedConfig
```rust
pub struct ResolvedConfig {
    pub mode: String,
    pub presets: HashMap<String, Value>,
    pub tools: Vec<String>,
    pub rules: Vec<String>,
    // NOTE: hooks are dropped during resolution
}
```

### 4-Layer Resolution Hierarchy
1. Global defaults (`~/.config/repo-manager/config.toml`) - NOT YET IMPLEMENTED
2. Organization config - NOT YET IMPLEMENTED
3. Repository config (`.repository/config.toml`)
4. Local overrides (`.repository/config.local.toml`, git-ignored)

## 3. Ledger System

### Intent
**File:** `crates/repo-core/src/ledger/intent.rs` (lines 19-31)

```rust
pub struct Intent {
    pub id: String,                    // e.g., "rule:style" or "tool:cursor"
    pub uuid: Uuid,
    pub timestamp: DateTime<Utc>,
    pub args: Value,
    projections: Vec<Projection>,      // private
}
```

### Projection
**File:** `crates/repo-core/src/ledger/projection.rs` (lines 16-24)

```rust
pub struct Projection {
    pub tool: String,
    pub file: PathBuf,
    pub kind: ProjectionKind,
}
```

### ProjectionKind
```rust
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum ProjectionKind {
    TextBlock { marker: Uuid, checksum: String },
    JsonKey { path: String, value: Value },
    FileManaged { checksum: String },
}
```

### Ledger File Operations
- **Loading:** Shared file lock via `fs2`, TOML parse
- **Saving:** Exclusive lock, write to temp file, atomic rename
- **Queries:** `find_by_rule()`, `projections_for_file()`, `get_intent()`, `get_intent_mut()`

## 4. SyncEngine

**File:** `crates/repo-core/src/sync/engine.rs`

```rust
pub struct SyncEngine {
    root: NormalizedPath,
    mode: Mode,
    backend: Box<dyn ModeBackend>,
}
```

### Sync Flow
```
sync_with_options():
  1. Load ledger (or create empty)
  2. Load config.toml -> Manifest
  3. ToolSyncer syncs each tool -> FileManaged projections
  4. RuleSyncer syncs rules to tool configs -> FileManaged projections
  5. Save ledger
```

### Check Flow
For each intent/projection, validates based on ProjectionKind:
- **FileManaged**: file exists + SHA-256 match
- **TextBlock**: file exists + marker found + block checksum match
- **JsonKey**: file exists + path navigable + value match

Returns: Healthy, Missing, Drifted, or Broken.

## 5. RuntimeContext

```rust
pub struct RuntimeContext {
    pub runtime: HashMap<String, Value>,   // env presets
    pub capabilities: Vec<String>,         // sorted capability strings
}
```

`env:*` presets -> `runtime` map. `tool:*` and `config:*` presets -> `capabilities` list.

## 6. Governance Functions

- `lint_rules()` - Checks empty tools, duplicates, unknown tools, no rules
- `diff_configs()` - Compares filesystem vs ledger (Modified/Missing/Extra)
- `export_agents_md()` - Generates AGENTS.md from rules
- `import_agents_md()` - Parses AGENTS.md back to rules

## 7. Hooks System

```rust
pub enum HookEvent {
    PreBranchCreate, PostBranchCreate,
    PreBranchDelete, PostBranchDelete,
    PreAgentComplete, PostAgentComplete,
    PreSync, PostSync,
}
```

```rust
pub struct HookConfig {
    pub event: HookEvent,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
}
```

Variable substitution: `${VAR_NAME}` in args. Fail-fast on non-zero exit.

## 8. Extension Points for [extensions] Section

To add `[extensions]` to config.toml:
1. Add `extensions: HashMap<String, Value>` to `Manifest`
2. Update `merge()`, `empty()`, `to_toml()`
3. Add to `ResolvedConfig`
4. Optionally expose in `RuntimeContext`
5. New `ExtensionSyncer` alongside `ToolSyncer`/`RuleSyncer`
6. New `ExtensionDefinition` schema in `repo-meta`
7. Intent IDs with `"ext:{name}"` prefix
