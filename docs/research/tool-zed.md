# Zed Editor

High-performance, Rust-native code editor with integrated AI capabilities.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Zed Industries |
| **Language** | Rust (native, not Electron) |
| **Type** | Desktop IDE |
| **Models** | Claude 3.5 Sonnet (default), configurable |
| **MCP Support** | Full Native |
| **AGENTS.md** | Native |

## Architecture

Built entirely in Rust using GPUI framework:
- Native performance (not Electron)
- GPU-accelerated rendering
- ~10x faster than VS Code on benchmarks
- Low memory footprint

## Configuration Files

### .zed/settings.json

Primary configuration file.

```json
{
  "assistant": {
    "enabled": true,
    "default_model": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022"
    }
  },
  "language_models": {
    "anthropic": {
      "api_key": "..."
    }
  },
  "mcp_servers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
    }
  }
}
```

### settings.json Location

- Project: `.zed/settings.json`
- User: `~/.config/zed/settings.json` (Linux/macOS)

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Inline completions | Full | Via configured models |
| Chat panel | Full | Assistant panel |
| Multi-file editing | Full | Native multi-buffer |
| Terminal access | Full | Integrated terminal |
| Autonomous coding | Partial | Growing agentic features |
| Git operations | Full | Built-in git UI |
| MCP | Full | Native client support |

## AI Features

### Assistant Panel

- Chat interface with AI models
- Inline editing suggestions
- Code explanation
- Multi-model support

### Inline Assists

- Context-aware completions
- Function generation
- Refactoring suggestions

## Model Configuration

Supports multiple providers:

```json
{
  "language_models": {
    "anthropic": {
      "api_key": "your-key"
    },
    "openai": {
      "api_key": "your-key"
    },
    "ollama": {
      "api_url": "http://localhost:11434"
    }
  }
}
```

## MCP Integration

Native MCP client support:

```json
{
  "mcp_servers": {
    "database": {
      "command": "npx",
      "args": ["-y", "@mcp/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://..."
      }
    }
  }
}
```

## Context Management

- Project-level settings
- Open buffer context
- MCP-provided resources
- User preferences

## Memory/Persistence

| Type | Persistence | Format |
|------|-------------|--------|
| Session | Editor state | Internal |
| Project | Via settings | JSON |
| User | Via user config | JSON |

## Pricing

| Tier | Price | Features |
|------|-------|----------|
| Free | $0 | Core editor, no AI |
| Pro | $20/month | AI features included |

## Unique Differentiators

1. **Native Performance**: Rust-based, GPU-accelerated
2. **Multiplayer**: Real-time collaboration built-in
3. **Model Agnostic**: Multiple AI providers supported
4. **Open Source**: Core editor is open source
5. **Low Resource**: Minimal memory footprint

## Limitations

- Smaller extension ecosystem than VS Code
- Younger product (less mature)
- macOS-first (Linux support growing)
- AI features require Pro subscription

## Quick Reference

```
./.zed/
└── settings.json              # Project settings (AI, MCP, editor)
./AGENTS.md                    # Universal format (supported)
~/.config/zed/
└── settings.json              # User settings
```

---

*Last updated: 2026-01-23*
*Status: Complete*
