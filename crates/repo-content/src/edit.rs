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
