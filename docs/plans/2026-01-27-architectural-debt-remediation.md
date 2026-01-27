# Architectural Debt Remediation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate code duplication and design violations identified in the architectural audit.

**Architecture:** Extract shared abstractions for HTML comment block operations, consolidate tool integrations via GenericToolIntegration, and clean up dead code. Each phase is independent and can be executed separately.

**Tech Stack:** Rust, regex, repo-content handlers, repo-tools integrations

---

## Phase 1: Extract HtmlCommentBlockHandler (HIGH PRIORITY)

### Task 1.1: Create html_comment module with shared functions

**Files:**
- Create: `crates/repo-content/src/handlers/html_comment.rs`
- Modify: `crates/repo-content/src/handlers/mod.rs`

**Step 1: Create the shared module file**

```rust
// crates/repo-content/src/handlers/html_comment.rs
//! Shared HTML comment block operations for PlainText and Markdown handlers

use std::sync::LazyLock;
use regex::Regex;
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::{Edit, EditKind};
use crate::error::{Error, Result};
use crate::format::CommentStyle;

/// Pattern to match block start markers and capture the UUID
pub static BLOCK_START_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<!--\s*repo:block:([0-9a-f-]{36})\s*-->").unwrap()
});

/// Find all managed blocks using HTML comment markers
pub fn find_blocks(source: &str) -> Vec<ManagedBlock> {
    let mut blocks = Vec::new();

    for cap in BLOCK_START_PATTERN.captures_iter(source) {
        let uuid_str = match cap.get(1) {
            Some(m) => m.as_str(),
            None => continue,
        };
        let uuid = match Uuid::parse_str(uuid_str) {
            Ok(u) => u,
            Err(_) => continue,
        };

        let start_match = cap.get(0).unwrap();
        let block_start = start_match.start();
        let content_start = start_match.end();

        // Find the corresponding end marker
        let end_marker = format!("<!-- /repo:block:{uuid} -->");
        let Some(end_pos) = source[content_start..].find(&end_marker) else {
            continue;
        };
        let end_pos = content_start + end_pos;
        let block_end = end_pos + end_marker.len();

        // Skip trailing newline if present
        let block_end = if source[block_end..].starts_with('\n') {
            block_end + 1
        } else {
            block_end
        };

        // Extract content between markers (skip leading newline if present)
        let content = &source[content_start..end_pos];
        let content = content.strip_prefix('\n').unwrap_or(content);

        blocks.push(ManagedBlock::new(uuid, content, block_start..block_end));
    }

    blocks
}

/// Insert a managed block using HTML comment markers
pub fn insert_block(
    source: &str,
    uuid: Uuid,
    content: &str,
    location: BlockLocation,
) -> Result<(String, Edit)> {
    let style = CommentStyle::Html;
    let block_text = format!(
        "{}\n{}\n{}\n",
        style.format_start(uuid),
        content,
        style.format_end(uuid)
    );

    let position = match location {
        BlockLocation::End => source.len(),
        BlockLocation::Offset(pos) => pos.min(source.len()),
        BlockLocation::After(ref marker) => source
            .find(marker)
            .map(|p| p + marker.len())
            .unwrap_or(source.len()),
        BlockLocation::Before(ref marker) => source.find(marker).unwrap_or(source.len()),
    };

    let mut result = String::with_capacity(source.len() + block_text.len());
    result.push_str(&source[..position]);
    if position > 0 && !source[..position].ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&block_text);
    result.push_str(&source[position..]);

    let edit = Edit {
        kind: EditKind::BlockInsert { uuid },
        span: position..position + block_text.len(),
        old_content: String::new(),
        new_content: block_text,
    };

    Ok((result, edit))
}

/// Update a managed block using HTML comment markers
pub fn update_block(source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
    let blocks = find_blocks(source);
    let block = blocks
        .iter()
        .find(|b| b.uuid == uuid)
        .ok_or(Error::BlockNotFound { uuid })?;

    let style = CommentStyle::Html;
    let new_block = format!(
        "{}\n{}\n{}",
        style.format_start(uuid),
        content,
        style.format_end(uuid)
    );

    let edit = Edit {
        kind: EditKind::BlockUpdate { uuid },
        span: block.span.clone(),
        old_content: source[block.span.clone()].to_string(),
        new_content: new_block.clone(),
    };

    let result = edit.apply(source);
    Ok((result, edit))
}

/// Remove a managed block using HTML comment markers
pub fn remove_block(source: &str, uuid: Uuid) -> Result<(String, Edit)> {
    let blocks = find_blocks(source);
    let block = blocks
        .iter()
        .find(|b| b.uuid == uuid)
        .ok_or(Error::BlockNotFound { uuid })?;

    let edit = Edit {
        kind: EditKind::BlockRemove { uuid },
        span: block.span.clone(),
        old_content: source[block.span.clone()].to_string(),
        new_content: String::new(),
    };

    let result = edit.apply(source);
    Ok((result, edit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_start_pattern_matches() {
        let source = "<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->";
        assert!(BLOCK_START_PATTERN.is_match(source));
    }

    #[test]
    fn test_find_blocks_empty() {
        let blocks = find_blocks("no blocks here");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_find_blocks_single() {
        let source = "prefix\n<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->\ncontent\n<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->\nsuffix";
        let blocks = find_blocks(source);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "content");
    }

    #[test]
    fn test_insert_block_at_end() {
        let (result, _edit) = insert_block(
            "existing content",
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            "new block",
            BlockLocation::End,
        ).unwrap();
        assert!(result.contains("existing content"));
        assert!(result.contains("new block"));
        assert!(result.contains("<!-- repo:block:550e8400"));
    }
}
```

**Step 2: Update mod.rs to export the module**

```rust
// Add to crates/repo-content/src/handlers/mod.rs
mod html_comment;
pub use html_comment::{find_blocks as find_html_blocks, insert_block as insert_html_block, update_block as update_html_block, remove_block as remove_html_block};
```

**Step 3: Run tests to verify**

Run: `cargo test -p repo-content html_comment`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-content/src/handlers/html_comment.rs crates/repo-content/src/handlers/mod.rs
git commit -m "feat(repo-content): add shared html_comment block operations"
```

---

### Task 1.2: Refactor PlainTextHandler to use shared module

**Files:**
- Modify: `crates/repo-content/src/handlers/plaintext.rs`

**Step 1: Replace implementations with delegations**

```rust
// crates/repo-content/src/handlers/plaintext.rs
//! Plain text format handler

use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

use super::html_comment;

/// Handler for plain text files with HTML comment markers
#[derive(Debug, Default)]
pub struct PlainTextHandler;

impl PlainTextHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for PlainTextHandler {
    fn format(&self) -> Format {
        Format::PlainText
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        Ok(Box::new(source.to_string()))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        html_comment::find_blocks(source)
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)> {
        html_comment::insert_block(source, uuid, content, location)
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        html_comment::update_block(source, uuid, content)
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        html_comment::remove_block(source, uuid)
    }

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        let normalized: String = source
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        Ok(serde_json::Value::String(normalized))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<String>()
            .cloned()
            .ok_or_else(|| Error::parse("plaintext", "invalid internal state"))
    }
}
```

**Step 2: Run tests to verify**

Run: `cargo test -p repo-content plaintext`
Expected: All existing tests pass

**Step 3: Commit**

```bash
git add crates/repo-content/src/handlers/plaintext.rs
git commit -m "refactor(repo-content): PlainTextHandler delegates to html_comment"
```

---

### Task 1.3: Refactor MarkdownHandler to use shared module

**Files:**
- Modify: `crates/repo-content/src/handlers/markdown.rs`

**Step 1: Replace implementations with delegations, keep markdown-specific logic**

```rust
// crates/repo-content/src/handlers/markdown.rs
//! Markdown format handler

use std::sync::LazyLock;
use regex::Regex;
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::Edit;
use crate::error::{Error, Result};
use crate::format::{Format, FormatHandler};

use super::html_comment;

/// Pattern to match multiple consecutive blank lines
static MULTIPLE_BLANK_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

/// Handler for Markdown files with HTML comment markers
#[derive(Debug, Default)]
pub struct MarkdownHandler;

impl MarkdownHandler {
    pub fn new() -> Self {
        Self
    }
}

impl FormatHandler for MarkdownHandler {
    fn format(&self) -> Format {
        Format::Markdown
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        Ok(Box::new(source.to_string()))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        html_comment::find_blocks(source)
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)> {
        html_comment::insert_block(source, uuid, content, location)
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        html_comment::update_block(source, uuid, content)
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        html_comment::remove_block(source, uuid)
    }

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        // Markdown-specific: collapse multiple blank lines
        let mut normalized: String = source
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        normalized = MULTIPLE_BLANK_LINES.replace_all(&normalized, "\n\n").to_string();
        normalized = normalized.trim().to_string();

        Ok(serde_json::Value::String(normalized))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<String>()
            .cloned()
            .ok_or_else(|| Error::parse("markdown", "invalid internal state"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_blank_lines_pattern() {
        let source = "a\n\n\n\nb";
        let result = MULTIPLE_BLANK_LINES.replace_all(source, "\n\n");
        assert_eq!(result, "a\n\nb");
    }
}
```

**Step 2: Run tests to verify**

Run: `cargo test -p repo-content markdown`
Expected: All existing tests pass

**Step 3: Run full repo-content tests**

Run: `cargo test -p repo-content`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-content/src/handlers/markdown.rs
git commit -m "refactor(repo-content): MarkdownHandler delegates to html_comment"
```

---

## Phase 2: Consolidate Tool Integrations (HIGH PRIORITY)

### Task 2.1: Create embedded ToolDefinition for Cursor

**Files:**
- Modify: `crates/repo-tools/src/cursor.rs`

**Step 1: Replace CursorIntegration struct with factory function**

```rust
// crates/repo-tools/src/cursor.rs
//! Cursor integration for Repository Manager.

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Create the Cursor tool integration
pub fn cursor_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Cursor".to_string(),
            slug: "cursor".to_string(),
            description: Some("Cursor AI IDE with .cursorrules support".to_string()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".cursorrules".to_string(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_rules: true,
            supports_mcp: false,
            supports_system_prompt: true,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{Rule, SyncContext, ToolIntegration};
    use repo_fs::NormalizedPath;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = cursor_integration();
        assert_eq!(integration.name(), "cursor");
    }

    #[test]
    fn test_config_paths() {
        let integration = cursor_integration();
        let paths = integration.config_paths();
        assert_eq!(paths, vec![".cursorrules"]);
    }

    #[test]
    fn test_sync_creates_cursorrules() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![
            Rule {
                id: "rule-1".to_string(),
                content: "First rule content".to_string(),
            },
        ];

        let integration = cursor_integration();
        integration.sync(&context, &rules).unwrap();

        let cursorrules_path = temp_dir.path().join(".cursorrules");
        assert!(cursorrules_path.exists());

        let content = fs::read_to_string(&cursorrules_path).unwrap();
        assert!(content.contains("rule-1"));
        assert!(content.contains("First rule content"));
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p repo-tools cursor`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/repo-tools/src/cursor.rs
git commit -m "refactor(repo-tools): CursorIntegration uses GenericToolIntegration"
```

---

### Task 2.2: Create embedded ToolDefinition for Claude

**Files:**
- Modify: `crates/repo-tools/src/claude.rs`

**Step 1: Replace ClaudeIntegration struct with factory function**

```rust
// crates/repo-tools/src/claude.rs
//! Claude integration for Repository Manager.

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Create the Claude tool integration
pub fn claude_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Claude".to_string(),
            slug: "claude".to_string(),
            description: Some("Claude AI with CLAUDE.md support".to_string()),
        },
        integration: ToolIntegrationConfig {
            config_path: "CLAUDE.md".to_string(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![".claude/rules/".to_string()],
        },
        capabilities: ToolCapabilities {
            supports_rules: true,
            supports_mcp: true,
            supports_system_prompt: true,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{Rule, SyncContext, ToolIntegration};
    use repo_fs::NormalizedPath;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = claude_integration();
        assert_eq!(integration.name(), "claude");
    }

    #[test]
    fn test_config_paths() {
        let integration = claude_integration();
        let paths = integration.config_paths();
        assert_eq!(paths, vec!["CLAUDE.md", ".claude/rules/"]);
    }

    #[test]
    fn test_sync_creates_claude_md() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![
            Rule {
                id: "project-context".to_string(),
                content: "This is a Rust project using cargo.".to_string(),
            },
        ];

        let integration = claude_integration();
        integration.sync(&context, &rules).unwrap();

        let claude_md_path = temp_dir.path().join("CLAUDE.md");
        assert!(claude_md_path.exists());

        let content = fs::read_to_string(&claude_md_path).unwrap();
        assert!(content.contains("project-context"));
        assert!(content.contains("This is a Rust project"));
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p repo-tools claude`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/repo-tools/src/claude.rs
git commit -m "refactor(repo-tools): ClaudeIntegration uses GenericToolIntegration"
```

---

### Task 2.3: Update dispatcher to use factory functions

**Files:**
- Modify: `crates/repo-tools/src/dispatcher.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Update dispatcher registration**

Find the tool registration code in dispatcher.rs and update it to use the new factory functions:

```rust
// In the dispatcher initialization, replace:
// tools.insert("cursor".to_string(), Box::new(CursorIntegration::new()));
// tools.insert("claude".to_string(), Box::new(ClaudeIntegration::new()));

// With:
tools.insert("cursor".to_string(), Box::new(cursor::cursor_integration()));
tools.insert("claude".to_string(), Box::new(claude::claude_integration()));
```

**Step 2: Update lib.rs exports**

```rust
// Update exports in lib.rs
pub use cursor::cursor_integration;
pub use claude::claude_integration;
// Remove: pub use cursor::CursorIntegration;
// Remove: pub use claude::ClaudeIntegration;
```

**Step 3: Run all repo-tools tests**

Run: `cargo test -p repo-tools`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/dispatcher.rs crates/repo-tools/src/lib.rs
git commit -m "refactor(repo-tools): update dispatcher to use tool factory functions"
```

---

## Phase 3: Clean Up Dead Code (LOW PRIORITY)

### Task 3.1: Remove dead tree-sitter code from MarkdownHandler

This was already done in Task 1.3 - the refactored MarkdownHandler no longer has the dead `parser_initialized` field.

**Verify:** Check that the new MarkdownHandler doesn't have `#[allow(dead_code)]` attributes.

---

### Task 3.2: Remove redundant from_definition alias

**Files:**
- Modify: `crates/repo-tools/src/generic.rs`

**Step 1: Remove the alias method**

```rust
// Remove these lines from generic.rs:
// /// Create from a definition (alias for new).
// pub fn from_definition(definition: ToolDefinition) -> Self {
//     Self::new(definition)
// }
```

**Step 2: Search for usages**

Run: `grep -r "from_definition" crates/`
Expected: No usages found

**Step 3: Run tests**

Run: `cargo test -p repo-tools`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/generic.rs
git commit -m "refactor(repo-tools): remove redundant from_definition alias"
```

---

## Phase 4: Extract Git Helpers (MEDIUM PRIORITY - OPTIONAL)

### Task 4.1: Create git helpers module

**Files:**
- Create: `crates/repo-git/src/helpers.rs`
- Modify: `crates/repo-git/src/lib.rs`

**Step 1: Extract shared worktree operations**

```rust
// crates/repo-git/src/helpers.rs
//! Shared git2 helper functions for worktree operations

use git2::{BranchType, Repository, WorktreeAddOptions, WorktreePruneOptions};
use std::path::Path;
use crate::{Error, Result};

/// Create a worktree with a new branch
pub fn create_worktree_with_branch(
    repo: &Repository,
    worktree_path: &Path,
    branch_name: &str,
    base: Option<&str>,
) -> Result<()> {
    // Get the commit to base the new branch on
    let base_commit = match base {
        Some(base_name) => {
            let branch = repo
                .find_branch(base_name, BranchType::Local)
                .map_err(|_| Error::BranchNotFound {
                    name: base_name.to_string(),
                })?;
            branch.get().peel_to_commit()?
        }
        None => {
            let head = repo.head()?;
            head.peel_to_commit()?
        }
    };

    // Create a new branch for the feature worktree
    let new_branch = repo.branch(branch_name, &base_commit, false)?;
    let new_branch_ref = new_branch.into_reference();

    // Create worktree with the new branch
    let mut opts = WorktreeAddOptions::new();
    opts.reference(Some(&new_branch_ref));

    repo.worktree(branch_name, worktree_path, Some(&opts))?;

    Ok(())
}

/// Remove a worktree and its branch
pub fn remove_worktree_and_branch(repo: &Repository, name: &str) -> Result<()> {
    let wt = repo
        .find_worktree(name)
        .map_err(|_| Error::WorktreeNotFound {
            name: name.to_string(),
        })?;

    // Configure prune options
    let mut prune_opts = WorktreePruneOptions::new();
    prune_opts.valid(true);
    prune_opts.working_tree(true);

    // Prune the worktree
    wt.prune(Some(&mut prune_opts))?;

    // Try to delete the branch
    if let Ok(mut branch) = repo.find_branch(name, BranchType::Local) {
        if let Err(e) = branch.delete() {
            tracing::warn!(
                branch = %name,
                error = %e,
                "Failed to delete branch after worktree removal"
            );
        }
    }

    Ok(())
}

/// Get the current branch name
pub fn get_current_branch(repo: &Repository) -> Result<String> {
    let head = repo.head()?;
    if head.is_branch() {
        Ok(head.shorthand().unwrap_or("HEAD").to_string())
    } else {
        Ok("HEAD".to_string())
    }
}
```

**Step 2: Update lib.rs**

```rust
// Add to crates/repo-git/src/lib.rs
mod helpers;
pub use helpers::{create_worktree_with_branch, remove_worktree_and_branch, get_current_branch};
```

**Step 3: Update ContainerLayout to use helpers**

Replace the duplicated code in `container.rs` with calls to the helper functions.

**Step 4: Update InRepoWorktreesLayout to use helpers**

Replace the duplicated code in `in_repo_worktrees.rs` with calls to the helper functions.

**Step 5: Run tests**

Run: `cargo test -p repo-git`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/repo-git/src/helpers.rs crates/repo-git/src/lib.rs crates/repo-git/src/container.rs crates/repo-git/src/in_repo_worktrees.rs
git commit -m "refactor(repo-git): extract shared worktree helper functions"
```

---

## Verification Checklist

After completing all phases:

- [ ] `cargo test --workspace` - All tests pass
- [ ] `cargo clippy --workspace` - No warnings
- [ ] `cargo build --release` - Builds successfully
- [ ] Lines of code reduced by ~250 lines

---

## Summary

| Phase | Tasks | Lines Saved | Priority |
|-------|-------|-------------|----------|
| 1: HTML Comment Handler | 3 | ~150 | HIGH |
| 2: Tool Integration | 3 | ~100 | HIGH |
| 3: Dead Code Cleanup | 2 | ~20 | LOW |
| 4: Git Helpers | 1 | ~80 | MEDIUM |

**Total estimated lines saved:** ~350
