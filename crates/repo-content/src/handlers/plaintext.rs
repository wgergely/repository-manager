//! Plain text format handler

use uuid::Uuid;

use super::html_comment;
use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

/// Handler for plain text files with HTML comment markers
#[derive(Debug, Default)]
pub struct PlainTextHandler;

impl PlainTextHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for PlainTextHandler {
    fn format(&self) -> Format {
        Format::PlainText
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        Ok(Box::new(source.to_string()))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        html_comment::find_blocks(source)
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)> {
        html_comment::insert_block(source, uuid, content, location)
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        html_comment::update_block(source, uuid, content)
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        html_comment::remove_block(source, uuid)
    }

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        // For plain text, normalize whitespace by trimming line endings and overall content
        let normalized: String = source
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        Ok(serde_json::Value::String(normalized))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<String>()
            .cloned()
            .ok_or_else(|| Error::parse("plaintext", "invalid internal state"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FormatHandler;

    #[test]
    fn test_plaintext_find_blocks() {
        let handler = PlainTextHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "Some text before\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nBlock content here\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nSome text after";
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, uuid);
        assert_eq!(blocks[0].content.trim(), "Block content here");
    }

    #[test]
    fn test_plaintext_insert_block() {
        let handler = PlainTextHandler::new();
        let uuid = Uuid::new_v4();
        let (result, _) = handler
            .insert_block(
                "Existing content\n",
                uuid,
                "New block content",
                BlockLocation::End,
            )
            .unwrap();
        assert!(result.contains("repo:block:"));
        assert!(result.contains("New block content"));
    }

    #[test]
    fn test_plaintext_remove_block() {
        let handler = PlainTextHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "Before\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nContent\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nAfter";
        let (result, _) = handler.remove_block(source, uuid).unwrap();
        assert!(!result.contains("repo:block:"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
    }

    #[test]
    fn test_plaintext_update_block() {
        let handler = PlainTextHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nOriginal content\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->";
        let (result, _) = handler
            .update_block(source, uuid, "Updated content")
            .unwrap();
        assert!(result.contains("Updated content"));
        assert!(!result.contains("Original content"));
    }

    #[test]
    fn test_plaintext_format() {
        let handler = PlainTextHandler::new();
        assert_eq!(handler.format(), Format::PlainText);
    }

    #[test]
    fn test_plaintext_parse_and_render() {
        let handler = PlainTextHandler::new();
        let source = "Hello, World!";
        let parsed = handler.parse(source).unwrap();
        let rendered = handler.render(parsed.as_ref()).unwrap();
        assert_eq!(rendered, source);
    }

    #[test]
    fn test_plaintext_normalize() {
        let handler = PlainTextHandler::new();
        let source = "  Line with trailing spaces   \n  Another line  \n";
        let normalized = handler.normalize(source).unwrap();
        let expected =
            serde_json::Value::String("Line with trailing spaces\n  Another line".to_string());
        assert_eq!(normalized, expected);
    }

    #[test]
    fn test_plaintext_block_not_found_error() {
        let handler = PlainTextHandler::new();
        let uuid = Uuid::new_v4();
        assert!(handler.remove_block("No blocks here", uuid).is_err());
    }

    #[test]
    fn test_plaintext_find_multiple_blocks() {
        let handler = PlainTextHandler::new();
        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let source = "Start\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nFirst block\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nMiddle\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440001 -->\nSecond block\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440001 -->\nEnd";
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].uuid, uuid1);
        assert_eq!(blocks[1].uuid, uuid2);
    }
}
