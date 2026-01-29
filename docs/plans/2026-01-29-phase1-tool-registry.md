# Phase 1: Tool Registry Foundation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create unified tool registry with single source of truth, eliminating 3-location duplication.

**Parent:** [2026-01-29-registry-architecture-index.md](2026-01-29-registry-architecture-index.md)
**Next Phase:** [2026-01-29-phase2-capability-translator.md](2026-01-29-phase2-capability-translator.md)

---

## What This Solves

Current `ToolDispatcher` has tool names in 3 places:
1. `get_integration()` match arms
2. `has_tool()` matches! macro
3. `list_available()` hardcoded vec

This phase creates `builtin_registrations()` as the SINGLE SOURCE OF TRUTH.

---

## Task 1.1: Create ToolCategory and ToolRegistration types

**Files:**
- Create: `crates/repo-tools/src/registry/mod.rs`
- Create: `crates/repo-tools/src/registry/types.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create types.rs**

```rust
//! Core types for the unified tool registry

use repo_meta::schema::ToolDefinition;
use serde::{Deserialize, Serialize};

/// Tool category for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    Ide,
    CliAgent,
    Autonomous,
    Copilot,
}

/// Complete tool registration
#[derive(Debug, Clone)]
pub struct ToolRegistration {
    pub slug: String,
    pub name: String,
    pub category: ToolCategory,
    pub priority: u8,
    pub definition: ToolDefinition,
}

impl ToolRegistration {
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

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn supports_instructions(&self) -> bool {
        self.definition.capabilities.supports_custom_instructions
    }

    pub fn supports_mcp(&self) -> bool {
        self.definition.capabilities.supports_mcp
    }

    pub fn has_any_capability(&self) -> bool {
        self.supports_instructions() || self.supports_mcp()
            || self.definition.capabilities.supports_rules_directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{ConfigType, ToolCapabilities, ToolIntegrationConfig, ToolMeta};

    fn make_def() -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta { name: "T".into(), slug: "t".into(), description: None },
            integration: ToolIntegrationConfig {
                config_path: ".t".into(),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        }
    }

    #[test]
    fn test_registration_new() {
        let reg = ToolRegistration::new("test", "Test", ToolCategory::Ide, make_def());
        assert_eq!(reg.slug, "test");
        assert_eq!(reg.priority, 50);
    }

    #[test]
    fn test_with_priority() {
        let reg = ToolRegistration::new("t", "T", ToolCategory::Ide, make_def())
            .with_priority(10);
        assert_eq!(reg.priority, 10);
    }
}
```

**Step 2: Create mod.rs**

```rust
//! Unified Tool Registry - Single Source of Truth

mod types;

pub use types::{ToolCategory, ToolRegistration};
```

**Step 3: Export from lib.rs**

Add: `pub mod registry;`

**Step 4: Test and commit**

```bash
cargo test -p repo-tools registry::types
git add crates/repo-tools/src/registry/ crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add ToolCategory and ToolRegistration"
```

---

## Task 1.2: Create ToolRegistry store

**Files:**
- Create: `crates/repo-tools/src/registry/store.rs`
- Modify: `crates/repo-tools/src/registry/mod.rs`

**Step 1: Create store.rs**

```rust
//! Tool registry storage

use super::{ToolCategory, ToolRegistration};
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, ToolRegistration>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, reg: ToolRegistration) {
        self.tools.insert(reg.slug.clone(), reg);
    }

    pub fn get(&self, slug: &str) -> Option<&ToolRegistration> {
        self.tools.get(slug)
    }

    pub fn contains(&self, slug: &str) -> bool {
        self.tools.contains_key(slug)
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    pub fn list(&self) -> Vec<&str> {
        let mut slugs: Vec<_> = self.tools.keys().map(|s| s.as_str()).collect();
        slugs.sort();
        slugs
    }

    pub fn by_category(&self, cat: ToolCategory) -> Vec<&str> {
        let mut slugs: Vec<_> = self.tools.iter()
            .filter(|(_, r)| r.category == cat)
            .map(|(s, _)| s.as_str())
            .collect();
        slugs.sort();
        slugs
    }

    pub fn by_priority(&self) -> Vec<&ToolRegistration> {
        let mut tools: Vec<_> = self.tools.values().collect();
        tools.sort_by_key(|t| t.priority);
        tools
    }

    pub fn iter(&self) -> impl Iterator<Item = &ToolRegistration> {
        self.tools.values()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    // Tests for register, get, list, by_category, by_priority
}
```

**Step 2: Update mod.rs**

```rust
mod store;
mod types;

pub use store::ToolRegistry;
pub use types::{ToolCategory, ToolRegistration};
```

**Step 3: Test and commit**

```bash
cargo test -p repo-tools registry
git add crates/repo-tools/src/registry/
git commit -m "feat(repo-tools): add ToolRegistry store"
```

---

## Task 1.3: Add definition() to GenericToolIntegration

**Files:**
- Modify: `crates/repo-tools/src/generic.rs`

**Step 1: Add method**

```rust
pub fn definition(&self) -> &ToolDefinition {
    &self.definition
}
```

**Step 2: Commit**

```bash
git add crates/repo-tools/src/generic.rs
git commit -m "feat(repo-tools): add definition() accessor"
```

---

## Task 1.4: Create builtin_registrations()

**Files:**
- Create: `crates/repo-tools/src/registry/builtins.rs`
- Modify: `crates/repo-tools/src/registry/mod.rs`

**Step 1: Create builtins.rs**

```rust
//! Built-in tool registrations - SINGLE SOURCE OF TRUTH

use super::{ToolCategory, ToolRegistration};
use crate::*;

pub const BUILTIN_COUNT: usize = 13;

pub fn builtin_registrations() -> Vec<ToolRegistration> {
    vec![
        // IDEs
        ToolRegistration::new("vscode", "VS Code", ToolCategory::Ide,
            vscode::VSCodeIntegration::new().definition().clone()),
        ToolRegistration::new("cursor", "Cursor", ToolCategory::Ide,
            cursor::cursor_integration().definition().clone()),
        ToolRegistration::new("zed", "Zed", ToolCategory::Ide,
            zed::zed_integration().definition().clone()),
        ToolRegistration::new("jetbrains", "JetBrains", ToolCategory::Ide,
            jetbrains::jetbrains_integration().definition().clone()),
        ToolRegistration::new("windsurf", "Windsurf", ToolCategory::Ide,
            windsurf::windsurf_integration().definition().clone()),
        ToolRegistration::new("antigravity", "Antigravity", ToolCategory::Ide,
            antigravity::antigravity_integration().definition().clone()),
        // CLI Agents
        ToolRegistration::new("claude", "Claude Code", ToolCategory::CliAgent,
            claude::claude_integration().definition().clone()),
        ToolRegistration::new("aider", "Aider", ToolCategory::CliAgent,
            aider::aider_integration().definition().clone()),
        ToolRegistration::new("gemini", "Gemini CLI", ToolCategory::CliAgent,
            gemini::gemini_integration().definition().clone()),
        // Autonomous
        ToolRegistration::new("cline", "Cline", ToolCategory::Autonomous,
            cline::cline_integration().definition().clone()),
        ToolRegistration::new("roo", "Roo", ToolCategory::Autonomous,
            roo::roo_integration().definition().clone()),
        // Copilots
        ToolRegistration::new("copilot", "GitHub Copilot", ToolCategory::Copilot,
            copilot::copilot_integration().definition().clone()),
        ToolRegistration::new("amazonq", "Amazon Q", ToolCategory::Copilot,
            amazonq::amazonq_integration().definition().clone()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        assert_eq!(builtin_registrations().len(), BUILTIN_COUNT);
    }

    #[test]
    fn test_no_duplicates() {
        let regs = builtin_registrations();
        let mut slugs: Vec<_> = regs.iter().map(|r| &r.slug).collect();
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), BUILTIN_COUNT);
    }
}
```

**Step 2: Update mod.rs and commit**

```bash
git add crates/repo-tools/src/registry/
git commit -m "feat(repo-tools): add builtin_registrations() single source of truth"
```

---

## Task 1.5: Add with_builtins() constructor

**Files:**
- Modify: `crates/repo-tools/src/registry/store.rs`

**Step 1: Add method**

```rust
pub fn with_builtins() -> Self {
    let mut registry = Self::new();
    for reg in super::builtins::builtin_registrations() {
        registry.register(reg);
    }
    registry
}
```

**Step 2: Test and commit**

```bash
cargo test -p repo-tools registry
git add crates/repo-tools/src/registry/
git commit -m "feat(repo-tools): add ToolRegistry::with_builtins()"
```

---

## Phase 1 Complete Checklist

- [ ] `ToolCategory` enum created
- [ ] `ToolRegistration` struct created
- [ ] `ToolRegistry` store created
- [ ] `definition()` added to `GenericToolIntegration`
- [ ] `builtin_registrations()` is SINGLE SOURCE OF TRUTH
- [ ] `with_builtins()` constructor works
- [ ] All tests pass

**Next:** [Phase 2 - Capability Translator](2026-01-29-phase2-capability-translator.md)
