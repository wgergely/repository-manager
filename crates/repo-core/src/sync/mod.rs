//! SyncEngine for coordinating state between configuration and filesystem
//!
//! This module provides:
//! - **check**: Validate ledger projections against filesystem state
//! - **sync**: Apply configuration changes to the filesystem
//! - **fix**: Re-synchronize to repair drift or missing files
//! - **tool_syncer**: Coordinate syncing of tool configurations
//! - **rule_syncer**: Synchronize rules from `.repository/rules/` to tool configurations

mod check;
mod engine;
mod rule_syncer;
mod tool_syncer;

pub use check::{CheckReport, CheckStatus, DriftItem};
pub use engine::{compute_file_checksum, get_json_path, SyncEngine, SyncOptions, SyncReport};
pub use rule_syncer::{RuleFile, RuleSyncer};
pub use tool_syncer::ToolSyncer;
