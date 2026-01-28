# Modern AI Coding Tools Integration Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:writing-plans to create implementation plan after design approval.

**Goal:** Extend repo-tools to support the full modern AI coding assistant ecosystem.

**Research Date:** 2026-01-28

---

## 1. Current State

### Already Implemented (repo-tools)
| Tool | Config File | Status |
|------|-------------|--------|
| Cursor | `.cursorrules` | ✅ Implemented |
| VSCode | `.vscode/settings.json` | ✅ Implemented |
| Claude Code | `CLAUDE.md` | ✅ Implemented |
| Windsurf | `.windsurfrules` | ✅ Implemented |
| Antigravity | `.agent/` directory | ✅ Implemented |
| Gemini | `.gemini/` | ✅ Implemented |

### Missing (High Priority)
| Tool | Config File(s) | Priority |
|------|----------------|----------|
| GitHub Copilot | `.github/copilot-instructions.md` | HIGH |
| JetBrains AI | `.aiassistant/rules/*.md` | HIGH |
| Zed | `.rules` or `.zed/settings.json` | HIGH |
| Aider | `.aider.conf.yml` | HIGH |
| Cline | `.clinerules` or `.clinerules/*.md` | HIGH |
| Roo Code | `.roo/rules/*.md` | HIGH |
| Amazon Q | `.amazonq/rules/*.md` | MEDIUM |
| Continue.dev | `~/.continue/config.yaml` | MEDIUM |
| Tabnine | `.tabnine` + `.tabnineignore` | LOW |

---

## 2. Tool Configuration Details

### 2.1 GitHub Copilot

**Source:** [GitHub Docs](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot)

**Config Files:**
- `.github/copilot-instructions.md` - Global project instructions
- `.github/*.instructions.md` - Scoped instructions with `applyTo` frontmatter

**Format:** Markdown with optional YAML frontmatter
```markdown
---
applyTo: "**/*.py"
---
Use type hints for all function parameters.
```

**Behavior:** Automatically appended to chat context. Cross-platform (VS Code, Visual Studio, JetBrains, GitHub.com).

---

### 2.2 JetBrains AI Assistant

**Source:** [JetBrains Docs](https://www.jetbrains.com/help/ai-assistant/configure-project-rules.html)

**Config Files:**
- `.aiassistant/rules/*.md` - Project rules (Markdown files)
- `.aiignore` - Exclude files from AI (same as .gitignore syntax)
- `.noai` - Empty file to disable AI for entire project

**Format:** Markdown files, one rule per file
```markdown
# Python Style Guide

All Python code must:
- Use type hints
- Follow PEP 8
- Include docstrings
```

**Behavior:** Rules auto-added to chat sessions. Referenced via `@rule:filename`.

**Cross-compatibility:** Also reads `.cursorignore`, `.codeiumignore`, `.aiexclude`.

---

### 2.3 Zed Editor

**Source:** [Zed Docs](https://zed.dev/docs/ai/rules)

**Config Files:**
- `.rules` - Project rules (single file, highest priority)
- `.zed/settings.json` - Project settings (JSON)
- `~/.config/zed/settings.json` - User settings

**Format:** Plain text or Markdown for `.rules`

**Priority Order:** `.rules` > `.cursorrules` > other rule files

**Settings Example:**
```json
{
  "assistant": {
    "default_model": {
      "provider": "anthropic",
      "model": "claude-sonnet-4-20250514"
    }
  }
}
```

---

### 2.4 Aider

**Source:** [Aider Docs](https://aider.chat/docs/config/aider_conf.html)

**Config Files:**
- `.aider.conf.yml` - Project config (YAML)
- `~/.aider.conf.yml` - User config
- `.env` - Environment variables (AIDER_xxx)

**Format:** YAML
```yaml
model: claude-sonnet-4-20250514
read:
  - CONVENTIONS.md
  - docs/ARCHITECTURE.md
auto-commits: true
```

**Load Order:** Home → Git root → Current dir (last wins)

---

### 2.5 Cline (VS Code)

**Source:** [Cline Docs](https://docs.cline.bot/features/cline-rules)

**Config Files:**
- `.clinerules` - Single file (plain text/Markdown)
- `.clinerules/*.md` - Directory of rule files

**Format:** Markdown
```markdown
# Project Guidelines

## Code Style
- Use TypeScript strict mode
- Prefer functional components

## Testing
- Write tests for all new features
```

**Behavior:** Version-controlled, toggleable per-file via UI.

---

### 2.6 Roo Code (Fork of Cline)

**Source:** [Roo Docs](https://docs.roocode.com/features/custom-instructions)

**Config Files:**
- `.roo/rules/*.md` - Workspace rules
- `.roo/rules-{mode-slug}/*.md` - Mode-specific rules
- `.roomodes` - Custom modes (YAML or JSON)
- `~/.roo/rules/` - Global rules

**Format:** Markdown for rules, YAML/JSON for modes
```yaml
# .roomodes
customModes:
  - slug: docs-writer
    name: Documentation Writer
    roleDefinition: You are a technical writer...
    groups:
      - read
      - edit
```

**Legacy:** Still reads `.clinerules` for backward compatibility.

---

### 2.7 Amazon Q Developer

**Source:** [AWS Docs](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/context-project-rules.html)

**Config Files:**
- `.amazonq/rules/*.md` - Project rules (Markdown)
- Agent config with `resources` field for context

**Format:** Markdown for rules
```markdown
# Coding Standards

All code must:
- Follow AWS best practices
- Use IAM roles instead of access keys
```

---

### 2.8 Continue.dev

**Source:** [Continue Docs](https://docs.continue.dev/reference)

**Config Files:**
- `~/.continue/config.yaml` - User config
- `~/.continue/config.json` - Legacy format

**Format:** YAML (preferred) or JSON
```yaml
models:
  - title: Claude
    provider: anthropic
    model: claude-sonnet-4-20250514
contextProviders:
  - name: diff
  - name: terminal
rules:
  - Always use TypeScript
  - Prefer functional programming
```

---

### 2.9 Tabnine

**Source:** [Tabnine GitHub](https://github.com/codota/TabNine/blob/master/TabNineProjectConfigurations.md)

**Config Files:**
- `.tabnine` - Project settings (JSON)
- `.tabnineignore` - Exclude patterns (gitignore syntax)
- `.tabnine/mcp_servers.json` - MCP configuration

**Format:** JSON
```json
{
  "team_learning": false
}
```

---

## 3. Implementation Strategy

### 3.1 Common Patterns Identified

1. **Markdown Rules:** Most tools use `.md` files for instructions
2. **Directory vs File:** Many support both single file and directory of files
3. **Priority/Fallback:** Tools often read multiple file types with priority
4. **Ignore Files:** Separate `.xxxignore` files common (gitignore syntax)
5. **Cross-Compatibility:** Some tools read each other's config files

### 3.2 Proposed ToolIntegration Additions

```rust
// New integrations to add to repo-tools
pub struct CopilotIntegration;      // .github/copilot-instructions.md
pub struct JetBrainsIntegration;    // .aiassistant/rules/
pub struct ZedIntegration;          // .rules, .zed/settings.json
pub struct AiderIntegration;        // .aider.conf.yml
pub struct ClineIntegration;        // .clinerules
pub struct RooIntegration;          // .roo/rules/
pub struct AmazonQIntegration;      // .amazonq/rules/
pub struct ContinueIntegration;     // ~/.continue/config.yaml
pub struct TabnineIntegration;      // .tabnine
```

### 3.3 Shared Infrastructure

Create common utilities for:
- Markdown rule file generation with managed blocks
- Directory-based rule collections
- Ignore file generation (shared syntax with gitignore)
- YAML/JSON config generation

---

## 4. Priority Ranking

### Tier 1: Essential (Widely Used)
1. **GitHub Copilot** - Most popular AI coding tool
2. **Cline/Roo** - Very popular VS Code extensions
3. **JetBrains AI** - Large JetBrains IDE user base

### Tier 2: Important (Growing)
4. **Zed** - Fast-growing modern editor
5. **Aider** - Popular CLI coding assistant
6. **Amazon Q** - AWS ecosystem

### Tier 3: Nice to Have
7. **Continue.dev** - Open source alternative
8. **Tabnine** - Established but declining

---

## 5. Questions for Discussion

Before proceeding to implementation planning:

1. **Scope:** Should we implement all 9 tools, or focus on Tier 1 first?
2. **Managed Blocks:** Should all tools use managed block markers, or respect each tool's native conventions?
3. **Cross-Compatibility:** Should we generate multiple config files for tools that read each other's configs?
4. **Ignore Files:** Should we manage `.xxxignore` files, or leave that to users?

---

## Sources

- [GitHub Copilot Instructions](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot)
- [JetBrains AI Project Rules](https://www.jetbrains.com/help/ai-assistant/configure-project-rules.html)
- [Zed AI Rules](https://zed.dev/docs/ai/rules)
- [Aider Configuration](https://aider.chat/docs/config/aider_conf.html)
- [Cline Rules](https://docs.cline.bot/features/cline-rules)
- [Roo Code Custom Instructions](https://docs.roocode.com/features/custom-instructions)
- [Amazon Q Project Rules](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/context-project-rules.html)
- [Continue.dev Configuration](https://docs.continue.dev/reference)
- [Tabnine Project Config](https://github.com/codota/TabNine/blob/master/TabNineProjectConfigurations.md)
