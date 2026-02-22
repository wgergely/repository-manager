//! Format validation tests for Antigravity .agent/rules/ output.
//!
//! Category: format-validation
//! Validates directory structure, file naming conventions, and content
//! format for Antigravity's per-rule file output.

use regex::Regex;
use repo_fs::NormalizedPath;
use repo_tools::{Rule, SyncContext, ToolIntegration, antigravity_integration};
use std::fs;
use tempfile::TempDir;

#[test]
fn antigravity_creates_rules_directory_not_file() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "test-rule".to_string(),
        content: "Test content".to_string(),
    }];

    let integration = antigravity_integration();
    integration.sync(&context, &rules).unwrap();

    let rules_path = temp.path().join(".agent/rules");
    assert!(
        rules_path.is_dir(),
        ".agent/rules must be a directory, not a file"
    );
    assert!(
        !rules_path.is_file(),
        ".agent/rules must not be a regular file"
    );
}

#[test]
fn antigravity_rule_files_follow_naming_convention() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root);
    let rules = vec![
        Rule {
            id: "code-style".to_string(),
            content: "Style content".to_string(),
        },
        Rule {
            id: "testing-guidelines".to_string(),
            content: "Testing content".to_string(),
        },
        Rule {
            id: "naming".to_string(),
            content: "Naming content".to_string(),
        },
    ];

    let integration = antigravity_integration();
    integration.sync(&context, &rules).unwrap();

    let rules_dir = temp.path().join(".agent/rules");
    let pattern = Regex::new(r"^\d{2}-[\w-]+\.md$").unwrap();

    let mut entries: Vec<String> = fs::read_dir(&rules_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    entries.sort();

    assert_eq!(
        entries.len(),
        rules.len(),
        "Expected {} rule files, found {}: {:?}",
        rules.len(),
        entries.len(),
        entries
    );

    for entry in &entries {
        assert!(
            pattern.is_match(entry),
            "Rule file '{entry}' does not match expected pattern NN-<id>.md"
        );
    }

    // Verify zero-padded sequential ordering matches rule order
    assert!(entries.contains(&"01-code-style.md".to_string()));
    assert!(entries.contains(&"02-testing-guidelines.md".to_string()));
    assert!(entries.contains(&"03-naming.md".to_string()));
}

#[test]
fn antigravity_rule_files_are_valid_markdown_without_block_markers() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "content-rule".to_string(),
        content: "This is meaningful rule content.".to_string(),
    }];

    let integration = antigravity_integration();
    integration.sync(&context, &rules).unwrap();

    let rule_path = temp.path().join(".agent/rules/01-content-rule.md");
    let content = fs::read_to_string(&rule_path).unwrap();

    // File must be non-empty
    assert!(!content.trim().is_empty(), "Rule file must not be empty");

    // Must contain the rule content
    assert!(
        content.contains("This is meaningful rule content."),
        "Rule file must contain the rule content"
    );

    // Antigravity uses one-file-per-rule — no managed block markers
    assert!(
        !content.contains("<!-- repo:block:"),
        "Per-rule files must NOT contain managed block markers (uses one-file-per-rule pattern)"
    );
    assert!(
        !content.contains("<!-- /repo:block:"),
        "Per-rule files must NOT contain managed block closing markers"
    );
}
