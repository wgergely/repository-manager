# Phase 1: Vertical Slice Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a complete vertical slice through all layers using `env:python` preset, proving the architecture works end-to-end.

**Architecture:** Four new crates (`repo-meta`, `repo-presets`, `repo-tools`, `repo-blocks`) building on Layer 0 (`repo-fs`, `repo-git`). The slice implements: config loading → preset check/apply → tool sync with managed blocks.

**Tech Stack:** Rust 2024, serde, toml, uuid, async-trait, tokio (minimal)

**Design References:**
- `docs/design/config-schema.md` - Configuration structure
- `docs/design/spec-presets.md` - PresetProvider trait
- `docs/design/spec-tools.md` - ToolIntegration trait
- `docs/design/config-ledger.md` - State tracking
- `docs/design/providers-reference.md` - Python provider design

---

## Overview: Crate Dependencies

```
repo-cli (future)
    ↓
repo-core (future)
    ↓
┌─────────────┬─────────────┐
│ repo-presets│ repo-tools  │  ← Phase 1
├─────────────┴─────────────┤
│        repo-meta          │  ← Phase 1
├─────────────┬─────────────┤
│  repo-blocks│             │  ← Phase 1 (managed block system)
├─────────────┼─────────────┤
│   repo-fs   │  repo-git   │  ← Layer 0 (complete)
└─────────────┴─────────────┘
```

---

## Phase 1.0: Project Setup

### Task 0.1: Add New Crates to Workspace

**Files:**
- Modify: `Cargo.toml` (workspace root)
- Create: `crates/repo-blocks/Cargo.toml`
- Create: `crates/repo-blocks/src/lib.rs`
- Create: `crates/repo-meta/Cargo.toml`
- Create: `crates/repo-meta/src/lib.rs`
- Create: `crates/repo-presets/Cargo.toml`
- Create: `crates/repo-presets/src/lib.rs`
- Create: `crates/repo-tools/Cargo.toml`
- Create: `crates/repo-tools/src/lib.rs`

**Step 1: Update workspace Cargo.toml with new dependencies**

Add to `Cargo.toml` workspace dependencies:
```toml
# Add under [workspace.dependencies]
uuid = { version = "1.11", features = ["v4", "serde"] }
async-trait = "0.1"
tokio = { version = "1.42", features = ["rt", "process", "fs"] }
regex = "1.11"
```

**Step 2: Create repo-blocks crate**

Create `crates/repo-blocks/Cargo.toml`:
```toml
[package]
name = "repo-blocks"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Managed block system for Repository Manager"

[dependencies]
repo-fs = { path = "../repo-fs" }
uuid = { workspace = true }
regex = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
rstest = { workspace = true }
pretty_assertions = { workspace = true }
```

Create `crates/repo-blocks/src/lib.rs`:
```rust
//! Managed block system for Repository Manager
//!
//! Handles UUID-tagged content blocks in text files for safe sync/unroll.

pub mod error;
pub mod parser;
pub mod writer;

pub use error::{Error, Result};
```

**Step 3: Create repo-meta crate**

Create `crates/repo-meta/Cargo.toml`:
```toml
[package]
name = "repo-meta"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Configuration and metadata management for Repository Manager"

[dependencies]
repo-fs = { path = "../repo-fs" }
serde = { workspace = true }
toml = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
rstest = { workspace = true }
pretty_assertions = { workspace = true }
```

Create `crates/repo-meta/src/lib.rs`:
```rust
//! Configuration and metadata management for Repository Manager
//!
//! Reads .repository/ config and provides resolved configuration to other crates.

pub mod error;
pub mod config;
pub mod registry;

pub use error::{Error, Result};
```

**Step 4: Create repo-presets crate**

Create `crates/repo-presets/Cargo.toml`:
```toml
[package]
name = "repo-presets"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Preset providers for Repository Manager"

[dependencies]
repo-fs = { path = "../repo-fs" }
repo-meta = { path = "../repo-meta" }
async-trait = { workspace = true }
tokio = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
rstest = { workspace = true }
pretty_assertions = { workspace = true }
tokio = { workspace = true, features = ["rt-multi-thread", "macros"] }
```

Create `crates/repo-presets/src/lib.rs`:
```rust
//! Preset providers for Repository Manager
//!
//! Implements PresetProvider trait for env:python, config:*, tool:*.

pub mod error;
pub mod provider;
pub mod context;
pub mod python;

pub use error::{Error, Result};
pub use provider::{PresetProvider, CheckReport, ApplyReport, PresetStatus};
pub use context::Context;
```

**Step 5: Create repo-tools crate**

Create `crates/repo-tools/Cargo.toml`:
```toml
[package]
name = "repo-tools"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "Tool integrations for Repository Manager"

[dependencies]
repo-fs = { path = "../repo-fs" }
repo-meta = { path = "../repo-meta" }
repo-blocks = { path = "../repo-blocks" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
rstest = { workspace = true }
pretty_assertions = { workspace = true }
```

Create `crates/repo-tools/src/lib.rs`:
```rust
//! Tool integrations for Repository Manager
//!
//! Syncs configuration to VSCode, Cursor, Claude, etc.

pub mod error;
pub mod integration;
pub mod vscode;
pub mod cursor;
pub mod claude;

pub use error::{Error, Result};
pub use integration::ToolIntegration;
```

**Step 6: Verify workspace compiles**

Run:
```bash
cargo check
```
Expected: Errors about missing modules (expected at this stage)

**Step 7: Commit**

```bash
git add -A
git commit -m "chore: add repo-blocks, repo-meta, repo-presets, repo-tools crates"
```

---

## Phase 1.1: Managed Block System (repo-blocks)

### Task 1.1: Error Types

**Files:**
- Create: `crates/repo-blocks/src/error.rs`

**Step 1: Write error module**

```rust
//! Error types for repo-blocks

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Block not found: {uuid} in {path}")]
    BlockNotFound { uuid: String, path: PathBuf },

    #[error("Malformed block markers in {path}: {message}")]
    MalformedMarkers { path: PathBuf, message: String },

    #[error("Unclosed block: {uuid} in {path}")]
    UnclosedBlock { uuid: String, path: PathBuf },

    #[error("Invalid UUID format: {value}")]
    InvalidUuid { value: String },
}
```

**Step 2: Commit**

```bash
git add crates/repo-blocks/src/error.rs
git commit -m "feat(repo-blocks): add error types"
```

---

### Task 1.2: Block Parser

**Files:**
- Create: `crates/repo-blocks/src/parser.rs`
- Create: `crates/repo-blocks/tests/parser_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-blocks/tests/parser_tests.rs`:
```rust
use repo_blocks::parser::{parse_blocks, Block};

#[test]
fn test_parse_no_blocks() {
    let content = "Just some text\nwith no blocks";
    let blocks = parse_blocks(content);
    assert!(blocks.is_empty());
}

#[test]
fn test_parse_single_block() {
    let content = r#"Before content

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
Block content here
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

After content"#;

    let blocks = parse_blocks(content);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].uuid, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(blocks[0].content.trim(), "Block content here");
}

#[test]
fn test_parse_multiple_blocks() {
    let content = r#"<!-- repo:block:uuid-1 -->
First
<!-- /repo:block:uuid-1 -->

<!-- repo:block:uuid-2 -->
Second
<!-- /repo:block:uuid-2 -->"#;

    let blocks = parse_blocks(content);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].uuid, "uuid-1");
    assert_eq!(blocks[1].uuid, "uuid-2");
}

#[test]
fn test_block_line_positions() {
    let content = r#"Line 1
<!-- repo:block:test-uuid -->
Block line 1
Block line 2
<!-- /repo:block:test-uuid -->
Line 6"#;

    let blocks = parse_blocks(content);
    assert_eq!(blocks[0].start_line, 2); // 0-indexed
    assert_eq!(blocks[0].end_line, 5);
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test -p repo-blocks parser_tests
```
Expected: FAIL - module not found

**Step 3: Write implementation**

Create `crates/repo-blocks/src/parser.rs`:
```rust
//! Parser for managed blocks in text files

use regex::Regex;
use std::sync::LazyLock;

static BLOCK_START: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<!--\s*repo:block:([a-zA-Z0-9\-]+)\s*-->").unwrap()
});

static BLOCK_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<!--\s*/repo:block:([a-zA-Z0-9\-]+)\s*-->").unwrap()
});

/// A parsed managed block from a text file
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// The UUID identifying this block
    pub uuid: String,
    /// The content between the markers (excluding markers)
    pub content: String,
    /// Start line number (0-indexed, line containing opening marker)
    pub start_line: usize,
    /// End line number (0-indexed, line containing closing marker)
    pub end_line: usize,
}

/// Parse all managed blocks from text content
pub fn parse_blocks(content: &str) -> Vec<Block> {
    let lines: Vec<&str> = content.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if let Some(caps) = BLOCK_START.captures(lines[i]) {
            let uuid = caps.get(1).unwrap().as_str().to_string();
            let start_line = i;

            // Find matching end
            let mut j = i + 1;
            while j < lines.len() {
                if let Some(end_caps) = BLOCK_END.captures(lines[j]) {
                    if end_caps.get(1).unwrap().as_str() == uuid {
                        let content = lines[i + 1..j].join("\n");
                        blocks.push(Block {
                            uuid,
                            content,
                            start_line,
                            end_line: j,
                        });
                        i = j;
                        break;
                    }
                }
                j += 1;
            }
        }
        i += 1;
    }

    blocks
}

/// Find a specific block by UUID
pub fn find_block(content: &str, uuid: &str) -> Option<Block> {
    parse_blocks(content).into_iter().find(|b| b.uuid == uuid)
}

/// Check if content contains a block with the given UUID
pub fn has_block(content: &str, uuid: &str) -> bool {
    find_block(content, uuid).is_some()
}
```

**Step 4: Run tests**

```bash
cargo test -p repo-blocks parser_tests
```
Expected: All PASS

**Step 5: Commit**

```bash
git add crates/repo-blocks/src/parser.rs crates/repo-blocks/tests/parser_tests.rs
git commit -m "feat(repo-blocks): implement block parser"
```

---

### Task 1.3: Block Writer

**Files:**
- Create: `crates/repo-blocks/src/writer.rs`
- Create: `crates/repo-blocks/tests/writer_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-blocks/tests/writer_tests.rs`:
```rust
use repo_blocks::writer::{insert_block, update_block, remove_block};

#[test]
fn test_insert_block_empty_file() {
    let content = "";
    let result = insert_block(content, "test-uuid", "New content");

    assert!(result.contains("<!-- repo:block:test-uuid -->"));
    assert!(result.contains("New content"));
    assert!(result.contains("<!-- /repo:block:test-uuid -->"));
}

#[test]
fn test_insert_block_existing_content() {
    let content = "Existing content\n";
    let result = insert_block(content, "test-uuid", "Block content");

    assert!(result.starts_with("Existing content"));
    assert!(result.contains("<!-- repo:block:test-uuid -->"));
}

#[test]
fn test_update_block_replaces_content() {
    let content = r#"Before
<!-- repo:block:uuid-1 -->
Old content
<!-- /repo:block:uuid-1 -->
After"#;

    let result = update_block(content, "uuid-1", "New content").unwrap();

    assert!(result.contains("New content"));
    assert!(!result.contains("Old content"));
    assert!(result.contains("Before"));
    assert!(result.contains("After"));
}

#[test]
fn test_update_nonexistent_block_fails() {
    let content = "No blocks here";
    let result = update_block(content, "missing-uuid", "Content");
    assert!(result.is_err());
}

#[test]
fn test_remove_block() {
    let content = r#"Before
<!-- repo:block:uuid-1 -->
Block content
<!-- /repo:block:uuid-1 -->
After"#;

    let result = remove_block(content, "uuid-1").unwrap();

    assert!(!result.contains("repo:block"));
    assert!(!result.contains("Block content"));
    assert!(result.contains("Before"));
    assert!(result.contains("After"));
}

#[test]
fn test_remove_preserves_other_blocks() {
    let content = r#"<!-- repo:block:keep -->
Keep this
<!-- /repo:block:keep -->
<!-- repo:block:remove -->
Remove this
<!-- /repo:block:remove -->"#;

    let result = remove_block(content, "remove").unwrap();

    assert!(result.contains("Keep this"));
    assert!(!result.contains("Remove this"));
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test -p repo-blocks writer_tests
```

**Step 3: Write implementation**

Create `crates/repo-blocks/src/writer.rs`:
```rust
//! Writer for managed blocks in text files

use crate::parser::{find_block, parse_blocks};
use crate::{Error, Result};

/// Format a block with markers
fn format_block(uuid: &str, content: &str) -> String {
    format!(
        "<!-- repo:block:{} -->\n{}\n<!-- /repo:block:{} -->",
        uuid, content, uuid
    )
}

/// Insert a new block at the end of content
pub fn insert_block(content: &str, uuid: &str, block_content: &str) -> String {
    let block = format_block(uuid, block_content);

    if content.is_empty() {
        block
    } else if content.ends_with('\n') {
        format!("{}\n{}", content, block)
    } else {
        format!("{}\n\n{}", content, block)
    }
}

/// Update an existing block's content
pub fn update_block(content: &str, uuid: &str, new_content: &str) -> Result<String> {
    let block = find_block(content, uuid).ok_or_else(|| Error::BlockNotFound {
        uuid: uuid.to_string(),
        path: "<content>".into(),
    })?;

    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    // Add lines before block
    result.extend(lines[..block.start_line].iter().copied());

    // Add updated block
    result.push(&format!("<!-- repo:block:{} -->", uuid));
    for line in new_content.lines() {
        result.push(line);
    }
    result.push(&format!("<!-- /repo:block:{} -->", uuid));

    // Add lines after block
    if block.end_line + 1 < lines.len() {
        result.extend(lines[block.end_line + 1..].iter().copied());
    }

    // Handle the borrowing issue by collecting owned strings
    let owned_lines: Vec<String> = result.iter().map(|s| s.to_string()).collect();
    Ok(owned_lines.join("\n"))
}

/// Remove a block from content
pub fn remove_block(content: &str, uuid: &str) -> Result<String> {
    let block = find_block(content, uuid).ok_or_else(|| Error::BlockNotFound {
        uuid: uuid.to_string(),
        path: "<content>".into(),
    })?;

    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    // Add lines before block
    result.extend(lines[..block.start_line].iter().copied());

    // Skip block entirely

    // Add lines after block
    if block.end_line + 1 < lines.len() {
        result.extend(lines[block.end_line + 1..].iter().copied());
    }

    Ok(result.join("\n"))
}

/// Insert or update a block (upsert)
pub fn upsert_block(content: &str, uuid: &str, block_content: &str) -> String {
    if find_block(content, uuid).is_some() {
        update_block(content, uuid, block_content).unwrap_or_else(|_| content.to_string())
    } else {
        insert_block(content, uuid, block_content)
    }
}
```

**Step 4: Run tests**

```bash
cargo test -p repo-blocks writer_tests
```

**Step 5: Commit**

```bash
git add crates/repo-blocks/src/writer.rs crates/repo-blocks/tests/writer_tests.rs
git commit -m "feat(repo-blocks): implement block writer with insert/update/remove"
```

---

### Task 1.4: Update lib.rs exports

**Files:**
- Modify: `crates/repo-blocks/src/lib.rs`

**Step 1: Update exports**

```rust
//! Managed block system for Repository Manager
//!
//! Handles UUID-tagged content blocks in text files for safe sync/unroll.

pub mod error;
pub mod parser;
pub mod writer;

pub use error::{Error, Result};
pub use parser::{parse_blocks, find_block, has_block, Block};
pub use writer::{insert_block, update_block, remove_block, upsert_block};
```

**Step 2: Run all tests**

```bash
cargo test -p repo-blocks
```

**Step 3: Commit**

```bash
git add crates/repo-blocks/src/lib.rs
git commit -m "feat(repo-blocks): finalize module exports"
```

---

## Phase 1.2: Configuration System (repo-meta)

### Task 2.1: Error Types

**Files:**
- Create: `crates/repo-meta/src/error.rs`

**Step 1: Write error module**

```rust
//! Error types for repo-meta

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Configuration not found at {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Invalid configuration at {path}: {message}")]
    InvalidConfig { path: PathBuf, message: String },

    #[error("Preset not found: {id}")]
    PresetNotFound { id: String },

    #[error("Tool not found: {id}")]
    ToolNotFound { id: String },

    #[error("Rule not found: {id}")]
    RuleNotFound { id: String },

    #[error("Provider not registered for preset: {preset_id}")]
    ProviderNotRegistered { preset_id: String },
}
```

**Step 2: Commit**

```bash
git add crates/repo-meta/src/error.rs
git commit -m "feat(repo-meta): add error types"
```

---

### Task 2.2: Configuration Types

**Files:**
- Create: `crates/repo-meta/src/config.rs`
- Create: `crates/repo-meta/tests/config_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-meta/tests/config_tests.rs`:
```rust
use repo_meta::config::{RepositoryConfig, load_config};
use repo_fs::NormalizedPath;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_load_minimal_config() {
    let temp = TempDir::new().unwrap();
    let repo_dir = temp.path().join(".repository");
    fs::create_dir(&repo_dir).unwrap();

    fs::write(repo_dir.join("config.toml"), r#"
[core]
version = "1.0"
mode = "standard"

[active]
tools = []
presets = []
"#).unwrap();

    let config = load_config(&NormalizedPath::new(temp.path())).unwrap();
    assert_eq!(config.core.version, "1.0");
    assert_eq!(config.core.mode, repo_meta::config::RepositoryMode::Standard);
}

#[test]
fn test_load_config_with_presets() {
    let temp = TempDir::new().unwrap();
    let repo_dir = temp.path().join(".repository");
    fs::create_dir(&repo_dir).unwrap();

    fs::write(repo_dir.join("config.toml"), r#"
[core]
version = "1.0"
mode = "worktrees"

[active]
tools = ["cursor", "vscode"]
presets = ["env:python"]

[presets.config]
"env:python" = { provider = "uv", version = "3.12" }
"#).unwrap();

    let config = load_config(&NormalizedPath::new(temp.path())).unwrap();
    assert_eq!(config.active.presets, vec!["env:python"]);
    assert!(config.presets_config.contains_key("env:python"));
}

#[test]
fn test_config_not_found() {
    let temp = TempDir::new().unwrap();
    let result = load_config(&NormalizedPath::new(temp.path()));
    assert!(result.is_err());
}
```

**Step 2: Write implementation**

Create `crates/repo-meta/src/config.rs`:
```rust
//! Repository configuration types and loading

use crate::{Error, Result};
use repo_fs::{ConfigStore, NormalizedPath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Repository operating mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryMode {
    #[default]
    Standard,
    Worktrees,
}

/// Core configuration section
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoreConfig {
    pub version: String,
    #[serde(default)]
    pub mode: RepositoryMode,
}

/// Active tools and presets
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ActiveConfig {
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub presets: Vec<String>,
}

/// Sync strategy configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    #[serde(default = "default_strategy")]
    pub strategy: String,
}

fn default_strategy() -> String {
    "smart-append".to_string()
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            strategy: default_strategy(),
        }
    }
}

/// Root repository configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepositoryConfig {
    pub core: CoreConfig,
    #[serde(default)]
    pub active: ActiveConfig,
    #[serde(default)]
    pub sync: SyncConfig,
    #[serde(default, rename = "presets.config")]
    pub presets_config: HashMap<String, toml::Value>,
}

/// Load configuration from .repository/config.toml
pub fn load_config(root: &NormalizedPath) -> Result<RepositoryConfig> {
    let config_path = root.join(".repository").join("config.toml");

    if !config_path.exists() {
        return Err(Error::ConfigNotFound {
            path: config_path.to_native(),
        });
    }

    let store = ConfigStore::new();
    store.load(&config_path).map_err(|e| Error::InvalidConfig {
        path: config_path.to_native(),
        message: e.to_string(),
    })
}

/// Get preset-specific configuration
pub fn get_preset_config(config: &RepositoryConfig, preset_id: &str) -> Option<&toml::Value> {
    config.presets_config.get(preset_id)
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-meta config_tests
```

**Step 4: Commit**

```bash
git add crates/repo-meta/src/config.rs crates/repo-meta/tests/config_tests.rs
git commit -m "feat(repo-meta): implement configuration loading"
```

---

### Task 2.3: Provider Registry

**Files:**
- Create: `crates/repo-meta/src/registry.rs`
- Create: `crates/repo-meta/tests/registry_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-meta/tests/registry_tests.rs`:
```rust
use repo_meta::registry::Registry;

#[test]
fn test_registry_register_and_get() {
    let mut registry = Registry::new();
    registry.register("env:python", "python_provider");

    assert_eq!(registry.get_provider("env:python"), Some(&"python_provider".to_string()));
}

#[test]
fn test_registry_unknown_preset() {
    let registry = Registry::new();
    assert_eq!(registry.get_provider("unknown:preset"), None);
}

#[test]
fn test_registry_list_presets() {
    let mut registry = Registry::new();
    registry.register("env:python", "python");
    registry.register("env:node", "node");

    let presets = registry.list_presets();
    assert!(presets.contains(&"env:python".to_string()));
    assert!(presets.contains(&"env:node".to_string()));
}
```

**Step 2: Write implementation**

Create `crates/repo-meta/src/registry.rs`:
```rust
//! Provider registry for mapping presets to providers

use std::collections::HashMap;

/// Registry mapping preset IDs to provider names
#[derive(Debug, Default)]
pub struct Registry {
    providers: HashMap<String, String>,
}

impl Registry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry with built-in providers
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();

        // Register built-in providers
        registry.register("env:python", "python_uv");
        registry.register("env:python:conda", "python_conda");

        registry
    }

    /// Register a provider for a preset
    pub fn register(&mut self, preset_id: &str, provider_name: &str) {
        self.providers.insert(preset_id.to_string(), provider_name.to_string());
    }

    /// Get the provider for a preset
    pub fn get_provider(&self, preset_id: &str) -> Option<&String> {
        self.providers.get(preset_id)
    }

    /// List all registered preset IDs
    pub fn list_presets(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Check if a preset has a registered provider
    pub fn has_provider(&self, preset_id: &str) -> bool {
        self.providers.contains_key(preset_id)
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-meta registry_tests
```

**Step 4: Commit**

```bash
git add crates/repo-meta/src/registry.rs crates/repo-meta/tests/registry_tests.rs
git commit -m "feat(repo-meta): implement provider registry"
```

---

### Task 2.4: Update lib.rs exports

**Files:**
- Modify: `crates/repo-meta/src/lib.rs`

```rust
//! Configuration and metadata management for Repository Manager
//!
//! Reads .repository/ config and provides resolved configuration to other crates.

pub mod config;
pub mod error;
pub mod registry;

pub use config::{load_config, get_preset_config, RepositoryConfig, RepositoryMode};
pub use error::{Error, Result};
pub use registry::Registry;
```

**Step 2: Commit**

```bash
git add crates/repo-meta/src/lib.rs
git commit -m "feat(repo-meta): finalize module exports"
```

---

## Phase 1.3: Preset System (repo-presets)

### Task 3.1: Error Types and Core Traits

**Files:**
- Create: `crates/repo-presets/src/error.rs`
- Create: `crates/repo-presets/src/provider.rs`
- Create: `crates/repo-presets/src/context.rs`

**Step 1: Write error module**

Create `crates/repo-presets/src/error.rs`:
```rust
//! Error types for repo-presets

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Meta error: {0}")]
    Meta(#[from] repo_meta::Error),

    #[error("Command failed: {command}")]
    CommandFailed { command: String },

    #[error("Command not found: {command}")]
    CommandNotFound { command: String },

    #[error("Environment creation failed at {path}: {message}")]
    EnvCreationFailed { path: PathBuf, message: String },

    #[error("Python not found. Install Python or uv first.")]
    PythonNotFound,

    #[error("uv not found. Install uv: https://docs.astral.sh/uv/")]
    UvNotFound,

    #[error("Preset check failed: {message}")]
    CheckFailed { message: String },
}
```

**Step 2: Write provider trait**

Create `crates/repo-presets/src/provider.rs`:
```rust
//! PresetProvider trait and related types

use crate::context::Context;
use crate::Result;
use async_trait::async_trait;

/// Status of a preset after checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetStatus {
    /// Preset is correctly configured and working
    Healthy,
    /// Preset is not installed/configured
    Missing,
    /// Preset exists but has drifted from desired state
    Drifted,
    /// Preset is broken and needs repair
    Broken,
}

/// Remedial action needed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    None,
    Install,
    Repair,
    Update,
}

/// Report from checking a preset
#[derive(Debug, Clone)]
pub struct CheckReport {
    pub status: PresetStatus,
    pub details: Vec<String>,
    pub action: ActionType,
}

impl CheckReport {
    pub fn healthy() -> Self {
        Self {
            status: PresetStatus::Healthy,
            details: vec![],
            action: ActionType::None,
        }
    }

    pub fn missing(detail: impl Into<String>) -> Self {
        Self {
            status: PresetStatus::Missing,
            details: vec![detail.into()],
            action: ActionType::Install,
        }
    }

    pub fn drifted(detail: impl Into<String>) -> Self {
        Self {
            status: PresetStatus::Drifted,
            details: vec![detail.into()],
            action: ActionType::Repair,
        }
    }
}

/// Report from applying a preset
#[derive(Debug, Clone)]
pub struct ApplyReport {
    pub success: bool,
    pub actions_taken: Vec<String>,
    pub errors: Vec<String>,
}

impl ApplyReport {
    pub fn success(actions: Vec<String>) -> Self {
        Self {
            success: true,
            actions_taken: actions,
            errors: vec![],
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            actions_taken: vec![],
            errors,
        }
    }
}

/// Core trait for preset providers
#[async_trait]
pub trait PresetProvider: Send + Sync {
    /// Unique identifier (e.g., "env:python")
    fn id(&self) -> &str;

    /// Check current state against desired state
    async fn check(&self, context: &Context) -> Result<CheckReport>;

    /// Apply changes to reach desired state
    async fn apply(&self, context: &Context) -> Result<ApplyReport>;
}
```

**Step 3: Write context**

Create `crates/repo-presets/src/context.rs`:
```rust
//! Execution context for preset providers

use repo_fs::{NormalizedPath, WorkspaceLayout};
use std::collections::HashMap;

/// Context passed to providers for check/apply operations
#[derive(Debug, Clone)]
pub struct Context {
    /// Workspace layout information
    pub layout: WorkspaceLayout,

    /// Root path where operations should occur
    pub root: NormalizedPath,

    /// Preset-specific configuration from config.toml
    pub config: HashMap<String, toml::Value>,
}

impl Context {
    /// Create a new context
    pub fn new(layout: WorkspaceLayout, config: HashMap<String, toml::Value>) -> Self {
        let root = layout.root.clone();
        Self { layout, root, config }
    }

    /// Get a configuration value as a string
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.config.get(key).and_then(|v| v.as_str().map(String::from))
    }

    /// Get the Python version from config, defaulting to "3.12"
    pub fn python_version(&self) -> String {
        self.get_string("version").unwrap_or_else(|| "3.12".to_string())
    }

    /// Get the provider preference (uv, conda, etc.)
    pub fn provider(&self) -> String {
        self.get_string("provider").unwrap_or_else(|| "uv".to_string())
    }

    /// Path where venv should be created
    pub fn venv_path(&self) -> NormalizedPath {
        self.root.join(".venv")
    }
}
```

**Step 4: Commit**

```bash
git add crates/repo-presets/src/error.rs crates/repo-presets/src/provider.rs crates/repo-presets/src/context.rs
git commit -m "feat(repo-presets): add error types, provider trait, and context"
```

---

### Task 3.2: Python Provider (uv)

**Files:**
- Create: `crates/repo-presets/src/python/mod.rs`
- Create: `crates/repo-presets/src/python/uv.rs`
- Create: `crates/repo-presets/tests/python_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-presets/tests/python_tests.rs`:
```rust
use repo_presets::python::UvProvider;
use repo_presets::provider::PresetProvider;
use repo_presets::context::Context;
use repo_fs::{NormalizedPath, WorkspaceLayout, LayoutMode};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_context(temp: &TempDir) -> Context {
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };
    let mut config = HashMap::new();
    config.insert("version".to_string(), toml::Value::String("3.12".to_string()));
    Context::new(layout, config)
}

#[tokio::test]
async fn test_uv_provider_id() {
    let provider = UvProvider::new();
    assert_eq!(provider.id(), "env:python");
}

#[tokio::test]
async fn test_uv_check_missing_venv() {
    let temp = TempDir::new().unwrap();
    let context = create_test_context(&temp);

    let provider = UvProvider::new();
    let report = provider.check(&context).await.unwrap();

    // Should report missing since no venv exists
    assert_eq!(report.status, repo_presets::PresetStatus::Missing);
}
```

**Step 2: Write implementation**

Create `crates/repo-presets/src/python/mod.rs`:
```rust
//! Python environment providers

mod uv;

pub use uv::UvProvider;
```

Create `crates/repo-presets/src/python/uv.rs`:
```rust
//! uv-based Python environment provider

use crate::context::Context;
use crate::error::{Error, Result};
use crate::provider::{ApplyReport, CheckReport, PresetProvider, PresetStatus, ActionType};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

/// Python environment provider using uv
pub struct UvProvider;

impl UvProvider {
    pub fn new() -> Self {
        Self
    }

    /// Check if uv is available
    async fn check_uv_available(&self) -> bool {
        Command::new("uv")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if venv exists and is valid
    fn check_venv_exists(&self, context: &Context) -> bool {
        let venv_path = context.venv_path();
        let python_path = if cfg!(windows) {
            venv_path.join("Scripts").join("python.exe")
        } else {
            venv_path.join("bin").join("python")
        };
        python_path.exists()
    }
}

impl Default for UvProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PresetProvider for UvProvider {
    fn id(&self) -> &str {
        "env:python"
    }

    async fn check(&self, context: &Context) -> Result<CheckReport> {
        // Check if uv is available
        if !self.check_uv_available().await {
            return Ok(CheckReport {
                status: PresetStatus::Broken,
                details: vec!["uv not found. Install from https://docs.astral.sh/uv/".to_string()],
                action: ActionType::Install,
            });
        }

        // Check if venv exists
        if !self.check_venv_exists(context) {
            return Ok(CheckReport::missing("Virtual environment not found"));
        }

        Ok(CheckReport::healthy())
    }

    async fn apply(&self, context: &Context) -> Result<ApplyReport> {
        let venv_path = context.venv_path();
        let python_version = context.python_version();

        // Create venv using uv
        let status = Command::new("uv")
            .args(["venv", "--python", &python_version])
            .arg(venv_path.to_native())
            .current_dir(context.root.to_native())
            .status()
            .await
            .map_err(|_| Error::UvNotFound)?;

        if !status.success() {
            return Ok(ApplyReport::failure(vec![
                format!("Failed to create venv with Python {}", python_version)
            ]));
        }

        Ok(ApplyReport::success(vec![
            format!("Created virtual environment at {}", venv_path),
            format!("Python version: {}", python_version),
        ]))
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-presets python_tests
```

**Step 4: Commit**

```bash
git add crates/repo-presets/src/python/
git add crates/repo-presets/tests/python_tests.rs
git commit -m "feat(repo-presets): implement UvProvider for env:python"
```

---

### Task 3.3: Update lib.rs exports

**Files:**
- Modify: `crates/repo-presets/src/lib.rs`

```rust
//! Preset providers for Repository Manager
//!
//! Implements PresetProvider trait for env:python, config:*, tool:*.

pub mod context;
pub mod error;
pub mod provider;
pub mod python;

pub use context::Context;
pub use error::{Error, Result};
pub use provider::{ApplyReport, CheckReport, PresetProvider, PresetStatus, ActionType};
pub use python::UvProvider;
```

**Step 2: Commit**

```bash
git add crates/repo-presets/src/lib.rs
git commit -m "feat(repo-presets): finalize module exports"
```

---

## Phase 1.4: Tool Integration (repo-tools)

### Task 4.1: Error Types and Core Trait

**Files:**
- Create: `crates/repo-tools/src/error.rs`
- Create: `crates/repo-tools/src/integration.rs`

**Step 1: Write error module**

Create `crates/repo-tools/src/error.rs`:
```rust
//! Error types for repo-tools

use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Filesystem error: {0}")]
    Fs(#[from] repo_fs::Error),

    #[error("Block error: {0}")]
    Block(#[from] repo_blocks::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Tool config not found at {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Sync failed for {tool}: {message}")]
    SyncFailed { tool: String, message: String },
}
```

**Step 2: Write trait**

Create `crates/repo-tools/src/integration.rs`:
```rust
//! ToolIntegration trait for syncing to external tools

use crate::Result;
use repo_fs::NormalizedPath;

/// Rule to be synced to tools
#[derive(Debug, Clone)]
pub struct Rule {
    pub id: String,
    pub content: String,
}

/// Context for tool sync operations
#[derive(Debug, Clone)]
pub struct SyncContext {
    pub root: NormalizedPath,
    pub python_path: Option<NormalizedPath>,
}

impl SyncContext {
    pub fn new(root: NormalizedPath) -> Self {
        Self {
            root,
            python_path: None,
        }
    }

    pub fn with_python(mut self, path: NormalizedPath) -> Self {
        self.python_path = Some(path);
        self
    }
}

/// Trait for tool integrations
pub trait ToolIntegration {
    /// Tool name (e.g., "vscode", "cursor")
    fn name(&self) -> &str;

    /// Config file paths relative to root
    fn config_paths(&self) -> Vec<&str>;

    /// Sync rules and state to tool's config
    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()>;
}
```

**Step 3: Commit**

```bash
git add crates/repo-tools/src/error.rs crates/repo-tools/src/integration.rs
git commit -m "feat(repo-tools): add error types and ToolIntegration trait"
```

---

### Task 4.2: VSCode Integration

**Files:**
- Create: `crates/repo-tools/src/vscode.rs`
- Create: `crates/repo-tools/tests/vscode_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-tools/tests/vscode_tests.rs`:
```rust
use repo_tools::vscode::VSCodeIntegration;
use repo_tools::integration::{ToolIntegration, SyncContext, Rule};
use repo_fs::NormalizedPath;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_vscode_name() {
    let vscode = VSCodeIntegration::new();
    assert_eq!(vscode.name(), "vscode");
}

#[test]
fn test_vscode_config_paths() {
    let vscode = VSCodeIntegration::new();
    let paths = vscode.config_paths();
    assert!(paths.contains(&".vscode/settings.json"));
}

#[test]
fn test_vscode_sync_creates_settings() {
    let temp = TempDir::new().unwrap();
    let context = SyncContext::new(NormalizedPath::new(temp.path()))
        .with_python(NormalizedPath::new(temp.path()).join(".venv/Scripts/python.exe"));

    let vscode = VSCodeIntegration::new();
    vscode.sync(&context, &[]).unwrap();

    assert!(temp.path().join(".vscode/settings.json").exists());
}

#[test]
fn test_vscode_sync_sets_python_path() {
    let temp = TempDir::new().unwrap();
    let venv_python = temp.path().join(".venv/Scripts/python.exe");
    let context = SyncContext::new(NormalizedPath::new(temp.path()))
        .with_python(NormalizedPath::new(&venv_python));

    let vscode = VSCodeIntegration::new();
    vscode.sync(&context, &[]).unwrap();

    let settings = fs::read_to_string(temp.path().join(".vscode/settings.json")).unwrap();
    assert!(settings.contains("python.defaultInterpreterPath"));
}
```

**Step 2: Write implementation**

Create `crates/repo-tools/src/vscode.rs`:
```rust
//! VSCode integration

use crate::error::{Error, Result};
use crate::integration::{Rule, SyncContext, ToolIntegration};
use repo_fs::NormalizedPath;
use serde_json::{json, Value};
use std::fs;

/// VSCode tool integration
pub struct VSCodeIntegration;

impl VSCodeIntegration {
    pub fn new() -> Self {
        Self
    }

    fn settings_path(&self, root: &NormalizedPath) -> NormalizedPath {
        root.join(".vscode").join("settings.json")
    }

    fn load_settings(&self, path: &NormalizedPath) -> Result<Value> {
        if path.exists() {
            let content = fs::read_to_string(path.to_native())
                .map_err(|e| repo_fs::Error::io(path.to_native(), e))?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(json!({}))
        }
    }

    fn save_settings(&self, path: &NormalizedPath, settings: &Value) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent.to_native())
                .map_err(|e| repo_fs::Error::io(parent.to_native(), e))?;
        }

        let content = serde_json::to_string_pretty(settings)?;
        fs::write(path.to_native(), content)
            .map_err(|e| repo_fs::Error::io(path.to_native(), e))?;
        Ok(())
    }
}

impl Default for VSCodeIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolIntegration for VSCodeIntegration {
    fn name(&self) -> &str {
        "vscode"
    }

    fn config_paths(&self) -> Vec<&str> {
        vec![".vscode/settings.json"]
    }

    fn sync(&self, context: &SyncContext, _rules: &[Rule]) -> Result<()> {
        let settings_path = self.settings_path(&context.root);
        let mut settings = self.load_settings(&settings_path)?;

        // Set Python interpreter path if available
        if let Some(ref python_path) = context.python_path {
            settings["python.defaultInterpreterPath"] =
                json!(python_path.as_str());
        }

        self.save_settings(&settings_path, &settings)?;
        Ok(())
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-tools vscode_tests
```

**Step 4: Commit**

```bash
git add crates/repo-tools/src/vscode.rs crates/repo-tools/tests/vscode_tests.rs
git commit -m "feat(repo-tools): implement VSCode integration"
```

---

### Task 4.3: Cursor Integration

**Files:**
- Create: `crates/repo-tools/src/cursor.rs`
- Create: `crates/repo-tools/tests/cursor_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-tools/tests/cursor_tests.rs`:
```rust
use repo_tools::cursor::CursorIntegration;
use repo_tools::integration::{ToolIntegration, SyncContext, Rule};
use repo_fs::NormalizedPath;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cursor_name() {
    let cursor = CursorIntegration::new();
    assert_eq!(cursor.name(), "cursor");
}

#[test]
fn test_cursor_sync_creates_cursorrules() {
    let temp = TempDir::new().unwrap();
    let context = SyncContext::new(NormalizedPath::new(temp.path()));

    let rules = vec![
        Rule { id: "test-rule".to_string(), content: "Test rule content".to_string() },
    ];

    let cursor = CursorIntegration::new();
    cursor.sync(&context, &rules).unwrap();

    assert!(temp.path().join(".cursorrules").exists());
}

#[test]
fn test_cursor_sync_uses_managed_blocks() {
    let temp = TempDir::new().unwrap();
    let context = SyncContext::new(NormalizedPath::new(temp.path()));

    let rules = vec![
        Rule { id: "rule-1".to_string(), content: "Rule 1 content".to_string() },
    ];

    let cursor = CursorIntegration::new();
    cursor.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
    assert!(content.contains("repo:block:"));
    assert!(content.contains("Rule 1 content"));
}

#[test]
fn test_cursor_sync_preserves_user_content() {
    let temp = TempDir::new().unwrap();

    // Create existing .cursorrules with user content
    fs::write(temp.path().join(".cursorrules"), "User's custom rules\n").unwrap();

    let context = SyncContext::new(NormalizedPath::new(temp.path()));
    let rules = vec![
        Rule { id: "managed-rule".to_string(), content: "Managed content".to_string() },
    ];

    let cursor = CursorIntegration::new();
    cursor.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
    assert!(content.contains("User's custom rules"));
    assert!(content.contains("Managed content"));
}
```

**Step 2: Write implementation**

Create `crates/repo-tools/src/cursor.rs`:
```rust
//! Cursor integration using managed blocks

use crate::error::{Error, Result};
use crate::integration::{Rule, SyncContext, ToolIntegration};
use repo_blocks::{upsert_block, remove_block, has_block};
use repo_fs::NormalizedPath;
use std::fs;

/// Cursor tool integration
pub struct CursorIntegration;

impl CursorIntegration {
    pub fn new() -> Self {
        Self
    }

    fn cursorrules_path(&self, root: &NormalizedPath) -> NormalizedPath {
        root.join(".cursorrules")
    }

    fn load_content(&self, path: &NormalizedPath) -> String {
        if path.exists() {
            fs::read_to_string(path.to_native()).unwrap_or_default()
        } else {
            String::new()
        }
    }

    fn save_content(&self, path: &NormalizedPath, content: &str) -> Result<()> {
        fs::write(path.to_native(), content)
            .map_err(|e| repo_fs::Error::io(path.to_native(), e))?;
        Ok(())
    }

    /// Generate block content for a rule
    fn format_rule_block(&self, rule: &Rule) -> String {
        format!("## {}\n\n{}", rule.id, rule.content)
    }
}

impl Default for CursorIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolIntegration for CursorIntegration {
    fn name(&self) -> &str {
        "cursor"
    }

    fn config_paths(&self) -> Vec<&str> {
        vec![".cursorrules"]
    }

    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let cursorrules_path = self.cursorrules_path(&context.root);
        let mut content = self.load_content(&cursorrules_path);

        // Update or insert each rule as a managed block
        for rule in rules {
            let block_content = self.format_rule_block(rule);
            content = upsert_block(&content, &rule.id, &block_content);
        }

        self.save_content(&cursorrules_path, &content)?;
        Ok(())
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-tools cursor_tests
```

**Step 4: Commit**

```bash
git add crates/repo-tools/src/cursor.rs crates/repo-tools/tests/cursor_tests.rs
git commit -m "feat(repo-tools): implement Cursor integration with managed blocks"
```

---

### Task 4.4: Claude Integration

**Files:**
- Create: `crates/repo-tools/src/claude.rs`
- Create: `crates/repo-tools/tests/claude_tests.rs`

**Step 1: Write failing tests**

Create `crates/repo-tools/tests/claude_tests.rs`:
```rust
use repo_tools::claude::ClaudeIntegration;
use repo_tools::integration::{ToolIntegration, SyncContext, Rule};
use repo_fs::NormalizedPath;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_claude_name() {
    let claude = ClaudeIntegration::new();
    assert_eq!(claude.name(), "claude");
}

#[test]
fn test_claude_sync_creates_claude_md() {
    let temp = TempDir::new().unwrap();
    let context = SyncContext::new(NormalizedPath::new(temp.path()));

    let rules = vec![
        Rule { id: "test-rule".to_string(), content: "Test content".to_string() },
    ];

    let claude = ClaudeIntegration::new();
    claude.sync(&context, &rules).unwrap();

    assert!(temp.path().join("CLAUDE.md").exists());
}

#[test]
fn test_claude_sync_uses_managed_blocks() {
    let temp = TempDir::new().unwrap();
    let context = SyncContext::new(NormalizedPath::new(temp.path()));

    let rules = vec![
        Rule { id: "python-style".to_string(), content: "Use snake_case".to_string() },
    ];

    let claude = ClaudeIntegration::new();
    claude.sync(&context, &rules).unwrap();

    let content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("repo:block:"));
    assert!(content.contains("Use snake_case"));
}
```

**Step 2: Write implementation**

Create `crates/repo-tools/src/claude.rs`:
```rust
//! Claude Code integration using managed blocks

use crate::error::Result;
use crate::integration::{Rule, SyncContext, ToolIntegration};
use repo_blocks::upsert_block;
use repo_fs::NormalizedPath;
use std::fs;

/// Claude Code tool integration
pub struct ClaudeIntegration;

impl ClaudeIntegration {
    pub fn new() -> Self {
        Self
    }

    fn claude_md_path(&self, root: &NormalizedPath) -> NormalizedPath {
        root.join("CLAUDE.md")
    }

    fn load_content(&self, path: &NormalizedPath) -> String {
        if path.exists() {
            fs::read_to_string(path.to_native()).unwrap_or_default()
        } else {
            // Default header for new CLAUDE.md
            "# Project Instructions\n\n".to_string()
        }
    }

    fn save_content(&self, path: &NormalizedPath, content: &str) -> Result<()> {
        fs::write(path.to_native(), content)
            .map_err(|e| repo_fs::Error::io(path.to_native(), e))?;
        Ok(())
    }

    fn format_rule_block(&self, rule: &Rule) -> String {
        format!("## {}\n\n{}", rule.id, rule.content)
    }
}

impl Default for ClaudeIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolIntegration for ClaudeIntegration {
    fn name(&self) -> &str {
        "claude"
    }

    fn config_paths(&self) -> Vec<&str> {
        vec!["CLAUDE.md", ".claude/rules/"]
    }

    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()> {
        let claude_md_path = self.claude_md_path(&context.root);
        let mut content = self.load_content(&claude_md_path);

        for rule in rules {
            let block_content = self.format_rule_block(rule);
            content = upsert_block(&content, &rule.id, &block_content);
        }

        self.save_content(&claude_md_path, &content)?;
        Ok(())
    }
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-tools claude_tests
```

**Step 4: Commit**

```bash
git add crates/repo-tools/src/claude.rs crates/repo-tools/tests/claude_tests.rs
git commit -m "feat(repo-tools): implement Claude integration"
```

---

### Task 4.5: Update lib.rs exports

**Files:**
- Modify: `crates/repo-tools/src/lib.rs`

```rust
//! Tool integrations for Repository Manager
//!
//! Syncs configuration to VSCode, Cursor, Claude, etc.

pub mod claude;
pub mod cursor;
pub mod error;
pub mod integration;
pub mod vscode;

pub use claude::ClaudeIntegration;
pub use cursor::CursorIntegration;
pub use error::{Error, Result};
pub use integration::{Rule, SyncContext, ToolIntegration};
pub use vscode::VSCodeIntegration;
```

**Step 2: Commit**

```bash
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): finalize module exports"
```

---

## Phase 1.5: Integration Test

### Task 5.1: End-to-End Integration Test

**Files:**
- Create: `tests/integration_test.rs` (workspace level)

**Step 1: Write integration test**

Create `tests/integration_test.rs`:
```rust
//! End-to-end integration test for the vertical slice

use repo_fs::{NormalizedPath, WorkspaceLayout, LayoutMode};
use repo_meta::{load_config, Registry};
use repo_presets::{Context, UvProvider, PresetProvider, PresetStatus};
use repo_tools::{VSCodeIntegration, CursorIntegration, ClaudeIntegration, ToolIntegration, SyncContext, Rule};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Set up a test repository with config
fn setup_test_repo() -> TempDir {
    let temp = TempDir::new().unwrap();
    let repo_dir = temp.path().join(".repository");
    fs::create_dir(&repo_dir).unwrap();

    fs::write(repo_dir.join("config.toml"), r#"
[core]
version = "1.0"
mode = "standard"

[active]
tools = ["vscode", "cursor", "claude"]
presets = ["env:python"]

[presets.config]
"env:python" = { provider = "uv", version = "3.12" }
"#).unwrap();

    temp
}

#[test]
fn test_load_config_and_registry() {
    let temp = setup_test_repo();
    let root = NormalizedPath::new(temp.path());

    // Load config
    let config = load_config(&root).unwrap();
    assert_eq!(config.active.presets, vec!["env:python"]);

    // Registry has provider
    let registry = Registry::with_builtins();
    assert!(registry.has_provider("env:python"));
}

#[tokio::test]
async fn test_python_provider_check() {
    let temp = setup_test_repo();
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(temp.path()),
        active_context: NormalizedPath::new(temp.path()),
        mode: LayoutMode::Classic,
    };

    let config = load_config(&layout.root).unwrap();
    let preset_config = config.presets_config.get("env:python")
        .map(|v| {
            let mut map = HashMap::new();
            if let Some(table) = v.as_table() {
                for (k, v) in table {
                    map.insert(k.clone(), v.clone());
                }
            }
            map
        })
        .unwrap_or_default();

    let context = Context::new(layout, preset_config);
    let provider = UvProvider::new();

    let report = provider.check(&context).await.unwrap();
    // Should be missing since we haven't applied yet
    assert!(report.status == PresetStatus::Missing || report.status == PresetStatus::Broken);
}

#[test]
fn test_tool_sync() {
    let temp = setup_test_repo();
    let root = NormalizedPath::new(temp.path());

    let rules = vec![
        Rule {
            id: "python-style".to_string(),
            content: "Use snake_case for variables".to_string()
        },
    ];

    let context = SyncContext::new(root.clone())
        .with_python(root.join(".venv/Scripts/python.exe"));

    // Sync to all tools
    VSCodeIntegration::new().sync(&context, &rules).unwrap();
    CursorIntegration::new().sync(&context, &rules).unwrap();
    ClaudeIntegration::new().sync(&context, &rules).unwrap();

    // Verify files created
    assert!(temp.path().join(".vscode/settings.json").exists());
    assert!(temp.path().join(".cursorrules").exists());
    assert!(temp.path().join("CLAUDE.md").exists());

    // Verify content
    let cursorrules = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
    assert!(cursorrules.contains("python-style"));
    assert!(cursorrules.contains("snake_case"));
}
```

**Step 2: Run integration test**

```bash
cargo test --test integration_test
```

**Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add end-to-end integration test for vertical slice"
```

---

## Phase 1.6: Final Verification

### Task 6.1: Run All Tests and Cleanup

**Step 1: Run all tests**

```bash
cargo test --workspace
```

**Step 2: Run clippy**

```bash
cargo clippy --workspace -- -D warnings
```

**Step 3: Format code**

```bash
cargo fmt --all
```

**Step 4: Final commit**

```bash
git add -A
git commit -m "chore: cleanup and formatting"
```

---

## Verification Checklist

### repo-blocks
- [ ] `parse_blocks()` extracts UUID-tagged blocks
- [ ] `insert_block()` adds new blocks at end
- [ ] `update_block()` replaces existing block content
- [ ] `remove_block()` removes block preserving other content
- [ ] `upsert_block()` handles both insert and update

### repo-meta
- [ ] `load_config()` parses .repository/config.toml
- [ ] `Registry` maps preset IDs to providers
- [ ] Preset-specific config accessible via `get_preset_config()`

### repo-presets
- [ ] `PresetProvider` trait defines check/apply lifecycle
- [ ] `UvProvider` checks for venv existence
- [ ] `UvProvider` creates venv via `uv venv`
- [ ] `Context` provides preset config and paths

### repo-tools
- [ ] `VSCodeIntegration` syncs python.defaultInterpreterPath
- [ ] `CursorIntegration` uses managed blocks in .cursorrules
- [ ] `ClaudeIntegration` uses managed blocks in CLAUDE.md
- [ ] All integrations preserve user content outside blocks

---

## Next Steps (Phase 2)

After Phase 1 vertical slice is complete:

1. **Conda Provider** - Add `CondaProvider` as alternative to `UvProvider`
2. **Additional Presets** - `env:node`, `config:git`, `tool:ruff`
3. **More Tools** - Windsurf, Antigravity, Copilot
4. **State Ledger** - Implement `.repository/ledger.toml` for unrolling
5. **CLI** - Start `repo-cli` with `repo sync` command
