# Tool Registry System Overhaul - Design Specification

**Date:** 2026-01-29
**Status:** Draft
**Author:** Architecture Review
**Related Research:** [2026-01-29-rust-registry-patterns.md](../research/2026-01-29-rust-registry-patterns.md)

---

## 1. Problem Statement

The current `ToolDispatcher` in `repo-tools` has tool names hardcoded in **three separate locations**:

1. `get_integration()` - match arms (lines 63-78)
2. `has_tool()` - matches! macro (lines 89-93)
3. `list_available()` - hardcoded vec (lines 119-134)

This violates DRY (Don't Repeat Yourself) and creates maintenance burden when adding new tools.

### Current Code Example

```rust
// Location 1: get_integration()
match tool_name {
    "vscode" => return Some(Box::new(VSCodeIntegration::new())),
    "cursor" => return Some(Box::new(cursor_integration())),
    // ... 11 more
}

// Location 2: has_tool()
matches!(tool_name,
    "vscode" | "cursor" | "claude" | /* ... 10 more */ )

// Location 3: list_available()
let mut tools = vec![
    "vscode".to_string(),
    "cursor".to_string(),
    // ... 11 more
];
```

---

## 2. Goals

### Primary Goals

1. **Single Source of Truth** - Define each tool exactly once
2. **Eliminate Duplication** - Remove the 3-location problem
3. **Unify Registration** - Treat built-in and schema-defined tools uniformly
4. **Add Capabilities** - Support categories, validation, priority, feature flags

### Non-Goals

- External plugin system (covered by key-decisions.md as subprocess-based)
- Runtime plugin loading (not needed for current use cases)
- Breaking changes to `ToolIntegration` trait (maintain compatibility)

---

## 3. Design Decisions

### 3.1 Hybrid Architecture

**Decision:** Use enum dispatch for built-ins, trait objects for extensions.

**Rationale:** Research shows `enum_dispatch` provides 10x performance for closed type sets, while trait objects provide necessary flexibility for user-defined tools. See [rust-registry-patterns.md](../research/2026-01-29-rust-registry-patterns.md#2-dispatch-performance).

### 3.2 Explicit Registration

**Decision:** Use Bevy-style explicit registration, not global/magic patterns.

**Rationale:** Rust community consensus favors explicit over implicit. Makes code auditable and feature flags straightforward. See [research](../research/2026-01-29-rust-registry-patterns.md#3-framework-patterns).

### 3.3 Declarative Macro for Definitions

**Decision:** Use a `define_builtins!` macro to generate all registration code from a single definition.

**Rationale:** Eliminates duplication while keeping definitions readable and maintainable.

### 3.4 Layered Configuration

**Decision:** Use figment for configuration layering (defaults → project → env).

**Rationale:** Already researched and recommended in [rust-config-libraries.md](../research/rust-config-libraries.md). Allows project-specific overrides without code changes.

---

## 4. Architecture

### 4.1 Component Overview

```
┌─────────────────────────────────────────────────────────┐
│                    ToolRegistry                          │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │  BuiltinTool    │    │  Schema-defined Tools       │ │
│  │  (enum_dispatch)│    │  (Box<dyn ToolIntegration>) │ │
│  │                 │    │                             │ │
│  │  - Cursor       │    │  - User TOML definitions    │ │
│  │  - Claude       │    │  - GenericToolIntegration   │ │
│  │  - VSCode       │    │                             │ │
│  │  - ... (13)     │    │                             │ │
│  └─────────────────┘    └─────────────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  Capabilities: Categories, Validation, Priority, Flags  │
└─────────────────────────────────────────────────────────┘
```

### 4.2 Data Flow

```
1. Startup
   ├── Load built-in tools (compile-time, enum)
   ├── Load project tools (.repository/tools/*.toml)
   ├── Apply config overrides (figment layering)
   └── Validate registry (conflict detection)

2. Tool Lookup
   ├── Check built-ins first (enum match, fast)
   └── Fall back to schema tools (HashMap lookup)

3. Sync Operation
   ├── Sort by priority
   ├── Filter by category (if specified)
   └── Execute in order
```

---

## 5. Type Definitions

### 5.1 Core Types

```rust
// crates/repo-tools/src/registry/types.rs

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

/// Metadata for a registered tool
#[derive(Debug, Clone)]
pub struct ToolRegistration {
    /// Unique identifier (slug)
    pub slug: String,

    /// Human-readable name
    pub name: String,

    /// Tool category
    pub category: ToolCategory,

    /// Sync priority (lower = syncs first)
    /// Default: 50, Range: 0-100
    pub priority: u8,

    /// The underlying tool definition
    pub definition: ToolDefinition,
}

/// Validation error for registry conflicts
#[derive(Debug, Clone)]
pub struct RegistryConflict {
    pub tool_a: String,
    pub tool_b: String,
    pub conflict_type: ConflictType,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum ConflictType {
    /// Two tools write to the same config path
    ConfigPathCollision,
    /// Circular priority dependency
    PriorityConflict,
}
```

### 5.2 Built-in Tools Enum

```rust
// crates/repo-tools/src/registry/builtins.rs

use enum_dispatch::enum_dispatch;

/// All built-in tool integrations
///
/// This enum is generated by the `define_builtins!` macro.
/// Using enum_dispatch for 10x performance over trait objects.
#[enum_dispatch(ToolIntegration)]
pub enum BuiltinTool {
    Cursor(CursorTool),
    Claude(ClaudeTool),
    VSCode(VSCodeTool),
    Windsurf(WindsurfTool),
    Copilot(CopilotTool),
    Cline(ClineTool),
    Roo(RooTool),
    JetBrains(JetBrainsTool),
    Zed(ZedTool),
    Aider(AiderTool),
    AmazonQ(AmazonQTool),
    Gemini(GeminiTool),
    Antigravity(AntigravityTool),
}
```

### 5.3 Registry Interface

```rust
// crates/repo-tools/src/registry/mod.rs

/// Unified tool registry combining built-ins and schema-defined tools
pub struct ToolRegistry {
    /// Built-in tools (fast enum dispatch)
    builtins: HashMap<String, (ToolRegistration, BuiltinTool)>,

    /// Schema-defined tools (flexible trait objects)
    schema_tools: HashMap<String, (ToolRegistration, Box<dyn ToolIntegration>)>,

    /// Configuration overrides from project/env
    overrides: ToolOverrides,
}

impl ToolRegistry {
    /// Create registry with all built-in tools
    pub fn with_builtins() -> Self;

    /// Load schema-defined tools from directory
    pub fn load_schema_tools(&mut self, dir: &Path) -> Result<()>;

    /// Apply configuration overrides (figment layering)
    pub fn apply_overrides(&mut self, overrides: ToolOverrides);

    /// Get integration by name (builtins first, then schema)
    pub fn get(&self, name: &str) -> Option<&dyn ToolIntegration>;

    /// Check if tool exists
    pub fn contains(&self, name: &str) -> bool;

    /// List all available tool names
    pub fn list(&self) -> Vec<&str>;

    /// List tools by category
    pub fn by_category(&self, category: ToolCategory) -> Vec<&str>;

    /// Validate registry for conflicts
    pub fn validate(&self) -> Result<(), Vec<RegistryConflict>>;

    /// Sync rules to tools, respecting priority order
    pub fn sync_all(&self, ctx: &SyncContext, tools: &[String], rules: &[Rule]) -> Result<Vec<String>>;
}
```

---

## 6. Macro Design

### 6.1 Definition Macro

```rust
// crates/repo-tools/src/registry/macros.rs

/// Define all built-in tools in a single location.
///
/// This macro generates:
/// 1. The `BuiltinTool` enum with enum_dispatch
/// 2. Tool wrapper structs
/// 3. `ToolRegistration` instances
/// 4. The `builtin_registrations()` function
///
/// # Example
///
/// ```rust
/// define_builtins! {
///     cursor => {
///         name: "Cursor",
///         category: Ide,
///         priority: 50,
///         #[cfg(feature = "cursor")]
///         definition: cursor::definition(),
///     },
///     claude => {
///         name: "Claude Code",
///         category: CliAgent,
///         priority: 50,
///         definition: claude::definition(),
///     },
/// }
/// ```
macro_rules! define_builtins {
    (
        $(
            $slug:ident => {
                name: $name:literal,
                category: $category:ident,
                priority: $priority:expr,
                $(#[$meta:meta])*
                definition: $def:expr $(,)?
            }
        ),* $(,)?
    ) => {
        // Implementation generates all necessary code
    };
}
```

### 6.2 Generated Output

The macro generates:

```rust
// 1. Wrapper structs for each tool
pub struct CursorTool(GenericToolIntegration);
impl ToolIntegration for CursorTool { /* delegates to inner */ }

// 2. The enum with enum_dispatch
#[enum_dispatch(ToolIntegration)]
pub enum BuiltinTool {
    #[cfg(feature = "cursor")]
    Cursor(CursorTool),
    Claude(ClaudeTool),
    // ...
}

// 3. Registration function
pub fn builtin_registrations() -> Vec<(ToolRegistration, BuiltinTool)> {
    vec![
        #[cfg(feature = "cursor")]
        (
            ToolRegistration {
                slug: "cursor".into(),
                name: "Cursor".into(),
                category: ToolCategory::Ide,
                priority: 50,
                definition: cursor::definition(),
            },
            BuiltinTool::Cursor(CursorTool(cursor::definition().into())),
        ),
        // ...
    ]
}
```

---

## 7. Configuration Schema

### 7.1 Project Tool Overrides

```toml
# .repository/tools.toml

# Override built-in tool settings
[tools.cursor]
enabled = true
priority = 40  # Sync before default (50)

[tools.claude]
enabled = true
# Override config path if tool changed location
config_path = ".claude/settings.json"

# Disable a built-in tool
[tools.jetbrains]
enabled = false

# Add a custom tool
[tools.custom-linter]
name = "Custom Linter"
category = "autonomous"
priority = 90
config_path = ".linter/rules.yaml"
config_type = "yaml"
```

### 7.2 Environment Variable Overrides

```bash
# Disable a tool via env
REPO_TOOL_JETBRAINS_ENABLED=false

# Override priority
REPO_TOOL_CURSOR_PRIORITY=30
```

---

## 8. Validation Rules

The registry validates for:

### 8.1 Config Path Collisions

```rust
// Two tools cannot write to the same path
if tool_a.config_path == tool_b.config_path {
    return Err(RegistryConflict {
        tool_a: tool_a.slug,
        tool_b: tool_b.slug,
        conflict_type: ConflictType::ConfigPathCollision,
        details: format!("Both write to {}", tool_a.config_path),
    });
}
```

### 8.2 Priority Validation

```rust
// Warn if multiple tools have same priority (order undefined)
// Error if circular priority dependencies detected
```

---

## 9. Migration Path

### 9.1 Phase 1: Add New Registry (Non-breaking)

1. Create `crates/repo-tools/src/registry/` module
2. Implement `ToolRegistry` with new architecture
3. Keep existing `ToolDispatcher` working
4. Add `enum_dispatch` dependency

### 9.2 Phase 2: Migrate Dispatcher

1. Update `ToolDispatcher` to use `ToolRegistry` internally
2. Deprecate direct match statements
3. Add `define_builtins!` macro with all 13 tools

### 9.3 Phase 3: Add Capabilities

1. Implement category filtering
2. Add validation logic
3. Implement priority-ordered sync
4. Add feature flag support

### 9.4 Phase 4: Remove Old Code

1. Remove hardcoded match statements
2. Remove duplicate tool lists
3. Update tests
4. Update documentation

---

## 10. Testing Strategy

### 10.1 Unit Tests

```rust
#[test]
fn test_registry_single_source_of_truth() {
    let registry = ToolRegistry::with_builtins();

    // All tools in list() should be gettable
    for name in registry.list() {
        assert!(registry.get(name).is_some());
        assert!(registry.contains(name));
    }
}

#[test]
fn test_category_filtering() {
    let registry = ToolRegistry::with_builtins();
    let ide_tools = registry.by_category(ToolCategory::Ide);

    assert!(ide_tools.contains(&"cursor"));
    assert!(ide_tools.contains(&"vscode"));
    assert!(!ide_tools.contains(&"claude")); // CLI agent, not IDE
}

#[test]
fn test_validation_detects_conflicts() {
    let mut registry = ToolRegistry::new();
    // Add two tools with same config path
    // Expect validation error
}

#[test]
fn test_priority_ordering() {
    let registry = ToolRegistry::with_builtins();
    // Modify priorities, verify sync order respects them
}
```

### 10.2 Integration Tests

```rust
#[test]
fn test_schema_tool_loading() {
    let temp = TempDir::new().unwrap();
    // Create .repository/tools/custom.toml
    // Load and verify tool is available
}

#[test]
fn test_config_override_layering() {
    // Built-in default → project override → env override
}
```

---

## 11. Dependencies

### New Dependencies

```toml
# crates/repo-tools/Cargo.toml
[dependencies]
enum_dispatch = "0.3"  # For fast built-in dispatch
```

### Existing Dependencies (Already Used)

- `figment` - Configuration layering
- `serde` - Serialization
- `repo-meta` - ToolDefinition types

---

## 12. Success Criteria

1. **Zero Duplication** - Each tool defined exactly once
2. **No Regressions** - All existing tests pass
3. **Performance** - Built-in tool lookup at least as fast (likely 10x faster)
4. **Extensibility** - Users can add tools via TOML without code changes
5. **Validation** - Registry detects and reports conflicts
6. **Feature Flags** - Individual tools can be disabled at compile time

---

## 13. Open Questions

1. **Macro complexity** - Should we use proc-macro for better error messages, or is declarative sufficient?
2. **Hot reload** - Should schema tools support runtime reloading?
3. **Tool dependencies** - Should tools be able to declare dependencies on other tools?

## 14. Implementation Notes

### VSCodeIntegration Refactoring

The current `VSCodeIntegration` is a standalone struct that doesn't use `GenericToolIntegration`.
During implementation, it should be refactored to use `GenericToolIntegration` with `ToolSchemaKeys`
for the Python path configuration. This ensures consistency and allows `definition()` access.

### enum_dispatch Consideration

The design originally specified `enum_dispatch` for 10x performance. However, after review:
- The current implementation already uses `Box<dyn ToolIntegration>` everywhere
- All tool integrations return `GenericToolIntegration` (same concrete type)
- The performance benefit is minimal for 13 tools
- Adding `enum_dispatch` would require significant refactoring

**Recommendation:** Defer `enum_dispatch` to a future optimization phase. The priority is
eliminating duplication first. The current `Box<dyn ToolIntegration>` approach is sufficient.

---

## 14. References

- [Rust Registry Patterns Research](../research/2026-01-29-rust-registry-patterns.md)
- [Rust Config Libraries](../research/rust-config-libraries.md)
- [Key Decisions - Plugin Architecture](key-decisions.md)
- [Current Dispatcher Implementation](../../crates/repo-tools/src/dispatcher.rs)

---

*Design created: 2026-01-29*
*Status: Ready for implementation planning*
