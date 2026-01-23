//! Plain text format handler

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
