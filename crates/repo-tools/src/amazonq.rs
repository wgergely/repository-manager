//! Amazon Q Developer integration for Repository Manager.
//!
//! Manages `.amazonq/rules/` directory for project-specific rules.
//!
//! Reference: https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/context-project-rules.html

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates an Amazon Q Developer integration.
///
/// Configuration files:
/// - `.amazonq/rules/` - Directory of rule files (*.md)
///
/// Rules are automatically applied to all chat sessions.
/// Individual rules can be toggled on/off via the Rules button in chat.
pub fn amazonq_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Amazon Q".into(),
            slug: "amazonq".into(),
            description: Some("Amazon Q Developer AI assistant".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".amazonq/rules/".into(),
            config_type: ConfigType::Markdown,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::ToolIntegration;

    #[test]
    fn test_name() {
        let integration = amazonq_integration();
        assert_eq!(integration.name(), "amazonq");
    }

    #[test]
    fn test_config_locations() {
        let integration = amazonq_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].path, ".amazonq/rules/");
        assert!(locations[0].is_directory);
    }
}
