//! SyncEngine for coordinating state between configuration and filesystem
//!
//! This module provides:
//! - **check**: Validate ledger projections against filesystem state
//! - **sync**: Apply configuration changes to the filesystem
//! - **fix**: Re-synchronize to repair drift or missing files

mod check;
mod engine;

pub use check::{CheckReport, CheckStatus, DriftItem};
pub use engine::{compute_file_checksum, get_json_path, SyncEngine, SyncOptions, SyncReport};
