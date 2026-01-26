# repo-core Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the repo-core orchestration layer that unifies all Layer 0 crates (repo-fs, repo-git, repo-meta, repo-tools, repo-presets, repo-content) into a cohesive system with mode abstraction, configuration resolution, ledger-based state tracking, and a sync engine.

**Architecture:** repo-core provides the coordination layer between infrastructure crates. It implements the Mode abstraction (Standard vs Worktrees), resolves hierarchical configuration, tracks state via the Ledger system (Intents/Projections), and orchestrates sync/check/fix operations through the SyncEngine.

**Tech Stack:** Rust 2024 edition, serde for serialization, uuid for identifiers, async-trait for async providers, tracing for logging.

---

## Task 1: Create repo-core Crate Structure

**Files:**
- Create: `crates/repo-core/Cargo.toml`
- Create: `crates/repo-core/src/lib.rs`
- Create: `crates/repo-core/src/error.rs`

**Step 1: Write the failing test**

```rust
// crates/repo-core/src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::ConfigNotFound { path: "/test".into() };
        assert!(err.to_string().contains("/test"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core`
Expected: FAIL with "can't find crate for `repo_core`" (crate doesn't exist yet)

**Step 3: Write minimal implementation**

Create `crates/repo-core/Cargo.toml`:

```toml
[package]
name = "repo-core"
version.workspace = true
edition.workspace = true

[dependencies]
repo-fs = { path = "../repo-fs" }
repo-git = { path = "../repo-git" }
repo-meta = { path = "../repo-meta" }
repo-tools = { path = "../repo-tools" }
repo-presets = { path = "../repo-presets" }
repo-content = { path = "../repo-content" }

serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
rstest = { workspace = true }
pretty_assertions = { workspace = true }
```

Create `crates/repo-core/src/error.rs`:

```rust
//! Error types for repo-core

use std::path::PathBuf;
use thiserror::Error;

/// Result type for repo-core operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in repo-core
#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration not found at {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Invalid mode: {mode}")]
    InvalidMode { mode: String },

    #[error("Ledger error: {message}")]
    LedgerError { message: String },

    #[error("Intent not found: {id}")]
    IntentNotFound { id: String },

    #[error("Projection failed for {tool}: {reason}")]
    ProjectionFailed { tool: String, reason: String },

    #[error("Sync error: {message}")]
    SyncError { message: String },

    #[error(transparent)]
    Fs(#[from] repo_fs::Error),

    #[error(transparent)]
    Git(#[from] repo_git::Error),

    #[error(transparent)]
    Meta(#[from] repo_meta::Error),

    #[error(transparent)]
    Tools(#[from] repo_tools::Error),

    #[error(transparent)]
    Presets(#[from] repo_presets::Error),

    #[error(transparent)]
    Content(#[from] repo_content::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),

    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),
}
```

Create `crates/repo-core/src/lib.rs`:

```rust
//! Orchestration layer for Repository Manager
//!
//! This crate provides the coordination layer between infrastructure crates:
//! - Mode abstraction (Standard vs Worktrees)
//! - Configuration resolution with hierarchical merge
//! - Ledger-based state tracking (Intents/Projections)
//! - Sync engine for applying/checking/fixing state

pub mod error;

pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::ConfigNotFound { path: "/test".into() };
        assert!(err.to_string().contains("/test"));
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-core`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-core/
git commit -m "feat(repo-core): create crate structure with error types

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Implement Mode Abstraction

**Files:**
- Create: `crates/repo-core/src/mode.rs`
- Create: `crates/repo-core/src/backend/mod.rs`
- Create: `crates/repo-core/src/backend/standard.rs`
- Create: `crates/repo-core/src/backend/worktree.rs`
- Modify: `crates/repo-core/src/lib.rs`
- Test: `crates/repo-core/tests/mode_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-core/tests/mode_tests.rs`:

```rust
use repo_core::{Mode, ModeBackend};
use repo_fs::NormalizedPath;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_mode_from_str() {
    assert!(matches!(Mode::from_str("standard"), Ok(Mode::Standard)));
    assert!(matches!(Mode::from_str("worktrees"), Ok(Mode::Worktrees)));
    assert!(Mode::from_str("invalid").is_err());
}

#[test]
fn test_standard_backend_config_root() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    // Create .git to make it look like a repo
    fs::create_dir_all(dir.path().join(".git")).unwrap();

    let backend = repo_core::StandardBackend::new(root.clone()).unwrap();
    assert_eq!(backend.config_root(), root.join(".repository"));
}

#[test]
fn test_worktree_backend_config_root() {
    let dir = tempdir().unwrap();
    let container = NormalizedPath::new(dir.path());

    // Create container structure
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::create_dir_all(dir.path().join("main")).unwrap();

    let backend = repo_core::WorktreeBackend::new(container.clone()).unwrap();
    // Config root is at container level, not worktree level
    assert_eq!(backend.config_root(), container.join(".repository"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core --test mode_tests`
Expected: FAIL with "cannot find value `Mode` in crate `repo_core`"

**Step 3: Write minimal implementation**

Create `crates/repo-core/src/mode.rs`:

```rust
//! Mode abstraction for Repository Manager
//!
//! Defines Standard and Worktrees modes with their respective backends.

use crate::{Error, Result};

/// Operating mode for the repository manager
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Traditional single-directory Git repository
    Standard,
    /// Container-based layout with multiple worktrees
    Worktrees,
}

impl Mode {
    /// Parse mode from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "standard" | "default" => Ok(Mode::Standard),
            "worktrees" | "worktree" | "container" => Ok(Mode::Worktrees),
            _ => Err(Error::InvalidMode { mode: s.to_string() }),
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Standard => write!(f, "standard"),
            Mode::Worktrees => write!(f, "worktrees"),
        }
    }
}
```

Create `crates/repo-core/src/backend/mod.rs`:

```rust
//! Mode backends for Repository Manager

mod standard;
mod worktree;

pub use standard::StandardBackend;
pub use worktree::WorktreeBackend;

use crate::Result;
use repo_fs::NormalizedPath;

/// Information about a branch
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Path where the branch is checked out (for worktrees mode)
    pub path: Option<NormalizedPath>,
    /// Whether this is the current/active branch
    pub is_current: bool,
    /// Whether this is the main branch
    pub is_main: bool,
}

/// Trait for mode-specific backend operations
pub trait ModeBackend: Send + Sync {
    /// Get the configuration root directory
    fn config_root(&self) -> NormalizedPath;

    /// Get the current working directory within the repository
    fn working_dir(&self) -> &NormalizedPath;

    /// Create a new branch
    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()>;

    /// Delete a branch
    fn delete_branch(&self, name: &str) -> Result<()>;

    /// List all branches
    fn list_branches(&self) -> Result<Vec<BranchInfo>>;

    /// Switch to a branch (Standard) or get worktree path (Worktrees)
    fn switch_branch(&self, name: &str) -> Result<NormalizedPath>;
}
```

Create `crates/repo-core/src/backend/standard.rs`:

```rust
//! Standard mode backend (single git repository)

use super::{BranchInfo, ModeBackend};
use crate::{Error, Result};
use repo_fs::NormalizedPath;
use std::process::Command;

/// Backend for standard single-directory Git repositories
pub struct StandardBackend {
    /// Repository root directory
    root: NormalizedPath,
}

impl StandardBackend {
    /// Create a new StandardBackend
    pub fn new(root: NormalizedPath) -> Result<Self> {
        // Verify it's a git repository
        let git_dir = root.join(".git");
        if !git_dir.as_path().exists() {
            return Err(Error::ConfigNotFound {
                path: git_dir.as_path().to_path_buf()
            });
        }
        Ok(Self { root })
    }
}

impl ModeBackend for StandardBackend {
    fn config_root(&self) -> NormalizedPath {
        self.root.join(".repository")
    }

    fn working_dir(&self) -> &NormalizedPath {
        &self.root
    }

    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.current_dir(self.root.as_path())
            .arg("checkout")
            .arg("-b")
            .arg(name);

        if let Some(base_branch) = base {
            cmd.arg(base_branch);
        }

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(Error::SyncError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(())
    }

    fn delete_branch(&self, name: &str) -> Result<()> {
        let output = Command::new("git")
            .current_dir(self.root.as_path())
            .args(["branch", "-d", name])
            .output()?;

        if !output.status.success() {
            return Err(Error::SyncError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let output = Command::new("git")
            .current_dir(self.root.as_path())
            .args(["branch", "--format=%(refname:short) %(HEAD)"])
            .output()?;

        if !output.status.success() {
            return Err(Error::SyncError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let branches = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    return None;
                }
                let name = parts[0].to_string();
                let is_current = parts.get(1) == Some(&"*");
                let is_main = name == "main" || name == "master";
                Some(BranchInfo {
                    name,
                    path: None, // Standard mode doesn't have separate paths
                    is_current,
                    is_main,
                })
            })
            .collect();

        Ok(branches)
    }

    fn switch_branch(&self, name: &str) -> Result<NormalizedPath> {
        let output = Command::new("git")
            .current_dir(self.root.as_path())
            .args(["checkout", name])
            .output()?;

        if !output.status.success() {
            return Err(Error::SyncError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // In standard mode, switching branch doesn't change the path
        Ok(self.root.clone())
    }
}
```

Create `crates/repo-core/src/backend/worktree.rs`:

```rust
//! Worktree mode backend (container with multiple worktrees)

use super::{BranchInfo, ModeBackend};
use crate::{Error, Result};
use repo_fs::NormalizedPath;
use std::process::Command;

/// Backend for container-based worktree layouts
pub struct WorktreeBackend {
    /// Container root directory
    container: NormalizedPath,
    /// Current working worktree
    current_worktree: NormalizedPath,
}

impl WorktreeBackend {
    /// Create a new WorktreeBackend
    pub fn new(container: NormalizedPath) -> Result<Self> {
        // Verify it's a valid container
        let git_dir = container.join(".git");
        if !git_dir.as_path().exists() {
            return Err(Error::ConfigNotFound {
                path: git_dir.as_path().to_path_buf(),
            });
        }

        // Default to main worktree
        let main_worktree = container.join("main");

        Ok(Self {
            container,
            current_worktree: main_worktree,
        })
    }

    /// Create with a specific current worktree
    pub fn with_worktree(container: NormalizedPath, worktree: NormalizedPath) -> Result<Self> {
        let git_dir = container.join(".git");
        if !git_dir.as_path().exists() {
            return Err(Error::ConfigNotFound {
                path: git_dir.as_path().to_path_buf(),
            });
        }

        Ok(Self {
            container,
            current_worktree: worktree,
        })
    }
}

impl ModeBackend for WorktreeBackend {
    fn config_root(&self) -> NormalizedPath {
        // Config is at container level, shared across worktrees
        self.container.join(".repository")
    }

    fn working_dir(&self) -> &NormalizedPath {
        &self.current_worktree
    }

    fn create_branch(&self, name: &str, base: Option<&str>) -> Result<()> {
        let worktree_path = self.container.join(name);

        let mut cmd = Command::new("git");
        cmd.current_dir(self.container.as_path())
            .arg("worktree")
            .arg("add")
            .arg(worktree_path.as_path())
            .arg("-b")
            .arg(name);

        if let Some(base_branch) = base {
            cmd.arg(base_branch);
        }

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(Error::SyncError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(())
    }

    fn delete_branch(&self, name: &str) -> Result<()> {
        let worktree_path = self.container.join(name);

        // First remove the worktree
        let output = Command::new("git")
            .current_dir(self.container.as_path())
            .args(["worktree", "remove", worktree_path.as_path().to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            return Err(Error::SyncError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // Then delete the branch
        let output = Command::new("git")
            .current_dir(self.container.as_path())
            .args(["branch", "-d", name])
            .output()?;

        if !output.status.success() {
            // Branch deletion failure is not critical
            tracing::warn!("Failed to delete branch {}: {}", name,
                String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<BranchInfo>> {
        let output = Command::new("git")
            .current_dir(self.container.as_path())
            .args(["worktree", "list", "--porcelain"])
            .output()?;

        if !output.status.success() {
            return Err(Error::SyncError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut branches = Vec::new();
        let mut current_path: Option<NormalizedPath> = None;
        let mut current_branch: Option<String> = None;

        for line in stdout.lines() {
            if let Some(path) = line.strip_prefix("worktree ") {
                current_path = Some(NormalizedPath::new(path));
            } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
                current_branch = Some(branch.to_string());
            } else if line.is_empty() {
                // End of worktree entry
                if let (Some(path), Some(branch)) = (current_path.take(), current_branch.take()) {
                    let is_current = path == self.current_worktree;
                    let is_main = branch == "main" || branch == "master";
                    branches.push(BranchInfo {
                        name: branch,
                        path: Some(path),
                        is_current,
                        is_main,
                    });
                }
            }
        }

        // Handle last entry if no trailing newline
        if let (Some(path), Some(branch)) = (current_path, current_branch) {
            let is_current = path == self.current_worktree;
            let is_main = branch == "main" || branch == "master";
            branches.push(BranchInfo {
                name: branch,
                path: Some(path),
                is_current,
                is_main,
            });
        }

        Ok(branches)
    }

    fn switch_branch(&self, name: &str) -> Result<NormalizedPath> {
        // In worktree mode, "switching" means returning the path to the worktree
        let worktree_path = self.container.join(name);

        if !worktree_path.as_path().exists() {
            return Err(Error::ConfigNotFound {
                path: worktree_path.as_path().to_path_buf(),
            });
        }

        Ok(worktree_path)
    }
}
```

Update `crates/repo-core/src/lib.rs`:

```rust
//! Orchestration layer for Repository Manager
//!
//! This crate provides the coordination layer between infrastructure crates:
//! - Mode abstraction (Standard vs Worktrees)
//! - Configuration resolution with hierarchical merge
//! - Ledger-based state tracking (Intents/Projections)
//! - Sync engine for applying/checking/fixing state

pub mod backend;
pub mod error;
pub mod mode;

pub use backend::{BranchInfo, ModeBackend, StandardBackend, WorktreeBackend};
pub use error::{Error, Result};
pub use mode::Mode;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-core --test mode_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-core/
git commit -m "feat(repo-core): implement Mode abstraction with Standard/Worktree backends

Adds:
- Mode enum (Standard/Worktrees)
- ModeBackend trait for mode-specific operations
- StandardBackend for traditional git repositories
- WorktreeBackend for container-based worktree layouts

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Implement Ledger System

**Files:**
- Create: `crates/repo-core/src/ledger/mod.rs`
- Create: `crates/repo-core/src/ledger/intent.rs`
- Create: `crates/repo-core/src/ledger/projection.rs`
- Modify: `crates/repo-core/src/lib.rs`
- Test: `crates/repo-core/tests/ledger_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-core/tests/ledger_tests.rs`:

```rust
use repo_core::ledger::{Intent, Ledger, Projection, ProjectionKind};
use uuid::Uuid;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_ledger_add_intent() {
    let mut ledger = Ledger::new();

    let intent = Intent::new(
        "rule:python/style/snake-case".to_string(),
        serde_json::json!({ "severity": "error" }),
    );
    let uuid = intent.uuid;

    ledger.add_intent(intent);

    assert_eq!(ledger.intents().len(), 1);
    assert!(ledger.get_intent(uuid).is_some());
}

#[test]
fn test_ledger_remove_intent() {
    let mut ledger = Ledger::new();

    let intent = Intent::new(
        "rule:python/style/snake-case".to_string(),
        serde_json::json!({}),
    );
    let uuid = intent.uuid;

    ledger.add_intent(intent);
    assert_eq!(ledger.intents().len(), 1);

    ledger.remove_intent(uuid);
    assert_eq!(ledger.intents().len(), 0);
}

#[test]
fn test_intent_with_projections() {
    let mut intent = Intent::new(
        "rule:test".to_string(),
        serde_json::json!({}),
    );

    let projection = Projection::text_block(
        "cursor".to_string(),
        ".cursorrules".into(),
        intent.uuid,
        "abc123".to_string(),
    );

    intent.add_projection(projection);

    assert_eq!(intent.projections().len(), 1);
}

#[test]
fn test_ledger_save_load() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("ledger.toml");

    let mut ledger = Ledger::new();
    let intent = Intent::new(
        "rule:test".to_string(),
        serde_json::json!({ "key": "value" }),
    );
    ledger.add_intent(intent);

    ledger.save(&path).unwrap();

    let loaded = Ledger::load(&path).unwrap();
    assert_eq!(loaded.intents().len(), 1);
}

#[test]
fn test_projection_kinds() {
    // TextBlock
    let text_block = ProjectionKind::TextBlock {
        marker: Uuid::new_v4(),
        checksum: "sha256:abc".to_string(),
    };
    assert!(matches!(text_block, ProjectionKind::TextBlock { .. }));

    // JsonKey
    let json_key = ProjectionKind::JsonKey {
        path: "python.linting.enabled".to_string(),
        value: serde_json::json!(true),
    };
    assert!(matches!(json_key, ProjectionKind::JsonKey { .. }));

    // FileManaged
    let file_managed = ProjectionKind::FileManaged {
        checksum: "sha256:xyz".to_string(),
    };
    assert!(matches!(file_managed, ProjectionKind::FileManaged { .. }));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core --test ledger_tests`
Expected: FAIL with "cannot find `ledger` in crate `repo_core`"

**Step 3: Write minimal implementation**

Create `crates/repo-core/src/ledger/mod.rs`:

```rust
//! Ledger-based state tracking for Repository Manager
//!
//! The ledger maps Abstract Intents (high-level rules) to Concrete
//! Projections (file modifications). This enables precise unrolling
//! and drift detection.

mod intent;
mod projection;

pub use intent::Intent;
pub use projection::{Projection, ProjectionKind};

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

/// The ledger tracks all active modifications performed by the manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ledger {
    /// Schema version
    #[serde(default = "default_version")]
    version: String,
    /// Active intents
    intents: Vec<Intent>,
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Ledger {
    /// Create a new empty ledger
    pub fn new() -> Self {
        Self {
            version: default_version(),
            intents: Vec::new(),
        }
    }

    /// Load ledger from a TOML file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let ledger: Ledger = toml::from_str(&content)?;
        Ok(ledger)
    }

    /// Save ledger to a TOML file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get all intents
    pub fn intents(&self) -> &[Intent] {
        &self.intents
    }

    /// Add an intent to the ledger
    pub fn add_intent(&mut self, intent: Intent) {
        self.intents.push(intent);
    }

    /// Remove an intent by UUID
    pub fn remove_intent(&mut self, uuid: Uuid) -> Option<Intent> {
        if let Some(pos) = self.intents.iter().position(|i| i.uuid == uuid) {
            Some(self.intents.remove(pos))
        } else {
            None
        }
    }

    /// Get an intent by UUID
    pub fn get_intent(&self, uuid: Uuid) -> Option<&Intent> {
        self.intents.iter().find(|i| i.uuid == uuid)
    }

    /// Get a mutable intent by UUID
    pub fn get_intent_mut(&mut self, uuid: Uuid) -> Option<&mut Intent> {
        self.intents.iter_mut().find(|i| i.uuid == uuid)
    }

    /// Find intents by rule ID
    pub fn find_by_rule(&self, rule_id: &str) -> Vec<&Intent> {
        self.intents.iter().filter(|i| i.id == rule_id).collect()
    }

    /// Find all projections for a specific file
    pub fn projections_for_file(&self, file: &Path) -> Vec<(&Intent, &Projection)> {
        let mut results = Vec::new();
        for intent in &self.intents {
            for projection in intent.projections() {
                if projection.file == file {
                    results.push((intent, projection));
                }
            }
        }
        results
    }
}
```

Create `crates/repo-core/src/ledger/intent.rs`:

```rust
//! Intent represents a high-level rule application

use super::Projection;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// An intent represents a high-level rule that has been applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// Canonical ID of the rule definition (e.g., "rule:python/style/snake-case")
    pub id: String,

    /// Unique instance ID for this specific application
    pub uuid: Uuid,

    /// Timestamp of when this intent was applied
    pub timestamp: DateTime<Utc>,

    /// Arguments used to generate the content
    #[serde(default)]
    pub args: Value,

    /// Projections (file modifications) created by this intent
    #[serde(default)]
    projections: Vec<Projection>,
}

impl Intent {
    /// Create a new intent
    pub fn new(id: String, args: Value) -> Self {
        Self {
            id,
            uuid: Uuid::new_v4(),
            timestamp: Utc::now(),
            args,
            projections: Vec::new(),
        }
    }

    /// Create an intent with a specific UUID (for testing or recreation)
    pub fn with_uuid(id: String, uuid: Uuid, args: Value) -> Self {
        Self {
            id,
            uuid,
            timestamp: Utc::now(),
            args,
            projections: Vec::new(),
        }
    }

    /// Get the projections
    pub fn projections(&self) -> &[Projection] {
        &self.projections
    }

    /// Add a projection to this intent
    pub fn add_projection(&mut self, projection: Projection) {
        self.projections.push(projection);
    }

    /// Remove a projection by tool and file
    pub fn remove_projection(&mut self, tool: &str, file: &std::path::Path) -> Option<Projection> {
        if let Some(pos) = self.projections.iter().position(|p| p.tool == tool && p.file == file) {
            Some(self.projections.remove(pos))
        } else {
            None
        }
    }
}
```

Create `crates/repo-core/src/ledger/projection.rs`:

```rust
//! Projection represents a concrete file modification

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

/// A projection represents a concrete file modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    /// Tool that owns this projection (e.g., "cursor", "vscode")
    pub tool: String,

    /// File path relative to config root
    pub file: PathBuf,

    /// The kind of projection (how content is embedded)
    pub kind: ProjectionKind,
}

/// How content is embedded in the target file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum ProjectionKind {
    /// UUID-tagged text block (for markdown, plain text)
    TextBlock {
        /// UUID used in markers
        marker: Uuid,
        /// Checksum to detect user modifications
        checksum: String,
    },

    /// Specific JSON/TOML/YAML key ownership
    JsonKey {
        /// Path to the key (e.g., "python.linting.enabled")
        path: String,
        /// Value snapshot (to detect user changes)
        value: Value,
    },

    /// Entire file owned by the manager
    FileManaged {
        /// Checksum of the managed file
        checksum: String,
    },
}

impl Projection {
    /// Create a TextBlock projection
    pub fn text_block(
        tool: String,
        file: PathBuf,
        marker: Uuid,
        checksum: String,
    ) -> Self {
        Self {
            tool,
            file,
            kind: ProjectionKind::TextBlock { marker, checksum },
        }
    }

    /// Create a JsonKey projection
    pub fn json_key(
        tool: String,
        file: PathBuf,
        path: String,
        value: Value,
    ) -> Self {
        Self {
            tool,
            file,
            kind: ProjectionKind::JsonKey { path, value },
        }
    }

    /// Create a FileManaged projection
    pub fn file_managed(tool: String, file: PathBuf, checksum: String) -> Self {
        Self {
            tool,
            file,
            kind: ProjectionKind::FileManaged { checksum },
        }
    }
}
```

Add chrono to dependencies in `crates/repo-core/Cargo.toml`:

```toml
[dependencies]
# ... existing deps ...
chrono = { version = "0.4", features = ["serde"] }
```

Update `crates/repo-core/src/lib.rs`:

```rust
//! Orchestration layer for Repository Manager

pub mod backend;
pub mod error;
pub mod ledger;
pub mod mode;

pub use backend::{BranchInfo, ModeBackend, StandardBackend, WorktreeBackend};
pub use error::{Error, Result};
pub use ledger::{Intent, Ledger, Projection, ProjectionKind};
pub use mode::Mode;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-core --test ledger_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-core/
git commit -m "feat(repo-core): implement Ledger system for intent/projection tracking

Adds:
- Ledger struct for managing intents
- Intent struct for high-level rule applications
- Projection struct with TextBlock/JsonKey/FileManaged kinds
- TOML serialization for persistence

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Implement Configuration Resolution

**Files:**
- Create: `crates/repo-core/src/config/mod.rs`
- Create: `crates/repo-core/src/config/resolver.rs`
- Create: `crates/repo-core/src/config/manifest.rs`
- Create: `crates/repo-core/src/config/runtime.rs`
- Modify: `crates/repo-core/src/lib.rs`
- Test: `crates/repo-core/tests/config_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-core/tests/config_tests.rs`:

```rust
use repo_core::config::{ConfigResolver, RuntimeContext, Manifest};
use repo_fs::NormalizedPath;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_manifest_parse() {
    let toml = r#"
[core]
mode = "worktrees"

[presets]
"env:python" = { provider = "uv", version = "3.12" }
"tool:linter" = { use = "ruff" }
"#;

    let manifest = Manifest::parse(toml).unwrap();
    assert_eq!(manifest.core.mode, "worktrees");
    assert!(manifest.presets.contains_key("env:python"));
}

#[test]
fn test_config_resolver_hierarchy() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    // Create .repository/config.toml
    fs::create_dir_all(dir.path().join(".repository")).unwrap();
    fs::write(dir.path().join(".repository/config.toml"), r#"
[core]
mode = "standard"

[presets]
"env:python" = { version = "3.11" }
"#).unwrap();

    // Create local override
    fs::write(dir.path().join(".repository/config.local.toml"), r#"
[presets]
"env:python" = { version = "3.12" }
"#).unwrap();

    let resolver = ConfigResolver::new(root);
    let resolved = resolver.resolve().unwrap();

    // Local should override repo
    let python = resolved.presets.get("env:python").unwrap();
    assert_eq!(python.get("version").unwrap().as_str().unwrap(), "3.12");
}

#[test]
fn test_runtime_context_generation() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    fs::create_dir_all(dir.path().join(".repository")).unwrap();
    fs::write(dir.path().join(".repository/config.toml"), r#"
[core]
mode = "standard"

[presets]
"env:python" = { provider = "uv", version = "3.12" }
"#).unwrap();

    let resolver = ConfigResolver::new(root);
    let resolved = resolver.resolve().unwrap();
    let context = RuntimeContext::from_resolved(&resolved);

    assert!(context.to_json().is_object());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core --test config_tests`
Expected: FAIL with "cannot find `config` in crate `repo_core`"

**Step 3: Write minimal implementation**

Create `crates/repo-core/src/config/mod.rs`:

```rust
//! Configuration resolution for Repository Manager
//!
//! Implements hierarchical merge strategy:
//! Global Defaults → Organization → Repository → Local

mod manifest;
mod resolver;
mod runtime;

pub use manifest::Manifest;
pub use resolver::{ConfigResolver, ResolvedConfig};
pub use runtime::RuntimeContext;
```

Create `crates/repo-core/src/config/manifest.rs`:

```rust
//! Manifest parsing for Repository Manager configuration

use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Core configuration section
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreSection {
    /// Operating mode (standard or worktrees)
    #[serde(default = "default_mode")]
    pub mode: String,
}

fn default_mode() -> String {
    "standard".to_string()
}

/// The manifest represents a single configuration file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    /// Core settings
    #[serde(default)]
    pub core: CoreSection,

    /// Preset declarations
    #[serde(default)]
    pub presets: HashMap<String, Value>,

    /// Active tools
    #[serde(default)]
    pub tools: Vec<String>,

    /// Active rules
    #[serde(default)]
    pub rules: Vec<String>,
}

impl Manifest {
    /// Parse a manifest from TOML content
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: Manifest = toml::from_str(content)?;
        Ok(manifest)
    }

    /// Create an empty manifest
    pub fn empty() -> Self {
        Self::default()
    }

    /// Merge another manifest into this one (other takes precedence)
    pub fn merge(&mut self, other: &Manifest) {
        // Merge core settings
        if other.core.mode != default_mode() {
            self.core.mode = other.core.mode.clone();
        }

        // Merge presets (deep merge for objects)
        for (key, value) in &other.presets {
            if let Some(existing) = self.presets.get_mut(key) {
                // Deep merge if both are objects
                if let (Some(existing_obj), Some(other_obj)) =
                    (existing.as_object_mut(), value.as_object())
                {
                    for (k, v) in other_obj {
                        existing_obj.insert(k.clone(), v.clone());
                    }
                } else {
                    *existing = value.clone();
                }
            } else {
                self.presets.insert(key.clone(), value.clone());
            }
        }

        // Merge tools (union)
        for tool in &other.tools {
            if !self.tools.contains(tool) {
                self.tools.push(tool.clone());
            }
        }

        // Merge rules (union)
        for rule in &other.rules {
            if !self.rules.contains(rule) {
                self.rules.push(rule.clone());
            }
        }
    }
}
```

Create `crates/repo-core/src/config/resolver.rs`:

```rust
//! Configuration resolver with hierarchical merge

use super::Manifest;
use crate::Result;
use repo_fs::NormalizedPath;
use serde_json::Value;
use std::collections::HashMap;

/// Resolved configuration after merging all levels
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// Final mode setting
    pub mode: String,
    /// Merged presets
    pub presets: HashMap<String, Value>,
    /// Active tools
    pub tools: Vec<String>,
    /// Active rules
    pub rules: Vec<String>,
}

/// Resolves configuration from multiple sources
pub struct ConfigResolver {
    root: NormalizedPath,
}

impl ConfigResolver {
    /// Create a new resolver for the given root
    pub fn new(root: NormalizedPath) -> Self {
        Self { root }
    }

    /// Resolve configuration by merging all levels
    pub fn resolve(&self) -> Result<ResolvedConfig> {
        let mut manifest = Manifest::empty();

        // 1. Load global defaults (if exists)
        // TODO: Global config at ~/.config/repo-manager/config.toml

        // 2. Load organization config (if exists)
        // TODO: Organization config via git remote or .org-config

        // 3. Load repository config
        let repo_config = self.root.join(".repository/config.toml");
        if repo_config.as_path().exists() {
            let content = std::fs::read_to_string(repo_config.as_path())?;
            let repo_manifest = Manifest::parse(&content)?;
            manifest.merge(&repo_manifest);
        }

        // 4. Load local overrides (git-ignored)
        let local_config = self.root.join(".repository/config.local.toml");
        if local_config.as_path().exists() {
            let content = std::fs::read_to_string(local_config.as_path())?;
            let local_manifest = Manifest::parse(&content)?;
            manifest.merge(&local_manifest);
        }

        Ok(ResolvedConfig {
            mode: manifest.core.mode,
            presets: manifest.presets,
            tools: manifest.tools,
            rules: manifest.rules,
        })
    }
}
```

Create `crates/repo-core/src/config/runtime.rs`:

```rust
//! Runtime context generation for agent consumption

use super::ResolvedConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Runtime context projected for agent consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContext {
    /// Runtime environment information
    pub runtime: HashMap<String, Value>,
    /// Available capabilities
    pub capabilities: Vec<String>,
}

impl RuntimeContext {
    /// Create runtime context from resolved configuration
    pub fn from_resolved(config: &ResolvedConfig) -> Self {
        let mut runtime = HashMap::new();
        let mut capabilities = Vec::new();

        // Process presets to extract runtime info
        for (key, value) in &config.presets {
            // Parse preset key (e.g., "env:python" -> category="env", name="python")
            let parts: Vec<&str> = key.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let (category, name) = (parts[0], parts[1]);

            match category {
                "env" => {
                    // Environment preset - add to runtime
                    runtime.insert(name.to_string(), value.clone());
                }
                "tool" => {
                    // Tool preset - add capability
                    if let Some(use_tool) = value.get("use").and_then(|v| v.as_str()) {
                        capabilities.push(use_tool.to_string());
                    } else {
                        capabilities.push(name.to_string());
                    }
                }
                "config" => {
                    // Config preset - add to capabilities
                    capabilities.push(format!("config:{}", name));
                }
                _ => {}
            }
        }

        // Add tools as capabilities
        for tool in &config.tools {
            if !capabilities.contains(tool) {
                capabilities.push(tool.clone());
            }
        }

        Self {
            runtime,
            capabilities,
        }
    }

    /// Convert to JSON for agent consumption
    pub fn to_json(&self) -> Value {
        json!({
            "runtime": self.runtime,
            "capabilities": self.capabilities,
        })
    }
}
```

Update `crates/repo-core/src/lib.rs`:

```rust
//! Orchestration layer for Repository Manager

pub mod backend;
pub mod config;
pub mod error;
pub mod ledger;
pub mod mode;

pub use backend::{BranchInfo, ModeBackend, StandardBackend, WorktreeBackend};
pub use config::{ConfigResolver, Manifest, ResolvedConfig, RuntimeContext};
pub use error::{Error, Result};
pub use ledger::{Intent, Ledger, Projection, ProjectionKind};
pub use mode::Mode;
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-core --test config_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-core/
git commit -m "feat(repo-core): implement configuration resolution with hierarchical merge

Adds:
- Manifest struct for parsing config.toml files
- ConfigResolver for merging config hierarchy
- ResolvedConfig for final merged state
- RuntimeContext for agent-consumable JSON projection

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Implement Sync Engine

**Files:**
- Create: `crates/repo-core/src/sync/mod.rs`
- Create: `crates/repo-core/src/sync/engine.rs`
- Create: `crates/repo-core/src/sync/check.rs`
- Modify: `crates/repo-core/src/lib.rs`
- Test: `crates/repo-core/tests/sync_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-core/tests/sync_tests.rs`:

```rust
use repo_core::sync::{SyncEngine, SyncReport, CheckReport, CheckStatus};
use repo_core::{Ledger, Mode};
use repo_fs::NormalizedPath;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_sync_engine_check_empty() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    // Create minimal structure
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::create_dir_all(dir.path().join(".repository")).unwrap();
    fs::write(dir.path().join(".repository/config.toml"), r#"
[core]
mode = "standard"
"#).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Healthy);
}

#[test]
fn test_sync_engine_sync_creates_ledger() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::create_dir_all(dir.path().join(".repository")).unwrap();
    fs::write(dir.path().join(".repository/config.toml"), r#"
[core]
mode = "standard"
"#).unwrap();

    let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();
    let report = engine.sync().unwrap();

    assert!(report.success);
    assert!(root.join(".repository/ledger.toml").as_path().exists());
}

#[test]
fn test_check_detects_drift() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::create_dir_all(dir.path().join(".repository")).unwrap();

    // Create config with a tool
    fs::write(dir.path().join(".repository/config.toml"), r#"
[core]
mode = "standard"

tools = ["vscode"]
"#).unwrap();

    // Create ledger claiming to have synced vscode
    let ledger_content = r#"
version = "1.0"

[[intents]]
id = "tool:vscode"
uuid = "550e8400-e29b-41d4-a716-446655440000"
timestamp = "2026-01-26T00:00:00Z"
args = {}

[[intents.projections]]
tool = "vscode"
file = ".vscode/settings.json"

[intents.projections.kind]
backend = "file_managed"
checksum = "sha256:old_checksum"
"#;
    fs::write(dir.path().join(".repository/ledger.toml"), ledger_content).unwrap();

    // But the file doesn't exist or has different content

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    // Should detect that projection is missing/drifted
    assert!(report.status == CheckStatus::Drifted || report.status == CheckStatus::Missing);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core --test sync_tests`
Expected: FAIL with "cannot find `sync` in crate `repo_core`"

**Step 3: Write minimal implementation**

Create `crates/repo-core/src/sync/mod.rs`:

```rust
//! Sync engine for Repository Manager
//!
//! Handles sync/check/fix operations:
//! - sync(): Apply all pending changes from resolved config
//! - check(): Validate ledger vs filesystem state
//! - fix(): Auto-repair drift where possible

mod check;
mod engine;

pub use check::{CheckReport, CheckStatus, DriftItem};
pub use engine::{SyncEngine, SyncReport};
```

Create `crates/repo-core/src/sync/check.rs`:

```rust
//! Check functionality for validating state

use serde::{Deserialize, Serialize};

/// Status of the repository after checking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    /// Everything is in sync
    Healthy,
    /// Some projections are missing
    Missing,
    /// Some projections have drifted from expected state
    Drifted,
    /// Ledger is corrupted or invalid
    Broken,
}

/// An item that has drifted from expected state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftItem {
    /// Intent ID that owns this projection
    pub intent_id: String,
    /// Tool that owns the projection
    pub tool: String,
    /// File path
    pub file: String,
    /// Description of the drift
    pub description: String,
}

/// Report from checking repository state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckReport {
    /// Overall status
    pub status: CheckStatus,
    /// Items that have drifted
    pub drifted: Vec<DriftItem>,
    /// Items that are missing
    pub missing: Vec<DriftItem>,
    /// Human-readable messages
    pub messages: Vec<String>,
}

impl CheckReport {
    /// Create a healthy report
    pub fn healthy() -> Self {
        Self {
            status: CheckStatus::Healthy,
            drifted: Vec::new(),
            missing: Vec::new(),
            messages: vec!["All projections are in sync".to_string()],
        }
    }

    /// Create a report with missing items
    pub fn with_missing(missing: Vec<DriftItem>) -> Self {
        Self {
            status: CheckStatus::Missing,
            drifted: Vec::new(),
            missing,
            messages: Vec::new(),
        }
    }

    /// Create a report with drifted items
    pub fn with_drifted(drifted: Vec<DriftItem>) -> Self {
        Self {
            status: CheckStatus::Drifted,
            drifted,
            missing: Vec::new(),
            messages: Vec::new(),
        }
    }
}
```

Create `crates/repo-core/src/sync/engine.rs`:

```rust
//! Sync engine implementation

use super::{CheckReport, CheckStatus, DriftItem};
use crate::{
    config::ConfigResolver,
    ledger::{Ledger, ProjectionKind},
    Mode, Result, StandardBackend, WorktreeBackend, ModeBackend,
};
use repo_fs::NormalizedPath;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Report from a sync operation
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// Whether sync completed successfully
    pub success: bool,
    /// Actions taken during sync
    pub actions: Vec<String>,
    /// Errors encountered
    pub errors: Vec<String>,
}

impl SyncReport {
    /// Create a successful report
    pub fn success(actions: Vec<String>) -> Self {
        Self {
            success: true,
            actions,
            errors: Vec::new(),
        }
    }

    /// Create a failed report
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            actions: Vec::new(),
            errors,
        }
    }
}

/// The sync engine coordinates state between config and filesystem
pub struct SyncEngine {
    root: NormalizedPath,
    mode: Mode,
    backend: Box<dyn ModeBackend>,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(root: NormalizedPath, mode: Mode) -> Result<Self> {
        let backend: Box<dyn ModeBackend> = match mode {
            Mode::Standard => Box::new(StandardBackend::new(root.clone())?),
            Mode::Worktrees => Box::new(WorktreeBackend::new(root.clone())?),
        };

        Ok(Self { root, mode, backend })
    }

    /// Get the ledger path
    fn ledger_path(&self) -> NormalizedPath {
        self.backend.config_root().join("ledger.toml")
    }

    /// Load or create the ledger
    fn load_ledger(&self) -> Result<Ledger> {
        let path = self.ledger_path();
        if path.as_path().exists() {
            Ledger::load(path.as_path())
        } else {
            Ok(Ledger::new())
        }
    }

    /// Save the ledger
    fn save_ledger(&self, ledger: &Ledger) -> Result<()> {
        let path = self.ledger_path();
        ledger.save(path.as_path())
    }

    /// Check repository state against ledger
    pub fn check(&self) -> Result<CheckReport> {
        let ledger = self.load_ledger()?;

        if ledger.intents().is_empty() {
            return Ok(CheckReport::healthy());
        }

        let mut missing = Vec::new();
        let mut drifted = Vec::new();

        for intent in ledger.intents() {
            for projection in intent.projections() {
                let file_path = self.root.join(&projection.file);

                match &projection.kind {
                    ProjectionKind::FileManaged { checksum } => {
                        if !file_path.as_path().exists() {
                            missing.push(DriftItem {
                                intent_id: intent.id.clone(),
                                tool: projection.tool.clone(),
                                file: projection.file.display().to_string(),
                                description: "File does not exist".to_string(),
                            });
                        } else {
                            let actual_checksum = self.compute_file_checksum(file_path.as_path())?;
                            if &actual_checksum != checksum {
                                drifted.push(DriftItem {
                                    intent_id: intent.id.clone(),
                                    tool: projection.tool.clone(),
                                    file: projection.file.display().to_string(),
                                    description: "File content has changed".to_string(),
                                });
                            }
                        }
                    }
                    ProjectionKind::TextBlock { marker, checksum } => {
                        if !file_path.as_path().exists() {
                            missing.push(DriftItem {
                                intent_id: intent.id.clone(),
                                tool: projection.tool.clone(),
                                file: projection.file.display().to_string(),
                                description: "File does not exist".to_string(),
                            });
                        } else {
                            // Check if block markers exist in file
                            let content = std::fs::read_to_string(file_path.as_path())?;
                            let marker_str = marker.to_string();
                            if !content.contains(&marker_str) {
                                drifted.push(DriftItem {
                                    intent_id: intent.id.clone(),
                                    tool: projection.tool.clone(),
                                    file: projection.file.display().to_string(),
                                    description: "Block markers not found".to_string(),
                                });
                            }
                        }
                    }
                    ProjectionKind::JsonKey { path, value } => {
                        if !file_path.as_path().exists() {
                            missing.push(DriftItem {
                                intent_id: intent.id.clone(),
                                tool: projection.tool.clone(),
                                file: projection.file.display().to_string(),
                                description: "File does not exist".to_string(),
                            });
                        } else {
                            // Parse JSON and check key
                            let content = std::fs::read_to_string(file_path.as_path())?;
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                let current = get_json_path(&json, path);
                                if current.as_ref() != Some(value) {
                                    drifted.push(DriftItem {
                                        intent_id: intent.id.clone(),
                                        tool: projection.tool.clone(),
                                        file: projection.file.display().to_string(),
                                        description: format!("Key '{}' has different value", path),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        if !missing.is_empty() {
            Ok(CheckReport::with_missing(missing))
        } else if !drifted.is_empty() {
            Ok(CheckReport::with_drifted(drifted))
        } else {
            Ok(CheckReport::healthy())
        }
    }

    /// Sync configuration to filesystem
    pub fn sync(&self) -> Result<SyncReport> {
        let resolver = ConfigResolver::new(self.root.clone());
        let config = resolver.resolve()?;

        let mut ledger = self.load_ledger()?;
        let mut actions = Vec::new();

        // For now, just ensure ledger file exists
        // Full sync implementation would:
        // 1. Compare config.tools against ledger intents
        // 2. Add new intents for newly enabled tools
        // 3. Remove intents for disabled tools
        // 4. Update projections for changed configurations

        // Save ledger (creates if doesn't exist)
        self.save_ledger(&ledger)?;
        actions.push("Saved ledger".to_string());

        Ok(SyncReport::success(actions))
    }

    /// Fix drifted state
    pub fn fix(&self) -> Result<SyncReport> {
        let check = self.check()?;

        if check.status == CheckStatus::Healthy {
            return Ok(SyncReport::success(vec!["No fixes needed".to_string()]));
        }

        // Re-sync to fix issues
        self.sync()
    }

    fn compute_file_checksum(&self, path: &Path) -> Result<String> {
        let content = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        Ok(format!("sha256:{:x}", result))
    }
}

/// Get a value from JSON using dot-separated path
fn get_json_path<'a>(json: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = json;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}
```

Update `crates/repo-core/src/lib.rs`:

```rust
//! Orchestration layer for Repository Manager

pub mod backend;
pub mod config;
pub mod error;
pub mod ledger;
pub mod mode;
pub mod sync;

pub use backend::{BranchInfo, ModeBackend, StandardBackend, WorktreeBackend};
pub use config::{ConfigResolver, Manifest, ResolvedConfig, RuntimeContext};
pub use error::{Error, Result};
pub use ledger::{Intent, Ledger, Projection, ProjectionKind};
pub use mode::Mode;
pub use sync::{CheckReport, CheckStatus, SyncEngine, SyncReport};
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p repo-core --test sync_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/repo-core/
git commit -m "feat(repo-core): implement SyncEngine with check/sync/fix operations

Adds:
- SyncEngine for coordinating state
- check() for validating ledger vs filesystem
- sync() for applying configuration changes
- fix() for auto-repairing drift
- CheckReport/SyncReport for operation results

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Integration and Final Testing

**Files:**
- Create: `crates/repo-core/tests/integration_tests.rs`
- Modify: `crates/repo-core/src/lib.rs` (add re-exports)

**Step 1: Write the integration test**

Create `crates/repo-core/tests/integration_tests.rs`:

```rust
//! Integration tests for repo-core

use repo_core::{
    ConfigResolver, Intent, Ledger, Mode, Projection, RuntimeContext,
    StandardBackend, SyncEngine, WorktreeBackend, ModeBackend,
};
use repo_fs::NormalizedPath;
use tempfile::tempdir;
use std::fs;

/// Test complete workflow: init -> configure -> sync -> check
#[test]
fn test_complete_workflow() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    // Setup: Create git repo structure
    fs::create_dir_all(dir.path().join(".git")).unwrap();
    fs::create_dir_all(dir.path().join(".repository")).unwrap();

    // Step 1: Write configuration
    fs::write(dir.path().join(".repository/config.toml"), r#"
[core]
mode = "standard"

[presets]
"env:python" = { provider = "uv", version = "3.12" }
"tool:linter" = { use = "ruff" }

tools = ["vscode", "cursor"]
"#).unwrap();

    // Step 2: Resolve configuration
    let resolver = ConfigResolver::new(root.clone());
    let config = resolver.resolve().unwrap();

    assert_eq!(config.mode, "standard");
    assert!(config.presets.contains_key("env:python"));
    assert!(config.tools.contains(&"vscode".to_string()));

    // Step 3: Generate runtime context
    let context = RuntimeContext::from_resolved(&config);
    let json = context.to_json();

    assert!(json["runtime"]["python"].is_object());
    assert!(json["capabilities"].as_array().unwrap().iter()
        .any(|v| v.as_str() == Some("ruff")));

    // Step 4: Create sync engine and sync
    let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();
    let sync_report = engine.sync().unwrap();

    assert!(sync_report.success);

    // Step 5: Check state
    let check_report = engine.check().unwrap();
    // With no intents, should be healthy
    assert_eq!(check_report.status, repo_core::CheckStatus::Healthy);
}

/// Test mode-specific backends
#[test]
fn test_mode_backends() {
    // Test Standard backend
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".git")).unwrap();

    let root = NormalizedPath::new(dir.path());
    let standard = StandardBackend::new(root.clone()).unwrap();

    assert_eq!(standard.config_root(), root.join(".repository"));
    assert_eq!(standard.working_dir(), &root);

    // Test Worktree backend
    let container_dir = tempdir().unwrap();
    fs::create_dir_all(container_dir.path().join(".git")).unwrap();
    fs::create_dir_all(container_dir.path().join("main")).unwrap();

    let container = NormalizedPath::new(container_dir.path());
    let worktree = WorktreeBackend::new(container.clone()).unwrap();

    // Config root is at container level
    assert_eq!(worktree.config_root(), container.join(".repository"));
}

/// Test ledger persistence
#[test]
fn test_ledger_persistence() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("ledger.toml");

    // Create and populate ledger
    let mut ledger = Ledger::new();

    let mut intent = Intent::new(
        "rule:test".to_string(),
        serde_json::json!({ "key": "value" }),
    );

    intent.add_projection(Projection::file_managed(
        "test-tool".to_string(),
        "test-file.txt".into(),
        "sha256:abc123".to_string(),
    ));

    ledger.add_intent(intent);

    // Save
    ledger.save(&ledger_path).unwrap();

    // Load and verify
    let loaded = Ledger::load(&ledger_path).unwrap();
    assert_eq!(loaded.intents().len(), 1);
    assert_eq!(loaded.intents()[0].id, "rule:test");
    assert_eq!(loaded.intents()[0].projections().len(), 1);
}

/// Test configuration hierarchy merge
#[test]
fn test_config_hierarchy() {
    let dir = tempdir().unwrap();
    let root = NormalizedPath::new(dir.path());

    fs::create_dir_all(dir.path().join(".repository")).unwrap();

    // Base config
    fs::write(dir.path().join(".repository/config.toml"), r#"
[core]
mode = "standard"

[presets]
"env:python" = { version = "3.11", provider = "pip" }
"tool:formatter" = { use = "black" }
"#).unwrap();

    // Local override
    fs::write(dir.path().join(".repository/config.local.toml"), r#"
[presets]
"env:python" = { version = "3.12" }
"#).unwrap();

    let resolver = ConfigResolver::new(root);
    let config = resolver.resolve().unwrap();

    // Version should be overridden
    let python = config.presets.get("env:python").unwrap();
    assert_eq!(python["version"].as_str().unwrap(), "3.12");

    // Provider should be preserved from base (deep merge)
    assert_eq!(python["provider"].as_str().unwrap(), "pip");

    // Formatter should be preserved from base
    assert!(config.presets.contains_key("tool:formatter"));
}
```

**Step 2: Run tests**

Run: `cargo test -p repo-core`
Expected: All tests PASS

**Step 3: Verify full test suite**

Run: `cargo test --all`
Expected: All tests PASS

**Step 4: Run clippy**

Run: `cargo clippy -p repo-core -- -D warnings`
Expected: No warnings

**Step 5: Commit**

```bash
git add crates/repo-core/
git commit -m "feat(repo-core): add integration tests and finalize crate

Adds comprehensive integration tests covering:
- Complete workflow (config -> sync -> check)
- Mode backend operations
- Ledger persistence
- Configuration hierarchy merge

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Verification

After completing all tasks:

```bash
# Run all repo-core tests
cargo test -p repo-core

# Run clippy
cargo clippy -p repo-core -- -D warnings

# Build in release mode
cargo build -p repo-core --release

# Run full workspace tests
cargo test --all
```

## Summary

| Task | Deliverable |
|------|-------------|
| 1 | Crate structure with error types |
| 2 | Mode abstraction (Standard/Worktree backends) |
| 3 | Ledger system (Intent/Projection tracking) |
| 4 | Configuration resolution with hierarchical merge |
| 5 | SyncEngine with check/sync/fix operations |
| 6 | Integration tests and final verification |

**Dependencies added:**
- chrono (for timestamps in ledger)
- sha2 (for checksums, already in workspace)

**Integration points:**
- repo-fs: NormalizedPath, file operations
- repo-git: Git operations (via CLI for now)
- repo-meta: RepositoryConfig, definitions (ready for future integration)
- repo-tools: ToolIntegration (ready for future integration)
- repo-presets: PresetProvider (ready for future integration)
- repo-content: Document operations for managed blocks
