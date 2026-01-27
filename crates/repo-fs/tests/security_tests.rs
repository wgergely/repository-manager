use repo_fs::path::NormalizedPath;

#[test]
fn test_path_traversal_mitigation() {
    let base = NormalizedPath::new("/var/www");
    let malicious_input = "../../etc/passwd";

    // AFTER FIX: join should resolve ".."
    // So /var/www + ../../etc/passwd should become /etc/passwd
    let joined = base.join(malicious_input);

    // It should NOT be the raw concatenated string anymore
    assert_ne!(
        joined.as_str(),
        "/var/www/../../etc/passwd",
        "Should resolve dot segments"
    );

    // It should be the resolved path
    assert_eq!(
        joined.as_str(),
        "/etc/passwd",
        "Should resolve to absolute path outside root, allowing boundary checks to work"
    );
}

#[test]
fn test_join_resolves_dots() {
    let base = NormalizedPath::new("/a/b");

    assert_eq!(base.join("c").as_str(), "/a/b/c");
    assert_eq!(base.join("./c").as_str(), "/a/b/c");
    assert_eq!(base.join("../c").as_str(), "/a/c");
    assert_eq!(base.join("../../c").as_str(), "/c");

    // Boundary check simulation
    let root = NormalizedPath::new("/workspace");
    let unsafe_path = root.join("../secrets.txt"); // Should be /secrets.txt

    assert_eq!(unsafe_path.as_str(), "/secrets.txt");
    assert!(
        !unsafe_path.as_str().starts_with("/workspace"),
        "Resolved path correctly fails starts_with check"
    );
}

#[test]
fn test_relative_path_sandboxing() {
    // Verify that relative paths starting with .. are sandboxed (.. is dropped)
    let path = NormalizedPath::new("../outside.txt");
    assert_eq!(
        path.as_str(),
        "outside.txt",
        "Leading .. should be dropped for relative paths to enforce sandboxing"
    );

    let path2 = NormalizedPath::new("a/../../b");
    // a -> ..(pop a) -> ..(empty/ignored) -> b
    assert_eq!(path2.as_str(), "b");
}

#[test]
#[cfg(unix)]
fn test_write_atomic_rejects_symlink_in_path() {
    use std::os::unix::fs::symlink;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let real_dir = dir.path().join("real");
    std::fs::create_dir(&real_dir).unwrap();

    let link = dir.path().join("link");
    symlink(&real_dir, &link).unwrap();

    let file_through_link = link.join("file.txt");
    let normalized_path = NormalizedPath::new(file_through_link.to_string_lossy());
    let result = repo_fs::io::write_text(&normalized_path, "content");

    assert!(result.is_err(), "Should reject writes through symlinks");
}
