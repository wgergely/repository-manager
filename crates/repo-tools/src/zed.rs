//! Zed editor integration for Repository Manager.
//!
//! Manages `.rules` file for AI agent instructions.
//!
//! Reference: https://zed.dev/docs/ai/rules

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates a Zed editor integration.
///
/// Configuration files:
/// - `.rules` - Project rules file (highest priority)
/// - `.zed/settings.json` - Project settings (for AI model config)
///
/// Priority order: .rules > .cursorrules > .windsurfrules > .clinerules >
///   .github/copilot-instructions.md > AGENT.md > AGENTS.md > CLAUDE.md > GEMINI.md
///
/// Only the first matching file is loaded.
pub fn zed_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Zed".into(),
            slug: "zed".into(),
            description: Some("Zed code editor with AI agent".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".rules".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![".zed/settings.json".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true,
            supports_rules_directory: false,
        },
        schema_keys: None,
    })
    .with_raw_content(true) // Direct content, no headers
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
        let integration = zed_integration();
        assert_eq!(integration.name(), "zed");
    }

    #[test]
    fn test_config_locations() {
        let integration = zed_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".rules");
        assert_eq!(locations[1].path, ".zed/settings.json");
    }

    #[test]
    fn test_sync_creates_rules() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "code-style".to_string(),
            content: "Use Rust best practices.".to_string(),
        }];

        let integration = zed_integration();
        integration.sync(&context, &rules).unwrap();

        let path = temp_dir.path().join(".rules");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Use Rust best practices"));
    }
}
