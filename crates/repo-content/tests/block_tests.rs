//! Tests for ManagedBlock

use repo_content::block::ManagedBlock;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[test]
fn checksum_matches_independently_computed_sha256() {
    // C6: Verify the checksum is a correct SHA-256 hash, not just "deterministic"
    let uuid = Uuid::new_v4();
    let content = "test content for checksum verification";
    let block = ManagedBlock::new(uuid, content, 0..40);

    // Compute expected SHA-256 independently
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let expected = format!("{:x}", hasher.finalize());

    assert_eq!(
        block.checksum(),
        expected,
        "Block checksum must match independently computed SHA-256"
    );
}

#[test]
fn checksum_changes_when_content_changes() {
    let uuid = Uuid::new_v4();
    let block1 = ManagedBlock::new(uuid, "content A", 0..10);
    let block2 = ManagedBlock::new(uuid, "content B", 0..10);

    assert_ne!(
        block1.checksum(),
        block2.checksum(),
        "Different content must produce different checksums"
    );
}

#[test]
fn checksum_is_independent_of_uuid_and_span() {
    // Checksum should only depend on content, not on UUID or span
    let uuid1 = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let uuid2 = Uuid::parse_str("6ba7b810-9dad-11d1-80b4-00c04fd430c8").unwrap();

    let block1 = ManagedBlock::new(uuid1, "same content", 0..12);
    let block2 = ManagedBlock::new(uuid2, "same content", 100..200);

    assert_eq!(
        block1.checksum(),
        block2.checksum(),
        "Checksum must only depend on content, not UUID or span"
    );
}

#[test]
fn has_drifted_returns_false_for_fresh_block() {
    let uuid = Uuid::new_v4();
    let block = ManagedBlock::new(uuid, "content", 0..10);
    assert!(
        !block.has_drifted(),
        "Freshly created block should not report drift"
    );
}

#[test]
fn verify_checksum_matches_stored_value() {
    let uuid = Uuid::new_v4();
    let block = ManagedBlock::new(uuid, "verify me", 0..10);
    let stored = block.checksum().to_string();

    assert!(
        block.verify_checksum(&stored),
        "verify_checksum should return true for the block's own checksum"
    );
    assert!(
        !block.verify_checksum("wrong-checksum"),
        "verify_checksum should return false for incorrect checksum"
    );
}

#[test]
fn update_content_recalculates_checksum_correctly() {
    let uuid = Uuid::new_v4();
    let mut block = ManagedBlock::new(uuid, "original", 0..10);
    let original_checksum = block.checksum().to_string();

    block.update_content("updated");

    assert_ne!(block.checksum(), original_checksum);
    assert_eq!(block.content, "updated");
    assert!(!block.has_drifted(), "Block should not drift after update_content");

    // Verify the new checksum is correct
    let mut hasher = Sha256::new();
    hasher.update(b"updated");
    let expected = format!("{:x}", hasher.finalize());
    assert_eq!(block.checksum(), expected);
}
