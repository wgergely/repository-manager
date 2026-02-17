//! JetBrains AI Assistant integration for Repository Manager.
//!
//! Manages `.aiassistant/rules/` directory for project-specific AI rules.
//!
//! Reference: https://www.jetbrains.com/help/ai-assistant/configure-project-rules.html

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates a JetBrains AI Assistant integration.
///
/// Configuration files:
/// - `.aiassistant/rules/` - Directory of rule files (*.md)
/// - `.aiignore` - Files to exclude from AI (gitignore syntax)
/// - `.noai` - Empty file to disable AI for entire project
///
/// Rule types: Always, Manually (@rule:), By Model Decision, By File Patterns, Off
/// Also reads: .cursorignore, .codeiumignore, .aiexclude
pub fn jetbrains_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "JetBrains AI".into(),
            slug: "jetbrains".into(),
            description: Some("JetBrains AI Assistant for IntelliJ IDEs".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".aiassistant/rules/".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![".aiignore".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true, // Supports MCP servers
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::ToolIntegration;

    #[test]
    fn test_name() {
        let integration = jetbrains_integration();
        assert_eq!(integration.name(), "jetbrains");
    }

    #[test]
    fn test_config_locations() {
        let integration = jetbrains_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".aiassistant/rules/");
        assert!(locations[0].is_directory);
        assert_eq!(locations[1].path, ".aiignore");
    }
}
