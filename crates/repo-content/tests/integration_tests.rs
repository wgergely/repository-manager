//! Integration tests for repo-content crate
//!
//! These tests verify end-to-end behavior across multiple operations and formats.

use repo_content::block::BlockLocation;
use repo_content::format::Format;
use repo_content::Document;
use uuid::Uuid;

/// Test complete TOML document lifecycle with rollback
#[test]
fn test_full_lifecycle_toml() {
    // Start with a valid TOML document
    let original_source = r#"[package]
name = "test"
version = "1.0.0"

[dependencies]
serde = "1.0"
"#;

    let mut doc = Document::parse_as(original_source, Format::Toml).unwrap();
    assert_eq!(doc.format(), Format::Toml);

    // Step 1: Insert a managed block
    let uuid = Uuid::new_v4();
    let insert_edit = doc
        .insert_block(
            uuid,
            "[managed]\nkey = \"initial\"",
            BlockLocation::End,
        )
        .unwrap();

    // Verify block exists
    let blocks = doc.find_blocks();
    assert_eq!(blocks.len(), 1, "Should have one managed block after insert");
    assert_eq!(blocks[0].uuid, uuid);
    assert!(blocks[0].content.contains("key = \"initial\""));

    // Step 2: Update block content
    let update_edit = doc
        .update_block(uuid, "[managed]\nkey = \"updated\"")
        .unwrap();

    // Verify update
    let block = doc.get_block(uuid).expect("Block should exist after update");
    assert!(block.content.contains("key = \"updated\""));
    assert!(!block.content.contains("key = \"initial\""));

    // Step 3: Remove block
    let remove_edit = doc.remove_block(uuid).unwrap();

    // Verify removal
    assert!(doc.find_blocks().is_empty(), "No blocks should remain after removal");
    assert!(doc.get_block(uuid).is_none());

    // Step 4: Apply inverse edits to rollback
    // Rollback remove -> update -> insert (in reverse order)

    // First, rollback the remove (re-insert the block)
    let inverse_remove = remove_edit.inverse();
    let source_after_remove_rollback = inverse_remove.apply(doc.source());

    // Re-parse to continue working with Document
    let doc = Document::parse_as(&source_after_remove_rollback, Format::Toml).unwrap();
    let block = doc.get_block(uuid).expect("Block should be back after remove rollback");
    assert!(block.content.contains("key = \"updated\""), "Should have updated content after remove rollback");

    // Rollback the update (restore initial content)
    let inverse_update = update_edit.inverse();
    let source_after_update_rollback = inverse_update.apply(doc.source());

    let doc = Document::parse_as(&source_after_update_rollback, Format::Toml).unwrap();
    let block = doc.get_block(uuid).expect("Block should still exist after update rollback");
    assert!(block.content.contains("key = \"initial\""), "Should have initial content after update rollback");

    // Rollback the insert (remove the block entirely)
    let inverse_insert = insert_edit.inverse();
    let source_after_insert_rollback = inverse_insert.apply(doc.source());

    let doc = Document::parse_as(&source_after_insert_rollback, Format::Toml).unwrap();
    assert!(doc.find_blocks().is_empty(), "No blocks should remain after full rollback");

    // Verify we're back to original content structure
    assert!(doc.source().contains("[package]"));
    assert!(doc.source().contains("[dependencies]"));
    assert!(!doc.source().contains("repo:block"));
}

/// Test semantic comparison across TOML and JSON formats
#[test]
fn test_semantic_comparison_across_formats() {
    // Create a TOML document
    let toml_source = r#"[data]
name = "test"
count = 42
enabled = true

[nested]
value = "hello"
"#;

    // Create a JSON document with the same logical data
    let json_source = r#"{
  "data": {
    "name": "test",
    "count": 42,
    "enabled": true
  },
  "nested": {
    "value": "hello"
  }
}"#;

    let toml_doc = Document::parse_as(toml_source, Format::Toml).unwrap();
    let json_doc = Document::parse_as(json_source, Format::Json).unwrap();

    // Get normalized representations
    let toml_normalized = toml_doc.normalize().unwrap();
    let json_normalized = json_doc.normalize().unwrap();

    // Both should normalize to equivalent JSON structures
    assert_eq!(
        toml_normalized.get("data").unwrap().get("name"),
        json_normalized.get("data").unwrap().get("name"),
        "name field should match"
    );
    assert_eq!(
        toml_normalized.get("data").unwrap().get("count"),
        json_normalized.get("data").unwrap().get("count"),
        "count field should match"
    );
    assert_eq!(
        toml_normalized.get("data").unwrap().get("enabled"),
        json_normalized.get("data").unwrap().get("enabled"),
        "enabled field should match"
    );
    assert_eq!(
        toml_normalized.get("nested").unwrap().get("value"),
        json_normalized.get("nested").unwrap().get("value"),
        "nested.value should match"
    );

    // Full comparison
    assert_eq!(toml_normalized, json_normalized, "Normalized representations should be equal");
}

/// Test semantic comparison with key order independence
#[test]
fn test_semantic_comparison_key_order() {
    // JSON with different key orders
    let json1 = r#"{"b": 2, "a": 1, "c": 3}"#;
    let json2 = r#"{"a": 1, "c": 3, "b": 2}"#;

    let doc1 = Document::parse_as(json1, Format::Json).unwrap();
    let doc2 = Document::parse_as(json2, Format::Json).unwrap();

    // Documents should be semantically equal
    assert!(doc1.semantic_eq(&doc2), "Documents with same data but different key order should be equal");

    // Normalized forms should be identical
    let norm1 = doc1.normalize().unwrap();
    let norm2 = doc2.normalize().unwrap();
    assert_eq!(norm1, norm2);
}

/// Test checksum drift detection
#[test]
fn test_block_checksum_drift() {
    // Create a PlainText document with a managed block
    let source = "# Config\n";
    let mut doc = Document::parse_as(source, Format::PlainText).unwrap();

    let uuid = Uuid::new_v4();
    doc.insert_block(uuid, "original content", BlockLocation::End)
        .unwrap();

    // Get the block and record its checksum
    let block = doc.get_block(uuid).unwrap();
    let original_checksum = block.checksum().to_string();

    // Verify the block hasn't drifted (checksum matches content)
    assert!(!block.has_drifted(), "Fresh block should not have drifted");

    // Verify checksum verification works
    assert!(block.verify_checksum(&original_checksum), "Checksum should verify correctly");

    // Update the block with new content
    doc.update_block(uuid, "modified content").unwrap();

    // Get the updated block
    let updated_block = doc.get_block(uuid).unwrap();
    let new_checksum = updated_block.checksum().to_string();

    // Checksums should be different for different content
    assert_ne!(
        original_checksum, new_checksum,
        "Different content should produce different checksums"
    );

    // Updated block should not have drifted (checksum matches its content)
    assert!(!updated_block.has_drifted(), "Updated block should not have drifted");

    // Verify that old checksum no longer matches
    assert!(
        !updated_block.verify_checksum(&original_checksum),
        "Old checksum should not verify against new content"
    );
}

/// Test checksum consistency for same content
#[test]
fn test_block_checksum_consistency() {
    // Create two blocks with identical content
    let source1 = "# Doc 1\n";
    let source2 = "# Doc 2\n";

    let mut doc1 = Document::parse_as(source1, Format::PlainText).unwrap();
    let mut doc2 = Document::parse_as(source2, Format::PlainText).unwrap();

    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();

    doc1.insert_block(uuid1, "identical content", BlockLocation::End)
        .unwrap();
    doc2.insert_block(uuid2, "identical content", BlockLocation::End)
        .unwrap();

    let block1 = doc1.get_block(uuid1).unwrap();
    let block2 = doc2.get_block(uuid2).unwrap();

    // Same content should produce same checksum regardless of UUID or document
    assert_eq!(
        block1.checksum(),
        block2.checksum(),
        "Identical content should produce identical checksums"
    );
}

/// Test multi-block lifecycle
#[test]
fn test_multi_block_lifecycle() {
    let source = r#"[config]
version = 1
"#;

    let mut doc = Document::parse_as(source, Format::Toml).unwrap();

    // Insert multiple blocks
    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();
    let uuid3 = Uuid::new_v4();

    doc.insert_block(uuid1, "block1 = true", BlockLocation::End).unwrap();
    doc.insert_block(uuid2, "block2 = true", BlockLocation::End).unwrap();
    doc.insert_block(uuid3, "block3 = true", BlockLocation::End).unwrap();

    // Verify all blocks exist
    let blocks = doc.find_blocks();
    assert_eq!(blocks.len(), 3);

    // Update middle block
    doc.update_block(uuid2, "block2 = \"updated\"").unwrap();

    // Verify update affected only the target block
    let block2 = doc.get_block(uuid2).unwrap();
    assert!(block2.content.contains("block2 = \"updated\""));

    // Remove first and last blocks
    doc.remove_block(uuid1).unwrap();
    doc.remove_block(uuid3).unwrap();

    // Only middle block should remain
    let blocks = doc.find_blocks();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid2);
}

/// Test document diff functionality
#[test]
fn test_document_diff_equivalent() {
    let doc1 = Document::parse(r#"{"x": 1, "y": 2}"#).unwrap();
    let doc2 = Document::parse(r#"{"y": 2, "x": 1}"#).unwrap();

    let diff = doc1.diff(&doc2);
    assert!(diff.is_equivalent, "Semantically equal documents should produce equivalent diff");
}

/// Test document diff with differences
#[test]
fn test_document_diff_not_equivalent() {
    let doc1 = Document::parse(r#"{"x": 1}"#).unwrap();
    let doc2 = Document::parse(r#"{"x": 2}"#).unwrap();

    let diff = doc1.diff(&doc2);
    assert!(!diff.is_equivalent, "Different documents should not be equivalent");
}

/// Test format auto-detection
#[test]
fn test_format_auto_detection() {
    // Clear JSON (starts with {)
    let json_doc = Document::parse(r#"{"key": "value"}"#).unwrap();
    assert_eq!(json_doc.format(), Format::Json);

    // Clear TOML (key = value lines followed by section, detected via "\n[")
    // Note: Content starting with "[" is detected as JSON, so we use key=value first
    let toml_doc = Document::parse("name = \"test\"\n[section]\nkey = \"value\"").unwrap();
    assert_eq!(toml_doc.format(), Format::Toml);

    // Plain text (no clear structure)
    let plain_doc = Document::parse("Hello, World!").unwrap();
    assert_eq!(plain_doc.format(), Format::PlainText);
}

/// Test cross-format block operations
#[test]
fn test_cross_format_block_insert() {
    // Test block insertion works across different formats
    let formats_and_sources = vec![
        (Format::PlainText, "Plain text content\n"),
        (Format::Toml, "[section]\nkey = \"value\"\n"),
        (Format::Json, r#"{"key": "value"}"#),
    ];

    for (format, source) in formats_and_sources {
        let mut doc = Document::parse_as(source, format).unwrap();
        let uuid = Uuid::new_v4();

        // Content appropriate for each format
        let content = match format {
            Format::PlainText | Format::Markdown => "managed content",
            Format::Toml => "managed = true",
            Format::Json => r#"{"managed": true}"#,
            Format::Yaml => "managed: true",
        };

        doc.insert_block(uuid, content, BlockLocation::End).unwrap();

        let blocks = doc.find_blocks();
        assert_eq!(blocks.len(), 1, "Block should be inserted for {:?}", format);
        assert_eq!(blocks[0].uuid, uuid);
    }
}
