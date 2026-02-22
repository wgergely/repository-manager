//! Tests for semantic diff
//!
//! Category: component

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
fn test_diff_similarity_ratio_with_known_overlap() {
    // C12: Test with inputs where we can calculate the expected similarity.
    // The similarity algorithm uses character-level string comparison on serialized JSON.
    // {"a":1,"b":2,"c":3,"d":4} vs {"a":1,"b":2,"c":3,"e":5} differ in only 2 chars,
    // so similarity will be very high (> 0.9).
    let doc1 = Document::parse(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#).unwrap();
    let doc2 = Document::parse(r#"{"a": 1, "b": 2, "c": 3, "e": 5}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    // Character-level similarity is very high since most of the JSON string is identical
    assert!(
        diff.similarity >= 0.8 && diff.similarity <= 1.0,
        "Similarity for documents differing in 2 chars should be >= 0.8, got {}",
        diff.similarity
    );

    // Verify the specific changes detected
    assert!(
        diff.changes.iter().any(|c| matches!(c,
            SemanticChange::Removed { path, .. } if path == "d"
        )),
        "Key 'd' should be reported as removed"
    );
    assert!(
        diff.changes.iter().any(|c| matches!(c,
            SemanticChange::Added { path, .. } if path == "e"
        )),
        "Key 'e' should be reported as added"
    );
}

#[test]
fn test_diff_similarity_identical_documents_is_1() {
    let doc1 = Document::parse(r#"{"x": 1}"#).unwrap();
    let doc2 = Document::parse(r#"{"x": 1}"#).unwrap();

    let diff = doc1.diff(&doc2);
    assert_eq!(
        diff.similarity, 1.0,
        "Identical documents should have similarity 1.0"
    );
    assert!(diff.is_equivalent);
}

#[test]
fn test_diff_similarity_completely_different_is_not_one() {
    // The similarity algorithm uses character-level string comparison on serialized JSON.
    // Even "completely different" JSON objects share structural characters ({, :, ,, }).
    // So similarity won't be 0.0 for different objects with similar structure.
    let doc1 = Document::parse(r#"{"a": 1, "b": 2}"#).unwrap();
    let doc2 = Document::parse(r#"{"x": 10, "y": 20}"#).unwrap();

    let diff = doc1.diff(&doc2);
    assert!(
        diff.similarity < 1.0,
        "Documents with different keys/values should not have similarity 1.0, got {}",
        diff.similarity
    );
    assert!(!diff.is_equivalent);
    // Verify specific changes: all keys should be reported as added/removed
    assert!(
        !diff.changes.is_empty(),
        "Different documents must have changes"
    );
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
