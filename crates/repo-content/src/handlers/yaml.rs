//! YAML format handler using serde_yaml

use serde_yaml::Value as YamlValue;
use uuid::Uuid;

use super::hash_comment;
use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

/// Handler for YAML files using serde_yaml
#[derive(Debug, Default)]
pub struct YamlHandler;

impl YamlHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for YamlHandler {
    fn format(&self) -> Format {
        Format::Yaml
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let value: YamlValue =
            serde_yaml::from_str(source).map_err(|e| Error::parse("YAML", e.to_string()))?;
        Ok(Box::new(value))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        hash_comment::find_blocks(source)
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)> {
        hash_comment::insert_block(source, uuid, content, location)
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        hash_comment::update_block(source, uuid, content)
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        hash_comment::remove_block(source, uuid)
    }

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        let yaml_value: YamlValue =
            serde_yaml::from_str(source).map_err(|e| Error::parse("YAML", e.to_string()))?;

        fn yaml_to_json_sorted(value: &YamlValue) -> serde_json::Value {
            match value {
                YamlValue::Null => serde_json::Value::Null,
                YamlValue::Bool(b) => serde_json::Value::Bool(*b),
                YamlValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        serde_json::Value::Number(i.into())
                    } else if let Some(u) = n.as_u64() {
                        serde_json::Value::Number(u.into())
                    } else if let Some(f) = n.as_f64() {
                        serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    } else {
                        serde_json::Value::Null
                    }
                }
                YamlValue::String(s) => serde_json::Value::String(s.clone()),
                YamlValue::Sequence(arr) => {
                    let items: Vec<_> = arr.iter().map(yaml_to_json_sorted).collect();
                    serde_json::Value::Array(items)
                }
                YamlValue::Mapping(map) => {
                    let mut json_map = serde_json::Map::new();
                    // Collect keys and sort them
                    let mut keys: Vec<_> = map
                        .keys()
                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                        .collect();
                    keys.sort();

                    for key in keys {
                        if let Some(v) = map.get(YamlValue::String(key.clone())) {
                            json_map.insert(key, yaml_to_json_sorted(v));
                        }
                    }
                    serde_json::Value::Object(json_map)
                }
                YamlValue::Tagged(tagged) => yaml_to_json_sorted(&tagged.value),
            }
        }

        Ok(yaml_to_json_sorted(&yaml_value))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<YamlValue>()
            .map(|value| serde_yaml::to_string(value).unwrap_or_else(|_| String::new()))
            .ok_or_else(|| Error::parse("YAML", "invalid internal state"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::EditKind;
    use crate::format::FormatHandler;

    #[test]
    fn test_yaml_find_blocks() {
        let handler = YamlHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "name: test\nversion: \"1.0\"\n\n# repo:block:550e8400-e29b-41d4-a716-446655440000\nmanaged:\n  key: value\n# /repo:block:550e8400-e29b-41d4-a716-446655440000\n\nother: data\n";
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, uuid);
        assert!(blocks[0].content.contains("managed:"));
    }

    #[test]
    fn test_yaml_normalize_key_order() {
        let handler = YamlHandler::new();
        let norm1 = handler.normalize("b: 2\na: 1\n").unwrap();
        let norm2 = handler.normalize("a: 1\nb: 2\n").unwrap();
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_yaml_parse_error() {
        let handler = YamlHandler::new();
        assert!(handler.parse("invalid: yaml: content: [unclosed").is_err());
    }

    #[test]
    fn test_yaml_insert_block() {
        let handler = YamlHandler::new();
        let uuid = Uuid::new_v4();
        let (result, _) = handler
            .insert_block(
                "name: test\nversion: \"1.0\"\n",
                uuid,
                "managed:\n  key: value",
                BlockLocation::End,
            )
            .unwrap();
        assert!(result.contains("# repo:block:"));
        assert!(result.contains("managed:"));
        assert!(result.contains("# /repo:block:"));
    }

    #[test]
    fn test_yaml_update_block() {
        let handler = YamlHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "name: test\n\n# repo:block:550e8400-e29b-41d4-a716-446655440000\nmanaged:\n  key: old\n# /repo:block:550e8400-e29b-41d4-a716-446655440000\n";
        let (result, edit) = handler
            .update_block(source, uuid, "managed:\n  key: new")
            .unwrap();
        assert!(result.contains("key: new"));
        assert!(!result.contains("key: old"));
        assert_eq!(edit.kind, EditKind::BlockUpdate { uuid });
    }

    #[test]
    fn test_yaml_remove_block() {
        let handler = YamlHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "name: test\n\n# repo:block:550e8400-e29b-41d4-a716-446655440000\nmanaged:\n  key: value\n# /repo:block:550e8400-e29b-41d4-a716-446655440000\n\nother: data\n";
        let (result, edit) = handler.remove_block(source, uuid).unwrap();
        assert!(!result.contains("repo:block"));
        assert!(!result.contains("managed:"));
        assert!(result.contains("name: test"));
        assert!(result.contains("other: data"));
        assert_eq!(edit.kind, EditKind::BlockRemove { uuid });
    }

    #[test]
    fn test_yaml_block_not_found() {
        let handler = YamlHandler::new();
        let uuid = Uuid::new_v4();
        assert!(
            handler
                .update_block("name: test\n", uuid, "new content")
                .is_err()
        );
    }

    #[test]
    fn test_yaml_normalize_nested() {
        let handler = YamlHandler::new();
        let source = "package:\n  name: test\ndependencies:\n  serde:\n    version: \"1.0\"\n";
        let normalized = handler.normalize(source).unwrap();
        assert!(normalized.get("package").is_some());
        assert!(normalized.get("dependencies").is_some());
    }
}
