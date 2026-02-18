//! Integration tests for PluginsProvider
//!
//! Note: Tests that require network access are marked with #[ignore]
//! Run with: cargo test -p repo-presets --test plugins_tests -- --ignored

use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::{Context, PluginsProvider, PresetProvider, PresetStatus};
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
    let provider = PluginsProvider::new();
    assert_eq!(provider.id(), "claude:plugins");
}

#[tokio::test]
async fn test_check_when_not_installed() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = PluginsProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Should be Missing since we haven't installed, and details should explain why
    assert_eq!(report.status, PresetStatus::Missing);
    assert!(
        !report.details.is_empty(),
        "Missing status should include details about what's missing"
    );
    assert!(
        report.details[0].contains("not installed"),
        "Details should mention 'not installed', got: {:?}",
        report.details
    );
}

#[tokio::test]
async fn test_check_report_includes_version_info() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = PluginsProvider::new().with_version("v4.1.1");

    let report = provider.check(&context).await.unwrap();

    // When missing, details should reference the specific version
    if report.status == PresetStatus::Missing {
        assert!(
            report.details.iter().any(|d| d.contains("v4.1.1")),
            "Missing report should reference the requested version, got: {:?}",
            report.details
        );
    }
}

#[tokio::test]
async fn test_with_version_changes_version() {
    let provider = PluginsProvider::new().with_version("v4.0.0");
    assert_eq!(provider.version, "v4.0.0");

    let provider = provider.with_version("v5.0.0");
    assert_eq!(provider.version, "v5.0.0");
}

#[tokio::test]
async fn test_uninstall_when_not_installed_succeeds() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = PluginsProvider::new();

    // Uninstalling when nothing is installed should succeed gracefully
    let report = provider.uninstall(&context).await.unwrap();
    assert!(
        report.success,
        "Uninstall should succeed even if nothing is installed"
    );
}

#[tokio::test]
#[ignore] // Requires network access
async fn test_install_and_uninstall() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    // Use a test version
    let provider = PluginsProvider::new().with_version("v4.1.1");

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
