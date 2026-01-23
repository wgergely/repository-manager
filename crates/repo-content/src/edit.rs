//! Edit types for content modification.

use serde::{Deserialize, Serialize};

use crate::block::BlockLocation;

/// The kind of edit operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditKind {
    /// Insert new content.
    Insert,
    /// Replace existing content.
    Replace,
    /// Delete content.
    Delete,
}

/// An edit operation on document content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    /// The kind of edit.
    pub kind: EditKind,
    /// Location of the edit.
    pub location: BlockLocation,
    /// New content (for Insert and Replace).
    pub new_content: Option<String>,
}
