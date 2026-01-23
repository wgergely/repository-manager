//! Tests for ManagedBlock

use repo_content::block::{BlockLocation, ManagedBlock};
use uuid::Uuid;

#[test]
fn test_managed_block_checksum() {
    let uuid = Uuid::new_v4();
    let block = ManagedBlock::new(uuid, "test content", 0..20);

    // Checksum should be consistent
    let checksum1 = block.checksum();
    let block2_same = ManagedBlock::new(uuid, "test content", 0..20);
    let checksum2 = block2_same.checksum();
    assert_eq!(checksum1, checksum2);

    // Different content should have different checksum
    let block2 = ManagedBlock::new(uuid, "different content", 0..25);
    assert_ne!(block.checksum(), block2.checksum());
}

#[test]
fn test_block_location_ordering() {
    // BlockLocation::End should work
    let _loc = BlockLocation::End;

    // BlockLocation::After should work
    let _loc = BlockLocation::After("section".to_string());

    // BlockLocation::Before should work
    let _loc = BlockLocation::Before("section".to_string());

    // BlockLocation::Offset should work
    let _loc = BlockLocation::Offset(100);
}

#[test]
fn test_block_update_content() {
    let uuid = Uuid::new_v4();
    let mut block = ManagedBlock::new(uuid, "original", 0..10);
    let original_checksum = block.checksum().to_string();

    block.update_content("updated");
    assert_ne!(block.checksum(), original_checksum);
    assert_eq!(block.content, "updated");
}

#[test]
fn test_block_default_location() {
    let loc: BlockLocation = Default::default();
    assert_eq!(loc, BlockLocation::End);
}
