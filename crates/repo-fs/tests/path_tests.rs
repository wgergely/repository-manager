use repo_fs::NormalizedPath;

#[test]
fn test_normalize_forward_slashes() {
    let path = NormalizedPath::new("foo/bar/baz");
    assert_eq!(path.as_str(), "foo/bar/baz");
}

#[test]
fn test_normalize_backslashes_to_forward() {
    let path = NormalizedPath::new("foo\\bar\\baz");
    assert_eq!(path.as_str(), "foo/bar/baz");
}

#[test]
fn test_normalize_mixed_slashes() {
    let path = NormalizedPath::new("foo/bar\\baz");
    assert_eq!(path.as_str(), "foo/bar/baz");
}

#[test]
fn test_join_paths() {
    let base = NormalizedPath::new("foo/bar");
    let joined = base.join("baz");
    assert_eq!(joined.as_str(), "foo/bar/baz");
}

#[test]
fn test_to_native_returns_pathbuf() {
    let path = NormalizedPath::new("foo/bar");
    let native = path.to_native();
    // On Windows this would have backslashes, on Unix forward slashes
    assert!(native.to_string_lossy().contains("bar"));
}

#[test]
fn test_is_network_path_unc() {
    let path = NormalizedPath::new("//server/share/path");
    assert!(path.is_network_path());
}

#[test]
fn test_is_network_path_local() {
    let path = NormalizedPath::new("/home/user/project");
    assert!(!path.is_network_path());
}

#[test]
fn test_parent() {
    let path = NormalizedPath::new("foo/bar/baz");
    let parent = path.parent().unwrap();
    assert_eq!(parent.as_str(), "foo/bar");
}

#[test]
fn test_file_name() {
    let path = NormalizedPath::new("foo/bar/baz.txt");
    assert_eq!(path.file_name(), Some("baz.txt"));
}

#[test]
fn test_exists_false_for_nonexistent() {
    let path = NormalizedPath::new("/nonexistent/path/that/does/not/exist");
    assert!(!path.exists());
}
