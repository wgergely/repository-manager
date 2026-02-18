use git2::Repository;
use repo_fs::NormalizedPath;
use repo_git::classic::ClassicLayout;
use repo_git::provider::LayoutProvider;
use tempfile::TempDir;

fn setup_classic_repo() -> (TempDir, ClassicLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Initialize a real git repo using git2
    Repository::init(root).unwrap();

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
