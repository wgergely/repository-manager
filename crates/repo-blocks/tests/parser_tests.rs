//! Integration tests for block parsing functionality.

use repo_blocks::parser::{find_block, has_block, parse_blocks};

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
    // The block starts on line 3 (opening marker)
    assert_eq!(blocks[0].start_line, 3);
    // The block ends on line 5 (closing marker)
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
    assert!(blocks[0]
        .content
        .contains("<html> tags & special chars \"quotes\" 'apostrophes'"));
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
