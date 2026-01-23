# repo-content Crate Design

> **Status:** Ready for implementation
> **Date:** 2026-01-23
> **Scope:** Content parsing, editing, diffing with semantic understanding

## Overview

`repo-content` provides robust content operations for Repository Manager - reading, writing, editing, matching, and diffing files with semantic understanding. It is the foundation for the managed block system and config sync functionality.

This crate directly addresses the "MAJOR RESEARCH ITEM" identified in the core building blocks design document.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      repo-content                           │
├─────────────────────────────────────────────────────────────┤
│  Unified API: Document, ManagedBlock, SemanticDiff          │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Format Handlers (impl FormatHandler)                        │
├─────────────────┼─────────────────┼─────────────────────────┤
│ TomlHandler     │ JsonHandler     │ MarkdownHandler         │
│ (toml_edit)     │ (tree-sitter)   │ (tree-sitter-md)        │
├─────────────────┼─────────────────┼─────────────────────────┤
│ YamlHandler     │ PlainTextHandler│ ExtensionPoint          │
│ (yaml-edit)     │ (regex markers) │ (custom trait impl)     │
└─────────────────┴─────────────────┴─────────────────────────┘
             ↓                              ↓
      Format-specific           tree-sitter (query/match)
      editing backends          similar (diffing)
```

### Design Decisions

1. **Hybrid Parsing Strategy**: Use format-specific crates (`toml_edit`, `yaml-edit`) for format-preserving editing, tree-sitter for querying and Markdown parsing.

2. **Semantic + Structural Matching**: Ignore whitespace, formatting, and key ordering when comparing documents.

3. **Plugin Architecture**: Core formats built-in with `FormatHandler` trait for extensibility.

## Core Types

### Document

```rust
/// Represents parsed content with semantic understanding
pub struct Document {
    /// Original source text
    source: String,
    /// Detected or declared format
    format: Format,
    /// Format-specific parsed representation
    inner: DocumentInner,
}

enum DocumentInner {
    Toml(toml_edit::DocumentMut),
    Yaml(yaml_edit::Document),
    Json { tree: tree_sitter::Tree, source: String },
    Markdown { tree: tree_sitter::Tree, source: String },
    PlainText(String),
}

pub enum Format {
    Toml,
    Yaml,
    Json,
    Markdown,
    PlainText,
    Custom(String),
}
```

### ManagedBlock

```rust
/// A managed block with UUID marker
pub struct ManagedBlock {
    /// Unique identifier for this block
    pub uuid: Uuid,
    /// Content within the block (excluding markers)
    pub content: String,
    /// Byte range in original source
    pub span: Range<usize>,
    /// SHA-256 checksum for drift detection
    pub checksum: String,
}

/// Where to insert a block in a document
pub enum BlockLocation {
    /// Append to end of document
    End,
    /// After specific section/key
    After(String),
    /// Before specific section/key
    Before(String),
    /// At specific byte offset
    Offset(usize),
}
```

### Edit and Rollback

```rust
/// Represents a reversible edit operation
pub struct Edit {
    pub kind: EditKind,
    /// Byte range affected
    pub span: Range<usize>,
    /// Original content (for rollback)
    pub old_content: String,
    /// New content
    pub new_content: String,
}

pub enum EditKind {
    Insert,
    Delete,
    Replace,
    BlockInsert { uuid: Uuid },
    BlockUpdate { uuid: Uuid },
    BlockRemove { uuid: Uuid },
    PathSet { path: String },
    PathRemove { path: String },
}

impl Edit {
    /// Create the inverse edit for rollback
    pub fn inverse(&self) -> Edit;
}
```

### SemanticDiff

```rust
/// Result of comparing two documents semantically
pub struct SemanticDiff {
    /// Are the documents semantically equivalent?
    pub is_equivalent: bool,
    /// List of semantic changes
    pub changes: Vec<SemanticChange>,
    /// Similarity ratio (0.0 to 1.0)
    pub similarity: f64,
}

pub enum SemanticChange {
    /// Key/path added
    Added { path: String, value: serde_json::Value },
    /// Key/path removed
    Removed { path: String, value: serde_json::Value },
    /// Value changed at path
    Modified { path: String, old: serde_json::Value, new: serde_json::Value },
    /// Block added (for Markdown/text)
    BlockAdded { uuid: Option<Uuid>, content: String },
    /// Block removed
    BlockRemoved { uuid: Option<Uuid>, content: String },
    /// Block content changed
    BlockModified { uuid: Option<Uuid>, old: String, new: String },
}
```

## Format Handler Trait

```rust
/// Trait for format handlers - extensibility point
pub trait FormatHandler: Send + Sync {
    /// Format identifier
    fn id(&self) -> &str;

    /// File extensions this handler supports
    fn extensions(&self) -> &[&str];

    /// Parse source into document inner
    fn parse(&self, source: &str) -> Result<DocumentInner>;

    /// Comment syntax for managed block markers
    fn comment_style(&self) -> CommentStyle;

    /// Find managed blocks in parsed document
    fn find_blocks(&self, doc: &DocumentInner, source: &str) -> Vec<ManagedBlock>;

    /// Insert a managed block
    fn insert_block(&self, doc: &mut DocumentInner, block: &ManagedBlock, location: BlockLocation) -> Result<Edit>;

    /// Remove a managed block
    fn remove_block(&self, doc: &mut DocumentInner, uuid: Uuid) -> Result<Edit>;

    /// Render document back to string
    fn render(&self, doc: &DocumentInner) -> String;

    /// Normalize for semantic comparison
    fn normalize(&self, doc: &DocumentInner) -> serde_json::Value;
}

pub enum CommentStyle {
    /// HTML-style: <!-- comment -->
    Html,
    /// Hash: # comment
    Hash,
    /// Slash: // comment
    Slash,
    /// None (embed in data structure)
    None,
}
```

## Document API

```rust
impl Document {
    // === Parsing ===

    /// Parse content with format auto-detection
    pub fn parse(source: &str) -> Result<Self>;

    /// Parse with explicit format
    pub fn parse_as(source: &str, format: Format) -> Result<Self>;

    /// Parse file with format from extension
    pub fn from_file(path: &NormalizedPath) -> Result<Self>;

    // === Managed Blocks ===

    /// Find all managed blocks
    pub fn find_blocks(&self) -> Vec<ManagedBlock>;

    /// Get block by UUID
    pub fn get_block(&self, uuid: Uuid) -> Option<&ManagedBlock>;

    /// Insert a new managed block
    pub fn insert_block(&mut self, uuid: Uuid, content: &str, location: BlockLocation) -> Result<Edit>;

    /// Update existing block content
    pub fn update_block(&mut self, uuid: Uuid, content: &str) -> Result<Edit>;

    /// Remove block by UUID
    pub fn remove_block(&mut self, uuid: Uuid) -> Result<Edit>;

    // === Structured Data (TOML/JSON/YAML) ===

    /// Get value at dot-separated path
    pub fn get_path(&self, path: &str) -> Option<serde_json::Value>;

    /// Set value at path, preserving surrounding structure
    pub fn set_path(&mut self, path: &str, value: impl Into<serde_json::Value>) -> Result<Edit>;

    /// Remove key at path
    pub fn remove_path(&mut self, path: &str) -> Result<Edit>;

    // === Comparison ===

    /// Check semantic equality (ignores formatting)
    pub fn semantic_eq(&self, other: &Document) -> bool;

    /// Compute semantic diff
    pub fn diff(&self, other: &Document) -> SemanticDiff;

    /// Check if document has changed from original
    pub fn is_modified(&self) -> bool;

    // === Rendering ===

    /// Render to string
    pub fn render(&self) -> String;

    /// Get format
    pub fn format(&self) -> Format;

    /// Get original source
    pub fn source(&self) -> &str;
}
```

## Managed Block Marker Formats

### Markdown / Plain Text (HTML comments)
```markdown
<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Block content here
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->
```

### TOML (Hash comments)
```toml
# repo:block:550e8400-e29b-41d4-a716-446655440000
[managed.section]
key = "value"
# /repo:block:550e8400-e29b-41d4-a716-446655440000
```

### YAML (Hash comments)
```yaml
# repo:block:550e8400-e29b-41d4-a716-446655440000
managed_section:
  key: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000
```

### JSON (Special key - no comment support)
```json
{
  "_repo_managed": {
    "550e8400-e29b-41d4-a716-446655440000": {
      "key": "value"
    }
  }
}
```

## Semantic Comparison Rules

### Normalization (applied before comparison)

| Format | Normalization Rules |
|--------|---------------------|
| **All** | Trim trailing whitespace, normalize line endings to LF |
| **JSON** | Sort object keys alphabetically, remove `_repo_managed` metadata |
| **TOML** | Sort keys alphabetically, normalize inline tables |
| **YAML** | Sort keys alphabetically, normalize flow/block style |
| **Markdown** | Collapse multiple blank lines, normalize heading levels |

### Equivalence

Two documents are **semantically equivalent** if their normalized representations are identical. This means:
- `{"a":1,"b":2}` equals `{ "b": 2, "a": 1 }`
- TOML with different whitespace but same keys/values are equal
- Markdown with different line wrapping but same content is equal

## Dependencies

```toml
[package]
name = "repo-content"
version = "0.1.0"
edition = "2024"
rust-version = "1.85"

[dependencies]
# Internal
repo-fs = { path = "../repo-fs" }

# Format-preserving editors
toml_edit = "0.23"              # TOML (0.23.7) - 354M downloads
yaml-edit = "0.1"               # YAML with rowan syntax trees

# tree-sitter (queries, Markdown)
tree-sitter = "0.26"            # Core (0.26.3)
tree-sitter-md = "0.5"          # Markdown (0.5.1)
tree-sitter-json = "0.24"       # JSON queries

# Diffing
similar = "2.7"                 # Diff algorithms (2.7.0)

# Data handling
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Utilities
uuid = { version = "1.0", features = ["v4"] }
thiserror = "2.0"               # Error handling (2.0.17)
sha2 = "0.10"                   # Checksums
regex = "1.10"                  # Plain text markers

[dev-dependencies]
insta = { version = "1.40", features = ["yaml"] }
proptest = "1.5"
tempfile = "3.10"
```

## Implementation Phases

### Phase 1: Core Infrastructure
- [ ] `Document` type with format detection
- [ ] `FormatHandler` trait definition
- [ ] `ManagedBlock` struct and marker parsing
- [ ] Basic round-trip tests

### Phase 2: Format Handlers
- [ ] `TomlHandler` using toml_edit
- [ ] `YamlHandler` using yaml-edit
- [ ] `JsonHandler` using tree-sitter
- [ ] `MarkdownHandler` using tree-sitter-md
- [ ] `PlainTextHandler` with regex

### Phase 3: Managed Block Operations
- [ ] Block extraction across formats
- [ ] Block insertion with location hints
- [ ] Block update and removal
- [ ] Checksum generation and drift detection

### Phase 4: Semantic Diffing
- [ ] Normalization functions per format
- [ ] Integration with `similar` crate
- [ ] `SemanticDiff` computation
- [ ] Similarity scoring

### Phase 5: Rollback & Transactions
- [ ] `Edit` type with inverse operations
- [ ] Transaction batching
- [ ] Integration with repo-fs atomic writes

### Phase 6: Integration & Polish
- [ ] Wire into repo-fs infrastructure
- [ ] Public API documentation
- [ ] Examples and usage guide
- [ ] Performance benchmarks

## Acceptance Criteria

### Document Parsing
- [ ] Auto-detect format from content and extension
- [ ] Parse valid TOML, YAML, JSON, Markdown without error
- [ ] Handle invalid input gracefully with clear error messages

### Managed Blocks
- [ ] Extract blocks with correct UUIDs and spans
- [ ] Insert blocks at End, Before, After locations
- [ ] Update block content preserving UUID
- [ ] Remove blocks cleanly
- [ ] Detect and report checksum drift

### Format Preservation
- [ ] TOML: preserve comments, whitespace, key order
- [ ] YAML: preserve comments, whitespace, flow style
- [ ] JSON: preserve whitespace where possible
- [ ] Markdown: preserve document structure

### Semantic Comparison
- [ ] Equivalent documents return `is_equivalent: true`
- [ ] Different key order does not affect equivalence
- [ ] Diff reports correct Added/Removed/Modified changes
- [ ] Similarity ratio is accurate

### Rollback
- [ ] Every edit operation returns reversible `Edit`
- [ ] Applying `edit.inverse()` restores original state
- [ ] Transaction rollback restores all files

## Research References

- [tree-sitter](https://crates.io/crates/tree-sitter) - Incremental parsing
- [toml_edit](https://crates.io/crates/toml_edit) - Format-preserving TOML
- [yaml-edit](https://crates.io/crates/yaml-edit) - Format-preserving YAML
- [similar](https://github.com/mitsuhiko/similar) - Diff algorithms
- [ast-grep](https://ast-grep.github.io/) - Structural search patterns
- [Rust 2024 Edition](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/) - Language features

## Integration Points

### With repo-fs
- Uses `NormalizedPath` for all file operations
- Uses `WorkspaceLayout` for path resolution
- Uses atomic write functions for safe saves

### With repo-tools (future)
- Enables managed block sync across tool configs
- Provides drift detection for config validation
- Supports the ledger-based state system

---

*Design approved for implementation: 2026-01-23*
