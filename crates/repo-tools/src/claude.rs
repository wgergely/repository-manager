//! Claude integration for Repository Manager.
//!
//! Manages `CLAUDE.md` and `.claude/rules/` using managed blocks for rule content.

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a Claude integration.
///
/// Returns a GenericToolIntegration configured for Claude's `CLAUDE.md` file.
/// Uses raw content mode (no headers) for backward compatibility.
pub fn claude_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Claude".into(),
            slug: "claude".into(),
            description: Some("Anthropic Claude AI assistant".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: "CLAUDE.md".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![".claude/rules/".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
    .with_raw_content(true)
}

/// Type alias for backward compatibility.
///
/// Prefer using `claude_integration()` factory function for new code.
pub type ClaudeIntegration = GenericToolIntegration;

/// Creates a new Claude integration (legacy API).
///
/// # Deprecated
/// Use `claude_integration()` instead.
pub fn new() -> GenericToolIntegration {
    claude_integration()
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
        let integration = claude_integration();
        assert_eq!(integration.name(), "claude");
    }

    #[test]
    fn test_config_paths() {
        let integration = claude_integration();
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

        let integration = claude_integration();
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

        let integration = claude_integration();
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

        let integration = claude_integration();
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
