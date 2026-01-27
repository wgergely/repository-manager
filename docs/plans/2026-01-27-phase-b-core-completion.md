# Phase B: Core System Completion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Date:** 2026-01-27
**Priority:** High
**Estimated Tasks:** 8
**Dependencies:** Phase A (Immediate Fixes)

---

## Goal

Complete the `repo-core` orchestration layer to deliver the core value proposition: applying intents (rules, presets, tools) to concrete projections (config files).

---

## Prerequisites

- Phase A completed
- All tests passing: `cargo test --workspace`

---

## Architecture Overview

```
User Command: "repo add-tool cursor"
        |
        v
    CLI Layer (repo-cli)
        |
        v
    Core Layer (repo-core)
        |
        +-- ConfigResolver: Load & merge configs
        +-- SyncEngine: Determine needed changes
        +-- ProjectionWriter: Apply changes to files
        |
        v
    Layer 0 Crates
        +-- repo-content: File manipulation
        +-- repo-tools: Tool-specific logic
        +-- repo-presets: Preset application
```

---

## Task B.1: Implement ProjectionWriter

**Files:**
- Create: `crates/repo-core/src/projection/mod.rs`
- Create: `crates/repo-core/src/projection/writer.rs`
- Modify: `crates/repo-core/src/lib.rs`

**Step 1: Create module structure**

```rust
// crates/repo-core/src/projection/mod.rs
mod writer;

pub use writer::ProjectionWriter;
```

**Step 2: Implement ProjectionWriter**

```rust
// crates/repo-core/src/projection/writer.rs
//! Writes projections to the filesystem

use crate::ledger::{Projection, ProjectionKind};
use crate::{Error, Result};
use repo_content::{BlockLocation, Document, Format};
use repo_fs::NormalizedPath;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub struct ProjectionWriter {
    root: NormalizedPath,
    dry_run: bool,
}

impl ProjectionWriter {
    pub fn new(root: NormalizedPath, dry_run: bool) -> Self {
        Self { root, dry_run }
    }

    /// Apply a projection to the filesystem
    pub fn apply(&self, projection: &Projection, content: &str) -> Result<String> {
        let file_path = self.root.join(projection.file.to_string_lossy().as_ref());

        match &projection.kind {
            ProjectionKind::FileManaged { .. } => {
                self.write_managed_file(&file_path, content)
            }
            ProjectionKind::TextBlock { marker, .. } => {
                self.write_text_block(&file_path, *marker, content)
            }
            ProjectionKind::JsonKey { path, .. } => {
                self.write_json_key(&file_path, path, content)
            }
        }
    }

    /// Remove a projection from the filesystem
    pub fn remove(&self, projection: &Projection) -> Result<String> {
        let file_path = self.root.join(projection.file.to_string_lossy().as_ref());

        match &projection.kind {
            ProjectionKind::FileManaged { .. } => {
                self.remove_managed_file(&file_path)
            }
            ProjectionKind::TextBlock { marker, .. } => {
                self.remove_text_block(&file_path, *marker)
            }
            ProjectionKind::JsonKey { path, .. } => {
                self.remove_json_key(&file_path, path)
            }
        }
    }

    fn write_managed_file(&self, path: &NormalizedPath, content: &str) -> Result<String> {
        if self.dry_run {
            return Ok(format!("[dry-run] Would create {}", path.display()));
        }

        // Create parent directories
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path.as_ref(), content)?;
        Ok(format!("Created {}", path.display()))
    }

    fn write_text_block(
        &self,
        path: &NormalizedPath,
        marker: Uuid,
        content: &str,
    ) -> Result<String> {
        let existing = if path.exists() {
            fs::read_to_string(path.as_ref())?
        } else {
            String::new()
        };

        let format = Format::from_path(path.as_ref());
        let mut doc = Document::parse_as(&existing, format)?;

        // Check if block already exists
        let blocks = doc.find_blocks();
        let existing_block = blocks.iter().find(|b| b.uuid == marker);

        let (new_source, _edit) = if existing_block.is_some() {
            doc.update_block(marker, content)?
        } else {
            doc.insert_block(marker, content, BlockLocation::End)?
        };

        if self.dry_run {
            return Ok(format!("[dry-run] Would update block {} in {}", marker, path.display()));
        }

        fs::write(path.as_ref(), new_source)?;
        Ok(format!("Updated block {} in {}", marker, path.display()))
    }

    fn write_json_key(&self, path: &NormalizedPath, key_path: &str, value: &str) -> Result<String> {
        let existing = if path.exists() {
            fs::read_to_string(path.as_ref())?
        } else {
            "{}".to_string()
        };

        let mut json: serde_json::Value = serde_json::from_str(&existing)
            .map_err(|e| Error::Content(format!("Invalid JSON: {}", e)))?;

        let value: serde_json::Value = serde_json::from_str(value)
            .map_err(|e| Error::Content(format!("Invalid value: {}", e)))?;

        set_json_path(&mut json, key_path, value);

        if self.dry_run {
            return Ok(format!("[dry-run] Would set {} in {}", key_path, path.display()));
        }

        let output = serde_json::to_string_pretty(&json)?;
        fs::write(path.as_ref(), output)?;
        Ok(format!("Set {} in {}", key_path, path.display()))
    }

    fn remove_managed_file(&self, path: &NormalizedPath) -> Result<String> {
        if self.dry_run {
            return Ok(format!("[dry-run] Would delete {}", path.display()));
        }

        if path.exists() {
            fs::remove_file(path.as_ref())?;
            Ok(format!("Deleted {}", path.display()))
        } else {
            Ok(format!("File already missing: {}", path.display()))
        }
    }

    fn remove_text_block(&self, path: &NormalizedPath, marker: Uuid) -> Result<String> {
        if !path.exists() {
            return Ok(format!("File already missing: {}", path.display()));
        }

        let existing = fs::read_to_string(path.as_ref())?;
        let format = Format::from_path(path.as_ref());
        let mut doc = Document::parse_as(&existing, format)?;

        let (new_source, _edit) = doc.remove_block(marker)?;

        if self.dry_run {
            return Ok(format!("[dry-run] Would remove block {} from {}", marker, path.display()));
        }

        fs::write(path.as_ref(), new_source)?;
        Ok(format!("Removed block {} from {}", marker, path.display()))
    }

    fn remove_json_key(&self, path: &NormalizedPath, key_path: &str) -> Result<String> {
        if !path.exists() {
            return Ok(format!("File already missing: {}", path.display()));
        }

        let existing = fs::read_to_string(path.as_ref())?;
        let mut json: serde_json::Value = serde_json::from_str(&existing)
            .map_err(|e| Error::Content(format!("Invalid JSON: {}", e)))?;

        remove_json_path(&mut json, key_path);

        if self.dry_run {
            return Ok(format!("[dry-run] Would remove {} from {}", key_path, path.display()));
        }

        let output = serde_json::to_string_pretty(&json)?;
        fs::write(path.as_ref(), output)?;
        Ok(format!("Removed {} from {}", key_path, path.display()))
    }
}

/// Set a value at a JSON path (e.g., "editor.fontSize")
fn set_json_path(json: &mut serde_json::Value, path: &str, value: serde_json::Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            if let serde_json::Value::Object(map) = current {
                map.insert(part.to_string(), value);
            }
            return;
        }

        // Navigate deeper, creating objects as needed
        if let serde_json::Value::Object(map) = current {
            current = map
                .entry(part.to_string())
                .or_insert(serde_json::Value::Object(serde_json::Map::new()));
        }
    }
}

/// Remove a key at a JSON path
fn remove_json_path(json: &mut serde_json::Value, path: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return;
    }

    let mut current = json;
    for part in &parts[..parts.len() - 1] {
        if let serde_json::Value::Object(map) = current {
            if let Some(next) = map.get_mut(*part) {
                current = next;
            } else {
                return;
            }
        } else {
            return;
        }
    }

    if let serde_json::Value::Object(map) = current {
        map.remove(parts.last().unwrap().to_string().as_str());
    }
}

/// Compute checksum of content
pub fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

**Step 3: Export from lib.rs**

```rust
pub mod projection;
pub use projection::ProjectionWriter;
```

**Step 4: Run tests**

```bash
cargo test -p repo-core
```

**Step 5: Commit**

```bash
git add crates/repo-core/src/projection/
git commit -m "feat(repo-core): implement ProjectionWriter

Handles writing and removing projections:
- FileManaged: whole file management
- TextBlock: UUID-marked blocks in text files
- JsonKey: specific keys in JSON files

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task B.2: Implement ToolSyncer

**Files:**
- Create: `crates/repo-core/src/sync/tool_syncer.rs`
- Modify: `crates/repo-core/src/sync/mod.rs`

**Step 1: Create ToolSyncer**

```rust
// crates/repo-core/src/sync/tool_syncer.rs
//! Synchronizes tool configurations

use crate::ledger::{Intent, Ledger, Projection, ProjectionKind};
use crate::projection::ProjectionWriter;
use crate::{Error, Result};
use chrono::Utc;
use repo_fs::NormalizedPath;
use repo_tools::{ToolDispatcher, ToolId};
use std::path::PathBuf;
use uuid::Uuid;

pub struct ToolSyncer {
    root: NormalizedPath,
    dry_run: bool,
}

impl ToolSyncer {
    pub fn new(root: NormalizedPath, dry_run: bool) -> Self {
        Self { root, dry_run }
    }

    /// Sync a tool, creating/updating its projections
    pub fn sync_tool(
        &self,
        tool_name: &str,
        ledger: &mut Ledger,
    ) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        let writer = ProjectionWriter::new(self.root.clone(), self.dry_run);

        // Get tool integration
        let tool_id = ToolId::from_str(tool_name)
            .ok_or_else(|| Error::Tool(format!("Unknown tool: {}", tool_name)))?;

        let dispatcher = ToolDispatcher::new(self.root.clone());
        let integration = dispatcher.get_integration(tool_id);

        // Get config files for this tool
        let config_files = integration.config_files();

        for config_file in config_files {
            let intent_id = format!("tool:{}", tool_name);

            // Check if intent already exists
            let existing = ledger.get_intents_by_id(&intent_id);
            if !existing.is_empty() {
                continue; // Already synced
            }

            // Create new intent
            let uuid = Uuid::new_v4();
            let projection = Projection {
                tool: tool_name.to_string(),
                file: PathBuf::from(&config_file.path),
                kind: ProjectionKind::FileManaged {
                    checksum: String::new(), // Will be computed after write
                },
            };

            // Generate content
            let content = integration.generate_config(&config_file)?;

            // Write projection
            let action = writer.apply(&projection, &content)?;
            actions.push(action);

            // Update intent with checksum
            let checksum = crate::projection::compute_checksum(&content);
            let intent = Intent {
                id: intent_id,
                uuid,
                timestamp: Utc::now(),
                args: serde_json::Value::Null,
                projections: vec![Projection {
                    tool: tool_name.to_string(),
                    file: PathBuf::from(&config_file.path),
                    kind: ProjectionKind::FileManaged { checksum },
                }],
            };

            if !self.dry_run {
                ledger.add_intent(intent);
            }
        }

        Ok(actions)
    }

    /// Remove a tool, deleting its projections
    pub fn remove_tool(
        &self,
        tool_name: &str,
        ledger: &mut Ledger,
    ) -> Result<Vec<String>> {
        let mut actions = Vec::new();
        let writer = ProjectionWriter::new(self.root.clone(), self.dry_run);

        let intent_id = format!("tool:{}", tool_name);
        let intents: Vec<_> = ledger
            .get_intents_by_id(&intent_id)
            .iter()
            .map(|i| i.uuid)
            .collect();

        for uuid in intents {
            if let Some(intent) = ledger.get_intent(uuid) {
                for projection in &intent.projections {
                    let action = writer.remove(projection)?;
                    actions.push(action);
                }
            }

            if !self.dry_run {
                ledger.remove_intent(uuid);
            }
        }

        Ok(actions)
    }
}
```

**Step 2: Export from mod.rs**

Add to `crates/repo-core/src/sync/mod.rs`:

```rust
mod tool_syncer;
pub use tool_syncer::ToolSyncer;
```

**Step 3: Run tests**

```bash
cargo test -p repo-core
```

**Step 4: Commit**

```bash
git add crates/repo-core/src/sync/
git commit -m "feat(repo-core): implement ToolSyncer

Handles syncing and removing tool configurations by
creating/deleting projections and updating the ledger.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task B.3: Complete SyncEngine Implementation

**Files:**
- Modify: `crates/repo-core/src/sync/engine.rs`

**Step 1: Update sync_with_options**

Replace the TODO stub with full implementation:

```rust
pub fn sync_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
    let mut ledger = self.load_ledger()?;
    let mut report = SyncReport::success();
    let tool_syncer = ToolSyncer::new(self.root.clone(), options.dry_run);

    // Load config to get active tools
    let config_path = self.backend.config_root().join("config.toml");
    if !config_path.exists() {
        return Ok(report.with_action("No config.toml found - nothing to sync".to_string()));
    }

    let config = repo_meta::RepositoryConfig::load(config_path.as_ref())?;

    // Sync each enabled tool
    for tool_name in &config.tools {
        match tool_syncer.sync_tool(tool_name, &mut ledger) {
            Ok(actions) => {
                for action in actions {
                    report = report.with_action(action);
                }
            }
            Err(e) => {
                report.errors.push(format!("Failed to sync {}: {}", tool_name, e));
            }
        }
    }

    // Remove tools that are no longer in config
    let active_tools: std::collections::HashSet<_> = config.tools.iter().collect();
    let ledger_tools: Vec<_> = ledger
        .intents()
        .iter()
        .filter(|i| i.id.starts_with("tool:"))
        .map(|i| i.id.strip_prefix("tool:").unwrap().to_string())
        .collect();

    for tool_name in ledger_tools {
        if !active_tools.contains(&tool_name) {
            match tool_syncer.remove_tool(&tool_name, &mut ledger) {
                Ok(actions) => {
                    for action in actions {
                        report = report.with_action(action);
                    }
                }
                Err(e) => {
                    report.errors.push(format!("Failed to remove {}: {}", tool_name, e));
                }
            }
        }
    }

    // Save ledger
    if !options.dry_run {
        self.save_ledger(&ledger)?;
    }

    report.success = report.errors.is_empty();
    Ok(report)
}
```

**Step 2: Update fix_with_options**

```rust
pub fn fix_with_options(&self, options: SyncOptions) -> Result<SyncReport> {
    // Check first to identify issues
    let check_report = self.check()?;

    let mut report = SyncReport::success();

    if check_report.status == CheckStatus::Healthy {
        return Ok(report.with_action("No fixes needed".to_string()));
    }

    // For drifted items, re-sync will overwrite with correct values
    // For missing items, re-sync will recreate them
    let sync_report = self.sync_with_options(options)?;

    report.actions = sync_report.actions;
    report.errors = sync_report.errors;
    report.success = sync_report.success;

    if !check_report.drifted.is_empty() {
        report = report.with_action(format!(
            "Fixed {} drifted projections",
            check_report.drifted.len()
        ));
    }

    if !check_report.missing.is_empty() {
        report = report.with_action(format!(
            "Recreated {} missing projections",
            check_report.missing.len()
        ));
    }

    Ok(report)
}
```

**Step 3: Run tests**

```bash
cargo test -p repo-core sync
```

**Step 4: Commit**

```bash
git add crates/repo-core/src/sync/
git commit -m "feat(repo-core): complete SyncEngine implementation

sync() now applies tool configurations from config.toml
fix() identifies drift and re-syncs to repair

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task B.4: Wire CLI Tool Commands to Sync

**Files:**
- Modify: `crates/repo-cli/src/commands/tool.rs`

**Step 1: Update add command**

After adding tool to config, trigger sync:

```rust
pub fn run_add_tool(name: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Validation warning
    let registry = repo_meta::ToolRegistry::with_builtins();
    if !registry.is_known(name) {
        eprintln!(
            "{}: '{}' is not a recognized tool",
            "Warning".yellow(),
            name
        );
    }

    // Load and update config
    let config_path = cwd.join(".repository/config.toml");
    let mut config = if config_path.exists() {
        repo_meta::RepositoryConfig::load(&config_path)?
    } else {
        return Err(anyhow::anyhow!("Not a repository - run 'repo init' first"));
    };

    if config.tools.contains(&name.to_string()) {
        println!("{} Tool '{}' is already enabled", "✓".green(), name);
        return Ok(());
    }

    config.tools.push(name.to_string());
    config.save(&config_path)?;

    // Trigger sync
    let root = repo_fs::NormalizedPath::new(&cwd)?;
    let mode = repo_core::Mode::from_config(&config);
    let engine = repo_core::SyncEngine::new(root, mode)?;
    let report = engine.sync()?;

    for action in &report.actions {
        println!("  {}", action);
    }

    println!("{} Added tool '{}'", "✓".green(), name);
    Ok(())
}
```

**Step 2: Update remove command similarly**

**Step 3: Run tests**

```bash
cargo test -p repo-cli
```

**Step 4: Commit**

```bash
git add crates/repo-cli/
git commit -m "feat(repo-cli): wire tool commands to trigger sync

Adding/removing tools now automatically syncs configurations.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task B.5: Implement Rule Commands

**Files:**
- Create: `crates/repo-cli/src/commands/rule.rs`
- Modify: `crates/repo-cli/src/cli.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`

**Step 1: Add rule commands to CLI**

In `cli.rs`, add:

```rust
/// Add a rule to the repository
AddRule {
    /// Rule ID (e.g., "python-style")
    id: String,
    /// Rule instruction text
    #[arg(short, long)]
    instruction: String,
    /// Tags for the rule
    #[arg(short, long)]
    tags: Vec<String>,
},

/// Remove a rule from the repository
RemoveRule {
    /// Rule ID to remove
    id: String,
},

/// List all active rules
ListRules,
```

**Step 2: Implement rule.rs**

```rust
// crates/repo-cli/src/commands/rule.rs

use anyhow::Result;
use colored::Colorize;
use repo_core::SyncEngine;
use repo_fs::NormalizedPath;
use repo_meta::Rule;
use std::fs;

pub fn run_add_rule(id: &str, instruction: &str, tags: Vec<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let rules_dir = cwd.join(".repository/rules");

    fs::create_dir_all(&rules_dir)?;

    let rule = Rule {
        id: id.to_string(),
        instruction: instruction.to_string(),
        tags,
        files: Vec::new(),
    };

    let rule_path = rules_dir.join(format!("{}.toml", id));
    let content = toml::to_string_pretty(&rule)?;
    fs::write(&rule_path, content)?;

    // Trigger sync to apply rule to tools
    let root = NormalizedPath::new(&cwd)?;
    let config = repo_meta::RepositoryConfig::load(cwd.join(".repository/config.toml"))?;
    let mode = repo_core::Mode::from_config(&config);
    let engine = SyncEngine::new(root, mode)?;
    let _report = engine.sync()?;

    println!("{} Added rule '{}'", "✓".green(), id);
    Ok(())
}

pub fn run_remove_rule(id: &str) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let rule_path = cwd.join(format!(".repository/rules/{}.toml", id));

    if !rule_path.exists() {
        return Err(anyhow::anyhow!("Rule '{}' not found", id));
    }

    fs::remove_file(&rule_path)?;

    // Trigger sync to remove rule from tools
    let root = NormalizedPath::new(&cwd)?;
    let config = repo_meta::RepositoryConfig::load(cwd.join(".repository/config.toml"))?;
    let mode = repo_core::Mode::from_config(&config);
    let engine = SyncEngine::new(root, mode)?;
    let _report = engine.sync()?;

    println!("{} Removed rule '{}'", "✓".green(), id);
    Ok(())
}

pub fn run_list_rules() -> Result<()> {
    let cwd = std::env::current_dir()?;
    let rules_dir = cwd.join(".repository/rules");

    if !rules_dir.exists() {
        println!("No rules defined");
        return Ok(());
    }

    for entry in fs::read_dir(rules_dir)? {
        let entry = entry?;
        if entry.path().extension().is_some_and(|e| e == "toml") {
            let content = fs::read_to_string(entry.path())?;
            let rule: Rule = toml::from_str(&content)?;
            println!("  {} - {}", rule.id.cyan(), rule.instruction);
        }
    }

    Ok(())
}
```

**Step 3: Export and wire up**

**Step 4: Run tests**

```bash
cargo test -p repo-cli
```

**Step 5: Commit**

```bash
git add crates/repo-cli/
git commit -m "feat(repo-cli): implement rule commands

Adds add-rule, remove-rule, list-rules commands with
automatic sync to apply rules to tool configurations.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task B.6-B.8: Additional Tasks

(Similar structure for remaining tasks):

- **B.6:** Implement RuleSyncer in repo-core
- **B.7:** Add integration tests for full sync workflow
- **B.8:** Add CLI integration tests

---

## Verification

After completing all tasks:

```bash
# Run full test suite
cargo test --workspace

# Manual verification
mkdir /tmp/test-repo && cd /tmp/test-repo
repo init --tools cursor vscode
repo check
repo add-rule python-style --instruction "Use snake_case" --tags python
repo sync
cat .cursorrules  # Should contain the rule
repo remove-rule python-style
cat .cursorrules  # Rule should be gone
```

---

## Summary

| Task | Description | Risk | Effort |
|------|-------------|------|--------|
| B.1 | ProjectionWriter | Medium | Medium |
| B.2 | ToolSyncer | Medium | Medium |
| B.3 | Complete SyncEngine | Medium | Medium |
| B.4 | Wire CLI tool commands | Low | Low |
| B.5 | Implement rule commands | Low | Medium |
| B.6 | RuleSyncer | Medium | Medium |
| B.7 | Integration tests | Low | Medium |
| B.8 | CLI integration tests | Low | Medium |

**Total Effort:** ~1-2 days of focused work
