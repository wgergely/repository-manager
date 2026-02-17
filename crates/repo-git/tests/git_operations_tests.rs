//! Tests for git push/pull/merge operations

use repo_fs::NormalizedPath;
use repo_git::NamingStrategy;
use repo_git::classic::ClassicLayout;
use repo_git::container::ContainerLayout;
use repo_git::in_repo_worktrees::InRepoWorktreesLayout;
use repo_git::provider::LayoutProvider;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// Classic Layout Tests
// ============================================================================

fn setup_classic_repo_with_git() -> (TempDir, ClassicLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Initialize a real git repo
    Command::new("git")
        .args(["init"])
        .arg(root)
        .output()
        .expect("Failed to init repo");

    // Configure git user for commits
    Command::new("git")
        .current_dir(root)
        .args(["config", "user.email", "test@example.com"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["config", "user.name", "Test User"])
        .output()
        .unwrap();

    // Create initial commit
    fs::write(root.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["add", "README.md"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    let layout = ClassicLayout::new(NormalizedPath::new(root)).unwrap();
    (temp, layout)
}

#[test]
fn test_classic_push_no_remote_returns_error() {
    let (_temp, layout) = setup_classic_repo_with_git();

    // Push should fail when no remote is configured
    let result = layout.push(None, None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Remote") && err_str.contains("not found"),
        "Expected 'Remote not found' error, got: {}",
        err_str
    );
}

#[test]
fn test_classic_push_named_remote_not_found() {
    let (_temp, layout) = setup_classic_repo_with_git();

    let result = layout.push(Some("upstream"), None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(err_str.contains("upstream"));
}

#[test]
fn test_classic_pull_no_remote_returns_error() {
    let (_temp, layout) = setup_classic_repo_with_git();

    let result = layout.pull(None, None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Remote") && err_str.contains("not found"),
        "Expected 'Remote not found' error, got: {}",
        err_str
    );
}

#[test]
fn test_classic_merge_branch_not_found() {
    let (_temp, layout) = setup_classic_repo_with_git();

    let result = layout.merge("nonexistent-branch");
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("nonexistent-branch") && err_str.contains("not found"),
        "Expected branch not found error, got: {}",
        err_str
    );
}

#[test]
fn test_classic_merge_fast_forward() {
    let (temp, layout) = setup_classic_repo_with_git();
    let root = temp.path();

    // Create a feature branch with a commit
    Command::new("git")
        .current_dir(root)
        .args(["checkout", "-b", "feature"])
        .output()
        .unwrap();

    fs::write(root.join("feature.txt"), "Feature content").unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["add", "feature.txt"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "Feature commit"])
        .output()
        .unwrap();

    // Go back to main/master
    let main_branch = layout.current_branch().unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["checkout", &main_branch])
        .output()
        .unwrap();

    // Merge feature branch (should fast-forward)
    let result = layout.merge("feature");
    assert!(result.is_ok(), "Merge failed: {:?}", result);

    // Verify the file exists after merge
    assert!(root.join("feature.txt").exists());
}

#[test]
fn test_classic_merge_already_up_to_date() {
    let (temp, layout) = setup_classic_repo_with_git();
    let root = temp.path();

    // Create a feature branch at the same commit
    Command::new("git")
        .current_dir(root)
        .args(["branch", "feature"])
        .output()
        .unwrap();

    // Merge should succeed (already up to date)
    let result = layout.merge("feature");
    assert!(result.is_ok(), "Merge failed: {:?}", result);
}

// ============================================================================
// Container Layout Tests
// ============================================================================

fn setup_container_repo_with_git() -> (TempDir, ContainerLayout) {
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

    // Configure git user in main worktree
    Command::new("git")
        .current_dir(&main_dir)
        .args(["config", "user.email", "test@example.com"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(&main_dir)
        .args(["config", "user.name", "Test User"])
        .output()
        .unwrap();

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
fn test_container_push_no_remote_returns_error() {
    let (_temp, layout) = setup_container_repo_with_git();

    let result = layout.push(None, None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Remote") && err_str.contains("not found"),
        "Expected 'Remote not found' error, got: {}",
        err_str
    );
}

#[test]
fn test_container_pull_no_remote_returns_error() {
    let (_temp, layout) = setup_container_repo_with_git();

    let result = layout.pull(None, None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Remote") && err_str.contains("not found"),
        "Expected 'Remote not found' error, got: {}",
        err_str
    );
}

#[test]
fn test_container_merge_branch_not_found() {
    let (_temp, layout) = setup_container_repo_with_git();

    let result = layout.merge("nonexistent-branch");
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("nonexistent-branch") && err_str.contains("not found"),
        "Expected branch not found error, got: {}",
        err_str
    );
}

#[test]
fn test_container_merge_fast_forward() {
    let (temp, layout) = setup_container_repo_with_git();
    let root = temp.path();
    let main_dir = root.join("main");

    // Create a feature worktree
    let feature_path = layout.create_feature("feature", None).unwrap();

    // Add a commit in feature
    fs::write(feature_path.join("feature.txt"), "Feature content").unwrap();
    Command::new("git")
        .current_dir(feature_path.to_native())
        .args(["add", "feature.txt"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(feature_path.to_native())
        .args(["commit", "-m", "Feature commit"])
        .output()
        .unwrap();

    // Merge feature branch into main
    let result = layout.merge("feature");
    assert!(result.is_ok(), "Merge failed: {:?}", result);

    // Verify the file exists in main after merge
    assert!(main_dir.join("feature.txt").exists());
}

// ============================================================================
// InRepoWorktrees Layout Tests
// ============================================================================

fn setup_in_repo_worktrees_with_git() -> (TempDir, InRepoWorktreesLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Initialize regular git repo
    Command::new("git")
        .args(["init"])
        .arg(root)
        .output()
        .expect("Failed to init repo");

    // Configure git user for commits
    Command::new("git")
        .current_dir(root)
        .args(["config", "user.email", "test@example.com"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["config", "user.name", "Test User"])
        .output()
        .unwrap();

    // Create initial commit
    fs::write(root.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["add", "README.md"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["commit", "-m", "Initial commit"])
        .output()
        .unwrap();

    // Create .worktrees directory
    fs::create_dir(root.join(".worktrees")).unwrap();

    let layout =
        InRepoWorktreesLayout::new(NormalizedPath::new(root), NamingStrategy::Slug).unwrap();

    (temp, layout)
}

#[test]
fn test_in_repo_push_no_remote_returns_error() {
    let (_temp, layout) = setup_in_repo_worktrees_with_git();

    let result = layout.push(None, None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Remote") && err_str.contains("not found"),
        "Expected 'Remote not found' error, got: {}",
        err_str
    );
}

#[test]
fn test_in_repo_pull_no_remote_returns_error() {
    let (_temp, layout) = setup_in_repo_worktrees_with_git();

    let result = layout.pull(None, None);
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("Remote") && err_str.contains("not found"),
        "Expected 'Remote not found' error, got: {}",
        err_str
    );
}

#[test]
fn test_in_repo_merge_branch_not_found() {
    let (_temp, layout) = setup_in_repo_worktrees_with_git();

    let result = layout.merge("nonexistent-branch");
    assert!(result.is_err());

    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("nonexistent-branch") && err_str.contains("not found"),
        "Expected branch not found error, got: {}",
        err_str
    );
}

#[test]
fn test_in_repo_merge_fast_forward() {
    let (temp, layout) = setup_in_repo_worktrees_with_git();
    let root = temp.path();

    // Create a feature worktree
    let feature_path = layout.create_feature("feature", None).unwrap();

    // Add a commit in feature
    fs::write(feature_path.join("feature.txt"), "Feature content").unwrap();
    Command::new("git")
        .current_dir(feature_path.to_native())
        .args(["add", "feature.txt"])
        .output()
        .unwrap();
    Command::new("git")
        .current_dir(feature_path.to_native())
        .args(["commit", "-m", "Feature commit"])
        .output()
        .unwrap();

    // Go back to main in the root worktree
    let main_branch = layout.current_branch().unwrap();
    Command::new("git")
        .current_dir(root)
        .args(["checkout", &main_branch])
        .output()
        .unwrap();

    // Merge feature branch
    let result = layout.merge("feature");
    assert!(result.is_ok(), "Merge failed: {:?}", result);

    // Verify the file exists in root after merge
    assert!(root.join("feature.txt").exists());
}
