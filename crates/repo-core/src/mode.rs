//! Repository mode abstraction
//!
//! Defines the two supported repository layouts:
//! - Standard: Traditional single-directory Git repository
//! - Worktrees: Container-based layout with multiple parallel worktrees

use std::fmt;
use std::str::FromStr;

use crate::{Error, Result};

/// Repository operation mode.
///
/// Determines how the repository is laid out and how branches are managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// Traditional single-directory Git repository.
    ///
    /// Branches are managed via `git checkout` and only one branch
    /// can be active at a time.
    Standard,

    /// Container-based layout with multiple worktrees.
    ///
    /// The repository uses a container directory with:
    /// - `.gt/` - Git database (or `.git` in some layouts)
    /// - `main/` - Main branch worktree
    /// - `{branch}/` - Feature worktrees as sibling directories
    ///
    /// Multiple branches can be worked on simultaneously.
    #[default]
    Worktrees,
}

impl Mode {
    /// Check if this mode supports parallel worktrees.
    pub fn supports_parallel_worktrees(&self) -> bool {
        matches!(self, Mode::Worktrees)
    }
}

impl FromStr for Mode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "standard" | "default" => Ok(Mode::Standard),
            "worktrees" | "worktree" | "container" => Ok(Mode::Worktrees),
            _ => Err(Error::InvalidMode { mode: s.to_string() }),
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Standard => write!(f, "standard"),
            Mode::Worktrees => write!(f, "worktrees"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_parallel_worktrees() {
        assert!(!Mode::Standard.supports_parallel_worktrees());
        assert!(Mode::Worktrees.supports_parallel_worktrees());
    }

    #[test]
    fn test_default() {
        assert_eq!(Mode::default(), Mode::Worktrees);
    }
}
