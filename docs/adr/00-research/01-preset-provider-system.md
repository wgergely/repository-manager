# Research: Preset Provider System

**Date:** 2026-02-19
**Researcher:** preset-researcher (Opus agent)
**Source:** `repo-presets` crate

---

## 1. PresetProvider Trait Interface

**File:** `crates/repo-presets/src/provider.rs` (lines 94-99)

```rust
#[async_trait]
pub trait PresetProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn check(&self, context: &Context) -> Result<PresetCheckReport>;
    async fn apply(&self, context: &Context) -> Result<ApplyReport>;
}
```

Three methods:
- **`id(&self) -> &str`** - Returns a string ID like `"env:rust"`, `"env:node"`, `"env:python"`, `"env:python-venv"`
- **`check(&self, context: &Context) -> Result<PresetCheckReport>`** - Async. Inspects the environment and returns a status report
- **`apply(&self, context: &Context) -> Result<ApplyReport>`** - Async. Takes remedial action (or reports detection-only for some providers)

The trait requires `Send + Sync` (all providers must be thread-safe).

## 2. PresetStatus Enum

**File:** `provider.rs` (lines 8-14)

```rust
pub enum PresetStatus {
    Healthy,   // Environment is present and working
    Missing,   // Not detected (e.g., no Cargo.toml, no .venv)
    Drifted,   // Currently UNUSED by any provider
    Broken,    // Detected but non-functional (e.g., Cargo.toml exists but no rustc)
}
```

## 3. ActionType Enum

**File:** `provider.rs` (lines 17-23)

```rust
pub enum ActionType {
    None,      // No action needed
    Install,   // Need to create/install
    Repair,    // Need to fix (drifted)
    Update,    // Currently UNUSED
}
```

## 4. PresetCheckReport

**File:** `provider.rs` (lines 26-65)

```rust
pub struct PresetCheckReport {
    pub status: PresetStatus,
    pub details: Vec<String>,
    pub action: ActionType,
}
```

Convenience constructors: `healthy()`, `missing(detail)`, `drifted(detail)`, `broken(detail)`.

## 5. ApplyReport

**File:** `provider.rs` (lines 67-91)

```rust
pub struct ApplyReport {
    pub success: bool,
    pub actions_taken: Vec<String>,
    pub errors: Vec<String>,
}
```

## 6. Context

**File:** `crates/repo-presets/src/context.rs` (lines 8-14)

```rust
pub struct Context {
    pub layout: WorkspaceLayout,
    pub root: NormalizedPath,
    pub config: HashMap<String, toml::Value>,
    pub venv_tag: Option<String>,
}
```

Key methods:
- `python_version()` - Returns config "version" or defaults to "3.12"
- `provider()` - Returns config "provider" or defaults to "uv"
- `venv_path()` - Returns `.venv` or `.venv-{tag}` depending on tag

## 7. Provider Implementations

### RustProvider
- **ID:** `"env:rust"`
- **Detection-only** - apply() returns success saying "detection-only"
- **check():** Looks for `Cargo.toml`, then checks `rustc --version`

### NodeProvider
- **ID:** `"env:node"`
- **Detection-only**
- **check():** Checks `package.json`, `node_modules/`, and `node --version`

### UvProvider
- **ID:** `"env:python"`
- **Actually creates environments** - apply() runs `uv venv --python {version} {path}`
- **check():** Checks `uv --version` then looks for python binary in `.venv/`

### VenvProvider
- **ID:** `"env:python-venv"`
- **Actually creates environments** - apply() runs `python -m venv {path}`
- **Extra methods beyond trait:** `create_tagged_sync()`, `create_tagged()`, `generate_tag()`, `check_venv_at_path()`

## 8. Error Types

```rust
pub enum Error {
    Fs(repo_fs::Error),
    Meta(repo_meta::Error),
    CommandFailed { command: String },
    CommandNotFound { command: String },
    EnvCreationFailed { path: PathBuf, message: String },
    PythonNotFound,
    UvNotFound,
    VenvCreationFailed { path: String },
    CheckFailed { message: String },
}
```

## 9. CRITICAL: No Orchestration Layer

There are **three separate preset systems** that are NOT connected:

1. **Runtime PresetProvider trait** (repo-presets) - Struct-based providers
2. **Static Registry** (repo-meta/registry.rs) - Name-to-name mapping (`"env:python" -> "uv"`)
3. **TOML PresetDefinition schema** (repo-meta/schema/preset.rs) - Definition files with `requires` field

**No code bridges config-level preset references to actual provider instances.** The `ResolvedConfig.presets["env:python"]` config data is never passed to `UvProvider::check()` or `UvProvider::apply()`.

## 10. Gaps for Extension Dependency Resolution

| Capability | Current State | Needed for Extensions |
|---|---|---|
| Provider discovery | Hardcoded structs | Dynamic loading / trait objects |
| Config -> Provider binding | Not connected | Orchestrator with registry lookup |
| Dependency ordering | Schema only (PresetRequires) | Topological sort + execution |
| Error recovery | None | Rollback / compensating actions |
| Provider versioning | None | Semver compatibility |
| Async orchestration | Individual async | Concurrent with dep ordering |
