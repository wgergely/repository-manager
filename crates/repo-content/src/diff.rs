//! Semantic diffing types and operations.

use serde::{Deserialize, Serialize};

/// A semantic change within a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticChange {
    /// Description of what changed.
    pub description: String,
    /// The old content (if any).
    pub old: Option<String>,
    /// The new content (if any).
    pub new: Option<String>,
}

/// A semantic diff between two versions of content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiff {
    /// List of semantic changes.
    pub changes: Vec<SemanticChange>,
}
