//! Configuration types and loading for Repository Manager
//!
//! This module provides types for loading and working with
//! the `.repository/config.toml` configuration file.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Repository operation mode.
///
/// Determines how the repository is laid out and how branches are managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryMode {
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

impl RepositoryMode {
    /// Check if this mode supports parallel worktrees.
    pub fn supports_parallel_worktrees(&self) -> bool {
        matches!(self, RepositoryMode::Worktrees)
    }
}

impl FromStr for RepositoryMode {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standard" | "default" => Ok(RepositoryMode::Standard),
            "worktrees" | "worktree" | "container" => Ok(RepositoryMode::Worktrees),
            _ => Err(Error::InvalidMode {
                mode: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for RepositoryMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryMode::Standard => write!(f, "standard"),
            RepositoryMode::Worktrees => write!(f, "worktrees"),
        }
    }
}

/// Core repository configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Configuration schema version
    #[serde(default = "default_version")]
    pub version: String,
    /// Repository operation mode
    #[serde(default)]
    pub mode: RepositoryMode,
}

fn default_version() -> String {
    "1".to_string()
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            mode: RepositoryMode::default(),
        }
    }
}

/// Active tools and presets configuration
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ActiveConfig {
    /// List of active tool IDs
    #[serde(default)]
    pub tools: Vec<String>,
    /// List of active preset IDs
    #[serde(default)]
    pub presets: Vec<String>,
}

/// Sync strategy for configuration synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SyncStrategy {
    /// Automatically sync on relevant operations
    #[default]
    Auto,
    /// Only sync when explicitly requested
    Manual,
    /// Sync on every commit
    OnCommit,
}

/// Synchronization configuration
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync strategy
    #[serde(default)]
    pub strategy: SyncStrategy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_mode_default() {
        assert_eq!(RepositoryMode::default(), RepositoryMode::Worktrees);
    }

    #[test]
    fn test_core_config_default() {
        let config = CoreConfig::default();
        assert_eq!(config.version, "1");
        assert_eq!(config.mode, RepositoryMode::Worktrees);
    }

    #[test]
    fn test_supports_parallel_worktrees() {
        assert!(!RepositoryMode::Standard.supports_parallel_worktrees());
        assert!(RepositoryMode::Worktrees.supports_parallel_worktrees());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            "standard".parse::<RepositoryMode>().unwrap(),
            RepositoryMode::Standard
        );
        assert_eq!(
            "default".parse::<RepositoryMode>().unwrap(),
            RepositoryMode::Standard
        );
        assert_eq!(
            "worktrees".parse::<RepositoryMode>().unwrap(),
            RepositoryMode::Worktrees
        );
        assert_eq!(
            "worktree".parse::<RepositoryMode>().unwrap(),
            RepositoryMode::Worktrees
        );
        assert_eq!(
            "container".parse::<RepositoryMode>().unwrap(),
            RepositoryMode::Worktrees
        );
        assert!("invalid".parse::<RepositoryMode>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(RepositoryMode::Standard.to_string(), "standard");
        assert_eq!(RepositoryMode::Worktrees.to_string(), "worktrees");
    }
}
