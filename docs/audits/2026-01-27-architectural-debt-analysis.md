# Architectural Debt Analysis

**Date:** 2026-01-27
**Analyst:** Claude Opus 4.5
**Scope:** Code duplication patterns vs. design specifications

---

## Executive Summary

Analysis of the Repository Manager codebase against design specifications reveals **3 high-priority architectural issues**, **2 medium-priority improvements**, and **1 critical functionality gap**. The issues fall into two categories:

1. **Design Pattern Violations** - Code that doesn't follow the specified architecture
2. **Acceptable Technical Debt** - Duplication that doesn't violate architecture but reduces maintainability

| Issue | Priority | Category | Estimated Lines Saved |
|-------|----------|----------|----------------------|
| Format Handler Duplication | HIGH | Design Violation | ~150 |
| Tool Integration Duplication | HIGH | Design Violation | ~100 |
| Git Layout Helper Extraction | MEDIUM | Technical Debt | ~80 |
| Dead Tree-sitter Code | LOW | Dead Code | ~10 |
| ToolIntegration Trait Mismatch | MEDIUM | API Drift | ~20 |
| Sync/Fix Incomplete | CRITICAL | Missing Functionality | N/A |

---

## Issue 1: Format Handler Duplication (HIGH)

### Observation

`PlainTextHandler` and `MarkdownHandler` in `repo-content` are 95% identical:

```
crates/repo-content/src/handlers/plaintext.rs  (185 lines)
crates/repo-content/src/handlers/markdown.rs   (239 lines)
```

Shared code (~150 lines):
- `BLOCK_START_PATTERN` regex (identical)
- `find_blocks()` implementation (identical)
- `insert_block()` implementation (identical)
- `update_block()` implementation (identical)
- `remove_block()` implementation (identical)

Only differences in MarkdownHandler:
- Unused tree-sitter initialization
- `MULTIPLE_BLANK_LINES` regex for normalization

### Design Spec Analysis

**Reference:** `docs/plans/2026-01-23-repo-content-design.md`

The design specified:
```rust
pub enum CommentStyle {
    Html,   // <!-- comment -->
    Hash,   // # comment
    None,   // JSON uses _repo_managed key
}
```

The `CommentStyle` abstraction exists in `format.rs:94-122` with `format_start()` and `format_end()` methods, but handlers don't use shared block operations.

### Root Cause

The handlers were implemented independently rather than delegating to a shared `HtmlCommentBlockHandler`. The design implied this abstraction but didn't make it explicit.

### Recommended Solution

Create shared module:

```rust
// handlers/html_comment.rs
pub fn find_blocks_html(source: &str) -> Vec<ManagedBlock>;
pub fn insert_block_html(source: &str, uuid: Uuid, content: &str, location: BlockLocation) -> Result<(String, Edit)>;
pub fn update_block_html(source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)>;
pub fn remove_block_html(source: &str, uuid: Uuid) -> Result<(String, Edit)>;
```

Handlers become thin wrappers:

```rust
impl FormatHandler for PlainTextHandler {
    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        html_comment::find_blocks_html(source)
    }
    // ... delegate other methods
}
```

---

## Issue 2: Tool Integration Duplication (HIGH)

### Observation

`CursorIntegration` and `ClaudeIntegration` have nearly identical implementations:

```rust
// cursor.rs:42-57
fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
    let path = context.root.join(".cursorrules");
    let mut content = Self::load_content(&path);
    for rule in rules {
        content = upsert_block(&content, &rule.id, &rule.content)?;
    }
    io::write_text(&path, &content)?;
    Ok(())
}

// claude.rs:42-57 - IDENTICAL except path
fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
    let path = context.root.join("CLAUDE.md");
    let mut content = Self::load_content(&path);
    for rule in rules {
        content = upsert_block(&content, &rule.id, &rule.content)?;
    }
    io::write_text(&path, &content)?;
    Ok(())
}
```

### Design Spec Analysis

**Reference:** `docs/design/spec-tools.md`

> "Generic tool integration driven by ToolDefinition schema. This allows new tools to be added without writing Rust code."

The `GenericToolIntegration` in `generic.rs` was designed exactly for this purpose! It handles:
- Text-based configs with managed blocks
- JSON configs with schema keys
- Markdown configs

### Root Cause

Built-in tools were implemented before `GenericToolIntegration` was complete, creating parallel code paths. The built-in tools should use `GenericToolIntegration` with embedded `ToolDefinition` structs.

### Recommended Solution

Replace dedicated structs with factory functions:

```rust
// cursor.rs
pub fn cursor_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Cursor".into(),
            slug: "cursor".into(),
            description: Some("Cursor AI IDE".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".cursorrules".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities::default(),
        schema_keys: None,
    })
}
```

This unifies all tools under one code path and enables future TOML-based tool definitions.

---

## Issue 3: Git Layout Helper Extraction (MEDIUM)

### Observation

`ContainerLayout` and `InRepoWorktreesLayout` share ~80 lines of identical git2 boilerplate:

| Method | Lines | Identical? |
|--------|-------|------------|
| `create_feature()` | 45 | 95% |
| `remove_feature()` | 30 | 100% |
| `current_branch()` | 8 | 100% |

### Design Spec Analysis

**Reference:** `docs/design/architecture-core.md`

> "Traits/Interfaces for RepositoryBackend"

The `LayoutProvider` trait abstraction is **architecturally correct**. The duplication is in git2 API boilerplate, not business logic.

### Assessment

This is **acceptable technical debt**. The architecture is sound; the issue is implementation convenience. Extracting helpers would improve maintainability but isn't blocking.

### Recommended Solution (Optional)

```rust
// git/helpers.rs
pub fn create_worktree_with_branch(
    repo: &Repository,
    worktree_path: &Path,
    branch_name: &str,
    base: Option<&str>,
) -> Result<()>;

pub fn remove_worktree_and_branch(repo: &Repository, name: &str) -> Result<()>;
pub fn get_current_branch(repo: &Repository) -> Result<String>;
```

---

## Issue 4: Dead Tree-sitter Code (LOW)

### Observation

`MarkdownHandler` initializes tree-sitter but never uses it:

```rust
#[derive(Default)]
pub struct MarkdownHandler {
    #[allow(dead_code)]
    parser_initialized: bool,  // Never read
}

impl MarkdownHandler {
    pub fn new() -> Self {
        let mut parser = tree_sitter::Parser::new();
        let initialized = parser.set_language(&tree_sitter_md::LANGUAGE.into()).is_ok();
        Self { parser_initialized: initialized }  // Discarded immediately
    }
}
```

### Recommendation

Remove dead code. Tree-sitter queries can be added later when "advanced queries" are needed per the design spec.

---

## Issue 5: ToolIntegration Trait API Drift (MEDIUM)

### Observation

The trait doesn't match the spec:

**Spec (`docs/design/spec-tools.md`):**
```rust
fn config_lines(&self) -> Vec<ConfigLocation>;
```

**Implementation (`integration.rs`):**
```rust
fn config_paths(&self) -> Vec<&str>;
```

### Missing Type

```rust
pub struct ConfigLocation {
    pub path: String,
    pub config_type: ConfigType,
    pub is_directory: bool,
}
```

### Impact

The dispatcher can't know the config type without parsing the path or hardcoding knowledge.

---

## Issue 6: Sync/Fix Operations Incomplete (CRITICAL)

### Observation

The core value proposition is not delivered:

| Command | Expected | Actual |
|---------|----------|--------|
| `repo sync` | Apply projections to filesystem | Creates ledger only |
| `repo fix` | Repair drift | Calls sync (stub) |
| `repo add-tool` | Add + sync | Adds to config only |

### Design Reference

**`docs/design/architecture-core.md`:**
> "You declare your intent, and the Repository Manager 'unrolls' this intent into tool-specific configs."

### Gap

The `ProjectionWriter` is implemented but `SyncEngine.sync()` doesn't call it. The "unroll" step is missing.

---

## Prioritized Action Plan

### Phase 1: Critical Functionality (sync/fix)
- Wire ProjectionWriter into SyncEngine
- Implement actual fix() logic
- Auto-sync on tool add/remove

### Phase 2: High-Priority Refactoring
- Extract HtmlCommentBlockHandler
- Consolidate tool integrations via GenericToolIntegration

### Phase 3: Medium-Priority Improvements
- Extract git2 helper functions
- Align ToolIntegration trait with spec

### Phase 4: Cleanup
- Remove dead tree-sitter code
- Remove redundant from_definition() alias

---

## Appendix: Files Analyzed

```
crates/repo-content/src/handlers/plaintext.rs
crates/repo-content/src/handlers/markdown.rs
crates/repo-content/src/format.rs
crates/repo-tools/src/cursor.rs
crates/repo-tools/src/claude.rs
crates/repo-tools/src/generic.rs
crates/repo-tools/src/integration.rs
crates/repo-git/src/container.rs
crates/repo-git/src/in_repo_worktrees.rs
crates/repo-git/src/provider.rs
crates/repo-core/src/sync/engine.rs
```
