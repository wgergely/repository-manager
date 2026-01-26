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
