//! Tests for JSON handler

use repo_content::block::BlockLocation;
use repo_content::format::FormatHandler;
use repo_content::handlers::JsonHandler;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_json_find_blocks() {
    let handler = JsonHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"{
  "name": "test",
  "_repo_managed": {
    "550e8400-e29b-41d4-a716-446655440000": {
      "key": "value"
    }
  },
  "other": "data"
}"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);
}

#[test]
fn test_json_normalize_key_order() {
    let handler = JsonHandler::new();

    let source1 = r#"{"b": 2, "a": 1}"#;
    let source2 = r#"{"a": 1, "b": 2}"#;

    let norm1 = handler.normalize(source1).unwrap();
    let norm2 = handler.normalize(source2).unwrap();

    // Different key order should normalize to same value
    assert_eq!(norm1, norm2);
}

#[test]
fn test_json_normalize_removes_managed() {
    let handler = JsonHandler::new();

    let source = r#"{
  "data": "value",
  "_repo_managed": { "uuid": {} }
}"#;

    let normalized = handler.normalize(source).unwrap();

    // _repo_managed should be stripped
    assert!(normalized.get("_repo_managed").is_none());
    assert_eq!(normalized.get("data"), Some(&json!("value")));
}

#[test]
fn test_json_insert_block() {
    let handler = JsonHandler::new();
    let uuid = Uuid::new_v4();

    let source = r#"{"name": "test"}"#;
    let (result, _edit) = handler
        .insert_block(source, uuid, r#"{"key": "value"}"#, BlockLocation::End)
        .unwrap();

    assert!(result.contains("_repo_managed"));
    assert!(result.contains(&uuid.to_string()));
}

#[test]
fn test_json_remove_block() {
    let handler = JsonHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"{
  "name": "test",
  "_repo_managed": {
    "550e8400-e29b-41d4-a716-446655440000": {"key": "value"}
  }
}"#;

    let (result, _edit) = handler.remove_block(source, uuid).unwrap();

    // _repo_managed should be removed when empty
    assert!(!result.contains("_repo_managed"));
    assert!(result.contains("name"));
}

#[test]
fn test_json_update_block() {
    let handler = JsonHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"{
  "name": "test",
  "_repo_managed": {
    "550e8400-e29b-41d4-a716-446655440000": {"key": "old"}
  }
}"#;

    let (result, edit) = handler
        .update_block(source, uuid, r#"{"key": "new"}"#)
        .unwrap();

    assert!(result.contains("\"new\""));
    assert!(!result.contains("\"old\""));
    assert_eq!(edit.kind, repo_content::EditKind::BlockUpdate { uuid });
}

#[test]
fn test_json_parse_error() {
    let handler = JsonHandler::new();

    let invalid_json = "{invalid: json";
    let result = handler.parse(invalid_json);

    assert!(result.is_err());
}

#[test]
fn test_json_block_not_found() {
    let handler = JsonHandler::new();
    let uuid = Uuid::new_v4();

    let source = r#"{"name": "test"}"#;
    let result = handler.update_block(source, uuid, r#"{"new": "content"}"#);

    assert!(result.is_err());
}

#[test]
fn test_json_insert_string_content() {
    let handler = JsonHandler::new();
    let uuid = Uuid::new_v4();

    let source = r#"{"name": "test"}"#;
    let (result, _edit) = handler
        .insert_block(source, uuid, "plain string content", BlockLocation::End)
        .unwrap();

    assert!(result.contains("_repo_managed"));
    assert!(result.contains("plain string content"));
}

#[test]
fn test_json_normalize_nested_objects() {
    let handler = JsonHandler::new();

    let source = r#"{
  "outer": {
    "z": 3,
    "a": 1,
    "m": {
      "y": 2,
      "b": 1
    }
  }
}"#;

    let normalized = handler.normalize(source).unwrap();

    // Nested objects should also be sorted
    let outer = normalized.get("outer").unwrap();
    let m = outer.get("m").unwrap();
    assert_eq!(m.get("b"), Some(&json!(1)));
    assert_eq!(m.get("y"), Some(&json!(2)));
}

#[test]
fn test_json_normalize_arrays() {
    let handler = JsonHandler::new();

    let source = r#"{"items": [{"z": 1, "a": 2}, {"y": 3, "b": 4}]}"#;

    let normalized = handler.normalize(source).unwrap();

    // Array elements should have their keys sorted
    let items = normalized.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 2);
}

#[test]
fn test_json_parse_render_round_trip_preserves_all_values() {
    // C7: Full round-trip: parse -> render -> re-parse -> compare normalized values
    let handler = JsonHandler::new();

    let source = r#"{"name": "test", "value": 42, "nested": {"a": [1, 2, 3], "b": true}}"#;

    // Parse original
    let parsed = handler.parse(source).unwrap();
    let rendered = handler.render(parsed.as_ref()).unwrap();

    // Re-parse the rendered output
    let reparsed = handler.parse(&rendered).unwrap();
    let re_rendered = handler.render(reparsed.as_ref()).unwrap();

    // Normalize both and compare: the semantic content must be identical
    let norm1 = handler.normalize(source).unwrap();
    let norm2 = handler.normalize(&rendered).unwrap();
    assert_eq!(
        norm1, norm2,
        "Normalized values must match after parse->render round-trip"
    );

    // Double round-trip should be stable (render is idempotent)
    assert_eq!(
        rendered, re_rendered,
        "Rendering must be idempotent (render(parse(render(parse(x)))) == render(parse(x)))"
    );

    // Verify specific values survived
    let reparsed_val: serde_json::Value = serde_json::from_str(&rendered).unwrap();
    assert_eq!(reparsed_val["name"], "test");
    assert_eq!(reparsed_val["value"], 42);
    assert_eq!(reparsed_val["nested"]["a"], json!([1, 2, 3]));
    assert_eq!(reparsed_val["nested"]["b"], true);
}

#[test]
fn test_json_insert_non_object_root() {
    let handler = JsonHandler::new();
    let uuid = Uuid::new_v4();

    // JSON with array root should fail
    let source = r#"[1, 2, 3]"#;
    let result = handler.insert_block(source, uuid, r#"{"key": "value"}"#, BlockLocation::End);

    assert!(result.is_err());
}

#[test]
fn test_json_multiple_blocks() {
    let handler = JsonHandler::new();
    let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let uuid2 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    let source = r#"{
  "name": "test",
  "_repo_managed": {
    "550e8400-e29b-41d4-a716-446655440000": {"key": "value1"},
    "550e8400-e29b-41d4-a716-446655440001": {"key": "value2"}
  }
}"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 2);

    let uuids: Vec<_> = blocks.iter().map(|b| b.uuid).collect();
    assert!(uuids.contains(&uuid1));
    assert!(uuids.contains(&uuid2));
}

#[test]
fn test_json_remove_keeps_other_blocks() {
    let handler = JsonHandler::new();
    let uuid_to_remove = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let uuid_to_keep = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

    let source = r#"{
  "name": "test",
  "_repo_managed": {
    "550e8400-e29b-41d4-a716-446655440000": {"key": "value1"},
    "550e8400-e29b-41d4-a716-446655440001": {"key": "value2"}
  }
}"#;

    let (result, _edit) = handler.remove_block(source, uuid_to_remove).unwrap();

    // _repo_managed should still exist with the remaining block
    assert!(result.contains("_repo_managed"));
    assert!(result.contains(&uuid_to_keep.to_string()));
    assert!(!result.contains(&uuid_to_remove.to_string()));
}
