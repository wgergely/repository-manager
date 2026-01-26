//! Tests for Document

use repo_content::block::BlockLocation;
use repo_content::format::Format;
use repo_content::Document;
use uuid::Uuid;

#[test]
fn test_document_parse_auto_detect() {
    // TOML with key = value at root level (not starting with [section])
    let toml = "name = \"test\"\n[package]";
    let doc = Document::parse(toml).unwrap();
    assert_eq!(doc.format(), Format::Toml);

    let json = r#"{"key": "value"}"#;
    let doc = Document::parse(json).unwrap();
    assert_eq!(doc.format(), Format::Json);

    let plain = "Hello world";
    let doc = Document::parse(plain).unwrap();
    assert_eq!(doc.format(), Format::PlainText);
}

#[test]
fn test_document_parse_explicit() {
    let source = "key = value";
    let doc = Document::parse_as(source, Format::PlainText).unwrap();
    assert_eq!(doc.format(), Format::PlainText);
}

#[test]
fn test_document_block_lifecycle() {
    let source = "# Config file\n";
    let mut doc = Document::parse_as(source, Format::PlainText).unwrap();

    // Insert block
    let uuid = Uuid::new_v4();
    let _edit = doc
        .insert_block(uuid, "managed content", BlockLocation::End)
        .unwrap();

    // Find block
    let blocks = doc.find_blocks();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);

    // Update block
    let _edit = doc.update_block(uuid, "updated content").unwrap();

    let blocks = doc.find_blocks();
    assert!(blocks[0].content.contains("updated"));

    // Remove block
    let _edit = doc.remove_block(uuid).unwrap();
    let blocks = doc.find_blocks();
    assert!(blocks.is_empty());
}

#[test]
fn test_document_semantic_eq() {
    let json1 = r#"{"a": 1, "b": 2}"#;
    let json2 = r#"{"b": 2, "a": 1}"#;

    let doc1 = Document::parse(json1).unwrap();
    let doc2 = Document::parse(json2).unwrap();

    assert!(doc1.semantic_eq(&doc2));
}

#[test]
fn test_document_render() {
    // Use explicit format to avoid auto-detection ambiguity
    let source = "[package]\nname = \"test\"\n";
    let doc = Document::parse_as(source, Format::Toml).unwrap();
    let rendered = doc.render();

    // Should preserve structure
    assert!(rendered.contains("[package]"));
    assert!(rendered.contains("name"));
}

#[test]
fn test_document_get_block() {
    let source = "# Config file\n";
    let mut doc = Document::parse_as(source, Format::PlainText).unwrap();

    let uuid = Uuid::new_v4();
    doc.insert_block(uuid, "content", BlockLocation::End)
        .unwrap();

    // Get existing block
    let block = doc.get_block(uuid);
    assert!(block.is_some());
    assert_eq!(block.unwrap().uuid, uuid);

    // Get non-existing block
    let other_uuid = Uuid::new_v4();
    assert!(doc.get_block(other_uuid).is_none());
}

#[test]
fn test_document_diff() {
    let doc1 = Document::parse(r#"{"a": 1}"#).unwrap();
    let doc2 = Document::parse(r#"{"a": 1}"#).unwrap();

    let diff = doc1.diff(&doc2);
    assert!(diff.is_equivalent);

    let doc3 = Document::parse(r#"{"a": 2}"#).unwrap();
    let diff = doc1.diff(&doc3);
    assert!(!diff.is_equivalent);
}
