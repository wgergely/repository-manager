use repo_git::container::ContainerLayout;
use repo_git::provider::LayoutProvider;
use repo_git::NamingStrategy;
use repo_fs::NormalizedPath;
use std::process::Command;
use std::fs;
use tempfile::TempDir;

fn setup_container_repo() -> (TempDir, ContainerLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Create container structure with real git
    let gt_dir = root.join(".gt");
    let main_dir = root.join("main");

    // Initialize bare repo in .gt
    Command::new("git")
        .args(["init", "--bare"])
        .arg(&gt_dir)
        .output()
        .expect("Failed to init bare repo");

    // Add main as worktree
    Command::new("git")
        .current_dir(&gt_dir)
        .args(["worktree", "add", "--orphan", "-b", "main"])
        .arg(&main_dir)
        .output()
        .expect("Failed to add main worktree");

    // Create an initial commit in main
    fs::write(main_dir.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .current_dir(&main_dir)
        .args(["add", "README.md"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(&main_dir)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    let layout = ContainerLayout::new(
        NormalizedPath::new(root),
        NamingStrategy::Slug,
    ).unwrap();

    (temp, layout)
}

#[test]
fn test_container_git_database() {
    let (_temp, layout) = setup_container_repo();
    assert!(layout.git_database().as_str().ends_with(".gt"));
}

#[test]
fn test_container_main_worktree() {
    let (_temp, layout) = setup_container_repo();
    assert!(layout.main_worktree().as_str().ends_with("main"));
}

#[test]
fn test_container_feature_worktree_path() {
    let (_temp, layout) = setup_container_repo();
    let path = layout.feature_worktree("feat-auth");
    assert!(path.as_str().ends_with("feat-auth"));
}

#[test]
fn test_container_list_worktrees() {
    let (_temp, layout) = setup_container_repo();
    let worktrees = layout.list_worktrees().unwrap();

    assert!(!worktrees.is_empty());
    assert!(worktrees.iter().any(|wt| wt.is_main));
}

#[test]
fn test_container_create_and_remove_feature() {
    let (_temp, layout) = setup_container_repo();

    // Create feature
    let path = layout.create_feature("test-feature", None).unwrap();
    assert!(path.exists());

    // Verify it's in list
    let worktrees = layout.list_worktrees().unwrap();
    assert!(worktrees.iter().any(|wt| wt.name == "test-feature"));

    // Remove feature
    layout.remove_feature("test-feature").unwrap();
    assert!(!path.exists());
}

#[test]
fn test_container_slug_naming() {
    let (_temp, layout) = setup_container_repo();

    // Create with slash in name - should be slugified
    let path = layout.create_feature("feat/user-auth", None).unwrap();
    assert!(path.as_str().ends_with("feat-user-auth"));

    // Cleanup
    layout.remove_feature("feat/user-auth").unwrap();
}
