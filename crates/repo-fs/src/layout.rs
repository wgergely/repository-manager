//! Workspace layout detection and management

use crate::{Error, NormalizedPath, RepoPath, Result};
use std::path::Path;

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
        let start = dunce::canonicalize(start_dir.as_ref())
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
        let has_gt = dir.join(RepoPath::GtDir).is_dir();
        let has_git = dir.join(RepoPath::GitDir).exists(); // Can be file or dir
        let has_main = dir.join(RepoPath::MainWorktree).is_dir();
        let has_worktrees = dir.join(RepoPath::WorktreesDir).is_dir();

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
            LayoutMode::Container => self.root.join(RepoPath::GtDir.as_str()),
            LayoutMode::InRepoWorktrees | LayoutMode::Classic => {
                self.root.join(RepoPath::GitDir.as_str())
            }
        }
    }

    /// Get the path to the .repository config directory.
    pub fn config_dir(&self) -> NormalizedPath {
        self.root.join(RepoPath::RepositoryConfig.as_str())
    }

    /// Validate that the filesystem matches the expected layout.
    pub fn validate(&self) -> Result<()> {
        match self.mode {
            LayoutMode::Container => {
                // Fix: Check for directory existence, not just file existence
                if !self.root.join(RepoPath::GtDir.as_str()).is_dir() {
                    return Err(Error::LayoutValidation {
                        message: format!(
                            "Git database missing. Expected {}/ directory.",
                            RepoPath::GtDir
                        ),
                    });
                }
                if !self.root.join(RepoPath::MainWorktree.as_str()).is_dir() {
                    return Err(Error::LayoutValidation {
                        message: format!(
                            "Primary worktree missing. Expected {}/ directory.",
                            RepoPath::MainWorktree
                        ),
                    });
                }
            }
            LayoutMode::InRepoWorktrees | LayoutMode::Classic => {
                if !self.root.join(RepoPath::GitDir.as_str()).exists() {
                    return Err(Error::LayoutValidation {
                        message: "Not a git repository.".into(),
                    });
                }
            }
        }
        Ok(())
    }
}
