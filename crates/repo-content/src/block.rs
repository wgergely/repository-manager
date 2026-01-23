//! Managed block types for content with semantic boundaries.

use serde::{Deserialize, Serialize};

/// Location of a block within a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockLocation {
    /// Starting line (0-indexed).
    pub start_line: usize,
    /// Ending line (exclusive, 0-indexed).
    pub end_line: usize,
    /// Starting byte offset.
    pub start_byte: usize,
    /// Ending byte offset.
    pub end_byte: usize,
}

/// A managed block of content with semantic meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedBlock {
    /// Unique identifier for this block.
    pub id: String,
    /// Location within the document.
    pub location: BlockLocation,
    /// The content of this block.
    pub content: String,
}
