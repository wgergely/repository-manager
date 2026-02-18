//! Antigravity integration for Repository Manager.
//!
//! Manages `.agent/rules.md` file using managed blocks for rule content.

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates an Antigravity integration.
///
/// Returns a GenericToolIntegration configured for Antigravity's `.agent/rules.md` file.
/// Supports custom instructions and rules directory.
pub fn antigravity_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Antigravity".into(),
            slug: "antigravity".into(),
            description: Some("Antigravity AI assistant".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".agent/rules.md".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
}

/// Type alias for backward compatibility.
///
/// Prefer using `antigravity_integration()` factory function for new code.
pub type AntigravityIntegration = GenericToolIntegration;

/// Creates a new Antigravity integration (legacy API).
///
/// # Deprecated
/// Use `antigravity_integration()` instead.
#[deprecated(note = "Use antigravity_integration() instead")]
pub fn new() -> GenericToolIntegration {
    antigravity_integration()
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
        let integration = antigravity_integration();
        assert_eq!(integration.name(), "antigravity");
    }

    #[test]
    fn test_config_locations() {
        let integration = antigravity_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].path, ".agent/rules.md");
    }

    #[test]
    fn test_sync_creates_rules_md() {
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

        let integration = antigravity_integration();
        integration.sync(&context, &rules).unwrap();

        let rules_path = temp_dir.path().join(".agent/rules.md");
        assert!(rules_path.exists());

        let content = fs::read_to_string(&rules_path).unwrap();
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

        let integration = antigravity_integration();
        integration.sync(&context, &rules).unwrap();

        // Update the same rule
        let rules = vec![Rule {
            id: "my-rule".to_string(),
            content: "Updated content".to_string(),
        }];
        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp_dir.path().join(".agent/rules.md")).unwrap();

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

        // Create .agent directory and rules.md with manual content
        fs::create_dir_all(temp_dir.path().join(".agent")).unwrap();
        let manual_content = "# Manual rules\n\nDo not modify managed blocks below.\n";
        fs::write(temp_dir.path().join(".agent/rules.md"), manual_content).unwrap();

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "auto-rule".to_string(),
            content: "Automated rule".to_string(),
        }];

        let integration = antigravity_integration();
        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp_dir.path().join(".agent/rules.md")).unwrap();

        // Manual content should be preserved
        assert!(content.contains("# Manual rules"));
        assert!(content.contains("Do not modify"));

        // Managed block should be added
        assert!(content.contains("<!-- repo:block:auto-rule -->"));
        assert!(content.contains("Automated rule"));
    }
}
