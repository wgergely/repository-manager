//! Integration tests for Claude integration.

use repo_fs::NormalizedPath;
use repo_tools::{claude_integration, ConfigType, Rule, SyncContext, ToolIntegration};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_claude_name() {
    let integration = claude_integration();
    assert_eq!(integration.name(), "claude");
}

#[test]
fn test_claude_config_locations() {
    let integration = claude_integration();
    let locations = integration.config_locations();
    assert_eq!(locations.len(), 2);
    assert_eq!(locations[0].path, "CLAUDE.md");
    assert_eq!(locations[0].config_type, ConfigType::Markdown);
    assert!(!locations[0].is_directory);
    assert_eq!(locations[1].path, ".claude/rules/");
    assert!(locations[1].is_directory);
}

#[test]
fn test_claude_creates_claude_md_with_correct_content() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "project-info".to_string(),
        content: "This is a test project.".to_string(),
    }];

    let integration = claude_integration();
    integration.sync(&context, &rules).unwrap();

    let claude_md_path = temp_dir.path().join("CLAUDE.md");
    let content = fs::read_to_string(&claude_md_path).unwrap();

    // Verify managed block structure: open marker, content, close marker
    assert!(content.contains("<!-- repo:block:project-info -->"));
    assert!(content.contains("This is a test project."));
    assert!(content.contains("<!-- /repo:block:project-info -->"));

    // Verify block ordering: open marker comes before content, content before close marker
    let open_pos = content.find("<!-- repo:block:project-info -->").unwrap();
    let content_pos = content.find("This is a test project.").unwrap();
    let close_pos = content.find("<!-- /repo:block:project-info -->").unwrap();
    assert!(open_pos < content_pos, "Open marker should precede content");
    assert!(content_pos < close_pos, "Content should precede close marker");
}

#[test]
fn test_claude_uses_managed_blocks() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "context-block".to_string(),
        content: "Project context information".to_string(),
    }];

    let integration = claude_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

    // Verify managed block markers are present
    assert!(content.contains("<!-- repo:block:context-block -->"));
    assert!(content.contains("Project context information"));
    assert!(content.contains("<!-- /repo:block:context-block -->"));
}

#[test]
fn test_claude_multiple_rules() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![
        Rule {
            id: "project-context".to_string(),
            content: "This is a Rust project.".to_string(),
        },
        Rule {
            id: "coding-standards".to_string(),
            content: "Use idiomatic Rust patterns.".to_string(),
        },
        Rule {
            id: "testing-guidelines".to_string(),
            content: "Write unit tests for all functions.".to_string(),
        },
    ];

    let integration = claude_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

    // All rules should be present
    assert!(content.contains("<!-- repo:block:project-context -->"));
    assert!(content.contains("This is a Rust project."));
    assert!(content.contains("<!-- repo:block:coding-standards -->"));
    assert!(content.contains("Use idiomatic Rust patterns."));
    assert!(content.contains("<!-- repo:block:testing-guidelines -->"));
    assert!(content.contains("Write unit tests for all functions."));
}

#[test]
fn test_claude_updates_existing_block() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root.clone());
    let integration = claude_integration();

    // First sync
    let rules = vec![Rule {
        id: "dynamic-context".to_string(),
        content: "Version 1.0".to_string(),
    }];
    integration.sync(&context, &rules).unwrap();

    // Second sync with updated content
    let rules = vec![Rule {
        id: "dynamic-context".to_string(),
        content: "Version 2.0".to_string(),
    }];
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

    // Should have updated content
    assert!(content.contains("Version 2.0"));
    assert!(!content.contains("Version 1.0"));

    // Should only have one block (not duplicated)
    assert_eq!(
        content
            .matches("<!-- repo:block:dynamic-context -->")
            .count(),
        1
    );
    assert_eq!(
        content
            .matches("<!-- /repo:block:dynamic-context -->")
            .count(),
        1
    );
}

#[test]
fn test_claude_preserves_manual_content() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    // Create CLAUDE.md with manual documentation
    let manual = r#"# Project Documentation

## Overview

This project does amazing things.

## Architecture

The codebase is organized into modules.
"#;
    fs::write(temp_dir.path().join("CLAUDE.md"), manual).unwrap();

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "auto-context".to_string(),
        content: "Managed content".to_string(),
    }];

    let integration = claude_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

    // Manual content preserved
    assert!(content.contains("# Project Documentation"));
    assert!(content.contains("## Overview"));
    assert!(content.contains("This project does amazing things."));
    assert!(content.contains("## Architecture"));

    // Managed block added
    assert!(content.contains("<!-- repo:block:auto-context -->"));
    assert!(content.contains("Managed content"));
}

#[test]
fn test_claude_empty_rules() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    // Create existing file
    fs::write(
        temp_dir.path().join("CLAUDE.md"),
        "# Existing Documentation",
    )
    .unwrap();

    let context = SyncContext::new(root);
    let rules: Vec<Rule> = vec![];

    let integration = claude_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

    // File should still exist with existing content
    assert!(content.contains("# Existing Documentation"));
}

#[test]
fn test_claude_markdown_content_in_rules() {
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root);
    let rules = vec![Rule {
        id: "rich-content".to_string(),
        content: r#"## Code Guidelines

- Use `snake_case` for variables
- Use `PascalCase` for types
- Run `cargo fmt` before commits

```rust
fn example() {
    println!("Hello");
}
```"#
            .to_string(),
    }];

    let integration = claude_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();

    assert!(content.contains("## Code Guidelines"));
    assert!(content.contains("`snake_case`"));
    assert!(content.contains("```rust"));
    assert!(content.contains("fn example()"));
}

#[test]
fn test_claude_without_python_path_still_writes_rules() {
    // Claude integration shouldn't require python path and should still write content
    let temp_dir = TempDir::new().unwrap();
    let root = NormalizedPath::new(temp_dir.path());

    let context = SyncContext::new(root); // No python path
    let rules = vec![Rule {
        id: "test".to_string(),
        content: "Test content".to_string(),
    }];

    let integration = claude_integration();
    integration.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp_dir.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("<!-- repo:block:test -->"));
    assert!(content.contains("Test content"));
    assert!(content.contains("<!-- /repo:block:test -->"));
}
