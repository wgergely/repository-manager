# Layer 0 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the foundational `repo-fs` and `repo-git` crates that all other Repository Manager components depend on.

**Architecture:** Cargo workspace with two crates (`repo-fs`, `repo-git`). `repo-fs` provides path normalization, atomic I/O, and config loading. `repo-git` provides layout-agnostic git operations via the `LayoutProvider` trait with three implementations (Container, InRepoWorktrees, Classic).

**Tech Stack:** Rust 2024 edition, git2 0.20, serde, toml, serde_json, serde_yaml, fs2, thiserror

**Design Reference:** `research-docs/docs/plans/2026-01-23-core-building-blocks-design.md`

---

## Phase 0: Project Setup

### Task 0.1: Initialize Cargo Workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/repo-fs/Cargo.toml`
- Create: `crates/repo-fs/src/lib.rs`
- Create: `crates/repo-git/Cargo.toml`
- Create: `crates/repo-git/src/lib.rs`

**Step 1: Create workspace Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/user/repository-manager"

[workspace.dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
serde_json = "1.0"
serde_yaml = "0.9"

# File operations
fs2 = "0.4"
tempfile = "3.10"
dunce = "1.0"

# Git
git2 = "0.20"

# Error handling
thiserror = "2.0"

# Testing
tempfile = "3.10"
```

**Step 2: Create crates directory structure**

Run:
```bash
mkdir -p crates/repo-fs/src
mkdir -p crates/repo-git/src
```

**Step 3: Create repo-fs Cargo.toml**

Create `crates/repo-fs/Cargo.toml`:
```toml
[package]
name = "repo-fs"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Filesystem abstraction for Repository Manager"

[dependencies]
serde = { workspace = true }
toml = { workspace = true }
serde_json = { workspace = true }
serde_yaml = { workspace = true }
fs2 = { workspace = true }
tempfile = { workspace = true }
dunce = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

**Step 4: Create repo-fs lib.rs stub**

Create `crates/repo-fs/src/lib.rs`:
```rust
//! Filesystem abstraction for Repository Manager
//!
//! Provides layout-agnostic path resolution and safe I/O operations.

pub mod error;
pub mod path;
pub mod io;
pub mod config;
pub mod layout;

pub use error::{Error, Result};
pub use path::NormalizedPath;
pub use layout::{WorkspaceLayout, LayoutMode};
pub use config::ConfigStore;
```

**Step 5: Create repo-git Cargo.toml**

Create `crates/repo-git/Cargo.toml`:
```toml
[package]
name = "repo-git"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Git abstraction for Repository Manager"

[dependencies]
repo-fs = { path = "../repo-fs" }
git2 = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

**Step 6: Create repo-git lib.rs stub**

Create `crates/repo-git/src/lib.rs`:
```rust
//! Git abstraction for Repository Manager
//!
//! Supports multiple worktree layout styles through a unified interface.

pub mod error;
pub mod provider;
pub mod naming;
pub mod container;
pub mod in_repo_worktrees;
pub mod classic;

pub use error::{Error, Result};
pub use provider::{LayoutProvider, WorktreeInfo};
pub use naming::NamingStrategy;
```

**Step 7: Verify workspace compiles**

Run:
```bash
cargo check
```
Expected: Compilation errors about missing modules (expected at this stage)

**Step 8: Commit**

```bash
git add -A
git commit -m "chore: initialize cargo workspace with repo-fs and repo-git crates"
```

---

## Phase 1: repo-fs Core Types

### Task 1.1: Error Types

**Files:**
- Create: `crates/repo-fs/src/error.rs`
- Test: `crates/repo-fs/src/error.rs` (doc tests)

**Step 1: Write error module**

Create `crates/repo-fs/src/error.rs`:
```rust
//! Error types for repo-fs

use std::path::PathBuf;

/// Result type for repo-fs operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in repo-fs operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse {format} config at {path}: {message}")]
    ConfigParse {
        path: PathBuf,
        format: String,
        message: String,
    },

    #[error("Unsupported config format: {extension}")]
    UnsupportedFormat { extension: String },

    #[error("Layout validation failed: {message}")]
    LayoutValidation { message: String },

    #[error("Could not detect layout mode from filesystem")]
    LayoutDetectionFailed,

    #[error("Config declares {declared:?} mode but filesystem shows {detected:?}")]
    LayoutMismatch {
        declared: super::LayoutMode,
        detected: super::LayoutMode,
    },

    #[error("Network path detected: {path}. Performance may be degraded.")]
    NetworkPathWarning { path: PathBuf },

    #[error("Lock acquisition failed for {path}")]
    LockFailed { path: PathBuf },
}

impl Error {
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
```

**Step 2: Verify compiles**

Run:
```bash
cargo check -p repo-fs
```
Expected: Errors about missing LayoutMode (expected)

**Step 3: Commit**

```bash
git add crates/repo-fs/src/error.rs
git commit -m "feat(repo-fs): add error types"
```

---

### Task 1.2: NormalizedPath

**Files:**
- Create: `crates/repo-fs/src/path.rs`
- Test: `crates/repo-fs/tests/path_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-fs/tests/path_tests.rs`:
```rust
use repo_fs::NormalizedPath;
use std::path::Path;

#[test]
fn test_normalize_forward_slashes() {
    let path = NormalizedPath::new("foo/bar/baz");
    assert_eq!(path.as_str(), "foo/bar/baz");
}

#[test]
fn test_normalize_backslashes_to_forward() {
    let path = NormalizedPath::new("foo\\bar\\baz");
    assert_eq!(path.as_str(), "foo/bar/baz");
}

#[test]
fn test_normalize_mixed_slashes() {
    let path = NormalizedPath::new("foo/bar\\baz");
    assert_eq!(path.as_str(), "foo/bar/baz");
}

#[test]
fn test_join_paths() {
    let base = NormalizedPath::new("foo/bar");
    let joined = base.join("baz");
    assert_eq!(joined.as_str(), "foo/bar/baz");
}

#[test]
fn test_to_native_returns_pathbuf() {
    let path = NormalizedPath::new("foo/bar");
    let native = path.to_native();
    // On Windows this would have backslashes, on Unix forward slashes
    assert!(native.to_string_lossy().contains("bar"));
}

#[test]
fn test_is_network_path_unc() {
    let path = NormalizedPath::new("//server/share/path");
    assert!(path.is_network_path());
}

#[test]
fn test_is_network_path_local() {
    let path = NormalizedPath::new("/home/user/project");
    assert!(!path.is_network_path());
}

#[test]
fn test_parent() {
    let path = NormalizedPath::new("foo/bar/baz");
    let parent = path.parent().unwrap();
    assert_eq!(parent.as_str(), "foo/bar");
}

#[test]
fn test_file_name() {
    let path = NormalizedPath::new("foo/bar/baz.txt");
    assert_eq!(path.file_name(), Some("baz.txt"));
}

#[test]
fn test_exists_false_for_nonexistent() {
    let path = NormalizedPath::new("/nonexistent/path/that/does/not/exist");
    assert!(!path.exists());
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-fs path_tests
```
Expected: FAIL - module not found

**Step 3: Write implementation**

Create `crates/repo-fs/src/path.rs`:
```rust
//! Normalized path handling for cross-platform compatibility

use std::path::{Path, PathBuf};

/// A path normalized to use forward slashes internally.
///
/// Provides consistent path handling across platforms by normalizing
/// all paths to forward slashes internally and converting to
/// platform-native format only at I/O boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedPath {
    /// Internal representation always uses forward slashes
    inner: String,
}

impl NormalizedPath {
    /// Create a new NormalizedPath from any path-like input.
    ///
    /// Converts backslashes to forward slashes for internal storage.
    pub fn new(path: impl AsRef<Path>) -> Self {
        let path_str = path.as_ref().to_string_lossy();
        let normalized = path_str.replace('\\', "/");
        Self { inner: normalized }
    }

    /// Get the internal normalized string representation.
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Convert to a platform-native PathBuf for I/O operations.
    pub fn to_native(&self) -> PathBuf {
        PathBuf::from(&self.inner)
    }

    /// Join this path with a segment.
    pub fn join(&self, segment: &str) -> Self {
        let segment_normalized = segment.replace('\\', "/");
        let joined = if self.inner.ends_with('/') {
            format!("{}{}", self.inner, segment_normalized)
        } else {
            format!("{}/{}", self.inner, segment_normalized)
        };
        Self { inner: joined }
    }

    /// Get the parent directory.
    pub fn parent(&self) -> Option<Self> {
        let trimmed = self.inner.trim_end_matches('/');
        match trimmed.rfind('/') {
            Some(idx) if idx > 0 => Some(Self {
                inner: trimmed[..idx].to_string(),
            }),
            Some(0) => Some(Self {
                inner: "/".to_string(),
            }),
            _ => None,
        }
    }

    /// Get the file name component.
    pub fn file_name(&self) -> Option<&str> {
        let trimmed = self.inner.trim_end_matches('/');
        trimmed.rsplit('/').next()
    }

    /// Check if this path exists on the filesystem.
    pub fn exists(&self) -> bool {
        self.to_native().exists()
    }

    /// Check if this is a directory.
    pub fn is_dir(&self) -> bool {
        self.to_native().is_dir()
    }

    /// Check if this is a file.
    pub fn is_file(&self) -> bool {
        self.to_native().is_file()
    }

    /// Check if this appears to be a network path.
    ///
    /// Detects UNC paths (//server/share or \\server\share)
    /// and warns but allows operation.
    pub fn is_network_path(&self) -> bool {
        self.inner.starts_with("//")
            || self.inner.starts_with("\\\\")
            || self.inner.starts_with("smb://")
            || self.inner.starts_with("nfs://")
    }

    /// Get the extension if present.
    pub fn extension(&self) -> Option<&str> {
        self.file_name().and_then(|name| {
            let idx = name.rfind('.')?;
            if idx == 0 {
                None
            } else {
                Some(&name[idx + 1..])
            }
        })
    }
}

impl AsRef<Path> for NormalizedPath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl std::fmt::Display for NormalizedPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<&str> for NormalizedPath {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for NormalizedPath {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<PathBuf> for NormalizedPath {
    fn from(p: PathBuf) -> Self {
        Self::new(p)
    }
}

impl From<&Path> for NormalizedPath {
    fn from(p: &Path) -> Self {
        Self::new(p)
    }
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-fs path_tests
```
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-fs/src/path.rs crates/repo-fs/tests/path_tests.rs
git commit -m "feat(repo-fs): implement NormalizedPath with cross-platform handling"
```

---

### Task 1.3: Layout Mode Types

**Files:**
- Create: `crates/repo-fs/src/layout.rs`
- Test: `crates/repo-fs/tests/layout_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-fs/tests/layout_tests.rs`:
```rust
use repo_fs::{LayoutMode, WorkspaceLayout, NormalizedPath};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_layout_mode_display() {
    assert_eq!(format!("{}", LayoutMode::Container), "Container");
    assert_eq!(format!("{}", LayoutMode::InRepoWorktrees), "InRepoWorktrees");
    assert_eq!(format!("{}", LayoutMode::Classic), "Classic");
}

#[test]
fn test_detect_container_layout() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create Container layout signals
    fs::create_dir(root.join(".gt")).unwrap();
    fs::create_dir(root.join("main")).unwrap();
    fs::create_dir(root.join(".repository")).unwrap();

    let layout = WorkspaceLayout::detect(root).unwrap();
    assert_eq!(layout.mode, LayoutMode::Container);
}

#[test]
fn test_detect_in_repo_worktrees_layout() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create InRepoWorktrees layout signals
    fs::create_dir(root.join(".git")).unwrap();
    fs::create_dir(root.join(".worktrees")).unwrap();

    let layout = WorkspaceLayout::detect(root).unwrap();
    assert_eq!(layout.mode, LayoutMode::InRepoWorktrees);
}

#[test]
fn test_detect_classic_layout() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create Classic layout signals
    fs::create_dir(root.join(".git")).unwrap();

    let layout = WorkspaceLayout::detect(root).unwrap();
    assert_eq!(layout.mode, LayoutMode::Classic);
}

#[test]
fn test_detect_fails_without_git() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // No git signals at all
    let result = WorkspaceLayout::detect(root);
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-fs layout_tests
```
Expected: FAIL - types not found

**Step 3: Write implementation**

Create `crates/repo-fs/src/layout.rs`:
```rust
//! Workspace layout detection and management

use std::path::Path;
use crate::{Error, NormalizedPath, Result};

/// The detected or configured layout mode for a workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// Container layout with `.gt/` database and sibling worktrees
    Container,
    /// In-repo worktrees with `.worktrees/` folder
    InRepoWorktrees,
    /// Classic single-checkout git repository
    Classic,
}

impl std::fmt::Display for LayoutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Container => write!(f, "Container"),
            Self::InRepoWorktrees => write!(f, "InRepoWorktrees"),
            Self::Classic => write!(f, "Classic"),
        }
    }
}

/// Workspace layout information.
///
/// Source of truth for "where am I?" resolution.
#[derive(Debug, Clone)]
pub struct WorkspaceLayout {
    /// Container or repo root (where .repository lives)
    pub root: NormalizedPath,

    /// Active working directory (may equal root in some modes)
    pub active_context: NormalizedPath,

    /// Detected or configured layout mode
    pub mode: LayoutMode,
}

impl WorkspaceLayout {
    /// Detect the workspace layout starting from the given directory.
    ///
    /// Walks up the directory tree looking for layout signals.
    pub fn detect(start_dir: impl AsRef<Path>) -> Result<Self> {
        let start = start_dir.as_ref().canonicalize()
            .map_err(|e| Error::io(start_dir.as_ref(), e))?;

        let mut current = Some(start.as_path());

        while let Some(dir) = current {
            if let Some(layout) = Self::detect_at(dir)? {
                return Ok(layout);
            }
            current = dir.parent();
        }

        Err(Error::LayoutDetectionFailed)
    }

    /// Attempt to detect layout at a specific directory.
    fn detect_at(dir: &Path) -> Result<Option<Self>> {
        let has_gt = dir.join(".gt").is_dir();
        let has_git = dir.join(".git").exists(); // Can be file or dir
        let has_main = dir.join("main").is_dir();
        let has_worktrees = dir.join(".worktrees").is_dir();

        let mode = if has_gt && has_main {
            // Container layout: .gt/ + main/
            Some(LayoutMode::Container)
        } else if has_git && has_worktrees {
            // In-repo worktrees: .git + .worktrees/
            Some(LayoutMode::InRepoWorktrees)
        } else if has_git {
            // Classic: just .git
            Some(LayoutMode::Classic)
        } else {
            None
        };

        Ok(mode.map(|mode| Self {
            root: NormalizedPath::new(dir),
            active_context: NormalizedPath::new(dir),
            mode,
        }))
    }

    /// Get the path to the git database.
    pub fn git_database(&self) -> NormalizedPath {
        match self.mode {
            LayoutMode::Container => self.root.join(".gt"),
            LayoutMode::InRepoWorktrees | LayoutMode::Classic => self.root.join(".git"),
        }
    }

    /// Get the path to the .repository config directory.
    pub fn config_dir(&self) -> NormalizedPath {
        self.root.join(".repository")
    }

    /// Validate that the filesystem matches the expected layout.
    pub fn validate(&self) -> Result<()> {
        match self.mode {
            LayoutMode::Container => {
                if !self.root.join(".gt").exists() {
                    return Err(Error::LayoutValidation {
                        message: "Git database missing. Expected .gt/ directory.".into(),
                    });
                }
                if !self.root.join("main").exists() {
                    return Err(Error::LayoutValidation {
                        message: "Primary worktree missing. Expected main/ directory.".into(),
                    });
                }
            }
            LayoutMode::InRepoWorktrees | LayoutMode::Classic => {
                if !self.root.join(".git").exists() {
                    return Err(Error::LayoutValidation {
                        message: "Not a git repository.".into(),
                    });
                }
            }
        }
        Ok(())
    }
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-fs layout_tests
```
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-fs/src/layout.rs crates/repo-fs/tests/layout_tests.rs
git commit -m "feat(repo-fs): implement WorkspaceLayout with mode detection"
```

---

### Task 1.4: Atomic I/O

**Files:**
- Create: `crates/repo-fs/src/io.rs`
- Test: `crates/repo-fs/tests/io_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-fs/tests/io_tests.rs`:
```rust
use repo_fs::{io, NormalizedPath};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_write_atomic_creates_file() {
    let temp = TempDir::new().unwrap();
    let path = NormalizedPath::new(temp.path().join("test.txt"));

    io::write_atomic(&path, b"hello world").unwrap();

    let content = fs::read_to_string(path.to_native()).unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_write_atomic_overwrites_existing() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "original").unwrap();

    let path = NormalizedPath::new(&file_path);
    io::write_atomic(&path, b"updated").unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "updated");
}

#[test]
fn test_write_atomic_no_partial_writes() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "original content").unwrap();

    let path = NormalizedPath::new(&file_path);

    // Even if this were to fail mid-write, we shouldn't see partial content
    io::write_atomic(&path, b"new content").unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    // Should be either "original content" or "new content", never partial
    assert!(content == "original content" || content == "new content");
}

#[test]
fn test_read_text_existing_file() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("test.txt");
    fs::write(&file_path, "hello").unwrap();

    let path = NormalizedPath::new(&file_path);
    let content = io::read_text(&path).unwrap();
    assert_eq!(content, "hello");
}

#[test]
fn test_read_text_nonexistent_file() {
    let path = NormalizedPath::new("/nonexistent/file.txt");
    let result = io::read_text(&path);
    assert!(result.is_err());
}

#[test]
fn test_write_text_creates_file() {
    let temp = TempDir::new().unwrap();
    let path = NormalizedPath::new(temp.path().join("test.txt"));

    io::write_text(&path, "hello world").unwrap();

    let content = fs::read_to_string(path.to_native()).unwrap();
    assert_eq!(content, "hello world");
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-fs io_tests
```
Expected: FAIL - module not found

**Step 3: Write implementation**

Create `crates/repo-fs/src/io.rs`:
```rust
//! Atomic I/O operations with file locking

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use fs2::FileExt;
use crate::{Error, NormalizedPath, Result};

/// Write content atomically to a file with locking.
///
/// Uses write-to-temp-then-rename strategy to prevent partial writes.
/// Acquires an advisory lock to prevent concurrent access.
pub fn write_atomic(path: &NormalizedPath, content: &[u8]) -> Result<()> {
    let native_path = path.to_native();

    // Ensure parent directory exists
    if let Some(parent) = native_path.parent() {
        fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
    }

    // Generate temp file path in same directory (ensures same filesystem)
    let temp_name = format!(
        ".{}.{}.tmp",
        native_path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        std::process::id()
    );
    let temp_path = native_path.with_file_name(&temp_name);

    // Write to temp file
    let mut temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_path)
        .map_err(|e| Error::io(&temp_path, e))?;

    // Acquire exclusive lock
    temp_file.lock_exclusive()
        .map_err(|_| Error::LockFailed { path: native_path.clone() })?;

    // Write content
    temp_file.write_all(content)
        .map_err(|e| Error::io(&temp_path, e))?;

    // Flush to disk
    temp_file.sync_all()
        .map_err(|e| Error::io(&temp_path, e))?;

    // Release lock (implicit on drop, but be explicit)
    temp_file.unlock()
        .map_err(|_| Error::LockFailed { path: native_path.clone() })?;

    // Atomic rename
    fs::rename(&temp_path, &native_path)
        .map_err(|e| Error::io(&native_path, e))?;

    Ok(())
}

/// Read text content from a file.
///
/// TODO: PLACEHOLDER - replace with ManagedBlockEditor
pub fn read_text(path: &NormalizedPath) -> Result<String> {
    let native_path = path.to_native();
    fs::read_to_string(&native_path)
        .map_err(|e| Error::io(&native_path, e))
}

/// Write text content to a file atomically.
///
/// TODO: PLACEHOLDER - replace with ManagedBlockEditor
pub fn write_text(path: &NormalizedPath, content: &str) -> Result<()> {
    write_atomic(path, content.as_bytes())
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-fs io_tests
```
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-fs/src/io.rs crates/repo-fs/tests/io_tests.rs
git commit -m "feat(repo-fs): implement atomic I/O with file locking"
```

---

### Task 1.5: ConfigStore

**Files:**
- Create: `crates/repo-fs/src/config.rs`
- Test: `crates/repo-fs/tests/config_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-fs/tests/config_tests.rs`:
```rust
use repo_fs::{ConfigStore, NormalizedPath};
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::TempDir;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    name: String,
    count: i32,
}

#[test]
fn test_load_toml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.toml");
    fs::write(&file_path, r#"name = "test"
count = 42"#).unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let config: TestConfig = store.load(&path).unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.count, 42);
}

#[test]
fn test_load_json() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.json");
    fs::write(&file_path, r#"{"name": "test", "count": 42}"#).unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let config: TestConfig = store.load(&path).unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.count, 42);
}

#[test]
fn test_load_yaml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.yaml");
    fs::write(&file_path, "name: test\ncount: 42").unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let config: TestConfig = store.load(&path).unwrap();

    assert_eq!(config.name, "test");
    assert_eq!(config.count, 42);
}

#[test]
fn test_save_toml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.toml");
    let path = NormalizedPath::new(&file_path);

    let config = TestConfig { name: "test".into(), count: 42 };
    let store = ConfigStore::new();
    store.save(&path, &config).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("name = \"test\""));
    assert!(content.contains("count = 42"));
}

#[test]
fn test_save_json() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.json");
    let path = NormalizedPath::new(&file_path);

    let config = TestConfig { name: "test".into(), count: 42 };
    let store = ConfigStore::new();
    store.save(&path, &config).unwrap();

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("\"name\""));
    assert!(content.contains("\"test\""));
}

#[test]
fn test_unsupported_format() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.xyz");
    fs::write(&file_path, "data").unwrap();

    let store = ConfigStore::new();
    let path = NormalizedPath::new(&file_path);
    let result: repo_fs::Result<TestConfig> = store.load(&path);

    assert!(result.is_err());
}

#[test]
fn test_roundtrip_toml() {
    let temp = TempDir::new().unwrap();
    let file_path = temp.path().join("config.toml");
    let path = NormalizedPath::new(&file_path);

    let original = TestConfig { name: "roundtrip".into(), count: 123 };
    let store = ConfigStore::new();

    store.save(&path, &original).unwrap();
    let loaded: TestConfig = store.load(&path).unwrap();

    assert_eq!(original, loaded);
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-fs config_tests
```
Expected: FAIL - ConfigStore not found

**Step 3: Write implementation**

Create `crates/repo-fs/src/config.rs`:
```rust
//! Format-agnostic configuration loading and saving

use serde::{de::DeserializeOwned, Serialize};
use crate::{io, Error, NormalizedPath, Result};

/// Format-agnostic configuration store.
///
/// Automatically detects format from file extension and handles
/// serialization/deserialization transparently.
#[derive(Debug, Default)]
pub struct ConfigStore;

impl ConfigStore {
    /// Create a new ConfigStore.
    pub fn new() -> Self {
        Self
    }

    /// Load configuration from a file.
    ///
    /// Format is detected from file extension:
    /// - `.toml` -> TOML
    /// - `.json` -> JSON
    /// - `.yaml`, `.yml` -> YAML
    pub fn load<T: DeserializeOwned>(&self, path: &NormalizedPath) -> Result<T> {
        let content = io::read_text(path)?;
        let extension = path.extension().unwrap_or("");

        match extension.to_lowercase().as_str() {
            "toml" => toml::from_str(&content).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "TOML".into(),
                message: e.to_string(),
            }),
            "json" => serde_json::from_str(&content).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "JSON".into(),
                message: e.to_string(),
            }),
            "yaml" | "yml" => serde_yaml::from_str(&content).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "YAML".into(),
                message: e.to_string(),
            }),
            _ => Err(Error::UnsupportedFormat {
                extension: extension.to_string(),
            }),
        }
    }

    /// Save configuration to a file.
    ///
    /// Format is determined from file extension.
    /// Uses atomic write to prevent corruption.
    pub fn save<T: Serialize>(&self, path: &NormalizedPath, value: &T) -> Result<()> {
        let extension = path.extension().unwrap_or("");

        let content = match extension.to_lowercase().as_str() {
            "toml" => toml::to_string_pretty(value).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "TOML".into(),
                message: e.to_string(),
            })?,
            "json" => serde_json::to_string_pretty(value).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "JSON".into(),
                message: e.to_string(),
            })?,
            "yaml" | "yml" => serde_yaml::to_string(value).map_err(|e| Error::ConfigParse {
                path: path.to_native(),
                format: "YAML".into(),
                message: e.to_string(),
            })?,
            _ => return Err(Error::UnsupportedFormat {
                extension: extension.to_string(),
            }),
        };

        io::write_text(path, &content)
    }
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-fs config_tests
```
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-fs/src/config.rs crates/repo-fs/tests/config_tests.rs
git commit -m "feat(repo-fs): implement format-agnostic ConfigStore"
```

---

### Task 1.6: Update lib.rs exports and verify repo-fs compiles

**Files:**
- Modify: `crates/repo-fs/src/lib.rs`

**Step 1: Update lib.rs with correct exports**

Update `crates/repo-fs/src/lib.rs`:
```rust
//! Filesystem abstraction for Repository Manager
//!
//! Provides layout-agnostic path resolution and safe I/O operations.

pub mod error;
pub mod path;
pub mod io;
pub mod config;
pub mod layout;

pub use error::{Error, Result};
pub use path::NormalizedPath;
pub use layout::{WorkspaceLayout, LayoutMode};
pub use config::ConfigStore;
```

**Step 2: Run all repo-fs tests**

Run:
```bash
cargo test -p repo-fs
```
Expected: All tests PASS

**Step 3: Commit**

```bash
git add crates/repo-fs/src/lib.rs
git commit -m "feat(repo-fs): finalize module exports"
```

---

## Phase 2: repo-git Core Types

### Task 2.1: Error Types

**Files:**
- Create: `crates/repo-git/src/error.rs`

**Step 1: Write error module**

Create `crates/repo-git/src/error.rs`:
```rust
//! Error types for repo-git

use std::path::PathBuf;

/// Result type for repo-git operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in repo-git operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Worktree '{name}' already exists at {path}")]
    WorktreeExists { name: String, path: PathBuf },

    #[error("Worktree '{name}' not found")]
    WorktreeNotFound { name: String },

    #[error("Branch '{name}' not found")]
    BranchNotFound { name: String },

    #[error("Operation '{operation}' not supported in {layout} layout. {hint}")]
    LayoutUnsupported {
        operation: String,
        layout: String,
        hint: String,
    },

    #[error("Invalid branch name: {name}")]
    InvalidBranchName { name: String },
}
```

**Step 2: Verify compiles**

Run:
```bash
cargo check -p repo-git
```
Expected: May have errors about missing modules (expected)

**Step 3: Commit**

```bash
git add crates/repo-git/src/error.rs
git commit -m "feat(repo-git): add error types"
```

---

### Task 2.2: Naming Strategy

**Files:**
- Create: `crates/repo-git/src/naming.rs`
- Test: `crates/repo-git/tests/naming_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-git/tests/naming_tests.rs`:
```rust
use repo_git::{NamingStrategy, naming::branch_to_directory};

#[test]
fn test_slug_simple_branch() {
    let result = branch_to_directory("feature-auth", NamingStrategy::Slug);
    assert_eq!(result, "feature-auth");
}

#[test]
fn test_slug_branch_with_slash() {
    let result = branch_to_directory("feat/user-auth", NamingStrategy::Slug);
    assert_eq!(result, "feat-user-auth");
}

#[test]
fn test_slug_multiple_slashes() {
    let result = branch_to_directory("feat/user/auth/login", NamingStrategy::Slug);
    assert_eq!(result, "feat-user-auth-login");
}

#[test]
fn test_slug_special_characters() {
    let result = branch_to_directory("fix:bug#123", NamingStrategy::Slug);
    assert_eq!(result, "fix-bug-123");
}

#[test]
fn test_hierarchical_simple_branch() {
    let result = branch_to_directory("feature-auth", NamingStrategy::Hierarchical);
    assert_eq!(result, "feature-auth");
}

#[test]
fn test_hierarchical_branch_with_slash() {
    let result = branch_to_directory("feat/user-auth", NamingStrategy::Hierarchical);
    assert_eq!(result, "feat/user-auth");
}

#[test]
fn test_slug_removes_leading_trailing_dashes() {
    let result = branch_to_directory("/feat/", NamingStrategy::Slug);
    assert_eq!(result, "feat");
}

#[test]
fn test_slug_collapses_multiple_dashes() {
    let result = branch_to_directory("feat//double//slash", NamingStrategy::Slug);
    assert_eq!(result, "feat-double-slash");
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-git naming_tests
```
Expected: FAIL - module not found

**Step 3: Write implementation**

Create `crates/repo-git/src/naming.rs`:
```rust
//! Branch name to directory name mapping strategies

/// Strategy for converting branch names to directory names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NamingStrategy {
    /// Convert slashes to dashes, remove unsafe characters.
    /// `feat/user-auth` -> `feat-user-auth`
    #[default]
    Slug,

    /// Preserve slashes as directory hierarchy.
    /// `feat/user-auth` -> `feat/user-auth`
    Hierarchical,
}

/// Convert a branch name to a directory name using the given strategy.
pub fn branch_to_directory(branch: &str, strategy: NamingStrategy) -> String {
    match strategy {
        NamingStrategy::Slug => slugify(branch),
        NamingStrategy::Hierarchical => sanitize_hierarchical(branch),
    }
}

/// Convert branch name to a flat slug.
fn slugify(branch: &str) -> String {
    let mut result = String::with_capacity(branch.len());
    let mut last_was_dash = true; // Start true to skip leading dashes

    for c in branch.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            if c == '-' || c == '_' {
                if !last_was_dash {
                    result.push('-');
                    last_was_dash = true;
                }
            } else {
                result.push(c);
                last_was_dash = false;
            }
        } else {
            // Replace other characters (including /) with dash
            if !last_was_dash {
                result.push('-');
                last_was_dash = true;
            }
        }
    }

    // Remove trailing dash
    if result.ends_with('-') {
        result.pop();
    }

    result
}

/// Sanitize for hierarchical naming, keeping slashes but removing unsafe chars.
fn sanitize_hierarchical(branch: &str) -> String {
    let mut result = String::with_capacity(branch.len());

    for c in branch.chars() {
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '/' {
            result.push(c);
        } else {
            result.push('-');
        }
    }

    // Clean up multiple slashes and leading/trailing slashes
    let result = result.trim_matches('/');
    let mut cleaned = String::with_capacity(result.len());
    let mut last_was_slash = false;

    for c in result.chars() {
        if c == '/' {
            if !last_was_slash {
                cleaned.push(c);
                last_was_slash = true;
            }
        } else {
            cleaned.push(c);
            last_was_slash = false;
        }
    }

    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("hello-world"), "hello-world");
    }

    #[test]
    fn test_slugify_with_slashes() {
        assert_eq!(slugify("feat/auth"), "feat-auth");
    }
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-git naming_tests
```
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-git/src/naming.rs crates/repo-git/tests/naming_tests.rs
git commit -m "feat(repo-git): implement branch naming strategies"
```

---

### Task 2.3: LayoutProvider Trait and WorktreeInfo

**Files:**
- Create: `crates/repo-git/src/provider.rs`

**Step 1: Write the trait and types**

Create `crates/repo-git/src/provider.rs`:
```rust
//! Layout provider trait for git operations

use repo_fs::NormalizedPath;
use crate::Result;

/// Information about a worktree.
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    /// Worktree name (matches branch name or slug)
    pub name: String,

    /// Filesystem path to the worktree
    pub path: NormalizedPath,

    /// Branch checked out in this worktree
    pub branch: String,

    /// Whether this is the main/primary worktree
    pub is_main: bool,
}

/// Trait for layout-agnostic git operations.
///
/// Implementations handle the specifics of each layout mode
/// (Container, InRepoWorktrees, Classic).
pub trait LayoutProvider {
    /// Path to the git database (.gt or .git)
    fn git_database(&self) -> &NormalizedPath;

    /// Path to the main branch worktree
    fn main_worktree(&self) -> &NormalizedPath;

    /// Compute path for a feature worktree by name
    fn feature_worktree(&self, name: &str) -> NormalizedPath;

    /// List all worktrees in this layout
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;

    /// Create a new feature worktree
    ///
    /// - `name`: Branch/worktree name
    /// - `base`: Optional base branch (defaults to current HEAD)
    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath>;

    /// Remove a feature worktree
    fn remove_feature(&self, name: &str) -> Result<()>;

    /// Get the current branch name
    fn current_branch(&self) -> Result<String>;
}
```

**Step 2: Verify compiles**

Run:
```bash
cargo check -p repo-git
```

**Step 3: Commit**

```bash
git add crates/repo-git/src/provider.rs
git commit -m "feat(repo-git): define LayoutProvider trait and WorktreeInfo"
```

---

### Task 2.4: Classic Layout Implementation

**Files:**
- Create: `crates/repo-git/src/classic.rs`
- Test: `crates/repo-git/tests/classic_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-git/tests/classic_tests.rs`:
```rust
use repo_git::classic::ClassicLayout;
use repo_git::provider::LayoutProvider;
use repo_fs::NormalizedPath;
use std::fs;
use tempfile::TempDir;

fn setup_classic_repo() -> (TempDir, ClassicLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Initialize a basic git repo structure
    fs::create_dir(root.join(".git")).unwrap();
    fs::write(root.join(".git/HEAD"), "ref: refs/heads/main").unwrap();
    fs::create_dir_all(root.join(".git/refs/heads")).unwrap();

    let layout = ClassicLayout::new(NormalizedPath::new(root)).unwrap();
    (temp, layout)
}

#[test]
fn test_classic_git_database() {
    let (temp, layout) = setup_classic_repo();
    assert!(layout.git_database().as_str().ends_with(".git"));
}

#[test]
fn test_classic_main_worktree_is_root() {
    let (temp, layout) = setup_classic_repo();
    // In classic layout, main worktree IS the root
    assert_eq!(layout.main_worktree().as_str(), layout.git_database().parent().unwrap().as_str());
}

#[test]
fn test_classic_create_feature_returns_error() {
    let (_temp, layout) = setup_classic_repo();
    let result = layout.create_feature("test-feature", None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("not supported"));
    assert!(err_str.contains("Classic"));
}

#[test]
fn test_classic_remove_feature_returns_error() {
    let (_temp, layout) = setup_classic_repo();
    let result = layout.remove_feature("test-feature");
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-git classic_tests
```
Expected: FAIL - module not found

**Step 3: Write implementation**

Create `crates/repo-git/src/classic.rs`:
```rust
//! Classic single-checkout layout implementation

use repo_fs::NormalizedPath;
use crate::{Error, Result, provider::{LayoutProvider, WorktreeInfo}};

/// Classic single-checkout git repository layout.
///
/// Does not support parallel worktrees. Feature operations
/// return errors with migration guidance.
pub struct ClassicLayout {
    root: NormalizedPath,
    git_dir: NormalizedPath,
}

impl ClassicLayout {
    /// Create a new ClassicLayout for the given root directory.
    pub fn new(root: NormalizedPath) -> Result<Self> {
        let git_dir = root.join(".git");
        if !git_dir.exists() {
            return Err(Error::Fs(repo_fs::Error::LayoutValidation {
                message: "Not a git repository: .git not found".into(),
            }));
        }
        Ok(Self { root, git_dir })
    }
}

impl LayoutProvider for ClassicLayout {
    fn git_database(&self) -> &NormalizedPath {
        &self.git_dir
    }

    fn main_worktree(&self) -> &NormalizedPath {
        &self.root
    }

    fn feature_worktree(&self, name: &str) -> NormalizedPath {
        // Classic layout doesn't have feature worktrees,
        // but we return a hypothetical path for error messages
        self.root.join(name)
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        // Classic layout only has one "worktree" - the repo itself
        let branch = self.current_branch().unwrap_or_else(|_| "unknown".into());
        Ok(vec![WorktreeInfo {
            name: "main".into(),
            path: self.root.clone(),
            branch,
            is_main: true,
        }])
    }

    fn create_feature(&self, _name: &str, _base: Option<&str>) -> Result<NormalizedPath> {
        Err(Error::LayoutUnsupported {
            operation: "create_feature".into(),
            layout: "Classic".into(),
            hint: "Run `repo migrate --layout in-repo-worktrees` to enable parallel worktrees.".into(),
        })
    }

    fn remove_feature(&self, _name: &str) -> Result<()> {
        Err(Error::LayoutUnsupported {
            operation: "remove_feature".into(),
            layout: "Classic".into(),
            hint: "Run `repo migrate --layout in-repo-worktrees` to enable parallel worktrees.".into(),
        })
    }

    fn current_branch(&self) -> Result<String> {
        let repo = git2::Repository::open(self.git_dir.to_native())?;
        let head = repo.head()?;

        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            // Detached HEAD
            Ok("HEAD".to_string())
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-git classic_tests
```
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-git/src/classic.rs crates/repo-git/tests/classic_tests.rs
git commit -m "feat(repo-git): implement ClassicLayout with migration hints"
```

---

### Task 2.5: Container Layout Implementation

**Files:**
- Create: `crates/repo-git/src/container.rs`
- Test: `crates/repo-git/tests/container_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-git/tests/container_tests.rs`:
```rust
use repo_git::container::ContainerLayout;
use repo_git::provider::LayoutProvider;
use repo_git::NamingStrategy;
use repo_fs::NormalizedPath;
use std::process::Command;
use std::fs;
use tempfile::TempDir;

fn setup_container_repo() -> (TempDir, ContainerLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create container structure with real git
    let gt_dir = root.join(".gt");
    let main_dir = root.join("main");

    // Initialize bare repo in .gt
    Command::new("git")
        .args(["init", "--bare"])
        .arg(&gt_dir)
        .output()
        .expect("Failed to init bare repo");

    // Add main as worktree
    Command::new("git")
        .current_dir(&gt_dir)
        .args(["worktree", "add", "--orphan", "-b", "main"])
        .arg(&main_dir)
        .output()
        .expect("Failed to add main worktree");

    // Create an initial commit in main
    fs::write(main_dir.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .current_dir(&main_dir)
        .args(["add", "README.md"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(&main_dir)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    let layout = ContainerLayout::new(
        NormalizedPath::new(root),
        NamingStrategy::Slug,
    ).unwrap();

    (temp, layout)
}

#[test]
fn test_container_git_database() {
    let (_temp, layout) = setup_container_repo();
    assert!(layout.git_database().as_str().ends_with(".gt"));
}

#[test]
fn test_container_main_worktree() {
    let (_temp, layout) = setup_container_repo();
    assert!(layout.main_worktree().as_str().ends_with("main"));
}

#[test]
fn test_container_feature_worktree_path() {
    let (_temp, layout) = setup_container_repo();
    let path = layout.feature_worktree("feat-auth");
    assert!(path.as_str().ends_with("feat-auth"));
}

#[test]
fn test_container_list_worktrees() {
    let (_temp, layout) = setup_container_repo();
    let worktrees = layout.list_worktrees().unwrap();

    assert!(!worktrees.is_empty());
    assert!(worktrees.iter().any(|wt| wt.is_main));
}

#[test]
fn test_container_create_and_remove_feature() {
    let (_temp, layout) = setup_container_repo();

    // Create feature
    let path = layout.create_feature("test-feature", None).unwrap();
    assert!(path.exists());

    // Verify it's in list
    let worktrees = layout.list_worktrees().unwrap();
    assert!(worktrees.iter().any(|wt| wt.name == "test-feature"));

    // Remove feature
    layout.remove_feature("test-feature").unwrap();
    assert!(!path.exists());
}

#[test]
fn test_container_slug_naming() {
    let (_temp, layout) = setup_container_repo();

    // Create with slash in name - should be slugified
    let path = layout.create_feature("feat/user-auth", None).unwrap();
    assert!(path.as_str().ends_with("feat-user-auth"));

    // Cleanup
    layout.remove_feature("feat/user-auth").unwrap();
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-git container_tests
```
Expected: FAIL - module not found

**Step 3: Write implementation**

Create `crates/repo-git/src/container.rs`:
```rust
//! Container layout implementation with .gt database

use git2::{BranchType, Repository, WorktreeAddOptions};
use repo_fs::NormalizedPath;
use crate::{
    Error, Result,
    naming::{branch_to_directory, NamingStrategy},
    provider::{LayoutProvider, WorktreeInfo},
};

/// Container layout with `.gt/` database and sibling worktrees.
///
/// ```text
/// {container}/
/// ├── .gt/          # Git database
/// ├── main/         # Main branch worktree
/// └── feature-x/    # Feature worktree
/// ```
pub struct ContainerLayout {
    root: NormalizedPath,
    git_dir: NormalizedPath,
    main_dir: NormalizedPath,
    naming: NamingStrategy,
}

impl ContainerLayout {
    /// Create a new ContainerLayout for the given root directory.
    pub fn new(root: NormalizedPath, naming: NamingStrategy) -> Result<Self> {
        let git_dir = root.join(".gt");
        let main_dir = root.join("main");

        Ok(Self {
            root,
            git_dir,
            main_dir,
            naming,
        })
    }

    fn open_repo(&self) -> Result<Repository> {
        Ok(Repository::open(self.git_dir.to_native())?)
    }
}

impl LayoutProvider for ContainerLayout {
    fn git_database(&self) -> &NormalizedPath {
        &self.git_dir
    }

    fn main_worktree(&self) -> &NormalizedPath {
        &self.main_dir
    }

    fn feature_worktree(&self, name: &str) -> NormalizedPath {
        let dir_name = branch_to_directory(name, self.naming);
        self.root.join(&dir_name)
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let repo = self.open_repo()?;
        let worktree_names = repo.worktrees()?;

        let mut result = Vec::new();

        for name in worktree_names.iter() {
            let name = match name {
                Some(n) => n,
                None => continue,
            };

            let wt = repo.find_worktree(name)?;
            let wt_path = wt.path();

            // Get branch for this worktree
            let wt_repo = Repository::open(wt_path)?;
            let branch = wt_repo.head()
                .ok()
                .and_then(|h| h.shorthand().map(String::from))
                .unwrap_or_else(|| "HEAD".into());

            let is_main = name == "main" || wt_path.ends_with("main");

            result.push(WorktreeInfo {
                name: name.to_string(),
                path: NormalizedPath::new(wt_path),
                branch,
                is_main,
            });
        }

        Ok(result)
    }

    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath> {
        let repo = self.open_repo()?;
        let worktree_path = self.feature_worktree(name);
        let dir_name = branch_to_directory(name, self.naming);

        // Check if worktree already exists
        if worktree_path.exists() {
            return Err(Error::WorktreeExists {
                name: name.to_string(),
                path: worktree_path.to_native(),
            });
        }

        // Get base reference
        let base_ref = match base {
            Some(base_name) => {
                repo.find_branch(base_name, BranchType::Local)
                    .map_err(|_| Error::BranchNotFound { name: base_name.to_string() })?
                    .into_reference()
            }
            None => repo.head()?,
        };

        // Create worktree with new branch
        let mut opts = WorktreeAddOptions::new();
        opts.reference(Some(&base_ref));

        repo.worktree(
            &dir_name,
            worktree_path.to_native().as_path(),
            Some(&opts),
        )?;

        Ok(worktree_path)
    }

    fn remove_feature(&self, name: &str) -> Result<()> {
        let repo = self.open_repo()?;
        let dir_name = branch_to_directory(name, self.naming);

        // Find and remove worktree
        let wt = repo.find_worktree(&dir_name)
            .map_err(|_| Error::WorktreeNotFound { name: name.to_string() })?;

        // Prune the worktree (removes directory and git references)
        wt.prune(None)?;

        // Also try to delete the branch
        if let Ok(mut branch) = repo.find_branch(&dir_name, BranchType::Local) {
            let _ = branch.delete(); // Ignore error if branch doesn't exist
        }

        Ok(())
    }

    fn current_branch(&self) -> Result<String> {
        let repo = self.open_repo()?;
        let head = repo.head()?;

        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-git container_tests -- --test-threads=1
```
Expected: All tests PASS (--test-threads=1 for git operations)

**Step 5: Commit**

```bash
git add crates/repo-git/src/container.rs crates/repo-git/tests/container_tests.rs
git commit -m "feat(repo-git): implement ContainerLayout with git2 worktrees"
```

---

### Task 2.6: InRepoWorktrees Layout Implementation

**Files:**
- Create: `crates/repo-git/src/in_repo_worktrees.rs`
- Test: `crates/repo-git/tests/in_repo_worktrees_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-git/tests/in_repo_worktrees_tests.rs`:
```rust
use repo_git::in_repo_worktrees::InRepoWorktreesLayout;
use repo_git::provider::LayoutProvider;
use repo_git::NamingStrategy;
use repo_fs::NormalizedPath;
use std::process::Command;
use std::fs;
use tempfile::TempDir;

fn setup_in_repo_worktrees() -> (TempDir, InRepoWorktreesLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Initialize regular git repo
    Command::new("git")
        .args(["init"])
        .arg(root)
        .output()
        .expect("Failed to init repo");

    // Create initial commit
    fs::write(root.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["add", "README.md"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    // Create .worktrees directory
    fs::create_dir(root.join(".worktrees")).unwrap();

    let layout = InRepoWorktreesLayout::new(
        NormalizedPath::new(root),
        NamingStrategy::Slug,
    ).unwrap();

    (temp, layout)
}

#[test]
fn test_in_repo_git_database() {
    let (_temp, layout) = setup_in_repo_worktrees();
    assert!(layout.git_database().as_str().ends_with(".git"));
}

#[test]
fn test_in_repo_main_worktree_is_root() {
    let (temp, layout) = setup_in_repo_worktrees();
    let root_str = NormalizedPath::new(temp.path()).as_str().to_string();
    assert_eq!(layout.main_worktree().as_str(), root_str);
}

#[test]
fn test_in_repo_feature_worktree_path() {
    let (_temp, layout) = setup_in_repo_worktrees();
    let path = layout.feature_worktree("feat-auth");
    assert!(path.as_str().contains(".worktrees"));
    assert!(path.as_str().ends_with("feat-auth"));
}

#[test]
fn test_in_repo_create_and_remove_feature() {
    let (_temp, layout) = setup_in_repo_worktrees();

    // Create feature
    let path = layout.create_feature("test-feature", None).unwrap();
    assert!(path.exists());
    assert!(path.as_str().contains(".worktrees"));

    // Remove feature
    layout.remove_feature("test-feature").unwrap();
    assert!(!path.exists());
}
```

**Step 2: Run tests to verify they fail**

Run:
```bash
cargo test -p repo-git in_repo_worktrees_tests
```
Expected: FAIL - module not found

**Step 3: Write implementation**

Create `crates/repo-git/src/in_repo_worktrees.rs`:
```rust
//! In-repo worktrees layout implementation

use git2::{BranchType, Repository, WorktreeAddOptions};
use repo_fs::NormalizedPath;
use crate::{
    Error, Result,
    naming::{branch_to_directory, NamingStrategy},
    provider::{LayoutProvider, WorktreeInfo},
};

/// In-repo worktrees layout with `.worktrees/` directory.
///
/// ```text
/// {repo}/
/// ├── .git/          # Git database
/// ├── .worktrees/    # Worktrees folder
/// │   └── feature-x/
/// └── src/           # Main branch files
/// ```
pub struct InRepoWorktreesLayout {
    root: NormalizedPath,
    git_dir: NormalizedPath,
    worktrees_dir: NormalizedPath,
    naming: NamingStrategy,
}

impl InRepoWorktreesLayout {
    /// Create a new InRepoWorktreesLayout for the given root directory.
    pub fn new(root: NormalizedPath, naming: NamingStrategy) -> Result<Self> {
        let git_dir = root.join(".git");
        let worktrees_dir = root.join(".worktrees");

        Ok(Self {
            root,
            git_dir,
            worktrees_dir,
            naming,
        })
    }

    fn open_repo(&self) -> Result<Repository> {
        Ok(Repository::open(self.root.to_native())?)
    }
}

impl LayoutProvider for InRepoWorktreesLayout {
    fn git_database(&self) -> &NormalizedPath {
        &self.git_dir
    }

    fn main_worktree(&self) -> &NormalizedPath {
        &self.root
    }

    fn feature_worktree(&self, name: &str) -> NormalizedPath {
        let dir_name = branch_to_directory(name, self.naming);
        self.worktrees_dir.join(&dir_name)
    }

    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let repo = self.open_repo()?;

        // Start with main worktree (the repo root)
        let main_branch = self.current_branch().unwrap_or_else(|_| "main".into());
        let mut result = vec![WorktreeInfo {
            name: "main".into(),
            path: self.root.clone(),
            branch: main_branch,
            is_main: true,
        }];

        // Add linked worktrees
        let worktree_names = repo.worktrees()?;
        for name in worktree_names.iter() {
            let name = match name {
                Some(n) => n,
                None => continue,
            };

            let wt = repo.find_worktree(name)?;
            let wt_path = wt.path();

            let wt_repo = Repository::open(wt_path)?;
            let branch = wt_repo.head()
                .ok()
                .and_then(|h| h.shorthand().map(String::from))
                .unwrap_or_else(|| "HEAD".into());

            result.push(WorktreeInfo {
                name: name.to_string(),
                path: NormalizedPath::new(wt_path),
                branch,
                is_main: false,
            });
        }

        Ok(result)
    }

    fn create_feature(&self, name: &str, base: Option<&str>) -> Result<NormalizedPath> {
        let repo = self.open_repo()?;
        let worktree_path = self.feature_worktree(name);
        let dir_name = branch_to_directory(name, self.naming);

        // Check if worktree already exists
        if worktree_path.exists() {
            return Err(Error::WorktreeExists {
                name: name.to_string(),
                path: worktree_path.to_native(),
            });
        }

        // Ensure .worktrees directory exists
        std::fs::create_dir_all(self.worktrees_dir.to_native())
            .map_err(|e| Error::Fs(repo_fs::Error::io(&self.worktrees_dir.to_native(), e)))?;

        // Get base reference
        let base_ref = match base {
            Some(base_name) => {
                repo.find_branch(base_name, BranchType::Local)
                    .map_err(|_| Error::BranchNotFound { name: base_name.to_string() })?
                    .into_reference()
            }
            None => repo.head()?,
        };

        // Create worktree
        let mut opts = WorktreeAddOptions::new();
        opts.reference(Some(&base_ref));

        repo.worktree(
            &dir_name,
            worktree_path.to_native().as_path(),
            Some(&opts),
        )?;

        Ok(worktree_path)
    }

    fn remove_feature(&self, name: &str) -> Result<()> {
        let repo = self.open_repo()?;
        let dir_name = branch_to_directory(name, self.naming);

        let wt = repo.find_worktree(&dir_name)
            .map_err(|_| Error::WorktreeNotFound { name: name.to_string() })?;

        wt.prune(None)?;

        if let Ok(mut branch) = repo.find_branch(&dir_name, BranchType::Local) {
            let _ = branch.delete();
        }

        Ok(())
    }

    fn current_branch(&self) -> Result<String> {
        let repo = self.open_repo()?;
        let head = repo.head()?;

        if head.is_branch() {
            Ok(head.shorthand().unwrap_or("HEAD").to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run:
```bash
cargo test -p repo-git in_repo_worktrees_tests -- --test-threads=1
```
Expected: All tests PASS

**Step 5: Commit**

```bash
git add crates/repo-git/src/in_repo_worktrees.rs crates/repo-git/tests/in_repo_worktrees_tests.rs
git commit -m "feat(repo-git): implement InRepoWorktreesLayout"
```

---

### Task 2.7: Update lib.rs exports and verify repo-git compiles

**Files:**
- Modify: `crates/repo-git/src/lib.rs`

**Step 1: Update lib.rs with correct exports**

Update `crates/repo-git/src/lib.rs`:
```rust
//! Git abstraction for Repository Manager
//!
//! Supports multiple worktree layout styles through a unified interface.

pub mod error;
pub mod provider;
pub mod naming;
pub mod container;
pub mod in_repo_worktrees;
pub mod classic;

pub use error::{Error, Result};
pub use provider::{LayoutProvider, WorktreeInfo};
pub use naming::NamingStrategy;
pub use container::ContainerLayout;
pub use in_repo_worktrees::InRepoWorktreesLayout;
pub use classic::ClassicLayout;
```

**Step 2: Run all repo-git tests**

Run:
```bash
cargo test -p repo-git -- --test-threads=1
```
Expected: All tests PASS

**Step 3: Run full workspace tests**

Run:
```bash
cargo test -- --test-threads=1
```
Expected: All tests PASS

**Step 4: Commit**

```bash
git add crates/repo-git/src/lib.rs
git commit -m "feat(repo-git): finalize module exports"
```

---

## Phase 3: Integration and Documentation

### Task 3.1: Add workspace-level documentation

**Files:**
- Create: `README.md` (workspace root)

**Step 1: Write README**

Create `README.md`:
```markdown
# Repository Manager

A Rust-based CLI tool for orchestrating agentic development workspaces.

## Crates

- **repo-fs** - Filesystem abstraction with path normalization and atomic I/O
- **repo-git** - Git abstraction supporting multiple worktree layouts

## Layout Modes

The repository manager supports three layout modes:

### Container Layout
```
{container}/
├── .gt/          # Git database
├── main/         # Main branch worktree
└── feature-x/    # Feature worktree
```

### In-Repo Worktrees Layout
```
{repo}/
├── .git/
├── .worktrees/
│   └── feature-x/
└── src/
```

### Classic Layout
```
{repo}/
├── .git/
└── src/
```

## Development

```bash
# Run tests
cargo test -- --test-threads=1

# Check compilation
cargo check

# Build
cargo build
```

## License

MIT
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add workspace README"
```

---

### Task 3.2: Final verification

**Step 1: Run all tests**

Run:
```bash
cargo test -- --test-threads=1
```
Expected: All tests PASS

**Step 2: Run clippy**

Run:
```bash
cargo clippy -- -D warnings
```
Expected: No warnings

**Step 3: Check formatting**

Run:
```bash
cargo fmt --check
```
Expected: No formatting issues (or run `cargo fmt` to fix)

**Step 4: Final commit with all fixes**

```bash
git add -A
git commit -m "chore: final cleanup and formatting"
```

---

## Verification Checklist

### repo-fs
- [ ] `NormalizedPath` handles Windows/Unix paths
- [ ] `NormalizedPath` detects network paths
- [ ] `WorkspaceLayout::detect()` identifies all three modes
- [ ] Atomic writes work with locking
- [ ] ConfigStore loads/saves TOML, JSON, YAML

### repo-git
- [ ] `ClassicLayout` returns migration errors for feature ops
- [ ] `ContainerLayout` creates/removes worktrees
- [ ] `InRepoWorktreesLayout` creates/removes worktrees
- [ ] Branch slug naming works
- [ ] All layouts implement `LayoutProvider` trait

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| git2 worktree API differences | Tests use real git commands for setup |
| Windows path handling | NormalizedPath abstracts differences |
| Concurrent access | File locking with fs2 |

## Rollback

If implementation fails:
```bash
git reset --hard HEAD~N  # Where N is number of commits to undo
```
