//! Main capability translator orchestrator
//!
//! This is the entry point for capability-based translation. It coordinates
//! all sub-translators and respects tool capabilities.

use super::{RuleTranslator, TranslatedContent};
use repo_meta::schema::{RuleDefinition, ToolDefinition};
use serde_json::Value;

/// Main translator that orchestrates capability-based content generation.
///
/// This translator:
/// 1. Checks what capabilities a tool has
/// 2. Delegates to appropriate sub-translators
/// 3. Combines results into a single TranslatedContent
pub struct CapabilityTranslator;

impl CapabilityTranslator {
    /// Translate rules and other content for a specific tool.
    ///
    /// Respects the tool's declared capabilities, only generating
    /// content the tool can actually use.
    pub fn translate(tool: &ToolDefinition, rules: &[RuleDefinition]) -> TranslatedContent {
        Self::translate_with_mcp(tool, rules, None)
    }

    /// Translate rules and MCP server config for a specific tool.
    ///
    /// This is the full-featured translation entry point. It respects
    /// tool capabilities and merges MCP server configuration when the
    /// tool supports it.
    pub fn translate_with_mcp(
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
        mcp_servers: Option<&Value>,
    ) -> TranslatedContent {
        let mut content = TranslatedContent::empty();
        content.format = tool.integration.config_type;

        // Custom instructions (if supported)
        if tool.capabilities.supports_custom_instructions {
            let rule_content = RuleTranslator::translate(tool, rules);
            content.instructions = rule_content.instructions;
        }

        // MCP servers (if tool supports MCP and config is provided)
        if tool.capabilities.supports_mcp
            && let Some(servers) = mcp_servers {
                content.mcp_servers = Some(servers.clone());
            }

        content
    }

    /// Check if a tool has any capabilities that require syncing.
    pub fn has_capabilities(tool: &ToolDefinition) -> bool {
        tool.capabilities.supports_custom_instructions
            || tool.capabilities.supports_mcp
            || tool.capabilities.supports_rules_directory
    }

    /// Check if a tool supports custom instructions.
    pub fn supports_instructions(tool: &ToolDefinition) -> bool {
        tool.capabilities.supports_custom_instructions
    }

    /// Check if a tool supports MCP servers.
    pub fn supports_mcp(tool: &ToolDefinition) -> bool {
        tool.capabilities.supports_mcp
    }

    /// Check if a tool supports rules directory.
    pub fn supports_rules_directory(tool: &ToolDefinition) -> bool {
        tool.capabilities.supports_rules_directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        ConfigType, RuleContent, RuleMeta, Severity, ToolCapabilities, ToolIntegrationConfig,
        ToolMeta,
    };

    fn make_tool(instructions: bool, mcp: bool, rules_dir: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Test".into(),
                slug: "test".into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".test".into(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: instructions,
                supports_mcp: mcp,
                supports_rules_directory: rules_dir,
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
                instruction: format!("Rule {} content", id),
            },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_translate_with_instructions_capability() {
        let tool = make_tool(true, false, false);
        let rules = vec![make_rule("r1")];

        let content = CapabilityTranslator::translate(&tool, &rules);
        assert!(!content.is_empty());
        assert!(content.instructions.is_some());
    }

    #[test]
    fn test_translate_without_capabilities() {
        let tool = make_tool(false, false, false);
        let rules = vec![make_rule("r1")];

        let content = CapabilityTranslator::translate(&tool, &rules);
        assert!(content.is_empty());
    }

    #[test]
    fn test_has_capabilities_all_false() {
        let tool = make_tool(false, false, false);
        assert!(!CapabilityTranslator::has_capabilities(&tool));
    }

    #[test]
    fn test_has_capabilities_instructions() {
        let tool = make_tool(true, false, false);
        assert!(CapabilityTranslator::has_capabilities(&tool));
    }

    #[test]
    fn test_has_capabilities_mcp() {
        let tool = make_tool(false, true, false);
        assert!(CapabilityTranslator::has_capabilities(&tool));
    }

    #[test]
    fn test_has_capabilities_rules_dir() {
        let tool = make_tool(false, false, true);
        assert!(CapabilityTranslator::has_capabilities(&tool));
    }

    #[test]
    fn test_capability_checks() {
        let tool = make_tool(true, true, false);

        assert!(CapabilityTranslator::supports_instructions(&tool));
        assert!(CapabilityTranslator::supports_mcp(&tool));
        assert!(!CapabilityTranslator::supports_rules_directory(&tool));
    }

    #[test]
    fn test_format_preserved() {
        let mut tool = make_tool(true, false, false);
        tool.integration.config_type = ConfigType::Json;

        let rules = vec![make_rule("r1")];
        let content = CapabilityTranslator::translate(&tool, &rules);

        assert_eq!(content.format, ConfigType::Json);
    }

    #[test]
    fn test_translate_with_mcp_when_supported() {
        use serde_json::json;

        let tool = make_tool(false, true, false);
        let servers = json!({"my-server": {"command": "python", "args": ["serve"]}});

        let content = CapabilityTranslator::translate_with_mcp(&tool, &[], Some(&servers));
        assert!(content.mcp_servers.is_some());
        assert_eq!(content.mcp_servers.unwrap()["my-server"]["command"], "python");
    }

    #[test]
    fn test_translate_with_mcp_when_not_supported() {
        use serde_json::json;

        let tool = make_tool(true, false, false);
        let servers = json!({"my-server": {"command": "python"}});

        let content = CapabilityTranslator::translate_with_mcp(&tool, &[], Some(&servers));
        assert!(content.mcp_servers.is_none());
    }

    #[test]
    fn test_translate_with_mcp_none_servers() {
        let tool = make_tool(false, true, false);
        let content = CapabilityTranslator::translate_with_mcp(&tool, &[], None);
        assert!(content.mcp_servers.is_none());
    }

    #[test]
    fn test_translate_with_mcp_and_instructions() {
        use serde_json::json;

        let tool = make_tool(true, true, false);
        let rules = vec![make_rule("r1")];
        let servers = json!({"srv": {"command": "test"}});

        let content = CapabilityTranslator::translate_with_mcp(&tool, &rules, Some(&servers));
        assert!(content.instructions.is_some());
        assert!(content.mcp_servers.is_some());
    }
}
