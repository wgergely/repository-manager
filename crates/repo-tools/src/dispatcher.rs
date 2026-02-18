//! Tool dispatcher that routes to appropriate integration
//!
//! The dispatcher uses ToolRegistry as the single source of truth for tool
//! definitions, eliminating the previous 3-location duplication.

use crate::aider::aider_integration;
use crate::amazonq::amazonq_integration;
use crate::antigravity::antigravity_integration;
use crate::claude::claude_integration;
use crate::cline::cline_integration;
use crate::copilot::copilot_integration;
use crate::cursor::cursor_integration;
use crate::error::Result;
use crate::gemini::gemini_integration;
use crate::generic::GenericToolIntegration;
use crate::integration::{Rule, SyncContext, ToolIntegration};
use crate::jetbrains::jetbrains_integration;
use crate::registry::{BUILTIN_COUNT, ToolRegistration, ToolRegistry};
use crate::roo::roo_integration;
use crate::vscode::VSCodeIntegration;
use crate::windsurf::windsurf_integration;
use crate::zed::zed_integration;
use repo_meta::schema::ToolDefinition;
use std::collections::HashMap;

/// Dispatches sync operations to appropriate tool integrations.
///
/// Uses ToolRegistry as the single source of truth for tool definitions.
/// The registry is populated with built-in tools and can have additional
/// schema-defined tools registered at runtime.
pub struct ToolDispatcher {
    registry: ToolRegistry,
    /// Additional schema-defined tools (loaded from .repository/tools/)
    schema_tools: HashMap<String, ToolDefinition>,
}

impl ToolDispatcher {
    /// Create a new dispatcher with all built-in tools.
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::with_builtins(),
            schema_tools: HashMap::new(),
        }
    }

    /// Create a dispatcher with pre-loaded tool definitions.
    pub fn with_definitions(definitions: HashMap<String, ToolDefinition>) -> Self {
        let mut dispatcher = Self::new();
        for (_, def) in definitions {
            dispatcher.register(def);
        }
        dispatcher
    }

    /// Register a schema-defined tool.
    pub fn register(&mut self, definition: ToolDefinition) {
        self.schema_tools
            .insert(definition.meta.slug.clone(), definition);
    }

    /// Get an integration for a tool by name.
    ///
    /// For built-in tools, returns optimized implementations.
    /// For schema-defined tools, returns GenericToolIntegration.
    pub fn get_integration(&self, tool_name: &str) -> Option<Box<dyn ToolIntegration>> {
        // Check built-in tools in registry
        if self.registry.contains(tool_name) {
            return Self::create_builtin_integration(tool_name);
        }

        // Fall back to schema-defined generic integration
        self.schema_tools.get(tool_name).map(|def| {
            Box::new(GenericToolIntegration::new(def.clone())) as Box<dyn ToolIntegration>
        })
    }

    /// Create a built-in integration by name.
    ///
    /// Returns `None` if the tool name is not recognized.
    fn create_builtin_integration(name: &str) -> Option<Box<dyn ToolIntegration>> {
        let integration: Box<dyn ToolIntegration> = match name {
            "vscode" => Box::new(VSCodeIntegration::new()),
            "cursor" => Box::new(cursor_integration()),
            "claude" => Box::new(claude_integration()),
            "windsurf" => Box::new(windsurf_integration()),
            "antigravity" => Box::new(antigravity_integration()),
            "gemini" => Box::new(gemini_integration()),
            "copilot" => Box::new(copilot_integration()),
            "cline" => Box::new(cline_integration()),
            "roo" => Box::new(roo_integration()),
            "jetbrains" => Box::new(jetbrains_integration()),
            "zed" => Box::new(zed_integration()),
            "aider" => Box::new(aider_integration()),
            "amazonq" => Box::new(amazonq_integration()),
            _ => {
                // Try to find in builtin registrations as fallback
                match crate::registry::builtin_registrations()
                    .into_iter()
                    .find(|r| r.slug == name)
                    .map(|r| r.definition)
                {
                    Some(def) => Box::new(GenericToolIntegration::new(def)),
                    None => return None,
                }
            }
        };
        Some(integration)
    }

    /// Check if a tool is available (built-in or schema-defined).
    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.registry.contains(tool_name) || self.schema_tools.contains_key(tool_name)
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
        let mut tools: Vec<String> = self.registry.list().iter().map(|s| s.to_string()).collect();

        for slug in self.schema_tools.keys() {
            if !tools.contains(slug) {
                tools.push(slug.clone());
            }
        }

        tools.sort();
        tools
    }

    /// Get the number of schema-defined tools (not including built-ins).
    pub fn schema_tool_count(&self) -> usize {
        self.schema_tools.len()
    }

    /// Get the total number of registered tools.
    pub fn total_tool_count(&self) -> usize {
        BUILTIN_COUNT + self.schema_tools.len()
    }

    /// Get a registration by slug (for capability checking).
    pub fn get_registration(&self, slug: &str) -> Option<&ToolRegistration> {
        self.registry.get(slug)
    }

    /// Get access to the underlying registry.
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
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

    fn create_custom_tool_definition() -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "CustomTool".to_string(),
                slug: "customtool".to_string(),
                description: Some("A custom tool for testing".to_string()),
            },
            integration: ToolIntegrationConfig {
                config_path: ".customtool/rules.md".to_string(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: true,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    #[test]
    fn test_new_dispatcher() {
        let dispatcher = ToolDispatcher::new();
        assert_eq!(dispatcher.schema_tool_count(), 0);
        assert_eq!(dispatcher.total_tool_count(), BUILTIN_COUNT);
    }

    #[test]
    fn test_builtin_tools() {
        let dispatcher = ToolDispatcher::new();

        assert!(dispatcher.get_integration("vscode").is_some());
        assert!(dispatcher.get_integration("cursor").is_some());
        assert!(dispatcher.get_integration("claude").is_some());
        assert!(dispatcher.get_integration("windsurf").is_some());
        assert!(dispatcher.get_integration("antigravity").is_some());
        assert!(dispatcher.get_integration("gemini").is_some());
        assert!(dispatcher.get_integration("copilot").is_some());
        assert!(dispatcher.get_integration("cline").is_some());
        assert!(dispatcher.get_integration("roo").is_some());
        assert!(dispatcher.get_integration("jetbrains").is_some());
        assert!(dispatcher.get_integration("zed").is_some());
        assert!(dispatcher.get_integration("aider").is_some());
        assert!(dispatcher.get_integration("amazonq").is_some());
    }

    #[test]
    fn test_has_tool() {
        let dispatcher = ToolDispatcher::new();

        // Built-in tools
        assert!(dispatcher.has_tool("vscode"));
        assert!(dispatcher.has_tool("cursor"));
        assert!(dispatcher.has_tool("claude"));
        assert!(dispatcher.has_tool("windsurf"));
        assert!(dispatcher.has_tool("antigravity"));
        assert!(dispatcher.has_tool("gemini"));
        assert!(dispatcher.has_tool("copilot"));
        assert!(dispatcher.has_tool("cline"));
        assert!(dispatcher.has_tool("roo"));
        assert!(dispatcher.has_tool("jetbrains"));
        assert!(dispatcher.has_tool("zed"));
        assert!(dispatcher.has_tool("aider"));
        assert!(dispatcher.has_tool("amazonq"));

        // Unknown tool
        assert!(!dispatcher.has_tool("unknown_tool"));
    }

    #[test]
    fn test_register_schema_tool() {
        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(create_custom_tool_definition());

        let integration = dispatcher.get_integration("customtool");
        assert!(integration.is_some());
        assert_eq!(integration.unwrap().name(), "customtool");
    }

    #[test]
    fn test_with_definitions() {
        let mut definitions = HashMap::new();
        definitions.insert("customtool".to_string(), create_custom_tool_definition());

        let dispatcher = ToolDispatcher::with_definitions(definitions);

        assert!(dispatcher.get_integration("customtool").is_some());
        assert_eq!(dispatcher.schema_tool_count(), 1);
    }

    #[test]
    fn test_list_available() {
        let mut dispatcher = ToolDispatcher::new();
        dispatcher.register(create_custom_tool_definition());

        let available = dispatcher.list_available();

        // Should be sorted and include all built-ins plus schema tool
        assert!(available.contains(&"vscode".to_string()));
        assert!(available.contains(&"cursor".to_string()));
        assert!(available.contains(&"claude".to_string()));
        assert!(available.contains(&"windsurf".to_string()));
        assert!(available.contains(&"antigravity".to_string()));
        assert!(available.contains(&"gemini".to_string()));
        assert!(available.contains(&"zed".to_string()));
        assert!(available.contains(&"customtool".to_string()));

        // First item should be "aider" (alphabetically first)
        assert_eq!(available[0], "aider");
    }

    #[test]
    fn test_unknown_tool_returns_none() {
        let dispatcher = ToolDispatcher::new();
        assert!(dispatcher.get_integration("unknown").is_none());
    }

    #[test]
    fn test_registry_access() {
        let dispatcher = ToolDispatcher::new();
        let registry = dispatcher.registry();

        assert_eq!(registry.len(), BUILTIN_COUNT);
        assert!(registry.contains("cursor"));
    }

    #[test]
    fn test_get_registration() {
        let dispatcher = ToolDispatcher::new();

        let reg = dispatcher.get_registration("cursor");
        assert!(reg.is_some());
        assert_eq!(reg.unwrap().name, "Cursor");
    }
}
