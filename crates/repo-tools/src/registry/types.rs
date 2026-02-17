//! Core types for the unified tool registry

use repo_meta::schema::ToolDefinition;
use serde::{Deserialize, Serialize};

/// Tool category for filtering and organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    /// IDE-based tools (VSCode, Cursor, Zed, JetBrains, Windsurf, Antigravity)
    Ide,
    /// CLI-based agents (Claude, Aider, Gemini)
    CliAgent,
    /// Autonomous coding agents (Cline, Roo)
    Autonomous,
    /// Copilot-style assistants (GitHub Copilot, Amazon Q)
    Copilot,
}

/// Complete tool registration containing all metadata and definition.
#[derive(Debug, Clone)]
pub struct ToolRegistration {
    /// Machine identifier (e.g., "vscode", "cursor")
    pub slug: String,
    /// Display name (e.g., "VS Code", "Cursor")
    pub name: String,
    /// Tool category for filtering
    pub category: ToolCategory,
    /// Priority for ordering (lower = higher priority)
    pub priority: u8,
    /// Full tool definition with capabilities and integration config
    pub definition: ToolDefinition,
}

impl ToolRegistration {
    /// Create a new tool registration with default priority.
    pub fn new(
        slug: impl Into<String>,
        name: impl Into<String>,
        category: ToolCategory,
        definition: ToolDefinition,
    ) -> Self {
        Self {
            slug: slug.into(),
            name: name.into(),
            category,
            priority: 50, // Default middle priority
            definition,
        }
    }

    /// Set the priority (builder pattern).
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Check if the tool supports custom instructions.
    pub fn supports_instructions(&self) -> bool {
        self.definition.capabilities.supports_custom_instructions
    }

    /// Check if the tool supports MCP servers.
    pub fn supports_mcp(&self) -> bool {
        self.definition.capabilities.supports_mcp
    }

    /// Check if the tool supports rules directory.
    pub fn supports_rules_directory(&self) -> bool {
        self.definition.capabilities.supports_rules_directory
    }

    /// Check if the tool has any capability that requires syncing.
    pub fn has_any_capability(&self) -> bool {
        self.supports_instructions() || self.supports_mcp() || self.supports_rules_directory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{ConfigType, ToolCapabilities, ToolIntegrationConfig, ToolMeta};

    fn make_def() -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Test".into(),
                slug: "test".into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".test".into(),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        }
    }

    #[test]
    fn test_registration_new() {
        let reg = ToolRegistration::new("test", "Test Tool", ToolCategory::Ide, make_def());
        assert_eq!(reg.slug, "test");
        assert_eq!(reg.name, "Test Tool");
        assert_eq!(reg.category, ToolCategory::Ide);
        assert_eq!(reg.priority, 50);
    }

    #[test]
    fn test_with_priority() {
        let reg =
            ToolRegistration::new("test", "Test", ToolCategory::Ide, make_def()).with_priority(10);
        assert_eq!(reg.priority, 10);
    }

    #[test]
    fn test_capability_checks() {
        let mut def = make_def();
        def.capabilities.supports_custom_instructions = true;
        def.capabilities.supports_mcp = false;
        def.capabilities.supports_rules_directory = false;

        let reg = ToolRegistration::new("test", "Test", ToolCategory::Ide, def);
        assert!(reg.supports_instructions());
        assert!(!reg.supports_mcp());
        assert!(!reg.supports_rules_directory());
        assert!(reg.has_any_capability());
    }

    #[test]
    fn test_no_capabilities() {
        let reg = ToolRegistration::new("test", "Test", ToolCategory::Ide, make_def());
        assert!(!reg.has_any_capability());
    }
}
