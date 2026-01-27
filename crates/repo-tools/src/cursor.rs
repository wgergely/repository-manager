//! Cursor integration for Repository Manager.
//!
//! Manages `.cursorrules` file using managed blocks for rule content.

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a Cursor integration.
///
/// Returns a GenericToolIntegration configured for Cursor's `.cursorrules` file.
/// Uses raw content mode (no headers) for backward compatibility.
pub fn cursor_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Cursor".into(),
            slug: "cursor".into(),
            description: Some("Cursor AI IDE".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".cursorrules".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: false,
        },
        schema_keys: None,
    })
    .with_raw_content(true)
}

/// Type alias for backward compatibility.
///
/// Prefer using `cursor_integration()` factory function for new code.
pub type CursorIntegration = GenericToolIntegration;

/// Creates a new Cursor integration (legacy API).
///
/// # Deprecated
/// Use `cursor_integration()` instead.
pub fn new() -> GenericToolIntegration {
    cursor_integration()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{Rule, SyncContext, ToolIntegration};
    use repo_fs::NormalizedPath;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = cursor_integration();
        assert_eq!(integration.name(), "cursor");
    }

    #[test]
    fn test_config_paths() {
        let integration = cursor_integration();
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

        let integration = cursor_integration();
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

        let integration = cursor_integration();
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

        let integration = cursor_integration();
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
