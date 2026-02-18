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
    let re = Regex::new(&pattern).expect("UUID should produce valid regex pattern");

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
    let re = Regex::new(&pattern).expect("UUID should produce valid regex pattern");

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

    #[test]
    fn test_insert_to_empty_file() {
        let result = insert_block("", "test-uuid", "my content");

        assert!(result.contains("<!-- repo:block:test-uuid -->"));
        assert!(result.contains("my content"));
        assert!(result.contains("<!-- /repo:block:test-uuid -->"));
    }

    #[test]
    fn test_insert_to_existing_content() {
        let existing = "This is existing content.\nLine 2.";
        let result = insert_block(existing, "new-block", "block content");

        assert!(result.starts_with("This is existing content."));
        assert!(result.contains("Line 2."));
        assert!(result.contains("<!-- repo:block:new-block -->"));
        assert!(result.contains("block content"));
        assert!(result.contains("<!-- /repo:block:new-block -->"));
    }

    #[test]
    fn test_insert_preserves_existing_blocks() {
        let existing = r#"<!-- repo:block:existing -->
existing content
<!-- /repo:block:existing -->"#;

        let result = insert_block(existing, "new-block", "new content");

        assert!(result.contains("<!-- repo:block:existing -->"));
        assert!(result.contains("existing content"));
        assert!(result.contains("<!-- repo:block:new-block -->"));
        assert!(result.contains("new content"));
    }

    #[test]
    fn test_update_replaces_content() {
        let content = r#"<!-- repo:block:my-uuid -->
old content here
<!-- /repo:block:my-uuid -->"#;

        let result = update_block(content, "my-uuid", "new content here").unwrap();

        assert!(result.contains("new content here"));
        assert!(!result.contains("old content here"));
        assert!(result.contains("<!-- repo:block:my-uuid -->"));
        assert!(result.contains("<!-- /repo:block:my-uuid -->"));
    }

    #[test]
    fn test_update_preserves_surrounding_content() {
        let content = r#"Header text
<!-- repo:block:middle -->
old middle
<!-- /repo:block:middle -->
Footer text"#;

        let result = update_block(content, "middle", "new middle").unwrap();

        assert!(result.contains("Header text"));
        assert!(result.contains("Footer text"));
        assert!(result.contains("new middle"));
        assert!(!result.contains("old middle"));
    }

    #[test]
    fn test_remove_block() {
        let content = r#"Header
<!-- repo:block:to-remove -->
content to remove
<!-- /repo:block:to-remove -->
Footer"#;

        let result = remove_block(content, "to-remove").unwrap();

        assert!(result.contains("Header"));
        assert!(result.contains("Footer"));
        assert!(!result.contains("content to remove"));
        assert!(!result.contains("<!-- repo:block:to-remove -->"));
    }

    #[test]
    fn test_remove_nonexistent_fails() {
        let content = "No blocks here";
        let result = remove_block(content, "nonexistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_remove_preserves_other_blocks() {
        let content = r#"<!-- repo:block:keep-1 -->
keep this 1
<!-- /repo:block:keep-1 -->

<!-- repo:block:remove-me -->
remove this
<!-- /repo:block:remove-me -->

<!-- repo:block:keep-2 -->
keep this 2
<!-- /repo:block:keep-2 -->"#;

        let result = remove_block(content, "remove-me").unwrap();

        assert!(!result.contains("remove this"));
        assert!(!result.contains("remove-me"));

        assert!(result.contains("<!-- repo:block:keep-1 -->"));
        assert!(result.contains("keep this 1"));
        assert!(result.contains("<!-- repo:block:keep-2 -->"));
        assert!(result.contains("keep this 2"));
    }

    #[test]
    fn test_upsert_inserts_when_missing() {
        let content = "Existing content";
        let result = upsert_block(content, "new-uuid", "new block").unwrap();

        assert!(result.contains("Existing content"));
        assert!(result.contains("<!-- repo:block:new-uuid -->"));
        assert!(result.contains("new block"));
    }

    #[test]
    fn test_upsert_updates_when_exists() {
        let content = r#"<!-- repo:block:existing -->
old content
<!-- /repo:block:existing -->"#;

        let result = upsert_block(content, "existing", "updated content").unwrap();

        assert!(result.contains("updated content"));
        assert!(!result.contains("old content"));
        assert_eq!(result.matches("<!-- repo:block:existing -->").count(), 1);
    }

    #[test]
    fn test_block_format_correct() {
        let result = insert_block("", "abc-123", "my content");

        assert_eq!(
            result,
            "<!-- repo:block:abc-123 -->\nmy content\n<!-- /repo:block:abc-123 -->"
        );
    }

    #[test]
    fn test_multiline_content_preserved() {
        let multiline = "Line 1\nLine 2\nLine 3";
        let result = insert_block("", "multi", multiline);

        assert!(result.contains("Line 1\nLine 2\nLine 3"));
    }

    #[test]
    fn test_update_with_multiline_content() {
        let content = r#"<!-- repo:block:test -->
old
<!-- /repo:block:test -->"#;

        let new_content = "new line 1\nnew line 2\nnew line 3";
        let result = update_block(content, "test", new_content).unwrap();

        assert!(result.contains("new line 1\nnew line 2\nnew line 3"));
    }

    #[test]
    fn insert_multiple_blocks_produces_parseable_output() {
        use crate::parser::parse_blocks;

        let mut content = String::new();
        content = insert_block(&content, "block-1", "content one");
        content = insert_block(&content, "block-2", "content two");
        content = insert_block(&content, "block-3", "content three");

        let blocks = parse_blocks(&content);
        assert_eq!(blocks.len(), 3, "All three blocks should be parseable");
        assert_eq!(blocks[0].uuid, "block-1");
        assert_eq!(blocks[0].content, "content one");
        assert_eq!(blocks[1].uuid, "block-2");
        assert_eq!(blocks[1].content, "content two");
        assert_eq!(blocks[2].uuid, "block-3");
        assert_eq!(blocks[2].content, "content three");
    }

    #[test]
    fn update_block_with_empty_content() {
        let content = "<!-- repo:block:test -->\nold content\n<!-- /repo:block:test -->";
        let result = update_block(content, "test", "").unwrap();

        assert!(result.contains("<!-- repo:block:test -->"));
        assert!(result.contains("<!-- /repo:block:test -->"));
        assert!(!result.contains("old content"));

        use crate::parser::find_block;
        let block = find_block(&result, "test").unwrap();
        assert!(
            block.content.is_empty(),
            "Updated block should have empty content"
        );
    }

    #[test]
    fn update_block_with_content_containing_marker_like_text() {
        let content = "<!-- repo:block:target -->\noriginal\n<!-- /repo:block:target -->";
        let tricky_content = "This has <!-- comments --> inside";
        let result = update_block(content, "target", tricky_content).unwrap();

        assert!(result.contains(tricky_content));
        use crate::parser::parse_blocks;
        let blocks = parse_blocks(&result);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, tricky_content);
    }

    #[test]
    fn remove_block_cleans_up_whitespace() {
        let content = r#"Header

<!-- repo:block:middle -->
to remove
<!-- /repo:block:middle -->

Footer"#;

        let result = remove_block(content, "middle").unwrap();

        assert!(
            !result.contains("\n\n\n"),
            "Remove should not leave triple newlines. Got:\n{:?}",
            result
        );
        assert!(result.contains("Header"));
        assert!(result.contains("Footer"));
    }

    #[test]
    fn remove_only_block_produces_clean_output() {
        let content = "<!-- repo:block:only -->\nthe content\n<!-- /repo:block:only -->";

        let result = remove_block(content, "only").unwrap();

        assert!(!result.contains("repo:block"));
        assert!(!result.contains("the content"));
    }

    #[test]
    fn upsert_then_remove_round_trip() {
        use crate::parser::has_block;

        let mut content = "base content".to_string();

        content = upsert_block(&content, "temp-block", "temporary").unwrap();
        assert!(has_block(&content, "temp-block"));
        assert!(content.contains("temporary"));

        content = upsert_block(&content, "temp-block", "updated").unwrap();
        assert!(has_block(&content, "temp-block"));
        assert!(content.contains("updated"));
        assert!(!content.contains("temporary"));

        content = remove_block(&content, "temp-block").unwrap();
        assert!(!has_block(&content, "temp-block"));
        assert!(content.contains("base content"));
    }

    #[test]
    fn update_specific_block_among_multiple() {
        use crate::parser::parse_blocks;

        let mut content = String::new();
        content = insert_block(&content, "first", "AAA");
        content = insert_block(&content, "second", "BBB");
        content = insert_block(&content, "third", "CCC");

        content = update_block(&content, "second", "UPDATED").unwrap();

        let blocks = parse_blocks(&content);
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].content, "AAA", "First block should be untouched");
        assert_eq!(
            blocks[1].content, "UPDATED",
            "Second block should be updated"
        );
        assert_eq!(blocks[2].content, "CCC", "Third block should be untouched");
    }

    #[test]
    fn cross_block_marker_injection_does_not_corrupt_other_blocks() {
        use crate::parser::parse_blocks;

        let mut content = insert_block("", "block-A", "content of A");
        let adversarial_b_content =
            "<!-- repo:block:block-A -->\nfake A content\n<!-- /repo:block:block-A -->";
        content = insert_block(&content, "block-B", adversarial_b_content);

        let blocks = parse_blocks(&content);
        assert_eq!(
            blocks.len(),
            3,
            "Parser finds 3 blocks (real A + real B + injected fake A), found {}",
            blocks.len()
        );

        let updated = update_block(&content, "block-A", "updated A content").unwrap();

        let blocks_after = parse_blocks(&updated);
        let real_a = blocks_after.iter().find(|b| b.uuid == "block-A").unwrap();
        assert_eq!(
            real_a.content, "updated A content",
            "Real block A should have updated content"
        );

        let real_b = blocks_after.iter().find(|b| b.uuid == "block-B").unwrap();
        assert!(
            real_b.content.contains("fake A content"),
            "Block B content should be preserved after updating A, got: {:?}",
            real_b.content
        );
    }

    #[test]
    fn remove_block_with_cross_block_injection() {
        use crate::parser::{has_block, parse_blocks};

        let mut content = insert_block("", "target", "real target content");
        let adversarial = "<!-- repo:block:target -->\nfake\n<!-- /repo:block:target -->";
        content = insert_block(&content, "container", adversarial);

        let result = remove_block(&content, "target").unwrap();

        assert!(
            has_block(&result, "container"),
            "Container block should survive removal of target"
        );

        let blocks = parse_blocks(&result);
        let container = blocks.iter().find(|b| b.uuid == "container").unwrap();
        assert!(
            container.content.contains("fake"),
            "Container block content should be preserved"
        );
    }

    #[test]
    fn insert_block_with_content_containing_newlines_at_boundaries() {
        let result = insert_block("", "boundary", "\nleading\ntrailing\n");

        use crate::parser::find_block;
        let block = find_block(&result, "boundary").unwrap();
        assert!(
            block.content.contains("leading"),
            "Content should contain 'leading'"
        );
        assert!(
            block.content.contains("trailing"),
            "Content should contain 'trailing'"
        );
    }
}
