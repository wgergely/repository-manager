//! Antigravity integration for Repository Manager.
//!
//! Manages `.agent/rules/` directory with per-rule files for rule content.

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates an Antigravity integration.
///
/// Returns a GenericToolIntegration configured for Antigravity's `.agent/rules/` directory.
/// Supports custom instructions and rules directory.
pub fn antigravity_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Antigravity".into(),
            slug: "antigravity".into(),
            description: Some("Antigravity AI assistant".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".agent/rules/".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true,
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
        assert_eq!(locations[0].path, ".agent/rules/");
        assert!(locations[0].is_directory);
    }

    #[test]
    fn test_sync_creates_rules_directory() {
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

        // Should create a directory, not a single file
        let rules_dir = temp_dir.path().join(".agent/rules");
        assert!(rules_dir.is_dir(), ".agent/rules/ should be a directory");

        // Should have individual rule files
        let rule1_path = rules_dir.join("01-rule-1.md");
        let rule2_path = rules_dir.join("02-rule-2.md");
        assert!(rule1_path.exists(), "Per-rule file for rule-1 should exist");
        assert!(rule2_path.exists(), "Per-rule file for rule-2 should exist");

        let content1 = fs::read_to_string(&rule1_path).unwrap();
        assert!(content1.contains("First rule content"));

        let content2 = fs::read_to_string(&rule2_path).unwrap();
        assert!(content2.contains("Second rule content"));
    }

    #[test]
    fn test_sync_overwrites_rule_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

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

        let rule_path = temp_dir.path().join(".agent/rules/01-my-rule.md");
        let content = fs::read_to_string(rule_path).unwrap();

        // Should have updated content
        assert!(content.contains("Updated content"));
        assert!(!content.contains("Original content"));
    }
}
