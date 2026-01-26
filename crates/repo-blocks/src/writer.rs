//! Block writing functionality for managed blocks.
//!
//! Provides functions to insert, update, remove, and upsert UUID-tagged blocks
//! in text content.

use crate::error::{Error, Result};
use crate::parser::has_block;
use regex::Regex;
use std::path::PathBuf;

/// Creates the opening marker for a block.
fn opening_marker(uuid: &str) -> String {
    format!("<!-- repo:block:{} -->", uuid)
}

/// Creates the closing marker for a block.
fn closing_marker(uuid: &str) -> String {
    format!("<!-- /repo:block:{} -->", uuid)
}

/// Creates a complete block with markers and content.
fn format_block(uuid: &str, block_content: &str) -> String {
    format!(
        "{}\n{}\n{}",
        opening_marker(uuid),
        block_content,
        closing_marker(uuid)
    )
}

/// Inserts a new block at the end of the content.
///
/// If the content is empty, the block is added directly.
/// If the content has existing text, the block is appended with a newline separator.
///
/// # Arguments
/// * `content` - The existing content
/// * `uuid` - The UUID for the new block
/// * `block_content` - The content to place inside the block
///
/// # Returns
/// The content with the new block appended.
///
/// # Example
/// ```
/// use repo_blocks::writer::insert_block;
///
/// let content = "existing content";
/// let result = insert_block(content, "abc-123", "new block");
/// assert!(result.contains("<!-- repo:block:abc-123 -->"));
/// ```
pub fn insert_block(content: &str, uuid: &str, block_content: &str) -> String {
    let block = format_block(uuid, block_content);

    if content.is_empty() {
        block
    } else {
        format!("{}\n\n{}", content, block)
    }
}

/// Updates an existing block's content.
///
/// # Arguments
/// * `content` - The content containing the block
/// * `uuid` - The UUID of the block to update
/// * `new_content` - The new content for the block
///
/// # Returns
/// The content with the block updated, or an error if the block doesn't exist.
///
/// # Errors
/// Returns `Error::BlockNotFound` if no block with the given UUID exists.
///
/// # Example
/// ```
/// use repo_blocks::writer::update_block;
///
/// let content = r#"<!-- repo:block:abc-123 -->
/// old content
/// <!-- /repo:block:abc-123 -->"#;
///
/// let result = update_block(content, "abc-123", "new content").unwrap();
/// assert!(result.contains("new content"));
/// assert!(!result.contains("old content"));
/// ```
pub fn update_block(content: &str, uuid: &str, new_content: &str) -> Result<String> {
    if !has_block(content, uuid) {
        return Err(Error::BlockNotFound {
            uuid: uuid.to_string(),
            path: PathBuf::from("<content>"),
        });
    }

    // Build regex to match this specific block
    let pattern = format!(
        r"(?s)<!-- repo:block:{} -->\n.*?\n<!-- /repo:block:{} -->",
        regex::escape(uuid),
        regex::escape(uuid)
    );
    let re = Regex::new(&pattern)?;

    let replacement = format_block(uuid, new_content);
    Ok(re.replace(content, replacement.as_str()).to_string())
}

/// Removes a block from the content.
///
/// # Arguments
/// * `content` - The content containing the block
/// * `uuid` - The UUID of the block to remove
///
/// # Returns
/// The content with the block removed, or an error if the block doesn't exist.
///
/// # Errors
/// Returns `Error::BlockNotFound` if no block with the given UUID exists.
///
/// # Example
/// ```
/// use repo_blocks::writer::remove_block;
///
/// let content = r#"before
/// <!-- repo:block:abc-123 -->
/// block content
/// <!-- /repo:block:abc-123 -->
/// after"#;
///
/// let result = remove_block(content, "abc-123").unwrap();
/// assert!(result.contains("before"));
/// assert!(result.contains("after"));
/// assert!(!result.contains("block content"));
/// ```
pub fn remove_block(content: &str, uuid: &str) -> Result<String> {
    if !has_block(content, uuid) {
        return Err(Error::BlockNotFound {
            uuid: uuid.to_string(),
            path: PathBuf::from("<content>"),
        });
    }

    // Build regex to match this specific block, including surrounding newlines
    let pattern = format!(
        r"(?s)\n?\n?<!-- repo:block:{} -->\n.*?\n<!-- /repo:block:{} -->\n?\n?",
        regex::escape(uuid),
        regex::escape(uuid)
    );
    let re = Regex::new(&pattern)?;

    let result = re.replace(content, "\n").to_string();

    // Clean up any leading/trailing whitespace issues
    let result = result.trim_start_matches('\n').to_string();

    Ok(result)
}

/// Inserts a new block or updates an existing one.
///
/// If a block with the given UUID exists, its content is updated.
/// Otherwise, a new block is inserted at the end.
///
/// # Arguments
/// * `content` - The existing content
/// * `uuid` - The UUID for the block
/// * `block_content` - The content for the block
///
/// # Returns
/// The content with the block inserted or updated.
///
/// # Errors
/// Returns an error if regex compilation fails (should not happen with valid UUIDs).
///
/// # Example
/// ```
/// use repo_blocks::writer::upsert_block;
///
/// // Insert new block
/// let content = "";
/// let result = upsert_block(content, "abc-123", "content").unwrap();
/// assert!(result.contains("abc-123"));
///
/// // Update existing block
/// let result = upsert_block(&result, "abc-123", "new content").unwrap();
/// assert!(result.contains("new content"));
/// ```
pub fn upsert_block(content: &str, uuid: &str, block_content: &str) -> Result<String> {
    if has_block(content, uuid) {
        update_block(content, uuid, block_content)
    } else {
        Ok(insert_block(content, uuid, block_content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_to_empty() {
        let result = insert_block("", "abc-123", "content");
        assert_eq!(
            result,
            "<!-- repo:block:abc-123 -->\ncontent\n<!-- /repo:block:abc-123 -->"
        );
    }

    #[test]
    fn test_insert_to_existing() {
        let result = insert_block("existing", "abc-123", "content");
        assert!(result.starts_with("existing"));
        assert!(result.contains("<!-- repo:block:abc-123 -->"));
    }

    #[test]
    fn test_update_existing() {
        let content = "<!-- repo:block:abc-123 -->\nold\n<!-- /repo:block:abc-123 -->";
        let result = update_block(content, "abc-123", "new").unwrap();
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
    }

    #[test]
    fn test_update_nonexistent_fails() {
        let content = "no blocks here";
        let result = update_block(content, "abc-123", "content");
        assert!(result.is_err());
    }

    #[test]
    fn test_upsert_insert() {
        let result = upsert_block("", "abc-123", "content").unwrap();
        assert!(result.contains("abc-123"));
    }

    #[test]
    fn test_upsert_update() {
        let content = "<!-- repo:block:abc-123 -->\nold\n<!-- /repo:block:abc-123 -->";
        let result = upsert_block(content, "abc-123", "new").unwrap();
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
    }
}
