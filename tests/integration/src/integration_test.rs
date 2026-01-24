//! End-to-end integration test for the vertical slice
//!
//! This test exercises the complete flow: config loading -> preset check -> tool sync.

use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_meta::{Registry, load_config};
use repo_presets::{Context, PresetProvider, PresetStatus, UvProvider};
use repo_tools::{
    ClaudeIntegration, CursorIntegration, Rule, SyncContext, ToolIntegration, VSCodeIntegration,
};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Set up a test repository with a valid config.toml
fn setup_test_repo() -> TempDir {
    let temp = TempDir::new().unwrap();
    let repo_dir = temp.path().join(".repository");
    fs::create_dir(&repo_dir).unwrap();

    fs::write(
        repo_dir.join("config.toml"),
        r#"
[core]
version = "1.0"
mode = "standard"

[active]
tools = ["vscode", "cursor", "claude"]
presets = ["env:python"]

["env:python"]
provider = "uv"
version = "3.12"
"#,
    )
    .unwrap();

    temp
}

#[test]
fn test_load_config_and_registry() {
    let temp = setup_test_repo();
    let root = NormalizedPath::new(temp.path());

    // Load configuration from .repository/config.toml
    let config = load_config(&root).unwrap();
    assert_eq!(config.active.presets, vec!["env:python"]);
    assert_eq!(config.active.tools, vec!["vscode", "cursor", "claude"]);

    // Registry should have builtin providers
    let registry = Registry::with_builtins();
    assert!(registry.has_provider("env:python"));
    assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));
}

#[tokio::test]
async fn test_python_provider_check() {
    let temp = setup_test_repo();
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };

    let context = Context::new(layout, HashMap::new());
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();
    // Should be Missing or Broken (depending on whether uv is installed)
    // Either way, it won't be Healthy since no venv exists
    assert!(report.status != PresetStatus::Healthy);
}

#[test]
fn test_tool_sync_creates_files() {
    let temp = setup_test_repo();
    let root = NormalizedPath::new(temp.path());

    let rules = vec![Rule {
        id: "python-style".to_string(),
        content: "Use snake_case for variables".to_string(),
    }];

    let python_path = if cfg!(windows) {
        root.join(".venv/Scripts/python.exe")
    } else {
        root.join(".venv/bin/python")
    };

    let context = SyncContext::new(root.clone()).with_python(python_path);

    // Sync all three tools
    VSCodeIntegration::new().sync(&context, &rules).unwrap();
    CursorIntegration::new().sync(&context, &rules).unwrap();
    ClaudeIntegration::new().sync(&context, &rules).unwrap();

    // Verify all files were created
    assert!(temp.path().join(".vscode/settings.json").exists());
    assert!(temp.path().join(".cursorrules").exists());
    assert!(temp.path().join("CLAUDE.md").exists());

    // Verify content for cursor and claude (they use managed blocks)
    let cursorrules = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
    assert!(cursorrules.contains("python-style"));
    assert!(cursorrules.contains("snake_case"));

    let claude_md = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
    assert!(claude_md.contains("python-style"));
    assert!(claude_md.contains("snake_case"));
}

#[test]
fn test_full_vertical_slice() {
    let temp = setup_test_repo();
    let root = NormalizedPath::new(temp.path());

    // 1. Load config
    let config = load_config(&root).unwrap();
    assert!(config.active.presets.contains(&"env:python".to_string()));
    assert!(config.active.tools.contains(&"vscode".to_string()));
    assert!(config.active.tools.contains(&"cursor".to_string()));
    assert!(config.active.tools.contains(&"claude".to_string()));

    // 2. Registry lookup
    let registry = Registry::with_builtins();
    assert!(registry.has_provider("env:python"));
    assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));

    // 3. Create rules and sync context
    let rules = vec![Rule {
        id: "test-rule".to_string(),
        content: "Test content for vertical slice".to_string(),
    }];
    let context = SyncContext::new(root.clone());

    // 4. Sync tools based on config
    for tool_name in &config.active.tools {
        match tool_name.as_str() {
            "vscode" => VSCodeIntegration::new().sync(&context, &rules).unwrap(),
            "cursor" => CursorIntegration::new().sync(&context, &rules).unwrap(),
            "claude" => ClaudeIntegration::new().sync(&context, &rules).unwrap(),
            _ => {}
        }
    }

    // 5. Verify all tool configs were created
    assert!(temp.path().join(".vscode/settings.json").exists());
    assert!(temp.path().join(".cursorrules").exists());
    assert!(temp.path().join("CLAUDE.md").exists());

    // 6. Verify managed blocks in cursor and claude files
    let cursorrules = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
    assert!(cursorrules.contains("<!-- repo:block:test-rule -->"));
    assert!(cursorrules.contains("Test content for vertical slice"));
    assert!(cursorrules.contains("<!-- /repo:block:test-rule -->"));

    let claude_md = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
    assert!(claude_md.contains("<!-- repo:block:test-rule -->"));
    assert!(claude_md.contains("Test content for vertical slice"));
    assert!(claude_md.contains("<!-- /repo:block:test-rule -->"));
}

#[test]
fn test_config_with_preset_options() {
    let temp = TempDir::new().unwrap();
    let repo_dir = temp.path().join(".repository");
    fs::create_dir(&repo_dir).unwrap();

    // Config with preset-specific options
    fs::write(
        repo_dir.join("config.toml"),
        r#"
[core]
version = "1"
mode = "worktrees"

[active]
tools = ["vscode"]
presets = ["env:python"]

[sync]
strategy = "manual"

["env:python"]
provider = "uv"
version = "3.11"
"#,
    )
    .unwrap();

    let root = NormalizedPath::new(temp.path());
    let config = load_config(&root).unwrap();

    // Verify core config
    assert_eq!(config.core.mode, repo_meta::RepositoryMode::Worktrees);

    // Verify sync config
    assert_eq!(config.sync.strategy, "manual");

    // Verify presets config is captured
    let python_config = config.presets_config.get("env:python").unwrap();
    assert_eq!(python_config.get("version").unwrap().as_str(), Some("3.11"));
    assert_eq!(python_config.get("provider").unwrap().as_str(), Some("uv"));
}

#[test]
fn test_registry_builtin_providers() {
    let registry = Registry::with_builtins();

    // env:python should be mapped to uv provider
    assert!(registry.has_provider("env:python"));
    assert_eq!(registry.get_provider("env:python"), Some(&"uv".to_string()));

    // Unknown presets should return None
    assert!(!registry.has_provider("env:node"));
    assert_eq!(registry.get_provider("env:node"), None);
}

#[test]
fn test_tool_integration_names_and_paths() {
    let vscode = VSCodeIntegration::new();
    assert_eq!(vscode.name(), "vscode");
    assert_eq!(vscode.config_paths(), vec![".vscode/settings.json"]);

    let cursor = CursorIntegration::new();
    assert_eq!(cursor.name(), "cursor");
    assert_eq!(cursor.config_paths(), vec![".cursorrules"]);

    let claude = ClaudeIntegration::new();
    assert_eq!(claude.name(), "claude");
    assert_eq!(claude.config_paths(), vec!["CLAUDE.md", ".claude/rules/"]);
}
