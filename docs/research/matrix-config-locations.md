# Configuration File Locations

Quick reference for where each tool stores configuration.

## Summary Table

| Tool | Primary Config | MCP Config | User Config |
|------|---------------|------------|-------------|
| **Claude Code** | `CLAUDE.md` | `.claude/settings.json` | `~/.claude/` |
| **Claude Desktop** | `claude_desktop_config.json` | Same file | App Support/Roaming |
| **Cursor** | `.cursorrules` | Cursor settings | `~/.cursor/` |
| **Windsurf** | `.windsurfrules` | Windsurf settings | `~/.windsurf/` |
| **Antigravity** | `.agent/rules/` | Via Gemini | VS Code settings |
| **Copilot** | `.github/copilot-instructions.md` | N/A | GitHub settings |
| **Zed** | `.zed/settings.json` | `.zed/settings.json` | `~/.config/zed/` |
| **Gemini** | `.gemini/settings.json` | N/A | `~/.gemini/` |
| **Amazon Q** | `.amazonq/` | IDE settings | AWS profile |
| **Continue** | `.continuerc.json` | `config.json` | `~/.continue/` |
| **Aider** | `.aider.conf.yml` | N/A | `~/.aider.conf.yml` |

## Claude Code

```
./CLAUDE.md                    # Project instructions
./CLAUDE.local.md              # Personal (gitignored)
./.claude/
├── settings.json              # Tool settings, MCP servers
├── settings.local.json        # Personal settings
├── rules/
│   └── *.md                   # Modular rules
├── skills/
│   └── *.md                   # Custom skills
└── memory.md                  # Project memory
~/.claude/
├── CLAUDE.md                  # User-wide instructions
├── settings.json              # Global settings
└── memory.md                  # Global memory
```

## Claude Desktop

```
# macOS
~/Library/Application Support/Claude/
└── claude_desktop_config.json    # MCP servers, settings

# Windows
%APPDATA%\Claude\
└── claude_desktop_config.json    # MCP servers, settings

# Desktop Extensions
extension.mcpb/
└── manifest.json                 # Extension definition
```

## Cursor

```
./.cursorrules                 # Project rules (legacy)
./.cursor/
├── rules                      # Project rules (current)
├── settings.json              # Cursor settings
└── mcp.json                   # MCP configuration
~/.cursor/
└── rules                      # Global rules
```

## Windsurf

```
./.windsurfrules               # Project rules
./.windsurf/
├── rules/                     # Modular rules
├── settings.json              # IDE settings
└── cascade.json               # Cascade config
./.codeiumignore               # Files to ignore
```

## Google Antigravity

```
./.agent/
├── rules/                     # Always-on rules
│   └── *.md
├── skills/                    # On-demand skills
│   └── skill-name/
│       ├── SKILL.md          # Required skill definition
│       └── scripts/          # Supporting scripts
└── workflows/                 # User-triggered macros
    └── *.md

./.vscode/                     # VS Code compatibility
├── settings.json
└── extensions.json

~/.gemini/antigravity/skills/  # Global skills
```

## GitHub Copilot

```
./.github/
└── copilot-instructions.md    # Repository instructions
./AGENTS.md                    # Universal format (supported)
# Plus: GitHub account settings
# Plus: Organization settings
```

## Zed

```
./.zed/
└── settings.json              # Project settings (AI, MCP)
./AGENTS.md                    # Universal format (supported)
~/.config/zed/
└── settings.json              # User settings
```

## Gemini Code Assist

```
./.gemini/
└── settings.json              # Project configuration
./GEMINI.md                    # Project rules
./AGENTS.md                    # Universal format
~/.gemini/
└── settings.json              # User configuration
```

## Amazon Q Developer

```
./.amazonq/
├── default.json               # Default configuration
├── rules/                     # Project rules
│   └── *.md
└── agents/                    # Custom agents
```

## Continue.dev

```
~/.continue/
├── config.json                # Global configuration
└── config.ts                  # TypeScript config
./.continuerc.json             # Project overrides
```

## Aider

```
./.aider.conf.yml              # Project configuration
~/.aider.conf.yml              # User configuration
# Command-line flags override all
```

## Universal (AGENTS.md)

```
./AGENTS.md                    # Repository root
./subdirectory/AGENTS.md       # Nested for monorepos
```

Supported natively by: OpenAI Codex, Google Jules, Cursor, Copilot, Aider, Zed, Factory AI

---

*Last updated: 2026-01-23*
*Status: Complete*
