//! TOML format handler using toml_edit

use toml_edit::DocumentMut;
use uuid::Uuid;

use super::hash_comment;
use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

/// Handler for TOML files using toml_edit for format preservation
#[derive(Debug, Default)]
pub struct TomlHandler;

impl TomlHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for TomlHandler {
    fn format(&self) -> Format {
        Format::Toml
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let doc: DocumentMut = source
            .parse()
            .map_err(|e: toml_edit::TomlError| Error::parse("TOML", e.to_string()))?;
        Ok(Box::new(doc))
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
        let doc: DocumentMut = source
            .parse()
            .map_err(|e: toml_edit::TomlError| Error::parse("TOML", e.to_string()))?;

        fn table_to_json(table: &toml_edit::Table) -> serde_json::Value {
            let mut map = serde_json::Map::new();
            let mut keys: Vec<_> = table.iter().map(|(k, _)| k.to_string()).collect();
            keys.sort();

            for key in keys {
                if let Some(item) = table.get(&key) {
                    map.insert(key, item_to_json(item));
                }
            }
            serde_json::Value::Object(map)
        }

        fn item_to_json(item: &toml_edit::Item) -> serde_json::Value {
            match item {
                toml_edit::Item::Value(v) => value_to_json(v),
                toml_edit::Item::Table(t) => table_to_json(t),
                toml_edit::Item::ArrayOfTables(arr) => {
                    let items: Vec<_> = arr.iter().map(table_to_json).collect();
                    serde_json::Value::Array(items)
                }
                toml_edit::Item::None => serde_json::Value::Null,
            }
        }

        fn value_to_json(v: &toml_edit::Value) -> serde_json::Value {
            match v {
                toml_edit::Value::String(s) => serde_json::Value::String(s.value().to_string()),
                toml_edit::Value::Integer(i) => serde_json::Value::Number((*i.value()).into()),
                toml_edit::Value::Float(f) => serde_json::Number::from_f64(*f.value())
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null),
                toml_edit::Value::Boolean(b) => serde_json::Value::Bool(*b.value()),
                toml_edit::Value::Datetime(d) => serde_json::Value::String(d.to_string()),
                toml_edit::Value::Array(arr) => {
                    let items: Vec<_> = arr.iter().map(value_to_json).collect();
                    serde_json::Value::Array(items)
                }
                toml_edit::Value::InlineTable(t) => {
                    let mut map = serde_json::Map::new();
                    let mut keys: Vec<_> = t.iter().map(|(k, _)| k.to_string()).collect();
                    keys.sort();
                    for key in keys {
                        if let Some(v) = t.get(&key) {
                            map.insert(key, value_to_json(v));
                        }
                    }
                    serde_json::Value::Object(map)
                }
            }
        }

        Ok(table_to_json(doc.as_table()))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<DocumentMut>()
            .map(|doc| doc.to_string())
            .ok_or_else(|| Error::parse("TOML", "invalid internal state"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockLocation;
    use crate::edit::EditKind;
    use crate::format::FormatHandler;

    #[test]
    fn test_toml_find_blocks() {
        let handler = TomlHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "[package]\nname = \"test\"\n\n# repo:block:550e8400-e29b-41d4-a716-446655440000\n[managed]\nkey = \"value\"\n# /repo:block:550e8400-e29b-41d4-a716-446655440000\n\n[other]\nfoo = \"bar\"\n";
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, uuid);
        assert!(blocks[0].content.contains("[managed]"));
    }

    #[test]
    fn test_toml_format_preserving_edit() {
        let handler = TomlHandler::new();
        let source =
            "[package]\nname = \"test\"\n\n# This is a comment\n[dependencies]\nserde = \"1.0\"\n";
        let parsed = handler.parse(source).unwrap();
        let rendered = handler.render(parsed.as_ref()).unwrap();
        assert!(rendered.contains("# This is a comment"));
    }

    #[test]
    fn test_toml_normalize() {
        let handler = TomlHandler::new();
        let source1 = "[a]\nx = 1\n[b]\ny = 2";
        let source2 = "[b]\ny = 2\n[a]\nx = 1";
        let norm1 = handler.normalize(source1).unwrap();
        let norm2 = handler.normalize(source2).unwrap();
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_toml_insert_block() {
        let handler = TomlHandler::new();
        let uuid = Uuid::new_v4();
        let (result, _) = handler
            .insert_block(
                "[package]\nname = \"test\"\n",
                uuid,
                "[managed]\nkey = \"value\"",
                BlockLocation::End,
            )
            .unwrap();
        assert!(result.contains("# repo:block:"));
        assert!(result.contains("[managed]"));
        assert!(result.contains("# /repo:block:"));
    }

    #[test]
    fn test_toml_update_block() {
        let handler = TomlHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "[package]\nname = \"test\"\n\n# repo:block:550e8400-e29b-41d4-a716-446655440000\n[managed]\nkey = \"old\"\n# /repo:block:550e8400-e29b-41d4-a716-446655440000\n";
        let (result, edit) = handler
            .update_block(source, uuid, "[managed]\nkey = \"new\"")
            .unwrap();
        assert!(result.contains("key = \"new\""));
        assert!(!result.contains("key = \"old\""));
        assert_eq!(edit.kind, EditKind::BlockUpdate { uuid });
    }

    #[test]
    fn test_toml_remove_block() {
        let handler = TomlHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "[package]\nname = \"test\"\n\n# repo:block:550e8400-e29b-41d4-a716-446655440000\n[managed]\nkey = \"value\"\n# /repo:block:550e8400-e29b-41d4-a716-446655440000\n\n[other]\nfoo = \"bar\"\n";
        let (result, edit) = handler.remove_block(source, uuid).unwrap();
        assert!(!result.contains("repo:block"));
        assert!(!result.contains("[managed]"));
        assert!(result.contains("[package]"));
        assert!(result.contains("[other]"));
        assert_eq!(edit.kind, EditKind::BlockRemove { uuid });
    }

    #[test]
    fn test_toml_parse_error() {
        let handler = TomlHandler::new();
        assert!(handler.parse("[invalid\nkey = ").is_err());
    }

    #[test]
    fn test_toml_block_not_found() {
        let handler = TomlHandler::new();
        let uuid = Uuid::new_v4();
        assert!(
            handler
                .update_block("[package]\nname = \"test\"\n", uuid, "new content")
                .is_err()
        );
    }

    #[test]
    fn test_toml_normalize_nested_tables() {
        let handler = TomlHandler::new();
        let source = "[package]\nname = \"test\"\n\n[package.metadata]\ncustom = \"value\"\n\n[dependencies]\nserde = { version = \"1.0\", features = [\"derive\"] }\n";
        let normalized = handler.normalize(source).unwrap();
        assert!(normalized.get("package").is_some());
        assert!(normalized.get("dependencies").is_some());
    }

    #[test]
    fn test_toml_normalize_arrays() {
        let handler = TomlHandler::new();
        let source = "[[bin]]\nname = \"first\"\npath = \"src/bin/first.rs\"\n\n[[bin]]\nname = \"second\"\npath = \"src/bin/second.rs\"\n";
        let normalized = handler.normalize(source).unwrap();
        let bin_array = normalized.get("bin").unwrap();
        assert!(bin_array.is_array());
        assert_eq!(bin_array.as_array().unwrap().len(), 2);
    }
}
