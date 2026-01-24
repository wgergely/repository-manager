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
