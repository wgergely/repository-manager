# Core Building Blocks Design

> **Status:** Approved for implementation
> **Date:** 2026-01-23
> **Scope:** `repo-fs` and `repo-git` foundational crates

## Overview

This document defines the foundational Layer 0 crates for Repository Manager. These must be implemented first as all other crates depend on them.

## Architecture Summary

```
┌─────────────────────────────────────────────────────────┐
│                      repo-cli                           │
├─────────────────────────────────────────────────────────┤
│                      repo-core                          │
├─────────────────────────────────────────────────────────┤
│                      repo-tools                         │
├─────────────────────────────────────────────────────────┤
│                      repo-meta                          │
├──────────────────────────┬──────────────────────────────┤
│        repo-git          │          repo-fs             │  ← Layer 0
└──────────────────────────┴──────────────────────────────┘
```

---

## Crate 1: `repo-fs`

File system abstraction layer providing layout-agnostic path resolution and safe I/O operations.

### 1.1 Core Types

#### `NormalizedPath`

Newtype wrapper ensuring consistent path handling across platforms.

```rust
pub struct NormalizedPath(PathBuf);

impl NormalizedPath {
    /// Create from any path, normalizing to forward slashes internally
    pub fn new(path: impl AsRef<Path>) -> Self;

    /// Convert to platform-native path for I/O operations
    pub fn to_native(&self) -> PathBuf;

    /// Join with another path segment
    pub fn join(&self, segment: &str) -> NormalizedPath;
}
```

**Requirements:**
- Internal representation uses forward slashes (`/`)
- Platform-native conversion only at I/O boundaries
- Handle UNC paths (`\\server\share`), mapped drives, NFS/SMB mounts
- Detect network paths and warn (allow but don't actively break)

#### `WorkspaceLayout`

Source of truth for "where am I?" resolution.

```rust
pub struct WorkspaceLayout {
    /// Container or repo root (where .repository lives)
    pub root: NormalizedPath,

    /// Active working directory (may equal root in some modes)
    pub active_context: NormalizedPath,

    /// Detected or configured layout mode
    pub mode: LayoutMode,
}

pub enum LayoutMode {
    Container,       // Our .gt model
    InRepoWorktrees, // Standard .worktrees/ folder
    Classic,         // Single checkout, no worktrees
}
```

### 1.2 Mode Detection

**Flow:**
1. Walk up from `current_dir` looking for `.repository/config.toml`
2. If found with explicit `mode`: use declared mode
3. If found without mode or not found: attempt detection
4. Validate filesystem matches declared mode
5. On mismatch: fail fast with specific repair hints

**Detection signals:**

| Mode | Primary Signal | Secondary Signal |
|------|----------------|------------------|
| Container | `.gt/` exists | `main/` sibling directory |
| InRepoWorktrees | `.git/` exists | `.worktrees/` sibling directory |
| Classic | `.git/` exists | No worktree indicators |

> **RESEARCH ITEM:** Edge case handling for ambiguous states requires dedicated investigation. Implement basic detection first, refine after real-world testing.

### 1.3 Filesystem Validation

When config declares a mode, validate filesystem state matches:

| Config Mode | Required State | Error If Missing |
|-------------|----------------|------------------|
| Container | `.gt/` exists | "Git database missing. Expected .gt/ directory." |
| Container | `main/` exists | "Primary worktree missing. Expected main/ directory." |
| InRepoWorktrees | `.git/` exists | "Not a git repository." |
| Classic | `.git/` exists | "Not a git repository." |

**Behavior:** Strict fail-fast with repair hints. No automatic repair.

### 1.4 Atomic File Operations

```rust
/// Write file atomically with locking
pub fn write_atomic(path: &NormalizedPath, content: &[u8]) -> Result<()>;
```

**Implementation:**
1. Write to `{path}.{random}.tmp` (same directory ensures same filesystem)
2. Acquire advisory lock via `fs2` or `fd-lock`
3. Flush and sync
4. Rename to target path
5. Release lock

### 1.5 Configuration Format Handling

Unified format-agnostic API:

```rust
pub trait ConfigStore {
    /// Load config from file, auto-detecting format from extension
    fn load<T: DeserializeOwned>(&self, path: &NormalizedPath) -> Result<T>;

    /// Save config to file, format determined by extension
    fn save<T: Serialize>(&self, path: &NormalizedPath, value: &T) -> Result<()>;
}
```

**Supported formats:** TOML, JSON, YAML (detected by extension)

**Implementation:** Internal dispatch to format-specific libraries, completely hidden from callers.

### 1.6 Managed Blocks

> **MAJOR RESEARCH ITEM**
>
> The managed block system requires a dedicated design phase. Scope includes:
> - Multi-format parsing (TOML, YAML, JSON, Markdown, plain text)
> - Chunk identification and extraction
> - UUID-based markers (`rp{uuid}`)
> - Pattern matching for block boundaries
> - Syntax-preserving edits
> - Format-aware comment styles
>
> This is a sophisticated text editing system, not a simple feature. Estimated as significant standalone workstream.
>
> **Blocks:** `repo-tools` sync functionality until resolved.

**Placeholder API:**
```rust
// TODO: PLACEHOLDER - replace with ManagedBlockEditor
pub fn read_text(path: &NormalizedPath) -> Result<String>;
pub fn write_text(path: &NormalizedPath, content: &str) -> Result<()>;
```

### 1.7 Dependencies

```toml
[dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
serde_json = "1.0"
serde_yaml = "0.9"

# File operations
fs2 = "0.4"           # File locking
tempfile = "3.10"     # Atomic write temp files
dunce = "1.0"         # Windows path canonicalization

# Error handling
thiserror = "2.0"
```

---

## Crate 2: `repo-git`

Git abstraction layer supporting multiple worktree layout styles through a unified interface.

### 2.1 Layout Provider Trait

```rust
pub trait LayoutProvider {
    /// Path to git database (.gt or .git)
    fn git_database(&self) -> &NormalizedPath;

    /// Path to main branch worktree
    fn main_worktree(&self) -> &NormalizedPath;

    /// Compute path for a feature worktree
    fn feature_worktree(&self, name: &str) -> NormalizedPath;

    /// List all worktrees
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;

    /// Create a new feature worktree
    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath>;

    /// Remove a feature worktree
    fn remove_feature(&self, name: &str) -> Result<()>;

    /// Get current branch name
    fn current_branch(&self) -> Result<String>;
}

pub struct WorktreeInfo {
    pub name: String,
    pub path: NormalizedPath,
    pub branch: String,
    pub is_main: bool,
}
```

### 2.2 Layout Implementations

#### Container Layout

```
{container}/
├── .gt/                  # Git database (hardcoded name)
├── .repository/          # Our config
├── main/                 # Main branch worktree
├── feature-x/            # Feature worktree (slug naming)
└── feature-y/            # Feature worktree
```

**Git database:** `.gt` (hardcoded, not configurable)

**Worktree creation:** Uses `git2::Repository::worktree()` with `WorktreeAddOptions`

#### In-Repo Worktrees Layout

```
{repo}/                   # Main branch files at root
├── .git/                 # Git database
├── .worktrees/           # Worktrees folder
│   ├── feature-x/
│   └── feature-y/
└── src/
```

**Git database:** `.git` (standard)

#### Classic Layout

```
{repo}/
├── .git/
└── src/
```

**Feature operations:** Return error with migration guidance.

```rust
impl LayoutProvider for ClassicLayout {
    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath> {
        Err(Error::LayoutUnsupported {
            operation: "create_feature",
            layout: "Classic",
            hint: "Run `repo migrate --layout in-repo-worktrees` to enable parallel worktrees.",
        })
    }
}
```

### 2.3 Branch Name to Directory Mapping

**Default:** Slug conversion
- `feat/user-auth` → `feat-user-auth`
- `bugfix/issue-123` → `bugfix-issue-123`

**Configurable:**
```toml
[worktrees]
naming = "slug"         # Default: replace / with -
# naming = "hierarchical" # Preserve as nested directories
```

```rust
pub enum NamingStrategy {
    Slug,
    Hierarchical,
}

pub fn branch_to_directory(branch: &str, strategy: NamingStrategy) -> String;
```

### 2.4 Git Operations

**Library:** `git2` only (proven by `worktree` crate)

```rust
use git2::{Repository, WorktreeAddOptions};

impl ContainerLayout {
    pub fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath> {
        let repo = Repository::open(self.git_database().to_native())?;

        let branch = match base {
            Some(base) => repo.find_branch(base, BranchType::Local)?,
            None => repo.head()?.resolve()?,
        };

        let worktree_path = self.feature_worktree(name);
        let mut opts = WorktreeAddOptions::new();
        opts.reference(Some(branch.get()));

        repo.worktree(&name, worktree_path.to_native(), Some(&opts))?;

        Ok(worktree_path)
    }
}
```

### 2.5 Layout Migrations

**Phase 2 (upgrade paths):**
- Classic → In-Repo Worktrees
- Classic → Container
- In-Repo Worktrees → Container

**Phase 3 (downgrade paths):**
- Container → In-Repo Worktrees
- Container → Classic
- In-Repo Worktrees → Classic

**Migration complexity:**

| Migration | Complexity | Notes |
|-----------|------------|-------|
| Classic → In-Repo WT | Low | Create `.worktrees/`, no file moves |
| Classic → Container | Medium | Create `.gt/`, move files to `main/`, restructure |
| In-Repo WT → Container | Medium | Create `.gt/`, restructure to siblings |
| Container → In-Repo WT | Medium | Rename `.gt` → `.git`, restructure |
| Container → Classic | High | Merge worktrees, flatten structure |
| In-Repo WT → Classic | Low | Merge worktrees, delete `.worktrees/` |

### 2.6 Dependencies

```toml
[dependencies]
git2 = "0.20"
thiserror = "2.0"

# Internal
repo-fs = { path = "../repo-fs" }
```

---

## Research Items

### Critical (Blocks Other Work)

1. **Managed Block System** (`repo-fs`)
   - Scope: Format-aware text editing with UUID markers
   - Blocks: `repo-tools` sync functionality
   - Estimated effort: Significant standalone workstream

### Important (Can Defer)

2. **Layout Detection Edge Cases** (`repo-git`)
   - Scope: Ambiguous filesystem states, mixed signals
   - Approach: Implement basic detection, refine with real-world data

---

## Implementation Order

```
Phase 1: Foundation
├── repo-fs
│   ├── NormalizedPath
│   ├── WorkspaceLayout (basic detection)
│   ├── Atomic I/O with locking
│   ├── ConfigStore (format-agnostic)
│   └── Placeholder text read/write
└── repo-git
    ├── LayoutProvider trait
    ├── ContainerLayout implementation
    ├── InRepoWorktreesLayout implementation
    ├── ClassicLayout implementation (error stubs for features)
    └── Branch naming strategies

Phase 2: Migrations
├── Classic → In-Repo Worktrees
├── Classic → Container
└── In-Repo Worktrees → Container

Phase 3: Research Items
├── Managed Block System (dedicated design)
├── Layout detection edge cases
└── Downgrade migrations
```

---

## Acceptance Criteria

### `repo-fs`

- [ ] `NormalizedPath` handles Windows UNC paths, network drives
- [ ] `WorkspaceLayout::detect()` correctly identifies all three modes
- [ ] Atomic writes prevent corruption under concurrent access
- [ ] ConfigStore loads/saves TOML, JSON, YAML transparently
- [ ] Network paths detected and warned

### `repo-git`

- [ ] `ContainerLayout` creates worktrees via git2
- [ ] `InRepoWorktreesLayout` creates worktrees in `.worktrees/`
- [ ] `ClassicLayout` returns helpful error for feature operations
- [ ] Branch slug conversion handles edge cases (`/`, special chars)
- [ ] Configurable naming strategy works

---

## Technology Choices

| Component | Choice | Rationale |
|-----------|--------|-----------|
| CLI framework | clap v4 | Industry standard, derive macros |
| Config loading | figment | Layered config, type-safe |
| Git operations | git2 | Worktree creation proven, full feature set |
| Serialization | serde + toml/json/yaml | Standard ecosystem |
| File locking | fs2 | Cross-platform advisory locks |

---

## References

- [worktree crate](https://github.com/cafreeman/worktree) - Proves git2 worktree creation works
- [Worktrunk](https://github.com/max-sixty/worktrunk) - AI agent worktree patterns
- [git2 docs](https://docs.rs/git2/latest/git2/) - API reference
- Research docs: `docs/research/rust-*.md`
