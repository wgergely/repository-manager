//! Constants and enums for repository filesystem paths.

use std::path::Path;

/// Standard repository filesystem markers and paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoPath {
    /// The `.gt` directory (Container mode database)
    GtDir,
    /// The `.git` directory (Git database)
    GitDir,
    /// The `.worktrees` directory (In-repo worktrees root)
    WorktreesDir,
    /// The `main` directory (Primary worktree in Container mode)
    MainWorktree,
    /// The `.repository` directory (Configuration root)
    RepositoryConfig,
}

impl RepoPath {
    /// Get the string representation of the path.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GtDir => ".gt",
            Self::GitDir => ".git",
            Self::WorktreesDir => ".worktrees",
            Self::MainWorktree => "main",
            Self::RepositoryConfig => ".repository",
        }
    }
}

impl AsRef<Path> for RepoPath {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl AsRef<str> for RepoPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for RepoPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
