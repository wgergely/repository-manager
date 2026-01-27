# Tool Configuration Formats Research

> **Research Date:** January 2026
> **Purpose:** Document exact configuration file formats for AI coding tools

---

## Table of Contents

1. [Cursor](#1-cursor)
2. [Claude Code](#2-claude-code)
3. [VSCode](#3-vscode)
4. [Windsurf](#4-windsurf)
5. [Gemini Code Assist](#5-gemini-code-assist)
6. [JetBrains IDEs](#6-jetbrains-ides)
7. [Agentic Tools](#7-agentic-tools)
   - [Cline](#71-cline)
   - [Aider](#72-aider)
   - [Continue](#73-continue)
   - [GitHub Copilot](#74-github-copilot)
   - [Amazon Q Developer](#75-amazon-q-developer)
   - [Roo Code](#76-roo-code)

---

## 1. Cursor

**Description:** AI-first code editor built on VSCode

### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `.cursorrules` | Project root | Legacy rules file (deprecated) |
| `.cursor/rules/*.mdc` | Project `.cursor/rules/` | Project rules (current format) |
| User Rules | Settings UI | Global rules for all projects |

### File Format

**New Format (.mdc - Markdown Component):**

```markdown
---
description: Brief summary (for Agent Requested rules)
globs: ["*.py", "src/**/*.js"]
alwaysApply: false
---

# Rule Content

Your instructions here in Markdown format.
```

**Legacy Format (.cursorrules):**

Plain Markdown file with natural language instructions. Still supported but deprecated.

### Schema/Structure

The `.mdc` frontmatter supports:
- `description`: String - When rule should be applied (for model-requested rules)
- `globs`: Array - File patterns for auto-attached rules
- `alwaysApply`: Boolean - If `true`, rule always applies

### Hot-Reload Behavior

Rules are loaded when Cursor starts or when the rules editor loses focus. Changes to `.mdc` files may require window reload for immediate effect.

### CLI Verification

No dedicated CLI command. Rules appear in the Cursor UI under Settings > Rules.

### Official Documentation

- [Cursor Rules for AI](https://docs.cursor.com/context/rules-for-ai)
- [Awesome Cursorrules Repository](https://github.com/PatrickJS/awesome-cursorrules)

---

## 2. Claude Code

**Description:** Anthropic's official CLI for Claude

### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `CLAUDE.md` | Project root | Project instructions (shared) |
| `CLAUDE.local.md` | Project root | Personal instructions (gitignored) |
| `.claude/CLAUDE.md` | Project `.claude/` | Alternative location |
| `~/.claude/CLAUDE.md` | Home directory | User-level defaults |
| `settings.json` | `~/.claude/settings.json` | User settings |
| `settings.json` | `.claude/settings.json` | Project settings (shared) |
| `settings.local.json` | `.claude/settings.local.json` | Project settings (personal) |
| `managed-settings.json` | OS-specific (see below) | Enterprise managed settings |

**Managed Settings Paths:**
- Windows: `C:\ProgramData\ClaudeCode\managed-settings.json`
- macOS: `/Library/Application Support/ClaudeCode/managed-settings.json`
- Linux/WSL: `/etc/claude-code/managed-settings.json`

### File Format

**CLAUDE.md (Markdown):**

```markdown
# Project Name

## Bash commands
- npm run build: Build the project
- npm run test: Run tests

## Code style
- Use ES modules (import/export) syntax
- Destructure imports when possible

## Workflow
- Run typecheck after code changes
- Prefer single test runs over full suite
```

**settings.json (JSON):**

```json
{
  "allowedTools": ["Read", "Write", "Bash"],
  "model": "claude-sonnet-4-20250514",
  "customInstructions": "Additional context here"
}
```

### Schema/Structure

CLAUDE.md has no required schema - natural language Markdown. Keep concise.

settings.json uses hierarchical merging:
- Arrays are merged (not replaced)
- Precedence: managed > project > user > defaults

### Hot-Reload Behavior

**Yes - Automatic hot-reload** (since v1.0.90). Changes to CLAUDE.md and settings.json take effect immediately without restart.

### CLI Verification

```bash
# Initialize CLAUDE.md from project structure
/init

# Verify settings and diagnose issues
/doctor

# Add instruction to CLAUDE.md
# (Press # key during session)
```

### Official Documentation

- [Claude Code Settings](https://code.claude.com/docs/en/settings)
- [Using CLAUDE.md Files](https://claude.com/blog/using-claude-md-files)
- [Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)

---

## 3. VSCode

**Description:** Microsoft's code editor with AI extensions

### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `settings.json` | `%APPDATA%\Code\User\` (Windows) | User settings |
| `settings.json` | `~/Library/Application Support/Code/User/` (macOS) | User settings |
| `settings.json` | `~/.config/Code/User/` (Linux) | User settings |
| `settings.json` | `.vscode/settings.json` | Workspace settings |
| `tasks.json` | `.vscode/tasks.json` | Task definitions |
| `launch.json` | `.vscode/launch.json` | Debug configurations |

### File Format

**JSON with Comments (JSONC):**

```jsonc
{
  // Editor settings
  "editor.fontSize": 14,
  "editor.tabSize": 2,

  // Extension settings
  "github.copilot.enable": {
    "*": true,
    "markdown": false
  },

  /* Block comment also supported */
  "files.exclude": {
    "**/.git": true
  }
}
```

### Schema/Structure

- Standard JSON with `//' and `/* */` comments allowed
- Trailing commas allowed (but show warning)
- Full IntelliSense for setting names and values
- Settings hierarchy: Default < User < Remote < Workspace < Folder < Language-specific

### Hot-Reload Behavior

**Yes - Automatic.** Most settings apply immediately. Some require window reload (shown in UI).

### CLI Verification

```bash
# Open settings via command palette
# Ctrl+Shift+P -> "Preferences: Open Settings (JSON)"

# Via CLI
code --list-extensions  # List installed extensions
```

### Official Documentation

- [User and Workspace Settings](https://code.visualstudio.com/docs/getstarted/settings)
- [Settings JSON Editing](https://code.visualstudio.com/docs/languages/json)

---

## 4. Windsurf

**Description:** Codeium's agentic AI IDE

### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `.windsurf/rules/*.md` | Project `.windsurf/rules/` | Workspace rules |
| `global_rules.md` | Global config | Global rules across workspaces |
| `.windsurfrules` | Project root | Legacy rules file |
| `mcp_config.json` | `~/.codeium/windsurf/mcp_config.json` | MCP server configuration |
| `.codeiumignore` | Project root or `~/.codeium/` | Ignore patterns |

### File Format

**Rules (Markdown):**

```markdown
# Project Rules

## Coding Standards
- Use TypeScript for all new files
- Follow ESLint configuration

## Architecture
- Components in src/components/
- API calls in src/api/
```

**MCP Config (JSON):**

```json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@example/mcp-server"],
      "disabled": false,
      "alwaysAllow": []
    }
  }
}
```

### Schema/Structure

Rules:
- Individual rule files: max 6000 characters
- Total combined rules: max 12,000 characters
- Global rules take priority, then workspace rules

Rule Trigger Types:
- Glob patterns (e.g., `*.js`, `src/**/*.ts`)
- Natural language descriptions
- Always-on rules

### Hot-Reload Behavior

Rules are loaded when Cascade starts. Changes may require starting a new Cascade session.

### CLI Verification

No dedicated CLI. Rules visible in Windsurf UI under Customizations > Rules panel.

### Official Documentation

- [Windsurf Documentation](https://docs.windsurf.com/)
- [Cascade Memories](https://docs.windsurf.com/windsurf/cascade/memories)

---

## 5. Gemini Code Assist

**Description:** Google's AI coding assistant

### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `settings.json` | `~/.gemini/settings.json` | Global user settings |
| `settings.json` | `.gemini/settings.json` | Project-specific settings |
| `styleguide.md` | Repository | Code review style guide |
| `.env` | `.gemini/.env` | Environment variables |

### File Format

**settings.json:**

```json
{
  "theme": "dark",
  "authentication": "oauth",
  "preferredEditor": "vscode",
  "mcpServers": {
    "example-server": {
      "command": "node",
      "args": ["server.js"]
    }
  }
}
```

**styleguide.md:**

Natural language description of code review preferences. No defined schema.

### Schema/Structure

Configuration precedence (low to high):
1. Default values
2. System defaults file
3. User settings file
4. Project settings file
5. Environment variables
6. Command-line arguments

### Hot-Reload Behavior

Settings loaded at startup. CLI requires restart for settings changes.

### CLI Verification

```bash
# Gemini CLI configuration commands
gemini config list
gemini config get <key>
gemini config set <key> <value>
```

### Official Documentation

- [Gemini Code Assist Setup](https://developers.google.com/gemini-code-assist/docs/set-up-gemini)
- [Gemini CLI Configuration](https://geminicli.com/docs/get-started/configuration/)
- [Customize Gemini Behavior](https://developers.google.com/gemini-code-assist/docs/customize-gemini-behavior-github)

---

## 6. JetBrains IDEs

**Description:** IntelliJ, PyCharm, WebStorm, etc. with AI Assistant

### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `.aiassistant/rules/*.md` | Project root | Project rules |
| `acp.json` | Project | Agent Client Protocol config |
| `.aiignore` | Project root | Ignore patterns for AI |
| `.noai` | Project root | Disable AI for project |

**Cache Locations:**
- Windows: `%LOCALAPPDATA%\JetBrains\<product><version>\aia\codex`
- macOS: `~/Library/Caches/JetBrains/<product><version>/aia/codex`
- Linux: `~/.cache/JetBrains/<product><version>/aia/codex`

### File Format

**Project Rules (Markdown):**

```markdown
# Coding Standards

- Use Kotlin for new code
- Follow project naming conventions
- Include KDoc for public APIs
```

**acp.json:**

```json
{
  "displayName": "My Agent",
  "command": "/path/to/agent",
  "args": ["--mode", "production"],
  "env": {
    "API_KEY": "${env:MY_API_KEY}"
  },
  "execution_environment": "wsl"
}
```

### Schema/Structure

Rules can be triggered by:
- Manual invocation (`@rule:` or `#rule:`)
- Model decision (requires `Instruction` field)
- File patterns (e.g., `*.kt`, `src/**`)

Also supports `.cursorignore`, `.codeiumignore`, or `.aiexclude` files.

### Hot-Reload Behavior

Rules loaded when IDE starts. May require IDE restart for changes.

### CLI Verification

No dedicated CLI. Rules configured via Settings > Tools > AI Assistant > Rules.

### Official Documentation

- [Configure Project Rules](https://www.jetbrains.com/help/ai-assistant/configure-project-rules.html)
- [AI Assistant Settings](https://www.jetbrains.com/help/ai-assistant/settings-reference-ai-assistant.html)
- [Agent Client Protocol](https://www.jetbrains.com/help/ai-assistant/acp.html)

---

## 7. Agentic Tools

### 7.1 Cline

**Description:** Autonomous coding agent for VS Code

#### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `.clinerules` | Project root | Single-file rules |
| `.clinerules/` | Project root | Rules directory |
| `.clinerules/workflows/` | Project root | Workflow definitions |

#### File Format

**Markdown files with natural language:**

```markdown
# Project Rules

## Role
You are an AI coding assistant focusing on JavaScript development.

## Coding Standards
- Always use semicolons at the end of statements
- Use camelCase for variable names
- Prefer const over let

## Workflow
- Ask for review after each file change
- Don't edit README unless requested
```

#### Hot-Reload Behavior

Rules loaded when Cline starts. Changes require restarting Cline session.

#### Official Documentation

- [Cline GitHub Repository](https://github.com/cline/cline)
- [Cline Rules Blog Post](https://cline.bot/blog/clinerules-version-controlled-shareable-and-ai-editable-instructions)

---

### 7.2 Aider

**Description:** AI pair programming in terminal

#### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `.aider.conf.yml` | Home directory | Global config |
| `.aider.conf.yml` | Git repo root | Repo config |
| `.aider.conf.yml` | Current directory | Local config |
| `.env` | Current directory | Environment variables |

Files load in order; later files override earlier ones.

#### File Format

**YAML:**

```yaml
# Model configuration
model: claude-sonnet-4-20250514

# API keys
api-key:
  - gemini=your-key
  - openrouter=your-key

# Files to always include
read:
  - CONVENTIONS.md
  - docs/architecture.md

# Behavior settings
auto-commits: false
vim: true
chat-history-file: .aider.chat.history

# Input history
input-history-file: .aider.input.history
restore-chat-history: true
```

#### Hot-Reload Behavior

Config loaded at startup. Requires restarting aider for changes.

#### CLI Verification

```bash
# Use specific config file
aider --config /path/to/config.yml

# Override config via CLI
aider --model claude-sonnet-4-20250514 --no-auto-commits
```

#### Official Documentation

- [Aider YAML Config](https://aider.chat/docs/config/aider_conf.html)
- [Aider Configuration](https://aider.chat/docs/config.html)
- [Aider Options Reference](https://aider.chat/docs/config/options.html)

---

### 7.3 Continue

**Description:** Open-source AI code assistant for VS Code and JetBrains

#### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `config.yaml` | `~/.continue/config.yaml` | User configuration (current) |
| `config.json` | `~/.continue/config.json` | User configuration (deprecated) |

#### File Format

**YAML (config.yaml - current):**

```yaml
name: My Continue Config
version: 1.0
models:
  - name: Claude
    provider: anthropic
    model: claude-sonnet-4-20250514
    apiKey: ${ANTHROPIC_API_KEY}
  - name: GPT-4
    provider: openai
    model: gpt-4
rules:
  - Always use TypeScript
  - Follow functional programming patterns
```

**JSON (config.json - deprecated):**

```json
{
  "models": [
    {
      "name": "Claude",
      "provider": "anthropic",
      "model": "claude-sonnet-4-20250514"
    }
  ]
}
```

#### Hot-Reload Behavior

Config changes apply on next chat session. May require extension reload.

#### Official Documentation

- [Continue config.yaml Reference](https://docs.continue.dev/reference)
- [Continue GitHub](https://github.com/continuedev/continue)

---

### 7.4 GitHub Copilot

**Description:** GitHub's AI pair programmer

#### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `copilot-instructions.md` | `.github/copilot-instructions.md` | Repository instructions |
| `*.instructions.md` | `.github/instructions/` | Path-specific instructions |
| `CONTRIBUTING.md` | `.github/copilot-workspace/` | Copilot Workspace guidance |
| `mcp-config.json` | `~/.copilot/mcp-config.json` | MCP server config (CLI) |
| `config` | `~/.copilot/config` | URL access rules (CLI) |

#### File Format

**copilot-instructions.md (Markdown):**

```markdown
# Project Guidelines

## Code Style
- Use TypeScript strict mode
- Prefer functional components in React
- Include JSDoc comments for public APIs

## Architecture
- Follow the repository pattern for data access
- Use dependency injection for services
```

**Path-specific instructions (*.instructions.md):**

```markdown
---
applyTo: "**/*.test.ts"
---

# Test File Guidelines

- Use describe/it blocks from Jest
- Include both positive and negative test cases
- Mock external dependencies
```

#### Hot-Reload Behavior

Instructions loaded per-request. Changes apply to next Copilot interaction.

#### CLI Verification

References shown in Chat view. Click reference to verify file was used.

#### Official Documentation

- [VS Code Custom Instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Repository Custom Instructions](https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot)

---

### 7.5 Amazon Q Developer

**Description:** AWS AI coding assistant

#### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `default.json` | `~/.aws/amazonq/default.json` | Global MCP config |
| `default.json` | `.amazonq/default.json` | Project MCP config |

#### File Format

**JSON:**

```json
{
  "mcpServers": {
    "my-server": {
      "command": "node",
      "args": ["server.js"],
      "env": {
        "API_KEY": "your-key"
      }
    }
  }
}
```

#### CLI Verification

```bash
# View settings
q settings list --all

# Set a setting
q settings chat.enableKnowledge true

# Open settings file
q settings open

# Delete a setting
q settings --delete setting.name
```

#### Hot-Reload Behavior

Workspace settings take precedence. May require extension reload.

#### Official Documentation

- [Amazon Q Developer Setup](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/q-in-IDE-setup.html)
- [MCP Configuration](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/mcp-ide.html)
- [Command-Line Settings](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-settings.html)

---

### 7.6 Roo Code

**Description:** AI dev team in VS Code (Cline fork)

#### Config File Paths

| File | Location | Purpose |
|------|----------|---------|
| `custom_modes.yaml` | `settings/custom_modes.yaml` | Global custom modes |
| `.roomodes` | Project root | Project custom modes |
| `.roo/rules/` | Project root | Workspace rules directory |
| `.roo/rules-{mode}/` | Project root | Mode-specific rules |
| `.roorules` | Project root | Single-file rules (fallback) |
| `AGENTS.md` | Project root | Agent-specific rules |

**Global Rules:**
- Linux/macOS: `~/.roo/rules/` and `~/.roo/rules-{modeSlug}/`
- Windows: `%USERPROFILE%\.roo\rules\`

#### File Format

**YAML (custom_modes.yaml):**

```yaml
modes:
  - slug: architect
    name: Architect Mode
    description: High-level design and planning
    rules:
      - Focus on architecture decisions
      - Avoid implementation details
```

**Markdown rules:**

```markdown
# Coding Standards

- Use TypeScript strict mode
- Follow functional patterns
- Write comprehensive tests
```

#### Hot-Reload Behavior

Rules loaded when Roo Code starts or mode changes. Restart session for changes.

#### Official Documentation

- [Roo Code Documentation](https://docs.roocode.com/)
- [Custom Modes](https://docs.roocode.com/features/custom-modes)
- [Custom Instructions](https://docs.roocode.com/features/custom-instructions)

---

## Summary Comparison Table

| Tool | Config Format | Hot Reload | Hierarchy Support |
|------|---------------|------------|-------------------|
| **Cursor** | MDC (Markdown) | Partial | User > Project |
| **Claude Code** | MD + JSON | Yes | Managed > Project > User |
| **VSCode** | JSONC | Yes | User > Workspace > Folder |
| **Windsurf** | Markdown + JSON | Partial | Global > Workspace |
| **Gemini** | JSON | No | CLI > Env > Project > User |
| **JetBrains** | Markdown + JSON | No | Project level |
| **Cline** | Markdown | No | Project level |
| **Aider** | YAML | No | CWD > Repo > Home |
| **Continue** | YAML (was JSON) | Partial | User level |
| **Copilot** | Markdown | Yes | Repo > Workspace |
| **Amazon Q** | JSON | Partial | Workspace > Global |
| **Roo Code** | YAML + Markdown | No | Global > Workspace > Mode |

---

## Cross-Tool Compatibility

Some tools recognize each other's ignore files:

| Ignore File | Recognized By |
|-------------|---------------|
| `.gitignore` | All tools |
| `.cursorignore` | Cursor, JetBrains |
| `.codeiumignore` | Windsurf, JetBrains |
| `.aiignore` | JetBrains |
| `.aiexclude` | JetBrains |

---

## Recommendations for Repository Manager

1. **Primary format:** Markdown for rules (widest compatibility)
2. **JSON for settings:** Most tools use JSON for structured config
3. **YAML alternative:** Aider, Roo Code, Continue prefer YAML
4. **Hot reload:** Claude Code and VSCode have best hot-reload support
5. **Hierarchy:** Support user > project > workspace levels
6. **Ignore files:** Consider supporting multiple ignore file formats

---

*Last updated: January 27, 2026*
