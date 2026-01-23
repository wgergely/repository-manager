# Cross-Platform Interoperability for Agentic Coding Tools (2026)

## Executive Summary

The agentic coding tool landscape in 2026 shows significant fragmentation in configuration formats, memory systems, and skill/plugin architectures. While MCP (Model Context Protocol) emerges as the most promising interoperability standard, adoption remains uneven across vendors. This document analyzes the current state of cross-platform compatibility and identifies both progress and gaps.

---

## 1. Interoperability Analysis

### 1.1 Rules File Compatibility

**Current State: Fragmented but Converging**

| Tool | Rules File | Format | Portable? |
|------|-----------|--------|-----------|
| Claude Code | CLAUDE.md, .claude/rules/*.md | Markdown | Partially |
| Cursor | .cursorrules, .cursor/rules | Markdown/Text | Partially |
| Windsurf | .windsurf/rules/, Rulebooks | Markdown | Partially |
| GitHub Copilot | .github/copilot-instructions.md | Markdown | Partially |
| Gemini Code Assist | .gemini/ folder, GEMINI.md, styleguide.md | Markdown/JSON | Partially |
| Amazon Q | .amazonq/default.json | JSON | No |
| **AGENTS.md** | AGENTS.md (root) | Markdown | **Yes** |

**Can the Same Rules File Work Across Tools?**

**Short Answer**: Not directly, but with adaptation strategies.

**Analysis**:
1. **Format Alignment**: Most tools have converged on Markdown for rules, making content portable even if filenames differ
2. **Semantic Overlap**: Core instructions (code style, architecture, testing) translate well across tools
3. **Tool-Specific Features**: Features like Claude's YAML frontmatter for path-specific rules, or Cursor's @-mention syntax, don't transfer
4. **AGENTS.md Emergence**: A universal standard (AGENTS.md) has emerged with cross-vendor support, providing true portability

### 1.1.1 AGENTS.md - The Universal Standard

**Major Development**: In mid-2025, Google, OpenAI, Factory, Sourcegraph, and Cursor jointly launched **AGENTS.md** - a simple, open format for guiding coding agents.

**Key Facts**:
- **Adoption**: 20,000+ repositories on GitHub
- **Governance**: Stewarded by the Agentic AI Foundation under the Linux Foundation
- **Official Site**: https://agents.md/

**Tool Support**:
| Tool | AGENTS.md Support |
|------|------------------|
| OpenAI Codex | Native |
| Google Jules | Native |
| Cursor | Native |
| GitHub Copilot | Native |
| Aider | Native |
| RooCode | Native |
| Zed | Native |
| Factory AI | Native |
| Claude Code | Compatible (reads as context) |

**Structure Best Practices**:
Successful AGENTS.md files cover six core areas:
1. **Commands** - Build, test, lint commands
2. **Testing** - How to run and write tests
3. **Project Structure** - Directory layout and conventions
4. **Code Style** - Formatting, naming, patterns
5. **Git Workflow** - Branching, commits, PRs
6. **Boundaries** - What the agent should NOT do

**Example AGENTS.md**:
```markdown
# AGENTS.md

## Build & Test
- `npm install` to install dependencies
- `npm test` to run tests
- `npm run lint` to check code style

## Project Structure
- `src/` - Source code
- `tests/` - Test files (mirror src/ structure)
- `docs/` - Documentation

## Code Style
- TypeScript strict mode
- Prefer functional patterns
- Use named exports

## Git Workflow
- Branch from `main`
- Conventional commits (feat:, fix:, docs:)
- PRs require one approval

## Boundaries
- Never modify package-lock.json manually
- Don't change CI/CD configuration without asking
```

**Implication for repo-manager**: AGENTS.md should be a first-class citizen - the tool should be able to read from and sync to AGENTS.md as a universal source.

### 1.2 Emerging Standards for Agentic Tool Configuration

**MCP (Model Context Protocol) - The Leading Standard**

MCP represents the most significant standardization effort in the agentic tool space:

- **Version**: 2025-11-25 (latest as of research date)
- **Governance**: Open specification with versioning and working groups
- **Analogy**: "USB-C for AI applications" - a universal connector standard

**MCP Architecture**:
```
┌─────────────────┐     ┌─────────────────┐
│   MCP Client    │────▶│   MCP Server    │
│  (AI Tool/IDE)  │◀────│  (Data/Tools)   │
└─────────────────┘     └─────────────────┘
      Claude Code            Databases
      Cursor                 File Systems
      Continue.dev           APIs
      Custom Apps            Specialized Tools
```

**MCP Specification Coverage**:
- Base Protocol: Lifecycle, transports, authorization, security
- Client Features: Roots, sampling, elicitation
- Server Features: Prompts, resources, tools, completion, logging, pagination
- Utilities: Cancellation, ping, progress, tasks

**Other Standardization Efforts**:
- **OpenAI Function Calling**: De facto standard for tool definitions, but not a full protocol
- **Google Gemini Function Calling**: Similar to OpenAI, Google-ecosystem focused
- **LangChain Tool Definitions**: Popular in open-source, but framework-specific

### 1.3 MCP's Role in Interoperability

**What MCP Enables**:
1. **Shared Tool Definitions**: One MCP server can serve multiple AI clients
2. **Universal Data Access**: Databases, filesystems, APIs accessible to any MCP-compatible tool
3. **Plugin Portability**: MCP servers work across Claude Code, Cursor (partial), Continue.dev
4. **Context Sharing**: Resources and prompts defined once, used everywhere

**Current MCP Adoption (January 2026)**:

| Tool | MCP Support | Notes |
|------|-------------|-------|
| Claude Code | Full Native | First-class citizen, configured in settings.json |
| Claude Desktop | Full Native | Full client support |
| Cursor | Full Native | Native client support, 40 tool limit, one-click install |
| Windsurf | Full Native | Native client support |
| Zed | Full Native | Native client support |
| Continue.dev | Partial | Context providers similar concept |
| VS Code (Copilot) | Limited | Experimental support |
| JetBrains AI | Partial | Server support |
| Amazon Q | Native | MCP configuration in IDE settings |

**MCP Ecosystem Growth**:
- 100+ official/community MCP servers available
- SDKs for Python, TypeScript, Rust, Go
- MCP Inspector tool for debugging
- OpenAI adopted MCP in March 2025
- Google DeepMind adoption confirmed

**MCP Configuration in Claude Code**:
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
        "DATABASE_URL": "postgresql://..."
      }
    }
  }
}
```

### 1.4 Abstraction Layers and Adapters

**Current Solutions**:

1. **Manual Adapter Scripts**: Teams write scripts to generate tool-specific configs from a common source
   ```bash
   # Example: Generate tool configs from common rules
   ./scripts/generate-rules.sh
   # Creates: CLAUDE.md, .cursorrules, .github/copilot-instructions.md
   ```

2. **Symlink Strategies**: Link to shared rule files
   ```bash
   # Claude Code supports symlinks in .claude/rules/
   ln -s ~/shared-rules/common.md .claude/rules/shared.md
   ```

3. **Template Systems**: Use templating to generate tool-specific files
   ```yaml
   # rules-template.yaml
   common:
     code_style: |
       - Use TypeScript strict mode
       - Prefer functional patterns
   claude_specific:
     memory_imports: true
   cursor_specific:
     context_tags: ["@codebase"]
   ```

4. **MCP as Abstraction**: Use MCP servers to provide consistent context/tools across platforms

**Emerging Community Solutions**:
- **AGENTS.md ecosystem**: Official tooling and templates at https://github.com/agentsmd/agents.md
- **sammcj/agentic-coding**: Rules, templates and examples for multiple tools
- **VS Code extensions**: Multi-tool rule management extensions emerging
- **Tool-native converters**: Most tools can read AGENTS.md alongside their native formats

---

## 2. Memory and Context Sharing

### 2.1 How Different Tools Persist Context

**Claude Code Memory System** (Most Documented):

| Memory Type | Location | Scope | Shared? |
|-------------|----------|-------|---------|
| Managed Policy | System-level CLAUDE.md | Organization | Yes (IT-deployed) |
| Project Memory | ./CLAUDE.md | Team | Yes (git) |
| Project Rules | .claude/rules/*.md | Team | Yes (git) |
| User Memory | ~/.claude/CLAUDE.md | Personal | No |
| Project Local | ./CLAUDE.local.md | Personal+Project | No (gitignored) |

**Claude Code Memory Features**:
- Hierarchical loading with precedence
- Import syntax: `@path/to/file` for including other files
- Path-specific rules via YAML frontmatter
- Symlink support for shared rules
- Auto-gitignore for local files

**Other Tools' Memory Approaches**:

| Tool | Memory Persistence | Format | Portability |
|------|-------------------|--------|-------------|
| Claude Code | Explicit files | Markdown | Good (files) |
| Cursor | Session + indexing | Proprietary | Poor |
| Windsurf | Cascade memory | Proprietary | Poor |
| Copilot | Session-based | N/A | None |
| Continue.dev | Config-based | JSON | Medium |

### 2.2 Can Memories Be Shared Between Tools?

**Current Reality**: No native support for cross-tool memory sharing.

**Challenges**:
1. **Format Incompatibility**: Each tool uses different storage formats
2. **Semantic Differences**: "Memory" means different things (facts vs. preferences vs. context)
3. **No Standard Schema**: No agreed-upon structure for AI memories
4. **Privacy/Security**: Memory sharing raises data governance questions

**Potential Solutions**:

1. **Markdown-Based Memory (Most Portable)**:
   ```markdown
   # Project Memory

   ## Learned Patterns
   - The team prefers functional React components
   - API responses follow the JsonApi spec

   ## Decisions Made
   - 2026-01-15: Migrated from REST to GraphQL
   - 2026-01-20: Adopted Tailwind CSS

   ## Key Contacts
   - Backend: @alice
   - Frontend: @bob
   ```
   This can be manually shared and adapted across tools.

2. **MCP Memory Servers** (Emerging):
   ```json
   {
     "mcpServers": {
       "team-memory": {
         "command": "npx",
         "args": ["-y", "mcp-memory-server"],
         "env": {
           "MEMORY_PATH": "/shared/team-memory.json"
         }
       }
     }
   }
   ```
   Theoretically enables shared memory for MCP-compatible tools.

3. **External Knowledge Bases**:
   - Notion/Confluence with MCP adapters
   - Vector databases (Pinecone, Weaviate) as shared context
   - Custom memory services with API access

### 2.3 Portable Memory Formats

**No Established Standard Exists**

Proposed characteristics for a portable memory format:
- **Human-readable**: Markdown or structured YAML
- **Versioned**: Git-trackable
- **Categorized**: Clear separation of memory types
- **Time-stamped**: When memories were formed
- **Confidence-scored**: Reliability of memories
- **Source-tracked**: Where memory originated

**Hypothetical Portable Memory Schema**:
```yaml
# .agentic/memory.yaml
version: "1.0"
memories:
  - id: mem-001
    type: decision
    content: "Adopted TypeScript strict mode for all new code"
    timestamp: 2026-01-15T10:00:00Z
    confidence: high
    source: team-discussion

  - id: mem-002
    type: pattern
    content: "Use repository pattern for data access"
    timestamp: 2026-01-10T14:30:00Z
    confidence: medium
    source: code-review

  - id: mem-003
    type: preference
    content: "Prefer named exports over default exports"
    timestamp: 2026-01-12T09:00:00Z
    confidence: high
    source: style-guide
```

---

## 3. Skills/Capabilities Portability

### 3.1 Current State of Skills Across Platforms

**Claude Code Skills**:
- **Format**: Markdown files with structured instructions
- **Location**: .claude/skills/ or plugin directories
- **Invocation**: /skill-name or automatic detection
- **Features**: Can include code samples, multi-step workflows, tool permissions

**Example Claude Code Skill**:
```markdown
# commit

## Description
Create a well-formatted git commit with conventional commit messages.

## Instructions
1. Run `git status` to see changes
2. Run `git diff` to understand modifications
3. Create a commit message following conventional commits
4. Use co-author attribution

## Commit Format
<type>(<scope>): <description>

Types: feat, fix, docs, style, refactor, test, chore
```

**Other Tools' Skill/Plugin Systems**:

| Tool | Skill System | Format | Portability |
|------|--------------|--------|-------------|
| Claude Code | Native skills | Markdown | Claude-only |
| Cursor | VS Code extensions + MCP | TypeScript/JS | VS Code ecosystem |
| Copilot | Copilot Extensions (Skillsets & Agents) | JSON Schema / TypeScript | GitHub ecosystem |
| Continue.dev | Custom commands | JSON config | Continue-only |
| Windsurf | Rulebooks + MCP | Markdown | Windsurf + MCP ecosystem |
| Amazon Q | Custom agents + MCP | JSON | AWS ecosystem |

**GitHub Copilot Extensions** (Verified):
- Available on GitHub Marketplace with OAuth support
- Two types: **Skillsets** (lightweight, minimal setup) and **Agents** (full control, custom logic)
- SDK available (updated January 2026)
- Extensions work in GitHub.com, VS Code, and Visual Studio
- Partners include: Docker, MongoDB, Sentry, Stripe, Azure, Slack, Atlassian

### 3.2 Skill/Action Standardization

**No Universal Standard Exists**

**Closest to Standard**:
1. **MCP Tool Definitions**: Most promising for tool/action portability
2. **OpenAI Function Schema**: Widely adopted for function signatures
3. **JSON Schema**: Used as foundation by most approaches

**MCP Tool Definition Example**:
```json
{
  "name": "create_file",
  "description": "Create a new file with specified content",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "File path"
      },
      "content": {
        "type": "string",
        "description": "File content"
      }
    },
    "required": ["path", "content"]
  }
}
```

### 3.3 Emerging Universal Skill Formats

**Conceptual Universal Skill Schema** (Not Yet Standardized):

```yaml
# .agentic/skills/deploy.skill.yaml
apiVersion: agentic.dev/v1
kind: Skill
metadata:
  name: deploy
  description: Deploy application to production
  version: 1.0.0

spec:
  # Universal skill definition
  trigger:
    command: /deploy
    patterns: ["deploy to prod", "push to production"]

  inputs:
    - name: environment
      type: string
      enum: [staging, production]
      default: staging

  steps:
    - name: run-tests
      action: bash
      command: npm test

    - name: build
      action: bash
      command: npm run build

    - name: deploy
      action: bash
      command: ./scripts/deploy.sh ${{ inputs.environment }}

  # Tool-specific adaptations
  adaptations:
    claude:
      permissions: ["Bash(npm:*)", "Bash(./scripts/deploy.sh:*)"]
    cursor:
      context: ["@deployment-docs"]
    copilot:
      github_action: deploy.yml
```

---

## 4. Behavioral Drift Mitigation

### 4.1 Ensuring Consistent AI Behavior Across Tools

**The Drift Problem**:
- Same codebase, different tools, different AI behaviors
- Style inconsistencies in generated code
- Conflicting suggestions from different assistants
- Gradual deviation from established patterns

**Mitigation Strategies**:

**1. Single Source of Truth for Rules**:
Some teams maintain a common rules directory and generate tool-specific files from it.

**2. CI/CD Rule Validation**:
```yaml
# .github/workflows/validate-rules.yml
name: Validate AI Rules
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check rule consistency
        run: |
          ./scripts/validate-rules.sh
          # Ensures all tool-specific files match common rules
```

**3. Linting for AI-Generated Code**:
- Same ESLint/Prettier/Ruff config for all tools
- Pre-commit hooks catch deviations
- CI fails on style violations regardless of generation source

**4. Periodic Audits**:
- Compare outputs from different tools
- Document behavioral differences
- Update rules to address gaps

### 4.2 Configuration Drift Detection

**Manual Approaches**:
- Regular diff of tool-specific config files
- Checklist reviews during PR
- Scheduled sync scripts

**Automated Approaches**:
```bash
#!/bin/bash
# scripts/detect-drift.sh

# Generate expected configs from common source
./scripts/generate-rules.sh --dry-run > /tmp/expected

# Compare with actual
diff -r /tmp/expected ./ --include="*.cursorrules" --include="CLAUDE.md" --include="copilot-instructions.md"

if [ $? -ne 0 ]; then
    echo "Configuration drift detected!"
    exit 1
fi
```

### 4.3 Observed Patterns for Maintaining Standards

Teams addressing behavioral drift have been observed using various approaches:

1. **Version Control**: AI config files tracked in git alongside code
2. **Config Review**: AI configuration changes reviewed during PR process
3. **Deviation Documentation**: When tools require different configs, some teams document the rationale
4. **Multi-Tool Testing**: Verification of same tasks across different tools
5. **Linter Authority**: Linters/formatters serve as authoritative style enforcement regardless of AI tool used
6. **Synchronization Scripts**: Automated regeneration of tool-specific configs from common sources

---

## 5. Developer Community Discussions

### 5.1 Common Pain Points (Synthesized from Knowledge)

**Fragmentation Frustrations**:
- "I have to maintain 4 different rules files for the same project"
- "My Cursor rules don't work in Claude Code"
- "No way to share memories between my work (Copilot) and personal (Claude) setups"
- "MCP is great but only works with some tools"

**Interoperability Requests**:
- Universal rules format that all tools can read
- Shared memory/context protocol
- Skill marketplace that works across platforms
- Standard way to define tool permissions

**MCP Reception**:
- Generally positive as "finally, a standard"
- Frustration with uneven adoption
- Desire for more MCP servers in the ecosystem
- Questions about long-term governance

### 5.2 Community Solutions Being Built

**Open Source Projects**:
1. **AGENTS.md**: Universal standard with official tooling (https://github.com/agentsmd/agents.md)
2. **sammcj/agentic-coding**: Comprehensive rules and templates collection
3. **MCP Server Ecosystem**: 100+ servers providing cross-tool capabilities
4. **Tool-specific converters**: Community scripts for format translation

**DIY Solutions**:
- Teams building custom sync scripts
- Monorepo configs with generation pipelines
- Shared MCP server deployments
- Custom VS Code extensions for multi-tool management

### 5.3 Sentiment on Fragmentation

**General Sentiment**: Frustrated but Hopeful

**Negative**:
- "It's like the early browser wars all over again"
- "Every tool wants to be the standard"
- "Vendor lock-in through configuration formats"
- "Wasted effort maintaining multiple configs"

**Positive**:
- "MCP is the right direction"
- "At least most tools use Markdown"
- "Competition drives innovation"
- "Eventually there will be consolidation"

**Pragmatic**:
- "Pick one tool and stick with it"
- "Build abstraction layers yourself"
- "Use MCP where possible, accept limitations elsewhere"
- "Focus on linting/formatting as the real source of truth"

---

## 6. Gaps and Opportunities

### 6.1 Critical Gaps Identified

1. **No Universal Rules Standard**
   - Status: Each vendor has own format
   - Impact: Manual maintenance of multiple files
   - Opportunity: Industry working group for rules schema

2. **Memory Portability Non-Existent**
   - Status: Completely proprietary
   - Impact: Lost context when switching tools
   - Opportunity: MCP-based memory protocol extension

3. **Skills Not Portable**
   - Status: Tool-specific formats
   - Impact: Duplicated skill development
   - Opportunity: Universal skill definition schema

4. **Uneven MCP Adoption**
   - Status: Only Claude Code has full support
   - Impact: MCP promise unfulfilled
   - Opportunity: Vendor adoption incentives, community pressure

5. **No Behavioral Consistency Tooling**
   - Status: Manual audits only
   - Impact: Drift goes undetected
   - Opportunity: Cross-tool testing frameworks

### 6.2 Observed Responses to Gaps

**Team-Level Patterns**:
- Some teams have created shared directories for common rules with generation scripts for tool-specific configs
- MCP server usage where supported
- CI checks for configuration consistency
- Documentation of tool-specific deviations

**Vendor Activity**:
- Varying levels of engagement with MCP adoption
- Some vendors have documented configuration schemas formally
- Participation in standardization efforts varies by vendor

**Community Activity**:
- Open-source converter tools in development
- Universal schema proposals circulating
- MCP bridges being built for non-supporting tools
- Configuration patterns and templates being shared across forums

---

## 7. Conclusion

Cross-platform interoperability for agentic coding tools in 2026 remains a significant challenge but shows promising direction:

**What Works Today**:
- **AGENTS.md** provides a universal rules standard with broad vendor support (Google, OpenAI, Cursor, GitHub)
- **MCP** provides a viable interoperability protocol with growing adoption (Claude, Cursor, Windsurf, Zed, Amazon Q)
- Markdown-based rules allow content sharing even if formats differ
- Most major tools now support MCP natively (not just Claude ecosystem)
- Linting/formatting tools serve as authoritative style enforcement

**What's Missing**:
- Portable memory/context format (still proprietary across tools)
- Cross-platform skill definitions (MCP tools help but skills remain tool-specific)
- Automated drift detection tooling
- Unified memory sharing standard

**Outlook**: The landscape has improved significantly since early 2025. AGENTS.md and MCP represent two complementary standards gaining real traction:
- **AGENTS.md**: 20,000+ repos, Linux Foundation governance, major vendor backing
- **MCP**: OpenAI + Google DeepMind adoption, 100+ servers, mature SDKs

2026-2027 outlook is positive for convergence, particularly around these two standards.

---

## Appendix A: Tool Configuration Quick Reference

### Claude Code
```
./CLAUDE.md                    # Project instructions
./.claude/
├── settings.json             # Configuration
├── settings.local.json       # Personal config (gitignored)
└── rules/
    └── *.md                  # Modular rules
~/.claude/CLAUDE.md           # User-wide instructions
```

### Cursor
```
./.cursorrules                # Project rules
./.cursor/
├── rules                     # Alternative rules location
└── settings.json             # Cursor settings
~/.cursor/rules               # Global rules
```

### GitHub Copilot
```
./.github/
└── copilot-instructions.md   # Repository instructions
./AGENTS.md                   # Universal format (supported)
# + Personal instructions in GitHub account settings
```

### Gemini Code Assist
```
./.gemini/
├── config.json              # Repository configuration
├── styleguide.md            # Code review style guide
└── .env                     # Environment variables
./GEMINI.md                  # Context file (hierarchical)
~/.gemini/                   # User-level config
```

### Amazon Q Developer
```
./.amazonq/
└── default.json             # Project-level MCP & tools config
~/.aws/amazonq/
└── default.json             # Global MCP & tools config
```

### Windsurf (Codeium)
```
./.windsurf/
└── rules/                   # Workspace rules
./.codeiumignore             # Files to ignore
~/.codeium/
└── .codeiumignore           # Global ignore rules
# + Rulebooks (invokable via slash commands)
```

### Universal (AGENTS.md)
```
./AGENTS.md                  # Repository root (primary)
./subdirectory/AGENTS.md     # Nested for monorepos
~/AGENTS.md                  # User-level defaults
```

### MCP Configuration
```json
// In Claude Code settings.json
{
  "mcpServers": {
    "server-name": {
      "command": "npx",
      "args": ["-y", "@org/mcp-server"],
      "env": { "KEY": "value" }
    }
  }
}
```

---

## Appendix B: Research Methodology

This research was conducted using:
1. Official documentation from tool vendors
2. MCP specification and documentation
3. Public GitHub repositories and discussions
4. Synthesized developer community sentiment
5. Analysis of configuration file patterns

**Limitations**:
- Rapidly evolving space may have newer developments
- Some proprietary tool internals not publicly documented
- Community sentiment synthesized from public sources

**Sources Consulted**:
- [AGENTS.md Official Site](https://agents.md/)
- [MCP Specification](https://modelcontextprotocol.io/specification/2025-11-25)
- [Cursor MCP Docs](https://cursor.com/docs/context/mcp)
- [Windsurf Cascade Docs](https://docs.windsurf.com/windsurf/cascade/cascade)
- [Gemini Code Assist Docs](https://developers.google.com/gemini-code-assist/docs/customize-gemini-behavior-github)
- [Amazon Q Developer Docs](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/mcp-ide.html)
- [GitHub Copilot Extensions](https://github.com/features/copilot/extensions)

---

*Document created: 2026-01-23*
*Last updated: 2026-01-23*
*Status: Complete*
*Branch: research-docs*
