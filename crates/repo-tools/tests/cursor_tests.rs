//! Integration tests for Cursor integration.

use repo_fs::NormalizedPath;
use repo_tools::{cursor_integration, Rule, SyncContext, ToolIntegration};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cursor_name() {
    let integration = cursor_integration();
    assert_eq!(integration.name(), "cursor");
}

#[test]
fn test_cursor_config_paths() {
    let integration = cursor_integration();
    let paths = integration.config_paths();
    assert_eq!(paths, vec![".cursorrules"]);
}

#[test]
fn test_cursor_creates_cursorrules() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "test-rule".to_string(),
        content: "Test rule content".to_string(),
    }];

    let integration = cursor_integration();
    integration.sync(&context, &rules).unwrap();

    let cursorrules_path = temp_dir.path().join(".cursorrules");
    assert!(cursorrules_path.exists());
}

#[test]
fn test_cursor_uses_managed_blocks() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "my-block".to_string(),
        content: "Block content here".to_string(),
    }];

    let integration = cursor_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

    // Verify managed block markers are present
    assert!(content.contains("<!-- repo:block:my-block -->"));
    assert!(content.contains("Block content here"));
    assert!(content.contains("<!-- /repo:block:my-block -->"));
}

#[test]
fn test_cursor_multiple_rules() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![
        Rule {
            id: "rule-alpha".to_string(),
            content: "Alpha content".to_string(),
        },
        Rule {
            id: "rule-beta".to_string(),
            content: "Beta content".to_string(),
        },
        Rule {
            id: "rule-gamma".to_string(),
            content: "Gamma content".to_string(),
        },
    ];

    let integration = cursor_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

    // All rules should be present
    assert!(content.contains("<!-- repo:block:rule-alpha -->"));
    assert!(content.contains("Alpha content"));
    assert!(content.contains("<!-- repo:block:rule-beta -->"));
    assert!(content.contains("Beta content"));
    assert!(content.contains("<!-- repo:block:rule-gamma -->"));
    assert!(content.contains("Gamma content"));
}

#[test]
fn test_cursor_updates_existing_block() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root.clone());
    let integration = cursor_integration();

    // First sync
    let rules = vec![Rule {
        id: "updatable".to_string(),
        content: "Initial content".to_string(),
    }];
    integration.sync(&context, &rules).unwrap();

    // Second sync with updated content
    let rules = vec![Rule {
        id: "updatable".to_string(),
        content: "Updated content".to_string(),
    }];
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

    // Should have updated content
    assert!(content.contains("Updated content"));
    assert!(!content.contains("Initial content"));

    // Should only have one block (not duplicated)
    assert_eq!(content.matches("<!-- repo:block:updatable -->").count(), 1);
    assert_eq!(content.matches("<!-- /repo:block:updatable -->").count(), 1);
}

#[test]
fn test_cursor_preserves_manual_content() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    // Create cursorrules with manual content
    let manual = r#"# Cursor Rules

These are manual rules for the project.

## Guidelines

- Follow best practices
- Write clean code
"#;
    fs::write(temp_dir.path().join(".cursorrules"), manual).unwrap();

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "auto-rule".to_string(),
        content: "Automated rule".to_string(),
    }];

    let integration = cursor_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

    // Manual content preserved
    assert!(content.contains("# Cursor Rules"));
    assert!(content.contains("These are manual rules"));
    assert!(content.contains("Follow best practices"));

    // Managed block added
    assert!(content.contains("<!-- repo:block:auto-rule -->"));
    assert!(content.contains("Automated rule"));
}

#[test]
fn test_cursor_empty_rules() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    // Create existing file
    fs::write(temp_dir.path().join(".cursorrules"), "Existing content").unwrap();

    let context = SyncContext::new(root);
    let rules: Vec<Rule> = vec![];

    let integration = cursor_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

    // File should still exist with existing content
    assert!(content.contains("Existing content"));
}

#[test]
fn test_cursor_multiline_rule_content() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "multiline".to_string(),
        content: "Line 1\nLine 2\nLine 3\n\nWith blank line".to_string(),
    }];

    let integration = cursor_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join(".cursorrules")).unwrap();

    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3"));
    assert!(content.contains("With blank line"));
}
