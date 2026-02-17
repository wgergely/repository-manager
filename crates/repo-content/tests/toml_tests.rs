//! Tests for TOML handler

use repo_content::block::BlockLocation;
use repo_content::format::FormatHandler;
use repo_content::handlers::TomlHandler;
use uuid::Uuid;

#[test]
fn test_toml_find_blocks() {
    let handler = TomlHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"[package]
name = "test"

# repo:block:550e8400-e29b-41d4-a716-446655440000
[managed]
key = "value"
# /repo:block:550e8400-e29b-41d4-a716-446655440000

[other]
foo = "bar"
"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);
    assert!(blocks[0].content.contains("[managed]"));
}

#[test]
fn test_toml_format_preserving_edit() {
    let handler = TomlHandler::new();

    let source = r#"[package]
name = "test"
version = "1.0.0"

# This is a comment
[dependencies]
serde = "1.0"
"#;

    let parsed = handler.parse(source).unwrap();
    let rendered = handler.render(parsed.as_ref()).unwrap();

    // Comments should be preserved
    assert!(rendered.contains("# This is a comment"));
}

#[test]
fn test_toml_normalize() {
    let handler = TomlHandler::new();

    let source1 = r#"[a]
x = 1
[b]
y = 2"#;

    let source2 = r#"[b]
y = 2
[a]
x = 1"#;

    let norm1 = handler.normalize(source1).unwrap();
    let norm2 = handler.normalize(source2).unwrap();

    // Different order should normalize to same value
    assert_eq!(norm1, norm2);
}

#[test]
fn test_toml_insert_block() {
    let handler = TomlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "[package]\nname = \"test\"\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "[managed]\nkey = \"value\"",
            BlockLocation::End,
        )
        .unwrap();

    assert!(result.contains("# repo:block:"));
    assert!(result.contains("[managed]"));
    assert!(result.contains("# /repo:block:"));
}

#[test]
fn test_toml_update_block() {
    let handler = TomlHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"[package]
name = "test"

# repo:block:550e8400-e29b-41d4-a716-446655440000
[managed]
key = "old"
# /repo:block:550e8400-e29b-41d4-a716-446655440000
"#;

    let (result, edit) = handler
        .update_block(source, uuid, "[managed]\nkey = \"new\"")
        .unwrap();

    assert!(result.contains("key = \"new\""));
    assert!(!result.contains("key = \"old\""));
    assert_eq!(edit.kind, repo_content::EditKind::BlockUpdate { uuid });
}

#[test]
fn test_toml_remove_block() {
    let handler = TomlHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"[package]
name = "test"

# repo:block:550e8400-e29b-41d4-a716-446655440000
[managed]
key = "value"
# /repo:block:550e8400-e29b-41d4-a716-446655440000

[other]
foo = "bar"
"#;

    let (result, edit) = handler.remove_block(source, uuid).unwrap();

    assert!(!result.contains("repo:block"));
    assert!(!result.contains("[managed]"));
    assert!(result.contains("[package]"));
    assert!(result.contains("[other]"));
    assert_eq!(edit.kind, repo_content::EditKind::BlockRemove { uuid });
}

#[test]
fn test_toml_parse_error() {
    let handler = TomlHandler::new();

    let invalid_toml = "[invalid\nkey = ";
    let result = handler.parse(invalid_toml);

    assert!(result.is_err());
}

#[test]
fn test_toml_block_not_found() {
    let handler = TomlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "[package]\nname = \"test\"\n";
    let result = handler.update_block(source, uuid, "new content");

    assert!(result.is_err());
}

#[test]
fn test_toml_insert_block_after() {
    let handler = TomlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "managed = \"value\"",
            BlockLocation::After("[dependencies]".to_string()),
        )
        .unwrap();

    // Block should appear after [dependencies]
    let deps_pos = result.find("[dependencies]").unwrap();
    let block_pos = result.find("# repo:block:").unwrap();
    assert!(block_pos > deps_pos);
}

#[test]
fn test_toml_insert_block_before() {
    let handler = TomlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "[package]\nname = \"test\"\n\n[dependencies]\nserde = \"1.0\"\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "managed = \"value\"",
            BlockLocation::Before("[dependencies]".to_string()),
        )
        .unwrap();

    // Block should appear before [dependencies]
    let deps_pos = result.find("[dependencies]").unwrap();
    let block_pos = result.find("# repo:block:").unwrap();
    assert!(block_pos < deps_pos);
}

#[test]
fn test_toml_normalize_nested_tables() {
    let handler = TomlHandler::new();

    let source = r#"[package]
name = "test"
version = "1.0.0"

[package.metadata]
custom = "value"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
"#;

    let normalized = handler.normalize(source).unwrap();

    // Should be able to traverse the normalized structure
    assert!(normalized.get("package").is_some());
    assert!(normalized.get("dependencies").is_some());
}

#[test]
fn test_toml_normalize_arrays() {
    let handler = TomlHandler::new();

    let source = r#"[[bin]]
name = "first"
path = "src/bin/first.rs"

[[bin]]
name = "second"
path = "src/bin/second.rs"
"#;

    let normalized = handler.normalize(source).unwrap();

    // Arrays of tables should be preserved
    let bin_array = normalized.get("bin").unwrap();
    assert!(bin_array.is_array());
    assert_eq!(bin_array.as_array().unwrap().len(), 2);
}
