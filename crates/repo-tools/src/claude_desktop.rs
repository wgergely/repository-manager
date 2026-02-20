//! Claude Desktop (GUI app) integration for Repository Manager.
//!
//! Claude Desktop is the standalone GUI application, separate from Claude Code (the CLI).
//! It stores MCP server config in a user-level config file but has no project-level config.
//!
//! Reference: https://modelcontextprotocol.io/quickstart/user

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{
    ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta,
};

/// Creates a Claude Desktop integration.
///
/// Configuration:
/// - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
/// - Linux: `~/.config/Claude/claude_desktop_config.json`
/// - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
///
/// Claude Desktop supports MCP servers (stdio transport only) via the
/// `mcpServers` key in `claude_desktop_config.json`. It has no project-level
/// configuration — all MCP servers are user-scoped.
pub fn claude_desktop_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Claude Desktop".into(),
            slug: "claude_desktop".into(),
            description: Some("Claude Desktop GUI application".into()),
        },
        integration: ToolIntegrationConfig {
            // Claude Desktop has no project-level rules file;
            // use a placeholder that won't be synced (no custom instructions support)
            config_path: ".claude-desktop".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: false,
            supports_mcp: true,
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
        let integration = claude_desktop_integration();
        assert_eq!(integration.name(), "claude_desktop");
    }

    #[test]
    fn test_capabilities() {
        let def = claude_desktop_integration().definition().clone();
        assert!(!def.capabilities.supports_custom_instructions);
        assert!(def.capabilities.supports_mcp);
        assert!(!def.capabilities.supports_rules_directory);
    }
}
