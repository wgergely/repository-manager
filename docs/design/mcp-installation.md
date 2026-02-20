# Design: MCP Server Installation Management

**Status:** Draft
**Date:** 2026-02-20
**Feature:** `mcp-installation`

## Problem Statement

RepoManager currently manages the **contents** of tool config files (rules, custom instructions, MCP server JSON entries via `mcp.json`), but it does not manage the **actual installation** of MCP servers into each supported tool's native configuration. Extensions that ship MCP servers get their config resolved and merged, but the final step — writing the MCP server definition into each tool's own config file at the correct path, in the correct format, at the correct scope — is missing.

What we need: a dedicated interface to **add, remove, sync, and verify** MCP server installations across all enabled tool configurations, with proper support for project-scoped and user-scoped installations.

---

## Current Architecture

### What exists today

- **`repo-extensions/src/mcp.rs`**: Resolves `mcp.json` templates from extensions, merges multiple extension MCP configs into a single `serde_json::Value`
- **`repo-tools/src/integration.rs`**: `SyncContext` carries an `mcp_servers: Option<serde_json::Value>` field
- **`repo-tools/src/generic.rs`**: `sync_json()` writes MCP server entries to a tool's config file using `schema_keys.mcp_key`
- **`repo-meta/src/schema/tool.rs`**: `ToolCapabilities.supports_mcp` flag and `ToolSchemaKeys.mcp_key` field

### What's missing

1. Only tools with JSON configs and `mcp_key` get MCP entries written — tools with dedicated MCP config files (Cursor, Windsurf, Claude Code, etc.) are not handled
2. No awareness of **where** each tool stores its MCP config (which varies wildly per tool and OS)
3. No scope management (project vs user vs global)
4. No transport-type awareness (stdio vs HTTP vs SSE)
5. No verification that an MCP server was correctly installed
6. No removal/cleanup of previously installed MCP servers
7. Current `supports_mcp` flags are outdated — only 4 of 13 tools have `true`, but research shows **11 of 13** tools now support MCP

---

## Research: MCP Support Matrix

All 13 supported tools were investigated against official documentation. Below is the complete support matrix.

### MCP Support Summary

| Tool | Slug | MCP? | Transports | Project Scope | User Scope | Config Format |
|------|------|------|-----------|---------------|------------|---------------|
| **Claude Code** | `claude` | Yes | stdio, http, sse(dep) | `.mcp.json` | `~/.claude.json` | `mcpServers` |
| **Gemini CLI** | `gemini` | Yes | stdio, http, sse | `.gemini/settings.json` | `~/.gemini/settings.json` | `mcpServers` |
| **Cursor** | `cursor` | Yes | stdio, http, sse(dep) | `.cursor/mcp.json` | `~/.cursor/mcp.json` | `mcpServers` |
| **Windsurf** | `windsurf` | Yes | stdio, http, sse | `.windsurf/mcp.json` | `~/.codeium/windsurf/mcp_config.json` | `mcpServers` |
| **VS Code** | `vscode` | Yes | stdio, http, sse(dep) | `.vscode/mcp.json` | `<UserData>/mcp.json` | `servers` (note: not `mcpServers`) |
| **Antigravity** | `antigravity` | Yes | stdio, http, sse(fallback) | N/A (user-only) | `~/.gemini/antigravity/mcp_config.json` | `mcpServers` |
| **JetBrains** | `jetbrains` | Yes | stdio, http, sse | `.junie/mcp/mcp.json` | `~/.junie/mcp/mcp.json` | `mcpServers` |
| **Zed** | `zed` | Yes | stdio, http | `.zed/settings.json` | `~/.zed/settings.json` (or `~/.config/zed/settings.json`) | `context_servers` (not `mcpServers`) |
| **Cline** | `cline` | Yes | stdio, http, sse | N/A (global only) | VS Code globalStorage | `mcpServers` |
| **Roo Code** | `roo` | Yes | stdio, http, sse | `.roo/mcp.json` | VS Code globalStorage | `mcpServers` |
| **Amazon Q** | `amazonq` | Yes | stdio, http | `.amazonq/mcp.json` (CLI) / `.amazonq/default.json` (IDE) | `~/.aws/amazonq/mcp.json` (CLI) / `~/.aws/amazonq/default.json` (IDE) | `mcpServers` |
| **Aider** | `aider` | **No** | N/A | N/A | N/A | N/A |
| **GitHub Copilot** | `copilot` | Yes (via VS Code) | stdio, http, sse(dep) | `.vscode/mcp.json` | `<UserData>/mcp.json` | `servers` |

**Key:** dep = deprecated, (fallback) = auto-fallback from HTTP to SSE

### Critical Findings

1. **11 of 13 tools support MCP** — only Aider lacks native support
2. **Two naming conventions exist**: Most use `mcpServers` as the top-level key, but VS Code/Copilot uses `servers`, and Zed uses `context_servers`
3. **Scope support varies**: Most tools support both project and user scopes; Antigravity and Cline are user-only
4. **Transport types are universal**: stdio and Streamable HTTP are supported by all 11 MCP-capable tools; SSE is deprecated but still supported as fallback
5. **Config file locations are completely different** per tool and per OS — this is the core complexity

---

## Per-Tool MCP Configuration Details

### 1. Claude Code (`claude`)

**Project scope:** `<repo>/.mcp.json`
```json
{
  "mcpServers": {
    "server-name": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "package-name"],
      "env": { "KEY": "value" }
    }
  }
}
```

**User scope:** `~/.claude.json` (under project-specific keys or global `mcpServers`)

**Features:**
- Supports `${VAR}` and `${VAR:-default}` env var expansion in `.mcp.json`
- HTTP transport: `{ "type": "http", "url": "...", "headers": {...} }`
- Managed scope (admin): `/etc/claude-code/managed-mcp.json` (Linux)
- CLI: `claude mcp add/remove/list/get`

### 2. Gemini CLI (`gemini`)

**Project scope:** `<repo>/.gemini/settings.json`
```json
{
  "mcpServers": {
    "server-name": {
      "command": "python3",
      "args": ["-m", "server"],
      "env": { "KEY": "$HOST_ENV_VAR" }
    }
  }
}
```

**User scope:** `~/.gemini/settings.json`

**Features:**
- Uses `httpUrl` for Streamable HTTP (not `url`)
- Uses `url` for SSE
- `$VAR` / `${VAR}` expansion in env and headers
- `trust`, `includeTools`, `excludeTools` fields
- `timeout` field (ms, default 600,000)
- CLI: `gemini mcp add/remove/list`

### 3. Cursor (`cursor`)

**Project scope:** `<repo>/.cursor/mcp.json`
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "package-name"],
      "env": { "KEY": "${env:HOST_VAR}" }
    }
  }
}
```

**User scope:** `~/.cursor/mcp.json`

**Features:**
- Auto-infers transport from fields (no `type` field needed)
- `${env:VAR}` expansion syntax
- Remote: `{ "url": "...", "headers": {...} }`
- Deeplink installation: `cursor://anysphere.cursor-deeplink/mcp/install?...`
- Extension API: `vscode.cursor.mcp.registerServer()`

### 4. Windsurf (`windsurf`)

**Project scope:** `<repo>/.windsurf/mcp.json`
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "package-name"],
      "env": { "KEY": "value" }
    }
  }
}
```

**User scope:**
- macOS/Linux: `~/.codeium/windsurf/mcp_config.json`
- Windows: `%USERPROFILE%\.codeium\windsurf\mcp_config.json`

**Features:**
- Uses `serverUrl` (not `url`) for HTTP remotes
- `${env:VAR}` expansion syntax
- 100-tool limit across all servers
- MCP Marketplace for one-click install
- Enterprise whitelist support

### 5. VS Code / GitHub Copilot (`vscode`, `copilot`)

**Project scope:** `<repo>/.vscode/mcp.json`
```json
{
  "inputs": [
    { "type": "promptString", "id": "api-key", "description": "API Key", "password": true }
  ],
  "servers": {
    "server-name": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "package-name"],
      "env": { "KEY": "${input:api-key}" },
      "envFile": "${workspaceFolder}/.env"
    }
  }
}
```

**User scope:**
- macOS: `~/Library/Application Support/Code/User/mcp.json`
- Linux: `~/.config/Code/User/mcp.json`
- Windows: `%APPDATA%\Code\User\mcp.json`

**IMPORTANT: VS Code uses `"servers"` NOT `"mcpServers"`**

**Features:**
- `${input:...}` variable system for secrets (prompts user, stores encrypted)
- `${workspaceFolder}` path variables
- `envFile` field for `.env` file loading
- `type` field required: `"stdio"`, `"http"`, or `"sse"`
- `dev` block for auto-restart on file changes
- MCP Gallery for browsing/installing
- CLI: `code --add-mcp '{...}'`
- Auto-discovery from other clients: `chat.mcp.discovery.enabled`

### 6. Antigravity (`antigravity`)

**User scope only:** `~/.gemini/antigravity/mcp_config.json`
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "package-name"],
      "env": { "KEY": "value" }
    }
  }
}
```

**Features:**
- No project-level scope (user-only, feature requested)
- Uses `serverUrl` (not `url`) for HTTP remotes
- Built-in MCP Store with 1,500+ servers
- No env var interpolation in config
- 50-tool recommended limit

### 7. JetBrains (`jetbrains`)

**Project scope (Junie):** `<repo>/.junie/mcp/mcp.json`
```json
{
  "mcpServers": {
    "server-name": {
      "type": "command",
      "command": "npx",
      "args": ["package-name"],
      "env": { "KEY": "value" },
      "enabled": true
    }
  }
}
```

**User scope (Junie):** `~/.junie/mcp/mcp.json`

**Features:**
- Junie uses `"type": "command"` for stdio and `"type": "url"` for HTTP
- AI Assistant uses Settings UI (not file-based config)
- IDE can itself be an MCP server (2025.2+)
- Import from Claude Desktop built-in
- `enabled` boolean field per server

### 8. Zed (`zed`)

**Project scope:** `<repo>/.zed/settings.json`
```json
{
  "context_servers": {
    "server-name": {
      "source": "custom",
      "command": "some-command",
      "args": ["arg1"],
      "env": { "KEY": "value" }
    }
  }
}
```

**User scope:**
- macOS: `~/.zed/settings.json`
- Linux: `~/.config/zed/settings.json`

**IMPORTANT: Zed uses `"context_servers"` NOT `"mcpServers"`**

**Features:**
- Extension overrides use nested `"command": { "path": "...", "args": [...], "env": {...} }`
- HTTP remote: `{ "url": "http://...", "enabled": true }`
- Worktree trust security for project configs
- Tool permissions: `agent.tool_permissions` with `mcp:<server>:<tool>` keys

### 9. Cline (`cline`)

**User scope only:** VS Code globalStorage (`saoudrizwan.claude-dev`), file `cline_mcp_settings.json`
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "package-name"],
      "env": { "KEY": "value" },
      "alwaysAllow": [],
      "disabled": false
    }
  }
}
```

**Features:**
- No project-level scope (global only)
- MCP Marketplace for one-click install
- `alwaysAllow` array for auto-approved tools
- `disabled` boolean field
- Config lives in VS Code extension storage, not easily user-editable

### 10. Roo Code (`roo`)

**Project scope:** `<repo>/.roo/mcp.json`
```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "package-name"],
      "env": { "KEY": "value" },
      "alwaysAllow": [],
      "disabled": false,
      "timeout": 60,
      "cwd": "/path",
      "disabledTools": [],
      "watchPaths": []
    }
  }
}
```

**User scope:** VS Code globalStorage (`rooveterinaryinc.roo-cline`), file `cline_mcp_settings.json`

**Features:**
- `${env:VAR}` expansion in args
- `cwd`, `watchPaths`, `disabledTools` fields
- HTTP: `{ "type": "streamable-http", "url": "...", "headers": {...} }`
- SSE: `{ "type": "sse", "url": "...", "headers": {...} }`
- Project-level overrides global when same name

### 11. Amazon Q (`amazonq`)

**Project scope (CLI):** `<repo>/.amazonq/mcp.json`
**Project scope (IDE):** `<repo>/.amazonq/default.json`
```json
{
  "mcpServers": {
    "server-name": {
      "command": "uvx",
      "args": ["package@latest"],
      "env": { "KEY": "value" },
      "timeout": 60000,
      "disabled": false,
      "autoApprove": ["tool1"]
    }
  }
}
```

**User scope (CLI):** `~/.aws/amazonq/mcp.json`
**User scope (IDE):** `~/.aws/amazonq/default.json`

**Features:**
- HTTP: `{ "type": "http", "url": "..." }`
- `autoApprove` array for tool-level permissions
- `disabled` field
- CLI: `q mcp add/remove/list/status/import`
- Enterprise governance via IAM Identity Center

### 12. Aider (`aider`) — No MCP Support

Aider does not support MCP. Feature requests exist (Issue #2525, #4506) but no implementation has been merged. Third-party bridges exist to expose Aider *as* an MCP tool to other clients, but Aider itself cannot consume MCP servers.

---

## Proposed Design

### Core Concept: `McpInstaller` Trait

A new trait that each tool implements to describe how MCP servers should be installed into that tool's configuration:

```rust
/// Describes how a tool stores MCP server configuration.
pub struct McpConfigSpec {
    /// Top-level JSON key for servers (e.g., "mcpServers", "servers", "context_servers")
    pub servers_key: &'static str,

    /// Project-scope config path relative to repo root (None if no project scope)
    pub project_config_path: Option<&'static str>,

    /// User-scope config path (OS-dependent, resolved at runtime)
    /// None if tool only supports project scope
    pub user_config_path: Option<McpUserConfigPath>,

    /// Config file format
    pub format: McpConfigFormat,

    /// Whether the MCP config is embedded in a larger settings file
    /// or a standalone file dedicated to MCP
    pub embedding: McpConfigEmbedding,

    /// Supported transport types
    pub transports: Vec<McpTransport>,

    /// Tool-specific field mappings (e.g., "serverUrl" vs "url" vs "httpUrl")
    pub field_mappings: McpFieldMappings,

    /// Environment variable interpolation syntax (e.g., "${env:VAR}", "$VAR", "${VAR:-default}")
    pub env_interpolation: Option<EnvInterpolationSyntax>,
}
```

### Scope Model

```rust
pub enum McpScope {
    /// Project-level: config stored in repo, can be committed to VCS
    Project,
    /// User-level: config stored in user's home dir, available across projects
    User,
}
```

### Config Format Variants

```rust
pub enum McpConfigFormat {
    /// Standalone JSON file (e.g., `.cursor/mcp.json`, `.mcp.json`)
    StandaloneJson,
    /// Embedded in a larger JSON settings file (e.g., Zed's `settings.json`)
    EmbeddedJson { parent_key: Option<&'static str> },
    /// VS Code extension storage (not directly user-editable)
    ExtensionStorage { extension_id: &'static str, filename: &'static str },
}

pub enum McpConfigEmbedding {
    /// File contains only MCP config
    Dedicated,
    /// MCP config is nested inside a larger config file
    Nested { path: Vec<&'static str> },
}
```

### User Config Path Resolution

```rust
pub enum McpUserConfigPath {
    /// Simple path relative to home directory
    HomeRelative(&'static str),
    /// OS-specific paths
    OsSpecific {
        macos: &'static str,
        linux: &'static str,
        windows: &'static str,
    },
    /// VS Code extension global storage
    VsCodeExtensionStorage {
        extension_id: &'static str,
        filename: &'static str,
    },
}
```

### Field Mapping (handles naming differences)

```rust
pub struct McpFieldMappings {
    /// Field name for HTTP URL ("url", "serverUrl", "httpUrl")
    pub http_url_field: &'static str,
    /// Field name for SSE URL (if different from http_url_field)
    pub sse_url_field: Option<&'static str>,
    /// Whether an explicit "type" field is needed
    pub requires_type_field: bool,
    /// Type field values for each transport
    pub type_values: McpTypeValues,
}

pub struct McpTypeValues {
    pub stdio: Option<&'static str>,       // e.g., "stdio", "command", or None (inferred)
    pub http: Option<&'static str>,        // e.g., "http", "streamable-http"
    pub sse: Option<&'static str>,         // e.g., "sse"
}
```

### Operations

```rust
pub trait McpInstaller {
    /// Return the MCP config specification for this tool
    fn mcp_config_spec(&self) -> Option<McpConfigSpec>;

    /// Install an MCP server definition into this tool's config
    fn install_mcp_server(
        &self,
        scope: McpScope,
        server_name: &str,
        server_config: &McpServerConfig,
        root: &NormalizedPath,
    ) -> Result<()>;

    /// Remove an MCP server definition from this tool's config
    fn remove_mcp_server(
        &self,
        scope: McpScope,
        server_name: &str,
        root: &NormalizedPath,
    ) -> Result<()>;

    /// List all MCP servers currently installed in this tool's config
    fn list_mcp_servers(
        &self,
        scope: McpScope,
        root: &NormalizedPath,
    ) -> Result<Vec<(String, serde_json::Value)>>;

    /// Verify that a server definition exists and is well-formed
    fn verify_mcp_server(
        &self,
        scope: McpScope,
        server_name: &str,
        root: &NormalizedPath,
    ) -> Result<McpVerifyResult>;

    /// Sync all MCP servers from the canonical source into this tool's config
    fn sync_mcp_servers(
        &self,
        scope: McpScope,
        servers: &serde_json::Value,
        root: &NormalizedPath,
    ) -> Result<McpSyncResult>;
}
```

### Canonical MCP Server Definition

The user-facing, tool-agnostic MCP server definition:

```rust
pub struct McpServerConfig {
    /// Transport configuration
    pub transport: McpTransportConfig,
    /// Environment variables
    pub env: Option<BTreeMap<String, String>>,
    /// Whether to auto-approve all tools
    pub auto_approve: bool,
}

pub enum McpTransportConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        cwd: Option<String>,
    },
    Http {
        url: String,
        headers: Option<BTreeMap<String, String>>,
    },
    Sse {
        url: String,
        headers: Option<BTreeMap<String, String>>,
    },
}
```

### Translation Layer

Each tool needs a translator that converts the canonical `McpServerConfig` into the tool's native JSON format:

```rust
fn to_tool_json(
    config: &McpServerConfig,
    spec: &McpConfigSpec,
) -> serde_json::Value {
    // Handle field naming differences:
    //   - "url" vs "serverUrl" vs "httpUrl"
    //   - "type" field presence/absence
    //   - "type" field values ("stdio"/"http"/"sse" vs "command"/"url")
    //   - Extra tool-specific fields (alwaysAllow, disabled, timeout, etc.)
}
```

### Tool Registration Updates

Each tool's `ToolDefinition` needs expanded MCP metadata. The `ToolSchemaKeys.mcp_key` approach is insufficient — we need the full `McpConfigSpec`:

```rust
// In each tool module, add mcp_config_spec():

// cursor.rs
pub fn cursor_mcp_spec() -> McpConfigSpec {
    McpConfigSpec {
        servers_key: "mcpServers",
        project_config_path: Some(".cursor/mcp.json"),
        user_config_path: Some(McpUserConfigPath::HomeRelative(".cursor/mcp.json")),
        format: McpConfigFormat::StandaloneJson,
        embedding: McpConfigEmbedding::Dedicated,
        transports: vec![McpTransport::Stdio, McpTransport::Http, McpTransport::Sse],
        field_mappings: McpFieldMappings {
            http_url_field: "url",
            sse_url_field: None,
            requires_type_field: false, // auto-inferred
            type_values: McpTypeValues::default(),
        },
        env_interpolation: Some(EnvInterpolationSyntax::DollarEnvColon), // ${env:VAR}
    }
}
```

### CLI Interface

```
repo mcp install <server-name> --tool <tool> --scope <project|user> [--transport stdio|http] -- <command> [args...]
repo mcp install <server-name> --tool <tool> --scope <project|user> --url <url>
repo mcp remove <server-name> --tool <tool> --scope <project|user>
repo mcp list [--tool <tool>] [--scope <project|user>]
repo mcp verify [--tool <tool>]
repo mcp sync [--scope <project|user>]
```

### Sync Behavior

`repo mcp sync` should:
1. Read the canonical MCP server definitions from `.repository/` config
2. For each enabled tool that supports MCP:
   a. Translate the canonical config to the tool's native format
   b. Read the tool's existing MCP config file
   c. Upsert managed servers (identified by a naming convention or metadata)
   d. Preserve user-added servers that aren't managed by RepoManager
3. Report what was added/updated/removed

---

## Flags to Update (`supports_mcp`)

Current vs correct values:

| Tool | Current | Correct | Action |
|------|---------|---------|--------|
| `claude` | `true` | `true` | No change |
| `gemini` | `false` | **`true`** | **Update** |
| `cursor` | `false` | **`true`** | **Update** |
| `windsurf` | `false` | **`true`** | **Update** |
| `vscode` | `false` | **`true`** | **Update** |
| `antigravity` | `false` | **`true`** | **Update** |
| `jetbrains` | `true` | `true` | No change |
| `zed` | `true` | `true` | No change |
| `cline` | `false` | **`true`** | **Update** |
| `roo` | `true` | `true` | No change |
| `copilot` | `false` | **`true`** | **Update** |
| `amazonq` | `false` | **`true`** | **Update** |
| `aider` | `false` | `false` | No change |

**8 tools need `supports_mcp` set to `true`.**

---

## Implementation Phases

### Phase 1: Update `supports_mcp` flags and add `McpConfigSpec`
- Update all 8 outdated `supports_mcp` flags
- Define `McpConfigSpec` struct and implement for all 11 MCP-capable tools
- Add `mcp_config_path()` methods that resolve OS-specific paths

### Phase 2: Core `McpInstaller` implementation
- Implement the generic JSON read/merge/write logic
- Handle the three naming conventions (`mcpServers`, `servers`, `context_servers`)
- Handle standalone vs embedded configs
- Handle the field mapping differences (`url` vs `serverUrl` vs `httpUrl`)

### Phase 3: CLI commands (`repo mcp install/remove/list/verify/sync`)
- Wire up CLI subcommands
- Implement scope resolution
- Add `--all-tools` flag for broadcasting an MCP server to all enabled tools

### Phase 4: Extension integration
- Connect extension MCP configs to the installation pipeline
- On `repo sync`, automatically install extension-provided MCP servers into all enabled tools

---

## Open Questions

1. **Managed server identification**: How do we distinguish RepoManager-managed MCP servers from user-added ones in a tool's config? Options:
   - Server name prefix (e.g., `repo:server-name`)
   - Separate tracking file (`.repository/mcp-state.json`)
   - Comment/metadata field if the tool supports it

2. **VS Code extension storage tools (Cline)**: Cline stores its config in VS Code's extension globalStorage, which is not a user-facing file path. Should we support writing to this location, or skip Cline for user-scope installations?

3. **Conflicting scopes**: If a server is installed at both project and user scope, most tools resolve project > user. Should RepoManager enforce this or just install as requested?

4. **Env var interpolation**: Different tools use different syntaxes (`${VAR}`, `$VAR`, `${env:VAR}`, `${VAR:-default}`). Should we normalize to one syntax in the canonical config and translate, or pass through raw?

5. **Claude Desktop**: Claude Desktop (the GUI app) has a separate config from Claude Code (the CLI). Should we treat them as one tool or two? Currently the codebase has a single `claude` slug.

---

## References

All findings are based on official documentation research conducted 2026-02-20:

- [Claude Code MCP Docs](https://code.claude.com/docs/en/mcp)
- [MCP Specification](https://modelcontextprotocol.io/specification/2025-03-26/basic/transports)
- [Gemini CLI MCP Server Docs](https://google-gemini.github.io/gemini-cli/docs/tools/mcp-server.html)
- [Cursor MCP Docs](https://docs.cursor.com/context/model-context-protocol)
- [Windsurf MCP Docs](https://docs.windsurf.com/windsurf/cascade/mcp)
- [VS Code MCP Docs](https://code.visualstudio.com/docs/copilot/customization/mcp-servers)
- [Antigravity MCP Docs](https://antigravity.google/docs/mcp)
- [JetBrains AI Assistant MCP Docs](https://www.jetbrains.com/help/ai-assistant/mcp.html)
- [Zed MCP Docs](https://zed.dev/docs/ai/mcp)
- [Roo Code MCP Docs](https://docs.roocode.com/features/mcp/using-mcp-in-roo)
- [Amazon Q Developer MCP Docs](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/qdev-mcp.html)
- [Aider GitHub Issue #2525](https://github.com/Aider-AI/aider/issues/2525)
