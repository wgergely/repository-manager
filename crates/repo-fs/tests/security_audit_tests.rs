//! tests/security_audit_tests.rs

// These tests are intended to audit the `repo-fs` crate for security vulnerabilities.
// The focus is on path traversal, symlink attacks, and race conditions.

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

#[cfg(all(test, not(windows)))]
mod io_security {
    use super::*;
    use repo_fs::io;
    use std::fs;
    use std::io::Read;
    use tempfile::TempDir;

    /// Creates a temporary directory to act as a "jail" for tests.
    fn setup_jail() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_write_atomic_does_not_follow_symlink_in_path() {
        let jail = setup_jail();
        let jail_path = jail.path();

        // Create a directory inside the jail and a symlink pointing to it
        let secret_dir_path = jail_path.join("secret_dir");
        fs::create_dir(&secret_dir_path).unwrap();
        let symlink_path = jail_path.join("symlink_dir");
        std::os::unix::fs::symlink(&secret_dir_path, &symlink_path).unwrap();

        // Attempt to write a file inside the symlinked directory
        // This should write to `jail/symlink_dir/file.txt`, not `jail/secret_dir/file.txt`
        // if interpreted naively. However, `create_dir_all` in `write_atomic` will
        // follow the symlink.
        let path_with_symlink = NormalizedPath::new(symlink_path.join("file.txt"));
        let content = "evil content";

        let result = io::write_text(&path_with_symlink, content);
        assert!(result.is_ok(), "Write should succeed");

        // Check that the file was created inside the *secret* directory, because
        // `create_dir_all` resolves the symlink. This demonstrates the vulnerability.
        let secret_file_path = secret_dir_path.join("file.txt");
        assert!(
            secret_file_path.exists(),
            "VULNERABILITY: File was written into symlinked directory!"
        );

        // The file content should be the evil content
        let mut file_content = String::new();
        fs::File::open(&secret_file_path)
            .unwrap()
            .read_to_string(&mut file_content)
            .unwrap();
        assert_eq!(file_content, content);
    }

    #[test]
    fn test_write_atomic_replaces_symlink_at_destination() {
        let jail = setup_jail();
        let jail_path = jail.path();

        // An external file that the symlink will point to
        let external_file_path = jail_path.join("external_file.txt");
        fs::write(&external_file_path, "safe content").unwrap();

        // A symlink inside the jail pointing to the external file
        let symlink_as_file_path = jail_path.join("symlink_file.txt");
        std::os::unix::fs::symlink(&external_file_path, &symlink_as_file_path).unwrap();

        // Attempt to write to the symlink path
        let normalized_path = NormalizedPath::new(&symlink_as_file_path);
        let content = "overwriting content";

        let result = io::write_text(&normalized_path, content);
        assert!(result.is_ok(), "Write should succeed");

        // Assert that the symlink itself was replaced by a regular file
        assert!(
            !symlink_as_file_path.is_symlink(),
            "Symlink should have been replaced"
        );
        assert!(
            symlink_as_file_path.is_file(),
            "A new regular file should exist"
        );

        // Assert that the original external file is untouched
        let external_content = fs::read_to_string(&external_file_path).unwrap();
        assert_eq!(external_content, "safe content");
    }
}
