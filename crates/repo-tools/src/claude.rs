//! Claude integration for Repository Manager.
//!
//! Manages `CLAUDE.md` and `.claude/rules/` using managed blocks for rule content.

use crate::error::Result;
use crate::integration::{Rule, SyncContext, ToolIntegration};
use repo_blocks::upsert_block;
use repo_fs::{NormalizedPath, io};

/// Claude integration.
///
/// Syncs rules to `CLAUDE.md` using managed blocks. Each rule is wrapped
/// in a block marker identified by its UUID.
#[derive(Debug, Default)]
pub struct ClaudeIntegration;

impl ClaudeIntegration {
    /// Creates a new Claude integration.
    pub fn new() -> Self {
        Self
    }

    /// Load existing CLAUDE.md content or empty string.
    fn load_content(path: &NormalizedPath) -> String {
        if path.exists() {
            io::read_text(path).unwrap_or_default()
        } else {
            String::new()
        }
    }
}

impl ToolIntegration for ClaudeIntegration {
    fn name(&self) -> &str {
        "claude"
    }

    fn config_paths(&self) -> Vec<&str> {
        vec!["CLAUDE.md", ".claude/rules/"]
    }

    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let claude_md_path = context.root.join("CLAUDE.md");

        // Load existing content
        let mut content = Self::load_content(&claude_md_path);

        // Upsert each rule as a managed block
        for rule in rules {
            content = upsert_block(&content, &rule.id, &rule.content)?;
        }

        // Write content back
        io::write_text(&claude_md_path, &content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = ClaudeIntegration::new();
        assert_eq!(integration.name(), "claude");
    }

    #[test]
    fn test_config_paths() {
        let integration = ClaudeIntegration::new();
        let paths = integration.config_paths();
        assert_eq!(paths, vec!["CLAUDE.md", ".claude/rules/"]);
    }

    #[test]
    fn test_sync_creates_claude_md() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![
            Rule {
                id: "project-context".to_string(),
                content: "This is a Rust project using cargo.".to_string(),
            },
            Rule {
                id: "coding-standards".to_string(),
                content: "Follow Rust best practices.".to_string(),
            },
        ];

        let integration = ClaudeIntegration::new();
        integration.sync(&context, &rules).unwrap();

        let claude_md_path = temp_dir.path().join("CLAUDE.md");
        assert!(claude_md_path.exists());

        let content = fs::read_to_string(&claude_md_path).unwrap();
        assert!(content.contains("<!-- repo:block:project-context -->"));
        assert!(content.contains("This is a Rust project using cargo."));
        assert!(content.contains("<!-- /repo:block:project-context -->"));
        assert!(content.contains("<!-- repo:block:coding-standards -->"));
        assert!(content.contains("Follow Rust best practices."));
        assert!(content.contains("<!-- /repo:block:coding-standards -->"));
    }

    #[test]
    fn test_sync_uses_managed_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        // Create with initial rule
        let context = SyncContext::new(root.clone());
        let rules = vec![Rule {
            id: "context".to_string(),
            content: "Initial context".to_string(),
        }];

        let integration = ClaudeIntegration::new();
        integration.sync(&context, &rules).unwrap();

        // Update the same rule
        let rules = vec![Rule {
            id: "context".to_string(),
            content: "Updated context".to_string(),
        }];
        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

        // Should have updated content, not duplicated blocks
        assert!(content.contains("Updated context"));
        assert!(!content.contains("Initial context"));

        // Should only have one block marker pair
        assert_eq!(content.matches("<!-- repo:block:context -->").count(), 1);
    }

    #[test]
    fn test_sync_preserves_manual_content() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        // Create CLAUDE.md with manual content
        let manual_content = "# Project Documentation\n\nThis is a manual section.\n";
        fs::write(temp_dir.path().join("CLAUDE.md"), manual_content).unwrap();

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "auto-context".to_string(),
            content: "Managed context".to_string(),
        }];

        let integration = ClaudeIntegration::new();
        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

        // Manual content should be preserved
        assert!(content.contains("# Project Documentation"));
        assert!(content.contains("This is a manual section."));

        // Managed block should be added
        assert!(content.contains("<!-- repo:block:auto-context -->"));
        assert!(content.contains("Managed context"));
    }
}
