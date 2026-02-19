# VaultSpec Audit: Repo Manager Extension Integration Surface

**Date:** 2026-02-19
**Auditor:** repoman-surface (Opus agent)
**Source:** `repo-meta`, `repo-core`, `repo-tools` crates

---

## 1. Core Type Signatures

### ToolDefinition
```rust
// repo-meta/src/schema/tool.rs:32-43
pub struct ToolDefinition {
    pub meta: ToolMeta,                          // name, slug, description
    pub integration: ToolIntegrationConfig,       // config_path, config_type, additional_paths
    pub capabilities: ToolCapabilities,           // supports_custom_instructions, mcp, rules_directory
    pub schema_keys: Option<ToolSchemaKeys>,      // instruction_key, mcp_key, python_path_key
}
```

### ToolCapabilities
```rust
// repo-meta/src/schema/tool.rs:88-99
pub struct ToolCapabilities {
    pub supports_custom_instructions: bool,
    pub supports_mcp: bool,
    pub supports_rules_directory: bool,
}
```

### ToolSchemaKeys
```rust
// repo-meta/src/schema/tool.rs:104-112
pub struct ToolSchemaKeys {
    pub instruction_key: Option<String>,
    pub mcp_key: Option<String>,
    pub python_path_key: Option<String>,
}
```

### PresetDefinition
```rust
// repo-meta/src/schema/preset.rs:28-41
pub struct PresetDefinition {
    pub meta: PresetMeta,
    pub requires: PresetRequires,    // tools: Vec<String>, presets: Vec<String>
    pub rules: PresetRules,          // include: Vec<String>
    pub config: HashMap<String, toml::Value>,
}
```

### TranslatedContent
```rust
// repo-tools/src/translator/content.rs:12-21
pub struct TranslatedContent {
    pub format: ConfigType,
    pub instructions: Option<String>,
    pub mcp_servers: Option<Value>,     // EXISTS but unused
    pub data: HashMap<String, Value>,
}
```

### Projection
```rust
pub struct Projection { pub tool: String, pub file: PathBuf, pub kind: ProjectionKind }
pub enum ProjectionKind {
    TextBlock { marker: Uuid, checksum: String },
    JsonKey { path: String, value: Value },
    FileManaged { checksum: String },
}
```

### Manifest
```rust
// repo-core/src/config/manifest.rs:37-63
pub struct Manifest {
    pub core: CoreSection,
    pub presets: HashMap<String, Value>,
    pub tools: Vec<String>,
    pub rules: Vec<String>,
    pub hooks: Vec<HookConfig>,
    // NO extensions field
}
```

## 2. Extension Injection Points in Sync Flow

```
SyncEngine::sync_with_options()
  1. Load ledger
  2. Load config.toml -> Manifest
  3. For each tool: ToolSyncer::sync_tool()        <-- tool config creation
  4. RuleSyncer::sync_rules()                       <-- rule content sync
  5. Save ledger
  // ExtensionSyncer would inject between 4 and 5
```

## 3. Structures Needing New Fields

### Manifest
```rust
pub extensions: HashMap<String, Value>,  // or Vec<ExtensionConfig>
```

### SyncContext (for MCP threading)
```rust
pub mcp_servers: Option<Value>,  // from extensions
```

## 4. MCP Pipeline: Scaffolded but Dormant

**CapabilityTranslator** (capability.rs:33-35):
```rust
// MCP servers: Future phase (Phase 5)
// if tool.capabilities.supports_mcp {
//     content.mcp_servers = ...
// }
```

To activate:
1. Extension provides MCP config as `serde_json::Value`
2. CapabilityTranslator Phase 5 populates `TranslatedContent.mcp_servers`
3. JsonWriter uses `schema_keys.mcp_key` for placement
4. Text/Markdown tools need new MCP handling

## 5. VaultSpec Tool Configs as ToolDefinitions

**Direct mapping exists.** Claude and Gemini are already built-in ToolDefinitions:

```toml
# Claude (already exists)
[meta]
name = "Claude"
slug = "claude"
[integration]
config_path = "CLAUDE.md"
type = "markdown"
additional_paths = [".claude/rules/"]
[capabilities]
supports_custom_instructions = true
supports_rules_directory = true
supports_mcp = true
```

VaultSpec rules -> `RuleDefinition` entries
VaultSpec per-tool output -> exactly the `Projection` targets in `RuleSyncer`

The `GenericToolIntegration` provides fully schema-driven integration for any config type.

## 6. Key Gaps

1. **No `[extensions]` in Manifest**
2. **MCP pipeline dormant** (Phase 5 commented out)
3. **No ExtensionSyncer** in sync flow
4. **SyncContext lacks MCP/extension data**
5. **No YAML/TOML-aware writer** (fall back to TextWriter with full file replacement)
6. Architecture is clearly designed for extensions - gap is implementation, not design
