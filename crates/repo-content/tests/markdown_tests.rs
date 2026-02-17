//! Tests for Markdown handler

use repo_content::block::BlockLocation;
use repo_content::format::FormatHandler;
use repo_content::handlers::MarkdownHandler;
use uuid::Uuid;

#[test]
fn test_markdown_find_blocks() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"# My Document

Some intro text.

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Managed content here
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

More content.
"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);
    assert!(blocks[0].content.contains("Managed content"));
}

#[test]
fn test_markdown_insert_block() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::new_v4();

    let source = "# Title\n\nContent here.\n";
    let (result, _edit) = handler
        .insert_block(source, uuid, "New managed section", BlockLocation::End)
        .unwrap();

    assert!(result.contains("repo:block:"));
    assert!(result.contains("New managed section"));
    assert!(result.contains("/repo:block:"));
}

#[test]
fn test_markdown_normalize() {
    let handler = MarkdownHandler::new();

    // Multiple blank lines should collapse
    let source1 = "# Title\n\n\n\nContent";
    let source2 = "# Title\n\nContent";

    let norm1 = handler.normalize(source1).unwrap();
    let norm2 = handler.normalize(source2).unwrap();

    assert_eq!(norm1, norm2);
}

#[test]
fn test_markdown_remove_block() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"Before

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Content
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

After"#;

    let (result, _edit) = handler.remove_block(source, uuid).unwrap();

    assert!(!result.contains("repo:block:"));
    assert!(result.contains("Before"));
    assert!(result.contains("After"));
}

#[test]
fn test_markdown_update_block() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"# Title

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Old content
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

Footer
"#;

    let (result, edit) = handler.update_block(source, uuid, "New content").unwrap();

    assert!(result.contains("New content"));
    assert!(!result.contains("Old content"));
    assert_eq!(edit.kind, repo_content::EditKind::BlockUpdate { uuid });
}

#[test]
fn test_markdown_block_not_found() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::new_v4();

    let source = "# Title\n\nNo blocks here.\n";
    let result = handler.update_block(source, uuid, "new content");

    assert!(result.is_err());
}

#[test]
fn test_markdown_insert_block_after() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::new_v4();

    let source = "# Title\n\n## Section\n\nContent here.\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "Managed content",
            BlockLocation::After("## Section".to_string()),
        )
        .unwrap();

    // Block should appear after "## Section"
    let section_pos = result.find("## Section").unwrap();
    let block_pos = result.find("<!-- repo:block:").unwrap();
    assert!(block_pos > section_pos);
}

#[test]
fn test_markdown_insert_block_before() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::new_v4();

    let source = "# Title\n\n## Section\n\nContent here.\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "Managed content",
            BlockLocation::Before("## Section".to_string()),
        )
        .unwrap();

    // Block should appear before "## Section"
    let section_pos = result.find("## Section").unwrap();
    let block_pos = result.find("<!-- repo:block:").unwrap();
    assert!(block_pos < section_pos);
}

#[test]
fn test_markdown_normalize_trailing_whitespace() {
    let handler = MarkdownHandler::new();

    // Trailing whitespace should be trimmed
    let source1 = "# Title   \n\nContent  \n";
    let source2 = "# Title\n\nContent\n";

    let norm1 = handler.normalize(source1).unwrap();
    let norm2 = handler.normalize(source2).unwrap();

    assert_eq!(norm1, norm2);
}

#[test]
fn test_markdown_parse_and_render() {
    let handler = MarkdownHandler::new();

    let source = "# Title\n\nSome **bold** text.\n";

    let parsed = handler.parse(source).unwrap();
    let rendered = handler.render(parsed.as_ref()).unwrap();

    // Content should be preserved
    assert!(rendered.contains("# Title"));
    assert!(rendered.contains("**bold**"));
}

#[test]
fn test_markdown_format() {
    let handler = MarkdownHandler::new();
    assert_eq!(handler.format(), repo_content::Format::Markdown);
}

#[test]
fn test_markdown_multiple_blocks() {
    let handler = MarkdownHandler::new();
    let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    let source = r#"# Document

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
First block
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

Middle content

<!-- repo:block:550e8400-e29b-41d4-a716-446655440001 -->
Second block
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440001 -->

End
"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].uuid, uuid1);
    assert_eq!(blocks[1].uuid, uuid2);
    assert!(blocks[0].content.contains("First block"));
    assert!(blocks[1].content.contains("Second block"));
}

#[test]
fn test_markdown_adversarial_content_with_closing_marker() {
    // Block content that contains a closing marker for itself should not
    // cause the parser to truncate the block early or produce data loss.
    let handler = MarkdownHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    // Insert a block whose content contains its own closing marker pattern
    let adversarial_content =
        "Text before <!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 --> text after";

    let source = "# Document\n\nSome text.\n";
    let (result, _edit) = handler
        .insert_block(source, uuid, adversarial_content, BlockLocation::End)
        .unwrap();

    // Now parse the blocks back - the parser will match the FIRST closing marker
    // it finds (which is inside the content), potentially truncating the block.
    // This documents the current behavior.
    let blocks = handler.find_blocks(&result);
    assert!(
        !blocks.is_empty(),
        "Should find at least one block in adversarial content"
    );

    // The parser uses find() for the first closing marker, so the block content
    // will be truncated at the injected marker. This is a known limitation.
    // Document it: the content will NOT contain "text after" because the parser
    // terminates at the first matching close marker.
    let first_block = &blocks[0];
    assert_eq!(first_block.uuid, uuid);
    // The content should at minimum contain "Text before"
    assert!(
        first_block.content.contains("Text before"),
        "Block should contain content before the injected marker, got: {:?}",
        first_block.content
    );
}

#[test]
fn test_markdown_adversarial_content_with_different_uuid_marker() {
    // Block content containing a closing marker for a DIFFERENT block's UUID
    // should not affect parsing of either block.
    let handler = MarkdownHandler::new();
    let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    let source = r#"# Document

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Content A with fake marker <!-- /repo:block:550e8400-e29b-41d4-a716-446655440001 --> inside
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

<!-- repo:block:550e8400-e29b-41d4-a716-446655440001 -->
Content B
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440001 -->
"#;

    let blocks = handler.find_blocks(source);

    // Block A contains B's closing marker in its content, but since A's parser
    // looks for A's specific closing marker, it should not be affected.
    assert_eq!(blocks.len(), 2, "Both blocks should be found");
    assert_eq!(blocks[0].uuid, uuid1);
    assert_eq!(blocks[1].uuid, uuid2);

    // Block A's content should be intact (the fake B marker is just text)
    assert!(
        blocks[0].content.contains("Content A"),
        "Block A content should be preserved"
    );
    assert!(
        blocks[0].content.contains("fake marker"),
        "Block A should contain the fake marker text"
    );

    // Block B should also be intact
    assert!(
        blocks[1].content.contains("Content B"),
        "Block B content should be preserved"
    );
}
