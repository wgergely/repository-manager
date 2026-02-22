//! Tests for Mode abstraction and backends

use repo_core::backend::{ModeBackend, StandardBackend, WorktreeBackend};
use repo_core::mode::Mode;
use repo_fs::NormalizedPath;
use repo_test_utils::git::fake_git_dir;
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

#[test]
fn test_standard_backend_new_valid() {
    let temp = TempDir::new().unwrap();
    fake_git_dir(temp.path());
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
    let temp = TempDir::new().unwrap();
    fake_git_dir(temp.path());
    let root = NormalizedPath::new(temp.path());

    let backend = StandardBackend::new(root.clone()).unwrap();
    let config_root = backend.config_root();

    // Should be {root}/.repository
    let expected = root.join(".repository");
    assert_eq!(config_root.as_str(), expected.as_str());
}

#[test]
fn test_standard_backend_working_dir() {
    let temp = TempDir::new().unwrap();
    fake_git_dir(temp.path());
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
        format!("gitdir: {}/.gt", dir.path().display()),
    )
    .unwrap();

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
        format!("gitdir: {}/.gt/worktrees/feature-x", temp.path().display()),
    )
    .unwrap();

    let container = NormalizedPath::new(temp.path());
    let feature_worktree = NormalizedPath::new(temp.path().join("feature-x"));

    let backend = WorktreeBackend::with_worktree(container, feature_worktree.clone()).unwrap();
    let working_dir = backend.working_dir();

    assert_eq!(working_dir.as_str(), feature_worktree.as_str());
}

// =============================================================================
// BranchInfo behavioral tests
// =============================================================================

#[test]
fn test_standard_backend_accepts_git_dir_without_head() {
    // StandardBackend only checks that .git exists, not its internal structure.
    // This documents the current behavior: .git as a directory is sufficient.
    let temp = TempDir::new().unwrap();

    // Create .git dir but no HEAD file
    fs::create_dir(temp.path().join(".git")).unwrap();

    let root = NormalizedPath::new(temp.path());
    let result = StandardBackend::new(root);

    // Production only checks .git exists, does not validate HEAD
    assert!(
        result.is_ok(),
        "StandardBackend accepts .git dir without HEAD"
    );
    let backend = result.unwrap();
    assert_eq!(
        backend.config_root().as_str(),
        NormalizedPath::new(temp.path())
            .join(".repository")
            .as_str()
    );
}

#[test]
fn test_worktree_backend_accepts_missing_main_worktree() {
    // WorktreeBackend only checks that .gt exists, it does NOT validate main/.
    // It sets main as the default worktree path but doesn't require it to exist.
    let temp = TempDir::new().unwrap();
    fs::create_dir(temp.path().join(".gt")).unwrap();
    // No main/ directory

    let container = NormalizedPath::new(temp.path());
    let result = WorktreeBackend::new(container);

    // Production only checks .gt exists, does not validate main/ directory
    assert!(
        result.is_ok(),
        "WorktreeBackend accepts container with .gt even if main/ is missing"
    );
    let backend = result.unwrap();
    // working_dir points to main/ even though it doesn't exist on disk
    assert!(backend.working_dir().as_str().contains("main"));
}

#[test]
fn test_worktree_backend_config_shared_across_worktrees() {
    // Verify that config_root is at the container level, shared across all worktrees
    let temp = setup_worktree_container();

    // Create a feature worktree
    fs::create_dir(temp.path().join("feature-x")).unwrap();
    fs::write(
        temp.path().join("feature-x/.git"),
        format!("gitdir: {}/.gt/worktrees/feature-x", temp.path().display()),
    )
    .unwrap();

    let container = NormalizedPath::new(temp.path());
    let main_backend = WorktreeBackend::new(container.clone()).unwrap();
    let feature_backend = WorktreeBackend::with_worktree(
        container,
        NormalizedPath::new(temp.path().join("feature-x")),
    )
    .unwrap();

    // Both backends must share the same config root (container/.repository)
    assert_eq!(
        main_backend.config_root().as_str(),
        feature_backend.config_root().as_str(),
        "Config root must be shared across all worktrees in the same container"
    );

    // But working directories must differ
    assert_ne!(
        main_backend.working_dir().as_str(),
        feature_backend.working_dir().as_str(),
        "Working directories must be different for different worktrees"
    );
}
