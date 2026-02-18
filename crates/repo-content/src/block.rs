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

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn checksum_matches_independently_computed_sha256() {
        let uuid = Uuid::new_v4();
        let content = "test content for checksum verification";
        let block = ManagedBlock::new(uuid, content, 0..40);

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let expected = format!("{:x}", hasher.finalize());

        assert_eq!(block.checksum(), expected);
    }

    #[test]
    fn checksum_changes_when_content_changes() {
        let uuid = Uuid::new_v4();
        let block1 = ManagedBlock::new(uuid, "content A", 0..10);
        let block2 = ManagedBlock::new(uuid, "content B", 0..10);

        assert_ne!(block1.checksum(), block2.checksum());
    }

    #[test]
    fn checksum_is_independent_of_uuid_and_span() {
        let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

        let block1 = ManagedBlock::new(uuid1, "same content", 0..12);
        let block2 = ManagedBlock::new(uuid2, "same content", 100..200);

        assert_eq!(block1.checksum(), block2.checksum());
    }

    #[test]
    fn has_drifted_returns_false_for_fresh_block() {
        let uuid = Uuid::new_v4();
        let block = ManagedBlock::new(uuid, "content", 0..10);
        assert!(!block.has_drifted());
    }

    #[test]
    fn verify_checksum_matches_stored_value() {
        let uuid = Uuid::new_v4();
        let block = ManagedBlock::new(uuid, "verify me", 0..10);
        let stored = block.checksum().to_string();

        assert!(block.verify_checksum(&stored));
        assert!(!block.verify_checksum("wrong-checksum"));
    }

    #[test]
    fn update_content_recalculates_checksum_correctly() {
        let uuid = Uuid::new_v4();
        let mut block = ManagedBlock::new(uuid, "original", 0..10);
        let original_checksum = block.checksum().to_string();

        block.update_content("updated");

        assert_ne!(block.checksum(), original_checksum);
        assert_eq!(block.content, "updated");
        assert!(!block.has_drifted());

        let mut hasher = Sha256::new();
        hasher.update(b"updated");
        let expected = format!("{:x}", hasher.finalize());
        assert_eq!(block.checksum(), expected);
    }
}
