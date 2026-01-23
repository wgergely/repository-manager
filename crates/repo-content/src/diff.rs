//! Semantic diff types and computation

use uuid::Uuid;

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
    BlockAdded {
        uuid: Option<Uuid>,
        content: String,
    },
    /// Block removed
    BlockRemoved {
        uuid: Option<Uuid>,
        content: String,
    },
    /// Block content changed
    BlockModified {
        uuid: Option<Uuid>,
        old: String,
        new: String,
    },
}
