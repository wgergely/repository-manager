//! Edit operations with rollback support

use std::ops::Range;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Types of edit operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditKind {
    /// Insert new content
    Insert,
    /// Delete existing content
    Delete,
    /// Replace content
    Replace,
    /// Insert a managed block
    BlockInsert { uuid: Uuid },
    /// Update a managed block
    BlockUpdate { uuid: Uuid },
    /// Remove a managed block
    BlockRemove { uuid: Uuid },
    /// Set a path value (structured data)
    PathSet { path: String },
    /// Remove a path (structured data)
    PathRemove { path: String },
}

/// Represents a reversible edit operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    /// Type of edit
    pub kind: EditKind,
    /// Byte range affected in original document
    pub span: Range<usize>,
    /// Original content (for rollback)
    pub old_content: String,
    /// New content
    pub new_content: String,
}

impl Edit {
    /// Create a new insert edit
    ///
    /// The span is a zero-width range at the insertion point.
    pub fn insert(position: usize, content: impl Into<String>) -> Self {
        Self {
            kind: EditKind::Insert,
            span: position..position,
            old_content: String::new(),
            new_content: content.into(),
        }
    }

    /// Create a new delete edit
    pub fn delete(span: Range<usize>, old_content: impl Into<String>) -> Self {
        Self {
            kind: EditKind::Delete,
            span,
            old_content: old_content.into(),
            new_content: String::new(),
        }
    }

    /// Create a new replace edit
    pub fn replace(
        span: Range<usize>,
        old_content: impl Into<String>,
        new_content: impl Into<String>,
    ) -> Self {
        Self {
            kind: EditKind::Replace,
            span,
            old_content: old_content.into(),
            new_content: new_content.into(),
        }
    }

    /// Create an edit to insert a managed block
    ///
    /// The span is a zero-width range at the insertion point.
    pub fn block_insert(uuid: Uuid, position: usize, content: impl Into<String>) -> Self {
        Self {
            kind: EditKind::BlockInsert { uuid },
            span: position..position,
            old_content: String::new(),
            new_content: content.into(),
        }
    }

    /// Create an edit to update a managed block
    pub fn block_update(
        uuid: Uuid,
        span: Range<usize>,
        old_content: impl Into<String>,
        new_content: impl Into<String>,
    ) -> Self {
        Self {
            kind: EditKind::BlockUpdate { uuid },
            span,
            old_content: old_content.into(),
            new_content: new_content.into(),
        }
    }

    /// Create an edit to remove a managed block
    pub fn block_remove(uuid: Uuid, span: Range<usize>, old_content: impl Into<String>) -> Self {
        Self {
            kind: EditKind::BlockRemove { uuid },
            span,
            old_content: old_content.into(),
            new_content: String::new(),
        }
    }

    /// Create an edit to set a path value
    pub fn path_set(
        path: impl Into<String>,
        span: Range<usize>,
        old_content: impl Into<String>,
        new_content: impl Into<String>,
    ) -> Self {
        Self {
            kind: EditKind::PathSet { path: path.into() },
            span,
            old_content: old_content.into(),
            new_content: new_content.into(),
        }
    }

    /// Create an edit to remove a path
    pub fn path_remove(
        path: impl Into<String>,
        span: Range<usize>,
        old_content: impl Into<String>,
    ) -> Self {
        Self {
            kind: EditKind::PathRemove { path: path.into() },
            span,
            old_content: old_content.into(),
            new_content: String::new(),
        }
    }

    /// Create the inverse edit for rollback
    ///
    /// The inverse of an Insert is a Delete that removes the inserted content.
    /// The inverse of a Delete is an Insert at the deletion point.
    /// The inverse of a Replace swaps old and new content.
    pub fn inverse(&self) -> Edit {
        match &self.kind {
            EditKind::Insert => {
                // Inverse of insert at position is delete of the inserted content
                // After insert, content spans from span.start to span.start + new_content.len()
                let len = self.new_content.len();
                Edit {
                    kind: EditKind::Delete,
                    span: self.span.start..self.span.start + len,
                    old_content: self.new_content.clone(),
                    new_content: String::new(),
                }
            }
            EditKind::Delete => Edit {
                kind: EditKind::Insert,
                span: self.span.start..self.span.start,
                old_content: String::new(),
                new_content: self.old_content.clone(),
            },
            EditKind::Replace => {
                // The inverse span starts at the same position but ends at
                // start + new_content.len() (the length in the modified document)
                let new_len = self.new_content.len();
                Edit {
                    kind: EditKind::Replace,
                    span: self.span.start..self.span.start + new_len,
                    old_content: self.new_content.clone(),
                    new_content: self.old_content.clone(),
                }
            }
            EditKind::BlockInsert { uuid } => {
                // Inverse of block insert is block remove of the inserted content
                let len = self.new_content.len();
                Edit {
                    kind: EditKind::BlockRemove { uuid: *uuid },
                    span: self.span.start..self.span.start + len,
                    old_content: self.new_content.clone(),
                    new_content: String::new(),
                }
            }
            EditKind::BlockUpdate { uuid } => {
                let new_len = self.new_content.len();
                Edit {
                    kind: EditKind::BlockUpdate { uuid: *uuid },
                    span: self.span.start..self.span.start + new_len,
                    old_content: self.new_content.clone(),
                    new_content: self.old_content.clone(),
                }
            }
            EditKind::BlockRemove { uuid } => Edit {
                kind: EditKind::BlockInsert { uuid: *uuid },
                span: self.span.start..self.span.start,
                old_content: String::new(),
                new_content: self.old_content.clone(),
            },
            EditKind::PathSet { path } => {
                let new_len = self.new_content.len();
                Edit {
                    kind: EditKind::PathSet { path: path.clone() },
                    span: self.span.start..self.span.start + new_len,
                    old_content: self.new_content.clone(),
                    new_content: self.old_content.clone(),
                }
            }
            EditKind::PathRemove { path } => Edit {
                kind: EditKind::PathSet { path: path.clone() },
                span: self.span.start..self.span.start,
                old_content: String::new(),
                new_content: self.old_content.clone(),
            },
        }
    }

    /// Apply this edit to a source string
    pub fn apply(&self, source: &str) -> String {
        let mut result = String::with_capacity(source.len() + self.new_content.len());
        result.push_str(&source[..self.span.start]);
        result.push_str(&self.new_content);
        result.push_str(&source[self.span.end..]);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_inverse_insert() {
        let edit = Edit {
            kind: EditKind::Insert,
            span: 10..10,
            old_content: String::new(),
            new_content: "inserted".to_string(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::Delete));
        assert_eq!(inverse.span, 10..18);
        assert_eq!(inverse.old_content, "inserted");
        assert_eq!(inverse.new_content, "");
    }

    #[test]
    fn test_edit_inverse_delete() {
        let edit = Edit {
            kind: EditKind::Delete,
            span: 10..20,
            old_content: "deleted".to_string(),
            new_content: String::new(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::Insert));
        assert_eq!(inverse.old_content, "");
        assert_eq!(inverse.new_content, "deleted");
        assert_eq!(inverse.span, 10..10);
    }

    #[test]
    fn test_edit_inverse_replace() {
        let edit = Edit {
            kind: EditKind::Replace,
            span: 10..20,
            old_content: "old".to_string(),
            new_content: "new".to_string(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::Replace));
        assert_eq!(inverse.span, 10..13);
        assert_eq!(inverse.old_content, "new");
        assert_eq!(inverse.new_content, "old");
    }

    #[test]
    fn test_edit_inverse_block_insert() {
        let uuid = Uuid::new_v4();
        let edit = Edit {
            kind: EditKind::BlockInsert { uuid },
            span: 0..0,
            old_content: String::new(),
            new_content: "block content".to_string(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::BlockRemove { uuid: u } if u == uuid));
        assert_eq!(inverse.span, 0..13);
        assert_eq!(inverse.old_content, "block content");
        assert_eq!(inverse.new_content, "");
    }

    #[test]
    fn test_edit_inverse_block_update() {
        let uuid = Uuid::new_v4();
        let edit = Edit {
            kind: EditKind::BlockUpdate { uuid },
            span: 0..50,
            old_content: "old block".to_string(),
            new_content: "new block".to_string(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::BlockUpdate { uuid: u } if u == uuid));
        assert_eq!(inverse.span, 0..9);
        assert_eq!(inverse.old_content, "new block");
        assert_eq!(inverse.new_content, "old block");
    }

    #[test]
    fn test_edit_inverse_block_remove() {
        let uuid = Uuid::new_v4();
        let edit = Edit {
            kind: EditKind::BlockRemove { uuid },
            span: 0..50,
            old_content: "removed block".to_string(),
            new_content: String::new(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::BlockInsert { uuid: u } if u == uuid));
        assert_eq!(inverse.old_content, "");
        assert_eq!(inverse.new_content, "removed block");
        assert_eq!(inverse.span, 0..0);
    }

    #[test]
    fn test_edit_inverse_path_set() {
        let edit = Edit {
            kind: EditKind::PathSet {
                path: "config.key".to_string(),
            },
            span: 10..20,
            old_content: "old_value".to_string(),
            new_content: "new_value".to_string(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::PathSet { ref path } if path == "config.key"));
        assert_eq!(inverse.span, 10..19);
        assert_eq!(inverse.old_content, "new_value");
        assert_eq!(inverse.new_content, "old_value");
    }

    #[test]
    fn test_edit_inverse_path_remove() {
        let edit = Edit {
            kind: EditKind::PathRemove {
                path: "config.key".to_string(),
            },
            span: 10..30,
            old_content: "removed_value".to_string(),
            new_content: String::new(),
        };
        let inverse = edit.inverse();
        assert!(matches!(inverse.kind, EditKind::PathSet { ref path } if path == "config.key"));
        assert_eq!(inverse.span, 10..10);
        assert_eq!(inverse.old_content, "");
        assert_eq!(inverse.new_content, "removed_value");
    }

    #[test]
    fn test_edit_apply() {
        let source = "Hello World";
        let edit = Edit::replace(6..11, "World", "Rust");
        let result = edit.apply(source);
        assert_eq!(result, "Hello Rust");
    }

    #[test]
    fn test_edit_apply_insert() {
        let source = "Hello World";
        let edit = Edit::insert(5, " Beautiful");
        let result = edit.apply(source);
        assert_eq!(result, "Hello Beautiful World");
    }

    #[test]
    fn test_edit_apply_delete() {
        let source = "Hello Beautiful World";
        let edit = Edit::delete(5..15, " Beautiful");
        let result = edit.apply(source);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn test_edit_helpers() {
        let edit = Edit::insert(5, "test");
        assert!(matches!(edit.kind, EditKind::Insert));
        assert_eq!(edit.span, 5..5);

        let edit = Edit::delete(5..10, "hello");
        assert!(matches!(edit.kind, EditKind::Delete));
        assert_eq!(edit.span, 5..10);

        let edit = Edit::replace(5..10, "hello", "world");
        assert!(matches!(edit.kind, EditKind::Replace));

        let uuid = Uuid::new_v4();
        let edit = Edit::block_insert(uuid, 10, "block content");
        assert!(matches!(edit.kind, EditKind::BlockInsert { .. }));

        let edit = Edit::block_update(uuid, 10..30, "old", "new");
        assert!(matches!(edit.kind, EditKind::BlockUpdate { .. }));

        let edit = Edit::block_remove(uuid, 10..30, "removed");
        assert!(matches!(edit.kind, EditKind::BlockRemove { .. }));

        let edit = Edit::path_set("config.key", 10..20, "old", "new");
        assert!(matches!(edit.kind, EditKind::PathSet { .. }));

        let edit = Edit::path_remove("config.key", 10..20, "removed");
        assert!(matches!(edit.kind, EditKind::PathRemove { .. }));
    }

    #[test]
    fn test_edit_roundtrip() {
        let source = "Hello World";
        let edit = Edit::replace(6..11, "World", "Rust");
        let modified = edit.apply(source);
        let inverse = edit.inverse();
        let restored = inverse.apply(&modified);
        assert_eq!(restored, source);
    }

    #[test]
    fn test_edit_insert_roundtrip() {
        let source = "Hello World";
        let edit = Edit::insert(5, " Beautiful");
        let modified = edit.apply(source);
        let inverse = edit.inverse();
        let restored = inverse.apply(&modified);
        assert_eq!(restored, source);
    }

    #[test]
    fn test_edit_delete_roundtrip() {
        let source = "Hello Beautiful World";
        let edit = Edit::delete(5..15, " Beautiful");
        let modified = edit.apply(source);
        let inverse = edit.inverse();
        let restored = inverse.apply(&modified);
        assert_eq!(restored, source);
    }
}
