//! SHA-256 checksum utilities
//!
//! Provides a single canonical checksum format (`sha256:<hex>`) used throughout
//! the workspace for content integrity verification and drift detection.

use sha2::{Digest, Sha256};
use std::path::Path;

/// Prefix for all checksums produced by this module
const PREFIX: &str = "sha256:";

/// Compute the SHA-256 checksum of string content.
///
/// Returns a string in the canonical format `"sha256:<hex>"`.
pub fn compute_content_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{}{:x}", PREFIX, hasher.finalize())
}

/// Compute the SHA-256 checksum of a file's contents.
///
/// Returns a string in the canonical format `"sha256:<hex>"`.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn compute_file_checksum(path: &Path) -> std::io::Result<String> {
    let content = std::fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{}{:x}", PREFIX, hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_checksum_has_prefix() {
        let checksum = compute_content_checksum("hello world");
        assert!(checksum.starts_with("sha256:"));
    }

    #[test]
    fn content_checksum_is_deterministic() {
        let a = compute_content_checksum("test");
        let b = compute_content_checksum("test");
        assert_eq!(a, b);
    }

    #[test]
    fn different_content_different_checksum() {
        let a = compute_content_checksum("aaa");
        let b = compute_content_checksum("bbb");
        assert_ne!(a, b);
    }

    #[test]
    fn content_checksum_known_value() {
        let checksum = compute_content_checksum("hello world");
        assert_eq!(
            checksum,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn file_checksum_matches_content_checksum() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello world").unwrap();

        let file_cs = compute_file_checksum(&path).unwrap();
        let content_cs = compute_content_checksum("hello world");
        assert_eq!(file_cs, content_cs);
    }
}
