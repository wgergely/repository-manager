# Other Agentic Coding Tools

Overview of additional AI coding tools beyond the major players.

## Aider

**Type**: CLI Tool
**Focus**: Terminal-based pair programming

### Overview

| Attribute | Value |
|-----------|-------|
| **Type** | CLI |
| **Models** | Multiple (Claude, GPT-4, local) |
| **AGENTS.md** | Native |
| **MCP Support** | No |

### Configuration

```yaml
# .aider.conf.yml
model: claude-3-5-sonnet
edit-format: diff
auto-commits: true
dirty-commits: true
```

### Unique Features
- Git-native: Automatic commit generation
- Model-agnostic: Works with many providers
- CLI-focused: No IDE dependency
- Voice mode: Speak to code

---

## Continue.dev

**Type**: Open-source IDE Extension
**Focus**: Customizable AI assistant

### Overview

| Attribute | Value |
|-----------|-------|
| **Type** | IDE Extension (VS Code, JetBrains) |
| **Models** | Multiple providers |
| **AGENTS.md** | Not confirmed |
| **MCP Support** | Partial (via context providers) |

### Configuration

```json
// ~/.continue/config.json
{
  "models": [
    {
      "title": "Claude 3.5 Sonnet",
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022"
    }
  ],
  "customCommands": [
    {
      "name": "test",
      "prompt": "Write unit tests for the selected code"
    }
  ],
  "contextProviders": [
    {"name": "code", "params": {}},
    {"name": "docs", "params": {}}
  ]
}
```

### Unique Features
- Open source
- Highly customizable
- Custom slash commands
- Context provider system

---

## Cody (Sourcegraph)

**Type**: IDE Extension
**Focus**: Codebase-aware assistant

### Overview

| Attribute | Value |
|-----------|-------|
| **Type** | IDE Extension |
| **Models** | Claude (default) |
| **AGENTS.md** | Not confirmed |
| **MCP Support** | Partial |

### Unique Features
- Sourcegraph integration
- Enterprise code search
- Cross-repository context
- Security compliance features

---

## JetBrains AI Assistant

**Type**: IDE Plugin
**Focus**: JetBrains IDE integration

### Overview

| Attribute | Value |
|-----------|-------|
| **Type** | IDE Plugin |
| **Models** | JetBrains proprietary + third-party |
| **AGENTS.md** | Not confirmed |
| **MCP Support** | Partial (server support) |

### Configuration

Via JetBrains IDE settings and `.idea/` directory.

### Unique Features
- Deep JetBrains integration
- Refactoring awareness
- Test generation
- Multi-language support

---

## Tabnine

**Type**: IDE Extension
**Focus**: Local/private AI completions

### Overview

| Attribute | Value |
|-----------|-------|
| **Type** | IDE Extension |
| **Models** | Proprietary + local |
| **Focus** | Privacy-first |
| **MCP Support** | No |

### Unique Features
- Runs locally option
- Privacy-focused
- Multiple IDE support
- Team training capability

---

## Factory AI

**Type**: Autonomous Coding Agent
**Focus**: Issue-to-code automation

### Overview

| Attribute | Value |
|-----------|-------|
| **Type** | Autonomous Agent |
| **AGENTS.md** | Native |
| **Focus** | Full automation |

### Unique Features
- Watches issues/tickets
- Autonomous implementation
- PR creation
- Minimal human intervention

---

## OpenAI Codex

**Type**: API + Editor Integration
**Focus**: General AI coding

### Overview

| Attribute | Value |
|-----------|-------|
| **Company** | OpenAI |
| **AGENTS.md** | Native (primary format) |
| **MCP Support** | Native |

### Configuration

AGENTS.md is the primary configuration format.

### Unique Features
- First-party AGENTS.md support
- MCP adoption (March 2025)
- API-first approach

---

## Google Jules

**Type**: Autonomous Coding Agent
**Focus**: Asynchronous coding tasks

### Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Google |
| **AGENTS.md** | Native (primary format) |
| **Focus** | Background automation |

### Unique Features
- Works in background
- Issue-based triggering
- Full PR workflow
- Google Cloud integration

---

## Comparison Matrix

| Tool | Type | AGENTS.md | MCP | Primary Use |
|------|------|-----------|-----|-------------|
| Aider | CLI | Native | No | Terminal pair programming |
| Continue | Extension | Partial | Partial | Customizable assistant |
| Cody | Extension | No | Partial | Enterprise code search |
| JetBrains AI | Plugin | No | Partial | JetBrains integration |
| Tabnine | Extension | No | No | Privacy-first completions |
| Factory AI | Agent | Native | Partial | Full automation |
| OpenAI Codex | API | Native | Native | API-first coding |
| Google Jules | Agent | Native | Partial | Async background coding |

---

*Last updated: 2026-01-23*
*Status: Complete*
