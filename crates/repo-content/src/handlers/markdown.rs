//! Markdown format handler
//!
//! Uses HTML comment markers for managed blocks, with markdown-specific
//! normalization (collapsing multiple blank lines).

use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use super::html_comment;
use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

/// Pattern to match multiple consecutive blank lines (markdown-specific normalization)
static MULTIPLE_BLANK_LINES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

/// Handler for Markdown files with HTML comment markers
#[derive(Debug, Default)]
pub struct MarkdownHandler;

impl MarkdownHandler {
    /// Create a new MarkdownHandler
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for MarkdownHandler {
    fn format(&self) -> Format {
        Format::Markdown
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
        // For Markdown, normalize by:
        // 1. Trimming trailing whitespace per line
        // 2. Collapsing multiple blank lines to a single blank line
        let mut normalized: String = source
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple consecutive blank lines (\n\n\n+) to single blank line (\n\n)
        normalized = MULTIPLE_BLANK_LINES
            .replace_all(&normalized, "\n\n")
            .to_string();

        // Trim overall content
        normalized = normalized.trim().to_string();

        Ok(serde_json::Value::String(normalized))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<String>()
            .cloned()
            .ok_or_else(|| Error::parse("markdown", "invalid internal state"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::EditKind;
    use crate::format::FormatHandler;

    #[test]
    fn test_multiple_blank_lines_pattern() {
        let source = "a\n\n\n\nb";
        let result = MULTIPLE_BLANK_LINES.replace_all(source, "\n\n");
        assert_eq!(result, "a\n\nb");
    }

    #[test]
    fn test_markdown_find_blocks() {
        let handler = MarkdownHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "# My Document\n\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nManaged content here\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\n\nMore content.\n";
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, uuid);
        assert!(blocks[0].content.contains("Managed content"));
    }

    #[test]
    fn test_markdown_insert_block() {
        let handler = MarkdownHandler::new();
        let uuid = Uuid::new_v4();
        let (result, _) = handler
            .insert_block(
                "# Title\n\nContent here.\n",
                uuid,
                "New managed section",
                BlockLocation::End,
            )
            .unwrap();
        assert!(result.contains("repo:block:"));
        assert!(result.contains("New managed section"));
    }

    #[test]
    fn test_markdown_normalize() {
        let handler = MarkdownHandler::new();
        let norm1 = handler.normalize("# Title\n\n\n\nContent").unwrap();
        let norm2 = handler.normalize("# Title\n\nContent").unwrap();
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_markdown_remove_block() {
        let handler = MarkdownHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "Before\n\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nContent\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\n\nAfter";
        let (result, _) = handler.remove_block(source, uuid).unwrap();
        assert!(!result.contains("repo:block:"));
        assert!(result.contains("Before"));
        assert!(result.contains("After"));
    }

    #[test]
    fn test_markdown_update_block() {
        let handler = MarkdownHandler::new();
        let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let source = "# Title\n\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nOld content\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\n\nFooter\n";
        let (result, edit) = handler.update_block(source, uuid, "New content").unwrap();
        assert!(result.contains("New content"));
        assert!(!result.contains("Old content"));
        assert_eq!(edit.kind, EditKind::BlockUpdate { uuid });
    }

    #[test]
    fn test_markdown_block_not_found() {
        let handler = MarkdownHandler::new();
        let uuid = Uuid::new_v4();
        assert!(
            handler
                .update_block("# Title\n\nNo blocks here.\n", uuid, "new content")
                .is_err()
        );
    }

    #[test]
    fn test_markdown_format() {
        let handler = MarkdownHandler::new();
        assert_eq!(handler.format(), Format::Markdown);
    }

    #[test]
    fn test_markdown_multiple_blocks() {
        let handler = MarkdownHandler::new();
        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
        let source = "# Document\n\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nFirst block\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\n\nMiddle\n\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440001 -->\nSecond block\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440001 -->\n\nEnd\n";
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].uuid, uuid1);
        assert_eq!(blocks[1].uuid, uuid2);
    }
}
