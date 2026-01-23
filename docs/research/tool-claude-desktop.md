# Claude Desktop (Anthropic)

Anthropic's native desktop application with MCP support and bundled Claude Code.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Anthropic |
| **Type** | Native Desktop App (Electron) |
| **Models** | Claude (exclusive) |
| **MCP Support** | Full Native |
| **Bundled** | Claude Code CLI (may lag latest version) |

## Distinction from Claude Code

| Aspect | Claude Desktop | Claude Code |
|--------|---------------|-------------|
| **Interface** | GUI application | CLI terminal |
| **Primary use** | General assistance, MCP tools | Development tasks |
| **Config location** | App Support/Roaming | Project `.claude/` |
| **CLAUDE.md** | Not read automatically | Read automatically |
| **Skills upload** | .zip via Settings UI | Directory-based |
| **Project context** | Manual / MCP | Automatic filesystem |

## Configuration Files

### Main Configuration

**Location**:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
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

### Accessing Settings

1. Click Claude menu in system menu bar
2. Select "Settings…"
3. Navigate to "Developer" tab
4. Click "Edit Config" to open configuration file

## Desktop Extensions

### Extension Format

Desktop Extensions use `.mcpb` (MCP Bundle) format:
- Previous format: `.dxt` (still supported)
- Recommended: `.mcpb` for new extensions

### Extension Structure

```
my-extension.mcpb/
└── manifest.json    # Required
```

**Features**:
- Built-in Node.js runtime (ships with Claude Desktop)
- Automatic updates
- Secure secrets stored in OS keychain
- One-click installation

### manifest.json Example

```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "description": "My MCP extension",
  "server": {
    "command": "node",
    "args": ["server.js"]
  }
}
```

## Skills in Claude Desktop

### Upload Method

Unlike Claude Code's directory-based skills:
1. Package skill as `.zip` file
2. Open Settings in Claude Desktop
3. Upload via Skills UI

**Note**: Same upload mechanism as claude.ai web interface.

### Skills Location Comparison

| Product | Location | Method |
|---------|----------|--------|
| Claude Code | `~/.claude/skills/` | Directory |
| Claude Desktop | N/A | .zip upload via UI |
| claude.ai | N/A | .zip upload via web |

## MCP Configuration

### Syntax Difference

| Product | Key Path |
|---------|----------|
| Claude Desktop, Cursor | `mcpServers` |
| VS Code | `mcp.servers` |

### Node.js Requirements

- Minimum: Node.js 22
- Claude Desktop may use wrong Node version - verify in terminal
- Restart required after config changes

## Bundled Claude Code

Claude Desktop includes Claude Code CLI, but:
- May not be latest version (Desktop prioritizes stability)
- Configuration still uses project `.claude/` directory
- CLAUDE.md read when using Claude Code features

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| MCP servers | Full | Native support |
| Desktop Extensions | Full | .mcpb format |
| Skills | Yes | Upload via UI |
| Claude Code CLI | Bundled | May lag CLI version |
| Filesystem access | Via MCP | Not direct |
| Project context | Limited | Requires MCP or upload |

## Troubleshooting

| Issue | Solution |
|-------|----------|
| MCP server not connecting | Check file paths are absolute |
| Wrong Node.js version | Set PATH or use absolute path |
| Config changes not applied | Restart Claude Desktop |
| Extension not loading | Verify manifest.json format |

## Quick Reference

```
# macOS
~/Library/Application Support/Claude/
└── claude_desktop_config.json    # Main config

# Windows
%APPDATA%\Claude\
└── claude_desktop_config.json    # Main config

# Config structure
{
  "mcpServers": {
    "server-name": {
      "command": "...",
      "args": [...],
      "env": {...}
    }
  }
}

# Desktop Extension
extension.mcpb/
└── manifest.json
```

## Sources

- [Claude Code Docs - Desktop Integration](https://code.claude.com/docs/en/desktop)
- [MCP - Connect Local Servers](https://modelcontextprotocol.io/docs/develop/connect-local-servers)
- [Anthropic - Desktop Extensions](https://www.anthropic.com/engineering/desktop-extensions)
- [MCP Quickstart](https://modelcontextprotocol.info/docs/quickstart/user/)

---

*Last updated: 2026-01-23*
*Status: Complete*
