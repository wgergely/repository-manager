use assert_fs::prelude::*;
use repo_fs::{LayoutMode, RepoPath, WorkspaceLayout};

#[test]
fn detect_at_prefers_container_layout() {
    let temp = assert_fs::TempDir::new().unwrap();

    temp.child(RepoPath::GtDir.as_str())
        .create_dir_all()
        .unwrap();
    temp.child(RepoPath::MainWorktree.as_str())
        .create_dir_all()
        .unwrap();
    temp.child(RepoPath::GitDir.as_str())
        .create_dir_all()
        .unwrap();
    temp.child(RepoPath::WorktreesDir.as_str())
        .create_dir_all()
        .unwrap();

    let layout = WorkspaceLayout::detect(temp.path()).unwrap();
    assert_eq!(layout.mode, LayoutMode::Container);
}

#[test]
fn detect_at_prefers_in_repo_worktrees_over_classic() {
    let temp = assert_fs::TempDir::new().unwrap();

    temp.child(RepoPath::GitDir.as_str())
        .create_dir_all()
        .unwrap();
    temp.child(RepoPath::WorktreesDir.as_str())
        .create_dir_all()
        .unwrap();

    let layout = WorkspaceLayout::detect(temp.path()).unwrap();
    assert_eq!(layout.mode, LayoutMode::InRepoWorktrees);
}

#[test]
fn detect_at_falls_back_to_classic() {
    let temp = assert_fs::TempDir::new().unwrap();

    temp.child(RepoPath::GitDir.as_str())
        .create_dir_all()
        .unwrap();

    let layout = WorkspaceLayout::detect(temp.path()).unwrap();
    assert_eq!(layout.mode, LayoutMode::Classic);
}

#[test]
fn detect_fails_when_expected_dir_is_a_file() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Test for Container layout failure (Container needs .gt and main as DIRS)
    // Case 1: .gt is a file
    temp.child(RepoPath::GtDir.as_str()).touch().unwrap(); // Create as file
    temp.child(RepoPath::MainWorktree.as_str())
        .create_dir_all()
        .unwrap();

    assert!(WorkspaceLayout::detect(temp.path()).is_err());

    // Clean up to try next case
    // Note: on Windows, deleting immediately might be flaky, so we use a fresh temp dir for the second case
    // to avoid "Access Denied" or lock issues.
    let temp2 = assert_fs::TempDir::new().unwrap();

    // Case 2: main is a file
    temp2
        .child(RepoPath::GtDir.as_str())
        .create_dir_all()
        .unwrap();
    temp2
        .child(RepoPath::MainWorktree.as_str())
        .touch()
        .unwrap(); // Create as file

    assert!(WorkspaceLayout::detect(temp2.path()).is_err());
}

#[test]
fn validate_fails_if_component_is_missing() {
    // Container
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(RepoPath::GtDir.as_str())
        .create_dir_all()
        .unwrap();
    // Missing main
    let layout = WorkspaceLayout {
        root: temp.path().to_path_buf().into(),
        active_context: temp.path().to_path_buf().into(),
        mode: LayoutMode::Container,
    };
    assert!(layout.validate().is_err());

    // InRepoWorktrees
    let temp2 = assert_fs::TempDir::new().unwrap();
    // Missing .git
    let layout = WorkspaceLayout {
        root: temp2.path().to_path_buf().into(),
        active_context: temp2.path().to_path_buf().into(),
        mode: LayoutMode::InRepoWorktrees,
    };
    assert!(layout.validate().is_err());
}

#[test]
fn validate_fails_if_component_is_file_instead_of_dir() {
    // Container with .gt as file
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(RepoPath::GtDir.as_str()).touch().unwrap();
    temp.child(RepoPath::MainWorktree.as_str())
        .create_dir_all()
        .unwrap();

    let layout = WorkspaceLayout {
        root: temp.path().to_path_buf().into(),
        active_context: temp.path().to_path_buf().into(),
        mode: LayoutMode::Container,
    };
    // This should fail now that we fixed the bug
    assert!(
        layout.validate().is_err(),
        "Validation should fail if .gt is a file"
    );

    // Container with main as file
    let temp2 = assert_fs::TempDir::new().unwrap();
    temp2
        .child(RepoPath::GtDir.as_str())
        .create_dir_all()
        .unwrap();
    temp2
        .child(RepoPath::MainWorktree.as_str())
        .touch()
        .unwrap();

    let layout = WorkspaceLayout {
        root: temp2.path().to_path_buf().into(),
        active_context: temp2.path().to_path_buf().into(),
        mode: LayoutMode::Container,
    };
    assert!(
        layout.validate().is_err(),
        "Validation should fail if main is a file"
    );
}

#[test]
fn detection_from_deep_subdirectory() {
    let temp = assert_fs::TempDir::new().unwrap();

    temp.child(RepoPath::GitDir.as_str())
        .create_dir_all()
        .unwrap();
    let deep = temp.child("a/b/c");
    deep.create_dir_all().unwrap();

    let layout = WorkspaceLayout::detect(deep.path()).unwrap();
    assert_eq!(layout.mode, LayoutMode::Classic);
    // Use to_native() as NormalizedPath does not have as_path()
    assert_eq!(
        layout.root.to_native(),
        dunce::canonicalize(temp.path()).unwrap()
    );
}
