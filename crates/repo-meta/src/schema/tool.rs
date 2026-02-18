//! Tool definition schema - loaded from .repository/tools/*.toml
//!
//! Tool definitions describe how to integrate with external tools like
//! IDEs, AI assistants, and other development tools.
//!
//! # Example TOML
//!
//! ```toml
//! [meta]
//! name = "Cursor"
//! slug = "cursor"
//! description = "AI-first code editor"
//!
//! [integration]
//! config_path = ".cursorrules"
//! type = "text"
//!
//! [capabilities]
//! supports_custom_instructions = true
//! supports_mcp = true
//! supports_rules_directory = false
//!
//! [schema]
//! instruction_key = "global_instructions"
//! mcp_key = "mcpServers"
//! ```

use serde::{Deserialize, Serialize};

/// Complete tool definition loaded from TOML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolDefinition {
    /// Basic metadata about the tool
    pub meta: ToolMeta,
    /// Integration configuration (paths, formats)
    pub integration: ToolIntegrationConfig,
    /// Tool capabilities
    #[serde(default)]
    pub capabilities: ToolCapabilities,
    /// Schema keys for JSON-based configs
    #[serde(default, rename = "schema")]
    pub schema_keys: Option<ToolSchemaKeys>,
}

/// Basic metadata about a tool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolMeta {
    /// Human-readable display name (e.g., "Cursor")
    pub name: String,
    /// Machine-readable identifier (e.g., "cursor")
    pub slug: String,
    /// Optional description of the tool
    #[serde(default)]
    pub description: Option<String>,
}

/// Configuration for how to integrate with the tool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolIntegrationConfig {
    /// Primary config file path relative to repo root (e.g., ".cursorrules")
    pub config_path: String,
    /// File format type
    #[serde(rename = "type")]
    pub config_type: ConfigType,
    /// Additional config paths (e.g., directories like ".cursor/rules/")
    #[serde(default)]
    pub additional_paths: Vec<String>,
}

/// Configuration file format types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConfigType {
    /// Plain text file (e.g., .cursorrules)
    #[default]
    Text,
    /// JSON format (e.g., settings.json)
    Json,
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// Markdown format (e.g., CLAUDE.md)
    Markdown,
}

/// Tool capabilities flags
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ToolCapabilities {
    /// Tool supports custom instructions/rules
    #[serde(default)]
    pub supports_custom_instructions: bool,
    /// Tool supports MCP (Model Context Protocol) servers
    #[serde(default)]
    pub supports_mcp: bool,
    /// Tool supports a rules directory (e.g., .cursor/rules/)
    #[serde(default)]
    pub supports_rules_directory: bool,
}

/// Schema keys for JSON-based configuration files
///
/// These specify where in the JSON structure to place various settings.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ToolSchemaKeys {
    /// JSON key for custom instructions (e.g., "global_instructions")
    pub instruction_key: Option<String>,
    /// JSON key for MCP servers configuration
    pub mcp_key: Option<String>,
    /// JSON key for Python interpreter path
    pub python_path_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_type_default() {
        let config_type = ConfigType::default();
        assert_eq!(config_type, ConfigType::Text);
    }

    #[test]
    fn test_capabilities_default() {
        let caps = ToolCapabilities::default();
        assert!(!caps.supports_custom_instructions);
        assert!(!caps.supports_mcp);
        assert!(!caps.supports_rules_directory);
    }

    #[test]
    fn test_parse_tool_definition_minimal() {
        let toml = r#"
[meta]
name = "Cursor"
slug = "cursor"

[integration]
config_path = ".cursorrules"
type = "text"
"#;

        let def: ToolDefinition = toml::from_str(toml).unwrap();
        assert_eq!(def.meta.name, "Cursor");
        assert_eq!(def.meta.slug, "cursor");
        assert!(def.meta.description.is_none());
        assert_eq!(def.integration.config_path, ".cursorrules");
        assert_eq!(def.integration.config_type, ConfigType::Text);
        assert!(!def.capabilities.supports_custom_instructions);
        assert!(!def.capabilities.supports_mcp);
    }

    #[test]
    fn test_parse_tool_definition_full() {
        let toml = r#"
[meta]
name = "Cursor"
slug = "cursor"
description = "AI-first code editor"

[integration]
config_path = ".cursorrules"
type = "text"
additional_paths = [".cursor/rules/"]

[capabilities]
supports_custom_instructions = true
supports_mcp = true
supports_rules_directory = true

[schema]
instruction_key = "global_instructions"
mcp_key = "mcpServers"
"#;

        let def: ToolDefinition = toml::from_str(toml).unwrap();
        assert_eq!(def.meta.slug, "cursor");
        assert_eq!(
            def.meta.description,
            Some("AI-first code editor".to_string())
        );
        assert_eq!(def.integration.config_type, ConfigType::Text);
        assert_eq!(def.integration.additional_paths, vec![".cursor/rules/"]);
        assert!(def.capabilities.supports_custom_instructions);
        assert!(def.capabilities.supports_mcp);
        assert!(def.capabilities.supports_rules_directory);

        let schema_keys = def.schema_keys.unwrap();
        assert_eq!(
            schema_keys.instruction_key,
            Some("global_instructions".to_string())
        );
        assert_eq!(schema_keys.mcp_key, Some("mcpServers".to_string()));
    }

    #[test]
    fn test_parse_tool_definition_json_type() {
        let toml = r#"
[meta]
name = "VSCode"
slug = "vscode"

[integration]
config_path = ".vscode/settings.json"
type = "json"

[schema]
python_path_key = "python.defaultInterpreterPath"
"#;

        let def: ToolDefinition = toml::from_str(toml).unwrap();
        assert_eq!(def.integration.config_type, ConfigType::Json);
        assert_eq!(
            def.schema_keys.unwrap().python_path_key,
            Some("python.defaultInterpreterPath".to_string())
        );
    }

    #[test]
    fn test_parse_tool_definition_markdown_type() {
        let toml = r#"
[meta]
name = "Claude"
slug = "claude"

[integration]
config_path = ".claude/CLAUDE.md"
type = "markdown"
additional_paths = [".claude/rules/"]

[capabilities]
supports_custom_instructions = true
supports_rules_directory = true
"#;

        let def: ToolDefinition = toml::from_str(toml).unwrap();
        assert_eq!(def.integration.config_type, ConfigType::Markdown);
        assert!(def.capabilities.supports_rules_directory);
    }

    #[test]
    fn test_config_type_all_variants() {
        let variants = [
            ("text", ConfigType::Text),
            ("json", ConfigType::Json),
            ("toml", ConfigType::Toml),
            ("yaml", ConfigType::Yaml),
            ("markdown", ConfigType::Markdown),
        ];

        for (str_val, expected) in variants {
            let toml = format!(
                r#"
[meta]
name = "Test"
slug = "test"

[integration]
config_path = ".test"
type = "{}"
"#,
                str_val
            );

            let def: ToolDefinition = toml::from_str(&toml).unwrap();
            assert_eq!(def.integration.config_type, expected);
        }
    }
}
