//! Semantic diff types and computation

use serde_json::Value;
use similar::TextDiff;
use uuid::Uuid;

/// Maximum recursion depth for diff operations
const MAX_DIFF_DEPTH: usize = 128;

/// Result of comparing two documents semantically
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticDiff {
    /// Are the documents semantically equivalent?
    pub is_equivalent: bool,
    /// List of semantic changes
    pub changes: Vec<SemanticChange>,
    /// Similarity ratio (0.0 to 1.0)
    pub similarity: f64,
}

impl SemanticDiff {
    /// Create a diff indicating documents are equivalent
    pub fn equivalent() -> Self {
        Self {
            is_equivalent: true,
            changes: Vec::new(),
            similarity: 1.0,
        }
    }

    /// Create a diff with changes
    pub fn with_changes(changes: Vec<SemanticChange>, similarity: f64) -> Self {
        Self {
            is_equivalent: changes.is_empty(),
            changes,
            similarity,
        }
    }

    /// Compute a semantic diff between two JSON values
    ///
    /// This recursively compares two JSON values and tracks all changes
    /// with their paths (e.g., "config.host" for nested keys).
    pub fn compute(old: &Value, new: &Value) -> Self {
        let mut changes = Vec::new();
        diff_values(old, new, String::new(), &mut changes);

        let similarity = compute_similarity(old, new);

        Self {
            is_equivalent: changes.is_empty(),
            changes,
            similarity,
        }
    }

    /// Compute a semantic diff between two text strings
    ///
    /// Uses the `similar` crate's TextDiff for line-by-line comparison.
    pub fn compute_text(old: &str, new: &str) -> Self {
        if old == new {
            return Self::equivalent();
        }

        let text_diff = TextDiff::from_lines(old, new);
        let similarity = text_diff.ratio() as f64;

        let mut changes = Vec::new();

        for change in text_diff.iter_all_changes() {
            match change.tag() {
                similar::ChangeTag::Delete => {
                    changes.push(SemanticChange::BlockRemoved {
                        uuid: None,
                        content: change.value().to_string(),
                    });
                }
                similar::ChangeTag::Insert => {
                    changes.push(SemanticChange::BlockAdded {
                        uuid: None,
                        content: change.value().to_string(),
                    });
                }
                similar::ChangeTag::Equal => {
                    // No change needed for equal lines
                }
            }
        }

        Self {
            is_equivalent: changes.is_empty(),
            changes,
            similarity,
        }
    }
}

impl Default for SemanticDiff {
    fn default() -> Self {
        Self::equivalent()
    }
}

/// A semantic change between documents
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticChange {
    /// Key/path added
    Added {
        path: String,
        value: serde_json::Value,
    },
    /// Key/path removed
    Removed {
        path: String,
        value: serde_json::Value,
    },
    /// Value changed at path
    Modified {
        path: String,
        old: serde_json::Value,
        new: serde_json::Value,
    },
    /// Block added (for Markdown/text)
    BlockAdded { uuid: Option<Uuid>, content: String },
    /// Block removed
    BlockRemoved { uuid: Option<Uuid>, content: String },
    /// Block content changed
    BlockModified {
        uuid: Option<Uuid>,
        old: String,
        new: String,
    },
}

/// Recursively diff two JSON values, collecting changes with path tracking
fn diff_values(old: &Value, new: &Value, path: String, changes: &mut Vec<SemanticChange>) {
    diff_values_with_depth(old, new, path, changes, 0);
}

/// Internal recursive diff with depth tracking
fn diff_values_with_depth(
    old: &Value,
    new: &Value,
    path: String,
    changes: &mut Vec<SemanticChange>,
    depth: usize,
) {
    // Depth limit: treat deeply nested differences as a single modification
    if depth > MAX_DIFF_DEPTH {
        if old != new {
            changes.push(SemanticChange::Modified {
                path,
                old: old.clone(),
                new: new.clone(),
            });
        }
        return;
    }

    match (old, new) {
        // Both are objects - compare keys
        (Value::Object(old_obj), Value::Object(new_obj)) => {
            // Check for removed and modified keys
            for (key, old_value) in old_obj {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };

                match new_obj.get(key) {
                    Some(new_value) => {
                        // Key exists in both - recurse
                        diff_values_with_depth(
                            old_value,
                            new_value,
                            child_path,
                            changes,
                            depth + 1,
                        );
                    }
                    None => {
                        // Key removed
                        changes.push(SemanticChange::Removed {
                            path: child_path,
                            value: old_value.clone(),
                        });
                    }
                }
            }

            // Check for added keys
            for (key, new_value) in new_obj {
                if !old_obj.contains_key(key) {
                    let child_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    changes.push(SemanticChange::Added {
                        path: child_path,
                        value: new_value.clone(),
                    });
                }
            }
        }

        // Both are arrays - compare element by element
        (Value::Array(old_arr), Value::Array(new_arr)) => {
            let max_len = old_arr.len().max(new_arr.len());
            for i in 0..max_len {
                let child_path = if path.is_empty() {
                    format!("[{}]", i)
                } else {
                    format!("{}[{}]", path, i)
                };

                match (old_arr.get(i), new_arr.get(i)) {
                    (Some(old_val), Some(new_val)) => {
                        diff_values_with_depth(old_val, new_val, child_path, changes, depth + 1);
                    }
                    (Some(old_val), None) => {
                        changes.push(SemanticChange::Removed {
                            path: child_path,
                            value: old_val.clone(),
                        });
                    }
                    (None, Some(new_val)) => {
                        changes.push(SemanticChange::Added {
                            path: child_path,
                            value: new_val.clone(),
                        });
                    }
                    (None, None) => unreachable!(),
                }
            }
        }

        // Different types or scalar values - compare directly
        _ => {
            if old != new {
                changes.push(SemanticChange::Modified {
                    path,
                    old: old.clone(),
                    new: new.clone(),
                });
            }
        }
    }
}

/// Compute similarity ratio between two JSON values
///
/// This uses a simple approach: serialize both to strings and use
/// similar::TextDiff::ratio() for a quick similarity estimate.
fn compute_similarity(old: &Value, new: &Value) -> f64 {
    if old == new {
        return 1.0;
    }

    // Serialize both values to canonical JSON strings for comparison
    let old_str = serde_json::to_string(old).unwrap_or_default();
    let new_str = serde_json::to_string(new).unwrap_or_default();

    let diff = TextDiff::from_chars(&old_str, &new_str);
    diff.ratio() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compute_empty_objects_equivalent() {
        let old = json!({});
        let new = json!({});
        let diff = SemanticDiff::compute(&old, &new);
        assert!(diff.is_equivalent);
        assert!(diff.changes.is_empty());
        assert_eq!(diff.similarity, 1.0);
    }

    #[test]
    fn test_compute_added_key() {
        let old = json!({"a": 1});
        let new = json!({"a": 1, "b": 2});
        let diff = SemanticDiff::compute(&old, &new);

        assert!(!diff.is_equivalent);
        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(
            &diff.changes[0],
            SemanticChange::Added { path, value } if path == "b" && value == &json!(2)
        ));
    }

    #[test]
    fn test_compute_removed_key() {
        let old = json!({"a": 1, "b": 2});
        let new = json!({"a": 1});
        let diff = SemanticDiff::compute(&old, &new);

        assert!(!diff.is_equivalent);
        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(
            &diff.changes[0],
            SemanticChange::Removed { path, value } if path == "b" && value == &json!(2)
        ));
    }

    #[test]
    fn test_compute_modified_value() {
        let old = json!({"a": 1});
        let new = json!({"a": 2});
        let diff = SemanticDiff::compute(&old, &new);

        assert!(!diff.is_equivalent);
        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(
            &diff.changes[0],
            SemanticChange::Modified { path, old, new } if path == "a" && old == &json!(1) && new == &json!(2)
        ));
    }

    #[test]
    fn test_compute_nested_path() {
        let old = json!({"config": {"host": "localhost"}});
        let new = json!({"config": {"host": "example.com"}});
        let diff = SemanticDiff::compute(&old, &new);

        assert!(!diff.is_equivalent);
        assert_eq!(diff.changes.len(), 1);
        assert!(matches!(
            &diff.changes[0],
            SemanticChange::Modified { path, .. } if path == "config.host"
        ));
    }

    #[test]
    fn test_compute_array_changes() {
        let old = json!({"items": [1, 2, 3]});
        let new = json!({"items": [1, 2, 4]});
        let diff = SemanticDiff::compute(&old, &new);

        assert!(!diff.is_equivalent);
        assert!(diff.changes.iter().any(|c| matches!(c,
            SemanticChange::Modified { path, .. } if path == "items[2]"
        )));
    }

    #[test]
    fn test_compute_text_equivalent() {
        let diff = SemanticDiff::compute_text("hello\nworld", "hello\nworld");
        assert!(diff.is_equivalent);
        assert!(diff.changes.is_empty());
        assert_eq!(diff.similarity, 1.0);
    }

    #[test]
    fn test_compute_text_added_line() {
        let diff = SemanticDiff::compute_text("line1\n", "line1\nline2\n");
        assert!(!diff.is_equivalent);
        assert!(diff.changes.iter().any(|c| matches!(c,
            SemanticChange::BlockAdded { content, .. } if content.contains("line2")
        )));
    }

    #[test]
    fn test_compute_text_removed_line() {
        let diff = SemanticDiff::compute_text("line1\nline2\n", "line1\n");
        assert!(!diff.is_equivalent);
        assert!(diff.changes.iter().any(|c| matches!(c,
            SemanticChange::BlockRemoved { content, .. } if content.contains("line2")
        )));
    }

    #[test]
    fn test_compute_handles_deep_nesting() {
        // Create deeply nested JSON (deeper than stack can handle without limit)
        fn create_nested(depth: usize) -> Value {
            let mut current = json!({"leaf": "value"});
            for _ in 0..depth {
                current = json!({"nested": current});
            }
            current
        }

        // 200 levels should work fine with depth limiting
        let old = create_nested(200);
        let new = create_nested(200);

        // Should not stack overflow
        let diff = SemanticDiff::compute(&old, &new);
        assert!(diff.is_equivalent);
    }

    #[test]
    fn test_compute_truncates_at_max_depth() {
        fn create_nested(depth: usize, leaf_value: &str) -> Value {
            let mut current = json!({"leaf": leaf_value});
            for _ in 0..depth {
                current = json!({"nested": current});
            }
            current
        }

        // Create structures deeper than MAX_DIFF_DEPTH
        let old = create_nested(150, "old");
        let new = create_nested(150, "new");

        // Should detect difference without stack overflow
        let diff = SemanticDiff::compute(&old, &new);
        assert!(!diff.is_equivalent);
    }

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
}
