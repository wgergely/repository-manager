//! Tests for YAML handler

use repo_content::format::FormatHandler;
use repo_content::handlers::YamlHandler;
use uuid::Uuid;

#[test]
fn test_yaml_find_blocks() {
    let handler = YamlHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"name: test
version: "1.0"

# repo:block:550e8400-e29b-41d4-a716-446655440000
managed:
  key: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000

other: data
"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);
    assert!(blocks[0].content.contains("managed:"));
}

#[test]
fn test_yaml_normalize_key_order() {
    let handler = YamlHandler::new();

    let source1 = "b: 2\na: 1\n";
    let source2 = "a: 1\nb: 2\n";

    let norm1 = handler.normalize(source1).unwrap();
    let norm2 = handler.normalize(source2).unwrap();

    assert_eq!(norm1, norm2);
}

#[test]
fn test_yaml_parse_error() {
    let handler = YamlHandler::new();
    let result = handler.parse("invalid: yaml: content: [unclosed");
    assert!(result.is_err());
}

#[test]
fn test_yaml_format_preserving_parse() {
    let handler = YamlHandler::new();

    let source = r#"name: test
version: "1.0"
# This is a comment
dependencies:
  - serde
  - tokio
"#;

    let parsed = handler.parse(source).unwrap();
    let rendered = handler.render(parsed.as_ref()).unwrap();

    // Core structure should be preserved
    assert!(rendered.contains("name:"));
    assert!(rendered.contains("dependencies:"));
}

#[test]
fn test_yaml_insert_block() {
    let handler = YamlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "name: test\nversion: \"1.0\"\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "managed:\n  key: value",
            repo_content::BlockLocation::End,
        )
        .unwrap();

    assert!(result.contains("# repo:block:"));
    assert!(result.contains("managed:"));
    assert!(result.contains("# /repo:block:"));
}

#[test]
fn test_yaml_update_block() {
    let handler = YamlHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"name: test

# repo:block:550e8400-e29b-41d4-a716-446655440000
managed:
  key: old
# /repo:block:550e8400-e29b-41d4-a716-446655440000
"#;

    let (result, edit) = handler
        .update_block(source, uuid, "managed:\n  key: new")
        .unwrap();

    assert!(result.contains("key: new"));
    assert!(!result.contains("key: old"));
    assert_eq!(edit.kind, repo_content::EditKind::BlockUpdate { uuid });
}

#[test]
fn test_yaml_remove_block() {
    let handler = YamlHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"name: test

# repo:block:550e8400-e29b-41d4-a716-446655440000
managed:
  key: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000

other: data
"#;

    let (result, edit) = handler.remove_block(source, uuid).unwrap();

    assert!(!result.contains("repo:block"));
    assert!(!result.contains("managed:"));
    assert!(result.contains("name: test"));
    assert!(result.contains("other: data"));
    assert_eq!(edit.kind, repo_content::EditKind::BlockRemove { uuid });
}

#[test]
fn test_yaml_block_not_found() {
    let handler = YamlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "name: test\n";
    let result = handler.update_block(source, uuid, "new content");

    assert!(result.is_err());
}

#[test]
fn test_yaml_insert_block_after() {
    let handler = YamlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "name: test\ndependencies:\n  - serde\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "managed: value",
            repo_content::BlockLocation::After("dependencies:".to_string()),
        )
        .unwrap();

    // Block should appear after dependencies:
    let deps_pos = result.find("dependencies:").unwrap();
    let block_pos = result.find("# repo:block:").unwrap();
    assert!(block_pos > deps_pos);
}

#[test]
fn test_yaml_insert_block_before() {
    let handler = YamlHandler::new();
    let uuid = Uuid::new_v4();

    let source = "name: test\ndependencies:\n  - serde\n";
    let (result, _edit) = handler
        .insert_block(
            source,
            uuid,
            "managed: value",
            repo_content::BlockLocation::Before("dependencies:".to_string()),
        )
        .unwrap();

    // Block should appear before dependencies:
    let deps_pos = result.find("dependencies:").unwrap();
    let block_pos = result.find("# repo:block:").unwrap();
    assert!(block_pos < deps_pos);
}

#[test]
fn test_yaml_normalize_nested() {
    let handler = YamlHandler::new();

    let source = r#"package:
  name: test
  version: "1.0.0"
  metadata:
    custom: value
dependencies:
  serde:
    version: "1.0"
    features:
      - derive
"#;

    let normalized = handler.normalize(source).unwrap();

    // Should be able to traverse the normalized structure
    assert!(normalized.get("package").is_some());
    assert!(normalized.get("dependencies").is_some());
}

#[test]
fn test_yaml_normalize_arrays() {
    let handler = YamlHandler::new();

    let source = r#"items:
  - name: first
    path: src/first.rs
  - name: second
    path: src/second.rs
"#;

    let normalized = handler.normalize(source).unwrap();

    // Arrays should be preserved
    let items_array = normalized.get("items").unwrap();
    assert!(items_array.is_array());
    assert_eq!(items_array.as_array().unwrap().len(), 2);
}
