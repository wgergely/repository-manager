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
static MULTIPLE_BLANK_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

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
