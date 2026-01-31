//! ConfigWriter trait and supporting types

use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::NormalizedPath;

/// Trait for config file writers.
///
/// Each writer handles a specific config format (JSON, Markdown, Text)
/// and knows how to merge content into existing files appropriately.
pub trait ConfigWriter: Send + Sync {
    /// Write translated content to a config file.
    ///
    /// # Arguments
    /// * `path` - The path to write to
    /// * `content` - The translated content to write
    /// * `schema_keys` - Optional keys for JSON schema placement
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        schema_keys: Option<&SchemaKeys>,
    ) -> Result<()>;

    /// Check if this writer can handle the given path.
    fn can_handle(&self, path: &NormalizedPath) -> bool;
}

/// Schema keys for JSON config file key placement.
///
/// These define where specific content types should be placed
/// in a JSON config file.
#[derive(Debug, Clone, Default)]
pub struct SchemaKeys {
    /// Key for custom instructions (e.g., "customInstructions")
    pub instruction_key: Option<String>,
    /// Key for MCP server configuration (e.g., "mcpServers")
    pub mcp_key: Option<String>,
    /// Key for Python path (e.g., "python.defaultInterpreterPath")
    pub python_path_key: Option<String>,
}

impl From<&repo_meta::schema::ToolSchemaKeys> for SchemaKeys {
    fn from(k: &repo_meta::schema::ToolSchemaKeys) -> Self {
        Self {
            instruction_key: k.instruction_key.clone(),
            mcp_key: k.mcp_key.clone(),
            python_path_key: k.python_path_key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::ToolSchemaKeys;

    #[test]
    fn test_schema_keys_default() {
        let keys = SchemaKeys::default();
        assert!(keys.instruction_key.is_none());
        assert!(keys.mcp_key.is_none());
        assert!(keys.python_path_key.is_none());
    }

    #[test]
    fn test_schema_keys_from_tool_schema() {
        let tool_keys = ToolSchemaKeys {
            instruction_key: Some("customInstructions".into()),
            mcp_key: Some("mcpServers".into()),
            python_path_key: Some("pythonPath".into()),
        };

        let keys = SchemaKeys::from(&tool_keys);
        assert_eq!(keys.instruction_key.as_deref(), Some("customInstructions"));
        assert_eq!(keys.mcp_key.as_deref(), Some("mcpServers"));
        assert_eq!(keys.python_path_key.as_deref(), Some("pythonPath"));
    }
}
