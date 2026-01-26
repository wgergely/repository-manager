//! Tests for Mode abstraction and backends

use repo_core::mode::Mode;
use repo_core::backend::{BranchInfo, ModeBackend, StandardBackend, WorktreeBackend};
use repo_fs::NormalizedPath;
use std::fs;
use tempfile::TempDir;

// =============================================================================
// Mode enum tests
// =============================================================================

#[test]
fn test_mode_from_str_standard() {
    assert_eq!("standard".parse::<Mode>().unwrap(), Mode::Standard);
    assert_eq!("default".parse::<Mode>().unwrap(), Mode::Standard);
}

#[test]
fn test_mode_from_str_worktrees() {
    assert_eq!("worktrees".parse::<Mode>().unwrap(), Mode::Worktrees);
    assert_eq!("worktree".parse::<Mode>().unwrap(), Mode::Worktrees);
    assert_eq!("container".parse::<Mode>().unwrap(), Mode::Worktrees);
}

#[test]
fn test_mode_from_str_case_insensitive() {
    assert_eq!("STANDARD".parse::<Mode>().unwrap(), Mode::Standard);
    assert_eq!("Standard".parse::<Mode>().unwrap(), Mode::Standard);
    assert_eq!("WORKTREES".parse::<Mode>().unwrap(), Mode::Worktrees);
    assert_eq!("Worktree".parse::<Mode>().unwrap(), Mode::Worktrees);
    assert_eq!("Container".parse::<Mode>().unwrap(), Mode::Worktrees);
}

#[test]
fn test_mode_from_str_invalid() {
    let result = "invalid".parse::<Mode>();
    assert!(result.is_err());

    let result = "unknown".parse::<Mode>();
    assert!(result.is_err());

    let result = "".parse::<Mode>();
    assert!(result.is_err());
}

#[test]
fn test_mode_display() {
    assert_eq!(Mode::Standard.to_string(), "standard");
    assert_eq!(Mode::Worktrees.to_string(), "worktrees");
}

// =============================================================================
// StandardBackend tests
// =============================================================================

fn setup_standard_repo() -> TempDir {
    let dir = TempDir::new().unwrap();

    // Create a basic git repository structure
    fs::create_dir(dir.path().join(".git")).unwrap();

    // Create HEAD file to make it look like a real repo
    fs::write(dir.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

    // Create refs structure
    fs::create_dir_all(dir.path().join(".git/refs/heads")).unwrap();
    fs::write(dir.path().join(".git/refs/heads/main"), "").unwrap();

    dir
}

#[test]
fn test_standard_backend_new_valid() {
    let temp = setup_standard_repo();
    let root = NormalizedPath::new(temp.path());

    let backend = StandardBackend::new(root);
    assert!(backend.is_ok());
}

#[test]
fn test_standard_backend_new_invalid() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());

    // No .git directory
    let backend = StandardBackend::new(root);
    assert!(backend.is_err());
}

#[test]
fn test_standard_backend_config_root() {
    let temp = setup_standard_repo();
    let root = NormalizedPath::new(temp.path());

    let backend = StandardBackend::new(root.clone()).unwrap();
    let config_root = backend.config_root();

    // Should be {root}/.repository
    let expected = root.join(".repository");
    assert_eq!(config_root.as_str(), expected.as_str());
}

#[test]
fn test_standard_backend_working_dir() {
    let temp = setup_standard_repo();
    let root = NormalizedPath::new(temp.path());

    let backend = StandardBackend::new(root.clone()).unwrap();
    let working_dir = backend.working_dir();

    assert_eq!(working_dir.as_str(), root.as_str());
}

// =============================================================================
// WorktreeBackend tests
// =============================================================================

fn setup_worktree_container() -> TempDir {
    let dir = TempDir::new().unwrap();

    // Create container structure with .gt (git database) and main worktree
    fs::create_dir(dir.path().join(".gt")).unwrap();
    fs::create_dir(dir.path().join("main")).unwrap();

    // Create a .git file in main that points to .gt
    fs::write(
        dir.path().join("main/.git"),
        format!("gitdir: {}/.gt", dir.path().display())
    ).unwrap();

    // Create basic git structure in .gt
    fs::write(dir.path().join(".gt/HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::create_dir_all(dir.path().join(".gt/refs/heads")).unwrap();
    fs::write(dir.path().join(".gt/refs/heads/main"), "").unwrap();

    dir
}

#[test]
fn test_worktree_backend_new_valid() {
    let temp = setup_worktree_container();
    let container = NormalizedPath::new(temp.path());

    let backend = WorktreeBackend::new(container);
    assert!(backend.is_ok());
}

#[test]
fn test_worktree_backend_new_invalid() {
    let temp = TempDir::new().unwrap();
    let container = NormalizedPath::new(temp.path());

    // No .gt directory
    let backend = WorktreeBackend::new(container);
    assert!(backend.is_err());
}

#[test]
fn test_worktree_backend_with_worktree_valid() {
    let temp = setup_worktree_container();
    let container = NormalizedPath::new(temp.path());
    let worktree = NormalizedPath::new(temp.path().join("main"));

    let backend = WorktreeBackend::with_worktree(container, worktree);
    assert!(backend.is_ok());
}

#[test]
fn test_worktree_backend_config_root() {
    let temp = setup_worktree_container();
    let container = NormalizedPath::new(temp.path());

    let backend = WorktreeBackend::new(container.clone()).unwrap();
    let config_root = backend.config_root();

    // Should be {container}/.repository (shared at container level)
    let expected = container.join(".repository");
    assert_eq!(config_root.as_str(), expected.as_str());
}

#[test]
fn test_worktree_backend_working_dir() {
    let temp = setup_worktree_container();
    let container = NormalizedPath::new(temp.path());
    let main_worktree = NormalizedPath::new(temp.path().join("main"));

    // Default to main worktree
    let backend = WorktreeBackend::new(container.clone()).unwrap();
    let working_dir = backend.working_dir();

    assert_eq!(working_dir.as_str(), main_worktree.as_str());
}

#[test]
fn test_worktree_backend_with_specific_worktree() {
    let temp = setup_worktree_container();

    // Create a feature worktree directory
    fs::create_dir(temp.path().join("feature-x")).unwrap();
    fs::write(
        temp.path().join("feature-x/.git"),
        format!("gitdir: {}/.gt/worktrees/feature-x", temp.path().display())
    ).unwrap();

    let container = NormalizedPath::new(temp.path());
    let feature_worktree = NormalizedPath::new(temp.path().join("feature-x"));

    let backend = WorktreeBackend::with_worktree(container, feature_worktree.clone()).unwrap();
    let working_dir = backend.working_dir();

    assert_eq!(working_dir.as_str(), feature_worktree.as_str());
}

// =============================================================================
// BranchInfo tests
// =============================================================================

#[test]
fn test_branch_info_standard_mode() {
    // In standard mode, path should be None
    let info = BranchInfo {
        name: "main".to_string(),
        path: None,
        is_current: true,
        is_main: true,
    };

    assert_eq!(info.name, "main");
    assert!(info.path.is_none());
    assert!(info.is_current);
    assert!(info.is_main);
}

#[test]
fn test_branch_info_worktree_mode() {
    // In worktree mode, path should be Some
    let path = NormalizedPath::new("/container/main");
    let info = BranchInfo {
        name: "main".to_string(),
        path: Some(path.clone()),
        is_current: true,
        is_main: true,
    };

    assert_eq!(info.name, "main");
    assert_eq!(info.path.as_ref().unwrap().as_str(), path.as_str());
    assert!(info.is_current);
    assert!(info.is_main);
}
