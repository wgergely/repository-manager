//! Format validation tests for directory-based tool outputs (JetBrains, Roo).
//!
//! Category: format-validation
//! Validates directory structure, file naming conventions, and content
//! format for tools that use per-rule directory output.
//!
//! Mapped from Phase 3 plan "format_json_tests.rs" — adapted because
//! JetBrains and Roo use directory-based markdown rules, not JSON.

use regex::Regex;
use repo_fs::NormalizedPath;
use repo_tools::{Rule, SyncContext, ToolIntegration, jetbrains_integration, roo_integration};
use std::fs;
use tempfile::TempDir;

#[test]
fn jetbrains_creates_rules_directory_with_valid_structure() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root);
    let rules = vec![
        Rule {
            id: "code-style".to_string(),
            content: "Use IntelliJ code style.".to_string(),
        },
        Rule {
            id: "testing".to_string(),
            content: "Write JUnit tests.".to_string(),
        },
    ];

    let integration = jetbrains_integration();
    integration.sync(&context, &rules).unwrap();

    // Primary path must be a directory
    let rules_dir = temp.path().join(".aiassistant/rules");
    assert!(
        rules_dir.is_dir(),
        ".aiassistant/rules/ must be a directory"
    );

    // Rule files must follow naming convention
    let pattern = Regex::new(r"^\d{2}-[\w-]+\.md$").unwrap();
    let mut entries: Vec<String> = fs::read_dir(&rules_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    entries.sort();

    assert_eq!(entries.len(), rules.len());
    for entry in &entries {
        assert!(
            pattern.is_match(entry),
            "JetBrains rule file '{entry}' does not match NN-<id>.md pattern"
        );
    }

    // Each file must contain its rule content and be non-empty
    let rule1 = fs::read_to_string(rules_dir.join("01-code-style.md")).unwrap();
    assert!(
        rule1.contains("Use IntelliJ code style."),
        "Rule file must contain rule content"
    );
    assert!(
        !rule1.contains("<!-- repo:block:"),
        "Directory-based rule files must not contain managed block markers"
    );
}

#[test]
fn roo_creates_rules_directory_with_valid_structure() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root);
    let rules = vec![
        Rule {
            id: "conventions".to_string(),
            content: "Follow project conventions.".to_string(),
        },
        Rule {
            id: "architecture".to_string(),
            content: "Maintain modular architecture.".to_string(),
        },
    ];

    let integration = roo_integration();
    integration.sync(&context, &rules).unwrap();

    // Primary path must be a directory
    let rules_dir = temp.path().join(".roo/rules");
    assert!(rules_dir.is_dir(), ".roo/rules/ must be a directory");

    // Verify rule files
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
        "Expected {} rule files, found {:?}",
        rules.len(),
        entries
    );
    for entry in &entries {
        assert!(
            pattern.is_match(entry),
            "Roo rule file '{entry}' does not match NN-<id>.md pattern"
        );
    }

    // Content validation
    let content = fs::read_to_string(rules_dir.join("01-conventions.md")).unwrap();
    assert!(
        !content.trim().is_empty(),
        "Roo rule file must not be empty"
    );
    assert!(
        content.contains("Follow project conventions."),
        "Roo rule file must contain rule content"
    );
    assert!(
        !content.contains("<!-- repo:block:"),
        "Directory-based rule files must not contain managed block markers"
    );
}
