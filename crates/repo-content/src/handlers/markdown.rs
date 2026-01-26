//! Markdown format handler using tree-sitter-md

use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::{Edit, EditKind};
use crate::error::{Error, Result};
use crate::format::{CommentStyle, Format, FormatHandler};

/// Pattern to match block start markers and capture the UUID
static BLOCK_START_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<!--\s*repo:block:([0-9a-f-]{36})\s*-->").unwrap()
});

/// Pattern to match multiple consecutive blank lines
static MULTIPLE_BLANK_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

/// Handler for Markdown files with HTML comment markers
///
/// Uses tree-sitter-md for parsing (available for advanced queries)
/// and regex for block detection (since HTML comments work well with regex).
#[derive(Default)]
pub struct MarkdownHandler {
    /// Tree-sitter parser for advanced markdown queries (optional, not used in basic operations)
    #[allow(dead_code)]
    parser_initialized: bool,
}

impl std::fmt::Debug for MarkdownHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarkdownHandler")
            .field("parser_initialized", &self.parser_initialized)
            .finish()
    }
}

impl MarkdownHandler {
    /// Create a new MarkdownHandler
    pub fn new() -> Self {
        // Initialize tree-sitter parser for potential future use
        // The tree-sitter-md language is available via tree_sitter_md::LANGUAGE
        let mut parser = tree_sitter::Parser::new();
        let initialized = parser.set_language(&tree_sitter_md::LANGUAGE.into()).is_ok();

        Self {
            parser_initialized: initialized,
        }
    }
}

impl FormatHandler for MarkdownHandler {
    fn format(&self) -> Format {
        Format::Markdown
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        // Store source as String for rendering
        // tree-sitter parsing is available but not required for basic operations
        Ok(Box::new(source.to_string()))
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

            // Find the corresponding end marker
            let end_marker = format!("<!-- /repo:block:{uuid} -->");
            let Some(end_pos) = source[content_start..].find(&end_marker) else {
                continue;
            };
            let end_pos = content_start + end_pos;
            let block_end = end_pos + end_marker.len();

            // Skip trailing newline if present
            let block_end = if source[block_end..].starts_with('\n') {
                block_end + 1
            } else {
                block_end
            };

            // Extract content between markers (skip leading newline if present)
            let content = &source[content_start..end_pos];
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
        let style = CommentStyle::Html;
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
                .map(|p| p + marker.len())
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

        let style = CommentStyle::Html;
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
        // For Markdown, normalize by:
        // 1. Trimming trailing whitespace per line
        // 2. Collapsing multiple blank lines to a single blank line
        let mut normalized: String = source
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        // Collapse multiple consecutive blank lines (\n\n\n+) to single blank line (\n\n)
        normalized = MULTIPLE_BLANK_LINES.replace_all(&normalized, "\n\n").to_string();

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

    #[test]
    fn test_block_start_pattern_matches() {
        let source = "<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->";
        assert!(BLOCK_START_PATTERN.is_match(source));
    }

    #[test]
    fn test_multiple_blank_lines_pattern() {
        let source = "a\n\n\n\nb";
        let result = MULTIPLE_BLANK_LINES.replace_all(source, "\n\n");
        assert_eq!(result, "a\n\nb");
    }
}
