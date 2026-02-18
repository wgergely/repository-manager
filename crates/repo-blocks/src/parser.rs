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
            // We strip a single leading and trailing newline if present (but not multiple)
            let raw_content = &content[open_end..close_start];
            let trimmed = raw_content.strip_prefix('\n').unwrap_or(raw_content);
            let block_content = trimmed.strip_suffix('\n').unwrap_or(trimmed).to_string();

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

    #[test]
    fn test_no_blocks_returns_empty_vec() {
        let content = "This is some text\nwith no blocks\nat all.";
        let blocks = parse_blocks(content);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_empty_content_returns_empty_vec() {
        let blocks = parse_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_single_block_parsed_correctly() {
        let content = r#"<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
This is the block content
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(blocks[0].content, "This is the block content");
    }

    #[test]
    fn test_multiple_blocks_parsed() {
        let content = r#"Some header text
<!-- repo:block:uuid-1 -->
First block content
<!-- /repo:block:uuid-1 -->

Middle text

<!-- repo:block:uuid-2 -->
Second block content
<!-- /repo:block:uuid-2 -->

Footer text"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 2);

        assert_eq!(blocks[0].uuid, "uuid-1");
        assert_eq!(blocks[0].content, "First block content");

        assert_eq!(blocks[1].uuid, "uuid-2");
        assert_eq!(blocks[1].content, "Second block content");
    }

    #[test]
    fn test_line_positions_correct() {
        let content = r#"Line 1
Line 2
<!-- repo:block:test-uuid -->
Block content
<!-- /repo:block:test-uuid -->
Line 6"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].start_line, 3);
        assert_eq!(blocks[0].end_line, 5);
    }

    #[test]
    fn test_multiline_block_content() {
        let content = r#"<!-- repo:block:multi -->
Line 1 of content
Line 2 of content
Line 3 of content
<!-- /repo:block:multi -->"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(
            blocks[0].content,
            "Line 1 of content\nLine 2 of content\nLine 3 of content"
        );
    }

    #[test]
    fn test_find_block_returns_correct_block() {
        let content = r#"<!-- repo:block:first -->
content 1
<!-- /repo:block:first -->
<!-- repo:block:second -->
content 2
<!-- /repo:block:second -->"#;

        let block = find_block(content, "second");
        assert!(block.is_some());
        let block = block.unwrap();
        assert_eq!(block.uuid, "second");
        assert_eq!(block.content, "content 2");
    }

    #[test]
    fn test_find_block_returns_none_for_missing() {
        let content = r#"<!-- repo:block:exists -->
content
<!-- /repo:block:exists -->"#;

        let block = find_block(content, "does-not-exist");
        assert!(block.is_none());
    }

    #[test]
    fn test_has_block_true_when_exists() {
        let content = r#"<!-- repo:block:my-uuid -->
content
<!-- /repo:block:my-uuid -->"#;

        assert!(has_block(content, "my-uuid"));
    }

    #[test]
    fn test_has_block_false_when_missing() {
        let content = r#"<!-- repo:block:my-uuid -->
content
<!-- /repo:block:my-uuid -->"#;

        assert!(!has_block(content, "other-uuid"));
    }

    #[test]
    fn test_block_with_special_characters_in_content() {
        let content = r#"<!-- repo:block:special -->
Content with <html> tags & special chars "quotes" 'apostrophes'
<!-- /repo:block:special -->"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert!(
            blocks[0]
                .content
                .contains("<html> tags & special chars \"quotes\" 'apostrophes'")
        );
    }

    #[test]
    fn test_case_insensitive_uuid() {
        let content = r#"<!-- repo:block:ABC-123-DEF -->
content
<!-- /repo:block:ABC-123-DEF -->"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, "ABC-123-DEF");
    }

    #[test]
    fn unclosed_block_is_silently_skipped() {
        let content = r#"before
<!-- repo:block:unclosed -->
orphaned content
after"#;

        let blocks = parse_blocks(content);
        assert!(
            blocks.is_empty(),
            "Unclosed blocks should not be parsed. Got: {:?}",
            blocks
        );
        assert!(!has_block(content, "unclosed"));
    }

    #[test]
    fn mismatched_uuid_open_close_not_paired() {
        let content = r#"<!-- repo:block:alpha -->
content
<!-- /repo:block:beta -->"#;

        let blocks = parse_blocks(content);
        assert!(
            blocks.is_empty(),
            "Mismatched open/close UUIDs should not form a block. Got: {:?}",
            blocks
        );
        assert!(!has_block(content, "alpha"));
        assert!(!has_block(content, "beta"));
    }

    #[test]
    fn closing_marker_without_opening_is_ignored() {
        let content = r#"some text
<!-- /repo:block:orphan-close -->
more text"#;

        let blocks = parse_blocks(content);
        assert!(blocks.is_empty());
    }

    #[test]
    fn duplicate_uuid_blocks_both_parsed() {
        let content = r#"<!-- repo:block:dup -->
first occurrence
<!-- /repo:block:dup -->
middle text
<!-- repo:block:dup -->
second occurrence
<!-- /repo:block:dup -->"#;

        let blocks = parse_blocks(content);
        assert_eq!(
            blocks.len(),
            2,
            "Both blocks with duplicate UUID should be parsed"
        );
        assert_eq!(blocks[0].content, "first occurrence");
        assert_eq!(blocks[1].content, "second occurrence");

        let found = find_block(content, "dup").unwrap();
        assert_eq!(found.content, "first occurrence");
    }

    #[test]
    fn nested_blocks_with_same_uuid_uses_first_close() {
        let content = r#"<!-- repo:block:nest -->
outer start
<!-- repo:block:nest -->
inner
<!-- /repo:block:nest -->
outer end
<!-- /repo:block:nest -->"#;

        let blocks = parse_blocks(content);
        assert!(
            !blocks.is_empty(),
            "Parser should extract at least one block from nested same-UUID markers, got {}",
            blocks.len()
        );

        let first = &blocks[0];
        assert_eq!(first.uuid, "nest");
        assert!(
            first.content.contains("outer start"),
            "First block content should contain 'outer start', got: {:?}",
            first.content
        );
        assert!(
            first.content.contains("inner"),
            "First block content should contain 'inner' (text before first close), got: {:?}",
            first.content
        );
        assert!(
            !first.content.contains("outer end"),
            "First block should NOT contain 'outer end' (it's after the first close marker)"
        );
    }

    #[test]
    fn block_with_empty_content() {
        let content = "<!-- repo:block:empty -->\n<!-- /repo:block:empty -->";

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, "empty");
        assert!(
            blocks[0].content.is_empty(),
            "Empty block should have empty content, got: {:?}",
            blocks[0].content
        );
    }

    #[test]
    fn marker_inside_code_block_still_parsed() {
        let content = r#"```
<!-- repo:block:in-code -->
code content
<!-- /repo:block:in-code -->
```"#;

        let blocks = parse_blocks(content);
        assert_eq!(
            blocks.len(),
            1,
            "Parser does not distinguish code blocks from regular text"
        );
    }

    #[test]
    fn uuid_with_only_underscores_and_hyphens() {
        let content = r#"<!-- repo:block:__--__ -->
content
<!-- /repo:block:__--__ -->"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, "__--__");
    }

    #[test]
    fn partial_opening_marker_not_matched() {
        let content = r#"<!-- repo:block -->
content
<!-- /repo:block -->"#;

        let blocks = parse_blocks(content);
        assert!(
            blocks.is_empty(),
            "Markers without UUID should not be matched"
        );
    }

    #[test]
    fn uuid_with_dots_or_spaces_not_matched() {
        let content = r#"<!-- repo:block:uuid.with.dots -->
content
<!-- /repo:block:uuid.with.dots -->"#;

        let blocks = parse_blocks(content);
        assert!(
            blocks.is_empty(),
            "UUIDs with dots should not match the block regex"
        );

        let content2 = r#"<!-- repo:block:uuid with spaces -->
content
<!-- /repo:block:uuid with spaces -->"#;

        let blocks2 = parse_blocks(content2);
        assert!(
            blocks2.is_empty(),
            "UUIDs with spaces should not match the block regex"
        );
    }

    #[test]
    fn content_containing_different_blocks_closing_marker() {
        let content = r#"<!-- repo:block:block-A -->
Content of A <!-- /repo:block:block-B --> fake B close inside A
<!-- /repo:block:block-A -->

<!-- repo:block:block-B -->
Real content of B
<!-- /repo:block:block-B -->"#;

        let blocks = parse_blocks(content);
        assert_eq!(blocks.len(), 2, "Both blocks should be parsed");

        assert_eq!(blocks[0].uuid, "block-A");
        assert!(
            blocks[0].content.contains("Content of A"),
            "Block A should have its full content"
        );
        assert!(
            blocks[0].content.contains("fake B close inside A"),
            "Block A should contain the fake B marker as plain text"
        );

        assert_eq!(blocks[1].uuid, "block-B");
        assert_eq!(
            blocks[1].content, "Real content of B",
            "Block B should have its real content, not be affected by fake marker in A"
        );
    }

    #[test]
    fn very_long_content_between_markers() {
        let large_content = "x\n".repeat(10_000);
        let content = format!(
            "<!-- repo:block:large -->\n{}<!-- /repo:block:large -->",
            large_content
        );

        let blocks = parse_blocks(&content);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].uuid, "large");
        assert!(
            blocks[0].content.len() > 9000,
            "Large content should be preserved"
        );
    }
}
