# Model Context Protocol (MCP) - Tool Integration Standard

The most significant tool integration standard in the agentic coding space.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Anthropic (open source) |
| **Current Version** | 2025-11-25 |
| **Analogy** | "USB-C for AI applications" |
| **Ecosystem** | 100+ servers, SDKs for Python/TypeScript/Rust/Go |
| **Specification** | https://modelcontextprotocol.io/specification/2025-11-25 |

## What is MCP?

MCP is a standardized protocol for connecting AI applications (clients) to external systems (servers). It enables:

- **Shared Tool Definitions**: One MCP server serves multiple AI clients
- **Universal Data Access**: Databases, filesystems, APIs accessible to any MCP-compatible tool
- **Plugin Portability**: MCP servers work across Claude Code, Cursor, Windsurf, etc.
- **Context Sharing**: Resources and prompts defined once, used everywhere

## Architecture

```
AI Application (Client)             External Systems (MCP Servers)
┌─────────────────────┐             ┌─────────────────────┐
│   Claude Code       │             │   Databases         │
│   Cursor            │◄──────────►│   File Systems      │
│   Windsurf          │   MCP       │   APIs              │
│   Custom Apps       │  Protocol   │   Specialized Tools │
└─────────────────────┘             └─────────────────────┘
```

## Specification Versions

| Version | Release Date | Key Changes |
|---------|--------------|-------------|
| 2024-11-05 | November 2024 | Initial release |
| 2025-03-26 | March 2025 | Transport layer improvements |
| 2025-06-18 | June 2025 | Authorization enhancements (OAuth 2.0) |
| **2025-11-25** | November 2025 | **Current stable** - Tasks, improved OAuth, extensions |

## MCP 2025-11-25 Key Features

### 1. Tasks Primitive (Async Execution)

Any request can return a task handle for "call-now, fetch-later":

```json
{
  "taskId": "task_abc123",
  "state": "working"
}
```

Task states: `working`, `input_required`, `completed`, `failed`, `cancelled`

Enables multi-step operations and long-running processes.

### 2. Improved OAuth & Authorization

- **CIMD** (Client ID Metadata Documents) as default registration method
- **PKCE mandatory** (must use S256 code challenge)
- Removes Dynamic Client Registration complexity

### 3. Server Discovery

Servers can publish identity documents at `.well-known` URLs:
- Enables discovery without connecting first
- Simplifies enterprise deployments

### 4. Standardized Tool Names (SEP-986)

Single canonical format for tool naming:
- Consistent display, sorting, referencing across SDKs
- Example: `mcp://server-name/tool-name`

### 5. Protocol Extensions

Official support for industry-specific extensions:
- Curated patterns for healthcare, finance, education domains

## Adoption Status (January 2026)

| Tool | MCP Support | Notes |
|------|-------------|-------|
| **Claude Code** | Full Native | First-class citizen, settings.json config |
| **Claude Desktop** | Full Native | Full client support |
| **Cursor** | Full Native | 40 tool limit, one-click install |
| **Windsurf** | Full Native | Native client support |
| **Zed** | Full Native | Native client support |
| **Amazon Q** | Native | MCP configuration in IDE settings |
| **OpenAI** | Native | Adopted March 2025 |
| **Google DeepMind** | Native | Confirmed adoption |
| **Continue.dev** | Partial | Via context providers |
| **VS Code (Copilot)** | Limited | Experimental support |
| **JetBrains AI** | Partial | Server support |

## Configuration Examples

### Claude Code

```json
// .claude/settings.json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/data"]
    },
    "postgres": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://localhost/mydb"
      }
    }
  }
}
```

### Cursor

```json
// Cursor settings
{
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@context7/mcp-server"]
    }
  }
}
```

## MCP Primitives

| Primitive | Purpose | Key Fields |
|-----------|---------|------------|
| Resources | Read-only data | uri, name, mimeType |
| Tools | Callable functions | name, description, inputSchema |
| Prompts | Reusable templates | name, description, arguments |

See [MCP Spec](https://modelcontextprotocol.io/specification/2025-11-25) for full schemas.

## Common MCP Servers

| Server | Purpose | Package |
|--------|---------|---------|
| Filesystem | File operations | `@modelcontextprotocol/server-filesystem` |
| PostgreSQL | Database access | `@modelcontextprotocol/server-postgres` |
| SQLite | Database access | `@modelcontextprotocol/server-sqlite` |
| GitHub | Repository access | `@modelcontextprotocol/server-github` |
| Brave Search | Web search | `@modelcontextprotocol/server-brave-search` |
| Memory | Persistent memory | `@modelcontextprotocol/server-memory` |
| Puppeteer | Browser automation | `@modelcontextprotocol/server-puppeteer` |
| Slack | Team communication | `@modelcontextprotocol/server-slack` |

## Governance

MCP follows a formal enhancement proposal process:

| SEP Number | Focus Area |
|------------|------------|
| SEP-932 | MCP Governance framework |
| SEP-973 | Metadata standards for Resources, Tools, Prompts |
| SEP-986 | Tool naming format specifications |
| SEP-985 | OAuth 2.0 Protected Resource Metadata |
| SEP-990 | Enterprise IdP policy controls |
| SEP-1046 | OAuth client credentials flow |

## Significance for repo-manager

MCP integration should provide:

1. **Server Registration**: Centralized MCP server configuration
2. **Tool Sync**: Propagate MCP config to all tool-specific formats
3. **Discovery**: Help users find and configure MCP servers
4. **Validation**: Verify MCP server connectivity and capabilities

### Integration Strategy

```
.repository/mcp/servers.yaml  (source of truth)
        |
        v
  [repo-manager sync]
        |
   +----+----+----+
   v    v    v    v
.claude/settings.json  Cursor MCP  Windsurf MCP  Zed config
```

## Sources

- [MCP Specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [MCP Blog - 2025-11-25 Release](https://blog.modelcontextprotocol.io/posts/2025-11-25-first-mcp-anniversary/)
- [WorkOS MCP Analysis](https://workos.com/blog/mcp-2025-11-25-spec-update)
- [Cursor MCP Documentation](https://cursor.com/docs/context/mcp)

---

*Last updated: 2026-01-23*
*Status: Complete*
