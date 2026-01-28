# Repository Manager Full Implementation Roadmap

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete the Repository Manager from current foundation to production-ready CLI and MCP server.

**Architecture:** Layered crate architecture with repo-fs (foundation), repo-git, repo-content, repo-meta, repo-presets, repo-tools, repo-blocks, repo-core (orchestration), repo-cli, and repo-mcp.

**Tech Stack:** Rust 2024, clap v4, tokio, mcp-sdk, git2, similar, serde

---

## Current State Summary

| Crate | Status | Notes |
|-------|--------|-------|
| repo-fs | ✅ Complete | NormalizedPath, atomic I/O, robustness |
| repo-git | ✅ Complete | ContainerLayout, worktree operations |
| repo-content | ✅ Complete | Document, Format handlers, managed blocks |
| repo-meta | 🔶 Partial | Config/schema done, ledger not implemented |
| repo-presets | 🔶 Partial | UvProvider only, needs more providers |
| repo-tools | 🔶 Partial | VSCode/Cursor/Claude, needs more tools |
| repo-blocks | ✅ Complete | Block parsing and writing |
| repo-core | ❌ Missing | Orchestration layer not started |
| repo-cli | ❌ Missing | CLI not started |
| repo-mcp | ❌ Missing | MCP server not started |

---

## Phase 2: Content System Enhancement

**Goal:** Complete remaining repo-content features: YAML handler, detailed diffing, and modification tracking.

### Task 2.1: Implement YAML Handler

**Files:**
- Create: `crates/repo-content/src/handlers/yaml.rs`
- Modify: `crates/repo-content/src/handlers/mod.rs`
- Modify: `crates/repo-content/src/document.rs:31-34`

**Step 1: Write the failing test**

Create test file:

```rust
// In crates/repo-content/src/handlers/yaml.rs (at bottom)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_parse_simple() {
        let handler = YamlHandler::new();
        let source = "key: value\nlist:\n  - item1\n  - item2";
        let result = handler.parse(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_normalize() {
        let handler = YamlHandler::new();
        let source1 = "key: value\nother: data";
        let source2 = "other: data\nkey: value";
        let norm1 = handler.normalize(source1).unwrap();
        let norm2 = handler.normalize(source2).unwrap();
        assert_eq!(norm1, norm2);
    }

    #[test]
    fn test_yaml_find_blocks() {
        let handler = YamlHandler::new();
        let source = r#"
# repo:block:550e8400-e29b-41d4-a716-446655440000
managed_key: managed_value
# /repo:block:550e8400-e29b-41d4-a716-446655440000
user_key: user_value
"#;
        let blocks = handler.find_blocks(source);
        assert_eq!(blocks.len(), 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-content yaml -- --nocapture`
Expected: FAIL with "cannot find module yaml"

**Step 3: Implement YamlHandler**

```rust
// crates/repo-content/src/handlers/yaml.rs
//! YAML format handler

use crate::block::ManagedBlock;
use crate::error::{Error, Result};
use crate::format::FormatHandler;
use serde_json::Value;
use serde_yaml;
use uuid::Uuid;

pub struct YamlHandler;

impl YamlHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for YamlHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatHandler for YamlHandler {
    fn parse(&self, source: &str) -> Result<Box<dyn std::any::Any>> {
        let value: serde_yaml::Value = serde_yaml::from_str(source)
            .map_err(|e| Error::Parse(format!("YAML parse error: {}", e)))?;
        Ok(Box::new(value))
    }

    fn render(&self, parsed: &dyn std::any::Any) -> Result<String> {
        let value = parsed
            .downcast_ref::<serde_yaml::Value>()
            .ok_or_else(|| Error::Parse("Invalid YAML value".to_string()))?;
        serde_yaml::to_string(value)
            .map_err(|e| Error::Parse(format!("YAML render error: {}", e)))
    }

    fn normalize(&self, source: &str) -> Result<Value> {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(source)
            .map_err(|e| Error::Parse(format!("YAML parse error: {}", e)))?;
        // Convert YAML to JSON for normalization
        let json_str = serde_json::to_string(&yaml_value)
            .map_err(|e| Error::Parse(format!("JSON conversion error: {}", e)))?;
        serde_json::from_str(&json_str)
            .map_err(|e| Error::Parse(format!("JSON parse error: {}", e)))
    }

    fn find_blocks(&self, source: &str) -> Vec<ManagedBlock> {
        crate::block::find_blocks_with_markers(source, "#", "")
    }

    fn insert_block(
        &self,
        source: &str,
        uuid: Uuid,
        content: &str,
        location: crate::block::BlockLocation,
    ) -> Result<(String, crate::edit::Edit)> {
        crate::block::insert_block_with_markers(source, uuid, content, location, "#", "")
    }

    fn update_block(&self, source: &str, uuid: Uuid, content: &str) -> Result<(String, crate::edit::Edit)> {
        crate::block::update_block_with_markers(source, uuid, content, "#", "")
    }

    fn remove_block(&self, source: &str, uuid: Uuid) -> Result<(String, crate::edit::Edit)> {
        crate::block::remove_block_with_markers(source, uuid, "#", "")
    }
}
```

**Step 4: Update handlers/mod.rs**

Add to `crates/repo-content/src/handlers/mod.rs`:

```rust
mod yaml;
pub use yaml::YamlHandler;
```

**Step 5: Update document.rs to use YamlHandler**

Replace lines 31-34 in `crates/repo-content/src/document.rs`:

```rust
            Format::Yaml => Box::new(YamlHandler::new()),
```

**Step 6: Add serde_yaml dependency**

Run: `cargo add serde_yaml -p repo-content`

**Step 7: Run tests to verify**

Run: `cargo test -p repo-content yaml`
Expected: All YAML tests pass

**Step 8: Run full test suite**

Run: `cargo test -p repo-content`
Expected: All tests pass (now 90+)

**Step 9: Commit**

```bash
git add crates/repo-content/
git commit -m "feat(repo-content): add YAML format handler

Implements YamlHandler with full FormatHandler trait support:
- Parse/render YAML documents
- Normalize for semantic comparison
- Managed block operations with # comment markers

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 2.2: Implement Detailed Semantic Diff

**Files:**
- Modify: `crates/repo-content/src/diff.rs`
- Modify: `crates/repo-content/src/document.rs:115-125`

**Step 1: Write the failing test**

Add to `crates/repo-content/src/diff.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_detailed_changes() {
        let diff = SemanticDiff {
            is_equivalent: false,
            changes: vec![
                Change::Added { path: "key".to_string(), value: "new".to_string() },
                Change::Removed { path: "old".to_string(), value: "gone".to_string() },
            ],
            similarity: 0.75,
        };
        assert_eq!(diff.changes.len(), 2);
        assert!(!diff.is_equivalent);
    }

    #[test]
    fn test_compute_diff_json() {
        let old = r#"{"a": 1, "b": 2}"#;
        let new = r#"{"a": 1, "c": 3}"#;
        let diff = compute_json_diff(old, new).unwrap();
        assert!(!diff.is_equivalent);
        assert!(diff.changes.iter().any(|c| matches!(c, Change::Removed { path, .. } if path == "b")));
        assert!(diff.changes.iter().any(|c| matches!(c, Change::Added { path, .. } if path == "c")));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-content diff -- --nocapture`
Expected: FAIL with "cannot find Change"

**Step 3: Implement detailed diff**

Update `crates/repo-content/src/diff.rs`:

```rust
//! Semantic diff types and computation

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single change in a semantic diff
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Change {
    /// A value was added
    Added { path: String, value: String },
    /// A value was removed
    Removed { path: String, value: String },
    /// A value was modified
    Modified { path: String, old: String, new: String },
}

/// Result of comparing two documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub is_equivalent: bool,
    pub changes: Vec<Change>,
    pub similarity: f64,
}

impl SemanticDiff {
    /// Create a diff indicating equivalent documents
    pub fn equivalent() -> Self {
        Self {
            is_equivalent: true,
            changes: Vec::new(),
            similarity: 1.0,
        }
    }
}

/// Compute detailed diff between two JSON values
pub fn compute_json_diff(old: &str, new: &str) -> Result<SemanticDiff, String> {
    let old_val: Value = serde_json::from_str(old)
        .map_err(|e| format!("Failed to parse old JSON: {}", e))?;
    let new_val: Value = serde_json::from_str(new)
        .map_err(|e| format!("Failed to parse new JSON: {}", e))?;

    if old_val == new_val {
        return Ok(SemanticDiff::equivalent());
    }

    let mut changes = Vec::new();
    diff_values("", &old_val, &new_val, &mut changes);

    let similarity = compute_similarity(&old_val, &new_val);

    Ok(SemanticDiff {
        is_equivalent: false,
        changes,
        similarity,
    })
}

fn diff_values(path: &str, old: &Value, new: &Value, changes: &mut Vec<Change>) {
    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            // Check for removed/modified keys
            for (key, old_val) in old_map {
                let key_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                match new_map.get(key) {
                    Some(new_val) if old_val != new_val => {
                        if old_val.is_object() && new_val.is_object() {
                            diff_values(&key_path, old_val, new_val, changes);
                        } else {
                            changes.push(Change::Modified {
                                path: key_path,
                                old: old_val.to_string(),
                                new: new_val.to_string(),
                            });
                        }
                    }
                    None => {
                        changes.push(Change::Removed {
                            path: key_path,
                            value: old_val.to_string(),
                        });
                    }
                    _ => {}
                }
            }
            // Check for added keys
            for (key, new_val) in new_map {
                if !old_map.contains_key(key) {
                    let key_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    changes.push(Change::Added {
                        path: key_path,
                        value: new_val.to_string(),
                    });
                }
            }
        }
        (Value::Array(old_arr), Value::Array(new_arr)) => {
            // Simple array diff - report size changes
            if old_arr.len() != new_arr.len() {
                changes.push(Change::Modified {
                    path: path.to_string(),
                    old: format!("[{} items]", old_arr.len()),
                    new: format!("[{} items]", new_arr.len()),
                });
            }
        }
        _ => {
            if old != new {
                changes.push(Change::Modified {
                    path: path.to_string(),
                    old: old.to_string(),
                    new: new.to_string(),
                });
            }
        }
    }
}

fn compute_similarity(old: &Value, new: &Value) -> f64 {
    if old == new {
        return 1.0;
    }

    match (old, new) {
        (Value::Object(old_map), Value::Object(new_map)) => {
            let all_keys: std::collections::HashSet<_> = old_map
                .keys()
                .chain(new_map.keys())
                .collect();
            let common_keys = old_map
                .keys()
                .filter(|k| new_map.contains_key(*k))
                .count();

            if all_keys.is_empty() {
                1.0
            } else {
                common_keys as f64 / all_keys.len() as f64
            }
        }
        _ => 0.0,
    }
}
```

**Step 4: Update document.rs diff method**

Replace `diff` method in `crates/repo-content/src/document.rs`:

```rust
    /// Compute semantic diff between two documents.
    ///
    /// Returns detailed changes between documents including:
    /// - Added, removed, and modified values (for structured formats)
    /// - Similarity score (1.0 = identical, 0.0 = completely different)
    pub fn diff(&self, other: &Document) -> SemanticDiff {
        if self.semantic_eq(other) {
            return SemanticDiff::equivalent();
        }

        // Try to compute detailed diff using normalized JSON
        let Ok(norm1) = self.normalize() else {
            return SemanticDiff {
                is_equivalent: false,
                changes: Vec::new(),
                similarity: 0.5,
            };
        };
        let Ok(norm2) = other.normalize() else {
            return SemanticDiff {
                is_equivalent: false,
                changes: Vec::new(),
                similarity: 0.5,
            };
        };

        crate::diff::compute_json_diff(
            &norm1.to_string(),
            &norm2.to_string(),
        ).unwrap_or_else(|_| SemanticDiff {
            is_equivalent: false,
            changes: Vec::new(),
            similarity: 0.5,
        })
    }
```

**Step 5: Export Change from lib.rs**

Update `crates/repo-content/src/lib.rs` exports:

```rust
pub use diff::{Change, SemanticDiff};
```

**Step 6: Run tests**

Run: `cargo test -p repo-content diff`
Expected: All diff tests pass

**Step 7: Commit**

```bash
git add crates/repo-content/src/diff.rs crates/repo-content/src/document.rs crates/repo-content/src/lib.rs
git commit -m "feat(repo-content): implement detailed semantic diff

Adds Change enum (Added/Removed/Modified) and compute_json_diff()
for detailed change tracking between documents.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 2.3: Implement Document Modification Tracking

**Files:**
- Modify: `crates/repo-content/src/document.rs`

**Step 1: Write the failing test**

Add to document tests:

```rust
#[test]
fn test_is_modified_tracks_changes() {
    let mut doc = Document::parse(r#"{"key": "value"}"#).unwrap();
    assert!(!doc.is_modified());

    let uuid = Uuid::new_v4();
    doc.insert_block(uuid, "new content", BlockLocation::End).unwrap();
    assert!(doc.is_modified());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-content is_modified`
Expected: FAIL (is_modified always returns false)

**Step 3: Update Document struct to track modifications**

Update `crates/repo-content/src/document.rs`:

```rust
/// Unified document type wrapping format-specific backends
pub struct Document {
    source: String,
    original_source: String,
    format: Format,
    handler: Box<dyn FormatHandler>,
}

impl Document {
    /// Parse content with format auto-detection
    pub fn parse(source: &str) -> Result<Self> {
        let format = Format::from_content(source);
        Self::parse_as(source, format)
    }

    /// Parse with explicit format
    pub fn parse_as(source: &str, format: Format) -> Result<Self> {
        let handler: Box<dyn FormatHandler> = match format {
            Format::Toml => Box::new(TomlHandler::new()),
            Format::Json => Box::new(JsonHandler::new()),
            Format::Yaml => Box::new(YamlHandler::new()),
            Format::PlainText | Format::Markdown => Box::new(PlainTextHandler::new()),
        };

        // Verify it parses
        let _ = handler.parse(source)?;

        Ok(Self {
            source: source.to_string(),
            original_source: source.to_string(),
            format,
            handler,
        })
    }

    // ... other methods unchanged ...

    /// Check if document has been modified from its original source.
    pub fn is_modified(&self) -> bool {
        self.source != self.original_source
    }

    /// Reset modification tracking (e.g., after save)
    pub fn mark_saved(&mut self) {
        self.original_source = self.source.clone();
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p repo-content is_modified`
Expected: Test passes

**Step 5: Run full test suite**

Run: `cargo test -p repo-content`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/repo-content/src/document.rs
git commit -m "feat(repo-content): implement modification tracking

Document now tracks original_source and reports is_modified() accurately.
Added mark_saved() to reset tracking after persistence.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 3: Core Orchestration Layer

**Goal:** Create repo-core crate that orchestrates all other crates, implements the ledger system, and provides the unified API.

### Task 3.1: Create repo-core Crate Structure

**Files:**
- Create: `crates/repo-core/Cargo.toml`
- Create: `crates/repo-core/src/lib.rs`
- Create: `crates/repo-core/src/error.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "repo-core"
version = "0.1.0"
edition = "2024"
description = "Core orchestration layer for Repository Manager"

[dependencies]
repo-fs = { path = "../repo-fs" }
repo-git = { path = "../repo-git" }
repo-content = { path = "../repo-content" }
repo-meta = { path = "../repo-meta" }
repo-presets = { path = "../repo-presets" }
repo-tools = { path = "../repo-tools" }
repo-blocks = { path = "../repo-blocks" }

thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3"
assert_fs = "1"
```

**Step 2: Create error.rs**

```rust
// crates/repo-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Repository not initialized: {0}")]
    NotInitialized(String),

    #[error("Repository already initialized at {0}")]
    AlreadyInitialized(String),

    #[error("Ledger error: {0}")]
    Ledger(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Git error: {0}")]
    Git(#[from] repo_git::Error),

    #[error("Content error: {0}")]
    Content(#[from] repo_content::Error),

    #[error("Metadata error: {0}")]
    Meta(#[from] repo_meta::Error),

    #[error("Preset error: {0}")]
    Preset(#[from] repo_presets::Error),

    #[error("Tool error: {0}")]
    Tool(#[from] repo_tools::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Step 3: Create lib.rs**

```rust
// crates/repo-core/src/lib.rs
//! Core orchestration layer for Repository Manager.
//!
//! This crate provides the unified API that coordinates all other crates
//! to implement repository management functionality.

pub mod error;
pub mod ledger;
pub mod manager;

pub use error::{Error, Result};
pub use ledger::{Intent, Ledger, Projection, ProjectionKind};
pub use manager::RepoManager;
```

**Step 4: Add to workspace Cargo.toml**

Add `"crates/repo-core"` to workspace members.

**Step 5: Verify compilation**

Run: `cargo build -p repo-core`
Expected: Compilation succeeds (with warnings about unused modules)

**Step 6: Commit**

```bash
git add crates/repo-core/ Cargo.toml
git commit -m "feat(repo-core): create core orchestration crate structure

Adds repo-core with error types and module structure.
Orchestrates repo-fs, repo-git, repo-content, repo-meta,
repo-presets, repo-tools, and repo-blocks.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 3.2: Implement Ledger System

**Files:**
- Create: `crates/repo-core/src/ledger.rs`
- Create: `crates/repo-core/tests/ledger_tests.rs`

**Step 1: Write the failing test**

```rust
// crates/repo-core/tests/ledger_tests.rs
use repo_core::{Intent, Ledger, Projection, ProjectionKind};
use uuid::Uuid;

#[test]
fn test_ledger_add_intent() {
    let mut ledger = Ledger::new();
    let uuid = Uuid::new_v4();

    let intent = Intent {
        id: "rule:python/snake-case".to_string(),
        uuid,
        timestamp: chrono::Utc::now(),
        args: serde_json::json!({"severity": "error"}),
        projections: vec![],
    };

    ledger.add_intent(intent.clone());
    assert_eq!(ledger.intents().len(), 1);
    assert_eq!(ledger.get_intent(uuid).unwrap().id, intent.id);
}

#[test]
fn test_ledger_remove_intent() {
    let mut ledger = Ledger::new();
    let uuid = Uuid::new_v4();

    let intent = Intent {
        id: "rule:test".to_string(),
        uuid,
        timestamp: chrono::Utc::now(),
        args: serde_json::Value::Null,
        projections: vec![],
    };

    ledger.add_intent(intent);
    assert!(ledger.remove_intent(uuid).is_some());
    assert!(ledger.get_intent(uuid).is_none());
}

#[test]
fn test_ledger_serialization() {
    let mut ledger = Ledger::new();
    let uuid = Uuid::new_v4();

    ledger.add_intent(Intent {
        id: "rule:serialize-test".to_string(),
        uuid,
        timestamp: chrono::Utc::now(),
        args: serde_json::json!({"key": "value"}),
        projections: vec![
            Projection {
                tool: "cursor".to_string(),
                file: ".cursorrules".into(),
                kind: ProjectionKind::TextBlock {
                    marker: uuid,
                    checksum: "abc123".to_string(),
                },
            },
        ],
    });

    let toml_str = ledger.to_toml().unwrap();
    let loaded = Ledger::from_toml(&toml_str).unwrap();

    assert_eq!(loaded.intents().len(), 1);
    assert_eq!(loaded.get_intent(uuid).unwrap().projections.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core ledger`
Expected: FAIL with "cannot find module ledger"

**Step 3: Implement ledger.rs**

```rust
// crates/repo-core/src/ledger.rs
//! Ledger-based state tracking for repository configuration.
//!
//! The ledger maps abstract intents (rules, presets) to concrete
//! projections (file edits, JSON keys, managed files).

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// The ledger tracks all active modifications performed by the manager.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ledger {
    #[serde(default)]
    pub meta: LedgerMeta,
    #[serde(default)]
    intents: Vec<Intent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerMeta {
    pub version: String,
    pub updated_at: DateTime<Utc>,
}

impl Default for LedgerMeta {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            updated_at: Utc::now(),
        }
    }
}

/// An intent represents a high-level rule or configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// Canonical ID (e.g., "rule:python/snake-case")
    pub id: String,
    /// Unique instance ID
    pub uuid: Uuid,
    /// When this intent was applied
    pub timestamp: DateTime<Utc>,
    /// Configuration arguments
    pub args: serde_json::Value,
    /// Concrete realizations of this intent
    pub projections: Vec<Projection>,
}

/// A projection is a concrete realization of an intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    /// Tool this projection belongs to
    pub tool: String,
    /// File path affected
    pub file: PathBuf,
    /// How the projection is stored
    pub kind: ProjectionKind,
}

/// How a projection is stored in the target file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectionKind {
    /// Text block with UUID markers
    TextBlock { marker: Uuid, checksum: String },
    /// JSON/TOML key ownership
    JsonKey { path: String, value: serde_json::Value },
    /// Entire file is managed
    FileManaged { checksum: String },
}

impl Ledger {
    /// Create a new empty ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load ledger from TOML string.
    pub fn from_toml(source: &str) -> Result<Self> {
        toml::from_str(source).map_err(|e| Error::Ledger(format!("Parse error: {}", e)))
    }

    /// Serialize ledger to TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| Error::Ledger(format!("Serialize error: {}", e)))
    }

    /// Get all intents.
    pub fn intents(&self) -> &[Intent] {
        &self.intents
    }

    /// Add an intent.
    pub fn add_intent(&mut self, intent: Intent) {
        self.intents.push(intent);
        self.meta.updated_at = Utc::now();
    }

    /// Get intent by UUID.
    pub fn get_intent(&self, uuid: Uuid) -> Option<&Intent> {
        self.intents.iter().find(|i| i.uuid == uuid)
    }

    /// Get intent by ID (may return multiple if same rule applied multiple times).
    pub fn get_intents_by_id(&self, id: &str) -> Vec<&Intent> {
        self.intents.iter().filter(|i| i.id == id).collect()
    }

    /// Remove intent by UUID, returning the removed intent.
    pub fn remove_intent(&mut self, uuid: Uuid) -> Option<Intent> {
        if let Some(pos) = self.intents.iter().position(|i| i.uuid == uuid) {
            self.meta.updated_at = Utc::now();
            Some(self.intents.remove(pos))
        } else {
            None
        }
    }

    /// Find all projections for a given file.
    pub fn projections_for_file(&self, file: &PathBuf) -> Vec<(&Intent, &Projection)> {
        self.intents
            .iter()
            .flat_map(|intent| {
                intent
                    .projections
                    .iter()
                    .filter(|p| &p.file == file)
                    .map(move |p| (intent, p))
            })
            .collect()
    }

    /// Find all projections for a given tool.
    pub fn projections_for_tool(&self, tool: &str) -> Vec<(&Intent, &Projection)> {
        self.intents
            .iter()
            .flat_map(|intent| {
                intent
                    .projections
                    .iter()
                    .filter(|p| p.tool == tool)
                    .map(move |p| (intent, p))
            })
            .collect()
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p repo-core ledger`
Expected: All ledger tests pass

**Step 5: Commit**

```bash
git add crates/repo-core/src/ledger.rs crates/repo-core/tests/
git commit -m "feat(repo-core): implement ledger system

Adds Ledger with Intent and Projection types for tracking
configuration state. Supports TOML serialization and queries
by UUID, ID, file, and tool.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 3.3: Implement RepoManager

**Files:**
- Create: `crates/repo-core/src/manager.rs`
- Create: `crates/repo-core/tests/manager_tests.rs`

**Step 1: Write the failing test**

```rust
// crates/repo-core/tests/manager_tests.rs
use repo_core::RepoManager;
use tempfile::tempdir;

#[test]
fn test_manager_init() {
    let dir = tempdir().unwrap();
    let manager = RepoManager::init(dir.path(), Default::default()).unwrap();

    assert!(dir.path().join(".repository").exists());
    assert!(dir.path().join(".repository/config.toml").exists());
    assert!(dir.path().join(".repository/ledger.toml").exists());
}

#[test]
fn test_manager_open() {
    let dir = tempdir().unwrap();

    // Init first
    let _ = RepoManager::init(dir.path(), Default::default()).unwrap();

    // Then open
    let manager = RepoManager::open(dir.path()).unwrap();
    assert!(manager.is_initialized());
}

#[test]
fn test_manager_check() {
    let dir = tempdir().unwrap();
    let manager = RepoManager::init(dir.path(), Default::default()).unwrap();

    let report = manager.check().unwrap();
    assert!(report.is_healthy());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p repo-core manager`
Expected: FAIL with "cannot find RepoManager"

**Step 3: Implement manager.rs**

```rust
// crates/repo-core/src/manager.rs
//! Repository manager - the main orchestration API.

use crate::error::{Error, Result};
use crate::ledger::Ledger;
use repo_fs::NormalizedPath;
use repo_meta::{RepositoryConfig, RepositoryMode};
use std::path::{Path, PathBuf};

const REPO_DIR: &str = ".repository";
const CONFIG_FILE: &str = "config.toml";
const LEDGER_FILE: &str = "ledger.toml";

/// Options for initializing a repository.
#[derive(Debug, Clone, Default)]
pub struct InitOptions {
    pub mode: Option<RepositoryMode>,
    pub tools: Vec<String>,
    pub presets: Vec<String>,
}

/// Health check report.
#[derive(Debug, Clone)]
pub struct CheckReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl CheckReport {
    pub fn is_healthy(&self) -> bool {
        self.errors.is_empty()
    }
}

/// The main repository manager.
pub struct RepoManager {
    root: PathBuf,
    config: RepositoryConfig,
    ledger: Ledger,
}

impl RepoManager {
    /// Initialize a new repository.
    pub fn init(path: impl AsRef<Path>, options: InitOptions) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        let repo_dir = root.join(REPO_DIR);

        if repo_dir.exists() {
            return Err(Error::AlreadyInitialized(root.display().to_string()));
        }

        // Create .repository directory structure
        std::fs::create_dir_all(&repo_dir)?;
        std::fs::create_dir_all(repo_dir.join("tools"))?;
        std::fs::create_dir_all(repo_dir.join("rules"))?;
        std::fs::create_dir_all(repo_dir.join("presets"))?;

        // Create default config
        let config = RepositoryConfig {
            mode: options.mode.unwrap_or(RepositoryMode::Worktrees),
            tools: options.tools,
            presets: options.presets,
            ..Default::default()
        };

        let config_str = toml::to_string_pretty(&config)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(repo_dir.join(CONFIG_FILE), config_str)?;

        // Create empty ledger
        let ledger = Ledger::new();
        let ledger_str = ledger.to_toml()?;
        std::fs::write(repo_dir.join(LEDGER_FILE), ledger_str)?;

        Ok(Self {
            root,
            config,
            ledger,
        })
    }

    /// Open an existing repository.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let root = path.as_ref().to_path_buf();
        let repo_dir = root.join(REPO_DIR);

        if !repo_dir.exists() {
            return Err(Error::NotInitialized(root.display().to_string()));
        }

        // Load config
        let config_path = repo_dir.join(CONFIG_FILE);
        let config_str = std::fs::read_to_string(&config_path)?;
        let config: RepositoryConfig = toml::from_str(&config_str)
            .map_err(|e| Error::Config(format!("Failed to parse config: {}", e)))?;

        // Load ledger
        let ledger_path = repo_dir.join(LEDGER_FILE);
        let ledger = if ledger_path.exists() {
            let ledger_str = std::fs::read_to_string(&ledger_path)?;
            Ledger::from_toml(&ledger_str)?
        } else {
            Ledger::new()
        };

        Ok(Self {
            root,
            config,
            ledger,
        })
    }

    /// Check if repository is initialized.
    pub fn is_initialized(&self) -> bool {
        self.root.join(REPO_DIR).exists()
    }

    /// Get repository root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get current configuration.
    pub fn config(&self) -> &RepositoryConfig {
        &self.config
    }

    /// Get current ledger.
    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    /// Perform health check.
    pub fn check(&self) -> Result<CheckReport> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check .repository exists
        let repo_dir = self.root.join(REPO_DIR);
        if !repo_dir.exists() {
            errors.push("Missing .repository directory".to_string());
        }

        // Check config exists
        if !repo_dir.join(CONFIG_FILE).exists() {
            errors.push("Missing config.toml".to_string());
        }

        // Check ledger exists
        if !repo_dir.join(LEDGER_FILE).exists() {
            warnings.push("Missing ledger.toml (will be created on first use)".to_string());
        }

        // Validate ledger projections (check files exist)
        for intent in self.ledger.intents() {
            for projection in &intent.projections {
                let file_path = self.root.join(&projection.file);
                if !file_path.exists() {
                    warnings.push(format!(
                        "Projection file missing: {} (intent: {})",
                        projection.file.display(),
                        intent.id
                    ));
                }
            }
        }

        Ok(CheckReport { errors, warnings })
    }

    /// Save current state to disk.
    pub fn save(&self) -> Result<()> {
        let repo_dir = self.root.join(REPO_DIR);

        // Save config
        let config_str = toml::to_string_pretty(&self.config)
            .map_err(|e| Error::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(repo_dir.join(CONFIG_FILE), config_str)?;

        // Save ledger
        let ledger_str = self.ledger.to_toml()?;
        std::fs::write(repo_dir.join(LEDGER_FILE), ledger_str)?;

        Ok(())
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p repo-core manager`
Expected: All manager tests pass

**Step 5: Run full test suite**

Run: `cargo test -p repo-core`
Expected: All tests pass

**Step 6: Commit**

```bash
git add crates/repo-core/src/manager.rs crates/repo-core/tests/
git commit -m "feat(repo-core): implement RepoManager

Adds RepoManager with init(), open(), check(), and save() methods.
Manages .repository directory with config.toml and ledger.toml.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Phase 4: CLI Implementation

**Goal:** Create repo-cli crate with full command-line interface using clap v4.

### Task 4.1: Create CLI Crate Structure

**Files:**
- Create: `crates/repo-cli/Cargo.toml`
- Create: `crates/repo-cli/src/main.rs`
- Create: `crates/repo-cli/src/commands/mod.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Create Cargo.toml**

```toml
[package]
name = "repo-cli"
version = "0.1.0"
edition = "2024"
description = "CLI for Repository Manager"

[[bin]]
name = "repo"
path = "src/main.rs"

[dependencies]
repo-core = { path = "../repo-core" }
repo-fs = { path = "../repo-fs" }
repo-git = { path = "../repo-git" }

clap = { version = "4", features = ["derive", "env"] }
anyhow = "1"
colored = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

**Step 2: Create main.rs**

```rust
// crates/repo-cli/src/main.rs
use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "repo")]
#[command(about = "Repository Manager CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    Init(commands::init::InitArgs),
    /// Check repository health
    Check,
    /// Repair repository issues
    Fix,
    /// Sync tool configurations
    Sync,
    /// Branch management
    Branch {
        #[command(subcommand)]
        action: commands::branch::BranchAction,
    },
    /// Add a tool
    AddTool { name: String },
    /// Remove a tool
    RemoveTool { name: String },
    /// Add a preset
    AddPreset { name: String },
    /// Remove a preset
    RemovePreset { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    match cli.command {
        Commands::Init(args) => commands::init::run(args),
        Commands::Check => commands::check::run(),
        Commands::Fix => commands::fix::run(),
        Commands::Sync => commands::sync::run(),
        Commands::Branch { action } => commands::branch::run(action),
        Commands::AddTool { name } => commands::tool::add(&name),
        Commands::RemoveTool { name } => commands::tool::remove(&name),
        Commands::AddPreset { name } => commands::preset::add(&name),
        Commands::RemovePreset { name } => commands::preset::remove(&name),
    }
}
```

**Step 3: Create commands/mod.rs**

```rust
// crates/repo-cli/src/commands/mod.rs
pub mod branch;
pub mod check;
pub mod fix;
pub mod init;
pub mod preset;
pub mod sync;
pub mod tool;
```

**Step 4: Create basic command stubs**

Create each command file with basic implementation.

**Step 5: Add to workspace**

Add `"crates/repo-cli"` to workspace members.

**Step 6: Build and verify**

Run: `cargo build -p repo-cli`
Expected: Binary builds successfully

**Step 7: Test help output**

Run: `cargo run -p repo-cli -- --help`
Expected: Shows help with all commands

**Step 8: Commit**

```bash
git add crates/repo-cli/ Cargo.toml
git commit -m "feat(repo-cli): create CLI crate with clap v4

Adds repo binary with subcommands: init, check, fix, sync,
branch, add-tool, remove-tool, add-preset, remove-preset.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 4.2: Implement Init Command

**Files:**
- Create: `crates/repo-cli/src/commands/init.rs`
- Create: `crates/repo-cli/tests/init_tests.rs`

**Step 1: Write integration test**

```rust
// crates/repo-cli/tests/init_tests.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_init_creates_repository() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("repo")
        .unwrap()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository initialized"));

    assert!(dir.path().join(".repository").exists());
    assert!(dir.path().join(".repository/config.toml").exists());
}

#[test]
fn test_init_with_tools() {
    let dir = tempdir().unwrap();

    Command::cargo_bin("repo")
        .unwrap()
        .current_dir(dir.path())
        .args(["init", "--tools", "cursor", "vscode"])
        .assert()
        .success();

    let config = std::fs::read_to_string(dir.path().join(".repository/config.toml")).unwrap();
    assert!(config.contains("cursor"));
    assert!(config.contains("vscode"));
}
```

**Step 2: Implement init.rs**

```rust
// crates/repo-cli/src/commands/init.rs
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use repo_core::{InitOptions, RepoManager};
use repo_meta::RepositoryMode;
use std::env;

#[derive(Args)]
pub struct InitArgs {
    /// Tools to enable
    #[arg(long, num_args = 1..)]
    tools: Vec<String>,

    /// Presets to apply
    #[arg(long, num_args = 1..)]
    presets: Vec<String>,

    /// Repository mode
    #[arg(long, default_value = "worktrees")]
    mode: String,
}

pub fn run(args: InitArgs) -> Result<()> {
    let cwd = env::current_dir()?;

    let mode = match args.mode.as_str() {
        "worktrees" => RepositoryMode::Worktrees,
        "standard" => RepositoryMode::Standard,
        _ => {
            eprintln!("{}: Invalid mode '{}'. Use 'worktrees' or 'standard'.",
                "Error".red(), args.mode);
            std::process::exit(1);
        }
    };

    let options = InitOptions {
        mode: Some(mode),
        tools: args.tools,
        presets: args.presets,
    };

    match RepoManager::init(&cwd, options) {
        Ok(_) => {
            println!("{} Repository initialized at {}", "✓".green(), cwd.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p repo-cli init`
Expected: All init tests pass

**Step 4: Commit**

```bash
git add crates/repo-cli/src/commands/init.rs crates/repo-cli/tests/
git commit -m "feat(repo-cli): implement init command

Supports --tools, --presets, and --mode flags.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

### Task 4.3 - 4.8: Implement Remaining Commands

(Similar structure for check, fix, sync, branch, tool, preset commands)

Each command follows the same pattern:
1. Write integration tests
2. Implement command handler
3. Run tests
4. Commit

---

## Phase 5: Additional Preset Providers

**Goal:** Add Conda, Node, and Rust preset providers.

### Task 5.1: Implement CondaProvider

**Files:**
- Create: `crates/repo-presets/src/conda.rs`
- Modify: `crates/repo-presets/src/lib.rs`

### Task 5.2: Implement NodeProvider

**Files:**
- Create: `crates/repo-presets/src/node.rs`
- Modify: `crates/repo-presets/src/lib.rs`

### Task 5.3: Implement RustProvider

**Files:**
- Create: `crates/repo-presets/src/rust_preset.rs`
- Modify: `crates/repo-presets/src/lib.rs`

Each provider follows the PresetProvider trait pattern established by UvProvider.

---

## Phase 6: Extended Tool Integrations

**Goal:** Add Windsurf and JetBrains IDE integrations.

### Task 6.1: Implement WindsurfIntegration

**Files:**
- Create: `crates/repo-tools/src/windsurf.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Task 6.2: Implement JetBrainsIntegration

**Files:**
- Create: `crates/repo-tools/src/jetbrains.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

---

## Phase 7: MCP Server Implementation

**Goal:** Create repo-mcp crate exposing Repository Manager via Model Context Protocol.

### Task 7.1: Create MCP Crate Structure

**Files:**
- Create: `crates/repo-mcp/Cargo.toml`
- Create: `crates/repo-mcp/src/lib.rs`
- Create: `crates/repo-mcp/src/server.rs`
- Create: `crates/repo-mcp/src/tools.rs`
- Create: `crates/repo-mcp/src/resources.rs`

**Dependencies:**
- mcp-sdk (or rmcp)
- tokio
- repo-core

### Task 7.2: Implement Repository Lifecycle Tools

Tools: `repo_init`, `repo_check`, `repo_fix`, `repo_sync`

### Task 7.3: Implement Branch Management Tools

Tools: `branch_create`, `branch_delete`, `branch_list`, `branch_checkout`

### Task 7.4: Implement Git Primitive Tools

Tools: `git_push`, `git_pull`, `git_merge`

### Task 7.5: Implement Configuration Tools

Tools: `tool_add`, `tool_remove`, `preset_add`, `preset_remove`, `rule_add`, `rule_modify`, `rule_remove`

### Task 7.6: Implement Resources

Resources: `repo://config`, `repo://state`, `repo://rules`

---

## Phase 8: Migration Tools & Rollback

**Goal:** Implement safe migration between modes and rollback capabilities.

### Task 8.1: Implement Mode Migration

**Files:**
- Create: `crates/repo-core/src/migration.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs` (add migrate command)

### Task 8.2: Implement Rollback System

**Files:**
- Create: `crates/repo-core/src/rollback.rs`
- Modify: `crates/repo-core/src/manager.rs`

---

## Phase 9: Performance & Security Audits

**Goal:** Optimize critical paths and audit for security vulnerabilities.

### Task 9.1: Benchmark Critical Paths

**Files:**
- Create: `crates/repo-core/benches/core_benchmarks.rs`
- Update: `crates/repo-fs/benches/fs_benchmarks.rs`

### Task 9.2: Security Audit

- Path traversal prevention (repo-fs already has NormalizedPath)
- Symlink attack prevention
- Race condition prevention (TOCTOU)
- Input validation audit

---

## Phase 10: Documentation & Examples

**Goal:** Complete documentation and add example configurations.

### Task 10.1: API Documentation

Run: `cargo doc --workspace --no-deps`
Fix any missing documentation warnings.

### Task 10.2: User Guide

**Files:**
- Create: `docs/user-guide.md`
- Create: `docs/configuration.md`
- Create: `docs/cli-reference.md`

### Task 10.3: Example Configurations

**Files:**
- Create: `examples/python-project/`
- Create: `examples/rust-project/`
- Create: `examples/web-project/`

---

## Summary

| Phase | Tasks | Estimated Complexity |
|-------|-------|---------------------|
| 2 | 3 tasks | Low |
| 3 | 3 tasks | Medium |
| 4 | 8 tasks | Medium |
| 5 | 3 tasks | Low |
| 6 | 2 tasks | Low |
| 7 | 6 tasks | High |
| 8 | 2 tasks | Medium |
| 9 | 2 tasks | Medium |
| 10 | 3 tasks | Low |

**Total:** 32 tasks across 9 phases

**Dependencies:**
- Phase 3 depends on Phase 2
- Phase 4 depends on Phase 3
- Phase 7 depends on Phase 3
- Phases 5, 6 can run in parallel with Phase 4
- Phase 8 depends on Phases 3 and 4
- Phases 9 and 10 can run after Phase 4
