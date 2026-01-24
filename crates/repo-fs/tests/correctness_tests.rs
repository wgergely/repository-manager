use assert_fs::prelude::*;
use repo_fs::{LayoutMode, WorkspaceLayout};

#[test]
fn test_detect_classic_from_subdir() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".git").touch().unwrap();
    let subdir = temp.child("src").child("core");
    subdir.create_dir_all().unwrap();

    let layout = WorkspaceLayout::detect(subdir.path()).expect("Should detect layout");
    assert_eq!(layout.mode, LayoutMode::Classic);
    assert_eq!(
        layout.root.as_str(),
        temp.path().to_str().unwrap().replace('\\', "/")
    );
}

#[test]
fn test_detect_container_from_subdir() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".gt").create_dir_all().unwrap();
    temp.child("main").create_dir_all().unwrap();

    let subdir = temp.child("main").child("src");
    subdir.create_dir_all().unwrap();

    let layout = WorkspaceLayout::detect(subdir.path()).expect("Should detect container layout");
    assert_eq!(layout.mode, LayoutMode::Container);
    // Root should be the container root (temp), not main
    assert_eq!(
        layout.root.as_str(),
        temp.path().to_str().unwrap().replace('\\', "/")
    );

    // In Container mode, we might expect active_context to be different?
    // detect code says:
    // root: NormalizedPath::new(dir), active_context: NormalizedPath::new(dir)
    // where `dir` is the detected root.
    // So context is reset to root?
    // Let's verify what the code does.
    // Code says: `Ok(mode.map(|mode| Self { root: ... dir, active_context: ... dir }))`
    // Yes, detect returns root as context. This seems intended for discovery.
}

#[test]
fn test_validate_container_success() {
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".gt").create_dir_all().unwrap();
    temp.child("main").create_dir_all().unwrap();

    let layout = WorkspaceLayout::detect(temp.path()).unwrap();
    assert!(layout.validate().is_ok());
}

#[test]
fn test_validate_container_missing_gt() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Create markers to trick detection, then remove one?
    // Or construct manually.
    // Detection requires them to exist.
    // So if detection succeeds, validation should succeed usually.
    // BUT what if we detect, then delete?

    temp.child(".gt").create_dir_all().unwrap();
    temp.child("main").create_dir_all().unwrap();

    let layout = WorkspaceLayout::detect(temp.path()).unwrap();

    // Now delete .gt
    std::fs::remove_dir(temp.child(".gt").path()).unwrap();

    let res = layout.validate();
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Git database missing")
    );
}

#[test]
fn test_precedence_container_over_classic() {
    // If we have .gt, main AND .git (weird hybrid)
    // Container should win
    let temp = assert_fs::TempDir::new().unwrap();
    temp.child(".gt").create_dir_all().unwrap();
    temp.child("main").create_dir_all().unwrap();
    temp.child(".git").create_dir_all().unwrap();

    let layout = WorkspaceLayout::detect(temp.path()).unwrap();
    assert_eq!(layout.mode, LayoutMode::Container);
    assert_eq!(
        layout.git_database().as_str(),
        temp.child(".gt")
            .path()
            .to_str()
            .unwrap()
            .replace('\\', "/")
    );
}
