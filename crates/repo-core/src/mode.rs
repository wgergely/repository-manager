//! Repository mode abstraction
//!
//! This module re-exports [`repo_meta::RepositoryMode`] as [`Mode`] for
//! convenient use within the core crate and downstream consumers.
//!
//! The canonical definition lives in `repo-meta`. This type alias avoids
//! a duplicate enum while preserving the short `Mode` name used throughout
//! `repo-core` and `repo-cli`.

use repo_fs::NormalizedPath;

use crate::config::ConfigResolver;
use crate::error::{Error, Result};

/// Repository operation mode (type alias for [`repo_meta::RepositoryMode`]).
pub type Mode = repo_meta::RepositoryMode;

/// Detect the repository mode from filesystem markers and configuration.
///
/// Detection follows this precedence:
///
/// 1. **Filesystem markers** — a `.gt` directory in `root` (or its parent)
///    indicates Worktrees mode; a `.git` directory indicates Standard mode.
/// 2. **Configuration file** — reads the mode from `.repository/config.toml`
///    using [`ConfigResolver`].
/// 3. **Default** — falls back to [`Mode::Standard`] (the safer default).
///
/// # Arguments
///
/// * `root` - The repository root directory
///
/// # Errors
///
/// Returns an error if the configuration file exists but contains an invalid mode.
pub fn detect_mode(root: &NormalizedPath) -> Result<Mode> {
    // Check for .gt (worktree container marker)
    if root.join(".gt").exists() {
        return Ok(Mode::Worktrees);
    }

    // Check for .git (standard repo marker)
    if root.join(".git").exists() {
        return Ok(Mode::Standard);
    }

    // Check if we're inside a worktree (parent has .gt)
    if let Some(parent) = root.as_ref().parent() {
        let parent_path = NormalizedPath::new(parent);
        if parent_path.join(".gt").exists() {
            return Ok(Mode::Worktrees);
        }
    }

    // Fall back to ConfigResolver for config.toml-based detection
    let resolver = ConfigResolver::new(root.clone());

    if !resolver.has_config() {
        // No config file — default to standard mode (the safer default)
        return Ok(Mode::Standard);
    }

    let config = resolver.resolve()?;
    let mode: Mode = config
        .mode
        .parse()
        .map_err(|e: repo_meta::Error| Error::Meta(e))?;

    Ok(mode)
}
