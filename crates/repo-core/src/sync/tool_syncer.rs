//! Tool synchronization logic
//!
//! This module provides the `ToolSyncer` for coordinating the syncing of
//! tool configurations using projections. It handles creating, updating,
//! and removing tool configurations in the filesystem and ledger.
//!
//! Uses `repo-tools` integrations for proper tool-specific configuration
//! generation, supporting all built-in tools (cursor, vscode, claude,
//! windsurf, antigravity, gemini) plus schema-defined custom tools.
//!
//! Includes backup/restore functionality for tool configurations.

use crate::backup::BackupManager;
use crate::ledger::{Intent, Ledger, Projection, ProjectionKind};
use crate::projection::compute_checksum;
use crate::{Error, Result};
use repo_fs::NormalizedPath;
use repo_tools::{Rule, SyncContext, ToolDispatcher};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

/// Write content to a file safely (with symlink protection)
fn safe_write(path: &NormalizedPath, content: &str) -> Result<()> {
    repo_fs::io::write_text(path, content)
        .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))
}

/// Synchronizes tool configurations
///
/// The `ToolSyncer` coordinates the creation, update, and removal of tool
/// configurations. It uses the ledger to track what tools have been synced
/// and their projections.
///
/// Uses `repo-tools` integrations for proper tool-specific configuration
/// generation, supporting all built-in tools (cursor, vscode, claude,
/// windsurf, antigravity, gemini) plus schema-defined custom tools.
///
/// Features backup/restore for tool configurations when tools are removed/re-added.
pub struct ToolSyncer {
    /// Root path for the repository
    root: NormalizedPath,
    /// Whether to run in dry-run mode (simulate changes without writing)
    dry_run: bool,
    /// Backup manager for tool configuration backup/restore
    backup_manager: BackupManager,
    /// Tool dispatcher for routing to appropriate integrations
    dispatcher: ToolDispatcher,
    /// Resolved MCP server configuration from extensions.
    mcp_servers: Option<Value>,
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
        let dispatcher = ToolDispatcher::new();
        Self {
            root,
            dry_run,
            backup_manager,
            dispatcher,
            mcp_servers: None,
        }
    }

    /// Set the resolved MCP server configuration from extensions.
    pub fn with_mcp_servers(mut self, servers: Value) -> Self {
        self.mcp_servers = Some(servers);
        self
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

        // Ensure tool config files exist (creates them if needed)
        let config_files = self.ensure_tool_config_files(tool_name);

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
    /// 2. Optionally creates a backup of the tool's config files
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
        self.remove_tool_impl(tool_name, ledger, true)
    }

    /// Remove a tool with option to skip backup
    pub fn remove_tool_no_backup(
        &self,
        tool_name: &str,
        ledger: &mut Ledger,
    ) -> Result<Vec<String>> {
        self.remove_tool_impl(tool_name, ledger, false)
    }

    /// Internal implementation for tool removal with optional backup
    fn remove_tool_impl(
        &self,
        tool_name: &str,
        ledger: &mut Ledger,
        backup: bool,
    ) -> Result<Vec<String>> {
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

        // Create backup before deleting (if requested and not dry-run)
        if backup && !self.dry_run {
            let mut files_to_backup = Vec::new();
            for uuid in &intents {
                if let Some(intent) = ledger.get_intent(*uuid) {
                    for projection in intent.projections() {
                        files_to_backup.push(projection.file.clone());
                    }
                }
            }

            if !files_to_backup.is_empty() {
                match self
                    .backup_manager
                    .create_backup(tool_name, &files_to_backup)
                {
                    Ok(bkp) => {
                        actions.push(format!(
                            "Created backup for {} ({} files)",
                            tool_name,
                            bkp.metadata.files.len()
                        ));
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create backup for {}: {}", tool_name, e);
                    }
                }
            }
        }

        // Delete the files and remove intents
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

    /// Get config file paths for a tool (read-only, no side effects).
    ///
    /// Returns a list of (file_path, content) tuples for existing tool config files.
    /// Does NOT create or write any files.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn get_tool_config_files(&self, tool_name: &str) -> Vec<(String, String)> {
        if let Some(integration) = self.dispatcher.get_integration(tool_name) {
            integration
                .config_locations()
                .into_iter()
                .filter(|loc| !loc.is_directory)
                .map(|loc| {
                    let full_path = self.root.join(&loc.path);
                    let content = if full_path.exists() {
                        match std::fs::read_to_string(full_path.as_ref()) {
                            Ok(c) => c,
                            Err(e) => {
                                tracing::warn!("Failed to read {}: {}", loc.path, e);
                                String::new()
                            }
                        }
                    } else {
                        String::new()
                    };
                    (loc.path, content)
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Ensure tool config files exist, creating them with initial content if needed.
    ///
    /// This is the write-side counterpart to `get_tool_config_files`.
    fn ensure_tool_config_files(&self, tool_name: &str) -> Vec<(String, String)> {
        if let Some(integration) = self.dispatcher.get_integration(tool_name) {
            let context = self.make_sync_context();
            let initial_rule = Rule {
                id: format!("{}-init", tool_name),
                content: format!(
                    "# {} Configuration\n\nManaged by Repository Manager.\n",
                    tool_name
                ),
            };

            if !self.dry_run
                && let Err(e) = integration.sync(&context, &[initial_rule])
            {
                tracing::warn!("Failed to sync tool {}: {}", tool_name, e);
                return vec![];
            }

            integration
                .config_locations()
                .into_iter()
                .filter(|loc| !loc.is_directory)
                .map(|loc| {
                    let full_path = self.root.join(&loc.path);
                    let content = if full_path.exists() {
                        match std::fs::read_to_string(full_path.as_ref()) {
                            Ok(c) => c,
                            Err(e) => {
                                tracing::warn!("Failed to read {}: {}", loc.path, e);
                                String::new()
                            }
                        }
                    } else {
                        String::new()
                    };
                    (loc.path, content)
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Sync a tool with specific rules
    ///
    /// This method uses repo-tools integrations to sync rules to a tool's
    /// configuration files using managed blocks.
    pub fn sync_tool_with_rules(
        &self,
        tool_name: &str,
        rules: &[Rule],
        ledger: &mut Ledger,
    ) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        let intent_id = format!("tool:{}", tool_name);

        // Get the integration
        let integration = match self.dispatcher.get_integration(tool_name) {
            Some(i) => i,
            None => {
                actions.push(format!("No integration found for tool {}", tool_name));
                return Ok(actions);
            }
        };

        // Create sync context (with MCP servers if available)
        let context = self.make_sync_context();

        if self.dry_run {
            actions.push(format!(
                "[dry-run] Would sync {} rules to {}",
                rules.len(),
                tool_name
            ));
            return Ok(actions);
        }

        // Sync rules using the integration
        integration
            .sync(&context, rules)
            .map_err(|e| Error::Io(std::io::Error::other(format!("Tool sync failed: {}", e))))?;

        // Create projections for ledger
        let mut projections = Vec::new();
        for loc in integration.config_locations() {
            if loc.is_directory {
                continue;
            }
            let full_path = self.root.join(&loc.path);
            if full_path.exists() {
                let content = std::fs::read_to_string(full_path.as_ref())?;
                let checksum = compute_checksum(&content);
                projections.push(Projection {
                    tool: tool_name.to_string(),
                    file: PathBuf::from(&loc.path),
                    kind: ProjectionKind::FileManaged { checksum },
                });
                actions.push(format!("Synced {}", loc.path));
            }
        }

        // Update or create intent
        let existing = self.get_intents_by_id(ledger, &intent_id);
        if existing.is_empty() {
            let mut intent = Intent::new(intent_id.clone(), serde_json::json!({}));
            for projection in projections {
                intent.add_projection(projection);
            }
            ledger.add_intent(intent);
            actions.push(format!("Added intent {}", intent_id));
        } else {
            actions.push(format!("Updated {}", tool_name));
        }

        Ok(actions)
    }

    /// Create a SyncContext with MCP servers if available.
    fn make_sync_context(&self) -> SyncContext {
        let mut ctx = SyncContext::new(self.root.clone());
        if let Some(ref servers) = self.mcp_servers {
            ctx.mcp_servers = Some(servers.clone());
        }
        ctx
    }

    /// Check if a tool is supported
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.dispatcher.has_tool(tool_name)
    }

    /// List all available tools
    pub fn list_available_tools(&self) -> Vec<String> {
        self.dispatcher.list_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn sync_tool_writes_config_file_to_disk() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let mut ledger = Ledger::new();
        let actions = syncer.sync_tool("cursor", &mut ledger).unwrap();

        // Sync should produce at least one action describing what it did
        assert!(!actions.is_empty(), "sync_tool should report actions taken");

        // The ledger should now contain an intent for this tool
        let intents = ledger.find_by_rule("tool:cursor");
        assert_eq!(
            intents.len(),
            1,
            "Ledger should contain exactly one intent for cursor"
        );

        // The .cursorrules file should exist on disk
        let cursorrules = dir.path().join(".cursorrules");
        assert!(
            cursorrules.exists(),
            ".cursorrules should be created on disk"
        );

        let content = std::fs::read_to_string(&cursorrules).unwrap();
        assert!(!content.is_empty(), ".cursorrules should have content");
    }

    #[test]
    fn sync_tool_dry_run_does_not_write_files() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, true);

        let mut ledger = Ledger::new();
        let actions = syncer.sync_tool("cursor", &mut ledger).unwrap();

        // Dry run should still report actions
        assert!(
            !actions.is_empty(),
            "dry_run should still report planned actions"
        );

        // But no file should be created on disk
        let cursorrules = dir.path().join(".cursorrules");
        assert!(
            !cursorrules.exists(),
            "dry_run must NOT create files on disk"
        );
    }

    #[test]
    fn sync_tool_skips_already_synced_tool() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let mut ledger = Ledger::new();

        // First sync
        let actions1 = syncer.sync_tool("cursor", &mut ledger).unwrap();
        assert!(!actions1.is_empty());

        // Second sync should detect already-synced and skip
        let actions2 = syncer.sync_tool("cursor", &mut ledger).unwrap();
        let already_synced = actions2.iter().any(|a| a.contains("already synced"));
        assert!(
            already_synced,
            "Re-syncing should report 'already synced', got: {:?}",
            actions2
        );
    }

    #[test]
    fn sync_unknown_tool_returns_no_config_files() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let mut ledger = Ledger::new();
        let actions = syncer
            .sync_tool("nonexistent_tool_xyz", &mut ledger)
            .unwrap();

        // Should indicate no config files
        let no_config = actions.iter().any(|a| a.contains("No config files"));
        assert!(
            no_config,
            "Unknown tool should report 'No config files', got: {:?}",
            actions
        );

        // Ledger should not have an intent for this tool
        assert!(
            ledger.find_by_rule("tool:nonexistent_tool_xyz").is_empty(),
            "Ledger should not contain intent for unknown tool"
        );
    }

    #[test]
    fn test_get_tool_config_files_cursor() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("cursor");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, ".cursorrules");
        // Read-only: content is empty when file doesn't exist on disk
    }

    #[test]
    fn test_get_tool_config_files_vscode() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("vscode");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, ".vscode/settings.json");
        // Read-only: content is empty when file doesn't exist on disk
    }

    #[test]
    fn test_get_tool_config_files_claude() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("claude");
        assert_eq!(files.len(), 1);
        // Claude integration uses CLAUDE.md
        assert_eq!(files[0].0, "CLAUDE.md");
        // Read-only: content is empty when file doesn't exist on disk
    }

    #[test]
    fn test_get_tool_config_files_unknown() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        // Unknown tools return empty (no integration)
        let files = syncer.get_tool_config_files("unknown_tool");
        assert!(files.is_empty());
    }

    #[test]
    fn test_get_tool_config_files_windsurf() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("windsurf");
        assert_eq!(files.len(), 1);
        // Read-only: only verify path, not content
    }

    #[test]
    fn test_get_tool_config_files_antigravity() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("antigravity");
        assert_eq!(files.len(), 1);
        // Read-only: only verify path, not content
    }

    #[test]
    fn test_get_tool_config_files_gemini() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let files = syncer.get_tool_config_files("gemini");
        assert_eq!(files.len(), 1);
        // Read-only: only verify path, not content
    }

    #[test]
    fn test_has_tool() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        // Built-in tools
        assert!(syncer.has_tool("cursor"));
        assert!(syncer.has_tool("vscode"));
        assert!(syncer.has_tool("claude"));
        assert!(syncer.has_tool("windsurf"));
        assert!(syncer.has_tool("antigravity"));
        assert!(syncer.has_tool("gemini"));

        // Unknown tool
        assert!(!syncer.has_tool("unknown_tool"));
    }

    #[test]
    fn test_list_available_tools() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = ToolSyncer::new(root, false);

        let tools = syncer.list_available_tools();
        assert!(tools.contains(&"cursor".to_string()));
        assert!(tools.contains(&"vscode".to_string()));
        assert!(tools.contains(&"claude".to_string()));
        assert!(tools.contains(&"windsurf".to_string()));
        assert!(tools.contains(&"antigravity".to_string()));
        assert!(tools.contains(&"gemini".to_string()));
    }

    #[test]
    fn test_compute_checksum() {
        let content = "hello world";
        let checksum = compute_checksum(content);

        // Known SHA-256 of "hello world" with canonical prefix
        assert_eq!(
            checksum,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
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
