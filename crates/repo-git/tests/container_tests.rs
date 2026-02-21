use repo_fs::NormalizedPath;
use repo_git::NamingStrategy;
use repo_git::container::ContainerLayout;
use repo_git::provider::LayoutProvider;
use std::fs;
use std::process::Command;
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

    // Set HEAD to refs/heads/main (git init --bare defaults to master)
    Command::new("git")
        .args(["symbolic-ref", "HEAD", "refs/heads/main"])
        .current_dir(&gt_dir)
        .status()
        .expect("Failed to set HEAD to refs/heads/main");

    // Add main as worktree
    Command::new("git")
        .current_dir(&gt_dir)
        .args(["worktree", "add", "--orphan", "-b", "main"])
        .arg(&main_dir)
        .output()
        .expect("Failed to add main worktree");

    // Configure git user and disable signing for test commits
    Command::new("git")
        .current_dir(&gt_dir)
        .args(["config", "user.email", "test@test.com"])
        .status()
        .expect("Failed to configure git email");
    Command::new("git")
        .current_dir(&gt_dir)
        .args(["config", "user.name", "Test User"])
        .status()
        .expect("Failed to configure git name");
    Command::new("git")
        .current_dir(&gt_dir)
        .args(["config", "commit.gpgsign", "false"])
        .status()
        .expect("Failed to disable commit signing");

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

    let layout = ContainerLayout::new(NormalizedPath::new(root), NamingStrategy::Slug).unwrap();

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

#[test]
fn test_container_create_duplicate_feature_returns_error() {
    let (_temp, layout) = setup_container_repo();

    // Create feature first time - should succeed
    let path = layout.create_feature("dup-feature", None).unwrap();
    assert!(path.exists());

    // Create same feature again - should return an error
    let result = layout.create_feature("dup-feature", None);
    assert!(
        result.is_err(),
        "Creating a duplicate feature worktree should return an error"
    );

    // Cleanup
    layout.remove_feature("dup-feature").unwrap();
}

#[test]
fn test_container_remove_nonexistent_feature_returns_error() {
    let (_temp, layout) = setup_container_repo();

    let result = layout.remove_feature("nonexistent-feature");
    assert!(
        result.is_err(),
        "Removing a non-existent feature should return an error"
    );
}
