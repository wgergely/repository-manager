//! Tests for error handling under adverse filesystem conditions
//!
//! These tests verify that repo-fs handles real error conditions gracefully.

use repo_fs::{NormalizedPath, io};
use tempfile::tempdir;

#[test]
fn read_text_nonexistent_file_returns_error() {
    let dir = tempdir().unwrap();
    let path = NormalizedPath::new(dir.path().join("does_not_exist.txt"));

    let result = io::read_text(&path);

    assert!(result.is_err(), "Reading non-existent file should fail");
}

#[test]
fn write_text_to_nonexistent_parent_creates_directories() {
    // write_atomic calls create_dir_all, so writing through missing parents should work
    let dir = tempdir().unwrap();
    let path = NormalizedPath::new(dir.path().join("a").join("b").join("c").join("file.txt"));

    let result = io::write_text(&path, "deep content");
    assert!(
        result.is_ok(),
        "write_text should create parent directories"
    );

    let content = io::read_text(&path).unwrap();
    assert_eq!(content, "deep content");
}

#[test]
fn write_atomic_cleans_up_temp_file_on_success() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("target.txt");
    let path = NormalizedPath::new(&file_path);

    io::write_text(&path, "content").unwrap();

    // The temp file pattern is .{filename}.{pid}.tmp
    // Verify no temp files remain in the directory
    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
        .collect();

    assert!(
        entries.is_empty(),
        "No temp files should remain after successful write, found: {:?}",
        entries.iter().map(|e| e.file_name()).collect::<Vec<_>>()
    );
}

#[cfg(unix)]
mod unix_tests {
    use super::*;
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt;

    fn is_root() -> bool {
        match std::process::Command::new("id").arg("-u").output() {
            Ok(output) => String::from_utf8_lossy(&output.stdout).trim() == "0",
            Err(_) => false,
        }
    }

    #[test]
    fn write_atomic_to_readonly_directory_returns_error() {
        if is_root() {
            eprintln!("Skipping test: running as root bypasses permission checks");
            return;
        }
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
    fn read_text_permission_denied_returns_error() {
        if is_root() {
            eprintln!("Skipping test: running as root bypasses permission checks");
            return;
        }
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
    fn write_atomic_unwritable_parent_returns_error() {
        if is_root() {
            eprintln!("Skipping test: running as root bypasses permission checks");
            return;
        }
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

        // Verify original content is untouched (atomicity guarantee)
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(
            content, "original",
            "Original file content must be preserved when write fails"
        );
    }
}

#[cfg(windows)]
mod windows_tests {
    use super::*;
    use std::fs;

    #[test]
    fn write_atomic_readonly_file_fails_and_preserves_original() {
        // On Windows, write_atomic with a readonly target file fails because
        // fs::rename cannot replace a readonly destination (Access denied).
        // The write_atomic retry loop eventually times out.
        // This test verifies: (1) the write fails, and (2) original content is preserved.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("readonly.txt");
        fs::write(&file_path, "original").unwrap();

        // Make file read-only on Windows
        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms).unwrap();

        // Use a short timeout to avoid long waits
        let path = NormalizedPath::new(&file_path);
        let config = repo_fs::RobustnessConfig {
            lock_timeout: std::time::Duration::from_millis(500),
            enable_fsync: false,
        };
        let result = io::write_atomic(&path, b"new content", config);

        // Restore permissions for cleanup
        #[allow(clippy::permissions_set_readonly_false)]
        {
            let mut perms = fs::metadata(&file_path).unwrap().permissions();
            perms.set_readonly(false);
            let _ = fs::set_permissions(&file_path, perms);
        }

        // Assert the write fails on Windows with readonly target
        assert!(
            result.is_err(),
            "write_atomic should fail when target file is readonly on Windows"
        );

        // Assert original content is preserved (atomicity guarantee)
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(
            content, "original",
            "Original file content must be preserved when write fails"
        );
    }

    #[test]
    fn write_atomic_to_invalid_path_returns_error() {
        // Windows-specific invalid path characters
        let path = NormalizedPath::new(r"C:\invalid<>path\file.txt");
        let result = io::write_text(&path, "content");
        assert!(
            result.is_err(),
            "Writing to path with invalid characters should fail"
        );
    }
}
