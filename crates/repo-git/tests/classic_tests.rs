use repo_fs::NormalizedPath;
use repo_git::classic::ClassicLayout;
use repo_git::provider::LayoutProvider;
use std::fs;
use tempfile::TempDir;

fn setup_classic_repo() -> (TempDir, ClassicLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Initialize a basic git repo structure
    fs::create_dir(root.join(".git")).unwrap();
    fs::write(root.join(".git/HEAD"), "ref: refs/heads/main").unwrap();
    fs::create_dir_all(root.join(".git/refs/heads")).unwrap();

    let layout = ClassicLayout::new(NormalizedPath::new(root)).unwrap();
    (temp, layout)
}

#[test]
fn test_classic_git_database() {
    let (_temp, layout) = setup_classic_repo();
    assert!(layout.git_database().as_str().ends_with(".git"));
}

#[test]
fn test_classic_main_worktree_is_root() {
    let (_temp, layout) = setup_classic_repo();
    // In classic layout, main worktree IS the root
    assert_eq!(
        layout.main_worktree().as_str(),
        layout.git_database().parent().unwrap().as_str()
    );
}

#[test]
fn test_classic_create_feature_returns_error() {
    let (_temp, layout) = setup_classic_repo();
    let result = layout.create_feature("test-feature", None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("not supported"));
    assert!(err_str.contains("Classic"));
}

#[test]
fn test_classic_remove_feature_returns_error() {
    let (_temp, layout) = setup_classic_repo();
    let result = layout.remove_feature("test-feature");
    assert!(result.is_err());
}
