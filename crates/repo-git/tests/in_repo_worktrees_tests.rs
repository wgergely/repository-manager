use repo_fs::NormalizedPath;
use repo_git::NamingStrategy;
use repo_git::in_repo_worktrees::InRepoWorktreesLayout;
use repo_git::provider::LayoutProvider;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_in_repo_worktrees() -> (TempDir, InRepoWorktreesLayout) {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    // Initialize regular git repo
    Command::new("git")
        .args(["init"])
        .arg(root)
        .output()
        .expect("Failed to init repo");

    // Configure git for test environment
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
    Command::new("git")
        .current_dir(root)
        .args(["config", "commit.gpgSign", "false"])
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
fn test_in_repo_git_database() {
    let (_temp, layout) = setup_in_repo_worktrees();
    assert!(layout.git_database().as_str().ends_with(".git"));
}

#[test]
fn test_in_repo_main_worktree_is_root() {
    let (temp, layout) = setup_in_repo_worktrees();
    let root_str = NormalizedPath::new(temp.path()).as_str().to_string();
    assert_eq!(layout.main_worktree().as_str(), root_str);
}

#[test]
fn test_in_repo_feature_worktree_path() {
    let (_temp, layout) = setup_in_repo_worktrees();
    let path = layout.feature_worktree("feat-auth");
    assert!(path.as_str().contains(".worktrees"));
    assert!(path.as_str().ends_with("feat-auth"));
}

#[test]
fn test_in_repo_create_and_remove_feature() {
    let (_temp, layout) = setup_in_repo_worktrees();

    // Create feature
    let path = layout.create_feature("test-feature", None).unwrap();
    assert!(path.exists());
    assert!(path.as_str().contains(".worktrees"));

    // Remove feature
    layout.remove_feature("test-feature").unwrap();
    assert!(!path.exists());
}

#[test]
fn test_in_repo_list_worktrees() {
    let (_temp, layout) = setup_in_repo_worktrees();
    let worktrees = layout.list_worktrees().unwrap();

    // Should have main worktree
    assert!(!worktrees.is_empty());
    assert!(worktrees.iter().any(|wt| wt.is_main));
}

#[test]
fn test_in_repo_slug_naming() {
    let (_temp, layout) = setup_in_repo_worktrees();

    // Create with slash in name - should be slugified
    let path = layout.create_feature("feat/user-auth", None).unwrap();
    assert!(path.as_str().ends_with("feat-user-auth"));

    // Cleanup
    layout.remove_feature("feat/user-auth").unwrap();
}

#[test]
fn test_in_repo_current_branch() {
    let (_temp, layout) = setup_in_repo_worktrees();
    let branch = layout.current_branch().unwrap();
    // Git init creates main or master depending on config
    assert!(branch == "main" || branch == "master");
}
