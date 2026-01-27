//! Tool synchronization logic
//!
//! This module provides the `ToolSyncer` for coordinating the syncing of
//! tool configurations using projections. It handles creating, updating,
//! and removing tool configurations in the filesystem and ledger.
//!
//! Includes backup/restore functionality for tool configurations.

use crate::backup::BackupManager;
use crate::ledger::{Intent, Ledger, Projection, ProjectionKind};
use crate::projection::compute_checksum;
use crate::{Error, Result};
use repo_fs::NormalizedPath;
use std::path::PathBuf;
use uuid::Uuid;

/// Write content to a file safely (with symlink protection)
fn safe_write(path: &NormalizedPath, content: &str) -> Result<()> {
    repo_fs::io::write_text(path, content).map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
}

/// Synchronizes tool configurations
///
/// The `ToolSyncer` coordinates the creation, update, and removal of tool
/// configurations. It uses the ledger to track what tools have been synced
/// and their projections.
///
/// Features backup/restore for tool configurations when tools are removed/re-added.
pub struct ToolSyncer {
    /// Root path for the repository
    root: NormalizedPath,
    /// Whether to run in dry-run mode (simulate changes without writing)
    dry_run: bool,
    /// Backup manager for tool configuration backup/restore
    backup_manager: BackupManager,
}

impl ToolSyncer {
    /// Create a new `ToolSyncer`
    ///
    /// # Arguments
    ///
    /// * `root` - The root path of the repository
    /// * `dry_run` - If true, simulate changes without modifying the filesystem
    pub fn new(root: NormalizedPath, dry_run: bool) -> Self {
        let backup_manager = BackupManager::new(root.clone());
        Self {
            root,
            dry_run,
            backup_manager,
        }
    }

    /// Check if a backup exists for a tool
    pub fn has_backup(&self, tool_name: &str) -> bool {
        self.backup_manager.has_backup(tool_name)
    }

    /// Sync a tool, creating/updating its projections in the ledger
    ///
    /// This method:
    /// 1. Checks if the tool is already synced (by looking for matching intents)
    /// 2. Gets the configuration files for the tool
    /// 3. Creates projections for each config file
    /// 4. Writes the files to disk (unless dry_run is true)
    /// 5. Adds the intent to the ledger
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool to sync (e.g., "cursor", "vscode", "claude")
    /// * `ledger` - Mutable reference to the ledger
    ///
    /// # Returns
    ///
    /// A list of action descriptions taken during the sync.
    pub fn sync_tool(&self, tool_name: &str, ledger: &mut Ledger) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        let intent_id = format!("tool:{}", tool_name);

        // Check if intent already exists
        let existing = self.get_intents_by_id(ledger, &intent_id);
        if !existing.is_empty() {
            actions.push(format!("Tool {} already synced", tool_name));
            return Ok(actions);
        }

        // Get tool config files based on tool type
        let config_files = self.get_tool_config_files(tool_name);

        if config_files.is_empty() {
            actions.push(format!("No config files for tool {}", tool_name));
            return Ok(actions);
        }

        // Create projections for each config file
        let mut projections = Vec::new();
        for (file_path, content) in &config_files {
            let checksum = compute_checksum(content);
            projections.push(Projection {
                tool: tool_name.to_string(),
                file: PathBuf::from(file_path),
                kind: ProjectionKind::FileManaged { checksum },
            });

            if self.dry_run {
                actions.push(format!("[dry-run] Would create {}", file_path));
            } else {
                // Write the file using symlink-safe write
                let full_path = self.root.join(file_path);
                safe_write(&full_path, content)?;
                actions.push(format!("Created {}", file_path));
            }
        }

        // Create intent with projections
        let mut intent = Intent::new(intent_id.clone(), serde_json::json!({}));
        for projection in projections {
            intent.add_projection(projection);
        }

        if !self.dry_run {
            ledger.add_intent(intent);
            actions.push(format!("Added intent {}", intent_id));
        }

        Ok(actions)
    }

    /// Remove a tool, deleting its projections
    ///
    /// This method:
    /// 1. Finds all intents for the specified tool
    /// 2. Creates a backup of the tool's config files
    /// 3. Deletes the files associated with each projection
    /// 4. Removes the intents from the ledger
    ///
    /// # Arguments
    ///
    /// * `tool_name` - The name of the tool to remove
    /// * `ledger` - Mutable reference to the ledger
    ///
    /// # Returns
    ///
    /// A list of action descriptions taken during removal.
    pub fn remove_tool(&self, tool_name: &str, ledger: &mut Ledger) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        let intent_id = format!("tool:{}", tool_name);

        let intents: Vec<Uuid> = self
            .get_intents_by_id(ledger, &intent_id)
            .iter()
            .map(|i| i.uuid)
            .collect();

        if intents.is_empty() {
            actions.push(format!("Tool {} not found in ledger", tool_name));
            return Ok(actions);
        }

        // Collect files to backup before deleting
        let mut files_to_backup = Vec::new();
        for uuid in &intents {
            if let Some(intent) = ledger.get_intent(*uuid) {
                for projection in intent.projections() {
                    files_to_backup.push(projection.file.clone());
                }
            }
        }

        // Create backup before deleting (unless dry-run)
        if !self.dry_run && !files_to_backup.is_empty() {
            match self.backup_manager.create_backup(tool_name, &files_to_backup) {
                Ok(backup) => {
                    actions.push(format!(
                        "Created backup for {} ({} files)",
                        tool_name,
                        backup.metadata.files.len()
                    ));
                }
                Err(e) => {
                    // Log warning but continue with removal
                    tracing::warn!("Failed to create backup for {}: {}", tool_name, e);
                }
            }
        }

        // Now delete the files
        for uuid in intents {
            if let Some(intent) = ledger.get_intent(uuid) {
                for projection in intent.projections() {
                    let file_path = self.root.join(projection.file.to_string_lossy().as_ref());

                    if self.dry_run {
                        actions.push(format!("[dry-run] Would delete {}", file_path));
                    } else if file_path.exists() {
                        std::fs::remove_file(file_path.as_ref())?;
                        actions.push(format!("Deleted {}", file_path));
                    }
                }
            }

            if !self.dry_run {
                ledger.remove_intent(uuid);
                actions.push(format!("Removed intent for {}", tool_name));
            }
        }

        Ok(actions)
    }

    /// Remove a tool with option to skip backup
    pub fn remove_tool_no_backup(&self, tool_name: &str, ledger: &mut Ledger) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        let intent_id = format!("tool:{}", tool_name);

        let intents: Vec<Uuid> = self
            .get_intents_by_id(ledger, &intent_id)
            .iter()
            .map(|i| i.uuid)
            .collect();

        if intents.is_empty() {
            actions.push(format!("Tool {} not found in ledger", tool_name));
            return Ok(actions);
        }

        for uuid in intents {
            if let Some(intent) = ledger.get_intent(uuid) {
                for projection in intent.projections() {
                    let file_path = self.root.join(projection.file.to_string_lossy().as_ref());

                    if self.dry_run {
                        actions.push(format!("[dry-run] Would delete {}", file_path));
                    } else if file_path.exists() {
                        std::fs::remove_file(file_path.as_ref())?;
                        actions.push(format!("Deleted {}", file_path));
                    }
                }
            }

            if !self.dry_run {
                ledger.remove_intent(uuid);
                actions.push(format!("Removed intent for {}", tool_name));
            }
        }

        Ok(actions)
    }

    /// Restore a tool from backup
    ///
    /// Returns the list of restored file paths.
    pub fn restore_from_backup(&self, tool_name: &str) -> Result<Vec<PathBuf>> {
        if self.dry_run {
            if self.backup_manager.has_backup(tool_name) {
                tracing::info!("[dry-run] Would restore backup for {}", tool_name);
            }
            return Ok(Vec::new());
        }

        self.backup_manager.restore_backup(tool_name)
    }

    /// Delete a tool's backup
    pub fn delete_backup(&self, tool_name: &str) -> Result<()> {
        if self.dry_run {
            if self.backup_manager.has_backup(tool_name) {
                tracing::info!("[dry-run] Would delete backup for {}", tool_name);
            }
            return Ok(());
        }

        self.backup_manager.delete_backup(tool_name)
    }

    /// Get intents by ID from the ledger
    ///
    /// Helper method to find all intents matching a given ID.
    fn get_intents_by_id<'a>(&self, ledger: &'a Ledger, intent_id: &str) -> Vec<&'a Intent> {
        ledger.find_by_rule(intent_id)
    }

    /// Get config files for a tool
    ///
    /// Returns a list of (file_path, content) tuples for the tool's configuration files.
    fn get_tool_config_files(&self, tool_name: &str) -> Vec<(String, String)> {
        match tool_name {
            "cursor" => vec![(".cursorrules".to_string(), self.generate_cursor_rules())],
            "vscode" => vec![(
                ".vscode/settings.json".to_string(),
                self.generate_vscode_settings(),
            )],
            "claude" => vec![(
                ".claude/config.json".to_string(),
                self.generate_claude_config(),
            )],
            _ => vec![],
        }
    }

    /// Generate default Cursor rules content
    fn generate_cursor_rules(&self) -> String {
        r#"# Cursor Rules
# Generated by Repository Manager

## Code Style
- Follow the project's existing code style
- Use consistent formatting
"#
        .to_string()
    }

    /// Generate default VSCode settings content
    fn generate_vscode_settings(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "editor.formatOnSave": true,
            "editor.tabSize": 4
        }))
        .unwrap()
    }

    /// Generate default Claude config content
    fn generate_claude_config(&self) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "version": "1.0"
        }))
        .unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tool_syncer_new() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);
        assert!(!syncer.dry_run);
    }

    #[test]
    fn test_tool_syncer_dry_run() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, true);
        assert!(syncer.dry_run);
    }

    #[test]
    fn test_get_tool_config_files_cursor() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("cursor");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, ".cursorrules");
        assert!(files[0].1.contains("Cursor Rules"));
    }

    #[test]
    fn test_get_tool_config_files_vscode() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("vscode");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, ".vscode/settings.json");
        assert!(files[0].1.contains("formatOnSave"));
    }

    #[test]
    fn test_get_tool_config_files_claude() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("claude");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, ".claude/config.json");
        assert!(files[0].1.contains("version"));
    }

    #[test]
    fn test_get_tool_config_files_unknown() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("unknown_tool");
        assert!(files.is_empty());
    }

    #[test]
    fn test_compute_checksum() {
        let content = "hello world";
        let checksum = compute_checksum(content);

        // Known SHA-256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_sync_tool_dry_run() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root.clone(), true);
        let mut ledger = Ledger::new();

        let actions = syncer.sync_tool("cursor", &mut ledger).unwrap();

        // Should have dry-run action
        assert!(actions.iter().any(|a| a.contains("[dry-run]")));
        // Ledger should be empty (no actual intent added in dry-run)
        assert!(ledger.intents().is_empty());
        // File should not be created
        let file_path = root.join(".cursorrules");
        assert!(!file_path.exists());
    }

    #[test]
    fn test_sync_tool_creates_file() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root.clone(), false);
        let mut ledger = Ledger::new();

        let actions = syncer.sync_tool("cursor", &mut ledger).unwrap();

        // Should have created action
        assert!(actions.iter().any(|a| a.contains("Created")));
        // Ledger should have one intent
        assert_eq!(ledger.intents().len(), 1);
        // File should be created
        let file_path = root.join(".cursorrules");
        assert!(file_path.exists());
    }

    #[test]
    fn test_sync_tool_already_synced() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);
        let mut ledger = Ledger::new();

        // First sync
        syncer.sync_tool("cursor", &mut ledger).unwrap();

        // Second sync should report already synced
        let actions = syncer.sync_tool("cursor", &mut ledger).unwrap();
        assert!(actions.iter().any(|a| a.contains("already synced")));
        // Ledger should still have only one intent
        assert_eq!(ledger.intents().len(), 1);
    }

    #[test]
    fn test_sync_tool_unknown_tool() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);
        let mut ledger = Ledger::new();

        let actions = syncer.sync_tool("unknown_tool", &mut ledger).unwrap();

        assert!(actions.iter().any(|a| a.contains("No config files")));
        assert!(ledger.intents().is_empty());
    }

    #[test]
    fn test_remove_tool() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root.clone(), false);
        let mut ledger = Ledger::new();

        // First sync the tool
        syncer.sync_tool("cursor", &mut ledger).unwrap();
        assert_eq!(ledger.intents().len(), 1);

        // Now remove it
        let actions = syncer.remove_tool("cursor", &mut ledger).unwrap();

        // Should have deleted action
        assert!(actions.iter().any(|a| a.contains("Deleted")));
        assert!(actions.iter().any(|a| a.contains("Removed intent")));
        // Ledger should be empty
        assert!(ledger.intents().is_empty());
        // File should be deleted
        let file_path = root.join(".cursorrules");
        assert!(!file_path.exists());
    }

    #[test]
    fn test_remove_tool_not_found() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);
        let mut ledger = Ledger::new();

        let actions = syncer.remove_tool("cursor", &mut ledger).unwrap();

        assert!(actions.iter().any(|a| a.contains("not found in ledger")));
    }

    #[test]
    fn test_remove_tool_dry_run() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer_write = ToolSyncer::new(root.clone(), false);
        let syncer_dry = ToolSyncer::new(root.clone(), true);
        let mut ledger = Ledger::new();

        // First sync the tool (not dry-run)
        syncer_write.sync_tool("cursor", &mut ledger).unwrap();

        // Now try to remove with dry-run
        let actions = syncer_dry.remove_tool("cursor", &mut ledger).unwrap();

        // Should have dry-run action
        assert!(actions.iter().any(|a| a.contains("[dry-run]")));
        // Ledger should still have the intent
        assert_eq!(ledger.intents().len(), 1);
        // File should still exist
        let file_path = root.join(".cursorrules");
        assert!(file_path.exists());
    }
}
