//! ToolSyncer - main entry point for capability-based sync
//!
//! This module provides the high-level API for syncing rules to tool configs.

use crate::error::Result;
use crate::translator::CapabilityTranslator;
use crate::writer::{SchemaKeys, WriterRegistry};
use repo_fs::NormalizedPath;
use repo_meta::schema::{RuleDefinition, ToolDefinition};
use serde_json::Value;

/// Main entry point for syncing rules (and MCP config) to tool configs.
///
/// This syncer:
/// 1. Checks tool capabilities via CapabilityTranslator
/// 2. Translates rules and MCP config into tool-specific format
/// 3. Writes using the appropriate ConfigWriter
pub struct ToolCapabilitySyncer {
    writers: WriterRegistry,
    /// Resolved MCP server config from extensions (all servers merged).
    mcp_servers: Option<Value>,
}

impl ToolCapabilitySyncer {
    /// Create a new syncer.
    pub fn new() -> Self {
        Self {
            writers: WriterRegistry::new(),
            mcp_servers: None,
        }
    }

    /// Create a syncer with pre-resolved MCP server configuration.
    pub fn with_mcp_servers(mut self, servers: Value) -> Self {
        self.mcp_servers = Some(servers);
        self
    }

    /// Sync rules to a tool's config.
    ///
    /// # Returns
    /// - `Ok(true)` if content was written
    /// - `Ok(false)` if tool has no capabilities (nothing to sync)
    /// - `Err(_)` on write failure
    pub fn sync(
        &self,
        root: &NormalizedPath,
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> Result<bool> {
        // Check if tool has any capabilities
        if !CapabilityTranslator::has_capabilities(tool) {
            return Ok(false);
        }

        // Translate rules and MCP config for this tool
        let content = CapabilityTranslator::translate_with_mcp(
            tool,
            rules,
            self.mcp_servers.as_ref(),
        );
        if content.is_empty() {
            return Ok(false);
        }

        // Get the appropriate writer
        let writer = self.writers.get_writer(tool.integration.config_type);

        // Convert schema keys
        let keys = tool.schema_keys.as_ref().map(SchemaKeys::from);

        // Build the full path
        let path = root.join(&tool.integration.config_path);

        // Write the content
        writer.write(&path, &content, keys.as_ref())?;

        Ok(true)
    }

    /// Sync rules to multiple tools.
    ///
    /// Returns the list of tool slugs that were successfully synced.
    pub fn sync_all(
        &self,
        root: &NormalizedPath,
        tools: &[ToolDefinition],
        rules: &[RuleDefinition],
    ) -> Result<Vec<String>> {
        let mut synced = Vec::new();

        for tool in tools {
            if self.sync(root, tool, rules)? {
                synced.push(tool.meta.slug.clone());
            }
        }

        Ok(synced)
    }
}

impl Default for ToolCapabilitySyncer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        ConfigType, RuleContent, RuleMeta, Severity, ToolCapabilities, ToolIntegrationConfig,
        ToolMeta,
    };
    use std::fs;
    use tempfile::TempDir;

    fn make_tool(slug: &str, supports_instructions: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: slug.to_uppercase(),
                slug: slug.into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: format!(".{}", slug),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: supports_instructions,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    fn make_rule(id: &str) -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta {
                id: id.into(),
                severity: Severity::Mandatory,
                tags: vec![],
            },
            content: RuleContent {
                instruction: format!("{} content", id),
            },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_sync_capable_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolCapabilitySyncer::new();

        let tool = make_tool("test", true);
        let rules = vec![make_rule("r1")];

        let result = syncer.sync(&root, &tool, &rules).unwrap();
        assert!(result);
        assert!(temp.path().join(".test").exists());

        let content = fs::read_to_string(temp.path().join(".test")).unwrap();
        assert!(content.contains("r1"));
    }

    #[test]
    fn test_sync_incapable_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolCapabilitySyncer::new();

        let tool = make_tool("test", false);
        let rules = vec![make_rule("r1")];

        let result = syncer.sync(&root, &tool, &rules).unwrap();
        assert!(!result);
        assert!(!temp.path().join(".test").exists());
    }

    #[test]
    fn test_sync_no_rules() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolCapabilitySyncer::new();

        let tool = make_tool("test", true);
        let rules: Vec<RuleDefinition> = vec![];

        let result = syncer.sync(&root, &tool, &rules).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_sync_all() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolCapabilitySyncer::new();

        let tools = vec![
            make_tool("a", true),
            make_tool("b", false), // No capability
            make_tool("c", true),
        ];
        let rules = vec![make_rule("r1")];

        let synced = syncer.sync_all(&root, &tools, &rules).unwrap();

        // Only capable tools should be synced
        assert_eq!(synced.len(), 2);
        assert!(synced.contains(&"a".to_string()));
        assert!(synced.contains(&"c".to_string()));
        assert!(!synced.contains(&"b".to_string()));

        // Files should exist only for capable tools
        assert!(temp.path().join(".a").exists());
        assert!(!temp.path().join(".b").exists());
        assert!(temp.path().join(".c").exists());
    }

    fn make_mcp_tool(slug: &str) -> ToolDefinition {
        use repo_meta::schema::ToolSchemaKeys;
        ToolDefinition {
            meta: ToolMeta {
                name: slug.to_uppercase(),
                slug: slug.into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: format!(".{}/settings.json", slug),
                config_type: ConfigType::Json,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: false,
                supports_mcp: true,
                supports_rules_directory: false,
            },
            schema_keys: Some(ToolSchemaKeys {
                instruction_key: None,
                mcp_key: Some("mcpServers".into()),
                python_path_key: None,
            }),
        }
    }

    #[test]
    fn test_sync_mcp_servers_to_json_tool() {
        use serde_json::json;

        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());

        let servers = json!({"my-server": {"command": "python", "args": ["serve"]}});
        let syncer = ToolCapabilitySyncer::new().with_mcp_servers(servers);

        let tool = make_mcp_tool("mcp-tool");
        let result = syncer.sync(&root, &tool, &[]).unwrap();
        assert!(result);

        let config_path = temp.path().join(".mcp-tool/settings.json");
        assert!(config_path.exists());

        let written = fs::read_to_string(&config_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&written).unwrap();
        assert_eq!(json["mcpServers"]["my-server"]["command"], "python");
    }

    #[test]
    fn test_sync_no_mcp_servers_when_none_configured() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolCapabilitySyncer::new(); // No MCP servers

        let tool = make_mcp_tool("mcp-tool");
        let result = syncer.sync(&root, &tool, &[]).unwrap();
        // MCP-only tool with no servers => nothing to write
        assert!(!result);
    }
}
