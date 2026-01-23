//! Tests for SemanticDiff

use repo_content::diff::{SemanticChange, SemanticDiff};
use serde_json::json;

#[test]
fn test_semantic_diff_equivalent() {
    let diff = SemanticDiff::equivalent();
    assert!(diff.is_equivalent);
    assert!(diff.changes.is_empty());
    assert_eq!(diff.similarity, 1.0);
}

#[test]
fn test_semantic_diff_with_changes() {
    let changes = vec![
        SemanticChange::Added {
            path: "new_key".to_string(),
            value: json!("value"),
        },
        SemanticChange::Removed {
            path: "old_key".to_string(),
            value: json!(42),
        },
    ];

    let diff = SemanticDiff::with_changes(changes, 0.75);
    assert!(!diff.is_equivalent);
    assert_eq!(diff.changes.len(), 2);
    assert_eq!(diff.similarity, 0.75);
}

#[test]
fn test_semantic_diff_default() {
    let diff: SemanticDiff = Default::default();
    assert!(diff.is_equivalent);
    assert_eq!(diff.similarity, 1.0);
}

#[test]
fn test_semantic_change_modified() {
    let change = SemanticChange::Modified {
        path: "config.enabled".to_string(),
        old: json!(false),
        new: json!(true),
    };

    if let SemanticChange::Modified { path, old, new } = change {
        assert_eq!(path, "config.enabled");
        assert_eq!(old, json!(false));
        assert_eq!(new, json!(true));
    } else {
        panic!("Expected Modified variant");
    }
}

#[test]
fn test_semantic_change_block_added() {
    use uuid::Uuid;

    let uuid = Uuid::new_v4();
    let change = SemanticChange::BlockAdded {
        uuid: Some(uuid),
        content: "new block content".to_string(),
    };

    if let SemanticChange::BlockAdded { uuid: u, content } = change {
        assert_eq!(u, Some(uuid));
        assert_eq!(content, "new block content");
    } else {
        panic!("Expected BlockAdded variant");
    }
}
