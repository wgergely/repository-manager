//! SyncEngine implementation
//!
//! The SyncEngine coordinates state between the ledger (configuration intents)
//! and the filesystem (actual tool configurations).

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::Result;
use crate::backend::{ModeBackend, StandardBackend, WorktreeBackend};
use crate::config::Manifest;
use crate::ledger::{Ledger, ProjectionKind};
use crate::mode::Mode;
use repo_fs::NormalizedPath;

use super::check::{CheckReport, CheckStatus, DriftItem};
use super::rule_syncer::RuleSyncer;
use super::tool_syncer::ToolSyncer;

/// Report from a sync or fix operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    /// Whether the operation completed successfully
    pub success: bool,
    /// Actions taken during the operation
    pub actions: Vec<String>,
    /// Errors encountered during the operation
    pub errors: Vec<String>,
}

impl SyncReport {
    /// Create a successful sync report
    pub fn success() -> Self {
        Self {
            success: true,
            actions: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create a failed sync report
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            actions: Vec::new(),
            errors,
        }
    }

    /// Add an action to the report
    pub fn with_action(mut self, action: String) -> Self {
        self.actions.push(action);
        self
    }
}

/// Options for sync and fix operations
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    /// If true, simulate changes without modifying the filesystem.
    /// Actions will be prefixed with "[dry-run] Would ..."
    pub dry_run: bool,
}

/// Engine for synchronizing configuration state
///
/// The SyncEngine provides three main operations:
/// - **check**: Validate that the filesystem matches the ledger
/// - **sync**: Apply configuration changes from the ledger to the filesystem
/// - **fix**: Re-synchronize to repair any drift
pub struct SyncEngine {
    /// Root path for the repository
    root: NormalizedPath,
    /// Repository mode (Standard or Worktrees)
    mode: Mode,
    /// Backend for mode-specific operations
    backend: Box<dyn ModeBackend>,
}

impl SyncEngine {
    /// Create a new SyncEngine
    ///
    /// # Arguments
    ///
    /// * `root` - The root path of the repository
    /// * `mode` - The repository mode (Standard or Worktrees)
    ///
    /// # Errors
    ///
    /// Returns an error if the backend cannot be created for the given mode.
    pub fn new(root: NormalizedPath, mode: Mode) -> Result<Self> {
        let backend: Box<dyn ModeBackend> = match mode {
            Mode::Standard => Box::new(StandardBackend::new(root.clone())?),
            Mode::Worktrees => Box::new(WorktreeBackend::new(root.clone())?),
        };

        Ok(Self {
            root,
            mode,
            backend,
        })
    }

    /// Get the path to the ledger file
    pub fn ledger_path(&self) -> NormalizedPath {
        self.backend.config_root().join("ledger.toml")
    }

    /// Load the ledger from disk, or create an empty one if it doesn't exist
    ///
    /// # Errors
    ///
    /// Returns an error if the ledger file exists but cannot be read or parsed.
    pub fn load_ledger(&self) -> Result<Ledger> {
        let path = self.ledger_path();
        if path.exists() {
            Ledger::load(path.as_ref())
        } else {
            Ok(Ledger::new())
        }
    }

    /// Save the ledger to disk
    ///
    /// Creates the parent directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the ledger cannot be written.
    pub fn save_ledger(&self, ledger: &Ledger) -> Result<()> {
        let path = self.ledger_path();

        // Create parent directory if needed
        if let Some(parent) = path.as_ref().parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        ledger.save(path.as_ref())
    }

    /// Check the synchronization state
    ///
    /// Validates that all projections in the ledger are correctly reflected
    /// in the filesystem.
    ///
    /// # Returns
    ///
    /// A `CheckReport` containing the status and any issues found.
    pub fn check(&self) -> Result<CheckReport> {
        let ledger = match self.load_ledger() {
            Ok(l) => l,
            Err(e) => {
                return Ok(CheckReport::broken(format!("Failed to load ledger: {}", e)));
            }
        };

        // If ledger is empty, everything is healthy
        if ledger.intents().is_empty() {
            return Ok(CheckReport::healthy());
        }

        let mut drifted = Vec::new();
        let mut missing = Vec::new();

        for intent in ledger.intents() {
            for projection in intent.projections() {
                let file_path = self.root.join(projection.file.to_string_lossy().as_ref());

                match &projection.kind {
                    ProjectionKind::FileManaged { checksum } => {
                        if !file_path.exists() {
                            missing.push(DriftItem {
                                intent_id: intent.id.clone(),
                                tool: projection.tool.clone(),
                                file: projection.file.to_string_lossy().to_string(),
                                description: "File not found".to_string(),
                            });
                        } else {
                            // Check checksum
                            match compute_file_checksum(file_path.as_ref()) {
                                Ok(actual_checksum) => {
                                    if &actual_checksum != checksum {
                                        drifted.push(DriftItem {
                                            intent_id: intent.id.clone(),
                                            tool: projection.tool.clone(),
                                            file: projection.file.to_string_lossy().to_string(),
                                            description: format!(
                                                "Checksum mismatch: expected {}, got {}",
                                                checksum, actual_checksum
                                            ),
                                        });
                                    }
                                }
                                Err(e) => {
                                    missing.push(DriftItem {
                                        intent_id: intent.id.clone(),
                                        tool: projection.tool.clone(),
                                        file: projection.file.to_string_lossy().to_string(),
                                        description: format!("Failed to read file: {}", e),
                                    });
                                }
                            }
                        }
                    }

                    ProjectionKind::TextBlock {
                        marker,
                        checksum: _,
                    } => {
                        if !file_path.exists() {
                            missing.push(DriftItem {
                                intent_id: intent.id.clone(),
                                tool: projection.tool.clone(),
                                file: projection.file.to_string_lossy().to_string(),
                                description: "File not found".to_string(),
                            });
                        } else {
                            // Check if the file contains the marker UUID
                            match fs::read_to_string(file_path.as_ref()) {
                                Ok(content) => {
                                    let marker_str = marker.to_string();
                                    if !content.contains(&marker_str) {
                                        missing.push(DriftItem {
                                            intent_id: intent.id.clone(),
                                            tool: projection.tool.clone(),
                                            file: projection.file.to_string_lossy().to_string(),
                                            description: format!(
                                                "Marker {} not found in file",
                                                marker
                                            ),
                                        });
                                    }
                                    // TODO: Could also check block checksum here
                                }
                                Err(e) => {
                                    missing.push(DriftItem {
                                        intent_id: intent.id.clone(),
                                        tool: projection.tool.clone(),
                                        file: projection.file.to_string_lossy().to_string(),
                                        description: format!("Failed to read file: {}", e),
                                    });
                                }
                            }
                        }
                    }

                    ProjectionKind::JsonKey { path, value } => {
                        if !file_path.exists() {
                            missing.push(DriftItem {
                                intent_id: intent.id.clone(),
                                tool: projection.tool.clone(),
                                file: projection.file.to_string_lossy().to_string(),
                                description: "File not found".to_string(),
                            });
                        } else {
                            // Parse JSON and check the key
                            match fs::read_to_string(file_path.as_ref()) {
                                Ok(content) => match serde_json::from_str::<Value>(&content) {
                                    Ok(json) => {
                                        let actual_value = get_json_path(&json, path);
                                        match actual_value {
                                            Some(actual) => {
                                                if actual != value {
                                                    drifted.push(DriftItem {
                                                        intent_id: intent.id.clone(),
                                                        tool: projection.tool.clone(),
                                                        file: projection
                                                            .file
                                                            .to_string_lossy()
                                                            .to_string(),
                                                        description: format!(
                                                            "Value mismatch at {}: expected {}, got {}",
                                                            path, value, actual
                                                        ),
                                                    });
                                                }
                                            }
                                            None => {
                                                missing.push(DriftItem {
                                                    intent_id: intent.id.clone(),
                                                    tool: projection.tool.clone(),
                                                    file: projection
                                                        .file
                                                        .to_string_lossy()
                                                        .to_string(),
                                                    description: format!(
                                                        "Key {} not found in JSON",
                                                        path
                                                    ),
                                                });
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        drifted.push(DriftItem {
                                            intent_id: intent.id.clone(),
                                            tool: projection.tool.clone(),
                                            file: projection.file.to_string_lossy().to_string(),
                                            description: format!("Invalid JSON: {}", e),
                                        });
                                    }
                                },
                                Err(e) => {
                                    missing.push(DriftItem {
                                        intent_id: intent.id.clone(),
                                        tool: projection.tool.clone(),
                                        file: projection.file.to_string_lossy().to_string(),
                                        description: format!("Failed to read file: {}", e),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Determine overall status
        if !drifted.is_empty() {
            Ok(CheckReport {
                status: CheckStatus::Drifted,
                drifted,
                missing,
                messages: Vec::new(),
            })
        } else if !missing.is_empty() {
            Ok(CheckReport {
                status: CheckStatus::Missing,
                drifted,
                missing,
                messages: Vec::new(),
            })
        } else {
            Ok(CheckReport::healthy())
        }
    }

    /// Synchronize configuration to the filesystem with options
    ///
    /// When `options.dry_run` is true, simulates changes without writing.
    pub fn sync_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
        let mut ledger = self.load_ledger()?;
        let mut report = SyncReport::success();

        // Create ledger if it doesn't exist
        let ledger_path = self.ledger_path();
        if !ledger_path.exists() {
            if options.dry_run {
                report = report.with_action("[dry-run] Would create ledger file".to_string());
            } else {
                self.save_ledger(&ledger)?;
                report = report.with_action("Created ledger file".to_string());
            }
        }

        // Load config to get active tools
        let config_path = self.backend.config_root().join("config.toml");
        if !config_path.exists() {
            return Ok(report.with_action("No config.toml found - nothing to sync".to_string()));
        }

        // Read config and sync tools using typed Manifest parsing
        let config_content = std::fs::read_to_string(config_path.as_ref())?;
        if let Ok(manifest) = Manifest::parse(&config_content) {
            let tool_syncer = ToolSyncer::new(self.root.clone(), options.dry_run);
            let tool_names = &manifest.tools;

            // Sync tool configurations
            for tool_name in tool_names {
                match tool_syncer.sync_tool(tool_name, &mut ledger) {
                    Ok(actions) => {
                        for action in actions {
                            report = report.with_action(action);
                        }
                    }
                    Err(e) => {
                        report
                            .errors
                            .push(format!("Failed to sync {}: {}", tool_name, e));
                    }
                }
            }

            // Sync rules to tool configurations
            let rule_syncer = RuleSyncer::new(self.root.clone(), options.dry_run);
            match rule_syncer.sync_rules(tool_names, &mut ledger) {
                Ok(actions) => {
                    for action in actions {
                        report = report.with_action(action);
                    }
                }
                Err(e) => {
                    report.errors.push(format!("Failed to sync rules: {}", e));
                }
            }
        }

        // Save ledger
        if !options.dry_run {
            self.save_ledger(&ledger)?;
        }

        report.success = report.errors.is_empty();
        Ok(report)
    }

    /// Synchronize configuration to the filesystem
    ///
    /// This operation:
    /// 1. Loads the resolved configuration and ledger
    /// 2. Creates/saves the ledger if it doesn't exist
    /// 3. (Future) Applies configuration changes
    ///
    /// # Returns
    ///
    /// A `SyncReport` containing the actions taken.
    pub fn sync(&self) -> Result<SyncReport> {
        self.sync_with_options(SyncOptions::default())
    }

    /// Fix synchronization issues with options
    ///
    /// When `options.dry_run` is true, simulates fixes without applying.
    pub fn fix_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
        // Check first to identify issues
        let check_report = self.check()?;

        let mut report = SyncReport::success();

        if check_report.status == CheckStatus::Healthy {
            return Ok(report.with_action("No fixes needed".to_string()));
        }

        // Re-sync will fix drift and recreate missing files
        let sync_report = self.sync_with_options(options)?;

        report.actions = sync_report.actions;
        report.errors = sync_report.errors;
        report.success = sync_report.success;

        if !check_report.drifted.is_empty() {
            report = report.with_action(format!(
                "Fixed {} drifted projections",
                check_report.drifted.len()
            ));
        }

        if !check_report.missing.is_empty() {
            report = report.with_action(format!(
                "Recreated {} missing projections",
                check_report.missing.len()
            ));
        }

        Ok(report)
    }

    /// Fix synchronization issues
    ///
    /// Re-synchronizes to repair any drift or missing files.
    ///
    /// # Returns
    ///
    /// A `SyncReport` containing the actions taken.
    pub fn fix(&self) -> Result<SyncReport> {
        self.fix_with_options(SyncOptions::default())
    }

    /// Get the repository root path
    pub fn root(&self) -> &NormalizedPath {
        &self.root
    }

    /// Get the repository mode
    pub fn mode(&self) -> Mode {
        self.mode
    }
}

/// Compute the SHA-256 checksum of a file
///
/// # Arguments
///
/// * `path` - Path to the file
///
/// # Returns
///
/// The hex-encoded SHA-256 checksum of the file contents.
pub fn compute_file_checksum(path: &Path) -> Result<String> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Get a value from a JSON object using a dot-separated path
///
/// # Arguments
///
/// * `json` - The JSON value to query
/// * `path` - A dot-separated path (e.g., "editor.fontSize")
///
/// # Returns
///
/// The value at the path, or None if not found.
pub fn get_json_path<'a>(json: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in parts {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            Value::Array(arr) => {
                let index: usize = part.parse().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_compute_file_checksum() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let checksum = compute_file_checksum(&file_path).unwrap();

        // Known SHA-256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_get_json_path_simple() {
        let json: Value = serde_json::json!({
            "editor": {
                "fontSize": 14
            }
        });

        let result = get_json_path(&json, "editor.fontSize");
        assert_eq!(result, Some(&serde_json::json!(14)));
    }

    #[test]
    fn test_get_json_path_nested() {
        let json: Value = serde_json::json!({
            "a": {
                "b": {
                    "c": "deep"
                }
            }
        });

        let result = get_json_path(&json, "a.b.c");
        assert_eq!(result, Some(&serde_json::json!("deep")));
    }

    #[test]
    fn test_get_json_path_not_found() {
        let json: Value = serde_json::json!({
            "editor": {
                "fontSize": 14
            }
        });

        let result = get_json_path(&json, "editor.tabSize");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_json_path_array() {
        let json: Value = serde_json::json!({
            "items": ["a", "b", "c"]
        });

        let result = get_json_path(&json, "items.1");
        assert_eq!(result, Some(&serde_json::json!("b")));
    }

    #[test]
    fn test_sync_report_success() {
        let report = SyncReport::success();
        assert!(report.success);
        assert!(report.actions.is_empty());
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_sync_report_with_action() {
        let report = SyncReport::success().with_action("Created file".to_string());
        assert!(report.success);
        assert_eq!(report.actions.len(), 1);
        assert_eq!(report.actions[0], "Created file");
    }
}
