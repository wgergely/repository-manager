# ADR-004: MCP Server Integration

**Status:** Approved (decisions confirmed)
**Date:** 2026-02-19
**Context:** Integrating extension-provided MCP servers with tool configurations

---

## Context

Extensions like VaultSpec provide MCP servers that need to be registered in tool configurations (Claude, Cursor, etc.). The repo manager already has dormant MCP infrastructure (Phase 5) that needs to be activated and connected to the extension system.

## Decisions

### 4.1 MCP Declaration

**Decision: Reference external file.**

Extension manifest declares `provides.mcp = "mcp.json"`. The repo manager reads the MCP configuration from the referenced file in the extension source, resolves runtime paths, and feeds it to the `ToolCapabilitySyncer`.

**VaultSpec's mcp.json:**
```json
{
  "mcpServers": {
    "vs-subagent-mcp": {
      "command": "python",
      "args": [".vaultspec/lib/scripts/subagent.py", "serve", "--root", "."],
      "env": {}
    }
  }
}
```

**Rejected alternatives:**
- Inline in manifest: duplication with existing mcp.json
- Both: unnecessary complexity

### 4.2 Command Resolution

**Decision: Automatic resolution with template escape hatch.**

The repo manager resolves the MCP server command automatically:
1. Reads `mcp.json` from extension
2. Replaces `"command": "python"` with the extension's venv Python path
3. Resolves `--root .` to the actual container/repo root
4. Writes resolved config to each tool's MCP configuration

Template variables (`{{runtime.python}}`, `{{root}}`) available as escape hatch for non-standard commands.

**VaultSpec resolved example:**
```json
{
  "vs-subagent-mcp": {
    "command": "/path/to/.repository/extensions/vault-spec/.venv/bin/python",
    "args": [".vaultspec/lib/scripts/subagent.py", "serve", "--root", "/path/to/container"],
    "env": {}
  }
}
```

### 4.3 MCP Process Supervision

**Decision: Tools launch their own servers.**

The repo manager's job is configuration sync, not process management. It writes resolved MCP config into each tool's config file via the existing `ToolCapabilitySyncer`. Claude Desktop, Cursor, and other tools launch the MCP server processes themselves.

**Rejected alternatives:**
- Repo manager supervises: adds daemon complexity, outside scope
- Hybrid: unnecessary for current tool landscape

## Implementation Path

### Phase 5 Activation
The existing MCP infrastructure needs:
1. Uncomment `CapabilityTranslator::translate()` MCP block
2. Extension system provides `serde_json::Value` MCP config
3. `TranslatedContent.mcp_servers` gets populated
4. `JsonWriter` places at `schema_keys.mcp_key`
5. Per-tool MCP config format handlers for non-JSON tools

### VaultSpec MCP Server Details
- **Server name:** `vs-subagent-mcp`
- **5 MCP tools:** list_agents, dispatch_agent, get_task_status, cancel_task, get_locks
- **Dynamic resources:** `agents://{name}` for each agent file
- **Startup requires:** `root_dir` (from `--root` arg or `VAULTSPEC_MCP_ROOT_DIR` env var)

### Tools With MCP Support
Currently 4 tools declare `supports_mcp = true`: claude, roo, jetbrains, zed. The resolved MCP config would only be written to these tools' configurations.

## Consequences

- Phase 5 MCP code activated in `CapabilityTranslator`
- Extension MCP configs resolved at sync time
- Tool config files updated with MCP server entries in managed blocks
- MCP server processes managed by the consuming tools, not repo manager
- `VAULTSPEC_MCP_ROOT_DIR` env var set to container root in resolved config
