//! Tool dispatcher that routes to appropriate integration
//!
//! The dispatcher manages tool integrations and routes sync operations to the
//! appropriate implementation. It prefers optimized built-in integrations for
//! known tools (vscode, cursor, claude) and falls back to schema-driven generic
//! integrations for other tools.

use crate::claude::claude_integration;
use crate::cursor::cursor_integration;
use crate::error::Result;
use crate::generic::GenericToolIntegration;
use crate::integration::{Rule, SyncContext, ToolIntegration};
use crate::vscode::VSCodeIntegration;
use repo_meta::schema::ToolDefinition;
use std::collections::HashMap;

/// Dispatches sync operations to appropriate tool integrations.
///
/// The dispatcher maintains a registry of schema-defined tools and routes
/// requests to either built-in integrations (for performance) or generic
/// schema-driven integrations.
pub struct ToolDispatcher {
    /// Schema-defined tools (loaded from .repository/tools/)
    schema_tools: HashMap<String, ToolDefinition>,
}

impl ToolDispatcher {
    /// Create a new empty dispatcher.
    pub fn new() -> Self {
        Self {
            schema_tools: HashMap::new(),
        }
    }

    /// Create a dispatcher with pre-loaded tool definitions.
    pub fn with_definitions(definitions: HashMap<String, ToolDefinition>) -> Self {
        Self {
            schema_tools: definitions,
        }
    }

    /// Register a tool definition.
    pub fn register(&mut self, definition: ToolDefinition) {
        self.schema_tools
            .insert(definition.meta.slug.clone(), definition);
    }

    /// Get an integration for a tool by name.
    ///
    /// Prefers built-in integrations, falls back to generic schema-driven.
    pub fn get_integration(&self, tool_name: &str) -> Option<Box<dyn ToolIntegration>> {
        // Check for built-in integrations first
        match tool_name {
            "vscode" => return Some(Box::new(VSCodeIntegration::new())),
            "cursor" => return Some(Box::new(cursor_integration())),
            "claude" => return Some(Box::new(claude_integration())),
            _ => {}
        }

        // Fall back to schema-defined generic integration
        self.schema_tools.get(tool_name).map(|def| {
            Box::new(GenericToolIntegration::new(def.clone())) as Box<dyn ToolIntegration>
        })
    }

    /// Check if a tool is available (built-in or schema-defined).
    pub fn has_tool(&self, tool_name: &str) -> bool {
        matches!(tool_name, "vscode" | "cursor" | "claude")
            || self.schema_tools.contains_key(tool_name)
    }

    /// Sync rules to all specified tools.
    ///
    /// Returns the list of tool names that were successfully synced.
    pub fn sync_all(
        &self,
        context: &SyncContext,
        tool_names: &[String],
        rules: &[Rule],
    ) -> Result<Vec<String>> {
        let mut synced = Vec::new();

        for name in tool_names {
            if let Some(integration) = self.get_integration(name) {
                integration.sync(context, rules)?;
                synced.push(name.clone());
            }
        }

        Ok(synced)
    }

    /// List all available tools (built-in + schema-defined).
    pub fn list_available(&self) -> Vec<String> {
        let mut tools = vec![
            "vscode".to_string(),
            "cursor".to_string(),
            "claude".to_string(),
        ];

        for slug in self.schema_tools.keys() {
            if !tools.contains(slug) {
                tools.push(slug.clone());
            }
        }

        tools.sort();
        tools
    }

    /// Get the number of schema-defined tools.
    pub fn schema_tool_count(&self) -> usize {
        self.schema_tools.len()
    }
}

impl Default for ToolDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{ConfigType, ToolCapabilities, ToolIntegrationConfig, ToolMeta};

    fn create_windsurf_definition() -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Windsurf".to_string(),
                slug: "windsurf".to_string(),
                description: Some("Codeium's AI IDE".to_string()),
            },
            integration: ToolIntegrationConfig {
                config_path: ".windsurfrules".to_string(),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: true,
                supports_mcp: true,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    #[test]
    fn test_new_dispatcher() {
        let dispatcher = ToolDispatcher::new();
        assert_eq!(dispatcher.schema_tool_count(), 0);
    }

    #[test]
    fn test_builtin_tools() {
        let dispatcher = ToolDispatcher::new();

        assert!(dispatcher.get_integration("vscode").is_some());
        assert!(dispatcher.get_integration("cursor").is_some());
        assert!(dispatcher.get_integration("claude").is_some());
    }

    #[test]
    fn test_has_tool() {
        let mut dispatcher = ToolDispatcher::new();

        // Built-in tools
        assert!(dispatcher.has_tool("vscode"));
        assert!(dispatcher.has_tool("cursor"));
        assert!(dispatcher.has_tool("claude"));

        // Unknown tool
        assert!(!dispatcher.has_tool("windsurf"));

        // Register and check again
        dispatcher.register(create_windsurf_definition());
        assert!(dispatcher.has_tool("windsurf"));
    }

    #[test]
    fn test_register_schema_tool() {
        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(create_windsurf_definition());

        let integration = dispatcher.get_integration("windsurf");
        assert!(integration.is_some());
        assert_eq!(integration.unwrap().name(), "windsurf");
    }

    #[test]
    fn test_with_definitions() {
        let mut definitions = HashMap::new();
        definitions.insert("windsurf".to_string(), create_windsurf_definition());

        let dispatcher = ToolDispatcher::with_definitions(definitions);

        assert!(dispatcher.get_integration("windsurf").is_some());
        assert_eq!(dispatcher.schema_tool_count(), 1);
    }

    #[test]
    fn test_list_available() {
        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(create_windsurf_definition());

        let available = dispatcher.list_available();

        // Should be sorted
        assert!(available.contains(&"vscode".to_string()));
        assert!(available.contains(&"cursor".to_string()));
        assert!(available.contains(&"claude".to_string()));
        assert!(available.contains(&"windsurf".to_string()));

        // First item should be "claude" (alphabetically first)
        assert_eq!(available[0], "claude");
    }

    #[test]
    fn test_unknown_tool_returns_none() {
        let dispatcher = ToolDispatcher::new();
        assert!(dispatcher.get_integration("unknown").is_none());
    }
}
