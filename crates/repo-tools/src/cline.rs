//! Cline (VS Code) integration for Repository Manager.
//!
//! Manages `.clinerules` file or `.clinerules/` directory using managed blocks.
//!
//! Reference: https://docs.cline.bot/features/cline-rules

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates a Cline integration.
///
/// Configuration files:
/// - `.clinerules` - Single rules file (Markdown/Text)
/// - `.clinerules/` - Directory of rule files (*.md)
///
/// Cline also reads `.cursorrules` and `AGENTS.md` as fallbacks.
/// Files in directory are processed alphabetically (use `01-`, `02-` prefixes).
pub fn cline_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Cline".into(),
            slug: "cline".into(),
            description: Some("Cline AI coding assistant for VS Code".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".clinerules".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![".clinerules/".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
    .with_raw_content(true) // No headers, direct content
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
        let integration = cline_integration();
        assert_eq!(integration.name(), "cline");
    }

    #[test]
    fn test_config_locations() {
        let integration = cline_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".clinerules");
        assert_eq!(locations[1].path, ".clinerules/");
        assert!(locations[1].is_directory);
    }

    #[test]
    fn test_sync_creates_clinerules() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "coding-style".to_string(),
            content: "Use TypeScript strict mode.".to_string(),
        }];

        let integration = cline_integration();
        integration.sync(&context, &rules).unwrap();

        let path = temp_dir.path().join(".clinerules");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Use TypeScript strict mode"));
        // Raw content mode - no headers
        assert!(!content.contains("## coding-style"));
    }
}
