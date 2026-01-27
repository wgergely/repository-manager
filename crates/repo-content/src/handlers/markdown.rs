//! Markdown format handler using tree-sitter-md

use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use super::html_comment;
use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

/// Pattern to match multiple consecutive blank lines (markdown-specific normalization)
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
    fn test_multiple_blank_lines_pattern() {
        let source = "a\n\n\n\nb";
        let result = MULTIPLE_BLANK_LINES.replace_all(source, "\n\n");
        assert_eq!(result, "a\n\nb");
    }
}
