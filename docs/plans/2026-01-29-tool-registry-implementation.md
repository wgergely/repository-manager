# Tool Registry Overhaul Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate the 3-location tool name duplication in `ToolDispatcher` by creating a unified `ToolRegistry` with single source of truth.

**Architecture:** Hybrid registry using `enum_dispatch` for 13 built-in tools (10x performance) and trait objects for user-defined schema tools. A declarative macro generates all registration code from a single definition.

**Tech Stack:** Rust, enum_dispatch crate, figment (existing), serde (existing)

---

## Phase 1: Core Types and Registry Foundation

### Task 1.1: Create ToolCategory enum

> **Note:** The original design specified `enum_dispatch` for performance. After review,
> this is deferred to a future optimization phase. The current `Box<dyn ToolIntegration>`
> approach is sufficient for 13 tools, and the priority is eliminating duplication first.

**Files:**
- Create: `crates/repo-tools/src/registry/types.rs`
- Create: `crates/repo-tools/src/registry/mod.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/repo-tools/src/registry/types.rs`:
```rust
//! Core types for the tool registry system

use serde::{Deserialize, Serialize};

/// Tool category for filtering and organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    /// IDE integrations (VSCode, Cursor, Zed, JetBrains)
    Ide,
    /// CLI-based agents (Claude Code, Aider, Gemini CLI)
    CliAgent,
    /// Autonomous coding agents (Cline, Roo, Antigravity)
    Autonomous,
    /// Code completion tools (GitHub Copilot, Amazon Q)
    Copilot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_serialize() {
        let cat = ToolCategory::Ide;
        let json = serde_json::to_string(&cat).unwrap();
        assert_eq!(json, "\"ide\"");
    }

    #[test]
    fn test_category_deserialize() {
        let cat: ToolCategory = serde_json::from_str("\"cli_agent\"").unwrap();
        assert_eq!(cat, ToolCategory::CliAgent);
    }

    #[test]
    fn test_category_equality() {
        assert_eq!(ToolCategory::Ide, ToolCategory::Ide);
        assert_ne!(ToolCategory::Ide, ToolCategory::CliAgent);
    }
}
```

**Step 2: Create registry module**

Create `crates/repo-tools/src/registry/mod.rs`:
```rust
//! Unified tool registry with single source of truth
//!
//! This module provides a centralized registry for all tool integrations,
//! eliminating the duplication of tool names across multiple locations.

mod types;

pub use types::ToolCategory;
```

**Step 3: Export from lib.rs**

Add to `crates/repo-tools/src/lib.rs` after other module declarations:
```rust
pub mod registry;
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools registry`
Expected: All 3 tests pass

**Step 5: Commit**

```bash
git add crates/repo-tools/src/registry/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add ToolCategory enum for registry"
```

---

### Task 1.2: Create ToolRegistration struct

**Files:**
- Modify: `crates/repo-tools/src/registry/types.rs`

**Step 1: Add ToolRegistration struct and tests**

Add to `crates/repo-tools/src/registry/types.rs`:
```rust
use repo_meta::schema::ToolDefinition;

/// Metadata for a registered tool
#[derive(Debug, Clone)]
pub struct ToolRegistration {
    /// Unique identifier (slug)
    pub slug: String,
    /// Human-readable name
    pub name: String,
    /// Tool category
    pub category: ToolCategory,
    /// Sync priority (lower = syncs first, default: 50, range: 0-100)
    pub priority: u8,
    /// The underlying tool definition
    pub definition: ToolDefinition,
}

impl ToolRegistration {
    /// Create a new tool registration with default priority (50)
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

    /// Set the sync priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}
```

Add tests:
```rust
    #[test]
    fn test_registration_new() {
        use repo_meta::schema::{ConfigType, ToolCapabilities, ToolIntegrationConfig, ToolMeta};

        let def = ToolDefinition {
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
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        };

        let reg = ToolRegistration::new("test", "Test Tool", ToolCategory::Ide, def);
        assert_eq!(reg.slug, "test");
        assert_eq!(reg.name, "Test Tool");
        assert_eq!(reg.category, ToolCategory::Ide);
        assert_eq!(reg.priority, 50); // Default
    }

    #[test]
    fn test_registration_with_priority() {
        use repo_meta::schema::{ConfigType, ToolCapabilities, ToolIntegrationConfig, ToolMeta};

        let def = ToolDefinition {
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
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        };

        let reg = ToolRegistration::new("test", "Test", ToolCategory::Ide, def)
            .with_priority(30);
        assert_eq!(reg.priority, 30);
    }
```

**Step 2: Update mod.rs exports**

Update `crates/repo-tools/src/registry/mod.rs`:
```rust
pub use types::{ToolCategory, ToolRegistration};
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry`
Expected: All 5 tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/types.rs
git add crates/repo-tools/src/registry/mod.rs
git commit -m "feat(repo-tools): add ToolRegistration struct"
```

---

### Task 1.3: Create ConflictType and RegistryConflict

**Files:**
- Modify: `crates/repo-tools/src/registry/types.rs`

**Step 1: Add conflict types**

Add to `crates/repo-tools/src/registry/types.rs`:
```rust
/// Type of conflict detected in registry validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// Two tools write to the same config path
    ConfigPathCollision,
    /// Duplicate tool slug registered
    DuplicateSlug,
}

/// Validation error for registry conflicts
#[derive(Debug, Clone)]
pub struct RegistryConflict {
    pub tool_a: String,
    pub tool_b: String,
    pub conflict_type: ConflictType,
    pub details: String,
}

impl std::fmt::Display for RegistryConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Registry conflict between '{}' and '{}': {}",
            self.tool_a, self.tool_b, self.details
        )
    }
}

impl std::error::Error for RegistryConflict {}
```

Add test:
```rust
    #[test]
    fn test_conflict_display() {
        let conflict = RegistryConflict {
            tool_a: "cursor".into(),
            tool_b: "vscode".into(),
            conflict_type: ConflictType::ConfigPathCollision,
            details: "Both write to .vscode/settings.json".into(),
        };
        let msg = format!("{}", conflict);
        assert!(msg.contains("cursor"));
        assert!(msg.contains("vscode"));
    }
```

**Step 2: Update exports**

Update `crates/repo-tools/src/registry/mod.rs`:
```rust
pub use types::{ConflictType, RegistryConflict, ToolCategory, ToolRegistration};
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry`
Expected: All 6 tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/
git commit -m "feat(repo-tools): add RegistryConflict types"
```

---

## Phase 2: ToolRegistry Core Implementation

### Task 2.1: Create empty ToolRegistry struct

**Files:**
- Create: `crates/repo-tools/src/registry/registry.rs`
- Modify: `crates/repo-tools/src/registry/mod.rs`

**Step 1: Write failing test**

Create `crates/repo-tools/src/registry/registry.rs`:
```rust
//! The unified tool registry

use std::collections::HashMap;
use crate::integration::ToolIntegration;
use super::types::{ToolCategory, ToolRegistration};

/// Unified tool registry combining built-ins and schema-defined tools
pub struct ToolRegistry {
    /// All registered tools by slug
    tools: HashMap<String, (ToolRegistration, Box<dyn ToolIntegration>)>,
}

impl ToolRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Get the number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
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

    #[test]
    fn test_new_registry_is_empty() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/registry/mod.rs`:
```rust
mod registry;
mod types;

pub use registry::ToolRegistry;
pub use types::{ConflictType, RegistryConflict, ToolCategory, ToolRegistration};
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry::registry`
Expected: 1 test passes

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/
git commit -m "feat(repo-tools): add empty ToolRegistry struct"
```

---

### Task 2.2: Implement register() and get() methods

**Files:**
- Modify: `crates/repo-tools/src/registry/registry.rs`

**Step 1: Write failing tests**

Add to `crates/repo-tools/src/registry/registry.rs` in the tests module:
```rust
    use crate::generic::GenericToolIntegration;
    use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

    fn create_test_definition(slug: &str) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: format!("{} Tool", slug),
                slug: slug.into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: format!(".{}", slug),
                config_type: ConfigType::Text,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities::default(),
            schema_keys: None,
        }
    }

    fn create_test_registration(slug: &str) -> (ToolRegistration, Box<dyn ToolIntegration>) {
        let def = create_test_definition(slug);
        let reg = ToolRegistration::new(slug, format!("{} Tool", slug), ToolCategory::Ide, def.clone());
        let integration = Box::new(GenericToolIntegration::new(def)) as Box<dyn ToolIntegration>;
        (reg, integration)
    }

    #[test]
    fn test_register_and_get() {
        let mut registry = ToolRegistry::new();
        let (reg, integration) = create_test_registration("test");

        registry.register(reg, integration);

        assert_eq!(registry.len(), 1);
        assert!(registry.get("test").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_contains() {
        let mut registry = ToolRegistry::new();
        let (reg, integration) = create_test_registration("test");

        registry.register(reg, integration);

        assert!(registry.contains("test"));
        assert!(!registry.contains("unknown"));
    }
```

**Step 2: Implement the methods**

Add to `ToolRegistry` impl in `crates/repo-tools/src/registry/registry.rs`:
```rust
    /// Register a tool with its integration
    pub fn register(&mut self, registration: ToolRegistration, integration: Box<dyn ToolIntegration>) {
        self.tools.insert(registration.slug.clone(), (registration, integration));
    }

    /// Get a tool integration by slug
    pub fn get(&self, slug: &str) -> Option<&dyn ToolIntegration> {
        self.tools.get(slug).map(|(_, i)| i.as_ref())
    }

    /// Check if a tool is registered
    pub fn contains(&self, slug: &str) -> bool {
        self.tools.contains_key(slug)
    }

    /// Get tool registration metadata by slug
    pub fn get_registration(&self, slug: &str) -> Option<&ToolRegistration> {
        self.tools.get(slug).map(|(r, _)| r)
    }
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry::registry`
Expected: All 3 tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/registry.rs
git commit -m "feat(repo-tools): implement register/get for ToolRegistry"
```

---

### Task 2.3: Implement list() and by_category() methods

**Files:**
- Modify: `crates/repo-tools/src/registry/registry.rs`

**Step 1: Write failing tests**

Add tests:
```rust
    #[test]
    fn test_list_tools() {
        let mut registry = ToolRegistry::new();
        let (reg1, int1) = create_test_registration("aaa");
        let (reg2, int2) = create_test_registration("zzz");

        registry.register(reg1, int1);
        registry.register(reg2, int2);

        let list = registry.list();
        assert_eq!(list.len(), 2);
        // Should be sorted alphabetically
        assert_eq!(list[0], "aaa");
        assert_eq!(list[1], "zzz");
    }

    #[test]
    fn test_by_category() {
        let mut registry = ToolRegistry::new();

        let def1 = create_test_definition("cursor");
        let reg1 = ToolRegistration::new("cursor", "Cursor", ToolCategory::Ide, def1.clone());
        registry.register(reg1, Box::new(GenericToolIntegration::new(def1)));

        let def2 = create_test_definition("claude");
        let reg2 = ToolRegistration::new("claude", "Claude", ToolCategory::CliAgent, def2.clone());
        registry.register(reg2, Box::new(GenericToolIntegration::new(def2)));

        let ide_tools = registry.by_category(ToolCategory::Ide);
        assert_eq!(ide_tools.len(), 1);
        assert!(ide_tools.contains(&"cursor"));

        let cli_tools = registry.by_category(ToolCategory::CliAgent);
        assert_eq!(cli_tools.len(), 1);
        assert!(cli_tools.contains(&"claude"));
    }
```

**Step 2: Implement the methods**

Add to `ToolRegistry` impl:
```rust
    /// List all registered tool slugs (sorted alphabetically)
    pub fn list(&self) -> Vec<&str> {
        let mut slugs: Vec<&str> = self.tools.keys().map(|s| s.as_str()).collect();
        slugs.sort();
        slugs
    }

    /// List tools by category (sorted alphabetically)
    pub fn by_category(&self, category: ToolCategory) -> Vec<&str> {
        let mut slugs: Vec<&str> = self.tools
            .iter()
            .filter(|(_, (reg, _))| reg.category == category)
            .map(|(slug, _)| slug.as_str())
            .collect();
        slugs.sort();
        slugs
    }
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry::registry`
Expected: All 5 tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/registry.rs
git commit -m "feat(repo-tools): implement list/by_category for ToolRegistry"
```

---

### Task 2.4: Implement validate() method

**Files:**
- Modify: `crates/repo-tools/src/registry/registry.rs`

**Step 1: Write failing tests**

Add tests:
```rust
    #[test]
    fn test_validate_no_conflicts() {
        let mut registry = ToolRegistry::new();
        let (reg1, int1) = create_test_registration("cursor");
        let (reg2, int2) = create_test_registration("claude");

        registry.register(reg1, int1);
        registry.register(reg2, int2);

        let result = registry.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_detects_path_collision() {
        let mut registry = ToolRegistry::new();

        // Both tools write to same path
        let mut def1 = create_test_definition("tool1");
        def1.integration.config_path = ".shared/config".into();
        let reg1 = ToolRegistration::new("tool1", "Tool 1", ToolCategory::Ide, def1.clone());
        registry.register(reg1, Box::new(GenericToolIntegration::new(def1)));

        let mut def2 = create_test_definition("tool2");
        def2.integration.config_path = ".shared/config".into();
        let reg2 = ToolRegistration::new("tool2", "Tool 2", ToolCategory::Ide, def2.clone());
        registry.register(reg2, Box::new(GenericToolIntegration::new(def2)));

        let result = registry.validate();
        assert!(result.is_err());

        let conflicts = result.unwrap_err();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, super::types::ConflictType::ConfigPathCollision);
    }
```

**Step 2: Implement validate()**

Add to `ToolRegistry` impl:
```rust
    /// Validate the registry for conflicts
    ///
    /// Checks for:
    /// - Config path collisions (two tools writing to same path)
    pub fn validate(&self) -> Result<(), Vec<super::types::RegistryConflict>> {
        use super::types::{ConflictType, RegistryConflict};

        let mut conflicts = Vec::new();
        let mut path_to_tool: HashMap<String, &str> = HashMap::new();

        for (slug, (reg, _)) in &self.tools {
            let path = &reg.definition.integration.config_path;

            if let Some(existing) = path_to_tool.get(path) {
                conflicts.push(RegistryConflict {
                    tool_a: existing.to_string(),
                    tool_b: slug.clone(),
                    conflict_type: ConflictType::ConfigPathCollision,
                    details: format!("Both write to '{}'", path),
                });
            } else {
                path_to_tool.insert(path.clone(), slug);
            }
        }

        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(conflicts)
        }
    }
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry::registry`
Expected: All 7 tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/registry.rs
git commit -m "feat(repo-tools): implement validate() for ToolRegistry"
```

---

## Phase 3: Built-in Tools Registration

### Task 3.1: Create builtins module with tool definitions

**Files:**
- Create: `crates/repo-tools/src/registry/builtins.rs`
- Modify: `crates/repo-tools/src/registry/mod.rs`
- Modify: `crates/repo-tools/src/generic.rs` (add `definition()` method)
- Modify: `crates/repo-tools/src/vscode.rs` (refactor to use GenericToolIntegration or add definition)

**Note:** `VSCodeIntegration` is currently a standalone struct, not using `GenericToolIntegration`.
We have two options:
1. Refactor VSCode to use `GenericToolIntegration` (consistent with other tools)
2. Create a `ToolDefinition` inline for VSCode in the builtins module

Option 1 is preferred for consistency. The implementation below assumes this refactoring.

**Step 0: Refactor VSCodeIntegration to use GenericToolIntegration**

Update `crates/repo-tools/src/vscode.rs`:
```rust
//! VSCode integration for Repository Manager.

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta, ToolSchemaKeys};

/// Creates a VSCode integration.
pub fn vscode_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "VS Code".into(),
            slug: "vscode".into(),
            description: Some("Microsoft Visual Studio Code".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".vscode/settings.json".into(),
            config_type: ConfigType::Json,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: false,
            supports_mcp: false,
            supports_rules_directory: false,
        },
        schema_keys: Some(ToolSchemaKeys {
            instruction_key: None,
            mcp_key: None,
            python_path_key: Some("python.defaultInterpreterPath".into()),
        }),
    })
}

/// Type alias for backward compatibility
pub type VSCodeIntegration = GenericToolIntegration;

/// Backward compatible constructor
impl VSCodeIntegration {
    pub fn new() -> GenericToolIntegration {
        vscode_integration()
    }
}
```

**Step 1: Create builtins module**

Create `crates/repo-tools/src/registry/builtins.rs`:
```rust
//! Built-in tool registrations
//!
//! This is the SINGLE SOURCE OF TRUTH for all built-in tools.
//! Each tool is defined once here and the registry is populated from this list.

use super::types::{ToolCategory, ToolRegistration};
use crate::integration::ToolIntegration;

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
/// This is the single source of truth for built-in tools.
/// Adding a new tool? Add it here and nowhere else.
pub fn builtin_registrations() -> Vec<(ToolRegistration, Box<dyn ToolIntegration>)> {
    vec![
        // IDE integrations
        builtin_vscode(),
        builtin_cursor(),
        builtin_zed(),
        builtin_jetbrains(),
        builtin_windsurf(),
        builtin_antigravity(),
        // CLI agents
        builtin_claude(),
        builtin_aider(),
        builtin_gemini(),
        // Autonomous agents
        builtin_cline(),
        builtin_roo(),
        // Copilots
        builtin_copilot(),
        builtin_amazonq(),
    ]
}

fn builtin_vscode() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    use crate::vscode::vscode_integration;
    let integration = vscode_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("vscode", "VS Code", ToolCategory::Ide, def);
    (reg, Box::new(integration))
}

fn builtin_cursor() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = cursor_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("cursor", "Cursor", ToolCategory::Ide, def);
    (reg, Box::new(integration))
}

fn builtin_zed() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = zed_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("zed", "Zed", ToolCategory::Ide, def);
    (reg, Box::new(integration))
}

fn builtin_jetbrains() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = jetbrains_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("jetbrains", "JetBrains", ToolCategory::Ide, def);
    (reg, Box::new(integration))
}

fn builtin_windsurf() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = windsurf_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("windsurf", "Windsurf", ToolCategory::Ide, def);
    (reg, Box::new(integration))
}

fn builtin_antigravity() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = antigravity_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("antigravity", "Antigravity", ToolCategory::Ide, def);
    (reg, Box::new(integration))
}

fn builtin_claude() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = claude_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("claude", "Claude Code", ToolCategory::CliAgent, def);
    (reg, Box::new(integration))
}

fn builtin_aider() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = aider_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("aider", "Aider", ToolCategory::CliAgent, def);
    (reg, Box::new(integration))
}

fn builtin_gemini() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = gemini_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("gemini", "Gemini CLI", ToolCategory::CliAgent, def);
    (reg, Box::new(integration))
}

fn builtin_cline() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = cline_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("cline", "Cline", ToolCategory::Autonomous, def);
    (reg, Box::new(integration))
}

fn builtin_roo() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = roo_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("roo", "Roo", ToolCategory::Autonomous, def);
    (reg, Box::new(integration))
}

fn builtin_copilot() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = copilot_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("copilot", "GitHub Copilot", ToolCategory::Copilot, def);
    (reg, Box::new(integration))
}

fn builtin_amazonq() -> (ToolRegistration, Box<dyn ToolIntegration>) {
    let integration = amazonq_integration();
    let def = integration.definition().clone();
    let reg = ToolRegistration::new("amazonq", "Amazon Q", ToolCategory::Copilot, def);
    (reg, Box::new(integration))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_count() {
        let builtins = builtin_registrations();
        assert_eq!(builtins.len(), 13, "Expected 13 built-in tools");
    }

    #[test]
    fn test_no_duplicate_slugs() {
        let builtins = builtin_registrations();
        let mut slugs: Vec<&str> = builtins.iter().map(|(r, _)| r.slug.as_str()).collect();
        let original_len = slugs.len();
        slugs.sort();
        slugs.dedup();
        assert_eq!(slugs.len(), original_len, "Duplicate slugs found in builtins");
    }

    #[test]
    fn test_all_categories_represented() {
        let builtins = builtin_registrations();

        let has_ide = builtins.iter().any(|(r, _)| r.category == ToolCategory::Ide);
        let has_cli = builtins.iter().any(|(r, _)| r.category == ToolCategory::CliAgent);
        let has_auto = builtins.iter().any(|(r, _)| r.category == ToolCategory::Autonomous);
        let has_copilot = builtins.iter().any(|(r, _)| r.category == ToolCategory::Copilot);

        assert!(has_ide, "No IDE tools found");
        assert!(has_cli, "No CLI agent tools found");
        assert!(has_auto, "No autonomous tools found");
        assert!(has_copilot, "No copilot tools found");
    }
}
```

**Step 2: This will fail because GenericToolIntegration doesn't expose definition()**

We need to add a `definition()` method. First, let's check if it exists.

**Step 3: Add definition() method to GenericToolIntegration**

Add to `crates/repo-tools/src/generic.rs` in the `GenericToolIntegration` impl:
```rust
    /// Get a reference to the underlying tool definition
    pub fn definition(&self) -> &ToolDefinition {
        &self.definition
    }
```

**Step 4: Add definition() to VSCodeIntegration**

Check `crates/repo-tools/src/vscode.rs` and add similar method if needed.

**Step 5: Update mod.rs**

Update `crates/repo-tools/src/registry/mod.rs`:
```rust
mod builtins;
mod registry;
mod types;

pub use builtins::builtin_registrations;
pub use registry::ToolRegistry;
pub use types::{ConflictType, RegistryConflict, ToolCategory, ToolRegistration};
```

**Step 6: Run tests**

Run: `cargo test -p repo-tools registry::builtins`
Expected: All 3 tests pass

**Step 7: Commit**

```bash
git add crates/repo-tools/src/registry/builtins.rs
git add crates/repo-tools/src/registry/mod.rs
git add crates/repo-tools/src/generic.rs
git add crates/repo-tools/src/vscode.rs
git commit -m "feat(repo-tools): add builtin_registrations() single source of truth"
```

---

### Task 3.2: Implement with_builtins() constructor

**Files:**
- Modify: `crates/repo-tools/src/registry/registry.rs`

**Step 1: Write failing test**

Add test:
```rust
    #[test]
    fn test_with_builtins() {
        let registry = ToolRegistry::with_builtins();

        // Should have all 13 built-in tools
        assert_eq!(registry.len(), 13);

        // Spot check some tools
        assert!(registry.contains("vscode"));
        assert!(registry.contains("cursor"));
        assert!(registry.contains("claude"));
        assert!(registry.contains("copilot"));
    }

    #[test]
    fn test_with_builtins_validates() {
        let registry = ToolRegistry::with_builtins();

        // Built-ins should have no conflicts
        assert!(registry.validate().is_ok());
    }
```

**Step 2: Implement with_builtins()**

Add to `ToolRegistry` impl:
```rust
    /// Create a registry populated with all built-in tools
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();

        for (registration, integration) in super::builtins::builtin_registrations() {
            registry.register(registration, integration);
        }

        registry
    }
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/registry.rs
git commit -m "feat(repo-tools): implement with_builtins() for ToolRegistry"
```

---

## Phase 4: Migrate ToolDispatcher

### Task 4.1: Refactor ToolDispatcher to use ToolRegistry internally

**Files:**
- Modify: `crates/repo-tools/src/dispatcher.rs`

**Step 1: Update ToolDispatcher to use ToolRegistry**

Replace the contents of `get_integration()`, `has_tool()`, and `list_available()` to delegate to a `ToolRegistry`:

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

    pub fn with_definitions(definitions: HashMap<String, ToolDefinition>) -> Self {
        let mut dispatcher = Self::new();
        for (_, def) in definitions {
            dispatcher.register(def);
        }
        dispatcher
    }

    pub fn register(&mut self, definition: ToolDefinition) {
        use crate::generic::GenericToolIntegration;
        use crate::registry::{ToolCategory, ToolRegistration};

        let slug = definition.meta.slug.clone();
        let name = definition.meta.name.clone();
        let reg = ToolRegistration::new(&slug, &name, ToolCategory::Autonomous, definition.clone());
        let integration = Box::new(GenericToolIntegration::new(definition));
        self.registry.register(reg, integration);
    }

    pub fn get_integration(&self, tool_name: &str) -> Option<Box<dyn ToolIntegration>> {
        // For backward compatibility, return a cloned Box
        // This is less efficient but maintains API compatibility
        self.registry.get(tool_name).map(|_| {
            // Re-create the integration (needed because we can't clone Box<dyn>)
            self.create_integration(tool_name)
        }).flatten()
    }

    fn create_integration(&self, tool_name: &str) -> Option<Box<dyn ToolIntegration>> {
        // Delegate to builtins or schema tools
        if let Some(reg) = self.registry.get_registration(tool_name) {
            Some(Box::new(GenericToolIntegration::new(reg.definition.clone())))
        } else {
            None
        }
    }

    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.registry.contains(tool_name)
    }

    pub fn sync_all(
        &self,
        context: &SyncContext,
        tool_names: &[String],
        rules: &[Rule],
    ) -> Result<Vec<String>> {
        let mut synced = Vec::new();
        for name in tool_names {
            if let Some(integration) = self.registry.get(name) {
                integration.sync(context, rules)?;
                synced.push(name.clone());
            }
        }
        Ok(synced)
    }

    pub fn list_available(&self) -> Vec<String> {
        self.registry.list().into_iter().map(String::from).collect()
    }

    pub fn schema_tool_count(&self) -> usize {
        // Count tools beyond the 13 built-ins
        self.registry.len().saturating_sub(13)
    }
}
```

**Step 2: Run existing tests**

Run: `cargo test -p repo-tools dispatcher`
Expected: All existing tests should pass

**Step 3: Commit**

```bash
git add crates/repo-tools/src/dispatcher.rs
git commit -m "refactor(repo-tools): migrate ToolDispatcher to use ToolRegistry"
```

---

### Task 4.2: Update integration tests

**Files:**
- Modify: `crates/repo-tools/tests/dispatcher_tests.rs`

**Step 1: Run integration tests**

Run: `cargo test -p repo-tools --test dispatcher_tests`
Expected: May need adjustments

**Step 2: Fix any failing tests**

Update tests as needed to work with new implementation.

**Step 3: Commit**

```bash
git add crates/repo-tools/tests/
git commit -m "test(repo-tools): update integration tests for registry migration"
```

---

## Phase 5: Add Priority-Ordered Sync

### Task 5.1: Implement sync_all with priority ordering

**Files:**
- Modify: `crates/repo-tools/src/registry/registry.rs`

**Step 1: Write failing test**

Add test:
```rust
    #[test]
    fn test_sync_respects_priority() {
        use crate::integration::{Rule, SyncContext};
        use repo_fs::NormalizedPath;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let mut registry = ToolRegistry::new();

        // Add tools with different priorities
        let def1 = create_test_definition("low_priority");
        let reg1 = ToolRegistration::new("low_priority", "Low", ToolCategory::Ide, def1.clone())
            .with_priority(90);
        registry.register(reg1, Box::new(GenericToolIntegration::new(def1)));

        let def2 = create_test_definition("high_priority");
        let reg2 = ToolRegistration::new("high_priority", "High", ToolCategory::Ide, def2.clone())
            .with_priority(10);
        registry.register(reg2, Box::new(GenericToolIntegration::new(def2)));

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![Rule { id: "test".into(), content: "content".into() }];

        // Get sync order
        let order = registry.get_sync_order(&["low_priority".into(), "high_priority".into()]);

        // High priority (10) should come before low priority (90)
        assert_eq!(order[0], "high_priority");
        assert_eq!(order[1], "low_priority");
    }
```

**Step 2: Implement get_sync_order()**

Add to `ToolRegistry` impl:
```rust
    /// Get tools in priority order (lower priority number = syncs first)
    pub fn get_sync_order(&self, tool_names: &[String]) -> Vec<String> {
        let mut tools_with_priority: Vec<(String, u8)> = tool_names
            .iter()
            .filter_map(|name| {
                self.get_registration(name).map(|reg| (name.clone(), reg.priority))
            })
            .collect();

        tools_with_priority.sort_by_key(|(_, priority)| *priority);
        tools_with_priority.into_iter().map(|(name, _)| name).collect()
    }

    /// Sync rules to tools in priority order
    pub fn sync_all(
        &self,
        context: &crate::integration::SyncContext,
        tool_names: &[String],
        rules: &[crate::integration::Rule],
    ) -> crate::error::Result<Vec<String>> {
        let ordered = self.get_sync_order(tool_names);
        let mut synced = Vec::new();

        for name in ordered {
            if let Some(integration) = self.get(&name) {
                integration.sync(context, rules)?;
                synced.push(name);
            }
        }

        Ok(synced)
    }
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools registry`
Expected: All tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/registry/registry.rs
git commit -m "feat(repo-tools): implement priority-ordered sync in ToolRegistry"
```

---

## Phase 6: Final Cleanup and Documentation

### Task 6.1: Remove duplicate code from dispatcher.rs

**Files:**
- Modify: `crates/repo-tools/src/dispatcher.rs`

**Step 1: Clean up old match statements**

Remove all the hardcoded tool lists that are now handled by the registry.

**Step 2: Run all tests**

Run: `cargo test -p repo-tools`
Expected: All tests pass

**Step 3: Commit**

```bash
git add crates/repo-tools/src/dispatcher.rs
git commit -m "refactor(repo-tools): remove duplicate tool lists from dispatcher"
```

---

### Task 6.2: Update module documentation

**Files:**
- Modify: `crates/repo-tools/src/registry/mod.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Add comprehensive module docs**

Update `crates/repo-tools/src/registry/mod.rs`:
```rust
//! Unified Tool Registry
//!
//! This module provides a centralized registry for all tool integrations,
//! solving the "3-location duplication" problem where tool names were
//! previously hardcoded in multiple places.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    ToolRegistry                          │
//! ├─────────────────────────────────────────────────────────┤
//! │  Built-in Tools (13)    │  Schema-defined Tools         │
//! │  - Populated from       │  - Loaded from TOML files     │
//! │    builtin_registrations│  - Uses GenericToolIntegration│
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Single Source of Truth
//!
//! All built-in tools are defined in [`builtins::builtin_registrations()`].
//! To add a new built-in tool, add it there and nowhere else.
//!
//! # Usage
//!
//! ```rust
//! use repo_tools::registry::{ToolRegistry, ToolCategory};
//!
//! // Create registry with all built-ins
//! let registry = ToolRegistry::with_builtins();
//!
//! // Query tools
//! assert!(registry.contains("cursor"));
//!
//! // Filter by category
//! let ide_tools = registry.by_category(ToolCategory::Ide);
//! ```
```

**Step 2: Commit**

```bash
git add crates/repo-tools/src/registry/
git add crates/repo-tools/src/lib.rs
git commit -m "docs(repo-tools): add comprehensive registry documentation"
```

---

### Task 6.3: Run full test suite and verify

**Step 1: Run all repo-tools tests**

Run: `cargo test -p repo-tools`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy -p repo-tools -- -D warnings`
Expected: No warnings

**Step 3: Verify documentation builds**

Run: `cargo doc -p repo-tools --no-deps`
Expected: Documentation builds without errors

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(repo-tools): complete tool registry overhaul

- Single source of truth for all 13 built-in tools
- ToolRegistry with category filtering and priority ordering
- Validation for config path conflicts
- Backward-compatible ToolDispatcher migration
- Comprehensive test coverage

Closes: registry-overhaul"
```

---

## Success Criteria Checklist

After completing all tasks, verify:

- [ ] **Zero Duplication**: Tool names defined in exactly one place (`builtins.rs`)
- [ ] **All Tests Pass**: `cargo test -p repo-tools` succeeds
- [ ] **No Clippy Warnings**: `cargo clippy -p repo-tools` clean
- [ ] **Categories Work**: `registry.by_category()` returns correct tools
- [ ] **Validation Works**: `registry.validate()` detects path conflicts
- [ ] **Priority Works**: `registry.sync_all()` respects priority order
- [ ] **Backward Compatible**: Existing `ToolDispatcher` API unchanged

---

## Estimated Task Count

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 3 | Core types (ToolCategory, ToolRegistration, conflicts) |
| 2 | 4 | ToolRegistry core (new, register, get, list, validate) |
| 3 | 2 | Built-in registrations (includes VSCode refactor) |
| 4 | 2 | Migrate ToolDispatcher |
| 5 | 1 | Priority-ordered sync |
| 6 | 3 | Cleanup and docs |
| **Total** | **15** | |

> **Note:** `enum_dispatch` optimization deferred to future phase. Current focus is on
> eliminating duplication with minimal refactoring risk.

---

*Plan created: 2026-01-29*
*Design reference: docs/design/2026-01-29-tool-registry-overhaul.md*
