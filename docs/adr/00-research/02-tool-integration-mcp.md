# Research: Tool Integration and MCP Sync System

**Date:** 2026-02-19
**Researcher:** tools-researcher (Opus agent)
**Source:** `repo-tools` crate

---

## 1. ToolIntegration Trait

**File:** `crates/repo-tools/src/integration.rs` (lines 73-84)

```rust
pub trait ToolIntegration {
    fn name(&self) -> &str;
    fn config_locations(&self) -> Vec<ConfigLocation>;
    fn sync(&self, context: &SyncContext, rules: &[Rule]) -> Result<()>;
}
```

Supporting types:
- `Rule`: `{ id: String, content: String }`
- `SyncContext`: `{ root: NormalizedPath, python_path: Option<NormalizedPath> }`
- `ConfigLocation`: `{ path: String, config_type: ConfigType, is_directory: bool }`

## 2. ToolCapabilitySyncer

**File:** `crates/repo-tools/src/syncer.rs` (lines 17-86)

Parallel sync system to the `ToolIntegration` trait. Works with `ToolDefinition` schemas directly.

```rust
pub struct ToolCapabilitySyncer {
    writers: WriterRegistry,
}

impl ToolCapabilitySyncer {
    pub fn new() -> Self;
    pub fn sync(&self, root: &NormalizedPath, tool: &ToolDefinition, rules: &[RuleDefinition]) -> Result<bool>;
    pub fn sync_all(&self, root: &NormalizedPath, tools: &[ToolDefinition], rules: &[RuleDefinition]) -> Result<Vec<String>>;
}
```

Sync flow:
1. `CapabilityTranslator::has_capabilities(tool)` - checks any capability
2. `CapabilityTranslator::translate(tool, rules)` - produces `TranslatedContent`
3. `WriterRegistry::get_writer(config_type)` - selects writer
4. `writer.write(&path, &content, keys)` - writes to disk

## 3. MCP Capability Map Across Built-in Tools

| Tool | instructions | mcp | rules_dir |
|------|:-----------:|:---:|:---------:|
| claude | true | **true** | true |
| roo | true | **true** | true |
| jetbrains | true | **true** | true |
| zed | true | **true** | false |
| cursor | true | false | false |
| windsurf | true | false | false |
| antigravity | true | false | true |
| gemini | true | false | false |
| copilot | true | false | true |
| cline | true | false | true |
| vscode | false | false | false |
| aider | true | false | false |
| amazonq | true | false | true |

**Only 4 tools declare `supports_mcp = true`.**

## 4. MCP Pipeline: Fully Scaffolded but Dormant

In `translator/capability.rs:32-35`, MCP translation is explicitly commented out:
```rust
// MCP servers: Future phase (Phase 5)
// if tool.capabilities.supports_mcp {
//     content.mcp_servers = ...
// }
```

**Infrastructure already in place:**
- `TranslatedContent.mcp_servers: Option<serde_json::Value>` (content.rs:18)
- `TranslatedContent::with_mcp_servers(servers: Value)` builder (content.rs:50-53)
- `JsonWriter::merge()` handles `mcp_servers` with `mcp_key` placement (json.rs:51-55)
- `SchemaKeys.mcp_key` maps the placement key (traits.rs:38)

## 5. GenericToolIntegration

**File:** `crates/repo-tools/src/generic.rs`

Most built-in tools delegate to `GenericToolIntegration`. Only `vscode` has a custom impl.

Sync routing by ConfigType:
- `Text` -> managed blocks via `repo_blocks::upsert_block`
- `Json` -> merges into JSON with schema_keys
- `Markdown` -> delegates to sync_text
- `Yaml` -> YAML comment-style block markers
- `Toml` -> reuses sync_yaml

If `config_path` ends with `/`, writes one file per rule as `{NN}-{sanitized-id}.md`.

## 6. ToolDispatcher

**File:** `crates/repo-tools/src/dispatcher.rs`

```rust
pub struct ToolDispatcher {
    registry: ToolRegistry,
    schema_tools: HashMap<String, ToolDefinition>,
}
```

Routing: built-in registry first, then schema_tools fallback. Methods: `register()`, `with_definitions()`, `sync_all()`, `list_available()`.

## 7. WriterRegistry

```rust
pub struct WriterRegistry {
    json: JsonWriter,
    markdown: MarkdownWriter,
    text: TextWriter,
}
```

- `Json` -> `JsonWriter` (preserves keys, merges instructions/mcp/data)
- `Markdown` -> `MarkdownWriter` (managed block markers)
- `Text | Yaml | Toml` -> `TextWriter` (full file replacement)

## 8. Extension MCP Data Flow Path

For an extension to provide MCP servers:
1. Extension declares MCP config in manifest
2. CapabilityTranslator Phase 5 uncommented, populates `TranslatedContent.mcp_servers`
3. WriterRegistry routes to appropriate writer
4. JsonWriter places at `schema_keys.mcp_key` for JSON-config tools
5. Text/Markdown tools need new MCP handling

**Gaps:**
- No MCP server definition schema in ToolDefinition
- Only JsonWriter handles MCP
- No per-tool MCP config format mapping
