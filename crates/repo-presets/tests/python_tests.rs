//! Integration tests for Python providers

use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::PresetStatus;
use repo_presets::context::Context;
use repo_presets::provider::PresetProvider;
use repo_presets::python::UvProvider;
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
async fn test_uv_provider_id() {
    let provider = UvProvider::new();
    assert_eq!(provider.id(), "env:python");
}

#[tokio::test]
async fn test_uv_provider_default() {
    let provider = UvProvider;
    assert_eq!(provider.id(), "env:python");
}

#[tokio::test]
async fn test_uv_check_missing_venv() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Should be Missing or Broken (if uv not installed)
    assert!(
        report.status != PresetStatus::Healthy,
        "Expected non-healthy status, got {:?}",
        report.status
    );
}

#[tokio::test]
async fn test_context_venv_path() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    let venv_path = context.venv_path();
    assert!(venv_path.as_str().ends_with(".venv"));
}

#[tokio::test]
async fn test_context_default_python_version() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    // Default should be 3.12
    assert_eq!(context.python_version(), "3.12");
}

#[tokio::test]
async fn test_context_custom_python_version() {
    let temp = TempDir::new().unwrap();
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };

    let mut config = HashMap::new();
    config.insert(
        "version".to_string(),
        toml::Value::String("3.11".to_string()),
    );

    let context = Context::new(layout, config);
    assert_eq!(context.python_version(), "3.11");
}

#[tokio::test]
async fn test_context_default_provider() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    // Default provider should be uv
    assert_eq!(context.provider(), "uv");
}
