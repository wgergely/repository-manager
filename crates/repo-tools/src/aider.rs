//! Aider integration for Repository Manager.
//!
//! Manages `.aider.conf.yml` configuration file.
//!
//! Reference: https://aider.chat/docs/config/aider_conf.html

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates an Aider integration.
///
/// Configuration files:
/// - `.aider.conf.yml` - Project configuration (YAML)
/// - `CONVENTIONS.md` - Coding conventions (loaded via `read:` config)
///
/// Config priority: home dir < git root < current dir (last wins)
/// Environment variables: AIDER_xxx
pub fn aider_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Aider".into(),
            slug: "aider".into(),
            description: Some("Aider AI pair programming CLI".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".aider.conf.yml".into(),
            config_type: ConfigType::Yaml,
            additional_paths: vec!["CONVENTIONS.md".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: false,
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
        let integration = aider_integration();
        assert_eq!(integration.name(), "aider");
    }

    #[test]
    fn test_config_locations() {
        let integration = aider_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".aider.conf.yml");
        assert_eq!(locations[1].path, "CONVENTIONS.md");
    }
}
