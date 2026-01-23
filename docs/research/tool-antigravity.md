# Google Antigravity

Google's AI-native agentic IDE built on VS Code, announced November 2025.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | Google |
| **Announced** | November 18, 2025 (with Gemini 3) |
| **Base** | VS Code fork (disputed - possibly Windsurf fork) |
| **Models** | Gemini 3 Pro, Deep Think, Flash; also Claude Sonnet/Opus, GPT-OSS |
| **MCP Support** | Native |
| **AGENTS.md** | Supported (uses rules in `.agent/`) |

## Architecture

Antigravity introduces an "agent-first" paradigm with two primary views:

1. **Editor View**: Traditional IDE interface with agent sidebar
2. **Manager View**: Control center for orchestrating multiple agents working in parallel across workspaces

## Configuration Files

### .agent/ Directory

Primary configuration location (distinct from `.vscode/`):

```
.agent/
├── rules/                    # Passive, always-on guardrails
│   └── *.md                  # Markdown rule files
├── skills/                   # Agent-triggered expertise
│   └── skill-name/
│       ├── SKILL.md          # Skill definition (required)
│       └── scripts/          # Supporting scripts
└── workflows/                # User-triggered macros
    └── *.md
```

### Global Skills Location

```
~/.gemini/antigravity/skills/    # User-wide skills
```

### SKILL.md Format

```markdown
---
name: Deploy to Staging
description: Deploy application to staging environment
---

## Instructions

1. Run tests first
2. Build production bundle
3. Deploy to staging server

## Scripts

Use `./scripts/deploy.sh` for deployment.
```

### .vscode/ Compatibility

Since Antigravity is a VS Code fork, it respects:
- `.vscode/settings.json` - Editor settings
- `.vscode/extensions.json` - Recommended extensions
- `.vscode/launch.json` - Debug configurations

**Note**: Uses OpenVSX registry by default, but can be configured to use VS Code Marketplace.

## Skills vs Rules vs Workflows

| Type | Location | Trigger | Behavior |
|------|----------|---------|----------|
| **Rules** | `.agent/rules/` | Always on | Passive guardrails |
| **Skills** | `.agent/skills/` | Agent-triggered | On-demand expertise |
| **Workflows** | `.agent/workflows/` | User command | Active macros |

## Terminal Policy

Granular permission system for shell commands:

| Policy | Description |
|--------|-------------|
| Auto | AI runs standard commands without prompting |
| Agent Decides | Agent determines when confirmation needed |
| Review Required | Always ask before executing |

Configuration includes Allow Lists and Deny Lists for command patterns.

## Development Modes

| Mode | Description |
|------|-------------|
| Agent-driven ("Autopilot") | AI writes code, creates files, runs commands autonomously |
| Review-driven | AI asks permission before almost any action |
| Agent-assisted (Recommended) | User stays in control, AI helps with safe automations |

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Multi-file editing | Full | Manager view coordinates |
| Terminal access | Full | Configurable policies |
| Autonomous coding | Full | Agent-first design |
| File creation | Full | Via agents |
| Git operations | Full | Integrated |
| Web browsing | Yes | Built-in browser view |
| MCP | Native | Supported |

## Pricing (2026 Projection)

| Tier | Price | Features |
|------|-------|----------|
| Individual | Free (Preview) | Rate limited |
| Pro | ~$20/month | Higher limits |
| Enterprise | ~$40-60/user/month | SSO, data residency |

## Unique Differentiators

1. **Agent Manager View**: Orchestrate multiple parallel agents
2. **Multi-Model Support**: Gemini, Claude, GPT in same IDE
3. **Skills System**: On-demand agent expertise
4. **VS Code Compatibility**: Existing workflows carry over
5. **Google Cloud Integration**: Deep GCP integration

## Limitations

- Some VS Code extensions not available (e.g., C# Dev Kit licensing)
- OpenVSX registry by default (can be changed)
- New product - ecosystem still maturing
- Compute costs for Gemini 3 are high

## Quick Reference

```
./.agent/
├── rules/                    # Always-on rules
│   └── *.md
├── skills/                   # On-demand expertise
│   └── skill-name/
│       ├── SKILL.md         # Required
│       └── scripts/
└── workflows/                # User-triggered
    └── *.md

./.vscode/                    # VS Code compatibility
├── settings.json
└── extensions.json

~/.gemini/antigravity/skills/ # Global skills
```

## Sources

- [Google Developers Blog - Antigravity Announcement](https://developers.googleblog.com/build-with-google-antigravity-our-new-agentic-development-platform/)
- [Google Codelabs - Getting Started](https://codelabs.developers.google.com/getting-started-google-antigravity)
- [Google Codelabs - Authoring Skills](https://codelabs.developers.google.com/getting-started-with-antigravity-skills)
- [Wikipedia - Google Antigravity](https://en.wikipedia.org/wiki/Google_Antigravity)

---

*Last updated: 2026-01-23*
*Status: Complete*
