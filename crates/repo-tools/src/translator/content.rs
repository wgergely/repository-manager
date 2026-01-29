//! Translated content ready for config writing

use repo_meta::schema::ConfigType;
use serde_json::Value;
use std::collections::HashMap;

/// Content translated from rules, ready to be written to tool configs.
///
/// This is the output of the capability translator, containing all the
/// data needed to write to a tool's configuration file.
#[derive(Debug, Clone, Default)]
pub struct TranslatedContent {
    /// The config format this content is for
    pub format: ConfigType,
    /// Custom instructions text (if tool supports them)
    pub instructions: Option<String>,
    /// MCP server configuration (if tool supports MCP)
    pub mcp_servers: Option<Value>,
    /// Additional data to merge into config
    pub data: HashMap<String, Value>,
}

impl TranslatedContent {
    /// Create an empty content container.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create content with instructions.
    pub fn with_instructions(format: ConfigType, instructions: String) -> Self {
        Self {
            format,
            instructions: Some(instructions),
            ..Default::default()
        }
    }

    /// Check if this content has anything to write.
    pub fn is_empty(&self) -> bool {
        self.instructions.is_none() && self.mcp_servers.is_none() && self.data.is_empty()
    }

    /// Add arbitrary data to the content.
    pub fn with_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Set the MCP servers configuration.
    pub fn with_mcp_servers(mut self, servers: Value) -> Self {
        self.mcp_servers = Some(servers);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let content = TranslatedContent::empty();
        assert!(content.is_empty());
        assert!(content.instructions.is_none());
        assert!(content.mcp_servers.is_none());
        assert!(content.data.is_empty());
    }

    #[test]
    fn test_with_instructions() {
        let content = TranslatedContent::with_instructions(
            ConfigType::Markdown,
            "Test instructions".to_string(),
        );
        assert!(!content.is_empty());
        assert_eq!(content.instructions.as_deref(), Some("Test instructions"));
        assert_eq!(content.format, ConfigType::Markdown);
    }

    #[test]
    fn test_with_data() {
        let content = TranslatedContent::empty()
            .with_data("key1", Value::String("value1".into()))
            .with_data("key2", Value::Bool(true));

        assert!(!content.is_empty());
        assert_eq!(content.data.len(), 2);
        assert_eq!(content.data["key1"], "value1");
        assert_eq!(content.data["key2"], true);
    }

    #[test]
    fn test_with_mcp_servers() {
        use serde_json::json;

        let content = TranslatedContent::empty()
            .with_mcp_servers(json!({"server1": {"command": "test"}}));

        assert!(!content.is_empty());
        assert!(content.mcp_servers.is_some());
    }
}
