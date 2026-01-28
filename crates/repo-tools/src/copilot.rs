//! GitHub Copilot integration for Repository Manager.
//!
//! Manages `.github/copilot-instructions.md` file using managed blocks.
//! Also supports path-specific instructions in `.github/instructions/`.
//!
//! Reference: https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a GitHub Copilot integration.
///
/// Configuration files:
/// - `.github/copilot-instructions.md` - Main instructions file (Markdown)
/// - `.github/instructions/` - Directory for path-specific `.instructions.md` files
///
/// Format: Markdown with optional YAML frontmatter for path-specific files:
/// ```yaml
/// ---
/// applyTo: "**/*.py"
/// ---
/// ```
pub fn copilot_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "GitHub Copilot".into(),
            slug: "copilot".into(),
            description: Some("GitHub Copilot AI coding assistant".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".github/copilot-instructions.md".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![".github/instructions/".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
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
        let integration = copilot_integration();
        assert_eq!(integration.name(), "copilot");
    }

    #[test]
    fn test_config_locations() {
        let integration = copilot_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".github/copilot-instructions.md");
        assert!(!locations[0].is_directory);
        assert_eq!(locations[1].path, ".github/instructions/");
        assert!(locations[1].is_directory);
    }

    #[test]
    fn test_sync_creates_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        // Create .github directory
        fs::create_dir_all(temp_dir.path().join(".github")).unwrap();

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "python-style".to_string(),
            content: "Use type hints for all function parameters.".to_string(),
        }];

        let integration = copilot_integration();
        integration.sync(&context, &rules).unwrap();

        let path = temp_dir.path().join(".github/copilot-instructions.md");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("python-style"));
        assert!(content.contains("Use type hints"));
    }
}
