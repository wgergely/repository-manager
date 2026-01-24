# GitHub Copilot / Copilot Workspace

GitHub's AI coding assistant, the most widely adopted in the market.

## Overview

| Attribute | Value |
|-----------|-------|
| **Company** | GitHub (Microsoft) |
| **Models** | OpenAI Codex, GPT-4 |
| **Type** | IDE Extension + Workspace |
| **MCP Support** | Limited (Experimental) |
| **AGENTS.md** | Native |

## Configuration Files

### .github/copilot-instructions.md

Repository-wide Copilot behavior instructions.

**Location**: `.github/copilot-instructions.md`

```markdown
# Copilot Instructions

## Language & Framework
This repository uses TypeScript with React and Node.js.

## Code Style
- Follow the Airbnb style guide
- Use functional components with hooks
- Prefer composition over inheritance

## Testing
- Write tests using Jest and React Testing Library
- Aim for 80% code coverage
- Include integration tests for API endpoints

## Security
- Never commit secrets or API keys
- Sanitize all user inputs
- Use parameterized queries for database operations
```

### Additional Configuration Levels

| Level | Location | Scope |
|-------|----------|-------|
| Personal | GitHub account settings | All repositories for user |
| Organization | Organization settings | All repos in org |
| Repository | `.github/copilot-instructions.md` | Single repository |

## Copilot Workspace

Agentic extension for issue-to-PR workflows:

- Issue-based context
- Automatic spec generation
- Plan-then-implement workflow
- Multi-file editing with review

## Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Inline completions | Full | Core feature |
| Chat panel | Full | Copilot Chat |
| Multi-file editing | Yes | Via Workspace |
| Terminal access | Partial | Limited |
| Autonomous coding | Yes | Workspace mode |
| Git operations | Full | Native GitHub integration |
| MCP | Limited | Experimental |

## Copilot Extensions

**Status**: Public beta (January 2026)

Two types:
1. **Skillsets**: Lightweight, minimal setup
2. **Agents**: Full control, custom logic

**Available partners**: Docker, MongoDB, Sentry, Stripe, Azure, Slack, Atlassian

**SDK**: Available and updated January 2026

**Configuration**: GitHub Marketplace, organization settings, IDE-specific settings

## Context Management

- File-level context from open editors
- Repository context via indexing
- Issue/PR context in Workspace mode
- Organization instructions for enterprise

## Memory/Persistence

| Type | Persistence | Format |
|------|-------------|--------|
| Session | No | N/A |
| Project | Via instructions file | Markdown |
| User | Via account settings | Text |

## Pricing

| Tier | Price | Features |
|------|-------|----------|
| Individual | $10/month | Basic completions, chat |
| Business | $19/user/month | Admin, policies, audit |
| Enterprise | $39/user/month | SSO, compliance, Workspace |

## Configuration Discovery

```
1. Extension loads in IDE
2. Check repository .github/ directory
3. Load copilot-instructions.md if present
4. Merge with personal instructions
5. Merge with organization instructions
6. Apply content exclusion policies
```

## Unique Differentiators

1. **Market Dominance**: Most widely adopted AI coding assistant
2. **GitHub Integration**: Native issue/PR/Actions workflow
3. **Workspace**: Issue-to-PR agentic capability
4. **Extensions Ecosystem**: Third-party integrations
5. **Enterprise Features**: Compliance, audit, policies

## Limitations

- No native MCP support (GitHub-native integrations instead)
- Limited multi-file editing outside Workspace
- No explicit memory system
- Less flexible model selection than competitors
- Configuration less granular than Claude Code

## Quick Reference

```
./.github/
└── copilot-instructions.md    # Repository instructions
./AGENTS.md                    # Universal format (supported)
# Plus: GitHub account settings for personal instructions
# Plus: Organization settings for org-wide instructions
```

---

*Last updated: 2026-01-23*
*Status: Complete*
