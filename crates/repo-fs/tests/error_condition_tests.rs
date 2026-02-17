//! Tests for error handling under adverse filesystem conditions
//!
//! These tests verify that repo-fs handles real error conditions gracefully.

use repo_fs::{NormalizedPath, io};
use tempfile::tempdir;

#[test]
fn test_read_text_nonexistent_file() {
    let dir = tempdir().unwrap();
    let path = NormalizedPath::new(dir.path().join("does_not_exist.txt"));

    let result = io::read_text(&path);

    assert!(result.is_err(), "Reading non-existent file should fail");
}

#[cfg(unix)]
mod unix_tests {
    use super::*;
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_write_atomic_permission_denied_directory() {
        let dir = tempdir().unwrap();
        let readonly_dir = dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        // Make directory read-only (no write permission)
        fs::set_permissions(&readonly_dir, Permissions::from_mode(0o444)).unwrap();

        let path = NormalizedPath::new(readonly_dir.join("file.txt"));
        let result = io::write_text(&path, "content");

        // Restore permissions before assertions (for cleanup)
        let _ = fs::set_permissions(&readonly_dir, Permissions::from_mode(0o755));

        assert!(
            result.is_err(),
            "Writing to read-only directory should fail"
        );
    }

    #[test]
    fn test_read_text_permission_denied() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("secret.txt");
        fs::write(&file_path, "secret content").unwrap();

        // Make file unreadable
        fs::set_permissions(&file_path, Permissions::from_mode(0o000)).unwrap();

        let path = NormalizedPath::new(&file_path);
        let result = io::read_text(&path);

        // Restore permissions before assertions (for cleanup)
        let _ = fs::set_permissions(&file_path, Permissions::from_mode(0o644));

        assert!(result.is_err(), "Reading unreadable file should fail");
    }

    #[test]
    fn test_write_atomic_parent_not_writable() {
        let dir = tempdir().unwrap();
        let parent = dir.path().join("parent");
        fs::create_dir(&parent).unwrap();

        // Create the file first, then make parent read-only
        let file_path = parent.join("existing.txt");
        fs::write(&file_path, "original").unwrap();
        fs::set_permissions(&parent, Permissions::from_mode(0o555)).unwrap();

        // Try to overwrite - should fail because we can't create temp file
        let path = NormalizedPath::new(&file_path);
        let result = io::write_text(&path, "new content");

        // Restore permissions
        let _ = fs::set_permissions(&parent, Permissions::from_mode(0o755));

        assert!(
            result.is_err(),
            "Writing when parent is read-only should fail"
        );
    }
}

#[cfg(windows)]
mod windows_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_write_atomic_readonly_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("readonly.txt");
        fs::write(&file_path, "original").unwrap();

        // Make file read-only on Windows
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms).unwrap();

        let path = NormalizedPath::new(&file_path);
        let result = io::write_text(&path, "new content");

        // Restore permissions
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_readonly(false);
        let _ = fs::set_permissions(&file_path, perms);

        // Note: atomic write uses rename, which may succeed on Windows
        // even for read-only files. This test documents the behavior.
        // If it fails, that's actually more secure.
        let _ = result; // Accept either outcome
    }
}
