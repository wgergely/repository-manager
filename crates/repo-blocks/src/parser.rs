//! Block parsing functionality for managed blocks.
//!
//! Parses UUID-tagged blocks in text files with the format:
//! ```text
//! <!-- repo:block:UUID -->
//! content here
//! <!-- /repo:block:UUID -->
//! ```

use regex::Regex;
use std::sync::LazyLock;

/// A parsed block with its UUID, content, and position information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    /// The UUID identifying this block.
    pub uuid: String,
    /// The content between the block markers (excluding the markers themselves).
    pub content: String,
    /// The 1-based line number where the opening marker starts.
    pub start_line: usize,
    /// The 1-based line number where the closing marker ends.
    pub end_line: usize,
}

/// Regex for matching opening block markers.
/// Supports alphanumeric IDs with hyphens and underscores.
static OPEN_MARKER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<!-- repo:block:([a-zA-Z0-9_-]+) -->").expect("Invalid open marker regex")
});

/// Parses all blocks from the given content.
///
/// # Arguments
/// * `content` - The text content to parse for blocks
///
/// # Returns
/// A vector of all parsed blocks, in order of appearance.
///
/// # Example
/// ```
/// use repo_blocks::parser::parse_blocks;
///
/// let content = r#"Some text
/// <!-- repo:block:abc-123 -->
/// block content
/// <!-- /repo:block:abc-123 -->
/// More text"#;
///
/// let blocks = parse_blocks(content);
/// assert_eq!(blocks.len(), 1);
/// assert_eq!(blocks[0].uuid, "abc-123");
/// ```
pub fn parse_blocks(content: &str) -> Vec<Block> {
    let mut blocks = Vec::new();

    for open_caps in OPEN_MARKER_REGEX.captures_iter(content) {
        let uuid = open_caps.get(1).unwrap().as_str();
        let open_match = open_caps.get(0).unwrap();
        let open_end = open_match.end();

        // Build the closing marker pattern for this specific UUID
        let close_marker = format!("<!-- /repo:block:{} -->", uuid);

        // Find the closing marker after the opening marker
        if let Some(close_pos) = content[open_end..].find(&close_marker) {
            let close_start = open_end + close_pos;
            let close_end = close_start + close_marker.len();

            // Extract content between markers
            // The content is everything between the opening marker end and the closing marker start
            // We strip the leading newline (if present) and trailing newline (if present)
            let raw_content = &content[open_end..close_start];
            let block_content = raw_content
                .strip_prefix('\n')
                .unwrap_or(raw_content)
                .strip_suffix('\n')
                .unwrap_or(raw_content.strip_prefix('\n').unwrap_or(raw_content))
                .to_string();

            // Calculate line numbers
            let start_line = content[..open_match.start()].lines().count() + 1;
            let end_line = content[..close_end].lines().count();

            blocks.push(Block {
                uuid: uuid.to_string(),
                content: block_content,
                start_line,
                end_line,
            });
        }
    }

    blocks
}

/// Finds a specific block by its UUID.
///
/// # Arguments
/// * `content` - The text content to search
/// * `uuid` - The UUID of the block to find
///
/// # Returns
/// The block if found, or None if no block with that UUID exists.
///
/// # Example
/// ```
/// use repo_blocks::parser::find_block;
///
/// let content = r#"<!-- repo:block:abc-123 -->
/// content
/// <!-- /repo:block:abc-123 -->"#;
///
/// let block = find_block(content, "abc-123");
/// assert!(block.is_some());
/// assert_eq!(block.unwrap().content, "content");
/// ```
pub fn find_block(content: &str, uuid: &str) -> Option<Block> {
    parse_blocks(content)
        .into_iter()
        .find(|block| block.uuid == uuid)
}

/// Checks if a block with the given UUID exists in the content.
///
/// # Arguments
/// * `content` - The text content to search
/// * `uuid` - The UUID to check for
///
/// # Returns
/// `true` if a block with the UUID exists, `false` otherwise.
///
/// # Example
/// ```
/// use repo_blocks::parser::has_block;
///
/// let content = r#"<!-- repo:block:abc-123 -->
/// content
/// <!-- /repo:block:abc-123 -->"#;
///
/// assert!(has_block(content, "abc-123"));
/// assert!(!has_block(content, "nonexistent"));
/// ```
pub fn has_block(content: &str, uuid: &str) -> bool {
    find_block(content, uuid).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blocks_empty() {
        let content = "No blocks here";
        let blocks = parse_blocks(content);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_parse_single_block() {
        let content = r#"<!-- repo:block:abc-123 -->
hello world
<!-- /repo:block:abc-123 -->"#;
        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, "abc-123");
        assert_eq!(blocks[0].content, "hello world");
    }

    #[test]
    fn test_find_block_exists() {
        let content = r#"<!-- repo:block:abc-123 -->
content
<!-- /repo:block:abc-123 -->"#;
        let block = find_block(content, "abc-123");
        assert!(block.is_some());
        assert_eq!(block.unwrap().content, "content");
    }

    #[test]
    fn test_find_block_not_exists() {
        let content = r#"<!-- repo:block:abc-123 -->
content
<!-- /repo:block:abc-123 -->"#;
        let block = find_block(content, "xyz-789");
        assert!(block.is_none());
    }

    #[test]
    fn test_has_block() {
        let content = r#"<!-- repo:block:abc-123 -->
content
<!-- /repo:block:abc-123 -->"#;
        assert!(has_block(content, "abc-123"));
        assert!(!has_block(content, "xyz-789"));
    }
}
