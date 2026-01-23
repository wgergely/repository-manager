use repo_fs::{LayoutMode, RepoPath, WorkspaceLayout};
use assert_fs::prelude::*;

#[test]
fn snapshot_container_layout_detection() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    // Setup a complex container layout
    temp.child(RepoPath::GtDir.as_str()).create_dir_all().unwrap();
    temp.child(RepoPath::MainWorktree.as_str()).create_dir_all().unwrap();
    temp.child(RepoPath::RepositoryConfig.as_str()).create_dir_all().unwrap();
    
    // We want to verify how the detected layout looks in structure
    let layout = WorkspaceLayout::detect(temp.path()).unwrap();
    
    // We snapshot the Debug output or a custom struct representation.
    // Since NormalizedPath contains absolute paths which vary by run (temp dir),
    // we need to be careful. Insta supports redactions or we can map to relative paths.
    // For now, let's snapshot the mode and verifying the relative path logic if we had it.
    // 
    // To make this stable, we'll verify the properties that SHOULD be invariant,
    // or mask the root path.
    
    // Use the actual detected root for replacement to ensure matching works
    // even if canonicalization changed the path string slightly.
    let root_str = layout.root.as_str();
    
    let debug_view = format!("{:?}", layout);
    // Mask the volatile temp path part
    let sanitized_view = debug_view.replace(root_str, "[ROOT]");
    let sanitized_view = sanitized_view.replace("\\", "/"); // Normalize windows slashes in debug output if any

    insta::assert_snapshot!(sanitized_view, @r###"WorkspaceLayout { root: NormalizedPath { inner: "[ROOT]" }, active_context: NormalizedPath { inner: "[ROOT]" }, mode: Container }"###);
}

#[test]
fn snapshot_in_repo_worktrees_layout_detection() {
    let temp = assert_fs::TempDir::new().unwrap();
    
    temp.child(RepoPath::GitDir.as_str()).create_dir_all().unwrap();
    temp.child(RepoPath::WorktreesDir.as_str()).create_dir_all().unwrap();

    let layout = WorkspaceLayout::detect(temp.path()).unwrap();
    
    let root_str = layout.root.as_str();
    let debug_view = format!("{:?}", layout);
    let sanitized_view = debug_view.replace(root_str, "[ROOT]");
    let sanitized_view = sanitized_view.replace("\\", "/");

    insta::assert_snapshot!(sanitized_view, @r###"WorkspaceLayout { root: NormalizedPath { inner: "[ROOT]" }, active_context: NormalizedPath { inner: "[ROOT]" }, mode: InRepoWorktrees }"###);
}
