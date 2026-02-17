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

// =============================================================================
// Malformed input edge cases (C8)
// =============================================================================

#[test]
fn unclosed_block_is_silently_skipped() {
    // An opening marker without a matching closing marker should NOT produce a block
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
    // Opening marker with one UUID and closing marker with a different UUID
    // should not form a block for either UUID
    let content = r#"<!-- repo:block:alpha -->
content
<!-- /repo:block:beta -->"#;

    let blocks = parse_blocks(content);
    // Neither alpha nor beta should be found as a valid block
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
    // A closing marker with no corresponding opening marker should not produce a block
    let content = r#"some text
<!-- /repo:block:orphan-close -->
more text"#;

    let blocks = parse_blocks(content);
    assert!(blocks.is_empty());
}

#[test]
fn duplicate_uuid_blocks_both_parsed() {
    // Two separate blocks with the same UUID should both be found
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

    // find_block should return the first match
    let found = find_block(content, "dup").unwrap();
    assert_eq!(found.content, "first occurrence");
}

#[test]
fn nested_blocks_with_same_uuid_uses_first_close() {
    // Nested blocks with the same UUID - the parser should match
    // the first closing marker it finds (greedy first-close behavior)
    let content = r#"<!-- repo:block:nest -->
outer start
<!-- repo:block:nest -->
inner
<!-- /repo:block:nest -->
outer end
<!-- /repo:block:nest -->"#;

    let blocks = parse_blocks(content);
    // The parser finds opening markers via regex iteration, then for each one
    // searches for the first matching close marker. With nested same-UUID,
    // the first open matches the first (inner) close.
    assert!(
        !blocks.is_empty(),
        "Parser should extract at least one block from nested same-UUID markers, got {}",
        blocks.len()
    );

    // The first block should end at the first closing marker (inner close)
    let first = &blocks[0];
    assert_eq!(first.uuid, "nest");
    // Content is from first open to first close, so includes "outer start"
    // and the second opening marker, and "inner"
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
    // "outer end" is between the first close and second close, so it should
    // NOT be in the first block's content
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
    // Block markers inside markdown code fences are still treated as markers
    // (the parser doesn't understand markdown context)
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
    // Incomplete/malformed markers should not be parsed
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
    // The regex only allows [a-zA-Z0-9_-], so dots and spaces should not match
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
    // HIGH: Block A's content contains B's closing marker.
    // Verify that parsing B is not affected by the fake marker inside A.
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
    // Ensure the parser handles large content gracefully
    let large_content = "x\n".repeat(10_000);
    let content = format!(
        "<!-- repo:block:large -->\n{}<!-- /repo:block:large -->",
        large_content
    );

    let blocks = parse_blocks(&content);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, "large");
    // Content should be the large block (minus leading/trailing newline trimming)
    assert!(blocks[0].content.len() > 9000, "Large content should be preserved");
}
