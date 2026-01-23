//! TOML format handler using toml_edit

use std::sync::LazyLock;

use regex::Regex;
use toml_edit::DocumentMut;
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::{Edit, EditKind};
use crate::error::{Error, Result};
use crate::format::{CommentStyle, Format, FormatHandler};

/// Pattern to match block start markers and capture the UUID
static BLOCK_START_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#\s*repo:block:([0-9a-f-]{36})").unwrap());

/// Handler for TOML files using toml_edit for format preservation
#[derive(Debug, Default)]
pub struct TomlHandler;

impl TomlHandler {
    pub fn new() -> Self {
        Self
    }

    fn find_block_end(source: &str, uuid: &Uuid, start_pos: usize) -> Option<usize> {
        let end_marker = format!("# /repo:block:{uuid}");
        source[start_pos..].find(&end_marker).map(|pos| {
            let abs_pos = start_pos + pos + end_marker.len();
            // Include trailing newline if present
            if source[abs_pos..].starts_with('\n') {
                abs_pos + 1
            } else {
                abs_pos
            }
        })
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
        let mut blocks = Vec::new();

        for cap in BLOCK_START_PATTERN.captures_iter(source) {
            let uuid_str = match cap.get(1) {
                Some(m) => m.as_str(),
                None => continue,
            };
            let uuid = match Uuid::parse_str(uuid_str) {
                Ok(u) => u,
                Err(_) => continue,
            };

            let start_match = cap.get(0).unwrap();
            let block_start = start_match.start();
            let content_start = start_match.end();

            let Some(block_end) = Self::find_block_end(source, &uuid, content_start) else {
                continue;
            };

            // Find where content ends (before the end marker)
            let end_marker = format!("# /repo:block:{uuid}");
            let content_end = source[content_start..]
                .find(&end_marker)
                .map(|p| content_start + p)
                .unwrap_or(block_end);

            // Extract content between markers (skip leading newline if present)
            let content = &source[content_start..content_end];
            let content = content.strip_prefix('\n').unwrap_or(content);

            blocks.push(ManagedBlock::new(uuid, content, block_start..block_end));
        }

        blocks
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)> {
        let style = CommentStyle::Hash;
        let block_text = format!(
            "{}\n{}\n{}\n",
            style.format_start(uuid),
            content,
            style.format_end(uuid)
        );

        let position = match location {
            BlockLocation::End => source.len(),
            BlockLocation::Offset(pos) => pos.min(source.len()),
            BlockLocation::After(ref marker) => source
                .find(marker)
                .and_then(|p| source[p..].find('\n').map(|np| p + np + 1))
                .unwrap_or(source.len()),
            BlockLocation::Before(ref marker) => source.find(marker).unwrap_or(source.len()),
        };

        let mut result = String::with_capacity(source.len() + block_text.len());
        result.push_str(&source[..position]);
        if position > 0 && !source[..position].ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&block_text);
        result.push_str(&source[position..]);

        let edit = Edit {
            kind: EditKind::BlockInsert { uuid },
            span: position..position + block_text.len(),
            old_content: String::new(),
            new_content: block_text,
        };

        Ok((result, edit))
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        let blocks = self.find_blocks(source);
        let block = blocks
            .iter()
            .find(|b| b.uuid == uuid)
            .ok_or(Error::BlockNotFound { uuid })?;

        let style = CommentStyle::Hash;
        let new_block = format!(
            "{}\n{}\n{}",
            style.format_start(uuid),
            content,
            style.format_end(uuid)
        );

        let edit = Edit {
            kind: EditKind::BlockUpdate { uuid },
            span: block.span.clone(),
            old_content: source[block.span.clone()].to_string(),
            new_content: new_block.clone(),
        };

        let result = edit.apply(source);
        Ok((result, edit))
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        let blocks = self.find_blocks(source);
        let block = blocks
            .iter()
            .find(|b| b.uuid == uuid)
            .ok_or(Error::BlockNotFound { uuid })?;

        let edit = Edit {
            kind: EditKind::BlockRemove { uuid },
            span: block.span.clone(),
            old_content: source[block.span.clone()].to_string(),
            new_content: String::new(),
        };

        let result = edit.apply(source);
        Ok((result, edit))
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
