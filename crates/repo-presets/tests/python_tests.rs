//! Integration tests for Python providers

mod common;

use common::create_test_context;
use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::PresetStatus;
use repo_presets::context::Context;
use repo_presets::provider::PresetProvider;
use repo_presets::python::UvProvider;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

fn create_context_with_config(temp: &TempDir, config: HashMap<String, toml::Value>) -> Context {
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };
    Context::new(layout, config)
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
async fn test_uv_check_empty_dir_reports_not_healthy() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Empty dir should be Missing (no venv) or Broken (uv not installed)
    assert!(
        report.status != PresetStatus::Healthy,
        "Empty directory must not report healthy, got {:?}",
        report.status
    );

    // Report must have actionable details
    assert!(
        !report.details.is_empty(),
        "Check report must include details explaining why it's not healthy"
    );
}

#[tokio::test]
async fn test_uv_check_with_fake_venv_structure() {
    // Simulate a partial venv (directory exists but no python binary)
    let temp = TempDir::new().unwrap();
    let venv_dir = temp.path().join(".venv");
    fs::create_dir_all(&venv_dir).unwrap();

    let context = create_test_context(&temp);
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();

    // A .venv dir without a python binary should NOT be Healthy
    assert_ne!(
        report.status,
        PresetStatus::Healthy,
        "Partial venv (dir but no python binary) must not be healthy"
    );
}

#[tokio::test]
async fn test_uv_check_with_complete_venv_structure() {
    // Simulate a complete venv with the expected python binary
    let temp = TempDir::new().unwrap();
    let venv_dir = temp.path().join(".venv");

    if cfg!(windows) {
        let scripts_dir = venv_dir.join("Scripts");
        fs::create_dir_all(&scripts_dir).unwrap();
        fs::write(scripts_dir.join("python.exe"), "fake").unwrap();
    } else {
        let bin_dir = venv_dir.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("python"), "fake").unwrap();
    }

    let context = create_test_context(&temp);
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();

    // With a complete fake venv, the check depends on whether uv is installed.
    // If uv is available, the venv check passes -> Healthy.
    // If uv is not available, Broken (because uv binary is missing).
    // Either way, it should NOT be Missing (the venv exists).
    if report.status == PresetStatus::Broken {
        assert!(
            report.details.iter().any(|d| d.contains("uv")),
            "Broken status must explain that uv is missing, got: {:?}",
            report.details
        );
    } else {
        assert_eq!(
            report.status,
            PresetStatus::Healthy,
            "With complete venv and uv available, should be healthy"
        );
    }
}

#[tokio::test]
async fn test_context_venv_path() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    let venv_path = context.venv_path();
    assert!(
        venv_path.as_str().ends_with(".venv"),
        "Default venv path should end with .venv, got: {}",
        venv_path
    );
}

#[tokio::test]
async fn test_context_venv_path_with_tag() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp).with_venv_tag("main-win-py312");

    let venv_path = context.venv_path();
    assert!(
        venv_path.as_str().ends_with(".venv-main-win-py312"),
        "Tagged venv path should end with .venv-main-win-py312, got: {}",
        venv_path
    );
}

#[tokio::test]
async fn test_context_default_python_version() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    assert_eq!(context.python_version(), "3.12");
}

#[tokio::test]
async fn test_context_custom_python_version() {
    let temp = TempDir::new().unwrap();

    let mut config = HashMap::new();
    config.insert(
        "version".to_string(),
        toml::Value::String("3.11".to_string()),
    );

    let context = create_context_with_config(&temp, config);
    assert_eq!(context.python_version(), "3.11");
}

#[tokio::test]
async fn test_context_default_provider() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    assert_eq!(context.provider(), "uv");
}

#[tokio::test]
async fn test_uv_check_report_details_are_actionable() {
    // Verify that check reports contain useful details, not empty strings
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();

    for detail in &report.details {
        assert!(
            !detail.is_empty(),
            "Check report details must not contain empty strings"
        );
        assert!(
            detail.len() > 5,
            "Check report details should be descriptive, got: '{}'",
            detail
        );
    }
}

#[tokio::test]
async fn test_uv_check_with_project_files_but_no_venv() {
    // A project with Python files but no venv should still report missing/broken
    let temp = TempDir::new().unwrap();

    // Create typical Python project structure
    fs::write(
        temp.path().join("requirements.txt"),
        "requests==2.31.0\nflask==3.0.0\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("pyproject.toml"),
        "[project]\nname = \"test-project\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(
        temp.path().join("src/main.py"),
        "def main():\n    print('hello')\n",
    )
    .unwrap();

    let context = create_test_context(&temp);
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();

    // Even with Python project files, no venv means not healthy
    assert_ne!(
        report.status,
        PresetStatus::Healthy,
        "Project with Python files but no venv must not be healthy"
    );
}
