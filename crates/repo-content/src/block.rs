//! Managed block types and operations

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ops::Range;
use uuid::Uuid;

/// A managed block with UUID marker
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedBlock {
    /// Unique identifier for this block
    pub uuid: Uuid,
    /// Content within the block (excluding markers)
    pub content: String,
    /// Byte range in original source (including markers)
    pub span: Range<usize>,
    checksum: String,
}

impl ManagedBlock {
    /// Create a new managed block
    pub fn new(uuid: Uuid, content: impl Into<String>, span: Range<usize>) -> Self {
        let content = content.into();
        let checksum = Self::compute_checksum(&content);
        Self {
            uuid,
            content,
            span,
            checksum,
        }
    }

    /// Get the checksum
    pub fn checksum(&self) -> &str {
        &self.checksum
    }

    /// Check if content has drifted from stored checksum
    pub fn has_drifted(&self) -> bool {
        Self::compute_checksum(&self.content) != self.checksum
    }

    /// Verify content matches a given checksum
    pub fn verify_checksum(&self, expected: &str) -> bool {
        self.checksum == expected
    }

    /// Update content and recalculate checksum
    pub fn update_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.checksum = Self::compute_checksum(&self.content);
    }

    fn compute_checksum(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

/// Where to insert a block in a document
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockLocation {
    /// Append to end of document
    #[default]
    End,
    /// After specific section/key
    After(String),
    /// Before specific section/key
    Before(String),
    /// At specific byte offset
    Offset(usize),
}
