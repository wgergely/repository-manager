//! TOML format handler for managed blocks
//!
//! Uses a reserved `[repo_managed]` table with UUID sub-tables.
//!
//! Example:
//! ```toml
//! [project]
//! name = "my-project"
//!
//! [repo_managed."550e8400-e29b-41d4-a716-446655440000"]
//! setting = "value"
//! enabled = true
//!
//! [dependencies]
//! serde = "1.0"
//! ```

use super::{FormatHandler, ManagedBlock};
use uuid::Uuid;

/// The reserved table name for managed blocks in TOML files
pub const MANAGED_TABLE: &str = "repo_managed";

/// TOML format handler
#[derive(Debug, Default, Clone)]
pub struct TomlFormatHandler;

impl TomlFormatHandler {
    /// Create a new TOML format handler
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for TomlFormatHandler {
    fn parse_blocks(&self, content: &str) -> Vec<ManagedBlock> {
        let Ok(table) = content.parse::<toml::Table>() else {
            return Vec::new();
        };

        let Some(managed) = table.get(MANAGED_TABLE) else {
            return Vec::new();
        };

        let Some(managed_table) = managed.as_table() else {
            return Vec::new();
        };

        managed_table
            .iter()
            .filter_map(|(key, value)| {
                let uuid = Uuid::parse_str(key).ok()?;
                let content = toml::to_string_pretty(value).ok()?;
                Some(ManagedBlock {
                    uuid,
                    content: content.trim().to_string(),
                })
            })
            .collect()
    }

    fn write_block(&self, content: &str, uuid: Uuid, block_content: &str) -> String {
        // Parse existing TOML or create empty table
        let mut table: toml::Table = if content.trim().is_empty() {
            toml::Table::new()
        } else {
            content.parse().unwrap_or_default()
        };

        // Parse the block content as TOML value
        let block_value: toml::Value = block_content
            .parse::<toml::Table>()
            .map(toml::Value::Table)
            .unwrap_or_else(|_| toml::Value::String(block_content.to_string()));

        // Get or create the managed table
        let managed = table
            .entry(MANAGED_TABLE)
            .or_insert_with(|| toml::Value::Table(toml::Table::new()));

        // Add or update the block
        if let Some(managed_table) = managed.as_table_mut() {
            managed_table.insert(uuid.to_string(), block_value);
        }

        toml::to_string_pretty(&table).unwrap_or_else(|_| content.to_string())
    }

    fn remove_block(&self, content: &str, uuid: Uuid) -> String {
        let Ok(mut table) = content.parse::<toml::Table>() else {
            return content.to_string();
        };

        // Get the managed table
        if let Some(managed) = table.get_mut(MANAGED_TABLE) {
            if let Some(managed_table) = managed.as_table_mut() {
                managed_table.remove(&uuid.to_string());

                // If managed table is now empty, remove it entirely
                if managed_table.is_empty() {
                    table.remove(MANAGED_TABLE);
                }
            }
        }

        toml::to_string_pretty(&table).unwrap_or_else(|_| content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        let handler = TomlFormatHandler::new();
        let blocks = handler.parse_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_no_managed_table() {
        let handler = TomlFormatHandler::new();
        let content = r#"
[project]
name = "test"
"#;
        let blocks = handler.parse_blocks(content);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_with_managed_blocks() {
        let handler = TomlFormatHandler::new();
        let content = r#"
[project]
name = "test"

[repo_managed."550e8400-e29b-41d4-a716-446655440000"]
setting = "value"
enabled = true
"#;

        let blocks = handler.parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].uuid,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert!(blocks[0].content.contains("setting"));
        assert!(blocks[0].content.contains("enabled"));
    }

    #[test]
    fn test_write_block_to_empty() {
        let handler = TomlFormatHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let result = handler.write_block("", uuid, "setting = \"value\"");
        let parsed: toml::Table = result.parse().unwrap();

        assert!(parsed
            .get(MANAGED_TABLE)
            .and_then(|m| m.get(&uuid.to_string()))
            .is_some());
    }

    #[test]
    fn test_write_block_preserves_user_tables() {
        let handler = TomlFormatHandler::new();
        let existing = r#"
[project]
name = "test"
version = "1.0"
"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.write_block(existing, uuid, "setting = \"value\"");

        let parsed: toml::Table = result.parse().unwrap();

        // User table preserved
        assert!(parsed.get("project").is_some());
        assert_eq!(
            parsed["project"]["name"].as_str().unwrap(),
            "test"
        );

        // Managed block added
        assert!(parsed
            .get(MANAGED_TABLE)
            .and_then(|m| m.get(&uuid.to_string()))
            .is_some());
    }

    #[test]
    fn test_write_block_updates_existing() {
        let handler = TomlFormatHandler::new();
        let existing = r#"
[repo_managed."550e8400-e29b-41d4-a716-446655440000"]
old = "value"
"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.write_block(existing, uuid, "new = \"value\"");

        let parsed: toml::Table = result.parse().unwrap();

        let managed = parsed[MANAGED_TABLE][&uuid.to_string()].as_table().unwrap();
        assert!(managed.get("new").is_some());
        // Old key is replaced (whole block is replaced)
        assert!(managed.get("old").is_none());
    }

    #[test]
    fn test_remove_block() {
        let handler = TomlFormatHandler::new();
        let existing = r#"
[project]
name = "test"

[repo_managed."550e8400-e29b-41d4-a716-446655440000"]
setting = "value"
"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.remove_block(existing, uuid);

        let parsed: toml::Table = result.parse().unwrap();

        // User table preserved
        assert!(parsed.get("project").is_some());
        // Managed table removed (was only block)
        assert!(parsed.get(MANAGED_TABLE).is_none());
    }

    #[test]
    fn test_remove_block_keeps_other_blocks() {
        let handler = TomlFormatHandler::new();
        let existing = r#"
[repo_managed."550e8400-e29b-41d4-a716-446655440000"]
a = 1

[repo_managed."6ba7b810-9dad-11d1-80b4-00c04fd430c8"]
b = 2
"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.remove_block(existing, uuid);

        let parsed: toml::Table = result.parse().unwrap();

        // Other block still exists
        let managed = parsed.get(MANAGED_TABLE).unwrap().as_table().unwrap();
        assert!(managed.get("6ba7b810-9dad-11d1-80b4-00c04fd430c8").is_some());
        // Removed block is gone
        assert!(managed.get("550e8400-e29b-41d4-a716-446655440000").is_none());
    }

    #[test]
    fn test_has_block() {
        let handler = TomlFormatHandler::new();
        let content = r#"
[repo_managed."550e8400-e29b-41d4-a716-446655440000"]
a = 1
"#;

        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

        assert!(handler.has_block(content, uuid1));
        assert!(!handler.has_block(content, uuid2));
    }

    #[test]
    fn test_multiple_blocks() {
        let handler = TomlFormatHandler::new();

        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

        let result = handler.write_block("", uuid1, "first = 1");
        let result = handler.write_block(&result, uuid2, "second = 2");

        let blocks = handler.parse_blocks(&result);
        assert_eq!(blocks.len(), 2);
    }
}
