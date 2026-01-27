//! Rule synchronization logic
//!
//! This module provides the `RuleSyncer` for synchronizing rules from
//! `.repository/rules/` to tool configurations. Rules are markdown files
//! that get combined and written to tool-specific config files.

use crate::ledger::{Intent, Ledger, Projection, ProjectionKind};
use crate::projection::ProjectionWriter;
use crate::Result;
use repo_fs::NormalizedPath;
use std::fs;
use std::path::PathBuf;

use super::compute_checksum_from_content;

/// A loaded rule file
#[derive(Debug, Clone)]
pub struct RuleFile {
    /// The rule identifier (filename without extension)
    pub id: String,
    /// The content of the rule file
    pub content: String,
}

/// Synchronizes rules to tool configurations
///
/// The `RuleSyncer` reads rule files from `.repository/rules/` and
/// combines them into tool-specific configuration files like `.cursorrules`.
pub struct RuleSyncer {
    /// Root path for the repository
    root: NormalizedPath,
    /// Whether to run in dry-run mode (simulate changes without writing)
    dry_run: bool,
}

impl RuleSyncer {
    /// Create a new `RuleSyncer`
    ///
    /// # Arguments
    ///
    /// * `root` - The root path of the repository
    /// * `dry_run` - If true, simulate changes without modifying the filesystem
    pub fn new(root: NormalizedPath, dry_run: bool) -> Self {
        Self { root, dry_run }
    }

    /// Load all rules from the rules directory
    ///
    /// Reads all `.md` files from `.repository/rules/` and returns them
    /// as `RuleFile` structs sorted by filename.
    ///
    /// # Returns
    ///
    /// A vector of `RuleFile` structs, empty if the rules directory doesn't exist.
    pub fn load_rules(&self) -> Result<Vec<RuleFile>> {
        let rules_dir = self.root.join(".repository/rules");
        let native_path = rules_dir.to_native();

        if !native_path.exists() {
            return Ok(Vec::new());
        }

        let mut rules = Vec::new();
        for entry in fs::read_dir(&native_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                let id = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let content = fs::read_to_string(&path)?;
                rules.push(RuleFile { id, content });
            }
        }

        // Sort by ID for consistent output
        rules.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(rules)
    }

    /// Sync all rules to applicable tool configurations
    ///
    /// This method:
    /// 1. Loads all rules from `.repository/rules/`
    /// 2. Combines them into a single content block
    /// 3. Writes to each tool's rules file (e.g., `.cursorrules`)
    /// 4. Updates the ledger with the projection
    ///
    /// # Arguments
    ///
    /// * `tools` - List of tool names to sync rules to
    /// * `ledger` - Mutable reference to the ledger
    ///
    /// # Returns
    ///
    /// A list of action descriptions taken during the sync.
    pub fn sync_rules(&self, tools: &[String], ledger: &mut Ledger) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        let rules = self.load_rules()?;

        if rules.is_empty() {
            actions.push("No rules found in .repository/rules/".to_string());
            return Ok(actions);
        }

        // Combine all rules into a single content block
        let combined_rules = self.combine_rules(&rules);

        let writer = ProjectionWriter::new(self.root.clone(), self.dry_run);

        // Apply rules to each applicable tool
        for tool in tools {
            let rules_file = self.get_rules_file_for_tool(tool);

            if let Some(file) = rules_file {
                let intent_id = format!("rules:{}", tool);

                // Check if already synced with same checksum
                let existing = ledger.find_by_rule(&intent_id);
                let new_checksum = compute_checksum_from_content(&combined_rules);

                // Check if content has changed
                let needs_update = if let Some(existing_intent) = existing.first() {
                    // Check if checksum differs
                    existing_intent.projections().iter().any(|p| {
                        if let ProjectionKind::FileManaged { checksum } = &p.kind {
                            checksum != &new_checksum
                        } else {
                            true
                        }
                    })
                } else {
                    true
                };

                if !needs_update {
                    actions.push(format!("Rules for {} unchanged", tool));
                    continue;
                }

                // Create projection for writing
                let projection = Projection::file_managed(
                    tool.clone(),
                    PathBuf::from(&file),
                    String::new(), // Checksum will be updated after
                );

                // Write the file
                let action = writer.apply(&projection, &combined_rules)?;
                actions.push(action);

                // Create intent with updated checksum
                let mut intent = Intent::new(intent_id.clone(), serde_json::Value::Null);
                intent.add_projection(Projection::file_managed(
                    tool.clone(),
                    PathBuf::from(&file),
                    new_checksum,
                ));

                if !self.dry_run {
                    // Remove old intent if exists
                    if let Some(existing_intent) = existing.first() {
                        ledger.remove_intent(existing_intent.uuid);
                    }
                    ledger.add_intent(intent);
                    actions.push(format!("Updated ledger for rules:{}", tool));
                }
            }
        }

        Ok(actions)
    }

    /// Get the rules file path for a specific tool
    ///
    /// Returns the path to the rules file for the tool, or None if the tool
    /// doesn't support rules files.
    fn get_rules_file_for_tool(&self, tool: &str) -> Option<String> {
        match tool {
            "cursor" => Some(".cursorrules".to_string()),
            // Claude uses CLAUDE.md which we don't manage through rules yet
            "claude" | "claude-desktop" => None,
            // VSCode doesn't have a standard rules file
            "vscode" => None,
            _ => None,
        }
    }

    /// Combine multiple rules into a single content block
    ///
    /// Each rule is formatted with its ID as a header and separated by
    /// horizontal rules for readability.
    fn combine_rules(&self, rules: &[RuleFile]) -> String {
        let header = "# Repository Rules\n\n\
            # This file is auto-generated by repository-manager.\n\
            # Do not edit directly - modify rules in .repository/rules/ instead.\n";

        let rule_content = rules
            .iter()
            .map(|r| format!("## {}\n\n{}", r.id, r.content.trim()))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        format!("{}\n\n{}", header, rule_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rule_syncer_new() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = RuleSyncer::new(root, false);
        assert!(!syncer.dry_run);
    }

    #[test]
    fn test_rule_syncer_dry_run() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = RuleSyncer::new(root, true);
        assert!(syncer.dry_run);
    }

    #[test]
    fn test_load_rules_empty_when_no_dir() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = RuleSyncer::new(root, false);

        let rules = syncer.load_rules().unwrap();
        assert!(rules.is_empty());
    }

    #[test]
    fn test_load_rules_finds_md_files() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());

        // Create rules directory and files
        let rules_dir = dir.path().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("code-style.md"), "Use 4 spaces").unwrap();
        fs::write(rules_dir.join("naming.md"), "Use snake_case").unwrap();
        fs::write(rules_dir.join("ignore.txt"), "Not a rule").unwrap();

        let syncer = RuleSyncer::new(root, false);
        let rules = syncer.load_rules().unwrap();

        assert_eq!(rules.len(), 2);
        // Should be sorted alphabetically
        assert_eq!(rules[0].id, "code-style");
        assert_eq!(rules[1].id, "naming");
    }

    #[test]
    fn test_combine_rules() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = RuleSyncer::new(root, false);

        let rules = vec![
            RuleFile {
                id: "style".to_string(),
                content: "Use consistent formatting".to_string(),
            },
            RuleFile {
                id: "naming".to_string(),
                content: "Use descriptive names".to_string(),
            },
        ];

        let combined = syncer.combine_rules(&rules);

        assert!(combined.contains("# Repository Rules"));
        assert!(combined.contains("## style"));
        assert!(combined.contains("## naming"));
        assert!(combined.contains("Use consistent formatting"));
        assert!(combined.contains("Use descriptive names"));
        assert!(combined.contains("---"));
    }

    #[test]
    fn test_get_rules_file_for_tool() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = RuleSyncer::new(root, false);

        assert_eq!(
            syncer.get_rules_file_for_tool("cursor"),
            Some(".cursorrules".to_string())
        );
        assert_eq!(syncer.get_rules_file_for_tool("claude"), None);
        assert_eq!(syncer.get_rules_file_for_tool("vscode"), None);
        assert_eq!(syncer.get_rules_file_for_tool("unknown"), None);
    }

    #[test]
    fn test_sync_rules_no_rules() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());
        let syncer = RuleSyncer::new(root, false);
        let mut ledger = Ledger::new();

        let tools = vec!["cursor".to_string()];
        let actions = syncer.sync_rules(&tools, &mut ledger).unwrap();

        assert!(actions.iter().any(|a| a.contains("No rules found")));
        assert!(ledger.intents().is_empty());
    }

    #[test]
    fn test_sync_rules_creates_cursorrules() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());

        // Create rules directory and files
        let rules_dir = dir.path().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("code-style.md"), "Use 4 spaces").unwrap();

        let syncer = RuleSyncer::new(root.clone(), false);
        let mut ledger = Ledger::new();

        let tools = vec!["cursor".to_string()];
        let actions = syncer.sync_rules(&tools, &mut ledger).unwrap();

        // Should have created the file
        assert!(actions.iter().any(|a| a.contains("Created")));
        // Ledger should have one intent
        assert_eq!(ledger.intents().len(), 1);
        assert_eq!(ledger.intents()[0].id, "rules:cursor");
        // File should exist
        let cursorrules = root.join(".cursorrules");
        assert!(cursorrules.exists());

        // Content should include the rule
        let content = fs::read_to_string(cursorrules.as_ref()).unwrap();
        assert!(content.contains("Use 4 spaces"));
    }

    #[test]
    fn test_sync_rules_dry_run() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());

        // Create rules directory and files
        let rules_dir = dir.path().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("code-style.md"), "Use 4 spaces").unwrap();

        let syncer = RuleSyncer::new(root.clone(), true);
        let mut ledger = Ledger::new();

        let tools = vec!["cursor".to_string()];
        let actions = syncer.sync_rules(&tools, &mut ledger).unwrap();

        // Should have dry-run action
        assert!(actions.iter().any(|a| a.contains("[dry-run]")));
        // Ledger should be empty (no actual intent added in dry-run)
        assert!(ledger.intents().is_empty());
        // File should not be created
        let cursorrules = root.join(".cursorrules");
        assert!(!cursorrules.exists());
    }

    #[test]
    fn test_sync_rules_updates_on_change() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());

        // Create rules directory and files
        let rules_dir = dir.path().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("code-style.md"), "Use 4 spaces").unwrap();

        let syncer = RuleSyncer::new(root.clone(), false);
        let mut ledger = Ledger::new();
        let tools = vec!["cursor".to_string()];

        // First sync
        syncer.sync_rules(&tools, &mut ledger).unwrap();
        let original_uuid = ledger.intents()[0].uuid;

        // Modify the rule
        fs::write(rules_dir.join("code-style.md"), "Use 2 spaces").unwrap();

        // Second sync should update
        let actions = syncer.sync_rules(&tools, &mut ledger).unwrap();

        assert!(actions.iter().any(|a| a.contains("Created") || a.contains("Updated")));
        // Should still have one intent (old removed, new added)
        assert_eq!(ledger.intents().len(), 1);
        // UUID should be different (new intent)
        assert_ne!(ledger.intents()[0].uuid, original_uuid);

        // Content should have new value
        let content = fs::read_to_string(root.join(".cursorrules").as_ref()).unwrap();
        assert!(content.contains("Use 2 spaces"));
    }

    #[test]
    fn test_sync_rules_skips_unchanged() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());

        // Create rules directory and files
        let rules_dir = dir.path().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("code-style.md"), "Use 4 spaces").unwrap();

        let syncer = RuleSyncer::new(root, false);
        let mut ledger = Ledger::new();
        let tools = vec!["cursor".to_string()];

        // First sync
        syncer.sync_rules(&tools, &mut ledger).unwrap();
        let original_uuid = ledger.intents()[0].uuid;

        // Second sync without changes
        let actions = syncer.sync_rules(&tools, &mut ledger).unwrap();

        assert!(actions.iter().any(|a| a.contains("unchanged")));
        // UUID should be the same
        assert_eq!(ledger.intents()[0].uuid, original_uuid);
    }

    #[test]
    fn test_sync_rules_ignores_unsupported_tools() {
        let dir = tempdir().unwrap();
        let root = NormalizedPath::new(dir.path());

        // Create rules directory and files
        let rules_dir = dir.path().join(".repository/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(rules_dir.join("code-style.md"), "Use 4 spaces").unwrap();

        let syncer = RuleSyncer::new(root, false);
        let mut ledger = Ledger::new();

        // Only sync to tools that don't support rules
        let tools = vec!["vscode".to_string(), "unknown".to_string()];
        let actions = syncer.sync_rules(&tools, &mut ledger).unwrap();

        // Should not have created any files
        assert!(!actions.iter().any(|a| a.contains("Created")));
        // Ledger should be empty
        assert!(ledger.intents().is_empty());
    }
}
