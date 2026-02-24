//! Tests for the SyncEngine

use pretty_assertions::assert_eq;
use repo_core::Mode;
use repo_core::ledger::{Intent, Ledger, Projection};
use repo_core::sync::{CheckReport, CheckStatus, DriftItem, SyncEngine};
use repo_fs::NormalizedPath;
use serde_json::json;
use std::fs;
use repo_test_utils::git::fake_git_dir;
use tempfile::TempDir;
use uuid::Uuid;

fn setup_git_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    fake_git_dir(dir.path());
    dir
}

#[test]
fn test_sync_engine_check_empty() {
    // An empty ledger should report healthy status
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Healthy);
    assert!(report.drifted.is_empty());
    assert!(report.missing.is_empty());
}

#[test]
fn test_sync_engine_sync_creates_ledger() {
    // Sync should create a ledger file if it doesn't exist
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();
    let ledger_path = engine.ledger_path();

    // Ledger file shouldn't exist yet
    assert!(!ledger_path.exists());

    // Run sync
    let report = engine.sync().unwrap();
    assert!(report.success);

    // Now ledger file should exist
    assert!(ledger_path.exists());
}

#[test]
fn test_check_detects_drift_file_managed_missing() {
    // When a file-managed projection references a file that doesn't exist,
    // check should report it as missing
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create .repository directory and ledger with a projection
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    // Add a projection for a file that doesn't exist
    intent.add_projection(Projection::file_managed(
        "test-tool".to_string(),
        std::path::PathBuf::from("config/nonexistent.json"),
        "abc123".to_string(),
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Missing);
    assert_eq!(report.missing.len(), 1);
    assert_eq!(report.missing[0].tool, "test-tool");
    assert!(report.missing[0].file.contains("nonexistent.json"));
}

#[test]
fn test_check_detects_drift_file_managed_checksum_mismatch() {
    // When a file-managed projection has a checksum that doesn't match,
    // check should report it as drifted
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create the config file with some content
    let config_dir = temp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("managed.json"), r#"{"key": "value"}"#).unwrap();

    // Create .repository directory and ledger with a projection
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    // Add a projection with a wrong checksum
    intent.add_projection(Projection::file_managed(
        "test-tool".to_string(),
        std::path::PathBuf::from("config/managed.json"),
        "wrong-checksum".to_string(),
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Drifted);
    assert_eq!(report.drifted.len(), 1);
    assert_eq!(report.drifted[0].tool, "test-tool");
}

#[test]
fn test_check_healthy_when_file_managed_matches() {
    // When a file-managed projection checksum matches, status should be healthy
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create the config file with some content
    let config_dir = temp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let content = r#"{"key": "value"}"#;
    fs::write(config_dir.join("managed.json"), content).unwrap();

    // Compute the correct checksum using canonical format
    let checksum = repo_fs::checksum::compute_content_checksum(content);

    // Create .repository directory and ledger with correct checksum
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    intent.add_projection(Projection::file_managed(
        "test-tool".to_string(),
        std::path::PathBuf::from("config/managed.json"),
        checksum,
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Healthy);
    assert!(report.drifted.is_empty());
    assert!(report.missing.is_empty());
}

#[test]
fn test_check_detects_text_block_marker_missing() {
    // When a text-block projection references a file that doesn't contain the marker,
    // check should report it as missing
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create the config file without the marker
    let config_dir = temp.path().join(".cursor").join("rules");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("test.mdc"),
        "# Some content without marker\n",
    )
    .unwrap();

    let marker = Uuid::new_v4();

    // Create .repository directory and ledger
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    intent.add_projection(Projection::text_block(
        "cursor".to_string(),
        std::path::PathBuf::from(".cursor/rules/test.mdc"),
        marker,
        "checksum".to_string(),
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    // Should be missing since marker UUID is not in the file
    assert_eq!(report.status, CheckStatus::Missing);
    assert_eq!(report.missing.len(), 1);
}

#[test]
fn test_check_healthy_when_text_block_marker_present() {
    // When a text-block projection marker is found in the file, status should be healthy
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    let marker = Uuid::new_v4();

    // Create the config file with the marker
    let config_dir = temp.path().join(".cursor").join("rules");
    fs::create_dir_all(&config_dir).unwrap();
    let text_content = format!(
        "# Some content\n<!-- BEGIN {} -->\nblock content\n<!-- END {} -->\n",
        marker, marker
    );
    fs::write(config_dir.join("test.mdc"), &text_content).unwrap();

    // Compute real checksum for the text block content
    let text_checksum = repo_fs::checksum::compute_content_checksum(&text_content);

    // Create .repository directory and ledger
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    intent.add_projection(Projection::text_block(
        "cursor".to_string(),
        std::path::PathBuf::from(".cursor/rules/test.mdc"),
        marker,
        text_checksum,
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Healthy);
}

#[test]
fn test_check_detects_json_key_missing_file() {
    // When a json-key projection references a file that doesn't exist
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create .repository directory and ledger
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    intent.add_projection(Projection::json_key(
        "vscode".to_string(),
        std::path::PathBuf::from(".vscode/settings.json"),
        "editor.fontSize".to_string(),
        json!(14),
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Missing);
    assert_eq!(report.missing.len(), 1);
}

#[test]
fn test_check_detects_json_key_wrong_value() {
    // When a json-key projection has a different value in the file
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create the config file with different value
    let vscode_dir = temp.path().join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap();
    fs::write(
        vscode_dir.join("settings.json"),
        r#"{"editor": {"fontSize": 12}}"#,
    )
    .unwrap();

    // Create .repository directory and ledger expecting fontSize=14
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    intent.add_projection(Projection::json_key(
        "vscode".to_string(),
        std::path::PathBuf::from(".vscode/settings.json"),
        "editor.fontSize".to_string(),
        json!(14),
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Drifted);
    assert_eq!(report.drifted.len(), 1);
}

#[test]
fn test_check_healthy_when_json_key_matches() {
    // When a json-key projection value matches
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create the config file with matching value
    let vscode_dir = temp.path().join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap();
    fs::write(
        vscode_dir.join("settings.json"),
        r#"{"editor": {"fontSize": 14}}"#,
    )
    .unwrap();

    // Create .repository directory and ledger
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let mut ledger = Ledger::new();
    let mut intent = Intent::new("rule:test".to_string(), json!({}));

    intent.add_projection(Projection::json_key(
        "vscode".to_string(),
        std::path::PathBuf::from(".vscode/settings.json"),
        "editor.fontSize".to_string(),
        json!(14),
    ));
    ledger.add_intent(intent);
    ledger.save(&repo_dir.join("ledger.toml")).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.check().unwrap();

    assert_eq!(report.status, CheckStatus::Healthy);
}

#[test]
fn test_check_report_constructors() {
    // Test CheckReport constructors
    let healthy = CheckReport::healthy();
    assert_eq!(healthy.status, CheckStatus::Healthy);
    assert!(healthy.drifted.is_empty());
    assert!(healthy.missing.is_empty());

    let missing_item = DriftItem {
        intent_id: "test-intent".to_string(),
        tool: "vscode".to_string(),
        file: ".vscode/settings.json".to_string(),
        description: "File not found".to_string(),
    };
    let with_missing = CheckReport::with_missing(vec![missing_item.clone()]);
    assert_eq!(with_missing.status, CheckStatus::Missing);
    assert_eq!(with_missing.missing.len(), 1);

    let drifted_item = DriftItem {
        intent_id: "test-intent".to_string(),
        tool: "vscode".to_string(),
        file: ".vscode/settings.json".to_string(),
        description: "Checksum mismatch".to_string(),
    };
    let with_drifted = CheckReport::with_drifted(vec![drifted_item.clone()]);
    assert_eq!(with_drifted.status, CheckStatus::Drifted);
    assert_eq!(with_drifted.drifted.len(), 1);
}

#[test]
fn test_sync_engine_fix() {
    // fix() should return a SyncReport (for now just re-syncs)
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.fix().unwrap();

    // fix() should succeed
    assert!(report.success);
}

#[test]
fn test_sync_engine_load_save_ledger() {
    // Test load_ledger and save_ledger methods
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();

    // Load should create empty ledger when file doesn't exist
    let ledger = engine.load_ledger().unwrap();
    assert!(ledger.intents().is_empty());

    // Create a ledger with content
    let mut ledger = Ledger::new();
    ledger.add_intent(Intent::new("rule:test".to_string(), json!({})));

    // Save the ledger
    engine.save_ledger(&ledger).unwrap();

    // Load and verify
    let loaded = engine.load_ledger().unwrap();
    assert_eq!(loaded.intents().len(), 1);
}

#[test]
fn test_sync_uses_rule_registry_uuids() {
    // Task 1.3: Verify that sync uses rule UUIDs from the registry as block markers
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create .repository directory structure
    let repo_dir = temp.path().join(".repository");
    let rules_dir = repo_dir.join("rules");
    fs::create_dir_all(&rules_dir).unwrap();

    // Create a rule registry with a test rule
    let registry_path = rules_dir.join("registry.toml");
    let mut registry = repo_core::RuleRegistry::new(registry_path.clone());
    let rule_uuid = registry
        .add_rule("test-rule", "Test rule content", vec!["test".to_string()])
        .unwrap()
        .uuid;

    // Create config.toml with cursor tool enabled
    let config_content = r#"
tools = ["cursor"]

[core]
mode = "standard"
"#;
    fs::write(repo_dir.join("config.toml"), config_content).unwrap();

    // Run sync
    let engine = SyncEngine::new(root.clone(), Mode::Standard).unwrap();
    let report = engine.sync().unwrap();
    assert!(report.success, "Sync should succeed: {:?}", report.errors);

    // Verify .cursorrules contains block with rule UUID
    let cursorrules_path = temp.path().join(".cursorrules");
    assert!(cursorrules_path.exists(), ".cursorrules should be created");

    let content = fs::read_to_string(&cursorrules_path).unwrap();
    let uuid_str = rule_uuid.to_string();
    assert!(
        content.contains(&uuid_str),
        ".cursorrules should contain rule UUID {}: got content:\n{}",
        uuid_str,
        content
    );
}

#[test]
fn test_sync_reads_tools_from_config_using_manifest() {
    // GAP-021: SyncEngine should use typed Manifest parsing instead of raw toml::Value
    // This test verifies that tools are correctly read from config.toml using Manifest::parse()
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    // Create .repository directory with config.toml containing tools
    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    // Write a config.toml with tools - the Manifest struct expects tools at the top level
    let config_content = r#"
tools = ["claude", "cursor"]

[core]
mode = "standard"
"#;
    fs::write(repo_dir.join("config.toml"), config_content).unwrap();

    // Run sync with dry_run to avoid triggering unrelated ledger serialization issues
    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let options = repo_core::sync::SyncOptions { dry_run: true };
    let report = engine.sync_with_options(options).unwrap();

    // Sync should succeed (dry_run doesn't write, so no serialization issues)
    assert!(report.success, "Sync should succeed");

    // Verify that the tools were processed by checking action strings directly.
    // At least one action should reference a configured tool name.
    let mentions_tool = report
        .actions
        .iter()
        .any(|action| action.contains("claude") || action.contains("cursor"));
    assert!(
        mentions_tool,
        "At least one action should reference a configured tool. Actions: {:?}",
        report.actions
    );
}

#[test]
fn test_sync_core_value_proposition() {
    // GAP-004 / Roadmap Phase 0.4: The single most important integration test.
    //
    // Verifies the full pipeline:
    //   .repository/config.toml  →  repo sync  →  .cursorrules, CLAUDE.md, .vscode/settings.json
    //
    // This test runs a real (non-dry-run) sync and asserts that the expected
    // tool config files are actually written to disk.
    let temp = setup_git_repo();
    let root = NormalizedPath::new(temp.path());

    let repo_dir = temp.path().join(".repository");
    fs::create_dir_all(&repo_dir).unwrap();

    let config_content = r#"
tools = ["cursor", "claude", "vscode"]

[core]
mode = "standard"
"#;
    fs::write(repo_dir.join("config.toml"), config_content).unwrap();

    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    let report = engine.sync().unwrap();

    assert!(
        report.success,
        "Sync should succeed. Actions: {:?}",
        report.actions
    );

    // .cursorrules must exist
    let cursorrules = temp.path().join(".cursorrules");
    assert!(
        cursorrules.exists(),
        ".cursorrules should be created by sync"
    );

    // CLAUDE.md must exist
    let claude_md = temp.path().join("CLAUDE.md");
    assert!(
        claude_md.exists(),
        "CLAUDE.md should be created by sync"
    );

    // .vscode/settings.json must exist
    let vscode_settings = temp.path().join(".vscode").join("settings.json");
    assert!(
        vscode_settings.exists(),
        ".vscode/settings.json should be created by sync"
    );

    // Ledger must record intents for all three tools
    let ledger_path = engine.ledger_path();
    assert!(ledger_path.exists(), "Ledger should be written after sync");
    let ledger_content = fs::read_to_string(&ledger_path).unwrap();
    assert!(
        ledger_content.contains("cursor"),
        "Ledger should record cursor intent"
    );
    assert!(
        ledger_content.contains("claude"),
        "Ledger should record claude intent"
    );
    assert!(
        ledger_content.contains("vscode"),
        "Ledger should record vscode intent"
    );
}
