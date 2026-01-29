# Unified Registry Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a complete capability-driven tool management system with: (1) unified tool registry as single source of truth, (2) capability-based translation layer, (3) semantic-aware config writers.

**Architecture:** Four-layer system that replaces ad-hoc tool registration and block injection with a proper registry → translator → writer pipeline.

**Tech Stack:** Rust, serde, serde_json, toml

---

## Consolidated Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│              LAYER 1: TOOL REGISTRY (repo-tools/registry)            │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ ToolRegistry                                                 │    │
│  │ ├── builtins: HashMap<slug, ToolRegistration>  ← 13 tools   │    │
│  │ └── schema_tools: HashMap<slug, ToolRegistration> ← TOML    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
│  builtin_registrations() - SINGLE SOURCE OF TRUTH (replaces 3x dup) │
│  ToolRegistration { slug, name, category, priority, definition }     │
│  ToolCategory { Ide, CliAgent, Autonomous, Copilot }                 │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ registry.get_tools_for_sync()
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│            LAYER 2: RULE REGISTRY (repo-meta - existing)             │
│                                                                      │
│  DefinitionLoader.load_rules() → HashMap<id, RuleDefinition>         │
│  RuleDefinition { meta, content, examples, targets }                 │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ (tools, rules)
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│          LAYER 3: CAPABILITY TRANSLATOR (repo-tools/translator)      │
│                                                                      │
│  CapabilityTranslator.translate(tool, rules) → TranslatedContent     │
│                                                                      │
│  Checks tool.capabilities:                                           │
│    supports_custom_instructions → RuleTranslator                     │
│    supports_mcp                 → MCPTranslator [Phase 4]            │
│    supports_rules_directory     → DirectoryTranslator [Phase 2]      │
│                                                                      │
│  TranslatedContent { format, instructions, mcp_servers, data }       │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ TranslatedContent
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│             LAYER 4: CONFIG WRITERS (repo-tools/writer)              │
│                                                                      │
│  WriterRegistry.get_writer(config_type) → &dyn ConfigWriter          │
│                                                                      │
│  ┌─────────────┐  ┌─────────────────┐  ┌─────────────┐              │
│  │ JsonWriter  │  │ MarkdownWriter  │  │ TextWriter  │              │
│  │ (merge)     │  │ (sections)      │  │ (replace)   │              │
│  └─────────────┘  └─────────────────┘  └─────────────┘              │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ write to file
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      TOOL CONFIG FILES                               │
│                                                                      │
│  .cursorrules, CLAUDE.md, .vscode/settings.json, etc.               │
└─────────────────────────────────────────────────────────────────────┘
```

---

## What This Replaces

| Current (Ad-hoc) | New (Unified) |
|------------------|---------------|
| Tool names in 3 places (dispatcher.rs) | `builtin_registrations()` single source |
| `ToolCapabilities` ignored | `CapabilityTranslator` checks capabilities |
| `<!-- repo:block -->` injection | `MarkdownWriter` section-based merge |
| Per-tool sync logic | `ToolSyncer` unified pipeline |
| No validation | `ToolRegistry.validate()` conflict detection |

---

## Phased Implementation

| Phase | Focus | Tasks |
|-------|-------|-------|
| **1** | Tool Registry Foundation | 1.1 - 1.5 |
| **2** | Capability Translator | 2.1 - 2.3 |
| **3** | Config Writers | 3.1 - 3.4 |
| **4** | Integration & Migration | 4.1 - 4.3 |

---

## Phase 1: Tool Registry Foundation

**Goal:** Create unified tool registry with single source of truth.

### Task 1.1: Create registry types (ToolCategory, ToolRegistration)

**Files:**
- Create: `crates/repo-tools/src/registry/mod.rs`
- Create: `crates/repo-tools/src/registry/types.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create types.rs with ToolCategory**

Create `crates/repo-tools/src/registry/types.rs`:
```rust
//! Core types for the unified tool registry

use repo_meta::schema::ToolDefinition;
use serde::{Deserialize, Serialize};

/// Tool category for filtering and organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    /// IDE integrations (VSCode, Cursor, Zed, JetBrains, Windsurf, Antigravity)
    Ide,
    /// CLI-based agents (Claude Code, Aider, Gemini CLI)
    CliAgent,
    /// Autonomous coding agents (Cline, Roo)
    Autonomous,
    /// Code completion tools (GitHub Copilot, Amazon Q)
    Copilot,
}

/// Complete tool registration with metadata
#[derive(Debug, Clone)]
pub struct ToolRegistration {
    /// Unique identifier (e.g., "cursor", "claude")
    pub slug: String,
    /// Human-readable name (e.g., "Cursor", "Claude Code")
    pub name: String,
    /// Tool category for filtering
    pub category: ToolCategory,
    /// Sync priority (lower = syncs first, default: 50)
    pub priority: u8,
    /// The underlying tool definition with capabilities and schema
    pub definition: ToolDefinition,
}

impl ToolRegistration {
    /// Create a new registration with default priority
    pub fn new(
        slug: impl Into<String>,
        name: impl Into<String>,
        category: ToolCategory,
        definition: ToolDefinition,
    ) -> Self {
        Self {
            slug: slug.into(),
            name: name.into(),
            category,
            priority: 50,
            definition,
        }
    }

    /// Set sync priority (builder pattern)
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Check if tool supports custom instructions
    pub fn supports_instructions(&self) -> bool {
        self.definition.capabilities.supports_custom_instructions
    }

    /// Check if tool supports MCP
    pub fn supports_mcp(&self) -> bool {
        self.definition.capabilities.supports_mcp
    }

    /// Check if tool supports rules directory
    pub fn supports_rules_directory(&self) -> bool {
        self.definition.capabilities.supports_rules_directory
    }

    /// Check if tool has any translatable capability
    pub fn has_any_capability(&self) -> bool {
        self.supports_instructions()
            || self.supports_mcp()
            || self.supports_rules_directory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{ConfigType, ToolCapabilities, ToolIntegrationConfig, ToolMeta};

    fn make_definition(supports_instructions: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Test".into(),
                slug: "test".into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".test".into(),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: supports_instructions,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    #[test]
    fn test_category_serialization() {
        assert_eq!(
            serde_json::to_string(&ToolCategory::Ide).unwrap(),
            "\"ide\""
        );
        assert_eq!(
            serde_json::to_string(&ToolCategory::CliAgent).unwrap(),
            "\"cli_agent\""
        );
    }

    #[test]
    fn test_registration_new() {
        let def = make_definition(true);
        let reg = ToolRegistration::new("test", "Test Tool", ToolCategory::Ide, def);

        assert_eq!(reg.slug, "test");
        assert_eq!(reg.name, "Test Tool");
        assert_eq!(reg.category, ToolCategory::Ide);
        assert_eq!(reg.priority, 50);
    }

    #[test]
    fn test_registration_with_priority() {
        let def = make_definition(true);
        let reg = ToolRegistration::new("test", "Test", ToolCategory::Ide, def)
            .with_priority(10);

        assert_eq!(reg.priority, 10);
    }

    #[test]
    fn test_capability_checks() {
        let def_with = make_definition(true);
        let def_without = make_definition(false);

        let reg_with = ToolRegistration::new("a", "A", ToolCategory::Ide, def_with);
        let reg_without = ToolRegistration::new("b", "B", ToolCategory::Ide, def_without);

        assert!(reg_with.supports_instructions());
        assert!(reg_with.has_any_capability());

        assert!(!reg_without.supports_instructions());
        assert!(!reg_without.has_any_capability());
    }
}
```

**Step 2: Create mod.rs**

Create `crates/repo-tools/src/registry/mod.rs`:
```rust
//! Unified Tool Registry
//!
//! This module provides a single source of truth for all tool registrations,
//! eliminating the 3-location duplication problem in the old ToolDispatcher.
//!
//! # Architecture
//!
//! ```text
//! ToolRegistry
//! ├── builtins: 13 built-in tools (cursor, claude, vscode, etc.)
//! └── schema_tools: User-defined tools from .repository/tools/*.toml
//! ```

mod types;

pub use types::{ToolCategory, ToolRegistration};
```

**Step 3: Export from lib.rs**

Add to `crates/repo-tools/src/lib.rs`:
```rust
pub mod registry;
pub use registry::{ToolCategory, ToolRegistration};
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools registry::types`
Expected: 4 tests pass

**Step 5: Commit**

```bash
git add crates/repo-tools/src/registry/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add ToolCategory and ToolRegistration types"
```

---

### Task 1.2: Create ToolRegistry struct

**Files:**
- Create: `crates/repo-tools/src/registry/store.rs`
- Modify: `crates/repo-tools/src/registry/mod.rs`

**Step 1: Create store.rs**

Create `crates/repo-tools/src/registry/store.rs`:
```rust
//! Tool registry storage and operations

use super::types::{ToolCategory, ToolRegistration};
use std::collections::HashMap;

/// Unified registry for all tool registrations
///
/// This is the SINGLE SOURCE OF TRUTH for tools, replacing the
/// 3-location duplication in the old ToolDispatcher.
pub struct ToolRegistry {
    /// All registered tools by slug
    tools: HashMap<String, ToolRegistration>,
}

impl ToolRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, registration: ToolRegistration) {
        self.tools.insert(registration.slug.clone(), registration);
    }

    /// Get a tool by slug
    pub fn get(&self, slug: &str) -> Option<&ToolRegistration> {
        self.tools.get(slug)
    }

    /// Check if a tool is registered
    pub fn contains(&self, slug: &str) -> bool {
        self.tools.contains_key(slug)
    }

    /// Get number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// List all tool slugs (sorted)
    pub fn list(&self) -> Vec<&str> {
        let mut slugs: Vec<&str> = self.tools.keys().map(|s| s.as_str()).collect();
        slugs.sort();
        slugs
    }

    /// List tools by category (sorted)
    pub fn by_category(&self, category: ToolCategory) -> Vec<&str> {
        let mut slugs: Vec<&str> = self.tools
            .iter()
            .filter(|(_, reg)| reg.category == category)
            .map(|(slug, _)| slug.as_str())
            .collect();
        slugs.sort();
        slugs
    }

    /// List tools with any translatable capability
    pub fn with_capabilities(&self) -> Vec<&ToolRegistration> {
        self.tools
            .values()
            .filter(|reg| reg.has_any_capability())
            .collect()
    }

    /// Get tools sorted by priority (for sync ordering)
    pub fn by_priority(&self) -> Vec<&ToolRegistration> {
        let mut tools: Vec<_> = self.tools.values().collect();
        tools.sort_by_key(|t| t.priority);
        tools
    }

    /// Iterate over all registrations
    pub fn iter(&self) -> impl Iterator<Item = &ToolRegistration> {
        self.tools.values()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

    fn make_registration(slug: &str, category: ToolCategory, priority: u8, supports: bool) -> ToolRegistration {
        let def = ToolDefinition {
            meta: ToolMeta {
                name: slug.into(),
                slug: slug.into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: format!(".{}", slug),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: supports,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        };
        ToolRegistration::new(slug, slug, category, def).with_priority(priority)
    }

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(make_registration("test", ToolCategory::Ide, 50, true));

        assert_eq!(registry.len(), 1);
        assert!(registry.contains("test"));
        assert!(registry.get("test").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_list_sorted() {
        let mut registry = ToolRegistry::new();
        registry.register(make_registration("zzz", ToolCategory::Ide, 50, true));
        registry.register(make_registration("aaa", ToolCategory::Ide, 50, true));

        let list = registry.list();
        assert_eq!(list, vec!["aaa", "zzz"]);
    }

    #[test]
    fn test_by_category() {
        let mut registry = ToolRegistry::new();
        registry.register(make_registration("cursor", ToolCategory::Ide, 50, true));
        registry.register(make_registration("claude", ToolCategory::CliAgent, 50, true));
        registry.register(make_registration("vscode", ToolCategory::Ide, 50, true));

        let ide_tools = registry.by_category(ToolCategory::Ide);
        assert_eq!(ide_tools.len(), 2);
        assert!(ide_tools.contains(&"cursor"));
        assert!(ide_tools.contains(&"vscode"));
        assert!(!ide_tools.contains(&"claude"));
    }

    #[test]
    fn test_by_priority() {
        let mut registry = ToolRegistry::new();
        registry.register(make_registration("low", ToolCategory::Ide, 90, true));
        registry.register(make_registration("high", ToolCategory::Ide, 10, true));
        registry.register(make_registration("mid", ToolCategory::Ide, 50, true));

        let ordered = registry.by_priority();
        assert_eq!(ordered[0].slug, "high");
        assert_eq!(ordered[1].slug, "mid");
        assert_eq!(ordered[2].slug, "low");
    }

    #[test]
    fn test_with_capabilities() {
        let mut registry = ToolRegistry::new();
        registry.register(make_registration("capable", ToolCategory::Ide, 50, true));
        registry.register(make_registration("incapable", ToolCategory::Ide, 50, false));

        let capable = registry.with_capabilities();
        assert_eq!(capable.len(), 1);
        assert_eq!(capable[0].slug, "capable");
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/registry/mod.rs`:
```rust
//! Unified Tool Registry

mod store;
mod types;

pub use store::ToolRegistry;
pub use types::{ToolCategory, ToolRegistration};
```

**Step 3: Update lib.rs exports**

Update `crates/repo-tools/src/lib.rs`:
```rust
pub use registry::{ToolCategory, ToolRegistration, ToolRegistry};
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools registry`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/repo-tools/src/registry/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add ToolRegistry with category and priority support"
```

---

### Task 1.3: Add definition() method to GenericToolIntegration

**Files:**
- Modify: `crates/repo-tools/src/generic.rs`

**Step 1: Add definition() method**

Add to `GenericToolIntegration` impl in `crates/repo-tools/src/generic.rs`:
```rust
    /// Get a reference to the underlying tool definition
    pub fn definition(&self) -> &ToolDefinition {
        &self.definition
    }
```

**Step 2: Run tests**

Run: `cargo test -p repo-tools generic`
Expected: All existing tests pass

**Step 3: Commit**

```bash
git add crates/repo-tools/src/generic.rs
git commit -m "feat(repo-tools): add definition() accessor to GenericToolIntegration"
```

---

### Task 1.4: Create builtin_registrations() single source of truth

**Files:**
- Create: `crates/repo-tools/src/registry/builtins.rs`
- Modify: `crates/repo-tools/src/registry/mod.rs`

**Step 1: Create builtins.rs**

Create `crates/repo-tools/src/registry/builtins.rs`:
```rust
//! Built-in tool registrations - SINGLE SOURCE OF TRUTH
//!
//! All 13 built-in tools are defined HERE and NOWHERE ELSE.
//! This eliminates the 3-location duplication in the old ToolDispatcher.

use super::types::{ToolCategory, ToolRegistration};

// Import all tool factory functions
use crate::aider::aider_integration;
use crate::amazonq::amazonq_integration;
use crate::antigravity::antigravity_integration;
use crate::claude::claude_integration;
use crate::cline::cline_integration;
use crate::copilot::copilot_integration;
use crate::cursor::cursor_integration;
use crate::gemini::gemini_integration;
use crate::jetbrains::jetbrains_integration;
use crate::roo::roo_integration;
use crate::vscode::VSCodeIntegration;
use crate::windsurf::windsurf_integration;
use crate::zed::zed_integration;

/// Returns all built-in tool registrations.
///
/// THIS IS THE SINGLE SOURCE OF TRUTH.
/// Adding a new built-in tool? Add it here and nowhere else.
pub fn builtin_registrations() -> Vec<ToolRegistration> {
    vec![
        // === IDE Integrations ===
        ToolRegistration::new(
            "vscode",
            "VS Code",
            ToolCategory::Ide,
            VSCodeIntegration::new().definition().clone(),
        ),
        ToolRegistration::new(
            "cursor",
            "Cursor",
            ToolCategory::Ide,
            cursor_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "zed",
            "Zed",
            ToolCategory::Ide,
            zed_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "jetbrains",
            "JetBrains",
            ToolCategory::Ide,
            jetbrains_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "windsurf",
            "Windsurf",
            ToolCategory::Ide,
            windsurf_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "antigravity",
            "Antigravity",
            ToolCategory::Ide,
            antigravity_integration().definition().clone(),
        ),

        // === CLI Agents ===
        ToolRegistration::new(
            "claude",
            "Claude Code",
            ToolCategory::CliAgent,
            claude_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "aider",
            "Aider",
            ToolCategory::CliAgent,
            aider_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "gemini",
            "Gemini CLI",
            ToolCategory::CliAgent,
            gemini_integration().definition().clone(),
        ),

        // === Autonomous Agents ===
        ToolRegistration::new(
            "cline",
            "Cline",
            ToolCategory::Autonomous,
            cline_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "roo",
            "Roo",
            ToolCategory::Autonomous,
            roo_integration().definition().clone(),
        ),

        // === Copilots ===
        ToolRegistration::new(
            "copilot",
            "GitHub Copilot",
            ToolCategory::Copilot,
            copilot_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "amazonq",
            "Amazon Q",
            ToolCategory::Copilot,
            amazonq_integration().definition().clone(),
        ),
    ]
}

/// Number of built-in tools
pub const BUILTIN_COUNT: usize = 13;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_count() {
        let builtins = builtin_registrations();
        assert_eq!(builtins.len(), BUILTIN_COUNT);
    }

    #[test]
    fn test_no_duplicate_slugs() {
        let builtins = builtin_registrations();
        let mut slugs: Vec<&str> = builtins.iter().map(|r| r.slug.as_str()).collect();
        let original_len = slugs.len();
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), original_len, "Duplicate slugs found");
    }

    #[test]
    fn test_all_categories_represented() {
        let builtins = builtin_registrations();

        let has_ide = builtins.iter().any(|r| r.category == ToolCategory::Ide);
        let has_cli = builtins.iter().any(|r| r.category == ToolCategory::CliAgent);
        let has_auto = builtins.iter().any(|r| r.category == ToolCategory::Autonomous);
        let has_copilot = builtins.iter().any(|r| r.category == ToolCategory::Copilot);

        assert!(has_ide, "No IDE tools");
        assert!(has_cli, "No CLI agent tools");
        assert!(has_auto, "No autonomous tools");
        assert!(has_copilot, "No copilot tools");
    }

    #[test]
    fn test_specific_tools_exist() {
        let builtins = builtin_registrations();
        let slugs: Vec<&str> = builtins.iter().map(|r| r.slug.as_str()).collect();

        assert!(slugs.contains(&"cursor"));
        assert!(slugs.contains(&"claude"));
        assert!(slugs.contains(&"vscode"));
        assert!(slugs.contains(&"copilot"));
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/registry/mod.rs`:
```rust
//! Unified Tool Registry

mod builtins;
mod store;
mod types;

pub use builtins::{builtin_registrations, BUILTIN_COUNT};
pub use store::ToolRegistry;
pub use types::{ToolCategory, ToolRegistration};
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry::builtins`
Expected: 4 tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/
git commit -m "feat(repo-tools): add builtin_registrations() as single source of truth"
```

---

### Task 1.5: Add with_builtins() constructor to ToolRegistry

**Files:**
- Modify: `crates/repo-tools/src/registry/store.rs`

**Step 1: Add with_builtins() method**

Add to `ToolRegistry` impl in `crates/repo-tools/src/registry/store.rs`:
```rust
    /// Create a registry populated with all built-in tools
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        for registration in super::builtins::builtin_registrations() {
            registry.register(registration);
        }
        registry
    }
```

**Step 2: Add test**

Add test:
```rust
    #[test]
    fn test_with_builtins() {
        let registry = ToolRegistry::with_builtins();

        assert_eq!(registry.len(), super::builtins::BUILTIN_COUNT);
        assert!(registry.contains("cursor"));
        assert!(registry.contains("claude"));
        assert!(registry.contains("vscode"));
    }
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/store.rs
git commit -m "feat(repo-tools): add ToolRegistry::with_builtins() constructor"
```

---

## Phase 2: Capability Translator

**Goal:** Create translation layer that respects tool capabilities.

### Task 2.1: Create TranslatedContent type

**Files:**
- Create: `crates/repo-tools/src/translator/mod.rs`
- Create: `crates/repo-tools/src/translator/content.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create content.rs**

Create `crates/repo-tools/src/translator/content.rs`:
```rust
//! Translated content ready for config writing

use repo_meta::schema::ConfigType;
use serde_json::Value;
use std::collections::HashMap;

/// Content translated from registry, ready to be written to tool config
#[derive(Debug, Clone, Default)]
pub struct TranslatedContent {
    /// Target format for this content
    pub format: ConfigType,
    /// Translated instruction/rules content
    pub instructions: Option<String>,
    /// MCP server configurations (Phase 4)
    pub mcp_servers: Option<Value>,
    /// Additional key-value data for structured configs
    pub data: HashMap<String, Value>,
}

impl TranslatedContent {
    /// Create empty content
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create content with instructions only
    pub fn with_instructions(format: ConfigType, instructions: String) -> Self {
        Self {
            format,
            instructions: Some(instructions),
            ..Default::default()
        }
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_none()
            && self.mcp_servers.is_none()
            && self.data.is_empty()
    }

    /// Add data field (builder)
    pub fn with_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_content() {
        let content = TranslatedContent::empty();
        assert!(content.is_empty());
    }

    #[test]
    fn test_with_instructions() {
        let content = TranslatedContent::with_instructions(
            ConfigType::Markdown,
            "test".into(),
        );
        assert!(!content.is_empty());
        assert!(content.instructions.is_some());
    }

    #[test]
    fn test_with_data() {
        let content = TranslatedContent::empty()
            .with_data("key", serde_json::json!("value"));
        assert!(!content.is_empty());
    }
}
```

**Step 2: Create mod.rs**

Create `crates/repo-tools/src/translator/mod.rs`:
```rust
//! Capability-based translation layer

mod content;

pub use content::TranslatedContent;
```

**Step 3: Export from lib.rs**

Add to `crates/repo-tools/src/lib.rs`:
```rust
pub mod translator;
pub use translator::TranslatedContent;
```

**Step 4: Run tests and commit**

```bash
cargo test -p repo-tools translator
git add crates/repo-tools/src/translator/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add TranslatedContent type"
```

---

### Task 2.2: Create RuleTranslator

**Files:**
- Create: `crates/repo-tools/src/translator/rules.rs`
- Modify: `crates/repo-tools/src/translator/mod.rs`

**Step 1: Create rules.rs**

Create `crates/repo-tools/src/translator/rules.rs`:
```rust
//! Rule translation with semantic formatting

use super::TranslatedContent;
use repo_meta::schema::{ConfigType, RuleDefinition, Severity, ToolDefinition};

/// Translates rules to tool-specific format
pub struct RuleTranslator;

impl RuleTranslator {
    /// Translate rules for a tool (checks capabilities)
    pub fn translate(
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> TranslatedContent {
        // KEY: Check capability before translating
        if !tool.capabilities.supports_custom_instructions {
            return TranslatedContent::empty();
        }

        if rules.is_empty() {
            return TranslatedContent::empty();
        }

        let format = tool.integration.config_type;
        let instructions = Self::format_rules(rules, format);

        TranslatedContent::with_instructions(format, instructions)
    }

    fn format_rules(rules: &[RuleDefinition], format: ConfigType) -> String {
        // Sort by severity (mandatory first)
        let mut sorted: Vec<_> = rules.iter().collect();
        sorted.sort_by_key(|r| match r.meta.severity {
            Severity::Mandatory => 0,
            Severity::Suggestion => 1,
        });

        let mut output = String::new();
        for rule in sorted {
            output.push_str(&Self::format_rule(rule, format));
            output.push_str("\n\n");
        }
        output.trim_end().to_string()
    }

    fn format_rule(rule: &RuleDefinition, format: ConfigType) -> String {
        match format {
            ConfigType::Markdown | ConfigType::Text => {
                Self::format_markdown(rule)
            }
            _ => rule.content.instruction.clone(),
        }
    }

    fn format_markdown(rule: &RuleDefinition) -> String {
        let mut output = String::new();

        // Header with severity
        let marker = match rule.meta.severity {
            Severity::Mandatory => "**[REQUIRED]**",
            Severity::Suggestion => "[Suggested]",
        };
        output.push_str(&format!("## {} {}\n\n", rule.meta.id, marker));
        output.push_str(&rule.content.instruction);

        // Examples if present
        if let Some(ref examples) = rule.examples {
            if !examples.positive.is_empty() {
                output.push_str("\n\n**Good:**\n");
                for ex in &examples.positive {
                    output.push_str(&format!("```\n{}\n```\n", ex));
                }
            }
            if !examples.negative.is_empty() {
                output.push_str("\n**Bad:**\n");
                for ex in &examples.negative {
                    output.push_str(&format!("```\n{}\n```\n", ex));
                }
            }
        }

        // File targets if present
        if let Some(ref targets) = rule.targets {
            if !targets.file_patterns.is_empty() {
                output.push_str(&format!(
                    "\n\n**Applies to:** {}",
                    targets.file_patterns.join(", ")
                ));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        RuleContent, RuleMeta, ToolCapabilities, ToolIntegrationConfig, ToolMeta,
    };

    fn tool_with_capability(supports: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta { name: "T".into(), slug: "t".into(), description: None },
            integration: ToolIntegrationConfig {
                config_path: ".t".into(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: supports,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    fn make_rule(id: &str, severity: Severity) -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta { id: id.into(), severity, tags: vec![] },
            content: RuleContent { instruction: format!("{} instruction", id) },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_empty_when_no_capability() {
        let tool = tool_with_capability(false);
        let rules = vec![make_rule("r1", Severity::Mandatory)];

        let content = RuleTranslator::translate(&tool, &rules);
        assert!(content.is_empty());
    }

    #[test]
    fn test_translates_when_capable() {
        let tool = tool_with_capability(true);
        let rules = vec![make_rule("r1", Severity::Mandatory)];

        let content = RuleTranslator::translate(&tool, &rules);
        assert!(!content.is_empty());
        assert!(content.instructions.unwrap().contains("r1"));
    }

    #[test]
    fn test_mandatory_first() {
        let tool = tool_with_capability(true);
        let rules = vec![
            make_rule("suggestion", Severity::Suggestion),
            make_rule("mandatory", Severity::Mandatory),
        ];

        let content = RuleTranslator::translate(&tool, &rules);
        let text = content.instructions.unwrap();

        let mpos = text.find("mandatory").unwrap();
        let spos = text.find("suggestion").unwrap();
        assert!(mpos < spos);
    }
}
```

**Step 2: Update mod.rs**

```rust
mod content;
mod rules;

pub use content::TranslatedContent;
pub use rules::RuleTranslator;
```

**Step 3: Run tests and commit**

```bash
cargo test -p repo-tools translator
git add crates/repo-tools/src/translator/
git commit -m "feat(repo-tools): add RuleTranslator with capability checking"
```

---

### Task 2.3: Create CapabilityTranslator orchestrator

**Files:**
- Create: `crates/repo-tools/src/translator/capability.rs`
- Modify: `crates/repo-tools/src/translator/mod.rs`

**Step 1: Create capability.rs**

Create `crates/repo-tools/src/translator/capability.rs`:
```rust
//! Main capability translator orchestrator

use super::{RuleTranslator, TranslatedContent};
use repo_meta::schema::{RuleDefinition, ToolDefinition};

/// Orchestrates translation based on tool capabilities
pub struct CapabilityTranslator;

impl CapabilityTranslator {
    /// Translate all content for a tool based on its capabilities
    pub fn translate(
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> TranslatedContent {
        let mut content = TranslatedContent::empty();
        content.format = tool.integration.config_type;

        // Custom instructions
        if tool.capabilities.supports_custom_instructions {
            let rule_content = RuleTranslator::translate(tool, rules);
            content.instructions = rule_content.instructions;
        }

        // MCP will be added in Phase 4
        // Rules directory will be added in Phase 2+

        content
    }

    /// Check if tool has any translatable capability
    pub fn has_capabilities(tool: &ToolDefinition) -> bool {
        tool.capabilities.supports_custom_instructions
            || tool.capabilities.supports_mcp
            || tool.capabilities.supports_rules_directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        ConfigType, RuleContent, RuleMeta, Severity,
        ToolCapabilities, ToolIntegrationConfig, ToolMeta,
    };

    fn make_tool(supports: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta { name: "T".into(), slug: "t".into(), description: None },
            integration: ToolIntegrationConfig {
                config_path: ".t".into(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: supports,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    fn make_rule() -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta { id: "r".into(), severity: Severity::Mandatory, tags: vec![] },
            content: RuleContent { instruction: "Do this".into() },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_translate_with_capability() {
        let tool = make_tool(true);
        let content = CapabilityTranslator::translate(&tool, &[make_rule()]);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_translate_without_capability() {
        let tool = make_tool(false);
        let content = CapabilityTranslator::translate(&tool, &[make_rule()]);
        assert!(content.is_empty());
    }

    #[test]
    fn test_has_capabilities() {
        assert!(CapabilityTranslator::has_capabilities(&make_tool(true)));
        assert!(!CapabilityTranslator::has_capabilities(&make_tool(false)));
    }
}
```

**Step 2: Update mod.rs and lib.rs**

```rust
// translator/mod.rs
mod capability;
mod content;
mod rules;

pub use capability::CapabilityTranslator;
pub use content::TranslatedContent;
pub use rules::RuleTranslator;
```

```rust
// lib.rs
pub use translator::{CapabilityTranslator, RuleTranslator, TranslatedContent};
```

**Step 3: Run tests and commit**

```bash
cargo test -p repo-tools translator
git add crates/repo-tools/src/translator/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add CapabilityTranslator orchestrator"
```

---

## Phase 3: Config Writers

**Goal:** Create semantic-aware config writers.

### Task 3.1: Create ConfigWriter trait

**Files:**
- Create: `crates/repo-tools/src/writer/mod.rs`
- Create: `crates/repo-tools/src/writer/traits.rs`

*(Content as in capability-based plan Task 1.4)*

### Task 3.2: Create JsonWriter

*(Content as in capability-based plan Task 1.4)*

### Task 3.3: Create MarkdownWriter

*(Content as in capability-based plan Task 1.5)*

### Task 3.4: Create TextWriter and WriterRegistry

*(Content as in capability-based plan Tasks 1.6, 1.7)*

---

## Phase 4: Integration & Migration

**Goal:** Wire everything together and migrate from old dispatcher.

### Task 4.1: Create ToolSyncer

**Files:**
- Create: `crates/repo-tools/src/syncer.rs`

*(Content as in capability-based plan Task 1.8)*

### Task 4.2: Update ToolDispatcher to use new system

**Files:**
- Modify: `crates/repo-tools/src/dispatcher.rs`

**Step 1: Refactor dispatcher**

Update `dispatcher.rs` to delegate to `ToolRegistry`:
```rust
use crate::registry::ToolRegistry;

pub struct ToolDispatcher {
    registry: ToolRegistry,
}

impl ToolDispatcher {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::with_builtins(),
        }
    }

    pub fn get_integration(&self, name: &str) -> Option<Box<dyn ToolIntegration>> {
        self.registry.get(name).map(|reg| {
            Box::new(GenericToolIntegration::new(reg.definition.clone()))
                as Box<dyn ToolIntegration>
        })
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.registry.contains(name)
    }

    pub fn list_available(&self) -> Vec<String> {
        self.registry.list().into_iter().map(String::from).collect()
    }
}
```

**Step 2: Run all tests**

```bash
cargo test -p repo-tools
```

**Step 3: Commit**

```bash
git commit -am "refactor(repo-tools): migrate ToolDispatcher to use ToolRegistry"
```

### Task 4.3: Final cleanup

- Remove duplicate tool lists
- Update documentation
- Run clippy

---

## Success Criteria

- [ ] **Single Source of Truth**: `builtin_registrations()` is the only place tools are defined
- [ ] **Capabilities Respected**: `ToolCapabilities` controls what gets generated
- [ ] **Semantic Merge**: JSON preserves user content, Markdown uses sections
- [ ] **All Tests Pass**: `cargo test -p repo-tools`
- [ ] **No Clippy Warnings**: `cargo clippy -p repo-tools`

---

## Task Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 5 | Tool Registry (types, store, builtins) |
| 2 | 3 | Capability Translator (content, rules, orchestrator) |
| 3 | 4 | Config Writers (trait, JSON, Markdown, Text) |
| 4 | 3 | Integration (ToolSyncer, migration, cleanup) |
| **Total** | **15** | |

---

*Plan created: 2026-01-29*
*Supersedes: 2026-01-29-tool-registry-implementation.md, 2026-01-29-capability-based-registry.md*
