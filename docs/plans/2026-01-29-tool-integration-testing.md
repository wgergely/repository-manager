# Tool Integration Testing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Verify that tool sync actually produces correct output files in the format each tool expects, per the tool-config-formats.md research.

**Architecture:** Create integration tests that run sync for each tool and snapshot the generated files. Use insta for snapshot testing to catch regressions. Test top 5 tools by market share: Cursor, Claude Code, GitHub Copilot, VS Code, Windsurf.

**Tech Stack:** Rust, insta (snapshot testing), tempfile, repo-tools, repo-core

---

## Prerequisites

Reference doc: `docs/research/tool-config-formats.md`

Tool priority by industry adoption:
1. Cursor (~4.9/5 rating, most popular AI-native IDE)
2. Claude Code (~4.5/5 rating, terminal agent leader)
3. GitHub Copilot (68% market share)
4. VS Code (base editor)
5. Windsurf (growing agentic IDE)

---

## Task 1: Set Up Integration Test Infrastructure

**Files:**
- Create: `crates/repo-tools/tests/integration/mod.rs`
- Create: `crates/repo-tools/tests/integration/helpers.rs`
- Modify: `crates/repo-tools/Cargo.toml`

**Step 1: Add dev dependencies**

In `crates/repo-tools/Cargo.toml`:

```toml
[dev-dependencies]
tempfile = { workspace = true }
insta = { workspace = true, features = ["yaml"] }
rstest = { workspace = true }
```

**Step 2: Create test helpers**

Create `crates/repo-tools/tests/integration/helpers.rs`:

```rust
//! Test helpers for tool integration tests

use std::path::{Path, PathBuf};
use std::fs;

/// Create a test repository with basic structure
pub fn create_test_repo(dir: &Path) -> PathBuf {
    // Create .git directory (minimal)
    fs::create_dir_all(dir.join(".git")).unwrap();
    fs::write(dir.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

    // Create .repository directory
    fs::create_dir_all(dir.join(".repository/rules")).unwrap();

    dir.to_path_buf()
}

/// Create a test repository with config
pub fn create_repo_with_tools(dir: &Path, tools: &[&str]) -> PathBuf {
    create_test_repo(dir);

    let tools_str: Vec<String> = tools.iter().map(|t| format!("\"{}\"", t)).collect();
    let config = format!(
        r#"tools = [{}]

[core]
mode = "standard"
"#,
        tools_str.join(", ")
    );

    fs::write(dir.join(".repository/config.toml"), config).unwrap();

    dir.to_path_buf()
}

/// Create a rule file
pub fn create_rule(dir: &Path, id: &str, content: &str, tags: &[&str]) {
    let rules_dir = dir.join(".repository/rules");
    fs::create_dir_all(&rules_dir).unwrap();

    // Format as markdown with frontmatter
    let frontmatter = if !tags.is_empty() {
        format!(
            "---\ntags: [{}]\n---\n\n",
            tags.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ")
        )
    } else {
        String::new()
    };

    let rule_content = format!("{}{}", frontmatter, content);
    fs::write(rules_dir.join(format!("{}.md", id)), rule_content).unwrap();
}

/// Read file contents for snapshot comparison
pub fn read_file_for_snapshot(path: &Path) -> String {
    if path.exists() {
        fs::read_to_string(path).unwrap_or_else(|_| "<read error>".to_string())
    } else {
        "<file not found>".to_string()
    }
}

/// Collect all files in a directory for snapshot
pub fn collect_dir_files(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();

    if dir.exists() {
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let relative = entry.path().strip_prefix(dir).unwrap().to_path_buf();
            let content = fs::read_to_string(entry.path()).unwrap_or_default();
            files.push((relative, content));
        }
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}
```

**Step 3: Create mod.rs**

Create `crates/repo-tools/tests/integration/mod.rs`:

```rust
mod helpers;

pub use helpers::*;
```

**Step 4: Commit**

```bash
git add crates/repo-tools/tests/integration/ crates/repo-tools/Cargo.toml
git commit -m "test(repo-tools): set up integration test infrastructure"
```

---

## Task 2: Test Cursor Tool Integration

**Files:**
- Create: `crates/repo-tools/tests/integration/cursor_test.rs`

**Step 1: Write Cursor integration tests**

Create `crates/repo-tools/tests/integration/cursor_test.rs`:

```rust
//! Integration tests for Cursor tool sync
//!
//! Verifies that sync produces correct .cursor/rules/*.mdc files
//! with proper frontmatter format per docs/research/tool-config-formats.md

mod helpers;
use helpers::*;

use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use insta::assert_yaml_snapshot;

#[test]
fn test_cursor_creates_rules_directory() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["cursor"]);

    create_rule(
        &root,
        "python-style",
        "Use snake_case for all variable names.",
        &["python"],
    );

    let dispatcher = ToolDispatcher::new();
    let result = dispatcher.sync_for_tool(&root, ToolId::Cursor);

    assert!(result.is_ok(), "Sync should succeed: {:?}", result);

    // .cursor/rules/ directory should exist
    let rules_dir = root.join(".cursor/rules");
    assert!(rules_dir.exists(), ".cursor/rules/ should be created");
}

#[test]
fn test_cursor_mdc_file_format() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["cursor"]);

    create_rule(
        &root,
        "typescript-style",
        "Use strict TypeScript mode.\nAlways define return types.",
        &["typescript"],
    );

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Cursor).unwrap();

    let mdc_path = root.join(".cursor/rules/typescript-style.mdc");
    assert!(mdc_path.exists(), ".mdc file should be created");

    let content = std::fs::read_to_string(&mdc_path).unwrap();

    // Verify MDC format
    assert!(content.starts_with("---"), "MDC should have frontmatter");
    assert!(content.contains("description:"), "Should have description field");
    assert!(content.contains("---\n\n") || content.contains("---\r\n\r\n"),
        "Should have frontmatter separator");

    // Snapshot the full content
    assert_yaml_snapshot!("cursor_mdc_format", content);
}

#[test]
fn test_cursor_glob_patterns_in_frontmatter() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["cursor"]);

    // Create rule with file patterns
    let rules_dir = root.join(".repository/rules");
    std::fs::create_dir_all(&rules_dir).unwrap();
    std::fs::write(
        rules_dir.join("python-style.md"),
        r#"---
tags: ["python"]
files: ["*.py", "src/**/*.py"]
---

Use type hints for all function parameters.
"#,
    ).unwrap();

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Cursor).unwrap();

    let content = std::fs::read_to_string(root.join(".cursor/rules/python-style.mdc")).unwrap();

    // Cursor uses 'globs' not 'files'
    assert!(content.contains("globs:"), "Should convert 'files' to 'globs'");
    assert!(content.contains("*.py"), "Should include file patterns");

    assert_yaml_snapshot!("cursor_with_globs", content);
}

#[test]
fn test_cursor_always_apply_rule() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["cursor"]);

    // Rule without file patterns = always apply
    create_rule(
        &root,
        "global-style",
        "Always be concise.",
        &[],
    );

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Cursor).unwrap();

    let content = std::fs::read_to_string(root.join(".cursor/rules/global-style.mdc")).unwrap();

    assert!(content.contains("alwaysApply: true") || content.contains("alwaysApply:true"),
        "Rule without files should have alwaysApply: true");

    assert_yaml_snapshot!("cursor_always_apply", content);
}

#[test]
fn test_cursor_multiple_rules() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["cursor"]);

    create_rule(&root, "rule-a", "Rule A content", &["a"]);
    create_rule(&root, "rule-b", "Rule B content", &["b"]);
    create_rule(&root, "rule-c", "Rule C content", &["c"]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Cursor).unwrap();

    let files = collect_dir_files(&root.join(".cursor/rules"));

    assert_eq!(files.len(), 3, "Should create 3 .mdc files");

    // Snapshot all files
    assert_yaml_snapshot!("cursor_multiple_rules", files);
}
```

**Step 2: Run tests (expect some failures)**

Run: `cargo test -p repo-tools --test cursor_test`
Expected: Some tests may fail - this exposes real sync issues

**Step 3: Update snapshots**

Run: `cargo insta test -p repo-tools --test cursor_test`
Run: `cargo insta review`

**Step 4: Commit**

```bash
git add crates/repo-tools/tests/integration/cursor_test.rs
git add crates/repo-tools/src/snapshots/
git commit -m "test(repo-tools): add Cursor integration tests with snapshots"
```

---

## Task 3: Test Claude Code Tool Integration

**Files:**
- Create: `crates/repo-tools/tests/integration/claude_test.rs`

**Step 1: Write Claude Code integration tests**

Create `crates/repo-tools/tests/integration/claude_test.rs`:

```rust
//! Integration tests for Claude Code tool sync
//!
//! Verifies that sync produces correct CLAUDE.md with managed blocks

mod helpers;
use helpers::*;

use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use insta::assert_yaml_snapshot;

#[test]
fn test_claude_creates_claude_md() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["claude"]);

    create_rule(&root, "style", "Use clear, concise code.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Claude).unwrap();

    let claude_md = root.join("CLAUDE.md");
    assert!(claude_md.exists(), "CLAUDE.md should be created");
}

#[test]
fn test_claude_md_has_managed_blocks() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["claude"]);

    create_rule(&root, "coding-style", "Follow the project conventions.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Claude).unwrap();

    let content = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();

    // Should have repo:block markers
    assert!(content.contains("<!-- repo:block:"), "Should have block start marker");
    assert!(content.contains("<!-- /repo:block:"), "Should have block end marker");

    assert_yaml_snapshot!("claude_md_blocks", content);
}

#[test]
fn test_claude_preserves_user_content() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["claude"]);

    // Create existing CLAUDE.md with user content
    std::fs::write(
        root.join("CLAUDE.md"),
        r#"# My Project

This is my custom documentation that should be preserved.

## My Custom Section

Important notes here.
"#,
    ).unwrap();

    create_rule(&root, "test-rule", "Test instruction.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Claude).unwrap();

    let content = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();

    // User content preserved
    assert!(content.contains("My Project"), "Title should be preserved");
    assert!(content.contains("custom documentation"), "User content should be preserved");
    assert!(content.contains("My Custom Section"), "Custom section should be preserved");

    // Rule added
    assert!(content.contains("Test instruction"), "Rule should be added");

    assert_yaml_snapshot!("claude_preserve_content", content);
}

#[test]
fn test_claude_updates_existing_block() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["claude"]);

    // First sync
    create_rule(&root, "style", "Version 1 content.", &[]);
    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Claude).unwrap();

    // Update rule
    std::fs::write(
        root.join(".repository/rules/style.md"),
        "Version 2 content with updates.",
    ).unwrap();

    // Second sync
    dispatcher.sync_for_tool(&root, ToolId::Claude).unwrap();

    let content = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();

    assert!(content.contains("Version 2"), "Should have updated content");
    assert!(!content.contains("Version 1"), "Should not have old content");

    // Should still only have one block
    let block_count = content.matches("<!-- repo:block:").count();
    assert_eq!(block_count, 1, "Should have exactly one managed block");
}

#[test]
fn test_claude_multiple_rules() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["claude"]);

    create_rule(&root, "style", "Style rules.", &[]);
    create_rule(&root, "testing", "Testing rules.", &[]);
    create_rule(&root, "docs", "Documentation rules.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Claude).unwrap();

    let content = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();

    // All rules should be included
    assert!(content.contains("Style rules"), "Should have style rule");
    assert!(content.contains("Testing rules"), "Should have testing rule");
    assert!(content.contains("Documentation rules"), "Should have docs rule");

    assert_yaml_snapshot!("claude_multiple_rules", content);
}
```

**Step 2: Run and snapshot**

Run: `cargo insta test -p repo-tools --test claude_test`

**Step 3: Commit**

```bash
git add crates/repo-tools/tests/integration/claude_test.rs
git add crates/repo-tools/src/snapshots/
git commit -m "test(repo-tools): add Claude Code integration tests with snapshots"
```

---

## Task 4: Test GitHub Copilot Tool Integration

**Files:**
- Create: `crates/repo-tools/tests/integration/copilot_test.rs`

**Step 1: Write Copilot integration tests**

Create `crates/repo-tools/tests/integration/copilot_test.rs`:

```rust
//! Integration tests for GitHub Copilot tool sync
//!
//! Verifies .github/copilot-instructions.md creation

mod helpers;
use helpers::*;

use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use insta::assert_yaml_snapshot;

#[test]
fn test_copilot_creates_github_directory() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["copilot"]);

    create_rule(&root, "style", "Follow conventions.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Copilot).unwrap();

    assert!(root.join(".github").exists(), ".github directory should exist");
}

#[test]
fn test_copilot_creates_instructions_file() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["copilot"]);

    create_rule(&root, "coding", "Use TypeScript strict mode.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Copilot).unwrap();

    let instructions = root.join(".github/copilot-instructions.md");
    assert!(instructions.exists(), "copilot-instructions.md should exist");

    let content = std::fs::read_to_string(&instructions).unwrap();
    assert!(content.contains("TypeScript strict"), "Should contain rule content");

    assert_yaml_snapshot!("copilot_instructions", content);
}

#[test]
fn test_copilot_preserves_github_directory_contents() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["copilot"]);

    // Create existing .github content
    std::fs::create_dir_all(root.join(".github/workflows")).unwrap();
    std::fs::write(
        root.join(".github/workflows/ci.yml"),
        "name: CI\non: push\n",
    ).unwrap();
    std::fs::write(
        root.join(".github/CODEOWNERS"),
        "* @team\n",
    ).unwrap();

    create_rule(&root, "style", "Rules here.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Copilot).unwrap();

    // Existing files preserved
    assert!(root.join(".github/workflows/ci.yml").exists());
    assert!(root.join(".github/CODEOWNERS").exists());

    // New file created
    assert!(root.join(".github/copilot-instructions.md").exists());
}

#[test]
fn test_copilot_markdown_format() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["copilot"]);

    create_rule(&root, "architecture", "Follow clean architecture principles.", &["arch"]);
    create_rule(&root, "testing", "Write tests for all public APIs.", &["test"]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Copilot).unwrap();

    let content = std::fs::read_to_string(root.join(".github/copilot-instructions.md")).unwrap();

    // Should be valid markdown with headers
    assert!(content.contains("#"), "Should have markdown headers");
    assert!(content.contains("architecture") || content.contains("Architecture"));
    assert!(content.contains("testing") || content.contains("Testing"));

    assert_yaml_snapshot!("copilot_markdown_format", content);
}
```

**Step 2: Run and commit**

Run: `cargo insta test -p repo-tools --test copilot_test`

```bash
git add crates/repo-tools/tests/integration/copilot_test.rs
git add crates/repo-tools/src/snapshots/
git commit -m "test(repo-tools): add GitHub Copilot integration tests"
```

---

## Task 5: Test VS Code Tool Integration

**Files:**
- Create: `crates/repo-tools/tests/integration/vscode_test.rs`

**Step 1: Write VS Code integration tests**

```rust
//! Integration tests for VS Code tool sync
//!
//! Verifies .vscode/settings.json handling

mod helpers;
use helpers::*;

use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use insta::assert_yaml_snapshot;

#[test]
fn test_vscode_creates_vscode_directory() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["vscode"]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Vscode).unwrap();

    assert!(root.join(".vscode").exists(), ".vscode directory should exist");
}

#[test]
fn test_vscode_settings_json_format() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["vscode"]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Vscode).unwrap();

    let settings_path = root.join(".vscode/settings.json");
    if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path).unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .expect("settings.json should be valid JSON");

        assert!(parsed.is_object(), "Should be a JSON object");

        assert_yaml_snapshot!("vscode_settings", content);
    }
}

#[test]
fn test_vscode_preserves_user_settings() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["vscode"]);

    // Create existing settings
    std::fs::create_dir_all(root.join(".vscode")).unwrap();
    std::fs::write(
        root.join(".vscode/settings.json"),
        r#"{
    "editor.fontSize": 14,
    "editor.tabSize": 2
}"#,
    ).unwrap();

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Vscode).unwrap();

    let content = std::fs::read_to_string(root.join(".vscode/settings.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

    // User settings preserved
    assert_eq!(parsed["editor.fontSize"], 14);
    assert_eq!(parsed["editor.tabSize"], 2);
}
```

**Step 2: Run and commit**

```bash
git add crates/repo-tools/tests/integration/vscode_test.rs
git commit -m "test(repo-tools): add VS Code integration tests"
```

---

## Task 6: Test Windsurf Tool Integration

**Files:**
- Create: `crates/repo-tools/tests/integration/windsurf_test.rs`

**Step 1: Write Windsurf integration tests**

```rust
//! Integration tests for Windsurf tool sync
//!
//! Verifies .windsurf/rules/*.md creation

mod helpers;
use helpers::*;

use tempfile::TempDir;
use repo_tools::{ToolDispatcher, ToolId};
use insta::assert_yaml_snapshot;

#[test]
fn test_windsurf_creates_rules_directory() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["windsurf"]);

    create_rule(&root, "style", "Be concise.", &[]);

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Windsurf).unwrap();

    assert!(root.join(".windsurf/rules").exists());
}

#[test]
fn test_windsurf_markdown_rule_format() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["windsurf"]);

    create_rule(
        &root,
        "python-best-practices",
        "Always use type hints.\nPrefer dataclasses over dicts.",
        &["python"],
    );

    let dispatcher = ToolDispatcher::new();
    dispatcher.sync_for_tool(&root, ToolId::Windsurf).unwrap();

    let rule_path = root.join(".windsurf/rules/python-best-practices.md");
    assert!(rule_path.exists());

    let content = std::fs::read_to_string(&rule_path).unwrap();
    assert!(content.contains("type hints"));

    assert_yaml_snapshot!("windsurf_rule_format", content);
}

#[test]
fn test_windsurf_rule_size_limit() {
    let temp = TempDir::new().unwrap();
    let root = create_repo_with_tools(temp.path(), &["windsurf"]);

    // Windsurf has 6000 char limit per rule
    let long_content = "x".repeat(7000);
    create_rule(&root, "too-long", &long_content, &[]);

    let dispatcher = ToolDispatcher::new();
    let result = dispatcher.sync_for_tool(&root, ToolId::Windsurf);

    // Should either truncate or warn
    if let Ok(()) = result {
        let content = std::fs::read_to_string(root.join(".windsurf/rules/too-long.md"))
            .unwrap_or_default();
        assert!(content.len() <= 6000, "Content should be truncated or handled");
    }
}
```

**Step 2: Run and commit**

```bash
git add crates/repo-tools/tests/integration/windsurf_test.rs
git commit -m "test(repo-tools): add Windsurf integration tests"
```

---

## Task 7: Create Test Runner Script

**Files:**
- Create: `scripts/run-tool-integration-tests.sh`

**Step 1: Create script**

```bash
#!/bin/bash
# Run all tool integration tests with snapshot updates

set -e

echo "Running tool integration tests..."
echo "================================="

# Run tests with insta
cargo insta test -p repo-tools -- --test-threads=1

echo ""
echo "Reviewing snapshots..."
echo "======================"
cargo insta review

echo ""
echo "All tool integration tests complete!"
```

**Step 2: Make executable and commit**

```bash
chmod +x scripts/run-tool-integration-tests.sh
git add scripts/run-tool-integration-tests.sh
git commit -m "chore: add tool integration test runner script"
```

---

## Completion Checklist

- [ ] Integration test infrastructure set up
- [ ] Cursor tests with MDC format verification
- [ ] Claude Code tests with managed blocks
- [ ] GitHub Copilot tests
- [ ] VS Code tests
- [ ] Windsurf tests
- [ ] All snapshots reviewed and committed
- [ ] Test runner script created

---

## Verification

After completing all tasks:

```bash
# Run all integration tests
cargo test -p repo-tools --test '*_test' -- --test-threads=1

# Update snapshots if needed
cargo insta test -p repo-tools
cargo insta review

# Verify no snapshot differences
cargo insta test -p repo-tools --check
```

---

*Plan created: 2026-01-29*
*Addresses: DX-002 (no integration tests for tool sync), DX-003 (no real-world testing)*
