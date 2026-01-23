//! Integration tests for VSCode integration.

use repo_fs::NormalizedPath;
use repo_tools::{Rule, SyncContext, ToolIntegration, VSCodeIntegration};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_vscode_name() {
    let integration = VSCodeIntegration::new();
    assert_eq!(integration.name(), "vscode");
}

#[test]
fn test_vscode_config_paths() {
    let integration = VSCodeIntegration::new();
    let paths = integration.config_paths();
    assert_eq!(paths, vec![".vscode/settings.json"]);
}

#[test]
fn test_vscode_creates_settings_directory() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());
    let python_path = NormalizedPath::new("/usr/bin/python3");

    let context = SyncContext::new(root).with_python(python_path);
    let integration = VSCodeIntegration::new();

    integration.sync(&context, &[]).unwrap();

    // Verify .vscode directory was created
    assert!(temp_dir.path().join(".vscode").exists());
    assert!(temp_dir.path().join(".vscode").is_dir());
}

#[test]
fn test_vscode_creates_settings_json() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());
    let python_path = NormalizedPath::new("/my/project/.venv/bin/python");

    let context = SyncContext::new(root).with_python(python_path);
    let integration = VSCodeIntegration::new();

    integration.sync(&context, &[]).unwrap();

    let settings_path = temp_dir.path().join(".vscode/settings.json");
    assert!(settings_path.exists());

    let content = fs::read_to_string(&settings_path).unwrap();
    let settings: Value = serde_json::from_str(&content).unwrap();

    assert_eq!(
        settings["python.defaultInterpreterPath"],
        "/my/project/.venv/bin/python"
    );
}

#[test]
fn test_vscode_sets_python_path() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    // First sync without python path
    let context = SyncContext::new(root.clone());
    let integration = VSCodeIntegration::new();
    integration.sync(&context, &[]).unwrap();

    // Verify no python path initially
    let content = fs::read_to_string(temp_dir.path().join(".vscode/settings.json")).unwrap();
    let settings: Value = serde_json::from_str(&content).unwrap();
    assert!(settings["python.defaultInterpreterPath"].is_null());

    // Now sync with python path
    let python_path = NormalizedPath::new("/updated/python");
    let context = SyncContext::new(root).with_python(python_path);
    integration.sync(&context, &[]).unwrap();

    // Verify python path is set
    let content = fs::read_to_string(temp_dir.path().join(".vscode/settings.json")).unwrap();
    let settings: Value = serde_json::from_str(&content).unwrap();
    assert_eq!(settings["python.defaultInterpreterPath"], "/updated/python");
}

#[test]
fn test_vscode_preserves_existing_settings() {
    let temp_dir = TempDir::new().unwrap();
    let vscode_dir = temp_dir.path().join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap();

    // Create existing settings with various configurations
    let existing = serde_json::json!({
        "editor.fontSize": 16,
        "editor.tabSize": 4,
        "files.autoSave": "onFocusChange",
        "rust-analyzer.cargo.features": ["all"]
    });
    fs::write(
        vscode_dir.join("settings.json"),
        serde_json::to_string_pretty(&existing).unwrap(),
    )
    .unwrap();

    let root = NormalizedPath::new(temp_dir.path());
    let python_path = NormalizedPath::new("/venv/python");
    let context = SyncContext::new(root).with_python(python_path);

    let integration = VSCodeIntegration::new();
    integration.sync(&context, &[]).unwrap();

    let content = fs::read_to_string(vscode_dir.join("settings.json")).unwrap();
    let settings: Value = serde_json::from_str(&content).unwrap();

    // Verify all existing settings are preserved
    assert_eq!(settings["editor.fontSize"], 16);
    assert_eq!(settings["editor.tabSize"], 4);
    assert_eq!(settings["files.autoSave"], "onFocusChange");
    assert_eq!(settings["rust-analyzer.cargo.features"][0], "all");

    // Verify new setting is added
    assert_eq!(settings["python.defaultInterpreterPath"], "/venv/python");
}

#[test]
fn test_vscode_updates_existing_python_path() {
    let temp_dir = TempDir::new().unwrap();
    let vscode_dir = temp_dir.path().join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap();

    // Create settings with existing python path
    let existing = serde_json::json!({
        "python.defaultInterpreterPath": "/old/python"
    });
    fs::write(
        vscode_dir.join("settings.json"),
        serde_json::to_string_pretty(&existing).unwrap(),
    )
    .unwrap();

    let root = NormalizedPath::new(temp_dir.path());
    let python_path = NormalizedPath::new("/new/python");
    let context = SyncContext::new(root).with_python(python_path);

    let integration = VSCodeIntegration::new();
    integration.sync(&context, &[]).unwrap();

    let content = fs::read_to_string(vscode_dir.join("settings.json")).unwrap();
    let settings: Value = serde_json::from_str(&content).unwrap();

    // Verify python path is updated
    assert_eq!(settings["python.defaultInterpreterPath"], "/new/python");
}

#[test]
fn test_vscode_rules_are_ignored() {
    // VSCode integration doesn't use rules, but should not fail when provided
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());
    let python_path = NormalizedPath::new("/python");

    let context = SyncContext::new(root).with_python(python_path);
    let rules = vec![Rule {
        id: "ignored-rule".to_string(),
        content: "This should be ignored".to_string(),
    }];

    let integration = VSCodeIntegration::new();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join(".vscode/settings.json")).unwrap();

    // Rules should not appear in settings
    assert!(!content.contains("ignored-rule"));
    assert!(!content.contains("This should be ignored"));
}
