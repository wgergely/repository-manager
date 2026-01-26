//! Integration tests for semantic diff

use repo_content::{Document, SemanticChange};
use serde_json::json;

#[test]
fn test_diff_added_key() {
    let doc1 = Document::parse(r#"{"name": "test"}"#).unwrap();
    let doc2 = Document::parse(r#"{"name": "test", "version": "1.0"}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Added { path, .. } if path == "version"
    )));
}

#[test]
fn test_diff_removed_key() {
    let doc1 = Document::parse(r#"{"name": "test", "version": "1.0"}"#).unwrap();
    let doc2 = Document::parse(r#"{"name": "test"}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Removed { path, .. } if path == "version"
    )));
}

#[test]
fn test_diff_modified_value() {
    let doc1 = Document::parse(r#"{"version": "1.0"}"#).unwrap();
    let doc2 = Document::parse(r#"{"version": "2.0"}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Modified { path, old, new }
        if path == "version" && old == &json!("1.0") && new == &json!("2.0")
    )));
}

#[test]
fn test_diff_equivalent() {
    let doc1 = Document::parse(r#"{"a": 1, "b": 2}"#).unwrap();
    let doc2 = Document::parse(r#"{"b": 2, "a": 1}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(diff.is_equivalent);
    assert!(diff.changes.is_empty());
    assert_eq!(diff.similarity, 1.0);
}

#[test]
fn test_diff_similarity_ratio() {
    let doc1 = Document::parse(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#).unwrap();
    let doc2 = Document::parse(r#"{"a": 1, "b": 2, "c": 3, "e": 5}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.similarity > 0.5);
    assert!(diff.similarity < 1.0);
}

#[test]
fn test_diff_nested_changes() {
    let doc1 = Document::parse(r#"{"config": {"host": "localhost"}}"#).unwrap();
    let doc2 = Document::parse(r#"{"config": {"host": "example.com"}}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Modified { path, .. } if path == "config.host"
    )));
}
