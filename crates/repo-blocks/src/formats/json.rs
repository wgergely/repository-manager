//! JSON format handler for managed blocks
//!
//! Uses a reserved `__repo_managed__` key to store managed blocks.
//! Each block is keyed by its UUID.
//!
//! Example:
//! ```json
//! {
//!     "user.setting": true,
//!     "__repo_managed__": {
//!         "550e8400-e29b-41d4-a716-446655440000": {
//!             "managed.setting": "value"
//!         }
//!     }
//! }
//! ```

use super::{FormatHandler, ManagedBlock};
use serde_json::{Map, Value};
use uuid::Uuid;

/// The reserved key for managed blocks in JSON files
pub const MANAGED_KEY: &str = "__repo_managed__";

/// JSON format handler
#[derive(Debug, Default, Clone)]
pub struct JsonFormatHandler;

impl JsonFormatHandler {
    /// Create a new JSON format handler
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for JsonFormatHandler {
    fn parse_blocks(&self, content: &str) -> Vec<ManagedBlock> {
        let Ok(json) = serde_json::from_str::<Value>(content) else {
            return Vec::new();
        };

        let Some(managed) = json.get(MANAGED_KEY) else {
            return Vec::new();
        };

        let Some(managed_obj) = managed.as_object() else {
            return Vec::new();
        };

        managed_obj
            .iter()
            .filter_map(|(key, value)| {
                let uuid = Uuid::parse_str(key).ok()?;
                let content = serde_json::to_string_pretty(value).ok()?;
                Some(ManagedBlock { uuid, content })
            })
            .collect()
    }

    fn write_block(&self, content: &str, uuid: Uuid, block_content: &str) -> String {
        // Parse existing JSON or create empty object
        let mut json: Value = if content.trim().is_empty() {
            Value::Object(Map::new())
        } else {
            serde_json::from_str(content).unwrap_or(Value::Object(Map::new()))
        };

        // Parse the block content as JSON
        let block_value: Value = serde_json::from_str(block_content)
            .unwrap_or(Value::String(block_content.to_string()));

        // Get or create the managed section
        let obj = json.as_object_mut().expect("Root must be object");
        let managed = obj
            .entry(MANAGED_KEY)
            .or_insert_with(|| Value::Object(Map::new()));

        // Add or update the block
        if let Some(managed_obj) = managed.as_object_mut() {
            managed_obj.insert(uuid.to_string(), block_value);
        }

        // Pretty print with 4-space indentation
        serde_json::to_string_pretty(&json).unwrap_or_else(|_| content.to_string())
    }

    fn remove_block(&self, content: &str, uuid: Uuid) -> String {
        let Ok(mut json) = serde_json::from_str::<Value>(content) else {
            return content.to_string();
        };

        let Some(obj) = json.as_object_mut() else {
            return content.to_string();
        };

        // Get the managed section
        if let Some(managed) = obj.get_mut(MANAGED_KEY) {
            if let Some(managed_obj) = managed.as_object_mut() {
                managed_obj.remove(&uuid.to_string());

                // If managed section is now empty, remove it entirely
                if managed_obj.is_empty() {
                    obj.remove(MANAGED_KEY);
                }
            }
        }

        serde_json::to_string_pretty(&json).unwrap_or_else(|_| content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_content() {
        let handler = JsonFormatHandler::new();
        let blocks = handler.parse_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_no_managed_section() {
        let handler = JsonFormatHandler::new();
        let content = r#"{"user.setting": true}"#;
        let blocks = handler.parse_blocks(content);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_with_managed_blocks() {
        let handler = JsonFormatHandler::new();
        let content = r#"{
            "user.setting": true,
            "__repo_managed__": {
                "550e8400-e29b-41d4-a716-446655440000": {
                    "managed.setting": "value"
                }
            }
        }"#;

        let blocks = handler.parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].uuid,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert!(blocks[0].content.contains("managed.setting"));
    }

    #[test]
    fn test_write_block_to_empty() {
        let handler = JsonFormatHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let result = handler.write_block("", uuid, r#"{"setting": true}"#);
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert!(parsed[MANAGED_KEY]["550e8400-e29b-41d4-a716-446655440000"]["setting"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_write_block_preserves_user_keys() {
        let handler = JsonFormatHandler::new();
        let existing = r#"{
            "editor.formatOnSave": true,
            "python.linting.enabled": true
        }"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.write_block(
            existing,
            uuid,
            r#"{"python.defaultInterpreterPath": ".venv/bin/python"}"#,
        );

        let parsed: Value = serde_json::from_str(&result).unwrap();

        // User keys preserved
        assert_eq!(parsed["editor.formatOnSave"], true);
        assert_eq!(parsed["python.linting.enabled"], true);

        // Managed key added
        assert!(parsed[MANAGED_KEY]["550e8400-e29b-41d4-a716-446655440000"].is_object());
        assert_eq!(
            parsed[MANAGED_KEY]["550e8400-e29b-41d4-a716-446655440000"]
                ["python.defaultInterpreterPath"],
            ".venv/bin/python"
        );
    }

    #[test]
    fn test_write_block_updates_existing() {
        let handler = JsonFormatHandler::new();
        let existing = r#"{
            "__repo_managed__": {
                "550e8400-e29b-41d4-a716-446655440000": {
                    "old.setting": "old"
                }
            }
        }"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.write_block(existing, uuid, r#"{"new.setting": "new"}"#);

        let parsed: Value = serde_json::from_str(&result).unwrap();

        // New content replaces old
        assert_eq!(
            parsed[MANAGED_KEY]["550e8400-e29b-41d4-a716-446655440000"]["new.setting"],
            "new"
        );
        // Old content is gone (replaced entirely)
        assert!(
            parsed[MANAGED_KEY]["550e8400-e29b-41d4-a716-446655440000"]["old.setting"].is_null()
        );
    }

    #[test]
    fn test_remove_block() {
        let handler = JsonFormatHandler::new();
        let existing = r#"{
            "user.setting": true,
            "__repo_managed__": {
                "550e8400-e29b-41d4-a716-446655440000": {
                    "managed.setting": "value"
                }
            }
        }"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.remove_block(existing, uuid);

        let parsed: Value = serde_json::from_str(&result).unwrap();

        // User setting preserved
        assert_eq!(parsed["user.setting"], true);
        // Managed section removed (was only block)
        assert!(parsed.get(MANAGED_KEY).is_none());
    }

    #[test]
    fn test_remove_block_keeps_other_blocks() {
        let handler = JsonFormatHandler::new();
        let existing = r#"{
            "__repo_managed__": {
                "550e8400-e29b-41d4-a716-446655440000": {"a": 1},
                "6ba7b810-9dad-11d1-80b4-00c04fd430c8": {"b": 2}
            }
        }"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.remove_block(existing, uuid);

        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Other block still exists
        assert!(parsed[MANAGED_KEY]["6ba7b810-9dad-11d1-80b4-00c04fd430c8"].is_object());
        // Removed block is gone
        assert!(parsed[MANAGED_KEY]["550e8400-e29b-41d4-a716-446655440000"].is_null());
    }

    #[test]
    fn test_has_block() {
        let handler = JsonFormatHandler::new();
        let content = r#"{
            "__repo_managed__": {
                "550e8400-e29b-41d4-a716-446655440000": {"a": 1}
            }
        }"#;

        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

        assert!(handler.has_block(content, uuid1));
        assert!(!handler.has_block(content, uuid2));
    }

    #[test]
    fn test_get_block() {
        let handler = JsonFormatHandler::new();
        let content = r#"{
            "__repo_managed__": {
                "550e8400-e29b-41d4-a716-446655440000": {"setting": "value"}
            }
        }"#;

        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let block = handler.get_block(content, uuid);

        assert!(block.is_some());
        assert!(block.unwrap().contains("setting"));
    }

    #[test]
    fn test_multiple_blocks() {
        let handler = JsonFormatHandler::new();

        // Start empty, add first block
        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let result = handler.write_block("{}", uuid1, r#"{"first": 1}"#);

        // Add second block
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();
        let result = handler.write_block(&result, uuid2, r#"{"second": 2}"#);

        let parsed: Value = serde_json::from_str(&result).unwrap();

        // Both blocks exist
        assert_eq!(
            parsed[MANAGED_KEY]["550e8400-e29b-41d4-a716-446655440000"]["first"],
            1
        );
        assert_eq!(
            parsed[MANAGED_KEY]["6ba7b810-9dad-11d1-80b4-00c04fd430c8"]["second"],
            2
        );
    }
}
