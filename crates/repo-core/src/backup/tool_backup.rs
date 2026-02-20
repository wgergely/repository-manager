//! Tool backup implementation
//!
//! Handles creating, listing, and restoring tool configuration backups.

use crate::Result;
use chrono::{DateTime, Utc};
use repo_fs::{NormalizedPath, validate_path_identifier};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Metadata for a tool backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Tool name
    pub tool: String,
    /// When the backup was created
    pub created: DateTime<Utc>,
    /// List of backed up files (relative paths)
    pub files: Vec<String>,
}

impl BackupMetadata {
    /// Create new backup metadata
    pub fn new(tool: impl Into<String>, files: Vec<String>) -> Self {
        Self {
            tool: tool.into(),
            created: Utc::now(),
            files,
        }
    }
}

/// Information about a tool backup
#[derive(Debug, Clone)]
pub struct ToolBackup {
    /// Tool name
    pub tool: String,
    /// Path to the backup directory
    pub path: NormalizedPath,
    /// Backup metadata
    pub metadata: BackupMetadata,
}

/// Manages tool configuration backups
pub struct BackupManager {
    /// Root of the repository
    root: NormalizedPath,
    /// Path to backups directory (.repository/backups)
    backups_dir: NormalizedPath,
}

impl BackupManager {
    /// Create a new BackupManager for the given repository root
    pub fn new(root: NormalizedPath) -> Self {
        let backups_dir = root.join(".repository").join("backups");
        Self { root, backups_dir }
    }

    /// Validate that a tool name is safe for use as a directory component.
    fn validate_tool_name(tool: &str) -> crate::Result<()> {
        validate_path_identifier(tool, "Tool name").map_err(|msg| crate::Error::SyncError {
            message: msg,
        })
    }

    /// Get the backup directory for a tool
    fn tool_backup_dir(&self, tool: &str) -> NormalizedPath {
        self.backups_dir.join(tool)
    }

    /// Get the metadata file path for a tool backup
    fn metadata_path(&self, tool: &str) -> NormalizedPath {
        self.tool_backup_dir(tool).join("metadata.toml")
    }

    /// Check if a backup exists for a tool
    pub fn has_backup(&self, tool: &str) -> bool {
        self.metadata_path(tool).exists()
    }

    /// Create a backup for a tool
    ///
    /// # Arguments
    /// - `tool`: Name of the tool to backup
    /// - `files`: List of file paths (relative to repo root) to backup
    ///
    /// # Returns
    /// The created ToolBackup
    pub fn create_backup(&self, tool: &str, files: &[PathBuf]) -> Result<ToolBackup> {
        Self::validate_tool_name(tool)?;
        let backup_dir = self.tool_backup_dir(tool);

        // Create backup directory
        fs::create_dir_all(backup_dir.as_ref())?;

        let mut backed_up_files = Vec::new();

        // Copy each file to backup directory
        for file in files {
            let source = self.root.join(file.to_string_lossy().as_ref());
            if source.exists() {
                // Use the filename as the backup name
                let filename = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                let dest = backup_dir.join(filename);

                // Copy the file
                fs::copy(source.as_ref(), dest.as_ref())?;

                // Store relative path
                backed_up_files.push(file.to_string_lossy().to_string());
            }
        }

        // Create and save metadata
        let metadata = BackupMetadata::new(tool, backed_up_files);
        let metadata_content = toml::to_string_pretty(&metadata)?;
        fs::write(self.metadata_path(tool).as_ref(), metadata_content)?;

        Ok(ToolBackup {
            tool: tool.to_string(),
            path: backup_dir,
            metadata,
        })
    }

    /// Get backup information for a tool
    pub fn get_backup(&self, tool: &str) -> Result<Option<ToolBackup>> {
        Self::validate_tool_name(tool)?;
        let metadata_path = self.metadata_path(tool);

        if !metadata_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(metadata_path.as_ref())?;
        let metadata: BackupMetadata = toml::from_str(&content)?;

        Ok(Some(ToolBackup {
            tool: tool.to_string(),
            path: self.tool_backup_dir(tool),
            metadata,
        }))
    }

    /// Restore a tool's backed up files
    ///
    /// # Arguments
    /// - `tool`: Name of the tool to restore
    ///
    /// # Returns
    /// List of restored file paths
    pub fn restore_backup(&self, tool: &str) -> Result<Vec<PathBuf>> {
        Self::validate_tool_name(tool)?;
        let backup = self
            .get_backup(tool)?
            .ok_or_else(|| crate::Error::SyncError {
                message: format!("No backup found for tool: {}", tool),
            })?;

        let mut restored = Vec::new();
        let backup_dir = self.tool_backup_dir(tool);

        // Resolve root to an absolute path for containment checking
        let root_prefix = self.root.as_str();

        for file_path in &backup.metadata.files {
            let file = PathBuf::from(file_path);
            let filename = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let source = backup_dir.join(filename);
            let dest = self.root.join(file_path);

            // Security: Verify the destination stays within the repository root.
            // NormalizedPath::join resolves ".." but the result could still land
            // outside root (e.g. file_path = "../../etc/crontab").
            if !dest.as_str().starts_with(root_prefix) {
                return Err(crate::Error::SyncError {
                    message: format!(
                        "Refusing to restore file outside repository: {} (resolves to {})",
                        file_path,
                        dest.as_str()
                    ),
                });
            }

            if source.exists() {
                // Create parent directory if needed
                if let Some(parent) = dest.as_ref().parent()
                    && !parent.exists()
                {
                    fs::create_dir_all(parent)?;
                }

                // Copy the file back
                fs::copy(source.as_ref(), dest.as_ref())?;
                restored.push(file);
            }
        }

        Ok(restored)
    }

    /// Delete a tool's backup
    pub fn delete_backup(&self, tool: &str) -> Result<()> {
        Self::validate_tool_name(tool)?;
        let backup_dir = self.tool_backup_dir(tool);

        if backup_dir.exists() {
            fs::remove_dir_all(backup_dir.as_ref())?;
        }

        Ok(())
    }

    /// List all available backups
    pub fn list_backups(&self) -> Result<Vec<ToolBackup>> {
        if !self.backups_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(self.backups_dir.as_ref())? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir()
                && let Some(tool_name) = path.file_name().and_then(|n| n.to_str())
                && let Ok(Some(backup)) = self.get_backup(tool_name)
            {
                backups.push(backup);
            }
        }

        Ok(backups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, BackupManager) {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());

        // Create .repository directory
        fs::create_dir_all(temp.path().join(".repository")).unwrap();

        let manager = BackupManager::new(root);
        (temp, manager)
    }

    #[test]
    fn test_backup_manager_new() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let manager = BackupManager::new(root.clone());

        assert_eq!(
            manager.backups_dir.as_str(),
            root.join(".repository").join("backups").as_str()
        );
    }

    #[test]
    fn test_has_backup_false_initially() {
        let (_temp, manager) = setup_test_repo();
        assert!(!manager.has_backup("cursor"));
    }

    #[test]
    fn test_create_backup() {
        let (temp, manager) = setup_test_repo();

        // Create a file to backup
        let file_path = PathBuf::from(".cursorrules");
        fs::write(temp.path().join(&file_path), "# Test content").unwrap();

        let backup = manager
            .create_backup("cursor", std::slice::from_ref(&file_path))
            .unwrap();

        assert_eq!(backup.tool, "cursor");
        assert_eq!(backup.metadata.files.len(), 1);
        assert!(manager.has_backup("cursor"));
    }

    #[test]
    fn test_get_backup() {
        let (temp, manager) = setup_test_repo();

        // Create a file and backup
        let file_path = PathBuf::from(".cursorrules");
        fs::write(temp.path().join(&file_path), "# Test content").unwrap();
        manager.create_backup("cursor", &[file_path]).unwrap();

        // Get the backup
        let backup = manager.get_backup("cursor").unwrap();
        assert!(backup.is_some());

        let backup = backup.unwrap();
        assert_eq!(backup.tool, "cursor");
        assert_eq!(backup.metadata.files.len(), 1);
    }

    #[test]
    fn test_restore_backup() {
        let (temp, manager) = setup_test_repo();

        // Create a file and backup
        let file_path = PathBuf::from(".cursorrules");
        let original_content = "# Test content";
        fs::write(temp.path().join(&file_path), original_content).unwrap();
        manager
            .create_backup("cursor", std::slice::from_ref(&file_path))
            .unwrap();

        // Delete the original file
        fs::remove_file(temp.path().join(&file_path)).unwrap();
        assert!(!temp.path().join(&file_path).exists());

        // Restore the backup
        let restored = manager.restore_backup("cursor").unwrap();
        assert_eq!(restored.len(), 1);

        // Verify the file was restored
        assert!(temp.path().join(&file_path).exists());
        let content = fs::read_to_string(temp.path().join(&file_path)).unwrap();
        assert_eq!(content, original_content);
    }

    #[test]
    fn test_delete_backup() {
        let (temp, manager) = setup_test_repo();

        // Create a file and backup
        let file_path = PathBuf::from(".cursorrules");
        fs::write(temp.path().join(&file_path), "# Test").unwrap();
        manager.create_backup("cursor", &[file_path]).unwrap();
        assert!(manager.has_backup("cursor"));

        // Delete the backup
        manager.delete_backup("cursor").unwrap();
        assert!(!manager.has_backup("cursor"));
    }

    #[test]
    fn test_list_backups() {
        let (temp, manager) = setup_test_repo();

        // Initially empty
        let backups = manager.list_backups().unwrap();
        assert!(backups.is_empty());

        // Create some files and backups
        fs::write(temp.path().join(".cursorrules"), "# Cursor").unwrap();
        fs::write(temp.path().join(".vscode").join("settings.json"), "{}").ok();
        fs::create_dir_all(temp.path().join(".vscode")).unwrap();
        fs::write(temp.path().join(".vscode/settings.json"), "{}").unwrap();

        manager
            .create_backup("cursor", &[PathBuf::from(".cursorrules")])
            .unwrap();
        manager
            .create_backup("vscode", &[PathBuf::from(".vscode/settings.json")])
            .unwrap();

        // List backups
        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 2);
    }

    #[test]
    fn test_backup_with_nested_file() {
        let (temp, manager) = setup_test_repo();

        // Create nested file
        let nested_dir = temp.path().join(".vscode");
        fs::create_dir_all(&nested_dir).unwrap();
        let file_path = PathBuf::from(".vscode/settings.json");
        let original_content = r#"{"editor.fontSize": 14}"#;
        fs::write(temp.path().join(&file_path), original_content).unwrap();

        // Create backup
        manager
            .create_backup("vscode", std::slice::from_ref(&file_path))
            .unwrap();

        // Delete original
        fs::remove_dir_all(&nested_dir).unwrap();

        // Restore
        manager.restore_backup("vscode").unwrap();

        // Verify
        assert!(temp.path().join(&file_path).exists());
        let content = fs::read_to_string(temp.path().join(&file_path)).unwrap();
        assert_eq!(content, original_content);
    }

    #[test]
    fn test_restore_nonexistent_backup() {
        let (_temp, manager) = setup_test_repo();

        let result = manager.restore_backup("nonexistent");
        assert!(result.is_err());
    }
}
