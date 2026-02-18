//! Tests for path operations

use repo_content::{Document, Format};
use serde_json::json;

#[test]
fn test_get_path_simple() {
    let source = r#"{"name": "test", "version": "1.0"}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("name"), Some(json!("test")));
    assert_eq!(doc.get_path("version"), Some(json!("1.0")));
    assert_eq!(doc.get_path("missing"), None);
}

#[test]
fn test_get_path_nested() {
    let source = r#"{"config": {"database": {"host": "localhost", "port": 5432}}}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(
        doc.get_path("config.database.host"),
        Some(json!("localhost"))
    );
    assert_eq!(doc.get_path("config.database.port"), Some(json!(5432)));
    assert_eq!(doc.get_path("config.database.missing"), None);
}

#[test]
fn test_get_path_array() {
    let source = r#"{"items": [{"name": "first"}, {"name": "second"}]}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("items[0].name"), Some(json!("first")));
    assert_eq!(doc.get_path("items[1].name"), Some(json!("second")));
    assert_eq!(doc.get_path("items[2].name"), None);
}

#[test]
fn test_get_path_toml() {
    let source = r#"[package]
name = "test"
version = "1.0"

[dependencies]
serde = "1.0"
"#;
    let doc = Document::parse_as(source, Format::Toml).unwrap();

    assert_eq!(doc.get_path("package.name"), Some(json!("test")));
    assert_eq!(doc.get_path("dependencies.serde"), Some(json!("1.0")));
}

#[test]
fn test_set_path() {
    let source = r#"{"name": "old"}"#;
    let mut doc = Document::parse(source).unwrap();

    let edit = doc.set_path("name", "new").unwrap();
    assert!(edit.old_content.contains("old"));

    assert_eq!(doc.get_path("name"), Some(json!("new")));
}

#[test]
fn test_remove_path() {
    let source = r#"{"name": "test", "version": "1.0"}"#;
    let mut doc = Document::parse(source).unwrap();

    let edit = doc.remove_path("version").unwrap();
    assert!(edit.old_content.contains("version"));

    assert_eq!(doc.get_path("version"), None);
    assert_eq!(doc.get_path("name"), Some(json!("test")));
}

// =============================================================================
// Unicode and special character path tests (C11)
// =============================================================================

#[test]
fn test_get_path_with_unicode_keys() {
    let source = r#"{"名前": "太郎", "配置": {"ホスト": "localhost"}}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("名前"), Some(json!("太郎")));
    assert_eq!(doc.get_path("配置.ホスト"), Some(json!("localhost")));
}

#[test]
fn test_get_path_with_unicode_values() {
    let source = r#"{"greeting": "こんにちは", "emoji": "🎉"}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("greeting"), Some(json!("こんにちは")));
    assert_eq!(doc.get_path("emoji"), Some(json!("🎉")));
}

#[test]
fn test_set_path_with_unicode() {
    let source = r#"{"name": "old"}"#;
    let mut doc = Document::parse(source).unwrap();

    doc.set_path("name", "新しい値").unwrap();
    assert_eq!(doc.get_path("name"), Some(json!("新しい値")));
}

#[test]
fn test_get_path_empty_string_key() {
    // JSON allows empty string as a key
    let source = r#"{"": "empty key value", "normal": "data"}"#;
    let doc = Document::parse(source).unwrap();

    // Empty path should not panic and should return a deterministic result.
    // Current behavior: returns the entire document root (the whole JSON object).
    let result = doc.get_path("");
    assert!(
        result.is_some(),
        "Empty path should return Some (the document root), got None",
    );

    assert_eq!(doc.get_path("normal"), Some(json!("data")));
}

#[test]
fn test_get_path_keys_with_special_chars_in_values() {
    // Keys are normal but values contain special characters
    let source = r#"{"path": "C:\\Users\\test", "url": "https://example.com?a=1&b=2"}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("path"), Some(json!("C:\\Users\\test")));
    assert_eq!(
        doc.get_path("url"),
        Some(json!("https://example.com?a=1&b=2"))
    );
}

#[test]
fn test_get_path_numeric_string_key() {
    // Keys that look like numbers
    let source = r#"{"0": "zero", "1": "one", "items": ["a", "b"]}"#;
    let doc = Document::parse(source).unwrap();

    // "0" as a key (not an array index)
    assert_eq!(doc.get_path("0"), Some(json!("zero")));
    assert_eq!(doc.get_path("1"), Some(json!("one")));
    // Array index notation
    assert_eq!(doc.get_path("items[0]"), Some(json!("a")));
}

#[test]
fn test_get_path_deeply_nested() {
    let source = r#"{"a": {"b": {"c": {"d": {"e": "deep"}}}}}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("a.b.c.d.e"), Some(json!("deep")));
    assert_eq!(doc.get_path("a.b.c.d"), Some(json!({"e": "deep"})));
    assert_eq!(doc.get_path("a.b.c.d.e.f"), None);
}

#[test]
fn test_get_path_with_hyphenated_keys() {
    let source = r#"{"my-key": "value", "nested": {"sub-key": "data"}}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("my-key"), Some(json!("value")));
    assert_eq!(doc.get_path("nested.sub-key"), Some(json!("data")));
}

#[test]
fn test_path_operations_with_boolean_and_null_values() {
    let source = r#"{"flag": true, "nothing": null, "count": 0}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("flag"), Some(json!(true)));
    assert_eq!(doc.get_path("nothing"), Some(json!(null)));
    assert_eq!(doc.get_path("count"), Some(json!(0)));
}

#[test]
fn test_set_path_rejects_nonexistent_key() {
    // set_path requires the path to already exist; it does NOT create new keys.
    let source = r#"{"existing": "value"}"#;
    let mut doc = Document::parse(source).unwrap();

    let result = doc.set_path("new_key", "new_value");
    assert!(
        result.is_err(),
        "set_path should return error for nonexistent path, got Ok"
    );

    // Original data should be intact
    assert_eq!(doc.get_path("existing"), Some(json!("value")));
}

#[test]
fn test_remove_path_nonexistent_key_returns_error() {
    // remove_path returns PathNotFound for keys that don't exist
    let source = r#"{"name": "test"}"#;
    let mut doc = Document::parse(source).unwrap();

    let result = doc.remove_path("nonexistent");
    assert!(
        result.is_err(),
        "remove_path should return error for nonexistent key, got Ok"
    );

    // Original data should be intact
    assert_eq!(doc.get_path("name"), Some(json!("test")));
}

#[test]
fn test_path_with_array_of_arrays() {
    let source = r#"{"matrix": [[1, 2], [3, 4]]}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("matrix[0]"), Some(json!([1, 2])));
    assert_eq!(doc.get_path("matrix[1]"), Some(json!([3, 4])));
    assert_eq!(doc.get_path("matrix[0][0]"), Some(json!(1)));
    assert_eq!(doc.get_path("matrix[1][1]"), Some(json!(4)));
}

#[test]
fn test_toml_path_with_dotted_table_keys() {
    let source = r#"
[package]
name = "test"

[dependencies]
serde = { version = "1.0", features = ["derive"] }

[build-dependencies]
cc = "1.0"
"#;
    let doc = Document::parse_as(source, Format::Toml).unwrap();

    assert_eq!(doc.get_path("package.name"), Some(json!("test")));
    assert_eq!(
        doc.get_path("dependencies.serde.version"),
        Some(json!("1.0"))
    );
    assert_eq!(doc.get_path("build-dependencies.cc"), Some(json!("1.0")));
}
