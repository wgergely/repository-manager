//! Format validation tests for Aider .aider.conf.yml output.
//!
//! Category: format-validation
//! Validates that Aider's YAML output is parseable and uses correct
//! YAML comment syntax for block markers (not HTML comments).

use repo_fs::NormalizedPath;
use repo_tools::{Rule, SyncContext, ToolIntegration, aider_integration};
use std::fs;
use tempfile::TempDir;

#[test]
fn aider_config_is_valid_yaml() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "style-guide".to_string(),
        content: "Follow PEP 8 for Python code.".to_string(),
    }];

    let integration = aider_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join(".aider.conf.yml")).unwrap();

    // Must parse without error as YAML (all-comment YAML is valid — parses as null)
    let result: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
    assert!(
        result.is_ok(),
        "Generated .aider.conf.yml must be valid YAML, parse error: {:?}",
        result.err()
    );
}

#[test]
fn aider_config_uses_yaml_comment_markers_not_html() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "testing".to_string(),
        content: "Write tests for all functions.".to_string(),
    }];

    let integration = aider_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join(".aider.conf.yml")).unwrap();

    // Block markers must use YAML comment syntax (# prefix)
    assert!(
        content.contains("# repo:block:testing"),
        "YAML block markers must use # prefix"
    );
    assert!(
        content.contains("# /repo:block:testing"),
        "YAML closing markers must use # prefix"
    );

    // Must NOT use HTML comment markers (would break YAML parsing)
    assert!(
        !content.contains("<!-- repo:block:"),
        "YAML file must not contain HTML-style <!-- markers"
    );
    assert!(
        !content.contains("<!-- /repo:block:"),
        "YAML file must not contain HTML-style <!-- closing markers"
    );
}

#[test]
fn aider_managed_blocks_have_matching_open_close() {
    let temp = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp.path());
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
    ];

    let integration = aider_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join(".aider.conf.yml")).unwrap();

    for rule in &rules {
        let open_marker = format!("# repo:block:{}", rule.id);
        let close_marker = format!("# /repo:block:{}", rule.id);

        let open_count = content.matches(&open_marker).count();
        let close_count = content.matches(&close_marker).count();

        assert_eq!(
            open_count, 1,
            "Expected exactly 1 open marker for '{}', found {open_count}",
            rule.id
        );
        assert_eq!(
            close_count, 1,
            "Expected exactly 1 close marker for '{}', found {close_count}",
            rule.id
        );
    }
}
