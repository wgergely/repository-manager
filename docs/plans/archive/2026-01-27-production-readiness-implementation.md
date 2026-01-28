# Production Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform repository-manager from proof-of-concept to production-ready tool with verified consumer compatibility.

**Architecture:** Phase-based approach starting with research, then core infrastructure (UUID registry, native format support), then CLI overhaul, and finally integration testing against real tools.

**Tech Stack:** Rust, TOML/JSON/YAML parsers, git2, tokio async, assert_cmd for CLI testing

---

## Phase 0: Research (BLOCKING - Must Complete First)

### Task 0.1: Tool Config Format Research

**Goal:** Document exact config format for every supported tool. NO ASSUMPTIONS.

**Files:**
- Create: `docs/research/tool-config-formats.md`

**Step 1: Research Cursor**
- Official docs: https://cursor.sh/docs
- Web search: "cursor IDE config file format 2026"
- Document: File path, format, schema, hot-reload behavior

**Step 2: Research Claude Code**
- Official docs: Anthropic documentation
- Web search: "claude code config file CLAUDE.md format"
- Document: File path, format, expected structure

**Step 3: Research VSCode**
- Official docs: https://code.visualstudio.com/docs/getstarted/settings
- Document: settings.json schema, managed keys vs user keys

**Step 4: Research Windsurf**
- Official docs: Find Windsurf/Codeium documentation
- Web search: "windsurf IDE rules config 2026"
- Document: File path, format

**Step 5: Research Gemini Code Assist**
- Official docs: Google AI documentation
- Web search: "gemini code assist config file"
- Document: File path, format (if any)

**Step 6: Research JetBrains IDEs**
- Official docs: JetBrains documentation
- Document: .idea/ structure, which files to manage

**Step 7: Research Agentic Tools**
- Survey: Cline, Aider, Continue, Copilot Workspace, others
- Web search: "agentic coding tools config 2026"
- Document: Each tool's config system

**Step 8: Commit research document**
```bash
git add docs/research/tool-config-formats.md
git commit -m "docs: add tool config format research"
```

---

### Task 0.2: Verify Existing Block Format

**Goal:** Confirm `repo-blocks` UUID format matches architectural expectations.

**Files:**
- Read: `crates/repo-blocks/src/parser.rs`
- Read: `crates/repo-core/src/ledger/intent.rs`

**Step 1: Document current format**
Current format in `repo-blocks`:
```
<!-- repo:block:UUID -->
content
<!-- /repo:block:UUID -->
```

**Step 2: Document ledger Intent structure**
- Intent has UUID, id (rule name), args, projections
- Projection tracks tool + file + content hash

**Step 3: Verify alignment**
- Confirm block UUIDs link to Intent UUIDs
- Document any gaps

**Step 4: Write findings to research doc**
- Add section to `docs/research/tool-config-formats.md`

---

## Phase 1: Central Rule Registry with UUIDs

### Task 1.1: Design Rule Registry Schema

**Files:**
- Create: `docs/design/rule-registry-schema.md`

**Step 1: Define registry structure**
```toml
# .repository/rules/registry.toml
version = "1.0"

[[rules]]
uuid = "550e8400-e29b-41d4-a716-446655440000"
id = "python-style"
content = "Use snake_case for variables..."
created = "2026-01-27T10:00:00Z"
tags = ["python", "style"]

[[rules]]
uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
id = "api-design"
content = "Return JSON with {data, error, meta}..."
created = "2026-01-27T10:05:00Z"
tags = ["api", "http"]
```

**Step 2: Document sync flow**
- Rule created → UUID generated → stored in registry
- Sync reads registry → writes blocks to tool configs
- Block format: `<!-- repo:block:{uuid} -->` (already implemented!)

**Step 3: Commit design doc**

---

### Task 1.2: Implement Rule Registry

**Files:**
- Create: `crates/repo-core/src/rules/mod.rs`
- Create: `crates/repo-core/src/rules/registry.rs`
- Create: `crates/repo-core/src/rules/rule.rs`
- Modify: `crates/repo-core/src/lib.rs`
- Test: `crates/repo-core/tests/rules_tests.rs`

**Step 1: Write failing test**
```rust
// crates/repo-core/tests/rules_tests.rs
use repo_core::rules::{Rule, RuleRegistry};
use tempfile::TempDir;

#[test]
fn test_registry_add_rule_generates_uuid() {
    let temp = TempDir::new().unwrap();
    let mut registry = RuleRegistry::new(temp.path().join("registry.toml"));

    let rule = registry.add_rule("python-style", "Use snake_case...", vec!["python"]).unwrap();

    assert!(!rule.uuid.is_nil());
    assert_eq!(rule.id, "python-style");
}
```

**Step 2: Run test to verify it fails**
```bash
cargo test -p repo-core test_registry_add_rule_generates_uuid
```
Expected: FAIL - module not found

**Step 3: Implement Rule struct**
```rust
// crates/repo-core/src/rules/rule.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub uuid: Uuid,
    pub id: String,
    pub content: String,
    pub created: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl Rule {
    pub fn new(id: impl Into<String>, content: impl Into<String>, tags: Vec<String>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            id: id.into(),
            content: content.into(),
            created: Utc::now(),
            tags,
        }
    }
}
```

**Step 4: Implement RuleRegistry**
```rust
// crates/repo-core/src/rules/registry.rs
use super::rule::Rule;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RuleRegistry {
    version: String,
    rules: Vec<Rule>,
    #[serde(skip)]
    path: PathBuf,
}

impl RuleRegistry {
    pub fn new(path: PathBuf) -> Self {
        Self {
            version: "1.0".to_string(),
            rules: Vec::new(),
            path,
        }
    }

    pub fn load(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let mut registry: Self = toml::from_str(&content)?;
        registry.path = path;
        Ok(registry)
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn add_rule(&mut self, id: &str, content: &str, tags: Vec<String>) -> Result<&Rule> {
        let rule = Rule::new(id, content, tags);
        self.rules.push(rule);
        self.save()?;
        Ok(self.rules.last().unwrap())
    }

    pub fn get_rule(&self, uuid: Uuid) -> Option<&Rule> {
        self.rules.iter().find(|r| r.uuid == uuid)
    }

    pub fn get_rule_by_id(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }

    pub fn remove_rule(&mut self, uuid: Uuid) -> Option<Rule> {
        let pos = self.rules.iter().position(|r| r.uuid == uuid)?;
        let rule = self.rules.remove(pos);
        self.save().ok()?;
        Some(rule)
    }

    pub fn all_rules(&self) -> &[Rule] {
        &self.rules
    }
}
```

**Step 5: Add module to lib.rs**
```rust
// crates/repo-core/src/lib.rs
pub mod rules;
pub use rules::{Rule, RuleRegistry};
```

**Step 6: Run test to verify it passes**
```bash
cargo test -p repo-core test_registry_add_rule_generates_uuid
```
Expected: PASS

**Step 7: Commit**
```bash
git add crates/repo-core/src/rules/
git commit -m "feat(repo-core): add UUID-based rule registry"
```

---

### Task 1.3: Wire Rule Registry to SyncEngine

**Files:**
- Modify: `crates/repo-core/src/sync/engine.rs`
- Test: `crates/repo-core/tests/sync_tests.rs`

**Step 1: Write failing test**
```rust
#[test]
fn test_sync_uses_rule_registry_uuids() {
    let temp = TempDir::new().unwrap();
    // Setup registry with a rule
    let mut registry = RuleRegistry::new(temp.path().join(".repository/rules/registry.toml"));
    let rule = registry.add_rule("test-rule", "Test content", vec![]).unwrap();
    let uuid = rule.uuid;

    // Setup config with cursor tool
    // ... setup code ...

    // Run sync
    let engine = SyncEngine::new(root, Mode::Standard).unwrap();
    engine.sync().unwrap();

    // Verify .cursorrules contains block with rule UUID
    let content = std::fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
    assert!(content.contains(&format!("<!-- repo:block:{} -->", uuid)));
}
```

**Step 2: Implement integration** (details depend on current SyncEngine structure)

**Step 3: Run tests, verify pass**

**Step 4: Commit**

---

## Phase 2: Native Format Support for Managed Sections

### Task 2.1: Design Multi-Format Block System

**Files:**
- Create: `docs/design/multi-format-blocks.md`

**Step 1: Document format-specific strategies**

| Format | Managed Section Approach |
|--------|-------------------------|
| Markdown | `<!-- repo:block:UUID -->` (existing) |
| JSON | `"__repo_managed__": { "UUID": { ... } }` |
| YAML | `# repo:block:UUID` + `# /repo:block:UUID` |
| TOML | `# repo:block:UUID` + `[repo_managed.UUID]` |
| XML | `<!-- repo:block:UUID -->` |
| JavaScript | `/* repo:block:UUID */` |
| Plain text | `# repo:block:UUID` |

**Step 2: Define trait interface**
```rust
pub trait FormatHandler {
    fn parse_blocks(&self, content: &str) -> Vec<Block>;
    fn write_block(&self, content: &str, uuid: Uuid, block_content: &str) -> String;
    fn remove_block(&self, content: &str, uuid: Uuid) -> String;
}
```

**Step 3: Commit design doc**

---

### Task 2.2: Implement JSON Format Handler

**Files:**
- Create: `crates/repo-blocks/src/formats/mod.rs`
- Create: `crates/repo-blocks/src/formats/json.rs`
- Test: `crates/repo-blocks/tests/json_format_tests.rs`

**Step 1: Write failing test**
```rust
#[test]
fn test_json_merge_preserves_user_keys() {
    let existing = r#"{
        "editor.formatOnSave": true,
        "python.linting.enabled": true
    }"#;

    let handler = JsonFormatHandler::new();
    let result = handler.write_block(
        existing,
        Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        r#"{"python.defaultInterpreterPath": ".venv/bin/python"}"#,
    );

    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // User keys preserved
    assert_eq!(parsed["editor.formatOnSave"], true);
    assert_eq!(parsed["python.linting.enabled"], true);
    // Managed key added
    assert!(parsed["__repo_managed__"]["550e8400-e29b-41d4-a716-446655440000"].is_object());
}
```

**Step 2-6: Implement and test** (TDD cycle)

**Step 7: Commit**

---

### Task 2.3: Implement YAML Format Handler

Similar TDD approach for YAML with comment-based blocks.

---

### Task 2.4: Implement TOML Format Handler

Similar TDD approach for TOML.

---

## Phase 3: CLI Overhaul

### Task 3.1: Implement `repo init <project-name>`

**Files:**
- Modify: `crates/repo-cli/src/commands/init.rs`
- Modify: `crates/repo-cli/src/cli.rs`
- Test: `crates/repo-cli/tests/integration_tests.rs`

**Step 1: Write failing test**
```rust
#[test]
fn test_init_creates_project_folder() {
    let parent = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(parent.path())
        .args(["init", "my-project", "--mode", "standard"])
        .assert()
        .success();

    // Verify folder created with sanitized name
    assert!(parent.path().join("my-project").exists());
    assert!(parent.path().join("my-project/.repository").exists());
}

#[test]
fn test_init_sanitizes_project_name() {
    let parent = tempdir().unwrap();

    let mut cmd = repo_cmd();
    cmd.current_dir(parent.path())
        .args(["init", "My Project Name!", "--mode", "standard"])
        .assert()
        .success();

    // Should create sanitized folder name
    assert!(parent.path().join("my-project-name").exists());
}
```

**Step 2: Update CLI argument parsing**
```rust
// crates/repo-cli/src/cli.rs
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new repository
    Init {
        /// Project name (creates folder)
        #[arg(default_value = ".")]
        name: String,

        /// Repository mode
        #[arg(short, long, default_value = "worktrees")]
        mode: String,

        /// Tools to configure
        #[arg(short, long)]
        tools: Vec<String>,

        /// Presets to configure
        #[arg(short, long)]
        presets: Vec<String>,

        /// Remote repository URL
        #[arg(short, long)]
        remote: Option<String>,

        /// Interactive mode
        #[arg(short, long)]
        interactive: bool,
    },
    // ...
}
```

**Step 3: Implement folder creation logic**

**Step 4: Implement name sanitization**

**Step 5: Run tests, verify pass**

**Step 6: Commit**

---

### Task 3.2: Implement Interactive Init Mode

**Files:**
- Create: `crates/repo-cli/src/interactive.rs`
- Modify: `crates/repo-cli/src/commands/init.rs`

**Step 1: Add dialoguer dependency**
```toml
# crates/repo-cli/Cargo.toml
[dependencies]
dialoguer = "0.11"
```

**Step 2: Implement interactive prompts**
```rust
// crates/repo-cli/src/interactive.rs
use dialoguer::{Select, MultiSelect, Input, Confirm};

pub fn interactive_init() -> Result<InitConfig> {
    let name: String = Input::new()
        .with_prompt("Project name")
        .interact_text()?;

    let mode = Select::new()
        .with_prompt("Repository mode")
        .items(&["standard", "worktrees", "in-repo-worktrees"])
        .default(1)
        .interact()?;

    let tools = MultiSelect::new()
        .with_prompt("Select tools (space to toggle)")
        .items(&["vscode", "cursor", "claude", "windsurf", "gemini"])
        .interact()?;

    // ... more prompts ...

    Ok(InitConfig { name, mode, tools, ... })
}
```

**Step 3-6: TDD cycle**

**Step 7: Commit**

---

### Task 3.3: Implement Consistent Git-Like CLI Syntax

**Files:**
- Modify: `crates/repo-cli/src/cli.rs`
- Modify: `crates/repo-cli/src/commands/mod.rs`
- Create: `crates/repo-cli/src/context.rs`

**Step 1: Write failing tests**
```rust
#[test]
fn test_merge_from_container_root() {
    // repo merge <source> [target]
    // from container root should work
}

#[test]
fn test_merge_from_worktree() {
    // repo merge <source> [target]
    // from inside worktree should work
}

#[test]
fn test_merge_default_target_is_main() {
    // repo merge feature
    // should merge feature into main
}
```

**Step 2: Implement context detection**
```rust
// crates/repo-cli/src/context.rs
pub enum RepoContext {
    ContainerRoot { path: PathBuf },
    Worktree { container: PathBuf, worktree: String },
    StandardRepo { path: PathBuf },
    NotARepo,
}

pub fn detect_context(cwd: &Path) -> RepoContext {
    // Check for .gt (container mode)
    // Check for .repository
    // Check for .git
    // Walk up to find container root
}
```

**Step 3-6: TDD cycle**

**Step 7: Commit**

---

## Phase 4: Worktree Architecture

### Task 4.1: Implement Container-Root Config Storage

**Files:**
- Modify: `crates/repo-git/src/container.rs`
- Test: `crates/repo-git/tests/container_tests.rs`

**Step 1: Write failing test**
```rust
#[test]
fn test_container_stores_configs_at_root() {
    let (temp, layout) = setup_container();

    // Tool configs should be at container root, not in worktrees
    assert!(temp.path().join(".vscode").exists());
    assert!(!temp.path().join("main/.vscode").exists());
}
```

**Step 2-6: Implement**

**Step 7: Commit**

---

### Task 4.2: Implement Tagged Venv System

**Files:**
- Modify: `crates/repo-presets/src/python/venv.rs`
- Create: `crates/repo-presets/src/python/tagged_venv.rs`
- Test: `crates/repo-presets/tests/tagged_venv_tests.rs`

**Step 1: Write failing test**
```rust
#[test]
fn test_venv_creates_tagged_environment() {
    let temp = TempDir::new().unwrap();
    let provider = VenvProvider::new();

    // Create tagged venv
    provider.create_tagged(temp.path(), "main-win-py311").unwrap();

    assert!(temp.path().join(".venv-main-win-py311").exists());
}

#[test]
fn test_venv_actually_runs_python() {
    let temp = TempDir::new().unwrap();
    let provider = VenvProvider::new();

    provider.create_tagged(temp.path(), "test").unwrap();

    // Verify python binary exists
    #[cfg(windows)]
    let python = temp.path().join(".venv-test/Scripts/python.exe");
    #[cfg(not(windows))]
    let python = temp.path().join(".venv-test/bin/python");

    assert!(python.exists(), "python binary should exist");
}
```

**Step 2-6: Implement with actual `python -m venv` execution**

**Step 7: Commit**

---

## Phase 5: Tool Backup/Restore System

### Task 5.1: Implement Tool Config Backup

**Files:**
- Create: `crates/repo-core/src/backup/mod.rs`
- Create: `crates/repo-core/src/backup/tool_backup.rs`
- Test: `crates/repo-core/tests/backup_tests.rs`

**Step 1: Write failing test**
```rust
#[test]
fn test_remove_tool_creates_backup() {
    let temp = TempDir::new().unwrap();
    setup_repo_with_tool(&temp, "cursor");

    // Remove tool
    run_remove_tool(temp.path(), "cursor").unwrap();

    // Verify backup exists
    let backup_dir = temp.path().join(".repository/backups/cursor");
    assert!(backup_dir.exists());
    assert!(backup_dir.join("cursorrules.backup").exists());
    assert!(backup_dir.join("metadata.toml").exists());
}

#[test]
fn test_add_tool_offers_restore() {
    let temp = TempDir::new().unwrap();
    setup_repo_with_tool(&temp, "cursor");
    run_remove_tool(temp.path(), "cursor").unwrap();

    // Re-add tool - should detect backup
    // (In interactive mode, would prompt to restore)
    let result = run_add_tool_with_restore(temp.path(), "cursor", true);

    // Verify restored content
    let content = std::fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
    assert!(content.contains("original content"));
}
```

**Step 2-6: Implement**

**Step 7: Commit**

---

## Phase 6: End-to-End Integration Tests

### Task 6.1: Full Workflow Test - Init to Sync

**Files:**
- Modify: `tests/integration/src/mission_tests.rs`

**Step 1: Write comprehensive E2E test**
```rust
#[test]
fn test_e2e_init_add_tool_add_rule_sync_verify() {
    let parent = tempdir().unwrap();

    // 1. Init project
    repo_cmd()
        .current_dir(parent.path())
        .args(["init", "test-project", "--mode", "standard", "--tools", "cursor"])
        .assert()
        .success();

    let project = parent.path().join("test-project");

    // 2. Add a rule
    repo_cmd()
        .current_dir(&project)
        .args(["add-rule", "test-rule", "--instruction", "Test instruction"])
        .assert()
        .success();

    // 3. Sync
    repo_cmd()
        .current_dir(&project)
        .arg("sync")
        .assert()
        .success();

    // 4. Verify .cursorrules exists with rule content
    let cursorrules = project.join(".cursorrules");
    assert!(cursorrules.exists());

    let content = std::fs::read_to_string(&cursorrules).unwrap();
    assert!(content.contains("Test instruction"));

    // 5. Verify rule uses UUID block format
    assert!(content.contains("<!-- repo:block:"));

    // 6. Verify registry has the rule
    let registry_path = project.join(".repository/rules/registry.toml");
    assert!(registry_path.exists());
    let registry_content = std::fs::read_to_string(&registry_path).unwrap();
    assert!(registry_content.contains("test-rule"));
}
```

**Step 2: Run test, fix issues until passing**

**Step 3: Commit**

---

### Task 6.2: Consumer Verification Tests (Per Research)

**Depends on:** Task 0.1 research findings

**Files:**
- Create: `tests/consumer_verification/` (new test crate)

**Step 1: Based on research, write tests that verify output format**

Example (if Cursor uses specific JSON schema):
```rust
#[test]
fn test_cursorrules_valid_format() {
    // Create .cursorrules via repo sync
    // Validate against Cursor's expected format
    // (Format determined by Task 0.1 research)
}
```

---

### Task 6.3: Concurrent Edit Preservation Test

**Files:**
- Modify: `tests/integration/src/mission_tests.rs`

**Step 1: Write test**
```rust
#[test]
fn test_sync_preserves_user_content_in_cursorrules() {
    let temp = setup_repo_with_cursor();

    // Simulate user adding content via Cursor UI
    let cursorrules = temp.path().join(".cursorrules");
    let mut content = std::fs::read_to_string(&cursorrules).unwrap();
    content = format!("# User's custom header\n\n{}", content);
    std::fs::write(&cursorrules, content).unwrap();

    // Add new rule and sync
    repo_cmd()
        .current_dir(temp.path())
        .args(["add-rule", "new-rule", "--instruction", "New instruction"])
        .assert()
        .success();

    repo_cmd()
        .current_dir(temp.path())
        .arg("sync")
        .assert()
        .success();

    // Verify user content preserved
    let final_content = std::fs::read_to_string(&cursorrules).unwrap();
    assert!(final_content.contains("# User's custom header"));
    assert!(final_content.contains("New instruction"));
}
```

**Step 2: Run test, fix sync logic if needed**

**Step 3: Commit**

---

## Phase 7: Update Mission Tests for Closed Gaps

### Task 7.1: Replace Panic Placeholders with Real Tests

**Files:**
- Modify: `tests/integration/src/mission_tests.rs`

**Step 1: Update GAP-006 (Antigravity)**
```rust
// BEFORE:
#[test]
#[ignore = "GAP-006: Antigravity tool not implemented"]
fn gap_006_antigravity_tool() {
    panic!("Antigravity tool integration not implemented");
}

// AFTER:
#[test]
fn test_antigravity_tool_integration() {
    let temp = setup_repo_with_tool("antigravity");

    // Add rule and sync
    add_rule(&temp, "test-rule", "Test content");
    run_sync(&temp);

    // Verify antigravity config created (path from research)
    let config_path = temp.path().join(".agent/rules.md"); // Adjust per research
    assert!(config_path.exists());

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("Test content"));
}
```

**Step 2: Repeat for GAP-007, GAP-008, GAP-010, GAP-018**

**Step 3: Run all tests**
```bash
cargo test --test mission_tests
```

**Step 4: Commit**
```bash
git add tests/integration/src/mission_tests.rs
git commit -m "test: replace gap placeholders with real integration tests"
```

---

## Execution Order

1. **Phase 0** (Research) - MUST complete first, blocks all other phases
2. **Phase 1** (Rule Registry) - Core infrastructure
3. **Phase 2** (Format Support) - Can parallel with Phase 1 after Task 1.2
4. **Phase 3** (CLI) - After Phase 1
5. **Phase 4** (Worktree) - After Phase 1
6. **Phase 5** (Backup) - After Phase 3
7. **Phase 6** (E2E Tests) - After Phases 1-5
8. **Phase 7** (Mission Tests) - After Phase 6

---

## Success Criteria

- [ ] All tool config formats verified via research
- [ ] UUID-based rule registry working
- [ ] `repo init <name>` creates folder
- [ ] Interactive init mode available
- [ ] JSON/YAML/TOML managed sections working
- [ ] CLI works from any directory (container root or worktree)
- [ ] Tagged venvs working (`.venv-{tag}`)
- [ ] Tool backup/restore on remove/add
- [ ] All mission tests passing (no ignored gaps for implemented features)
- [ ] E2E workflow test passing
- [ ] Consumer verification tests passing (per research findings)
