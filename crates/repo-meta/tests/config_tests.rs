//! Integration tests for configuration loading

use repo_fs::NormalizedPath;
use repo_meta::config::{RepositoryMode, get_preset_config, load_config};
use std::fs;
use tempfile::TempDir;

fn setup_config_file(temp: &TempDir, content: &str) -> NormalizedPath {
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();
    let config_path = repo_dir.join("config.toml");
    fs::write(&config_path, content).unwrap();
    NormalizedPath::new(temp.path())
}

#[test]
fn test_load_minimal_config() {
    let temp = TempDir::new().unwrap();
    let config_content = r#"
[core]
version = "1"
"#;
    let root = setup_config_file(&temp, config_content);

    let config = load_config(&root).unwrap();

    assert_eq!(config.core.version, "1");
    assert_eq!(config.core.mode, RepositoryMode::Standard);
    assert!(config.active.tools.is_empty());
    assert!(config.active.presets.is_empty());
}

#[test]
fn test_load_config_with_worktrees_mode() {
    let temp = TempDir::new().unwrap();
    let config_content = r#"
[core]
version = "1"
mode = "worktrees"
"#;
    let root = setup_config_file(&temp, config_content);

    let config = load_config(&root).unwrap();

    assert_eq!(config.core.mode, RepositoryMode::Worktrees);
}

#[test]
fn test_load_config_with_presets() {
    let temp = TempDir::new().unwrap();
    let config_content = r#"
[core]
version = "1"

[active]
tools = ["vscode", "cursor"]
presets = ["env:python", "env:node"]

[sync]
strategy = "on-commit"

["env:python"]
version = "3.11"
packages = ["pytest", "black"]

["env:node"]
version = "20"
"#;
    let root = setup_config_file(&temp, config_content);

    let config = load_config(&root).unwrap();

    // Check active section
    assert_eq!(config.active.tools, vec!["vscode", "cursor"]);
    assert_eq!(config.active.presets, vec!["env:python", "env:node"]);

    // Check sync section
    assert_eq!(config.sync.strategy, "on-commit");

    // Check preset configs
    let python_config = get_preset_config(&config, "env:python").unwrap();
    assert_eq!(python_config.get("version").unwrap().as_str(), Some("3.11"));

    let packages = python_config.get("packages").unwrap().as_array().unwrap();
    assert_eq!(packages.len(), 2);

    let node_config = get_preset_config(&config, "env:node").unwrap();
    assert_eq!(node_config.get("version").unwrap().as_str(), Some("20"));
}

#[test]
fn test_config_not_found_error() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());

    let result = load_config(&root);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("not found"),
        "Error should mention 'not found': {}",
        err_str
    );
}

#[test]
fn test_get_preset_config_returns_none_for_unknown() {
    let temp = TempDir::new().unwrap();
    let config_content = r#"
[core]
version = "1"
"#;
    let root = setup_config_file(&temp, config_content);

    let config = load_config(&root).unwrap();
    let result = get_preset_config(&config, "unknown:preset");

    assert!(result.is_none());
}

#[test]
fn test_load_config_with_all_defaults() {
    let temp = TempDir::new().unwrap();
    // Minimal config with all sections present but empty
    // This triggers default values for each field
    let config_content = r#"
[core]
[active]
[sync]
"#;
    let root = setup_config_file(&temp, config_content);

    let config = load_config(&root).unwrap();

    // All defaults should be applied
    assert_eq!(config.core.version, "1");
    assert_eq!(config.core.mode, RepositoryMode::Standard);
    assert!(config.active.tools.is_empty());
    assert!(config.active.presets.is_empty());
    assert_eq!(config.sync.strategy, "auto");
}

#[test]
fn test_load_empty_config_file() {
    let temp = TempDir::new().unwrap();
    // Completely empty config file - tests that struct defaults work
    let config_content = "";
    let root = setup_config_file(&temp, config_content);

    let config = load_config(&root).unwrap();

    // Core defaults should apply even when section is missing
    assert_eq!(config.core.version, "1");
    assert_eq!(config.core.mode, RepositoryMode::Standard);
    assert!(config.active.tools.is_empty());
    assert!(config.active.presets.is_empty());
}
