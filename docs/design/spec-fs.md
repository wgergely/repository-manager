# File Management Subsystem Specification

**Crate**: `repo-fs` (or module `repo_manager::fs`)

## 1. Overview

Abstracts filesystem operations for Standard/Worktree dual-mode. Higher-level logic remains layout-agnostic.

## 2. Core Responsibilities

1. **Path Resolution**: deterministically locating "Project Root", ".repository", and "Worktree Root".
2. **Atomic I/O**: Wrapper around file writing to prevent corruption of configuration files.
3. **Format Support**: Built-in serialization/deserialization for TOML, JSON, and YAML.
4. **Symlink Management**: Handling the complexity of linking shared resources in Worktree mode (on Windows and Unix).

## 3. Architecture & Interfaces

### 3.1 The `WorkspaceLayout` Struct

This struct is the source of truth for "Where am I?".

```rust
pub struct WorkspaceLayout {
    /// The absolute path to the "Container" or "Repo Root"
    pub root: PathBuf,
    
    /// The specific directory for the active worktree (if applicable)
    /// In Normal mode, this equals `root`.
    pub active_working_dir: PathBuf,
    
    /// The mode of operation detected
    pub mode: RepositoryMode,
}

impl WorkspaceLayout {
    /// Resolves the path to the central .repository definition
    pub fn config_dir(&self) -> PathBuf;

    /// Resolves the path to a tool's config (which might be shared or local)
    pub fn tool_config(&self, tool_name: &str) -> PathBuf;
}
```

### 3.2 Path Naming Convention

To avoid confusion between "Git Root" and "Worktree Root":

* **Container Root**: The top-level directory holding `.repository` (and `.git` in worktrees mode).
* **Context Root**: The directory where code lives.
  * *Standard Mode*: Context Root == Container Root.
  * *Worktree Mode*: Context Root == `Container Root / {branch_name}`.

The FS subsystem forces all other crates to request paths relative to one of these two roots.

## 4. Atomic File Operations

To prevent partial writes during `repo sync`, we implement a transaction-like file writer.

```rust
pub fn write_atomic(path: &Path, content: &[u8]) -> Result<()>;
```

* **Strategy**: Write to `path.tmp`, flush, then rename to `path`.
* **Locking**: Optional integration with `fs2` or `fd-lock` to prevent concurrent CLI runs from stomping state.

## 5. Supported Formats

The subsystem exports unified traits for configuration loading.

* **Parsed Configs**:
  * `load_toml<T>(path)`
  * `load_json<T>(path)`
* **Managed Blocks**:
  * `update_managed_block(path, block_id, content)`
  * Parses text files for `<!-- repo:start:id -->` markers (as defined in [State Ledger](config-ledger.md)).

## 6. Windows Considerations

* **Path Separators**: Must canonicalize to forward slashes for internal logic/globbing, resolving back to backend-specific separators only at I/O time.
* **Symlinks**: The subsystem must detect if Developer Mode is enabled or if Admin privileges are present before attempting symlinks. If not available, fallback to `Junctions` or Hard Copies (with a warning).
