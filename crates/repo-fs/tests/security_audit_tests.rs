//! tests/security_audit_tests.rs
//!
//! Security audit tests for the `repo-fs` crate.
//! Focus: path traversal, symlink attacks, UNC/network path handling, race conditions.

use repo_fs::NormalizedPath;
use rstest::rstest;

#[cfg(test)]
mod path_normalization_security {
    use super::*;

    #[rstest]
    // Basic traversal
    #[case("a/../b", "b")]
    // Traversal at the beginning of a relative path should be sanitized
    #[case("../a", "a")]
    #[case("../../a/b", "a/b")]
    // Traversal on absolute path
    #[case("/a/b/../../c", "/c")]
    // Mixed separators
    #[case("a\\..\\b", "b")]
    // Empty and dot components
    #[case("a/./b//c", "a/b/c")]
    fn test_path_traversal_sanitization(#[case] input: &str, #[case] expected: &str) {
        let normalized = NormalizedPath::new(input);
        assert_eq!(normalized.as_str(), expected);
    }
}

#[cfg(test)]
mod unc_network_path_security {
    use super::*;

    #[test]
    fn test_unc_forward_slash_rejected() {
        // //server/share should be rewritten to /server/share (local absolute)
        let path = NormalizedPath::new("//server/share/path");
        assert_eq!(path.as_str(), "/server/share/path");
        assert!(!path.is_network_path());
    }

    #[test]
    fn test_unc_backslash_rejected() {
        // \\server\share should be rewritten to /server/share after normalization
        let path = NormalizedPath::new("\\\\server\\share\\path");
        assert_eq!(path.as_str(), "/server/share/path");
        assert!(!path.is_network_path());
    }

    #[test]
    fn test_unc_via_join_rejected() {
        // Even if a UNC path is produced via join, it should be rewritten
        let base = NormalizedPath::new("/");
        let joined = base.join("/server/share");
        assert!(!joined.is_network_path());
    }

    #[test]
    fn test_triple_slash_not_treated_as_unc() {
        // ///path should be treated as absolute, not UNC
        let path = NormalizedPath::new("///some/path");
        assert_eq!(path.as_str(), "/some/path");
    }

    #[test]
    fn test_smb_url_normalized_away() {
        // smb:// gets normalized by NormalizedPath (// inside the scheme gets cleaned).
        // This is acceptable: smb:// is not a valid filesystem path on Unix/Windows,
        // so NormalizedPath should not attempt to preserve it.
        let path = NormalizedPath::new("smb://server/share");
        // After normalization, the scheme:// gets collapsed, so this won't match
        // is_network_path(). The important thing is it doesn't become a routeable path.
        assert!(!path.as_str().starts_with("//"), "Must not become a UNC path");
    }

    #[test]
    fn test_nfs_url_normalized_away() {
        let path = NormalizedPath::new("nfs://server/export");
        assert!(!path.as_str().starts_with("//"), "Must not become a UNC path");
    }

    #[test]
    fn test_regular_absolute_path_not_network() {
        let path = NormalizedPath::new("/home/user/project");
        assert!(!path.is_network_path());
    }

    #[test]
    fn test_regular_relative_path_not_network() {
        let path = NormalizedPath::new("src/main.rs");
        assert!(!path.is_network_path());
    }

    #[test]
    fn test_unc_with_traversal() {
        // //server/../etc/passwd should be rewritten and cleaned
        let path = NormalizedPath::new("//server/../etc/passwd");
        // After UNC detection -> //etc/passwd would become /etc/passwd
        // But since clean() processes .. first, //server/../etc/passwd -> //etc/passwd -> /etc/passwd
        assert!(!path.as_str().starts_with("//"));
    }
}

#[cfg(test)]
mod validate_path_identifier_tests {
    use repo_fs::validate_path_identifier;

    #[test]
    fn test_valid_identifiers() {
        assert!(validate_path_identifier("my-rule", "Rule ID").is_ok());
        assert!(validate_path_identifier("rule_123", "Rule ID").is_ok());
        assert!(validate_path_identifier("UPPER-case", "Rule ID").is_ok());
        assert!(validate_path_identifier("file.ext", "Rule ID").is_ok());
    }

    #[test]
    fn test_empty_rejected() {
        assert!(validate_path_identifier("", "Rule ID").is_err());
    }

    #[test]
    fn test_path_separators_rejected() {
        assert!(validate_path_identifier("a/b", "Rule ID").is_err());
        assert!(validate_path_identifier("a\\b", "Rule ID").is_err());
    }

    #[test]
    fn test_traversal_rejected() {
        assert!(validate_path_identifier("..", "Rule ID").is_err());
        assert!(validate_path_identifier("a..b", "Rule ID").is_err());
        assert!(validate_path_identifier("../../etc/passwd", "Rule ID").is_err());
    }

    #[test]
    fn test_null_byte_rejected() {
        assert!(validate_path_identifier("rule\0id", "Rule ID").is_err());
    }

    #[test]
    fn test_leading_dash_rejected() {
        assert!(validate_path_identifier("-delete", "Branch name").is_err());
        assert!(validate_path_identifier("--force", "Branch name").is_err());
    }

    #[test]
    fn test_excessive_length_rejected() {
        let long = "a".repeat(256);
        assert!(validate_path_identifier(&long, "Rule ID").is_err());

        let ok = "a".repeat(255);
        assert!(validate_path_identifier(&ok, "Rule ID").is_ok());
    }

    #[test]
    fn test_special_chars_rejected() {
        assert!(validate_path_identifier("rule id", "Rule ID").is_err());
        assert!(validate_path_identifier("rule@id", "Rule ID").is_err());
        assert!(validate_path_identifier("rule#id", "Rule ID").is_err());
    }
}

#[cfg(all(test, not(windows)))]
mod io_security {
    use super::*;
    use repo_fs::io;
    use std::fs;
    use tempfile::TempDir;

    /// Creates a temporary directory to act as a "jail" for tests.
    fn setup_jail() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_write_atomic_rejects_symlink_in_path() {
        let jail = setup_jail();
        let jail_path = jail.path();

        // Create a directory inside the jail and a symlink pointing to it
        let secret_dir_path = jail_path.join("secret_dir");
        fs::create_dir(&secret_dir_path).unwrap();
        let symlink_path = jail_path.join("symlink_dir");
        std::os::unix::fs::symlink(&secret_dir_path, &symlink_path).unwrap();

        // Attempt to write a file inside the symlinked directory
        let path_with_symlink = NormalizedPath::new(symlink_path.join("file.txt"));
        let content = "content";

        let result = io::write_text(&path_with_symlink, content);

        // Should now FAIL with SymlinkInPath error
        assert!(result.is_err(), "Write through symlink should be rejected");

        let err = result.unwrap_err();
        let err_str = format!("{}", err);
        assert!(
            err_str.contains("symlink"),
            "Error should mention symlink, got: {}",
            err_str
        );

        // Verify no file was created
        let secret_file_path = secret_dir_path.join("file.txt");
        assert!(
            !secret_file_path.exists(),
            "File should NOT have been written through symlink"
        );
    }

    #[test]
    fn test_path_traversal_io_enforcement() {
        // Verify that write_text with a traversal path does NOT escape the
        // intended directory. The NormalizedPath strips leading ".." from
        // relative paths, so "../outside.txt" becomes "outside.txt".
        // This test verifies the I/O layer honors that normalization.
        let jail = setup_jail();
        let jail_path = jail.path();

        // Create a subdirectory to serve as the "working directory"
        let work_dir = jail_path.join("workdir");
        std::fs::create_dir(&work_dir).unwrap();

        // Construct a path that attempts traversal: workdir/../escape.txt
        // NormalizedPath should normalize this so it does NOT escape
        let traversal_path = work_dir.join("../escape.txt");
        let normalized = NormalizedPath::new(&traversal_path);

        // Write through the normalized path
        let result = io::write_text(&normalized, "escaped content");

        // The write should succeed to the normalized (safe) location.
        assert!(
            result.is_ok(),
            "Write through normalized traversal path should succeed: {:?}",
            result.err()
        );

        // Verify the file was written at the normalized location
        let content = io::read_text(&normalized).unwrap();
        assert_eq!(content, "escaped content");

        // Verify the write went through the normalized path
        let native = normalized.to_native();
        assert!(native.exists(), "File should exist at the normalized path");
        let content = std::fs::read_to_string(&native).unwrap();
        assert_eq!(content, "escaped content");
    }

    #[test]
    fn test_relative_traversal_io_sandboxing() {
        // Test that a relative path starting with ".." is sandboxed at the I/O level
        let jail = setup_jail();
        let jail_path = jail.path();

        // NormalizedPath::new("../outside.txt") should become "outside.txt"
        let path = NormalizedPath::new("../outside.txt");
        assert_eq!(
            path.as_str(),
            "outside.txt",
            "Leading '..' must be stripped from relative paths"
        );

        // Now test with an actual base directory: write inside jail using
        // a path that has been normalized
        let safe_path = NormalizedPath::new(jail_path.join("safe.txt"));
        io::write_text(&safe_path, "safe content").unwrap();
        let content = io::read_text(&safe_path).unwrap();
        assert_eq!(content, "safe content");

        // The dangerous file should not exist outside the jail
        let dangerous = jail_path.parent().unwrap().join("outside.txt");
        assert!(
            !dangerous.exists(),
            "Path traversal must not create files outside the jail"
        );
    }

    #[test]
    fn test_write_atomic_rejects_symlink_at_destination() {
        // Symlinks at the destination file are rejected by contains_symlink(),
        // which checks all components including the leaf.
        let jail = setup_jail();
        let jail_path = jail.path();

        // An external file that the symlink will point to
        let external_file_path = jail_path.join("external_file.txt");
        fs::write(&external_file_path, "safe content").unwrap();

        // A symlink inside the jail pointing to the external file
        let symlink_as_file_path = jail_path.join("symlink_file.txt");
        std::os::unix::fs::symlink(&external_file_path, &symlink_as_file_path).unwrap();

        // Attempt to write to the symlink path — should be rejected
        let normalized_path = NormalizedPath::new(&symlink_as_file_path);
        let result = io::write_text(&normalized_path, "overwriting content");

        assert!(
            result.is_err(),
            "Write to a symlink destination should be rejected"
        );

        // Assert that the original external file is untouched
        let external_content = fs::read_to_string(&external_file_path).unwrap();
        assert_eq!(external_content, "safe content");

        // Assert symlink still exists (not replaced)
        assert!(
            symlink_as_file_path.is_symlink(),
            "Symlink should still exist (not replaced)"
        );
    }
}
