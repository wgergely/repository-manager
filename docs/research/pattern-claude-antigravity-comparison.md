# Claude Code vs Google Antigravity: Configuration Comparison

Direct comparison of two major agentic tools to identify patterns for repo-manager.

## Structural Comparison

| Aspect | Claude Code | Antigravity | Overlap |
|--------|-------------|-------------|---------|
| **Config root** | `.claude/` | `.agent/` | Similar pattern |
| **Rules location** | `.claude/rules/*.md` | `.agent/rules/*.md` | **Identical** |
| **Skills location** | `.claude/skills/*.md` | `.agent/skills/*/SKILL.md` | Similar |
| **Global location** | `~/.claude/` | `~/.gemini/antigravity/` | Same pattern |
| **Main instructions** | `CLAUDE.md` | `.agent/rules/*.md` | Different |
| **Format** | Markdown | Markdown | **Identical** |

## Directory Structure Alignment

```
Claude Code                          Antigravity
──────────────────────────────────   ──────────────────────────────────
.claude/                             .agent/
├── rules/                           ├── rules/
│   └── *.md          ←───────────→  │   └── *.md
├── skills/                          ├── skills/
│   └── *.md                         │   └── skill-name/
│                                    │       ├── SKILL.md
│                                    │       └── scripts/
├── settings.json                    └── workflows/
└── memory.md                            └── *.md

CLAUDE.md (root)                     (no equivalent - rules in .agent/)
```

## Rules: Nearly Identical

Both use Markdown files in a `rules/` subdirectory.

### Rules Comparison

Content is **identical**; only paths differ:
- Claude Code: `.claude/rules/coding-standards.md`
- Antigravity: `.agent/rules/coding-standards.md`

**Implication**: Rules are directly portable between the two systems with only path changes.

## Skills: Similar but Different Structure

### Claude Code Skill (Flat)
```markdown
# .claude/skills/deploy.md

## Description
Deploy application to staging.

## Instructions
1. Run tests
2. Build
3. Deploy
```

### Antigravity Skill (Directory + YAML frontmatter)
```markdown
# .agent/skills/deploy/SKILL.md
---
name: Deploy to Staging
description: Deploy application to staging
---

## Instructions
1. Run tests
2. Build
3. Deploy

## Scripts
Use `./scripts/deploy.sh`
```

### Key Differences

| Aspect | Claude Code | Antigravity |
|--------|-------------|-------------|
| Structure | Single `.md` file | Directory with `SKILL.md` |
| Metadata | In markdown body | YAML frontmatter |
| Scripts | Inline or referenced | Dedicated `scripts/` folder |
| Trigger | `/skill-name` | Agent-triggered (automatic) |

## Concept Mapping

| Claude Code | Antigravity | Equivalence |
|-------------|-------------|-------------|
| Rules | Rules | **Direct** |
| Skills | Skills | Structural difference |
| Memory | (none) | No equivalent |
| (none) | Workflows | No equivalent |
| Permissions | Terminal Policy | Conceptually similar |

### Antigravity's Unique "Workflows"

Antigravity has a third concept that Claude Code lacks:

| Type | Trigger | Purpose |
|------|---------|---------|
| Rules | Always on | Passive guardrails |
| Skills | Agent decides | On-demand expertise |
| **Workflows** | User command | Active macros (like `/command`) |

Claude Code combines Skills + Workflows into one concept (slash commands can be both).

## Permission Systems

### Claude Code
```json
// .claude/settings.json
{
  "permissions": {
    "allow": ["Bash", "Read", "Write"],
    "deny": ["WebFetch"]
  }
}
```

### Antigravity
- Terminal Policy: Auto / Agent Decides / Review Required
- Allow Lists / Deny Lists for command patterns
- Per-workspace configuration

**Overlap**: Both have allowlist/denylist patterns for shell commands.

## MCP Configuration

### Claude Code
```json
// .claude/settings.json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem"]
    }
  }
}
```

### Antigravity
- MCP support via Gemini integration
- Configuration through IDE settings
- (Specific format not publicly documented)

## Hierarchical Config

### Claude Code
```
1. ./subdirectory/CLAUDE.md
2. ./CLAUDE.md
3. ~/.claude/CLAUDE.md
4. System defaults

Child overrides parent
```

### Antigravity
```
1. ./.agent/rules/*.md (project)
2. ~/.gemini/antigravity/skills/ (global skills only)

Project rules are merged, not hierarchical
```

**Difference**: Claude Code has deeper hierarchy (per-directory). Antigravity is flatter.

## Implications for repo-manager

### What Can Be Shared Directly
1. **Rules content** - Markdown format is identical
2. **Basic skill instructions** - Content is portable
3. **Permission concepts** - Allow/deny patterns

### What Needs Translation
1. **Skill structure** - Flat file vs directory
2. **Skill metadata** - Body vs YAML frontmatter
3. **Config locations** - `.claude/` vs `.agent/`
4. **Main instructions file** - `CLAUDE.md` vs distributed rules

### Proposed repo-manager Mapping

```yaml
# .repository/config.yaml
providers:
  claude:
    enabled: true
    map_rules_to: CLAUDE.md        # Aggregate rules
    map_skills_to: .claude/skills/  # Flat files

  antigravity:
    enabled: true
    map_rules_to: .agent/rules/     # Keep distributed
    map_skills_to: .agent/skills/   # Directory structure
    add_frontmatter: true           # YAML metadata
```

### Universal Source Format

Given the overlap, a universal source could work:

```
.repository/
├── rules/
│   └── coding-standards.md    # Pure markdown, no frontmatter
├── skills/
│   └── deploy/
│       ├── skill.md           # Instructions
│       ├── meta.yaml          # Metadata (optional)
│       └── scripts/           # Scripts (optional)
```

**Translation**:
- Claude Code: Flatten to `.claude/skills/deploy.md`
- Antigravity: Copy to `.agent/skills/deploy/SKILL.md` with frontmatter

## Conclusion

The overlap is substantial enough that a unified configuration system is viable:

| Category | Portability | Strategy |
|----------|-------------|----------|
| Rules | High | Direct copy with path mapping |
| Skills | Medium | Structure translation |
| Permissions | Medium | Concept mapping |
| MCP | High | Format translation |
| Workflows | Low | Antigravity-specific |
| Memory | Low | Claude Code-specific |

---

*Last updated: 2026-01-23*
*Status: Complete*
