//! Cursor integration for Repository Manager.
//!
//! Manages `.cursorrules` file using managed blocks for rule content.

use crate::error::Result;
use crate::integration::{Rule, SyncContext, ToolIntegration};
use repo_blocks::upsert_block;
use repo_fs::{NormalizedPath, io};

/// Cursor integration.
///
/// Syncs rules to `.cursorrules` using managed blocks. Each rule is wrapped
/// in a block marker identified by its UUID.
#[derive(Debug, Default)]
pub struct CursorIntegration;

impl CursorIntegration {
    /// Creates a new Cursor integration.
    pub fn new() -> Self {
        Self
    }

    /// Load existing cursorrules content or empty string.
    fn load_content(path: &NormalizedPath) -> String {
        if path.exists() {
            io::read_text(path).unwrap_or_default()
        } else {
            String::new()
        }
    }
}

impl ToolIntegration for CursorIntegration {
    fn name(&self) -> &str {
        "cursor"
    }

    fn config_paths(&self) -> Vec<&str> {
        vec![".cursorrules"]
    }

    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let cursorrules_path = context.root.join(".cursorrules");

        // Load existing content
        let mut content = Self::load_content(&cursorrules_path);

        // Upsert each rule as a managed block
        for rule in rules {
            content = upsert_block(&content, &rule.id, &rule.content)?;
        }

        // Write content back
        io::write_text(&cursorrules_path, &content)?;

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
        let integration = CursorIntegration::new();
        assert_eq!(integration.name(), "cursor");
    }

    #[test]
    fn test_config_paths() {
        let integration = CursorIntegration::new();
        let paths = integration.config_paths();
        assert_eq!(paths, vec![".cursorrules"]);
    }

    #[test]
    fn test_sync_creates_cursorrules() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![
            Rule {
                id: "rule-1".to_string(),
                content: "First rule content".to_string(),
            },
            Rule {
                id: "rule-2".to_string(),
                content: "Second rule content".to_string(),
            },
        ];

        let integration = CursorIntegration::new();
        integration.sync(&context, &rules).unwrap();

        let cursorrules_path = temp_dir.path().join(".cursorrules");
        assert!(cursorrules_path.exists());

        let content = fs::read_to_string(&cursorrules_path).unwrap();
        assert!(content.contains("<!-- repo:block:rule-1 -->"));
        assert!(content.contains("First rule content"));
        assert!(content.contains("<!-- /repo:block:rule-1 -->"));
        assert!(content.contains("<!-- repo:block:rule-2 -->"));
        assert!(content.contains("Second rule content"));
        assert!(content.contains("<!-- /repo:block:rule-2 -->"));
    }

    #[test]
    fn test_sync_uses_managed_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        // Create with initial rule
        let context = SyncContext::new(root.clone());
        let rules = vec![Rule {
            id: "my-rule".to_string(),
            content: "Original content".to_string(),
        }];

        let integration = CursorIntegration::new();
        integration.sync(&context, &rules).unwrap();

        // Update the same rule
        let rules = vec![Rule {
            id: "my-rule".to_string(),
            content: "Updated content".to_string(),
        }];
        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

        // Should have updated content, not duplicated blocks
        assert!(content.contains("Updated content"));
        assert!(!content.contains("Original content"));

        // Should only have one block marker pair
        assert_eq!(content.matches("<!-- repo:block:my-rule -->").count(), 1);
    }

    #[test]
    fn test_sync_preserves_manual_content() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        // Create cursorrules with manual content
        let manual_content = "# Manual rules\n\nDo not modify managed blocks below.\n";
        fs::write(temp_dir.path().join(".cursorrules"), manual_content).unwrap();

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "auto-rule".to_string(),
            content: "Automated rule".to_string(),
        }];

        let integration = CursorIntegration::new();
        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

        // Manual content should be preserved
        assert!(content.contains("# Manual rules"));
        assert!(content.contains("Do not modify"));

        // Managed block should be added
        assert!(content.contains("<!-- repo:block:auto-rule -->"));
        assert!(content.contains("Automated rule"));
    }
}
