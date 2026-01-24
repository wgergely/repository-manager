# Gemini Code Assist (Google)

Google's AI coding assistant integrated with Google Cloud and IDEs.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Google |
| **Models** | Gemini Pro, Gemini Ultra |
| **Type** | IDE Extension + Cloud Integration |
| **MCP Support** | Native (via DeepMind adoption) |
| **AGENTS.md** | Native |

## Configuration Files

### .gemini/settings.json

Primary configuration file for Gemini Code Assist.

**Locations** (hierarchical):
- Project: `your-project/.gemini/settings.json`
- User: `~/.gemini/settings.json`
- System: `/etc/gemini-cli/settings.json` (Linux) or `C:\ProgramData\gemini-cli\settings.json` (Windows)

```json
{
  "project": {
    "language": "typescript",
    "framework": "nextjs"
  },
  "style": {
    "preferFunctional": true,
    "strictTypes": true
  },
  "context": {
    "include": ["src/**/*.ts"],
    "exclude": ["node_modules/**", "dist/**"]
  }
}
```

### GEMINI.md

Optional Markdown rules file.

```markdown
# GEMINI.md Example

## Project Type
Next.js 14 application with TypeScript.

## Conventions
- Server components by default
- Client components in `components/client/`
- API routes in `app/api/`

## Testing
Run `npm test` for unit tests.
```

### Configuration Precedence

1. Command-line flags (highest)
2. Project `.gemini/settings.json`
3. User `~/.gemini/settings.json`
4. System settings (lowest)

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Inline completions | Full | Core feature |
| Chat panel | Full | Integrated chat |
| Multi-file editing | Full | Coordinated edits |
| Terminal access | Partial | Via IDE terminal |
| Autonomous coding | Partial | Less agentic than Cursor |
| Git operations | Partial | Basic integration |
| MCP | Native | DeepMind adoption |

## Google Cloud Integration

- **BigQuery**: Natural language queries
- **Cloud Storage**: File operations
- **Pub/Sub**: Message handling
- **Cloud Functions**: Deployment assistance

## Context Management

- Project-level JSON configuration
- Hierarchical settings merging
- Google Cloud project context
- File pattern-based inclusion/exclusion

## Memory/Persistence

| Type | Persistence | Format |
|------|-------------|--------|
| Session | Limited | In-memory |
| Project | Via config | JSON |
| User | Via user config | JSON |
| Cloud | Via GCP | Cloud-native |

## Pricing

| Tier | Price | Features |
|------|-------|----------|
| Standard | Included with GCP | Basic features |
| Enterprise | Custom | Enhanced features, SLA |

## Configuration Discovery

```
1. IDE extension loads
2. Check .gemini/ in workspace root
3. Load settings.json if present
4. Merge with user settings
5. Merge with system settings
6. Apply command-line overrides
```

## Unique Differentiators

1. **Google Cloud Native**: Deep GCP integration
2. **Multi-modal**: Can process images, docs
3. **Grounding**: Search integration for current info
4. **Enterprise Ready**: GCP compliance and security

## Limitations

- Tightly coupled to Google ecosystem
- Configuration focused on tool behavior, not portable rules
- Less community adoption than Copilot/Cursor
- Limited MCP ecosystem compared to Anthropic tools

## Quick Reference

```
./.gemini/
└── settings.json              # Project configuration
./GEMINI.md                    # Project rules (Markdown)
./AGENTS.md                    # Universal format (supported)
~/.gemini/
└── settings.json              # User configuration
```

---

*Last updated: 2026-01-23*
*Status: Complete*
