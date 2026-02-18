//! Shared hash-comment block operations for TOML and YAML handlers
//!
//! Both TOML and YAML use `# repo:block:<uuid>` / `# /repo:block:<uuid>`
//! comment markers. This module provides the shared logic, following the
//! same pattern as `html_comment.rs` does for HTML-style markers.

use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::{Edit, EditKind};
use crate::error::{Error, Result};
use crate::format::CommentStyle;

/// Pattern to match hash-comment block start markers and capture the UUID
pub static BLOCK_START_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#\s*repo:block:([0-9a-f-]{36})").unwrap());

/// Find the end position of a hash-comment block (after the end marker and trailing newline)
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

/// Find all managed blocks using hash-comment markers
pub fn find_blocks(source: &str) -> Vec<ManagedBlock> {
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

        let Some(block_end) = find_block_end(source, &uuid, content_start) else {
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

/// Insert a managed block using hash-comment markers
pub fn insert_block(
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

/// Update a managed block using hash-comment markers
pub fn update_block(source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
    let blocks = find_blocks(source);
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

/// Remove a managed block using hash-comment markers
pub fn remove_block(source: &str, uuid: Uuid) -> Result<(String, Edit)> {
    let blocks = find_blocks(source);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_start_pattern_matches() {
        let source = "# repo:block:550e8400-e29b-41d4-a716-446655440000";
        assert!(BLOCK_START_PATTERN.is_match(source));
    }

    #[test]
    fn test_find_blocks_empty() {
        let blocks = find_blocks("no blocks here");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_find_blocks_single() {
        let source = "prefix\n# repo:block:550e8400-e29b-41d4-a716-446655440000\ncontent\n# /repo:block:550e8400-e29b-41d4-a716-446655440000\nsuffix";
        let blocks = find_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content.trim(), "content");
    }

    #[test]
    fn test_insert_block_at_end() {
        let (result, _edit) = insert_block(
            "existing content",
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            "new block",
            BlockLocation::End,
        )
        .unwrap();
        assert!(result.contains("existing content"));
        assert!(result.contains("new block"));
        assert!(result.contains("# repo:block:550e8400"));
    }
}
