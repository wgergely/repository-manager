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

    assert_eq!(doc.get_path("config.database.host"), Some(json!("localhost")));
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
