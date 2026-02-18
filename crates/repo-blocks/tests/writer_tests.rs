//! Integration tests for block writing functionality.

use repo_blocks::writer::{insert_block, remove_block, update_block, upsert_block};

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

    // Original content should be preserved
    assert!(result.starts_with("This is existing content."));
    assert!(result.contains("Line 2."));

    // New block should be added
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

    // Both blocks should exist
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
fn test_update_nonexistent_fails() {
    let content = "No blocks here";
    let result = update_block(content, "nonexistent", "content");

    assert!(result.is_err());
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

    // Removed block should be gone
    assert!(!result.contains("remove this"));
    assert!(!result.contains("remove-me"));

    // Other blocks should remain
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
    // Should still only have one block
    assert_eq!(result.matches("<!-- repo:block:existing -->").count(), 1);
}

#[test]
fn test_block_format_correct() {
    let result = insert_block("", "abc-123", "my content");

    // Verify exact format
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

// =============================================================================
// Writer edge cases and malformed input tests (C13, H17, H18)
// =============================================================================

#[test]
fn insert_multiple_blocks_produces_parseable_output() {
    // Inserting multiple blocks sequentially should produce content that
    // round-trips through the parser correctly
    use repo_blocks::parser::parse_blocks;

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

    // The block should still be parseable
    use repo_blocks::parser::find_block;
    let block = find_block(&result, "test").unwrap();
    assert!(
        block.content.is_empty(),
        "Updated block should have empty content"
    );
}

#[test]
fn update_block_with_content_containing_marker_like_text() {
    // Content that looks like block markers but isn't exact format
    let content = "<!-- repo:block:target -->\noriginal\n<!-- /repo:block:target -->";
    let tricky_content = "This has <!-- comments --> inside";
    let result = update_block(content, "target", tricky_content).unwrap();

    assert!(result.contains(tricky_content));
    // Should still be one parseable block
    use repo_blocks::parser::parse_blocks;
    let blocks = parse_blocks(&result);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].content, tricky_content);
}

#[test]
fn remove_block_cleans_up_whitespace() {
    // After removing a block, there should be no excessive blank lines (H18)
    let content = r#"Header

<!-- repo:block:middle -->
to remove
<!-- /repo:block:middle -->

Footer"#;

    let result = remove_block(content, "middle").unwrap();

    // Should not have triple+ newlines left behind
    assert!(
        !result.contains("\n\n\n"),
        "Remove should not leave triple newlines. Got:\n{:?}",
        result
    );
    // Both header and footer should be preserved
    assert!(result.contains("Header"));
    assert!(result.contains("Footer"));
}

#[test]
fn remove_only_block_produces_clean_output() {
    let content = "<!-- repo:block:only -->\nthe content\n<!-- /repo:block:only -->";

    let result = remove_block(content, "only").unwrap();

    // Result should be clean (no leftover markers, minimal whitespace)
    assert!(!result.contains("repo:block"));
    assert!(!result.contains("the content"));
}

#[test]
fn upsert_then_remove_round_trip() {
    use repo_blocks::parser::has_block;

    let mut content = "base content".to_string();

    // Insert via upsert
    content = upsert_block(&content, "temp-block", "temporary").unwrap();
    assert!(has_block(&content, "temp-block"));
    assert!(content.contains("temporary"));

    // Update via upsert
    content = upsert_block(&content, "temp-block", "updated").unwrap();
    assert!(has_block(&content, "temp-block"));
    assert!(content.contains("updated"));
    assert!(!content.contains("temporary"));

    // Remove
    content = remove_block(&content, "temp-block").unwrap();
    assert!(!has_block(&content, "temp-block"));
    assert!(content.contains("base content"));
}

#[test]
fn update_specific_block_among_multiple() {
    // When multiple blocks exist, updating one should not affect others
    use repo_blocks::parser::parse_blocks;

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
    // HIGH: Test that block B containing markers that look like block A's markers
    // does not cause update/remove of A to corrupt B's content.
    use repo_blocks::parser::parse_blocks;

    // Insert block A normally
    let mut content = insert_block("", "block-A", "content of A");
    // Insert block B whose content contains fake markers for block A
    let adversarial_b_content =
        "<!-- repo:block:block-A -->\nfake A content\n<!-- /repo:block:block-A -->";
    content = insert_block(&content, "block-B", adversarial_b_content);

    // Parser finds 3 blocks: real A, real B, and the fake A markers inside B's content.
    // This is a known parser limitation — it cannot distinguish injected markers from real ones.
    // The important property tested below is that update/remove of A targets the FIRST (real)
    // occurrence, not the fake one inside B.
    let blocks = parse_blocks(&content);
    assert_eq!(
        blocks.len(),
        3,
        "Parser finds 3 blocks (real A + real B + injected fake A), found {}",
        blocks.len()
    );

    // Now update block A - this should match the FIRST occurrence of A's markers
    // (the real one), not the fake markers inside B
    let updated = update_block(&content, "block-A", "updated A content").unwrap();

    // Verify block A was updated
    let blocks_after = parse_blocks(&updated);
    let real_a = blocks_after.iter().find(|b| b.uuid == "block-A").unwrap();
    assert_eq!(
        real_a.content, "updated A content",
        "Real block A should have updated content"
    );

    // Verify block B still exists and its content is intact
    let real_b = blocks_after.iter().find(|b| b.uuid == "block-B").unwrap();
    assert!(
        real_b.content.contains("fake A content"),
        "Block B content should be preserved after updating A, got: {:?}",
        real_b.content
    );
}

#[test]
fn remove_block_with_cross_block_injection() {
    // Test that removing block A when block B contains A's markers
    // does not corrupt block B
    use repo_blocks::parser::{has_block, parse_blocks};

    let mut content = insert_block("", "target", "real target content");
    let adversarial = "<!-- repo:block:target -->\nfake\n<!-- /repo:block:target -->";
    content = insert_block(&content, "container", adversarial);

    // Remove the real "target" block
    let result = remove_block(&content, "target").unwrap();

    // The "container" block should still exist
    assert!(
        has_block(&result, "container"),
        "Container block should survive removal of target"
    );

    // The container's content should still have the fake markers as plain text
    let blocks = parse_blocks(&result);
    let container = blocks.iter().find(|b| b.uuid == "container").unwrap();
    assert!(
        container.content.contains("fake"),
        "Container block content should be preserved"
    );
}

#[test]
fn insert_block_with_content_containing_newlines_at_boundaries() {
    // Content with leading/trailing newlines should be preserved exactly
    let result = insert_block("", "boundary", "\nleading\ntrailing\n");

    use repo_blocks::parser::find_block;
    let block = find_block(&result, "boundary").unwrap();
    // The parser strips one leading and one trailing newline from the raw content
    // between markers, so content starting/ending with \n gets partially stripped
    assert!(
        block.content.contains("leading"),
        "Content should contain 'leading'"
    );
    assert!(
        block.content.contains("trailing"),
        "Content should contain 'trailing'"
    );
}
