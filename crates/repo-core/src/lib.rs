//! Core orchestration layer for Repository Manager
//!
//! This crate provides the high-level coordination between all Layer 0 crates,
//! implementing:
//!
//! - **Mode abstraction**: Unified interface for Standard and Worktree repository layouts
//! - **Ledger system**: Intent and projection tracking for configuration management
//! - **Configuration resolution**: Hierarchical merge of workspace, repository, and user configs
//! - **SyncEngine**: Check, sync, and fix operations for tool configurations
//!
//! # Architecture
//!
//! `repo-core` sits above the Layer 0 crates and below the CLI/API layer:
//!
//! ```text
//!                    CLI / API
//!                        |
//!                   repo-core
//!                        |
//!     +------+------+----+----+-------+--------+
//!     |      |      |         |       |        |
//! repo-fs repo-git repo-meta repo-tools repo-presets repo-content
//! ```
//!
//! # Example
//!
//! ```ignore
//! use repo_core::{Result, Error};
//!
//! fn example() -> Result<()> {
//!     // Core functionality will be added in subsequent tasks
//!     Ok(())
//! }
//! ```

pub mod backend;
pub mod backup;
pub mod config;
pub mod error;
pub mod governance;
pub mod ledger;
pub mod mode;
pub mod projection;
pub mod rules;
pub mod sync;

pub use backend::{BranchInfo, ModeBackend, StandardBackend, WorktreeBackend};
pub use backup::{BackupManager, BackupMetadata, ToolBackup};
pub use config::{ConfigResolver, Manifest, ResolvedConfig, RuntimeContext, json_to_toml_value};
pub use error::{Error, Result};
pub use ledger::{Intent, Ledger, Projection, ProjectionKind};
pub use mode::Mode;
pub use projection::{ProjectionWriter, compute_checksum};
pub use governance::{ConfigDrift, DriftType, LintWarning, WarnLevel};
pub use rules::{Rule, RuleRegistry};
pub use sync::{
    CheckReport, CheckStatus, DriftItem, RuleFile, RuleSyncer, SyncEngine, SyncOptions, SyncReport,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn error_config_not_found_displays_correctly() {
        let path = PathBuf::from("/path/to/config.toml");
        let error = Error::ConfigNotFound { path: path.clone() };

        let display = format!("{}", error);
        assert!(
            display.contains("/path/to/config.toml"),
            "Error display should contain the path, got: {}",
            display
        );
        assert!(
            display.to_lowercase().contains("config")
                || display.to_lowercase().contains("not found"),
            "Error display should mention config or not found, got: {}",
            display
        );
    }
}
