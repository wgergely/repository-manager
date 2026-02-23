//! Format validation tests for Markdown-based tool outputs.
//!
//! Category: format-validation
//! Covers: Cursor (.cursorrules), Claude (CLAUDE.md), Copilot
//! (.github/copilot-instructions.md), Windsurf (.windsurfrules),
//! Gemini (GEMINI.md)
//!
//! These tests validate managed-block structural integrity across all
//! file-based Markdown tools. Directory-based tools (Cline, Roo,
//! Antigravity) are tested in separate files.

use regex::Regex;
use repo_fs::NormalizedPath;
use repo_tools::{
    Rule, SyncContext, ToolIntegration, claude_integration, copilot_integration,
    cursor_integration, gemini_integration, windsurf_integration,
};
use std::fs;
use tempfile::TempDir;

/// Returns (tool_name, integration, primary_config_path) for each
/// file-based Markdown tool.
fn file_based_markdown_tools() -> Vec<(&'static str, Box<dyn ToolIntegration>, &'static str)> {
    vec![
        ("cursor", Box::new(cursor_integration()), ".cursorrules"),
        ("claude", Box::new(claude_integration()), "CLAUDE.md"),
        (
            "copilot",
            Box::new(copilot_integration()),
            ".github/copilot-instructions.md",
        ),
        (
            "windsurf",
            Box::new(windsurf_integration()),
            ".windsurfrules",
        ),
        ("gemini", Box::new(gemini_integration()), "GEMINI.md"),
    ]
}

fn sample_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "format-alpha".to_string(),
            content: "Alpha rule content for testing.".to_string(),
        },
        Rule {
            id: "format-beta".to_string(),
            content: "Beta rule content\nwith multiple lines.".to_string(),
        },
    ]
}

#[test]
fn managed_block_markers_are_balanced() {
    let open_re = Regex::new(r"<!-- repo:block:([\w.-]+) -->").unwrap();
    let close_re = Regex::new(r"<!-- /repo:block:([\w.-]+) -->").unwrap();

    for (tool_name, integration, config_path) in file_based_markdown_tools() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let context = SyncContext::new(root);

        integration.sync(&context, &sample_rules()).unwrap();

        let content = fs::read_to_string(temp.path().join(config_path))
            .unwrap_or_else(|e| panic!("[{tool_name}] Failed to read {config_path}: {e}"));

        let open_ids: Vec<String> = open_re
            .captures_iter(&content)
            .map(|c| c[1].to_string())
            .collect();
        let close_ids: Vec<String> = close_re
            .captures_iter(&content)
            .map(|c| c[1].to_string())
            .collect();

        assert_eq!(
            open_ids.len(),
            close_ids.len(),
            "[{tool_name}] Mismatched block marker count: {} open vs {} close",
            open_ids.len(),
            close_ids.len()
        );

        for open_id in &open_ids {
            assert!(
                close_ids.contains(open_id),
                "[{tool_name}] Open marker for '{open_id}' has no matching close marker"
            );
        }
    }
}

#[test]
fn managed_block_markers_use_consistent_format() {
    let valid_open = Regex::new(r"^<!-- repo:block:[\w.-]+ -->$").unwrap();
    let valid_close = Regex::new(r"^<!-- /repo:block:[\w.-]+ -->$").unwrap();

    for (tool_name, integration, config_path) in file_based_markdown_tools() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let context = SyncContext::new(root);

        integration.sync(&context, &sample_rules()).unwrap();

        let content = fs::read_to_string(temp.path().join(config_path))
            .unwrap_or_else(|e| panic!("[{tool_name}] Failed to read {config_path}: {e}"));

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.contains("repo:block:") && trimmed.starts_with("<!--") {
                if trimmed.contains("/repo:block:") {
                    assert!(
                        valid_close.is_match(trimmed),
                        "[{tool_name}] Invalid close marker format: '{trimmed}'"
                    );
                } else {
                    assert!(
                        valid_open.is_match(trimmed),
                        "[{tool_name}] Invalid open marker format: '{trimmed}'"
                    );
                }
            }
        }
    }
}

#[test]
fn markdown_output_has_balanced_html_comments() {
    for (tool_name, integration, config_path) in file_based_markdown_tools() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let context = SyncContext::new(root);

        integration.sync(&context, &sample_rules()).unwrap();

        let content = fs::read_to_string(temp.path().join(config_path))
            .unwrap_or_else(|e| panic!("[{tool_name}] Failed to read {config_path}: {e}"));

        // Every HTML comment opener must have a matching closer
        let openers = content.matches("<!--").count();
        let closers = content.matches("-->").count();

        assert_eq!(
            openers, closers,
            "[{tool_name}] Unbalanced HTML comments: {openers} openers vs {closers} closers"
        );
    }
}

#[test]
fn user_content_outside_blocks_is_preserved_after_sync() {
    for (tool_name, integration, config_path) in file_based_markdown_tools() {
        let temp = TempDir::new().unwrap();

        // Pre-populate with user content
        let user_content = "# My Custom Rules\n\nThese are my hand-written rules.\n\n## Guidelines\n\n- Be consistent\n- Write tests\n";
        let full_path = temp.path().join(config_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, user_content).unwrap();

        let root = NormalizedPath::new(temp.path());
        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "auto-rule".to_string(),
            content: "Automated content".to_string(),
        }];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(&full_path)
            .unwrap_or_else(|e| panic!("[{tool_name}] Failed to read after sync: {e}"));

        assert!(
            content.contains("# My Custom Rules"),
            "[{tool_name}] User heading was lost after sync"
        );
        assert!(
            content.contains("These are my hand-written rules."),
            "[{tool_name}] User paragraph was lost after sync"
        );
        assert!(
            content.contains("- Be consistent"),
            "[{tool_name}] User bullet was lost after sync"
        );
    }
}
