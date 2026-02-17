//! Roo Code integration for Repository Manager.
//!
//! Manages `.roo/rules/` directory and `.roomodes` file.
//!
//! Reference: https://docs.roocode.com/features/custom-instructions

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates a Roo Code integration.
///
/// Configuration files:
/// - `.roo/rules/` - Directory of instruction files (*.md, *.txt)
/// - `.roo/rules-{mode}/` - Mode-specific rules directories
/// - `.roomodes` - Custom modes configuration (YAML or JSON)
///
/// Files are loaded recursively in alphabetical order.
/// Workspace rules override global rules (~/.roo/rules/).
pub fn roo_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Roo Code".into(),
            slug: "roo".into(),
            description: Some("Roo Code AI assistant (fork of Cline)".into()),
        },
        integration: ToolIntegrationConfig {
            // Primary path is the rules directory
            config_path: ".roo/rules/".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![".roomodes".into()],
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
        let integration = roo_integration();
        assert_eq!(integration.name(), "roo");
    }

    #[test]
    fn test_config_locations() {
        let integration = roo_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".roo/rules/");
        assert!(locations[0].is_directory);
        assert_eq!(locations[1].path, ".roomodes");
    }
}
