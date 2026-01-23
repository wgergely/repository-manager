# Agentic Coding Tool Support Matrix (2026)

## Overview

This document provides a comprehensive feature matrix for major agentic coding tools, analyzing their capabilities across key dimensions relevant to cross-platform interoperability and enterprise adoption.

**Tools Covered:**
- Claude Code (Anthropic)
- Gemini Code Assist (Google)
- GitHub Copilot / Copilot Workspace
- Cursor IDE
- Windsurf (Codeium)
- Continue.dev
- Aider
- Amazon Q Developer
- OpenAI Codex/Assistants

**Last Updated:** 2026-01-23

---

## 1. Feature Support Matrix

### 1.1 Rules/Instructions File Support

| Tool | Supported | File Format | File Name(s) | Location | Hierarchical |
|------|-----------|-------------|--------------|----------|--------------|
| **Claude Code** | Yes | Markdown | `CLAUDE.md`, `.claude/rules/*.md` | Root, any directory, `~/.claude/` | Yes |
| **Gemini Code Assist** | Yes      | JSON        | `settings.json`              | Project, User, System | Yes          |
| **GitHub Copilot** | Yes | Markdown | `copilot-instructions.md` | `.github/` | Partial (Org > Repo > User) |
| **Cursor IDE** | Yes | Markdown/Text | `.cursorrules`, `.cursor/rules` | Root, `.cursor/` | Limited |
| **Windsurf** | Yes | Markdown | `.windsurfrules`, rules files | Root, `.windsurf/` | Limited |
| **Continue.dev** | Yes | JSON | `config.json`, `.continuerc.json` | `~/.continue/`, root | Yes |
| **Aider** | Yes | YAML | `.aider.conf.yml` | Root, home directory | Yes |
| **Amazon Q Developer** | Partial | JSON/YAML | `.amazonq/` configuration | `.amazonq/`, AWS profile | Unknown |
| **OpenAI Codex/Assistants** | Partial | JSON | Assistant configuration | API/Dashboard | No |

#### Detailed Analysis

**Claude Code**
- **How it works:** CLAUDE.md files are read from all directories in the path hierarchy. Files are merged with child directories taking precedence for conflicts.
- **Configuration location:** Project root (`CLAUDE.md`), `.claude/rules/` directory, user home (`~/.claude/CLAUDE.md`)
- **Special features:**
  - `@import` syntax for including other files
  - YAML frontmatter for path-specific rules
  - Symlink support for shared rules
  - `CLAUDE.local.md` for gitignored personal rules
- **Known limitations:** Claude-specific syntax (imports, frontmatter) not portable

**Gemini Code Assist**
- **How it works:** Configuration is loaded from JSON `settings.json` files at the project, user, and system levels. Settings are hierarchical, with command-line flags having the highest precedence.
- **Configuration location:**
  - Project: `your-project/.gemini/settings.json`
  - User: `~/.gemini/settings.json`
  - System: `/etc/gemini-cli/settings.json` (Linux) or `C:\ProgramData\gemini-cli\settings.json` (Windows)
- **Known limitations:** Tightly coupled to the Google ecosystem; configuration is primarily for tool behavior, not portable project rules.

**GitHub Copilot**
- **How it works:** Single markdown file read from `.github/` directory; merged with personal instructions from account settings
- **Configuration location:** `.github/copilot-instructions.md`, GitHub account settings, organization settings
- **Known limitations:** No multi-file support; limited to one instructions file per repository

**Cursor IDE**
- **How it works:** Rules file read at project load; applied to all AI interactions
- **Configuration location:** `.cursorrules` (legacy), `.cursor/rules` (current), global `~/.cursor/rules`
- **Known limitations:** No hierarchical directory support; single file per project

**Windsurf**
- **How it works:** Rules integrated into Cascade agentic system; influences AI planning and execution
- **Configuration location:** `.windsurfrules`, `.windsurf/` directory
- **Known limitations:** Proprietary Cascade system may limit interoperability

**Continue.dev**
- **How it works:** JSON configuration with support for custom commands, context providers, and model selection
- **Configuration location:** `~/.continue/config.json` (global), `.continuerc.json` (project)
- **Known limitations:** JSON format less readable than Markdown for rules

**Aider**
- **How it works:** YAML configuration for model, behavior, and conventions
- **Configuration location:** `.aider.conf.yml` in root or home; command-line flags override
- **Known limitations:** Configuration focused on tool behavior rather than project rules

**Amazon Q Developer**
- **How it works:** Integration with AWS services and IDE plugins [VERIFY]
- **Configuration location:** `.amazonq/` directory, AWS Toolkit settings
- **Known limitations:** Heavily AWS-ecosystem focused; limited standalone configuration

**OpenAI Codex/Assistants**
- **How it works:** Assistants configured via API with instructions field; no file-based project config
- **Configuration location:** API configuration, OpenAI Dashboard
- **Known limitations:** No native file-based project rules; requires API integration

---

### 1.2 Memory/Context Persistence

| Tool | Supported | Memory Type | Storage Format | Scope Options | Cross-Session |
|------|-----------|-------------|----------------|---------------|---------------|
| **Claude Code** | Yes | Explicit files | Markdown | Project, User, System | Yes |
| **Gemini Code Assist** | Partial | Session + Index | Proprietary | Project | Limited |
| **GitHub Copilot** | No | Session only | N/A | Session | No |
| **Cursor IDE** | Partial | Codebase index | Proprietary | Project | Limited |
| **Windsurf** | Yes | Cascade memory | Proprietary | Project, User | Yes |
| **Continue.dev** | Partial | Config-based | JSON | Project, User | Limited |
| **Aider** | Partial | Git-based | Commits | Project | Yes (via git) |
| **Amazon Q Developer** | Partial | AWS context | Proprietary | Project, AWS | Unknown |
| **OpenAI Assistants** | Yes | Thread-based | API storage | Thread, Assistant | Yes |

#### Detailed Analysis

**Claude Code**
- **How it works:** Explicit memory files (`.claude/memory.md`) store learned facts, decisions, and preferences. Memory is loaded at session start and can be updated during conversation.
- **Configuration location:** `.claude/memory.md` (project), `~/.claude/memory.md` (user)
- **Features:** Markdown format allows version control; hierarchical memory with project overriding user
- **Known limitations:** Manual memory management; no automatic memory formation

**Cursor IDE**
- **How it works:** Codebase indexed via embeddings for semantic search. Chat history persisted per project.
- **Configuration location:** Internal Cursor storage; not directly configurable
- **Features:** `@codebase` command for semantic queries across entire repository
- **Known limitations:** Memory tied to Cursor installation; not exportable

**Windsurf**
- **How it works:** Cascade system automatically forms memories from interactions. Stores decisions, patterns, and context.
- **Configuration location:** Windsurf internal storage
- **Features:** Automatic memory formation; context carried across sessions
- **Known limitations:** Proprietary format; not portable to other tools

**OpenAI Assistants**
- **How it works:** Thread-based conversations persist on OpenAI servers. Assistants have persistent instructions.
- **Configuration location:** OpenAI API/Dashboard
- **Features:** Long-running threads; file attachments; retrieval augmentation
- **Known limitations:** Requires API calls; storage on OpenAI infrastructure

---

### 1.3 Skills/Plugins/Extensions

| Tool | Supported | Skill Format | Location | Invocation | Marketplace |
|------|-----------|--------------|----------|------------|-------------|
| **Claude Code** | Yes | Markdown | `.claude/skills/` | `/skill-name` | No |
| **Gemini Code Assist** | Partial | Google Extensions | IDE/Cloud | Integrated | Google Cloud |
| **GitHub Copilot** | Partial | Extensions (preview) | GitHub | Integrated | GitHub |
| **Cursor IDE** | Yes | VS Code Extensions | Extension store | Commands | VS Code |
| **Windsurf** | Partial | Cascade Actions | Built-in | Automatic | No |
| **Continue.dev** | Yes | JSON Commands | Config file | `/command` | Community |
| **Aider** | No | N/A | N/A | N/A | No |
| **Amazon Q Developer** | Partial | AWS Integrations | AWS Toolkit | Commands | AWS |
| **OpenAI Assistants** | Yes | Function Calling | API config | Automatic | No |

#### Detailed Analysis

**Claude Code**
- **How it works:** Skills are Markdown files with structured instructions. Loaded automatically from `.claude/skills/` or plugin directories.
- **Configuration location:** `.claude/skills/*.md`, plugin directories
- **Invocation:** `/skill-name` commands or automatic detection based on task
- **Features:**
  - Multi-step workflow definitions
  - Tool permission specifications
  - Code samples and templates
- **Known limitations:** Claude Code specific; skills not portable to other tools

**Example Skill Definition:**
```markdown
# deploy

## Description
Deploy application to specified environment.

## Instructions
1. Run tests to verify build
2. Build production artifacts
3. Deploy to target environment
4. Verify deployment health

## Permissions
- Bash(npm:*)
- Bash(./scripts/deploy.sh:*)
```

**GitHub Copilot**
- **How it works:** The Copilot extension system is in public beta. A new Copilot SDK allows developers to create extensions that integrate external tools and services. These extensions enable "agentic" workflows, moving beyond simple completions. Integrations with IDEs like JetBrains are also emerging, using an "agent mode" and the Model Context Protocol (MCP) to connect to external services.
- **Configuration location:** GitHub Marketplace, organization settings, and IDE-specific settings.
- **Known limitations:** The extension ecosystem is still under development. Building and distributing extensions requires using the GitHub Apps platform.

**Continue.dev**
- **How it works:** Custom slash commands defined in configuration with associated prompts and context
- **Configuration location:** `~/.continue/config.json` under `customCommands`
- **Features:** Context providers, custom prompts, model routing
- **Known limitations:** JSON format less expressive than Markdown for complex workflows

**OpenAI Assistants**
- **How it works:** Function calling allows tools to be invoked by the model. Tools defined via JSON Schema.
- **Configuration location:** Assistant configuration via API
- **Features:** Code interpreter, file search, custom functions
- **Known limitations:** Requires API integration; not file-based

---

### 1.4 MCP (Model Context Protocol) Support

| Tool | Support Level | Client | Server | Configuration |
|------|---------------|--------|--------|---------------|
| **Claude Code** | Full | Yes | N/A | `settings.json` |
| **Gemini Code Assist** | None | No | No | N/A |
| **GitHub Copilot** | None | No | No | N/A |
| **Cursor IDE** | Full | Yes | N/A | Settings JSON |
| **Windsurf** | Full | Yes | N/A | Native support |
| **Continue.dev** | Partial | Experimental | N/A | Context providers |
| **Aider** | None | No | No | N/A |
| **Amazon Q Developer** | None | No | No | N/A |
| **OpenAI Assistants** | None | No | No | N/A |

#### Detailed Analysis

**Claude Code (Full Support)**
- **How it works:** Native MCP client implementation. MCP servers configured in settings and started automatically.
- **Configuration location:** `.claude/settings.json` under `mcpServers`
- **Capabilities:** Resources, Tools, Prompts, Sampling
- **Transport:** stdio (local), HTTP+SSE (remote)

**Example Configuration:**
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

**Cursor IDE (Full Support)**
- **How it works:** MCP client support for tool and resource integration
- **Configuration location:** Cursor settings JSON
- **Known limitations:** Implementation details may vary from Claude Code

**Windsurf (Full Support)**
- **How it works:** Native MCP support integrated with Cascade system
- **Configuration location:** Windsurf settings
- **Known limitations:** May prioritize proprietary Cascade over MCP

**Continue.dev (Partial Support)**
- **How it works:** Context providers offer similar functionality to MCP resources
- **Configuration location:** `config.json` under `contextProviders`
- **Known limitations:** Not full MCP protocol compliance; conceptually similar

---

### 1.5 Git Hooks Integration

| Tool | Supported | Hook Types | Configuration | Auto-Setup |
|------|-----------|------------|---------------|------------|
| **Claude Code** | Yes | Pre-commit, commit-msg | Native git hooks | No |
| **Gemini Code Assist** | Partial | Via IDE | IDE settings | No |
| **GitHub Copilot** | Partial | Via GitHub Actions | `.github/workflows/` | No |
| **Cursor IDE** | Partial | Via VS Code | Task configuration | No |
| **Windsurf** | Partial | Via IDE | IDE settings | No |
| **Continue.dev** | Partial | Via VS Code | Task configuration | No |
| **Aider** | Yes | Auto-commit | `.aider.conf.yml` | Yes |
| **Amazon Q Developer** | Partial | Via AWS | CodeCommit hooks | No |
| **OpenAI Assistants** | No | N/A | N/A | N/A |

#### Detailed Analysis

**Claude Code**
- **How it works:** Respects existing git hooks; can be configured to run hooks on commits; skills can define hook behaviors
- **Configuration location:** Standard `.git/hooks/`, skill definitions
- **Features:** Honors pre-commit hooks; waits for hook completion; reports hook failures
- **Known limitations:** Does not auto-install hooks; hooks must be pre-configured

**Aider**
- **How it works:** Auto-commits changes after successful edits; integrates with git workflow
- **Configuration location:** `.aider.conf.yml` (`auto-commits: true/false`)
- **Features:** Automatic commit generation; descriptive commit messages
- **Known limitations:** May create many small commits; requires git repository

---

### 1.6 Workspace/Project Awareness

| Tool | Supported | Detection Method | Monorepo Support | Multi-Root |
|------|-----------|------------------|------------------|------------|
| **Claude Code** | Yes | Working directory | Yes | Manual |
| **Gemini Code Assist** | Yes | IDE workspace | Yes | IDE-dependent |
| **GitHub Copilot** | Yes | IDE workspace | Partial | IDE-dependent |
| **Cursor IDE** | Yes | Workspace indexing | Yes | Yes |
| **Windsurf** | Yes | Workspace indexing | Yes | Yes |
| **Continue.dev** | Yes | VS Code workspace | Yes | Yes |
| **Aider** | Yes | Git repository | Partial | No |
| **Amazon Q Developer** | Yes | IDE + AWS context | Yes | IDE-dependent |
| **OpenAI Assistants** | Partial | File uploads | No | No |

#### Detailed Analysis

**Claude Code**
- **How it works:** Aware of current working directory and git repository structure. Reads project configuration from directory hierarchy.
- **Features:**
  - Hierarchical CLAUDE.md discovery
  - Git worktree awareness
  - Package.json/requirements.txt detection
- **Known limitations:** CLI-based; manual directory navigation

**Cursor IDE**
- **How it works:** Full workspace indexing with semantic embeddings. Understands project structure via file patterns.
- **Features:**
  - Automatic codebase indexing
  - Multi-root workspace support
  - @-mention for file/folder references
- **Known limitations:** Large codebases may have indexing delays

---

### 1.7 Multi-File Editing

| Tool | Supported | Mode | Preview | Undo Support |
|------|-----------|------|---------|--------------|
| **Claude Code** | Yes | Sequential + Parallel | Yes | Git-based |
| **Gemini Code Assist** | Yes | Parallel | Yes | IDE undo |
| **GitHub Copilot** | Yes | Workspace (preview) | Yes | IDE undo |
| **Cursor IDE** | Yes | Composer | Yes | IDE undo |
| **Windsurf** | Yes | Cascade | Yes | IDE undo |
| **Continue.dev** | Yes | Edit mode | Yes | IDE undo |
| **Aider** | Yes | Native | No | Git revert |
| **Amazon Q Developer** | Partial | IDE-based | Yes | IDE undo |
| **OpenAI Assistants** | Partial | API-based | No | Manual |

#### Detailed Analysis

**Claude Code**
- **How it works:** Edit tool modifies files via exact string replacement. Can make multiple edits across files in single response.
- **Features:**
  - Exact match replacement
  - Preview before apply (via diff)
  - Git integration for undo
- **Known limitations:** Requires unique match strings; no fuzzy matching

**Cursor IDE (Composer)**
- **How it works:** Composer mode allows AI to plan and execute changes across multiple files
- **Features:**
  - Multi-file change planning
  - Visual diff preview
  - Accept/reject per-file
- **Known limitations:** May struggle with very large change sets

**Aider**
- **How it works:** Native multi-file editing with git integration
- **Features:**
  - Automatic file detection
  - Git commit per change
  - Undo via git revert
- **Known limitations:** No visual preview; terminal-based

---

### 1.8 Terminal/Shell Access

| Tool | Supported | Execution Mode | Sandboxing | Timeout |
|------|-----------|----------------|------------|---------|
| **Claude Code** | Yes | Direct | Configurable | Yes |
| **Gemini Code Assist** | Partial | IDE terminal | IDE sandbox | IDE-based |
| **GitHub Copilot** | Partial | Via Workspace | GitHub sandbox | Yes |
| **Cursor IDE** | Partial | IDE terminal | None | No |
| **Windsurf** | Yes | Cascade | Configurable | Yes |
| **Continue.dev** | Partial | VS Code terminal | None | No |
| **Aider** | Yes | Direct | None | No |
| **Amazon Q Developer** | Partial | IDE terminal | AWS sandbox | Yes |
| **OpenAI Assistants** | Partial | Code Interpreter | Sandboxed | Yes |

#### Detailed Analysis

**Claude Code**
- **How it works:** Bash tool executes commands in user's shell environment. Supports background execution and timeout configuration.
- **Features:**
  - Full shell access
  - Environment variable support
  - Background process execution
  - Configurable permissions
- **Configuration:** Permissions in `settings.json`
- **Known limitations:** Requires explicit permission grants; potential security implications

**OpenAI Code Interpreter**
- **How it works:** Sandboxed Python execution environment with file access
- **Features:**
  - Safe execution sandbox
  - File upload/download
  - Library access (pre-installed)
- **Known limitations:** Python only; limited system access; no persistent state

---

### 1.9 Custom Tool Definitions

| Tool | Supported | Definition Format | Invocation | Dynamic |
|------|-----------|-------------------|------------|---------|
| **Claude Code** | Yes | MCP + Skills | Auto/Manual | Yes |
| **Gemini Code Assist** | Partial | Function calling | Auto | Yes |
| **GitHub Copilot** | Partial | Extensions | Integrated | Limited |
| **Cursor IDE** | Yes | MCP | Auto | Yes |
| **Windsurf** | Yes | MCP + Native | Auto | Yes |
| **Continue.dev** | Yes | Context providers | Manual | Yes |
| **Aider** | No | N/A | N/A | No |
| **Amazon Q Developer** | Partial | AWS integrations | Integrated | Limited |
| **OpenAI Assistants** | Yes | JSON Schema | Auto | Yes |

#### Detailed Analysis

**Claude Code**
- **How it works:** Tools defined via MCP servers (external) or skills (internal). Model automatically selects appropriate tools.
- **Definition formats:**
  - MCP: JSON schema tool definitions
  - Skills: Markdown workflow definitions
- **Features:**
  - Dynamic tool discovery
  - Permission controls
  - Tool result integration

**MCP Tool Definition Example:**
```json
{
  "name": "search_codebase",
  "description": "Search for code patterns across the repository",
  "inputSchema": {
    "type": "object",
    "properties": {
      "pattern": {
        "type": "string",
        "description": "Regex pattern to search for"
      },
      "file_type": {
        "type": "string",
        "description": "File extension filter"
      }
    },
    "required": ["pattern"]
  }
}
```

**OpenAI Assistants**
- **How it works:** Functions defined via JSON Schema, invoked automatically by model
- **Features:**
  - Structured function definitions
  - Automatic parameter extraction
  - Parallel function calling

---

### 1.10 API/Programmatic Access

| Tool | Supported | API Type | Authentication | SDK |
|------|-----------|----------|----------------|-----|
| **Claude Code** | Yes | CLI + MCP | API key | TypeScript |
| **Gemini Code Assist** | Yes | REST/gRPC | OAuth/API key | Python, JS |
| **GitHub Copilot** | Partial | GitHub API | GitHub token | REST |
| **Cursor IDE** | No | N/A | N/A | N/A |
| **Windsurf** | No | N/A | N/A | N/A |
| **Continue.dev** | Partial | Config-based | Various | TypeScript |
| **Aider** | Yes | CLI + Python | API keys | Python |
| **Amazon Q Developer** | Yes | AWS SDK | AWS IAM | Multi-lang |
| **OpenAI Assistants** | Yes | REST | API key | Python, JS |

#### Detailed Analysis

**Claude Code**
- **How it works:** CLI tool with JSON output mode; MCP servers provide programmatic integration
- **Access methods:**
  - CLI with flags (`--output json`)
  - MCP server integration
  - Direct Claude API
- **Authentication:** Anthropic API key
- **Known limitations:** No dedicated SDK; relies on CLI or direct API

**OpenAI Assistants**
- **How it works:** Full REST API with SDKs for common languages
- **Features:**
  - Thread management
  - Run execution
  - File handling
  - Streaming responses
- **Authentication:** OpenAI API key
- **SDKs:** Python, JavaScript/TypeScript, .NET, Java

---

## 2. Configuration Location Summary

| Tool | Primary Config | Secondary Config | MCP Config | User Config |
|------|---------------|------------------|------------|-------------|
| **Claude Code** | `CLAUDE.md` | `.claude/settings.json` | `.claude/settings.json` | `~/.claude/` |
| **Gemini Code Assist** | `.gemini/settings.json` | IDE settings | N/A | Google account |
| **GitHub Copilot** | `.github/copilot-instructions.md` | VS Code settings | N/A | GitHub settings |
| **Cursor IDE** | `.cursorrules` | `.cursor/settings.json` | Cursor settings | `~/.cursor/` |
| **Windsurf** | `.windsurfrules` | `.windsurf/` | Windsurf settings | IDE settings |
| **Continue.dev** | `.continuerc.json` | N/A | `config.json` | `~/.continue/` |
| **Aider** | `.aider.conf.yml` | N/A | N/A | `~/.aider.conf.yml` |
| **Amazon Q Developer** | `.amazonq/` | AWS Toolkit | N/A | AWS profile |
| **OpenAI Assistants** | API config | N/A | N/A | Dashboard |

---

## 3. Interoperability Analysis

### 3.1 Features That Can Be Abstracted

These features have sufficient commonality across tools to support abstraction layers:

| Feature | Abstraction Potential | Common Format | Notes |
|---------|----------------------|---------------|-------|
| **Rules/Instructions** | High | Markdown | All major tools support Markdown rules |
| **Code Style Guidelines** | High | Markdown | Universal concept; content portable |
| **Project Context** | Medium | Markdown/YAML | Structure varies but content similar |
| **File Patterns** | High | Glob patterns | Standard glob syntax supported |
| **MCP Tools** | Medium | JSON Schema | For tools supporting MCP |
| **Test Commands** | High | Shell commands | Universal execution model |

### 3.2 Tool-Specific Features

These features are proprietary and cannot be easily abstracted:

| Feature | Tool(s) | Why Not Portable |
|---------|---------|------------------|
| **@import syntax** | Claude Code | Claude-specific include mechanism |
| **@codebase queries** | Cursor | Cursor's embedding/RAG system |
| **Cascade memory** | Windsurf | Proprietary memory formation |
| **Code Interpreter** | OpenAI | Sandboxed execution environment |
| **AWS integrations** | Amazon Q | AWS service dependencies |
| **Thread persistence** | OpenAI | API-specific storage |
| **Copilot Extensions** | GitHub | GitHub ecosystem lock-in |

### 3.3 Common Denominator Features

Features supported by all/most tools that form a reliable baseline:

| Feature | Support Level | Implementation Notes |
|---------|--------------|---------------------|
| **Markdown rules** | Universal | All tools parse Markdown |
| **Project-level config** | Universal | All support per-project config |
| **Code completion** | Universal | Core feature of all tools |
| **Multi-file awareness** | Universal | All index/understand projects |
| **Natural language chat** | Universal | All support conversational interface |
| **Code explanation** | Universal | All can explain code |
| **Test generation** | High | Most tools support this |
| **Refactoring** | High | Most tools handle refactoring |

---

## 4. Migration Pathways

### 4.1 From Cursor to Claude Code

| Cursor Feature | Claude Code Equivalent | Migration Steps |
|---------------|----------------------|-----------------|
| `.cursorrules` | `CLAUDE.md` | Copy content; adjust syntax |
| @codebase | Grep/Glob tools | Use explicit search |
| Composer | Multi-file Edit | Same capability, different UX |
| Chat history | `.claude/memory.md` | Manual memory migration |

### 4.2 From Copilot to Claude Code

| Copilot Feature | Claude Code Equivalent | Migration Steps |
|----------------|----------------------|-----------------|
| `copilot-instructions.md` | `CLAUDE.md` | Direct content transfer |
| Inline completions | N/A | Different paradigm (agentic) |
| GitHub Actions | Skills + Bash | Define equivalent workflows |

### 4.3 Migration Checklist Template

```markdown
# Migration Checklist

## Rules/Instructions
- [ ] Export existing rules to Markdown
- [ ] Identify tool-specific syntax
- [ ] Create common rules document
- [ ] Generate tool-specific files

## Context/Memory
- [ ] Document key project decisions
- [ ] Create architecture summary
- [ ] Export any exportable memory
- [ ] Recreate in target format

## Skills/Automations
- [ ] List existing automations
- [ ] Map to target tool capabilities
- [ ] Reimplement in target format
- [ ] Test equivalence
```

---

## 5. Observed Standardization Trends

| Trend | Observed Timeline | Current State |
|-------|-------------------|---------------|
| MCP adoption | 2026-2027 | Some tools adding support |
| Rules format convergence | 2026-2027 | Markdown widely used |
| Memory portability | 2027+ | No standards yet |
| Skill interoperability | 2027+ | No standards yet |

---

## Appendix A: Quick Reference Cards

### Claude Code Configuration
```
./CLAUDE.md                    # Project instructions
./CLAUDE.local.md              # Personal (gitignored)
./.claude/
├── settings.json              # Tool settings, MCP servers
├── settings.local.json        # Personal settings
├── rules/
│   └── *.md                   # Modular rules
└── skills/
    └── *.md                   # Custom skills
~/.claude/
├── CLAUDE.md                  # User-wide instructions
├── settings.json              # Global settings
└── memory.md                  # Global memory
```

### Cursor Configuration
```
./.cursorrules                 # Project rules (legacy)
./.cursor/
├── rules                      # Project rules (current)
└── settings.json              # Cursor settings
~/.cursor/
└── rules                      # Global rules
```

### GitHub Copilot Configuration
```
./.github/
└── copilot-instructions.md    # Repository instructions
# Plus: GitHub account settings for personal instructions
# Plus: Organization settings for org-wide instructions
```

### Continue.dev Configuration
```
~/.continue/
├── config.json                # Global configuration
└── config.ts                  # TypeScript config (optional)
./.continuerc.json             # Project overrides
```

### Aider Configuration
```
./.aider.conf.yml              # Project configuration
~/.aider.conf.yml              # User configuration
# Plus: Command-line flags override all
```

---

## Appendix B: MCP Server Registry

Common MCP servers useful across tools:

| Server | Purpose | Package |
|--------|---------|---------|
| Filesystem | File operations | `@modelcontextprotocol/server-filesystem` |
| PostgreSQL | Database access | `@modelcontextprotocol/server-postgres` |
| SQLite | Database access | `@modelcontextprotocol/server-sqlite` |
| GitHub | Repository access | `@modelcontextprotocol/server-github` |
| Brave Search | Web search | `@modelcontextprotocol/server-brave-search` |
| Memory | Persistent memory | `@modelcontextprotocol/server-memory` |
| Puppeteer | Browser automation | `@modelcontextprotocol/server-puppeteer` |
| Slack | Team communication | `@modelcontextprotocol/server-slack` |

---

## Appendix C: Research Methodology

This matrix was compiled using:
1. Official documentation from tool vendors
2. MCP specification (v2025-11-25)
3. Existing research documents in this repository
4. Public configuration file examples
5. Community knowledge and patterns

**Items marked [VERIFY]** indicate areas where documentation was unclear or potentially outdated. These should be validated against current tool versions before relying on them for critical decisions.

---

*Document created: 2026-01-23*
*Last updated: 2026-01-23*
*Status: Comprehensive research - [VERIFY] items need validation*
*Branch: research-docs*
