//! Tool configuration backup and restore system
//!
//! This module provides functionality to backup tool configurations when tools
//! are removed, and restore them when tools are re-added.
//!
//! Backups are stored at `.repository/backups/{tool}/` with:
//! - metadata.toml: Contains backup timestamp and original file paths
//! - Original files copied with preserved names

mod tool_backup;

pub use tool_backup::{BackupManager, BackupMetadata, ToolBackup};
