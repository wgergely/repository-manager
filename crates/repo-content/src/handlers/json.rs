//! JSON format handler

use serde_json::{Map, Value};
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::{Edit, EditKind};
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

const MANAGED_KEY: &str = "_repo_managed";

/// Handler for JSON files
#[derive(Debug, Default)]
pub struct JsonHandler;

impl JsonHandler {
    pub fn new() -> Self {
        Self
    }

    fn sort_value(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut sorted = Map::new();
                let mut keys: Vec<_> = map.keys().collect();
                keys.sort();
                for key in keys {
                    if let Some(v) = map.get(key) {
                        sorted.insert(key.clone(), Self::sort_value(v));
                    }
                }
                Value::Object(sorted)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(Self::sort_value).collect()),
            other => other.clone(),
        }
    }
}

impl FormatHandler for JsonHandler {
    fn format(&self) -> Format {
        Format::Json
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let value: Value = serde_json::from_str(source)?;
        Ok(Box::new(value))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        let Ok(value) = serde_json::from_str::<Value>(source) else {
            return Vec::new();
        };

        let Some(managed) = value.get(MANAGED_KEY).and_then(Value::as_object) else {
            return Vec::new();
        };

        managed
            .iter()
            .filter_map(|(uuid_str, content)| {
                let uuid = Uuid::parse_str(uuid_str).ok()?;
                let content_str = serde_json::to_string_pretty(content).ok()?;
                // Approximate span - JSON doesn't have precise spans without tree-sitter
                Some(ManagedBlock::new(uuid, content_str, 0..0))
            })
            .collect()
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        _location: BlockLocation,
    ) -> Result<(String, Edit)> {
        let mut value: Value = serde_json::from_str(source)?;

        let content_value: Value =
            serde_json::from_str(content).unwrap_or_else(|_| Value::String(content.to_string()));

        let managed = value
            .as_object_mut()
            .ok_or_else(|| Error::parse("JSON", "root must be object"))?
            .entry(MANAGED_KEY)
            .or_insert_with(|| Value::Object(Map::new()));

        if let Some(obj) = managed.as_object_mut() {
            obj.insert(uuid.to_string(), content_value);
        }

        let new_source = serde_json::to_string_pretty(&value)?;

        let edit = Edit {
            kind: EditKind::BlockInsert { uuid },
            span: 0..source.len(),
            old_content: source.to_string(),
            new_content: new_source.clone(),
        };

        Ok((new_source, edit))
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        let mut value: Value = serde_json::from_str(source)?;

        let content_value: Value =
            serde_json::from_str(content).unwrap_or_else(|_| Value::String(content.to_string()));

        let managed = value
            .get_mut(MANAGED_KEY)
            .and_then(Value::as_object_mut)
            .ok_or(Error::BlockNotFound { uuid })?;

        if !managed.contains_key(&uuid.to_string()) {
            return Err(Error::BlockNotFound { uuid });
        }

        managed.insert(uuid.to_string(), content_value);

        let new_source = serde_json::to_string_pretty(&value)?;

        let edit = Edit {
            kind: EditKind::BlockUpdate { uuid },
            span: 0..source.len(),
            old_content: source.to_string(),
            new_content: new_source.clone(),
        };

        Ok((new_source, edit))
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        let mut value: Value = serde_json::from_str(source)?;

        let managed = value
            .get_mut(MANAGED_KEY)
            .and_then(Value::as_object_mut)
            .ok_or(Error::BlockNotFound { uuid })?;

        if managed.remove(&uuid.to_string()).is_none() {
            return Err(Error::BlockNotFound { uuid });
        }

        // Remove _repo_managed if empty
        if managed.is_empty() {
            value.as_object_mut().unwrap().remove(MANAGED_KEY);
        }

        let new_source = serde_json::to_string_pretty(&value)?;

        let edit = Edit {
            kind: EditKind::BlockRemove { uuid },
            span: 0..source.len(),
            old_content: source.to_string(),
            new_content: new_source.clone(),
        };

        Ok((new_source, edit))
    }

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        let mut value: Value = serde_json::from_str(source)?;

        // Remove _repo_managed for comparison
        if let Some(obj) = value.as_object_mut() {
            obj.remove(MANAGED_KEY);
        }

        // Sort all keys recursively
        Ok(Self::sort_value(&value))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<Value>()
            .map(|v| serde_json::to_string_pretty(v).unwrap_or_default())
            .ok_or_else(|| Error::parse("JSON", "invalid internal state"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FormatHandler;
    use serde_json::json;

    #[test]
    fn test_json_find_blocks() {
        let handler = JsonHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = r#"{"name": "test", "_repo_managed": {"550e8400-e29b-41d4-a716-446655440000": {"key": "value"}}, "other": "data"}"#;
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, uuid);
    }

    #[test]
    fn test_json_normalize_key_order() {
        let handler = JsonHandler::new();
        let norm1 = handler.normalize(r#"{"b": 2, "a": 1}"#).unwrap();
        let norm2 = handler.normalize(r#"{"a": 1, "b": 2}"#).unwrap();
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_json_normalize_removes_managed() {
        let handler = JsonHandler::new();
        let source = r#"{"data": "value", "_repo_managed": {"uuid": {}}}"#;
        let normalized = handler.normalize(source).unwrap();
        assert!(normalized.get("_repo_managed").is_none());
        assert_eq!(normalized.get("data"), Some(&json!("value")));
    }

    #[test]
    fn test_json_insert_block() {
        let handler = JsonHandler::new();
        let uuid = Uuid::new_v4();
        let (result, _) = handler
            .insert_block(
                r#"{"name": "test"}"#,
                uuid,
                r#"{"key": "value"}"#,
                BlockLocation::End,
            )
            .unwrap();
        assert!(result.contains("_repo_managed"));
        assert!(result.contains(&uuid.to_string()));
    }

    #[test]
    fn test_json_remove_block() {
        let handler = JsonHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = r#"{"name": "test", "_repo_managed": {"550e8400-e29b-41d4-a716-446655440000": {"key": "value"}}}"#;
        let (result, _) = handler.remove_block(source, uuid).unwrap();
        assert!(!result.contains("_repo_managed"));
        assert!(result.contains("name"));
    }

    #[test]
    fn test_json_update_block() {
        let handler = JsonHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = r#"{"name": "test", "_repo_managed": {"550e8400-e29b-41d4-a716-446655440000": {"key": "old"}}}"#;
        let (result, edit) = handler
            .update_block(source, uuid, r#"{"key": "new"}"#)
            .unwrap();
        assert!(result.contains("\"new\""));
        assert!(!result.contains("\"old\""));
        assert_eq!(edit.kind, EditKind::BlockUpdate { uuid });
    }

    #[test]
    fn test_json_parse_error() {
        let handler = JsonHandler::new();
        assert!(handler.parse("{invalid: json").is_err());
    }

    #[test]
    fn test_json_block_not_found() {
        let handler = JsonHandler::new();
        let uuid = Uuid::new_v4();
        assert!(
            handler
                .update_block(r#"{"name": "test"}"#, uuid, r#"{"new": "content"}"#)
                .is_err()
        );
    }

    #[test]
    fn test_json_parse_render_round_trip() {
        let handler = JsonHandler::new();
        let source = r#"{"name": "test", "value": 42, "nested": {"a": [1, 2, 3], "b": true}}"#;
        let parsed = handler.parse(source).unwrap();
        let rendered = handler.render(parsed.as_ref()).unwrap();
        let norm1 = handler.normalize(source).unwrap();
        let norm2 = handler.normalize(&rendered).unwrap();
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_json_insert_non_object_root() {
        let handler = JsonHandler::new();
        let uuid = Uuid::new_v4();
        assert!(
            handler
                .insert_block(
                    r#"[1, 2, 3]"#,
                    uuid,
                    r#"{"key": "value"}"#,
                    BlockLocation::End
                )
                .is_err()
        );
    }

    #[test]
    fn test_json_multiple_blocks() {
        let handler = JsonHandler::new();
        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let source = r#"{"name": "test", "_repo_managed": {"550e8400-e29b-41d4-a716-446655440000": {"key": "value1"}, "550e8400-e29b-41d4-a716-446655440001": {"key": "value2"}}}"#;
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 2);
        let uuids: Vec<_> = blocks.iter().map(|b| b.uuid).collect();
        assert!(uuids.contains(&uuid1));
        assert!(uuids.contains(&uuid2));
    }

    #[test]
    fn test_json_remove_keeps_other_blocks() {
        let handler = JsonHandler::new();
        let uuid_to_remove = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid_to_keep = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let source = r#"{"name": "test", "_repo_managed": {"550e8400-e29b-41d4-a716-446655440000": {"key": "value1"}, "550e8400-e29b-41d4-a716-446655440001": {"key": "value2"}}}"#;
        let (result, _) = handler.remove_block(source, uuid_to_remove).unwrap();
        assert!(result.contains("_repo_managed"));
        assert!(result.contains(&uuid_to_keep.to_string()));
        assert!(!result.contains(&uuid_to_remove.to_string()));
    }
}
