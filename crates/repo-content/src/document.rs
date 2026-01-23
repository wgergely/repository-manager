//! Document type representing parsed content.

use serde::{Deserialize, Serialize};

use crate::block::ManagedBlock;
use crate::format::Format;

/// A parsed document with semantic structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    /// The raw content of the document.
    pub content: String,
    /// The detected or specified format.
    pub format: Format,
    /// Semantic blocks within the document.
    pub blocks: Vec<ManagedBlock>,
}
