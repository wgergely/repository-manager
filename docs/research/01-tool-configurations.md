# Agentic Coding Tool Configuration Landscape (2026)

## Research Methodology Note

This document synthesizes information about agentic coding tool configurations based on available knowledge through early 2025. The rapidly evolving nature of this space means some details may have changed. Sections marked with `[VERIFY]` should be validated against current documentation.

---

## 1. Claude Code (Anthropic)

### Overview
Claude Code is Anthropic's official CLI and agentic coding tool, designed for terminal-based AI-assisted development with strong emphasis on safety and transparency.

### Configuration Files

#### CLAUDE.md (Project Instructions)
- **Location**: Repository root, or any directory in the path hierarchy
- **Format**: Markdown
- **Purpose**: Project-specific instructions, conventions, and context

```markdown
# CLAUDE.md Example

## Project Overview
This is a TypeScript monorepo using pnpm workspaces.

## Code Style
- Use functional components with hooks
- Prefer named exports over default exports
- Always use strict TypeScript

## Commands
- Build: `pnpm build`
- Test: `pnpm test`
- Lint: `pnpm lint`

## Architecture
[Description of project structure and patterns]
```

**Key Features:**
- Hierarchical: Files are read from all directories in the path
- Merged: Multiple CLAUDE.md files combine (child overrides parent for conflicts)
- Markdown-native: Uses standard Markdown with optional sections

#### .claude/ Directory
- **Location**: Repository root or home directory
- **Purpose**: Configuration, memory, and settings

```
.claude/
├── settings.json          # Local settings
├── settings.local.json    # User-specific (gitignored)
├── memory.md              # Persistent memory/context
└── skills/                # Custom skill definitions
    └── my-skill.md
```

#### settings.json
```json
{
  "permissions": {
    "allow": ["Bash", "Read", "Write", "Edit"],
    "deny": ["WebFetch"]
  },
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@anthropic/mcp-server-context7"]
    }
  }
}
```

#### Memory System
- **File**: `.claude/memory.md` or project-level memory
- **Format**: Markdown
- **Persistence**: Stored between sessions
- **Scope**: Can be global (~/.claude/) or project-specific

### Skills/Plugin Architecture
- **Location**: `.claude/skills/` or referenced via skill tool
- **Format**: Markdown files with structured instructions
- **Invocation**: Via `/skill-name` commands or automatic detection

### MCP (Model Context Protocol) Support
- **Full support** for MCP servers
- Configured in `settings.json` under `mcpServers`
- Enables external tool integration (databases, APIs, specialized tools)

### Context Management
- Hierarchical file reading (CLAUDE.md files in path)
- Explicit context via Read tool
- MCP-based context providers
- Conversation history within session

---

## 2. Cursor IDE

### Overview
Cursor is a VS Code fork with integrated AI capabilities, popular for its .cursorrules system and AI-first editing experience.

### Configuration Files

#### .cursorrules (Legacy) / .cursor/rules (Current)
- **Location**: Repository root
- **Format**: Plain text or Markdown
- **Purpose**: Project-specific AI behavior rules

```markdown
# .cursorrules Example

You are an expert TypeScript developer working on a Next.js 14 application.

## Code Style
- Always use TypeScript strict mode
- Prefer server components unless client interactivity is needed
- Use Tailwind CSS for styling

## Patterns
- Use the app router exclusively
- Implement error boundaries for all pages
- Use Zod for runtime validation

## Don't
- Never use `any` type
- Don't create files in the pages/ directory
- Avoid inline styles
```

#### .cursor/ Directory [VERIFY]
```
.cursor/
├── rules              # Project rules (replaces .cursorrules)
├── settings.json      # Cursor-specific settings
└── prompts/           # Custom prompt templates
```

#### Global Rules
- **Location**: `~/.cursor/rules` or Cursor settings UI
- **Scope**: Applied to all projects unless overridden

### Context Management
- **Codebase Indexing**: Automatic embedding of repository
- **@-mentions**: Reference files, folders, docs, or web content
- **Composer Context**: Multi-file editing context
- **Chat History**: Persistent within project

### RAG/Embeddings
- Uses vector embeddings for semantic code search
- Indexes entire codebase on project open
- Supports `@codebase` for semantic queries

### Plugin Architecture [VERIFY]
- VS Code extension compatibility
- Custom commands via settings
- No formal skill/plugin system like Claude Code

### MCP Support
- **Status**: Limited or experimental [VERIFY]
- May support MCP through extensions

---

## 3. Windsurf (Codeium)

### Overview
Windsurf is Codeium's AI-native IDE, known for its "Cascade" agentic system and flow-based coding experience.

### Configuration Files

#### .windsurfrules / rules.md [VERIFY]
- **Location**: Repository root
- **Format**: Markdown
- **Purpose**: Project-specific instructions

```markdown
# Windsurf Rules

## Project Context
This is a Python FastAPI backend with SQLAlchemy ORM.

## Conventions
- Use async/await for all database operations
- Follow PEP 8 strictly
- Type hints required for all functions

## Architecture
- Repository pattern for data access
- Service layer for business logic
- Pydantic models for validation
```

#### Cascade Memory
- **Persistent context** across sessions
- Stores decisions, patterns, and learned behaviors
- Automatic memory management

### Context Management
- **Flows**: Contextual understanding of task sequences
- **Cascade**: Agentic reasoning with multi-step planning
- **Supercomplete**: Context-aware completions

### Memory System
- Automatic memory formation from interactions
- Project-specific memory storage
- Memory retrieval during relevant tasks

### Plugin Architecture
- Extension system (VS Code compatible)
- Custom tool definitions [VERIFY]
- Integration with external services

### MCP Support
- **Status**: Unknown/Limited [VERIFY]
- Focus on proprietary Cascade system

---

## 4. GitHub Copilot / Copilot Workspace

### Overview
GitHub Copilot is the most widely adopted AI coding assistant. Copilot Workspace extends this with agentic capabilities for issue-to-PR workflows.

### Configuration Files

#### .github/copilot-instructions.md
- **Location**: `.github/` directory in repository root
- **Format**: Markdown
- **Purpose**: Repository-wide Copilot behavior instructions

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

#### Personal Instructions
- **Location**: GitHub account settings
- **Scope**: Applies to all repositories for that user
- **Format**: Plain text

#### Organization Instructions [VERIFY]
- **Location**: Organization settings
- **Scope**: Applies to all repos in the organization

### Copilot Workspace Configuration
- Issue-based context
- Automatic spec generation
- Plan-then-implement workflow

### Context Management
- File-level context from open editors
- Repository context via indexing
- Issue/PR context in Workspace mode

### Plugin Architecture
- GitHub Actions integration
- Copilot Extensions (preview) [VERIFY]
- VS Code extension ecosystem

### MCP Support
- **Status**: Not natively supported (as of early 2025)
- GitHub's focus is on native integrations

---

## 5. Gemini Code Assist (Google)

### Overview
Google's AI coding assistant, integrated into various Google Cloud and IDE environments.

### Configuration Files [VERIFY]

#### .gemini/config.yaml or .gemini-assist/ [VERIFY]
- **Location**: Repository root or IDE settings
- **Format**: YAML (likely)
- **Status**: Configuration system may still be evolving

```yaml
# Hypothetical .gemini/config.yaml
project:
  language: python
  framework: django

rules:
  - Use type hints for all functions
  - Follow Google Python style guide
  - Prefer dataclasses over plain dicts

context:
  include:
    - src/
    - tests/
  exclude:
    - node_modules/
    - .venv/
```

### Context Management
- Integration with Google Cloud resources
- Repository indexing
- IDX environment awareness

### Plugin Architecture
- Google Cloud integrations
- Firebase/Vertex AI connections
- IDE-specific extensions

### MCP Support
- **Status**: Unknown [VERIFY]
- Google has its own AI infrastructure priorities

---

## 6. Amazon Q Developer

### Overview
AWS's AI coding assistant, deeply integrated with AWS services and development workflows.

### Configuration Files [VERIFY]

#### .amazonq/ or AWS configuration
- **Location**: Repository or AWS profile
- **Format**: JSON/YAML
- Integration with AWS CDK and CloudFormation

```json
{
  "amazonq": {
    "projectType": "serverless",
    "runtime": "nodejs18.x",
    "preferences": {
      "useTypeScript": true,
      "testFramework": "jest"
    }
  }
}
```

### Context Management
- AWS resource awareness
- CloudFormation/CDK context
- Repository indexing

### Plugin Architecture
- AWS Toolkit integration
- CodeWhisperer customizations
- Enterprise customization options

### MCP Support
- **Status**: Unknown [VERIFY]
- Focus on AWS service integrations

---

## 7. Other Notable Tools

### Aider
- **Config**: `.aider.conf.yml` in repo root or home directory
- **Format**: YAML
- **Memory**: Git-based, commits as checkpoints
- **Rules**: Via config file or command-line flags

```yaml
# .aider.conf.yml
model: claude-3-opus
auto-commits: true
gitignore: true
conventions:
  - Always write tests
  - Use type hints
```

### Continue.dev
- **Config**: `~/.continue/config.json` and `.continuerc.json`
- **Format**: JSON
- **Extensible**: Custom slash commands and context providers
- **MCP**: Supports context providers similar to MCP

```json
{
  "models": [...],
  "customCommands": [...],
  "contextProviders": [
    {
      "name": "code",
      "params": { "nRetrieve": 25 }
    }
  ]
}
```

### Cody (Sourcegraph)
- **Config**: VS Code settings or `.cody/` directory [VERIFY]
- **Context**: Sourcegraph code intelligence
- **Enterprise**: Custom context and guardrails

### Tabnine
- **Config**: IDE settings and `.tabnine/` [VERIFY]
- **Privacy**: Local/cloud model options
- **Team**: Shared team configurations

---

## Comparative Analysis

### Configuration File Formats

| Tool | Primary Format | File Name | Location |
|------|---------------|-----------|----------|
| Claude Code | Markdown | CLAUDE.md | Root/hierarchy |
| Cursor | Markdown/Text | .cursorrules | Root |
| Windsurf | Markdown | .windsurfrules | Root |
| GitHub Copilot | Markdown | copilot-instructions.md | .github/ |
| Gemini | YAML (likely) | config.yaml | .gemini/ |
| Amazon Q | JSON/YAML | Various | .amazonq/ |
| Aider | YAML | .aider.conf.yml | Root/home |
| Continue | JSON | config.json | ~/.continue/ |

### Key Patterns Observed

1. **Markdown Dominance**: Most tools favor Markdown for rules/instructions
   - Human-readable and version-control friendly
   - Supports structured sections naturally
   - Easy for AI models to parse

2. **Hierarchical Configuration**:
   - Claude Code: Full hierarchy support
   - Others: Typically root-only

3. **Global vs Project Settings**:
   - All tools support some form of global defaults
   - Project-level always overrides global

4. **Memory/Persistence**:
   - Claude Code: Explicit memory.md
   - Windsurf: Automatic Cascade memory
   - Others: Session-based or limited

### MCP Support Matrix

| Tool | MCP Support | Notes |
|------|-------------|-------|
| Claude Code | Full | Native support |
| Cursor | Limited | Via extensions |
| Windsurf | Unknown | Proprietary system |
| GitHub Copilot | No | GitHub-native integrations |
| Gemini | Unknown | Google Cloud focused |
| Continue.dev | Partial | Context providers |

---

## Cross-Platform Interoperability Observations

### Rules Portability
- Each tool uses different rule file formats and names
- Markdown-based rules have the most overlap across tools
- No standard rules schema currently exists

### Memory Sharing
- Memory formats are proprietary to each tool
- Semantic meaning varies between implementations
- MCP provides a potential protocol layer, though adoption varies

### Skills/Plugins
- Skill definitions are tool-specific
- Claude Code uses Markdown-based skills
- Other tools use various extension formats
- MCP defines a standard for tool definitions, but implementation varies

---

## Research Gaps

### Items Requiring Verification [VERIFY Items]
1. Windsurf exact configuration file names and format
2. Gemini Code Assist configuration system
3. Amazon Q Developer local configuration
4. Cursor's current .cursor/ directory structure
5. MCP support status for non-Claude tools

### Areas With Limited Documentation
1. Emerging standards initiatives
2. Tool vendor interoperability discussions
3. Community-driven configuration schemas
4. Enterprise/team configuration patterns

---

## Sources & References

### Official Documentation
- Anthropic Claude Code: docs.anthropic.com/en/docs/claude-code
- GitHub Copilot: docs.github.com/en/copilot
- Cursor: cursor.com/docs
- Windsurf/Codeium: codeium.com/windsurf
- Continue.dev: continue.dev/docs

### Community Resources
- Reddit: r/ClaudeAI, r/cursor, r/LocalLLaMA
- Discord: Tool-specific servers
- GitHub: Configuration file examples in public repos

---

*Document created: 2026-01-23*
*Last updated: 2026-01-23*
*Status: Initial research - verification needed for [VERIFY] items*
*Branch: research-docs*
