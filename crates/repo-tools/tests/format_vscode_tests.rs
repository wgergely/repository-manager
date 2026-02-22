//! Format validation tests for VSCode settings.json output.
//!
//! Category: format-validation
//! These tests verify structural properties of generated .vscode/settings.json
//! that existing content-assertion tests do not cover.

use repo_fs::NormalizedPath;
use repo_tools::{Rule, SyncContext, ToolIntegration, VSCodeIntegration};
use serde_json::Value;
use std::fs;
use tempfile::TempDir;

/// Sync with the given context and rules, return raw settings.json content.
fn sync_and_read(context: &SyncContext, rules: &[Rule]) -> String {
    let integration = VSCodeIntegration::new();
    integration.sync(context, rules).unwrap();
    let settings_path = context.root.to_native().join(".vscode/settings.json");
    fs::read_to_string(settings_path).unwrap()
}

#[test]
fn vscode_settings_is_valid_json_object() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let python = NormalizedPath::new("/usr/bin/python3");
    let context = SyncContext::new(root).with_python(python);

    let content = sync_and_read(&context, &[]);
    let parsed: Value =
        serde_json::from_str(&content).expect("settings.json must be valid JSON");
    assert!(
        parsed.is_object(),
        "Top-level value must be a JSON object, got: {parsed}"
    );
}

#[test]
fn vscode_settings_without_python_is_still_valid_json() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root); // no python path

    let content = sync_and_read(&context, &[]);
    let parsed: Value = serde_json::from_str(&content)
        .expect("settings.json without python must still be valid JSON");
    assert!(
        parsed.is_object(),
        "Must be a JSON object even without python path"
    );
}

#[test]
fn vscode_settings_has_no_duplicate_keys() {
    let temp = TempDir::new().unwrap();
    let vscode_dir = temp.path().join(".vscode");
    fs::create_dir_all(&vscode_dir).unwrap();

    // Pre-populate with a key that sync will also write
    let existing = serde_json::json!({
        "python.defaultInterpreterPath": "/old/python",
        "editor.fontSize": 14
    });
    fs::write(
        vscode_dir.join("settings.json"),
        serde_json::to_string_pretty(&existing).unwrap(),
    )
    .unwrap();

    let root = NormalizedPath::new(temp.path());
    let python = NormalizedPath::new("/new/python");
    let context = SyncContext::new(root).with_python(python);

    let content = sync_and_read(&context, &[]);

    // Count occurrences of the python key in raw text.
    // serde_json silently deduplicates, so we must inspect the raw output.
    let key = "python.defaultInterpreterPath";
    let count = content.matches(key).count();
    assert_eq!(
        count, 1,
        "Key '{key}' appears {count} times in raw JSON — expected exactly 1"
    );
}

#[test]
fn vscode_settings_round_trips_through_multiple_syncs() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let python = NormalizedPath::new("/usr/bin/python3");
    let context = SyncContext::new(root).with_python(python);

    // Sync three times in a row
    let integration = VSCodeIntegration::new();
    for _ in 0..3 {
        integration.sync(&context, &[]).unwrap();
    }

    let settings_path = temp.path().join(".vscode/settings.json");
    let content = fs::read_to_string(settings_path).unwrap();
    let parsed: Value = serde_json::from_str(&content)
        .expect("settings.json must remain valid JSON after multiple syncs");
    assert!(parsed.is_object());
    assert_eq!(
        parsed["python.defaultInterpreterPath"], "/usr/bin/python3",
        "Python path must survive multiple syncs"
    );
}
