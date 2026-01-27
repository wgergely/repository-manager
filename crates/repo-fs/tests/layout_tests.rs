use repo_fs::{LayoutMode, WorkspaceLayout};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_layout_mode_display() {
    assert_eq!(format!("{}", LayoutMode::Container), "Container");
    assert_eq!(
        format!("{}", LayoutMode::InRepoWorktrees),
        "InRepoWorktrees"
    );
    assert_eq!(format!("{}", LayoutMode::Classic), "Classic");
}

#[test]
fn test_detect_container_layout() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create Container layout signals
    fs::create_dir(root.join(".gt")).unwrap();
    fs::create_dir(root.join("main")).unwrap();
    fs::create_dir(root.join(".repository")).unwrap();

    let layout = WorkspaceLayout::detect(root).unwrap();
    assert_eq!(layout.mode, LayoutMode::Container);
}

#[test]
fn test_detect_in_repo_worktrees_layout() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create InRepoWorktrees layout signals
    fs::create_dir(root.join(".git")).unwrap();
    fs::create_dir(root.join(".worktrees")).unwrap();

    let layout = WorkspaceLayout::detect(root).unwrap();
    assert_eq!(layout.mode, LayoutMode::InRepoWorktrees);
}

#[test]
fn test_detect_classic_layout() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create Classic layout signals
    fs::create_dir(root.join(".git")).unwrap();

    let layout = WorkspaceLayout::detect(root).unwrap();
    assert_eq!(layout.mode, LayoutMode::Classic);
}

#[test]
fn test_detect_fails_without_git() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // No git signals at all
    let result = WorkspaceLayout::detect(root);
    assert!(result.is_err());
}

#[test]
fn test_detect_accepts_git_file_as_gitdir_pointer() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create .git as a FILE (gitdir pointer format used by Git worktrees)
    // In real Git, this would be: "gitdir: /path/to/actual/.git"
    fs::write(root.join(".git"), "gitdir: /some/path/.git").unwrap();

    let result = WorkspaceLayout::detect(root);
    // .git files ARE valid in Git (used for worktrees), so this should be detected
    assert!(result.is_ok(), "Should detect .git file as valid repository");
    assert_eq!(result.unwrap().mode, LayoutMode::Classic);
}
