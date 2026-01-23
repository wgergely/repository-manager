//! Tests for PlainText handler

use repo_content::block::BlockLocation;
use repo_content::format::FormatHandler;
use repo_content::handlers::PlainTextHandler;
use uuid::Uuid;

#[test]
fn test_plaintext_find_blocks() {
    let handler = PlainTextHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"Some text before
<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Block content here
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Some text after"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);
    assert_eq!(blocks[0].content.trim(), "Block content here");
}

#[test]
fn test_plaintext_insert_block() {
    let handler = PlainTextHandler::new();
    let uuid = Uuid::new_v4();

    let source = "Existing content\n";
    let (result, _edit) = handler
        .insert_block(source, uuid, "New block content", BlockLocation::End)
        .unwrap();

    assert!(result.contains("repo:block:"));
    assert!(result.contains("New block content"));
    assert!(result.contains("/repo:block:"));
}

#[test]
fn test_plaintext_remove_block() {
    let handler = PlainTextHandler::new();
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
fn test_plaintext_update_block() {
    let handler = PlainTextHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Original content
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->"#;

    let (result, _edit) = handler.update_block(source, uuid, "Updated content").unwrap();

    assert!(result.contains("Updated content"));
    assert!(!result.contains("Original content"));
}

#[test]
fn test_plaintext_format() {
    let handler = PlainTextHandler::new();
    assert_eq!(handler.format(), repo_content::Format::PlainText);
}

#[test]
fn test_plaintext_parse_and_render() {
    let handler = PlainTextHandler::new();
    let source = "Hello, World!";

    let parsed = handler.parse(source).unwrap();
    let rendered = handler.render(parsed.as_ref()).unwrap();

    assert_eq!(rendered, source);
}

#[test]
fn test_plaintext_normalize() {
    let handler = PlainTextHandler::new();
    let source = "  Line with trailing spaces   \n  Another line  \n";

    let normalized = handler.normalize(source).unwrap();
    let expected = serde_json::Value::String("Line with trailing spaces\n  Another line".to_string());

    assert_eq!(normalized, expected);
}

#[test]
fn test_plaintext_find_multiple_blocks() {
    let handler = PlainTextHandler::new();
    let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    let source = r#"Start
<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
First block
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Middle
<!-- repo:block:550e8400-e29b-41d4-a716-446655440001 -->
Second block
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440001 -->
End"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].uuid, uuid1);
    assert_eq!(blocks[1].uuid, uuid2);
}

#[test]
fn test_plaintext_insert_block_after_marker() {
    let handler = PlainTextHandler::new();
    let uuid = Uuid::new_v4();

    let source = "Header\n---\nContent below";
    let (result, _edit) = handler
        .insert_block(source, uuid, "Inserted", BlockLocation::After("---".to_string()))
        .unwrap();

    assert!(result.contains("Header\n---"));
    assert!(result.contains("Inserted"));
}

#[test]
fn test_plaintext_insert_block_before_marker() {
    let handler = PlainTextHandler::new();
    let uuid = Uuid::new_v4();

    let source = "Header\n---\nContent below";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "Inserted",
            BlockLocation::Before("---".to_string()),
        )
        .unwrap();

    assert!(result.contains("Inserted"));
    // The block should appear before "---"
    let block_pos = result.find("repo:block:").unwrap();
    let marker_pos = result.find("---").unwrap();
    assert!(block_pos < marker_pos);
}

#[test]
fn test_plaintext_block_not_found_error() {
    let handler = PlainTextHandler::new();
    let uuid = Uuid::new_v4();
    let source = "No blocks here";

    let result = handler.remove_block(source, uuid);
    assert!(result.is_err());
}
