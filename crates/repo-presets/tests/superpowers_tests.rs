//! Integration tests for SuperpowersProvider
//!
//! Note: Tests that require network access are marked with #[ignore]
//! Run with: cargo test -p repo-presets --test superpowers_tests -- --ignored

use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::{Context, PresetProvider, PresetStatus, SuperpowersProvider};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_context(temp: &TempDir) -> Context {
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };
    Context::new(layout, HashMap::new())
}

#[tokio::test]
async fn test_provider_id() {
    let provider = SuperpowersProvider::new();
    assert_eq!(provider.id(), "claude:superpowers");
}

#[tokio::test]
async fn test_check_when_not_installed() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = SuperpowersProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Should be Missing since we haven't installed
    assert_eq!(report.status, PresetStatus::Missing);
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_install_and_uninstall() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    // Use a test version
    let provider = SuperpowersProvider::new().with_version("v4.1.1");

    // Install
    let report = provider.apply(&context).await.unwrap();
    assert!(report.success, "Install failed: {:?}", report.errors);

    // Check should show healthy
    let check = provider.check(&context).await.unwrap();
    assert_eq!(check.status, PresetStatus::Healthy);

    // Uninstall
    let report = provider.uninstall(&context).await.unwrap();
    assert!(report.success, "Uninstall failed: {:?}", report.errors);

    // Check should show missing
    let check = provider.check(&context).await.unwrap();
    assert_eq!(check.status, PresetStatus::Missing);
}
