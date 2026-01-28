# repo-content Phase B Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete repo-content Phases 3-4 by adding YAML handler, Markdown handler, path operations, and semantic diff computation.

**Architecture:** Extend the existing `FormatHandler` trait pattern. YAML uses `serde_yaml` with regex-based block markers (like TOML). Markdown uses `tree-sitter-md` for AST parsing. Path operations work on normalized JSON representations. Semantic diff uses the `similar` crate for text comparison.

**Tech Stack:**
- `serde_yaml = "0.9"` (already in workspace) - YAML parsing
- `tree-sitter = "0.24"` + `tree-sitter-md = "0.3"` (already in workspace) - Markdown parsing
- `similar = "2.7"` (already in workspace) - Diff algorithms
- Hash comment markers for YAML (`# repo:block:UUID`)
- HTML comment markers for Markdown (`<!-- repo:block:UUID -->`)

**Note:** The original design specified `yaml-edit` for format-preserving YAML editing. However, `serde_yaml` is already in the workspace and provides sufficient functionality for our use case. Block markers use regex patterns (consistent with TOML handler approach).

---

## Task 1: Add YamlHandler

**Files:**
- Create: `crates/repo-content/src/handlers/yaml.rs`
- Modify: `crates/repo-content/src/handlers/mod.rs`
- Modify: `crates/repo-content/src/document.rs`
- Create: `crates/repo-content/tests/yaml_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-content/tests/yaml_tests.rs`:

```rust
//! Tests for YAML handler

use repo_content::format::FormatHandler;
use repo_content::handlers::YamlHandler;
use uuid::Uuid;

#[test]
fn test_yaml_find_blocks() {
    let handler = YamlHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"name: test
version: "1.0"

# repo:block:550e8400-e29b-41d4-a716-446655440000
managed:
  key: value
# /repo:block:550e8400-e29b-41d4-a716-446655440000

other: data
"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);
    assert!(blocks[0].content.contains("managed:"));
}

#[test]
fn test_yaml_normalize_key_order() {
    let handler = YamlHandler::new();

    let source1 = "b: 2\na: 1\n";
    let source2 = "a: 1\nb: 2\n";

    let norm1 = handler.normalize(source1).unwrap();
    let norm2 = handler.normalize(source2).unwrap();

    assert_eq!(norm1, norm2);
}

#[test]
fn test_yaml_parse_error() {
    let handler = YamlHandler::new();
    let result = handler.parse("invalid: yaml: content: [unclosed");
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-content yaml_tests`
Expected: FAIL - `YamlHandler` not found

**Step 3: Write the YamlHandler implementation**

Create `crates/repo-content/src/handlers/yaml.rs`:

```rust
//! YAML format handler using serde_yaml

use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::{Edit, EditKind};
use crate::error::{Error, Result};
use crate::format::{CommentStyle, Format, FormatHandler};

/// Pattern to match block start markers and capture the UUID
static BLOCK_START_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#\s*repo:block:([0-9a-f-]{36})").unwrap());

/// Handler for YAML files
#[derive(Debug, Default)]
pub struct YamlHandler;

impl YamlHandler {
    pub fn new() -> Self {
        Self
    }

    fn find_block_end(source: &str, uuid: &Uuid, start_pos: usize) -> Option<usize> {
        let end_marker = format!("# /repo:block:{uuid}");
        source[start_pos..].find(&end_marker).map(|pos| {
            let abs_pos = start_pos + pos + end_marker.len();
            if source[abs_pos..].starts_with('\n') {
                abs_pos + 1
            } else {
                abs_pos
            }
        })
    }
}

impl FormatHandler for YamlHandler {
    fn format(&self) -> Format {
        Format::Yaml
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        let value: serde_yaml::Value = serde_yaml::from_str(source)
            .map_err(|e| Error::parse("YAML", e.to_string()))?;
        Ok(Box::new(value))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
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

            let Some(block_end) = Self::find_block_end(source, &uuid, content_start) else {
                continue;
            };

            let end_marker = format!("# /repo:block:{uuid}");
            let content_end = source[content_start..]
                .find(&end_marker)
                .map(|p| content_start + p)
                .unwrap_or(block_end);

            let content = &source[content_start..content_end];
            let content = content.strip_prefix('\n').unwrap_or(content);

            blocks.push(ManagedBlock::new(uuid, content, block_start..block_end));
        }

        blocks
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: BlockLocation,
    ) -> Result<(String, Edit)> {
        let style = CommentStyle::Hash;
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
                .and_then(|p| source[p..].find('\n').map(|np| p + np + 1))
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

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        let blocks = self.find_blocks(source);
        let block = blocks
            .iter()
            .find(|b| b.uuid == uuid)
            .ok_or(Error::BlockNotFound { uuid })?;

        let style = CommentStyle::Hash;
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

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        let blocks = self.find_blocks(source);
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

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(source)
            .map_err(|e| Error::parse("YAML", e.to_string()))?;

        fn yaml_to_json_sorted(yaml: &serde_yaml::Value) -> serde_json::Value {
            match yaml {
                serde_yaml::Value::Null => serde_json::Value::Null,
                serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
                serde_yaml::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        serde_json::Value::Number(i.into())
                    } else if let Some(f) = n.as_f64() {
                        serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or(serde_json::Value::Null)
                    } else {
                        serde_json::Value::Null
                    }
                }
                serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
                serde_yaml::Value::Sequence(seq) => {
                    serde_json::Value::Array(seq.iter().map(yaml_to_json_sorted).collect())
                }
                serde_yaml::Value::Mapping(map) => {
                    let mut json_map = serde_json::Map::new();
                    let mut keys: Vec<_> = map.keys().collect();
                    keys.sort_by(|a, b| {
                        let a_str = a.as_str().unwrap_or("");
                        let b_str = b.as_str().unwrap_or("");
                        a_str.cmp(b_str)
                    });
                    for key in keys {
                        if let Some(val) = map.get(key) {
                            let key_str = key.as_str().unwrap_or("").to_string();
                            json_map.insert(key_str, yaml_to_json_sorted(val));
                        }
                    }
                    serde_json::Value::Object(json_map)
                }
                serde_yaml::Value::Tagged(tagged) => yaml_to_json_sorted(&tagged.value),
            }
        }

        Ok(yaml_to_json_sorted(&yaml_value))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<serde_yaml::Value>()
            .map(|v| serde_yaml::to_string(v).unwrap_or_default())
            .ok_or_else(|| Error::parse("YAML", "invalid internal state"))
    }
}
```

**Step 4: Update handlers/mod.rs**

Modify `crates/repo-content/src/handlers/mod.rs`:

```rust
//! Format handlers

mod json;
mod plaintext;
mod toml;
mod yaml;

pub use self::json::JsonHandler;
pub use self::toml::TomlHandler;
pub use self::yaml::YamlHandler;
pub use plaintext::PlainTextHandler;
```

**Step 5: Update document.rs to use YamlHandler**

In `crates/repo-content/src/document.rs`, change the YAML case:

```rust
Format::Yaml => Box::new(YamlHandler::new()),
```

And add the import:

```rust
use crate::handlers::{JsonHandler, PlainTextHandler, TomlHandler, YamlHandler};
```

**Step 6: Add serde_yaml to repo-content Cargo.toml**

Add to `crates/repo-content/Cargo.toml` dependencies:

```toml
serde_yaml = { workspace = true }
```

**Step 7: Run tests to verify they pass**

Run: `cargo test -p repo-content yaml`
Expected: PASS

**Step 8: Commit**

```bash
git add crates/repo-content/
git commit -m "feat(repo-content): add YamlHandler with serde_yaml

- Hash comment block markers (# repo:block:UUID)
- Key-sorted normalization for semantic comparison
- Full FormatHandler trait implementation

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add MarkdownHandler with tree-sitter

**Files:**
- Create: `crates/repo-content/src/handlers/markdown.rs`
- Modify: `crates/repo-content/src/handlers/mod.rs`
- Modify: `crates/repo-content/src/document.rs`
- Create: `crates/repo-content/tests/markdown_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-content/tests/markdown_tests.rs`:

```rust
//! Tests for Markdown handler

use repo_content::format::FormatHandler;
use repo_content::handlers::MarkdownHandler;
use repo_content::block::BlockLocation;
use uuid::Uuid;

#[test]
fn test_markdown_find_blocks() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"# My Document

Some intro text.

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Managed content here
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

More content.
"#;

    let blocks = handler.find_blocks(source);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, uuid);
    assert!(blocks[0].content.contains("Managed content"));
}

#[test]
fn test_markdown_insert_block() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::new_v4();

    let source = "# Title\n\nContent here.\n";
    let (result, _edit) = handler
        .insert_block(source, uuid, "New managed section", BlockLocation::End)
        .unwrap();

    assert!(result.contains("repo:block:"));
    assert!(result.contains("New managed section"));
    assert!(result.contains("/repo:block:"));
}

#[test]
fn test_markdown_normalize() {
    let handler = MarkdownHandler::new();

    // Multiple blank lines should collapse
    let source1 = "# Title\n\n\n\nContent";
    let source2 = "# Title\n\nContent";

    let norm1 = handler.normalize(source1).unwrap();
    let norm2 = handler.normalize(source2).unwrap();

    assert_eq!(norm1, norm2);
}

#[test]
fn test_markdown_remove_block() {
    let handler = MarkdownHandler::new();
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

    let source = r#"Before

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Content
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

After"#;

    let (result, _edit) = handler.remove_block(source, uuid).unwrap();

    assert!(!result.contains("repo:block:"));
    assert!(result.contains("Before"));
    assert!(result.contains("After"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-content markdown_tests`
Expected: FAIL - `MarkdownHandler` not found

**Step 3: Write the MarkdownHandler implementation**

Create `crates/repo-content/src/handlers/markdown.rs`:

```rust
//! Markdown format handler using tree-sitter-md

use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use crate::block::{BlockLocation, ManagedBlock};
use crate::edit::{Edit, EditKind};
use crate::error::{Error, Result};
use crate::format::{CommentStyle, Format, FormatHandler};

/// Pattern to match HTML comment block markers
static BLOCK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?s)<!--\s*repo:block:([0-9a-f-]{36})\s*-->\n?(.*?)<!--\s*/repo:block:\1\s*-->\n?",
    )
    .unwrap()
});

/// Pattern to collapse multiple blank lines
static MULTI_BLANK_LINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());

/// Handler for Markdown files
#[derive(Debug, Default)]
pub struct MarkdownHandler {
    _parser: Option<tree_sitter::Parser>,
}

impl MarkdownHandler {
    pub fn new() -> Self {
        // Initialize tree-sitter parser for Markdown
        let mut parser = tree_sitter::Parser::new();
        let _ = parser.set_language(&tree_sitter_md::language());
        Self {
            _parser: Some(parser),
        }
    }
}

impl FormatHandler for MarkdownHandler {
    fn format(&self) -> Format {
        Format::Markdown
    }

    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any + Send + Sync>> {
        // For Markdown, we store the source directly
        // Tree-sitter parsing happens on-demand for queries
        Ok(Box::new(source.to_string()))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        BLOCK_PATTERN
            .captures_iter(source)
            .filter_map(|cap| {
                let uuid = Uuid::parse_str(cap.get(1)?.as_str()).ok()?;
                let content = cap.get(2)?.as_str().to_string();
                let full_match = cap.get(0)?;
                let span = full_match.start()..full_match.end();
                Some(ManagedBlock::new(uuid, content, span))
            })
            .collect()
    }

    fn insert_block(
        &self,
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
                .and_then(|p| source[p..].find('\n').map(|np| p + np + 1))
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

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, Edit)> {
        let blocks = self.find_blocks(source);
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

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, Edit)> {
        let blocks = self.find_blocks(source);
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

    fn normalize(&self, source: &str) -> Result<serde_json::Value> {
        // Normalize Markdown:
        // 1. Trim trailing whitespace per line
        // 2. Collapse multiple blank lines to single blank line
        // 3. Normalize line endings to LF
        let normalized: String = source
            .lines()
            .map(|l| l.trim_end())
            .collect::<Vec<_>>()
            .join("\n");

        let normalized = MULTI_BLANK_LINES.replace_all(&normalized, "\n\n");
        let normalized = normalized.trim().to_string();

        Ok(serde_json::Value::String(normalized))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        parsed
            .downcast_ref::<String>()
            .cloned()
            .ok_or_else(|| Error::parse("Markdown", "invalid internal state"))
    }
}
```

**Step 4: Update handlers/mod.rs**

Modify `crates/repo-content/src/handlers/mod.rs`:

```rust
//! Format handlers

mod json;
mod markdown;
mod plaintext;
mod toml;
mod yaml;

pub use self::json::JsonHandler;
pub use self::markdown::MarkdownHandler;
pub use self::toml::TomlHandler;
pub use self::yaml::YamlHandler;
pub use plaintext::PlainTextHandler;
```

**Step 5: Update document.rs to use MarkdownHandler**

In `crates/repo-content/src/document.rs`, update imports and the Markdown case:

```rust
use crate::handlers::{JsonHandler, MarkdownHandler, PlainTextHandler, TomlHandler, YamlHandler};
```

Change:
```rust
Format::PlainText | Format::Markdown => Box::new(PlainTextHandler::new()),
```

To:
```rust
Format::PlainText => Box::new(PlainTextHandler::new()),
Format::Markdown => Box::new(MarkdownHandler::new()),
```

**Step 6: Run tests to verify they pass**

Run: `cargo test -p repo-content markdown`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/repo-content/
git commit -m "feat(repo-content): add MarkdownHandler with tree-sitter-md

- HTML comment block markers (<!-- repo:block:UUID -->)
- Collapses multiple blank lines for normalization
- Trims trailing whitespace per line

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Implement Path Operations

**Files:**
- Create: `crates/repo-content/src/path.rs`
- Modify: `crates/repo-content/src/lib.rs`
- Modify: `crates/repo-content/src/document.rs`
- Create: `crates/repo-content/tests/path_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-content/tests/path_tests.rs`:

```rust
//! Tests for path operations

use repo_content::Document;
use serde_json::json;

#[test]
fn test_get_path_simple() {
    let source = r#"{"name": "test", "version": "1.0"}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("name"), Some(json!("test")));
    assert_eq!(doc.get_path("version"), Some(json!("1.0")));
    assert_eq!(doc.get_path("missing"), None);
}

#[test]
fn test_get_path_nested() {
    let source = r#"{"config": {"database": {"host": "localhost", "port": 5432}}}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("config.database.host"), Some(json!("localhost")));
    assert_eq!(doc.get_path("config.database.port"), Some(json!(5432)));
    assert_eq!(doc.get_path("config.database.missing"), None);
}

#[test]
fn test_get_path_array() {
    let source = r#"{"items": [{"name": "first"}, {"name": "second"}]}"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("items[0].name"), Some(json!("first")));
    assert_eq!(doc.get_path("items[1].name"), Some(json!("second")));
    assert_eq!(doc.get_path("items[2].name"), None);
}

#[test]
fn test_get_path_toml() {
    let source = r#"[package]
name = "test"
version = "1.0"

[dependencies]
serde = "1.0"
"#;
    let doc = Document::parse(source).unwrap();

    assert_eq!(doc.get_path("package.name"), Some(json!("test")));
    assert_eq!(doc.get_path("dependencies.serde"), Some(json!("1.0")));
}

#[test]
fn test_set_path() {
    let source = r#"{"name": "old"}"#;
    let mut doc = Document::parse(source).unwrap();

    let edit = doc.set_path("name", "new").unwrap();
    assert!(edit.old_content.contains("old"));

    assert_eq!(doc.get_path("name"), Some(json!("new")));
}

#[test]
fn test_remove_path() {
    let source = r#"{"name": "test", "version": "1.0"}"#;
    let mut doc = Document::parse(source).unwrap();

    let edit = doc.remove_path("version").unwrap();
    assert!(edit.old_content.contains("version"));

    assert_eq!(doc.get_path("version"), None);
    assert_eq!(doc.get_path("name"), Some(json!("test")));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-content path_tests`
Expected: FAIL - `get_path` method not found

**Step 3: Create the path module**

Create `crates/repo-content/src/path.rs`:

```rust
//! Path parsing and traversal utilities

use serde_json::Value;

/// Parse a dot-separated path with optional array indexing
/// Examples: "config.database.host", "items[0].name"
pub fn parse_path(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current = String::new();

    let chars: Vec<char> = path.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '.' {
            if !current.is_empty() {
                segments.push(PathSegment::Key(current.clone()));
                current.clear();
            }
        } else if c == '[' {
            if !current.is_empty() {
                segments.push(PathSegment::Key(current.clone()));
                current.clear();
            }
            // Parse array index
            i += 1;
            let mut idx_str = String::new();
            while i < chars.len() && chars[i] != ']' {
                idx_str.push(chars[i]);
                i += 1;
            }
            if let Ok(idx) = idx_str.parse::<usize>() {
                segments.push(PathSegment::Index(idx));
            }
        } else {
            current.push(c);
        }
        i += 1;
    }

    if !current.is_empty() {
        segments.push(PathSegment::Key(current));
    }

    segments
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

/// Get a value at a path from a JSON value
pub fn get_at_path(value: &Value, segments: &[PathSegment]) -> Option<Value> {
    let mut current = value;

    for segment in segments {
        match segment {
            PathSegment::Key(key) => {
                current = current.get(key)?;
            }
            PathSegment::Index(idx) => {
                current = current.get(idx)?;
            }
        }
    }

    Some(current.clone())
}

/// Set a value at a path in a JSON value, returning the modified value
pub fn set_at_path(value: &mut Value, segments: &[PathSegment], new_value: Value) -> bool {
    if segments.is_empty() {
        return false;
    }

    let mut current = value;

    for (i, segment) in segments.iter().enumerate() {
        let is_last = i == segments.len() - 1;

        match segment {
            PathSegment::Key(key) => {
                if is_last {
                    if let Some(obj) = current.as_object_mut() {
                        obj.insert(key.clone(), new_value);
                        return true;
                    }
                    return false;
                }
                current = current.get_mut(key)?;
            }
            PathSegment::Index(idx) => {
                if is_last {
                    if let Some(arr) = current.as_array_mut() {
                        if *idx < arr.len() {
                            arr[*idx] = new_value;
                            return true;
                        }
                    }
                    return false;
                }
                current = current.get_mut(idx)?;
            }
        }
    }

    None?
}

/// Remove a value at a path from a JSON value
pub fn remove_at_path(value: &mut Value, segments: &[PathSegment]) -> Option<Value> {
    if segments.is_empty() {
        return None;
    }

    if segments.len() == 1 {
        return match &segments[0] {
            PathSegment::Key(key) => value.as_object_mut()?.remove(key),
            PathSegment::Index(idx) => {
                let arr = value.as_array_mut()?;
                if *idx < arr.len() {
                    Some(arr.remove(*idx))
                } else {
                    None
                }
            }
        };
    }

    // Navigate to parent
    let parent_segments = &segments[..segments.len() - 1];
    let last_segment = &segments[segments.len() - 1];

    let mut current = value;
    for segment in parent_segments {
        match segment {
            PathSegment::Key(key) => {
                current = current.get_mut(key)?;
            }
            PathSegment::Index(idx) => {
                current = current.get_mut(idx)?;
            }
        }
    }

    match last_segment {
        PathSegment::Key(key) => current.as_object_mut()?.remove(key),
        PathSegment::Index(idx) => {
            let arr = current.as_array_mut()?;
            if *idx < arr.len() {
                Some(arr.remove(*idx))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_simple_path() {
        let segments = parse_path("name");
        assert_eq!(segments, vec![PathSegment::Key("name".to_string())]);
    }

    #[test]
    fn test_parse_nested_path() {
        let segments = parse_path("config.database.host");
        assert_eq!(
            segments,
            vec![
                PathSegment::Key("config".to_string()),
                PathSegment::Key("database".to_string()),
                PathSegment::Key("host".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_array_path() {
        let segments = parse_path("items[0].name");
        assert_eq!(
            segments,
            vec![
                PathSegment::Key("items".to_string()),
                PathSegment::Index(0),
                PathSegment::Key("name".to_string()),
            ]
        );
    }

    #[test]
    fn test_get_at_path() {
        let value = json!({"config": {"host": "localhost"}});
        let segments = parse_path("config.host");
        assert_eq!(get_at_path(&value, &segments), Some(json!("localhost")));
    }
}
```

**Step 4: Update lib.rs**

Add to `crates/repo-content/src/lib.rs`:

```rust
pub mod path;
```

**Step 5: Add path methods to Document**

Add these methods to `Document` in `crates/repo-content/src/document.rs`:

```rust
use crate::path::{get_at_path, parse_path, remove_at_path, set_at_path};

impl Document {
    // ... existing methods ...

    /// Get value at a dot-separated path (e.g., "config.database.host")
    pub fn get_path(&self, path: &str) -> Option<serde_json::Value> {
        let normalized = self.handler.normalize(&self.source).ok()?;
        let segments = parse_path(path);
        get_at_path(&normalized, &segments)
    }

    /// Set value at a path
    pub fn set_path(&mut self, path: &str, value: impl Into<serde_json::Value>) -> Result<Edit> {
        let mut normalized = self.handler.normalize(&self.source)?;
        let segments = parse_path(path);
        let old_value = get_at_path(&normalized, &segments);

        if !set_at_path(&mut normalized, &segments, value.into()) {
            return Err(crate::error::Error::PathNotFound {
                path: path.to_string(),
            });
        }

        // For JSON, re-render the document
        let old_source = self.source.clone();
        let new_source = serde_json::to_string_pretty(&normalized)?;

        let edit = Edit {
            kind: crate::edit::EditKind::PathSet {
                path: path.to_string(),
            },
            span: 0..old_source.len(),
            old_content: old_source,
            new_content: new_source.clone(),
        };

        self.source = new_source;
        Ok(edit)
    }

    /// Remove value at a path
    pub fn remove_path(&mut self, path: &str) -> Result<Edit> {
        let mut normalized = self.handler.normalize(&self.source)?;
        let segments = parse_path(path);

        let removed = remove_at_path(&mut normalized, &segments)
            .ok_or_else(|| crate::error::Error::PathNotFound {
                path: path.to_string(),
            })?;

        let old_source = self.source.clone();
        let new_source = serde_json::to_string_pretty(&normalized)?;

        let edit = Edit {
            kind: crate::edit::EditKind::PathRemove {
                path: path.to_string(),
            },
            span: 0..old_source.len(),
            old_content: old_source,
            new_content: new_source.clone(),
        };

        self.source = new_source;
        Ok(edit)
    }
}
```

**Step 6: Run tests to verify they pass**

Run: `cargo test -p repo-content path`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/repo-content/
git commit -m "feat(repo-content): add path operations (get/set/remove)

- Dot-separated path syntax: config.database.host
- Array indexing: items[0].name
- Works with normalized JSON representation
- Returns Edit for rollback support

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Complete Semantic Diff with similar crate

**Files:**
- Modify: `crates/repo-content/src/diff.rs`
- Modify: `crates/repo-content/src/document.rs`
- Create: `crates/repo-content/tests/diff_integration_tests.rs`

**Step 1: Write the failing test**

Create `crates/repo-content/tests/diff_integration_tests.rs`:

```rust
//! Integration tests for semantic diff

use repo_content::{Document, SemanticChange};
use serde_json::json;

#[test]
fn test_diff_added_key() {
    let doc1 = Document::parse(r#"{"name": "test"}"#).unwrap();
    let doc2 = Document::parse(r#"{"name": "test", "version": "1.0"}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Added { path, .. } if path == "version"
    )));
}

#[test]
fn test_diff_removed_key() {
    let doc1 = Document::parse(r#"{"name": "test", "version": "1.0"}"#).unwrap();
    let doc2 = Document::parse(r#"{"name": "test"}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Removed { path, .. } if path == "version"
    )));
}

#[test]
fn test_diff_modified_value() {
    let doc1 = Document::parse(r#"{"version": "1.0"}"#).unwrap();
    let doc2 = Document::parse(r#"{"version": "2.0"}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Modified { path, old, new }
        if path == "version" && old == &json!("1.0") && new == &json!("2.0")
    )));
}

#[test]
fn test_diff_equivalent() {
    let doc1 = Document::parse(r#"{"a": 1, "b": 2}"#).unwrap();
    let doc2 = Document::parse(r#"{"b": 2, "a": 1}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(diff.is_equivalent);
    assert!(diff.changes.is_empty());
    assert_eq!(diff.similarity, 1.0);
}

#[test]
fn test_diff_similarity_ratio() {
    let doc1 = Document::parse(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#).unwrap();
    let doc2 = Document::parse(r#"{"a": 1, "b": 2, "c": 3, "e": 5}"#).unwrap();

    let diff = doc1.diff(&doc2);

    // 3 out of 4 keys match, plus one added and one removed
    assert!(!diff.is_equivalent);
    assert!(diff.similarity > 0.5);
    assert!(diff.similarity < 1.0);
}

#[test]
fn test_diff_nested_changes() {
    let doc1 = Document::parse(r#"{"config": {"host": "localhost"}}"#).unwrap();
    let doc2 = Document::parse(r#"{"config": {"host": "example.com"}}"#).unwrap();

    let diff = doc1.diff(&doc2);

    assert!(!diff.is_equivalent);
    assert!(diff.changes.iter().any(|c| matches!(c,
        SemanticChange::Modified { path, .. } if path == "config.host"
    )));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-content diff_integration`
Expected: FAIL - changes are empty in current implementation

**Step 3: Update diff.rs with full implementation**

Replace `crates/repo-content/src/diff.rs`:

```rust
//! Semantic diff types and computation

use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use uuid::Uuid;

/// Result of comparing two documents semantically
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticDiff {
    /// Are the documents semantically equivalent?
    pub is_equivalent: bool,
    /// List of semantic changes
    pub changes: Vec<SemanticChange>,
    /// Similarity ratio (0.0 to 1.0)
    pub similarity: f64,
}

impl SemanticDiff {
    /// Create a diff indicating documents are equivalent
    pub fn equivalent() -> Self {
        Self {
            is_equivalent: true,
            changes: Vec::new(),
            similarity: 1.0,
        }
    }

    /// Create a diff with changes
    pub fn with_changes(changes: Vec<SemanticChange>, similarity: f64) -> Self {
        Self {
            is_equivalent: changes.is_empty(),
            changes,
            similarity,
        }
    }

    /// Compute diff between two normalized JSON values
    pub fn compute(old: &Value, new: &Value) -> Self {
        let mut changes = Vec::new();
        diff_values(old, new, String::new(), &mut changes);

        let similarity = compute_similarity(old, new);

        Self {
            is_equivalent: changes.is_empty(),
            changes,
            similarity,
        }
    }

    /// Compute diff for text content (Markdown, PlainText)
    pub fn compute_text(old: &str, new: &str) -> Self {
        if old == new {
            return Self::equivalent();
        }

        let text_diff = TextDiff::from_lines(old, new);
        let mut changes = Vec::new();

        for change in text_diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => {
                    changes.push(SemanticChange::BlockAdded {
                        uuid: None,
                        content: change.value().to_string(),
                    });
                }
                ChangeTag::Delete => {
                    changes.push(SemanticChange::BlockRemoved {
                        uuid: None,
                        content: change.value().to_string(),
                    });
                }
                ChangeTag::Equal => {}
            }
        }

        let similarity = text_diff.ratio();

        Self {
            is_equivalent: changes.is_empty(),
            changes,
            similarity,
        }
    }
}

impl Default for SemanticDiff {
    fn default() -> Self {
        Self::equivalent()
    }
}

/// A semantic change between documents
#[derive(Debug, Clone, PartialEq)]
pub enum SemanticChange {
    /// Key/path added
    Added { path: String, value: Value },
    /// Key/path removed
    Removed { path: String, value: Value },
    /// Value changed at path
    Modified { path: String, old: Value, new: Value },
    /// Block added (for Markdown/text)
    BlockAdded { uuid: Option<Uuid>, content: String },
    /// Block removed
    BlockRemoved { uuid: Option<Uuid>, content: String },
    /// Block content changed
    BlockModified {
        uuid: Option<Uuid>,
        old: String,
        new: String,
    },
}

/// Recursively diff two JSON values
fn diff_values(old: &Value, new: &Value, path: String, changes: &mut Vec<SemanticChange>) {
    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            // Check for removed keys
            for key in old_map.keys() {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                if !new_map.contains_key(key) {
                    changes.push(SemanticChange::Removed {
                        path: key_path,
                        value: old_map[key].clone(),
                    });
                }
            }

            // Check for added and modified keys
            for key in new_map.keys() {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };

                match old_map.get(key) {
                    None => {
                        changes.push(SemanticChange::Added {
                            path: key_path,
                            value: new_map[key].clone(),
                        });
                    }
                    Some(old_val) => {
                        diff_values(old_val, &new_map[key], key_path, changes);
                    }
                }
            }
        }
        (Value::Array(old_arr), Value::Array(new_arr)) => {
            let max_len = old_arr.len().max(new_arr.len());
            for i in 0..max_len {
                let idx_path = if path.is_empty() {
                    format!("[{i}]")
                } else {
                    format!("{path}[{i}]")
                };

                match (old_arr.get(i), new_arr.get(i)) {
                    (Some(old_val), Some(new_val)) => {
                        diff_values(old_val, new_val, idx_path, changes);
                    }
                    (Some(old_val), None) => {
                        changes.push(SemanticChange::Removed {
                            path: idx_path,
                            value: old_val.clone(),
                        });
                    }
                    (None, Some(new_val)) => {
                        changes.push(SemanticChange::Added {
                            path: idx_path,
                            value: new_val.clone(),
                        });
                    }
                    (None, None) => {}
                }
            }
        }
        _ => {
            if old != new {
                changes.push(SemanticChange::Modified {
                    path,
                    old: old.clone(),
                    new: new.clone(),
                });
            }
        }
    }
}

/// Compute similarity ratio between two JSON values
fn compute_similarity(old: &Value, new: &Value) -> f64 {
    let old_str = serde_json::to_string(old).unwrap_or_default();
    let new_str = serde_json::to_string(new).unwrap_or_default();

    let diff = TextDiff::from_chars(&old_str, &new_str);
    diff.ratio()
}
```

**Step 4: Update Document::diff() method**

In `crates/repo-content/src/document.rs`, replace the `diff` method:

```rust
/// Compute semantic diff between two documents
pub fn diff(&self, other: &Document) -> SemanticDiff {
    // For text formats, use text diff
    if matches!(self.format, Format::PlainText | Format::Markdown) {
        return SemanticDiff::compute_text(&self.source, &other.source);
    }

    // For structured formats, use JSON diff
    let Ok(norm1) = self.handler.normalize(&self.source) else {
        return SemanticDiff::default();
    };
    let Ok(norm2) = other.handler.normalize(&other.source) else {
        return SemanticDiff::default();
    };

    SemanticDiff::compute(&norm1, &norm2)
}
```

And add the import at the top:

```rust
use crate::diff::SemanticDiff;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test -p repo-content diff`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/repo-content/
git commit -m "feat(repo-content): complete semantic diff with similar crate

- Recursive JSON diff with path tracking
- Text diff using similar::TextDiff
- Similarity ratio computation
- Added/Removed/Modified change detection

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Final Integration and Verification

**Files:**
- Modify: `crates/repo-content/src/lib.rs` (re-exports)
- Run comprehensive tests

**Step 1: Update lib.rs re-exports**

Ensure `crates/repo-content/src/lib.rs` exports everything:

```rust
//! # repo-content
//!
//! Content parsing, editing, and diffing for Repository Manager.

pub mod block;
pub mod diff;
pub mod document;
pub mod edit;
pub mod error;
pub mod format;
pub mod handlers;
pub mod path;

pub use block::{BlockLocation, ManagedBlock};
pub use diff::{SemanticChange, SemanticDiff};
pub use document::Document;
pub use edit::{Edit, EditKind};
pub use error::{Error, Result};
pub use format::{CommentStyle, Format, FormatHandler};
pub use handlers::{JsonHandler, MarkdownHandler, PlainTextHandler, TomlHandler, YamlHandler};
```

**Step 2: Run all tests**

Run: `cargo test -p repo-content`
Expected: All tests pass

**Step 3: Run clippy**

Run: `cargo clippy -p repo-content -- -D warnings`
Expected: No warnings

**Step 4: Run example**

Run: `cargo run -p repo-content --example basic_usage`
Expected: Successful output

**Step 5: Final commit**

```bash
git add crates/repo-content/
git commit -m "chore(repo-content): complete Phase B implementation

Phase 3-4 complete:
- YamlHandler with serde_yaml
- MarkdownHandler with tree-sitter-md
- Path operations (get/set/remove)
- Full semantic diff with similar crate

All handlers now support:
- Block find/insert/update/remove
- Semantic normalization
- Format-specific markers

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| tree-sitter-md version mismatch | Use workspace version (0.3), API should be stable |
| YAML edge cases (anchors, tags) | serde_yaml handles most cases; block markers use regex |
| Path operations on non-JSON formats | Operations work on normalized JSON representation |
| similar crate API changes | Version 2.7 is stable and widely used |

## Rollback Plan

Each task has atomic commits. To rollback:

```bash
git revert HEAD~N  # where N is number of commits to revert
```

---

## Verification Checklist

After completing all tasks:

- [ ] `cargo test -p repo-content` - All tests pass
- [ ] `cargo clippy -p repo-content -- -D warnings` - No warnings
- [ ] `cargo doc -p repo-content --no-deps` - Documentation builds
- [ ] YAML files can have managed blocks inserted/updated/removed
- [ ] Markdown files can have managed blocks inserted/updated/removed
- [ ] Path operations work on JSON/TOML/YAML documents
- [ ] Semantic diff reports Added/Removed/Modified changes
- [ ] Similarity ratio is computed correctly

---

**Sources:**
- [yaml-edit crate](https://crates.io/crates/yaml-edit)
- [serde-yaml](https://github.com/dtolnay/serde-yaml)
