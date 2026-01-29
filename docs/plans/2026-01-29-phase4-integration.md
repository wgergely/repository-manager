# Phase 4: Integration & Migration

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Wire everything together and migrate from old dispatcher.

**Parent:** [2026-01-29-registry-architecture-index.md](2026-01-29-registry-architecture-index.md)
**Depends on:** [2026-01-29-phase3-config-writers.md](2026-01-29-phase3-config-writers.md)

---

## What This Solves

Connects all the pieces:
- `ToolRegistry` → `CapabilityTranslator` → `WriterRegistry`

And migrates the old `ToolDispatcher` to use the new system.

---

## Task 4.1: Create ToolSyncer

**Files:**
- Create: `crates/repo-tools/src/syncer.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create syncer.rs**

```rust
//! ToolSyncer - main entry point for capability-based sync

use crate::error::Result;
use crate::translator::CapabilityTranslator;
use crate::writer::{SchemaKeys, WriterRegistry};
use repo_fs::NormalizedPath;
use repo_meta::schema::{RuleDefinition, ToolDefinition};

pub struct ToolSyncer {
    writers: WriterRegistry,
}

impl ToolSyncer {
    pub fn new() -> Self {
        Self { writers: WriterRegistry::new() }
    }

    /// Sync rules to a tool's config
    ///
    /// Returns Ok(true) if content was written, Ok(false) if tool has no capabilities
    pub fn sync(
        &self,
        root: &NormalizedPath,
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> Result<bool> {
        if !CapabilityTranslator::has_capabilities(tool) {
            return Ok(false);
        }

        let content = CapabilityTranslator::translate(tool, rules);
        if content.is_empty() {
            return Ok(false);
        }

        let writer = self.writers.get_writer(tool.integration.config_type);
        let keys = tool.schema_keys.as_ref().map(SchemaKeys::from);
        let path = root.join(&tool.integration.config_path);

        writer.write(&path, &content, keys.as_ref())?;
        Ok(true)
    }

    /// Sync rules to multiple tools
    pub fn sync_all(
        &self,
        root: &NormalizedPath,
        tools: &[ToolDefinition],
        rules: &[RuleDefinition],
    ) -> Result<Vec<String>> {
        let mut synced = Vec::new();
        for tool in tools {
            if self.sync(root, tool, rules)? {
                synced.push(tool.meta.slug.clone());
            }
        }
        Ok(synced)
    }
}

impl Default for ToolSyncer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::*;
    use tempfile::TempDir;
    use std::fs;

    fn make_tool(slug: &str, supports: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta { name: slug.into(), slug: slug.into(), description: None },
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
        }
    }

    fn make_rule(id: &str) -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta { id: id.into(), severity: Severity::Mandatory, tags: vec![] },
            content: RuleContent { instruction: format!("{} content", id) },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_sync_capable_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolSyncer::new();

        let tool = make_tool("test", true);
        let rules = vec![make_rule("r1")];

        let result = syncer.sync(&root, &tool, &rules).unwrap();
        assert!(result);
        assert!(temp.path().join(".test").exists());
    }

    #[test]
    fn test_sync_incapable_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolSyncer::new();

        let tool = make_tool("test", false);
        let rules = vec![make_rule("r1")];

        let result = syncer.sync(&root, &tool, &rules).unwrap();
        assert!(!result);
        assert!(!temp.path().join(".test").exists());
    }

    #[test]
    fn test_sync_all() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());
        let syncer = ToolSyncer::new();

        let tools = vec![
            make_tool("a", true),
            make_tool("b", false),
            make_tool("c", true),
        ];
        let rules = vec![make_rule("r1")];

        let synced = syncer.sync_all(&root, &tools, &rules).unwrap();
        assert_eq!(synced.len(), 2);
        assert!(synced.contains(&"a".to_string()));
        assert!(synced.contains(&"c".to_string()));
    }
}
```

**Step 2: Export and commit**

```bash
cargo test -p repo-tools syncer
git add crates/repo-tools/src/syncer.rs crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add ToolSyncer main entry point"
```

---

## Task 4.2: Migrate ToolDispatcher to use ToolRegistry

**Files:**
- Modify: `crates/repo-tools/src/dispatcher.rs`

**Step 1: Refactor dispatcher**

Replace the hardcoded tool lists with registry delegation:

```rust
use crate::generic::GenericToolIntegration;
use crate::integration::ToolIntegration;
use crate::registry::ToolRegistry;

pub struct ToolDispatcher {
    registry: ToolRegistry,
}

impl ToolDispatcher {
    pub fn new() -> Self {
        Self { registry: ToolRegistry::with_builtins() }
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

    // Keep schema_tools for backward compatibility
    pub fn register(&mut self, definition: repo_meta::schema::ToolDefinition) {
        use crate::registry::{ToolCategory, ToolRegistration};
        let reg = ToolRegistration::new(
            &definition.meta.slug,
            &definition.meta.name,
            ToolCategory::Autonomous, // Default for user-defined
            definition,
        );
        self.registry.register(reg);
    }

    pub fn schema_tool_count(&self) -> usize {
        self.registry.len().saturating_sub(crate::registry::BUILTIN_COUNT)
    }
}
```

**Step 2: Run ALL existing tests**

```bash
cargo test -p repo-tools
```

All existing tests should pass because the API is unchanged.

**Step 3: Commit**

```bash
git add crates/repo-tools/src/dispatcher.rs
git commit -m "refactor(repo-tools): migrate ToolDispatcher to use ToolRegistry"
```

---

## Task 4.3: Final cleanup and verification

**Step 1: Run full test suite**

```bash
cargo test -p repo-tools
```

**Step 2: Run clippy**

```bash
cargo clippy -p repo-tools -- -D warnings
```

**Step 3: Verify no duplicate tool definitions**

Check that all hardcoded tool lists are gone:
- `get_integration()` no longer has 13 match arms
- `has_tool()` no longer has matches! with 13 tools
- `list_available()` no longer has hardcoded vec

**Step 4: Final commit**

```bash
git add -A
git commit -m "refactor(repo-tools): complete unified registry migration

- ToolRegistry is now single source of truth
- ToolCapabilities controls what gets generated
- ConfigWriters handle semantic merge
- Old dispatcher delegated to new system
- Zero duplication in tool definitions"
```

---

## Phase 4 Complete Checklist

- [ ] `ToolSyncer` created and tested
- [ ] `ToolDispatcher` migrated to use `ToolRegistry`
- [ ] All existing tests pass
- [ ] No clippy warnings
- [ ] No duplicate tool definitions remain
- [ ] API backward compatible

---

## Architecture Complete

```
┌─────────────────────────────────────────┐
│  ToolRegistry                           │
│  └── builtin_registrations()            │  ✓ Single source of truth
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  CapabilityTranslator                   │
│  └── checks tool.capabilities           │  ✓ Capabilities respected
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  WriterRegistry                         │
│  ├── JsonWriter (merge)                 │
│  ├── MarkdownWriter (sections)          │  ✓ Semantic merge
│  └── TextWriter (replace)               │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  Tool Config Files                      │  ✓ Generated correctly
└─────────────────────────────────────────┘
```

---

## Future Work

- **Phase 5:** MCP Server Support
- **Phase 6:** YAML/TOML AST-aware writers
- **Phase 7:** Rules directory support
