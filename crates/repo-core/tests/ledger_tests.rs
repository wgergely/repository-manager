//! Tests for the Ledger system

use pretty_assertions::assert_eq;
use repo_core::ledger::{Intent, Ledger, Projection, ProjectionKind};
use serde_json::json;
use std::path::PathBuf;
use tempfile::tempdir;
use uuid::Uuid;

#[test]
fn test_ledger_add_intent() {
    let mut ledger = Ledger::new();
    assert!(ledger.intents().is_empty());

    let intent = Intent::new(
        "rule:python/style/snake-case".to_string(),
        json!({"severity": "warning"}),
    );
    let uuid = intent.uuid;

    ledger.add_intent(intent);

    assert_eq!(ledger.intents().len(), 1);
    assert!(ledger.get_intent(uuid).is_some());
    assert_eq!(
        ledger.get_intent(uuid).unwrap().id,
        "rule:python/style/snake-case"
    );
}

#[test]
fn test_ledger_remove_intent() {
    let mut ledger = Ledger::new();

    let intent1 = Intent::new("rule:rust/style/naming".to_string(), json!({}));
    let intent2 = Intent::new("rule:python/style/snake-case".to_string(), json!({}));
    let uuid1 = intent1.uuid;
    let uuid2 = intent2.uuid;

    ledger.add_intent(intent1);
    ledger.add_intent(intent2);
    assert_eq!(ledger.intents().len(), 2);

    // Remove first intent
    let removed = ledger.remove_intent(uuid1);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().uuid, uuid1);
    assert_eq!(ledger.intents().len(), 1);

    // Try to remove non-existent intent
    let not_found = ledger.remove_intent(Uuid::new_v4());
    assert!(not_found.is_none());

    // Verify remaining intent is intact
    assert!(ledger.get_intent(uuid2).is_some());
}

#[test]
fn test_ledger_find_by_rule() {
    let mut ledger = Ledger::new();

    let intent1 = Intent::new("rule:python/style/snake-case".to_string(), json!({}));
    let intent2 = Intent::new(
        "rule:python/style/snake-case".to_string(),
        json!({"strict": true}),
    );
    let intent3 = Intent::new("rule:rust/style/naming".to_string(), json!({}));

    ledger.add_intent(intent1);
    ledger.add_intent(intent2);
    ledger.add_intent(intent3);

    let python_intents = ledger.find_by_rule("rule:python/style/snake-case");
    assert_eq!(python_intents.len(), 2);

    let rust_intents = ledger.find_by_rule("rule:rust/style/naming");
    assert_eq!(rust_intents.len(), 1);

    let empty = ledger.find_by_rule("rule:nonexistent");
    assert!(empty.is_empty());
}

#[test]
fn test_intent_with_projections() {
    let mut intent = Intent::new(
        "rule:python/style/snake-case".to_string(),
        json!({"severity": "error"}),
    );

    // Add projections
    let marker = Uuid::new_v4();
    let proj1 = Projection::text_block(
        "cursor".to_string(),
        PathBuf::from(".cursor/rules/python.mdc"),
        marker,
        "abc123".to_string(),
    );
    let proj2 = Projection::json_key(
        "vscode".to_string(),
        PathBuf::from(".vscode/settings.json"),
        "python.linting.enabled".to_string(),
        json!(true),
    );

    intent.add_projection(proj1);
    intent.add_projection(proj2);

    assert_eq!(intent.projections().len(), 2);
    assert_eq!(intent.projections()[0].tool, "cursor");
    assert_eq!(intent.projections()[1].tool, "vscode");

    // Remove projection
    let removed = intent.remove_projection("cursor", &PathBuf::from(".cursor/rules/python.mdc"));
    assert!(removed.is_some());
    assert_eq!(intent.projections().len(), 1);

    // Try to remove non-existent projection
    let not_found = intent.remove_projection("cursor", &PathBuf::from("nonexistent"));
    assert!(not_found.is_none());
}

#[test]
fn test_ledger_projections_for_file() {
    let mut ledger = Ledger::new();

    let mut intent1 = Intent::new("rule:python/style/snake-case".to_string(), json!({}));
    let mut intent2 = Intent::new("rule:python/docstrings".to_string(), json!({}));

    let marker1 = Uuid::new_v4();
    let marker2 = Uuid::new_v4();

    // Both intents project to the same file
    intent1.add_projection(Projection::text_block(
        "cursor".to_string(),
        PathBuf::from(".cursor/rules/python.mdc"),
        marker1,
        "checksum1".to_string(),
    ));
    intent2.add_projection(Projection::text_block(
        "cursor".to_string(),
        PathBuf::from(".cursor/rules/python.mdc"),
        marker2,
        "checksum2".to_string(),
    ));

    // Second intent also projects to another file
    intent2.add_projection(Projection::json_key(
        "vscode".to_string(),
        PathBuf::from(".vscode/settings.json"),
        "python.docstring.style".to_string(),
        json!("google"),
    ));

    ledger.add_intent(intent1);
    ledger.add_intent(intent2);

    // Find projections for python.mdc
    let projections = ledger.projections_for_file(&PathBuf::from(".cursor/rules/python.mdc"));
    assert_eq!(projections.len(), 2);

    // Find projections for settings.json
    let projections = ledger.projections_for_file(&PathBuf::from(".vscode/settings.json"));
    assert_eq!(projections.len(), 1);

    // No projections for unknown file
    let projections = ledger.projections_for_file(&PathBuf::from("nonexistent"));
    assert!(projections.is_empty());
}

#[test]
fn test_ledger_save_load() {
    let dir = tempdir().unwrap();
    let ledger_path = dir.path().join("ledger.toml");

    // Create and populate ledger
    let mut ledger = Ledger::new();

    let mut intent = Intent::new(
        "rule:python/style/snake-case".to_string(),
        json!({"severity": "warning", "autofix": true}),
    );

    let marker = Uuid::new_v4();
    intent.add_projection(Projection::text_block(
        "cursor".to_string(),
        PathBuf::from(".cursor/rules/python.mdc"),
        marker,
        "abc123def456".to_string(),
    ));
    intent.add_projection(Projection::file_managed(
        "vscode".to_string(),
        PathBuf::from(".vscode/python.json"),
        "xyz789".to_string(),
    ));

    let uuid = intent.uuid;
    ledger.add_intent(intent);

    // Save ledger
    ledger.save(&ledger_path).unwrap();
    assert!(ledger_path.exists());

    // Load ledger
    let loaded = Ledger::load(&ledger_path).unwrap();

    // Verify loaded content
    assert_eq!(loaded.intents().len(), 1);
    let loaded_intent = loaded.get_intent(uuid).unwrap();
    assert_eq!(loaded_intent.id, "rule:python/style/snake-case");
    assert_eq!(loaded_intent.args["severity"], "warning");
    assert_eq!(loaded_intent.args["autofix"], true);
    assert_eq!(loaded_intent.projections().len(), 2);
}

#[test]
fn test_projection_kinds() {
    // TextBlock projection
    let marker = Uuid::new_v4();
    let text_block = Projection::text_block(
        "cursor".to_string(),
        PathBuf::from(".cursor/rules/test.mdc"),
        marker,
        "checksum123".to_string(),
    );
    assert_eq!(text_block.tool, "cursor");
    assert_eq!(text_block.file, PathBuf::from(".cursor/rules/test.mdc"));
    match &text_block.kind {
        ProjectionKind::TextBlock {
            marker: m,
            checksum,
        } => {
            assert_eq!(m, &marker);
            assert_eq!(checksum, "checksum123");
        }
        _ => panic!("Expected TextBlock kind"),
    }

    // JsonKey projection
    let json_key = Projection::json_key(
        "vscode".to_string(),
        PathBuf::from(".vscode/settings.json"),
        "editor.fontSize".to_string(),
        json!(14),
    );
    assert_eq!(json_key.tool, "vscode");
    match &json_key.kind {
        ProjectionKind::JsonKey { path, value } => {
            assert_eq!(path, "editor.fontSize");
            assert_eq!(value, &json!(14));
        }
        _ => panic!("Expected JsonKey kind"),
    }

    // FileManaged projection
    let file_managed = Projection::file_managed(
        "prettier".to_string(),
        PathBuf::from(".prettierrc"),
        "sha256abc".to_string(),
    );
    assert_eq!(file_managed.tool, "prettier");
    match &file_managed.kind {
        ProjectionKind::FileManaged { checksum } => {
            assert_eq!(checksum, "sha256abc");
        }
        _ => panic!("Expected FileManaged kind"),
    }
}

#[test]
fn test_intent_creation() {
    let intent = Intent::new("rule:test/example".to_string(), json!({"key": "value"}));

    assert_eq!(intent.id, "rule:test/example");
    assert!(!intent.uuid.is_nil());
    assert_eq!(intent.args["key"], "value");
    assert!(intent.projections().is_empty());

    // Test with_uuid constructor
    let fixed_uuid = Uuid::new_v4();
    let intent2 = Intent::with_uuid("rule:test/example2".to_string(), fixed_uuid, json!({}));
    assert_eq!(intent2.uuid, fixed_uuid);
}

#[test]
fn test_ledger_get_intent_mut() {
    let mut ledger = Ledger::new();
    let intent = Intent::new("rule:test".to_string(), json!({}));
    let uuid = intent.uuid;
    ledger.add_intent(intent);

    // Get mutable reference and modify
    let intent_mut = ledger.get_intent_mut(uuid).unwrap();
    intent_mut.add_projection(Projection::file_managed(
        "test".to_string(),
        PathBuf::from("test.txt"),
        "checksum".to_string(),
    ));

    // Verify modification persisted
    assert_eq!(ledger.get_intent(uuid).unwrap().projections().len(), 1);
}

#[test]
fn test_ledger_default_version() {
    let ledger = Ledger::new();
    // Version should be accessible through serialization
    let serialized = toml::to_string(&ledger).unwrap();
    assert!(serialized.contains("version = \"1.0\""));
}
