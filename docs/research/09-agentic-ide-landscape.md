# Frontier Agentic IDE Landscape (2026)

## Executive Summary

The agentic IDE landscape in 2026 represents a significant evolution from traditional AI-assisted coding tools toward autonomous, multi-step reasoning systems. This document analyzes the major players, their architectures, capabilities, and differentiators to inform orchestrator tool design and cross-platform interoperability strategies.

**Key Findings:**

- The market is bifurcated between VS Code forks (Cursor, Windsurf, Zed) and native implementations (Claude Code CLI)
- MCP (Model Context Protocol) is emerging as the de facto standard for tool integration
- "Agentic" capabilities (autonomous multi-file editing, terminal access, persistent memory) are now table stakes
- Configuration portability remains a significant challenge with no universal standard
- Antigravity represents a potential new entrant focused on "zero-latency" AI coding

**Research Note:** This document synthesizes information from training data through May 2025 with projected trends. Items marked [2026-VERIFY] should be validated against current documentation.

---

## 1. Major Agentic IDEs Analysis

### 1.1 Windsurf (Codeium)

#### Windsurf Company/Origin

- **Company:** Codeium (formerly Exafunction)
- **Founded:** 2021
- **Headquarters:** Mountain View, California
- **Focus:** AI-powered code acceleration
- **Background:** Codeium initially focused on autocomplete; Windsurf represents their agentic IDE evolution

#### Windsurf Architecture

- **Base:** VS Code fork (Electron-based)
- **Core Technology:** Proprietary "Cascade" AI engine
- **Performance:** Claims 60% less latency than competitors via proprietary optimizations
- **Deployment:** Desktop application (Windows, macOS, Linux)

#### Windsurf AI Model(s) Used

- **Primary:** Proprietary fine-tuned models on code
- **Secondary:** Integration with Claude, GPT-4 [2026-VERIFY]
- **Unique:** "Cascade" is not a single model but an orchestration system that combines:
  - Fast autocomplete model (low latency)
  - Reasoning model (multi-step planning)
  - Code generation model (implementation)

#### Windsurf Agentic Capabilities

| Capability         | Support | Notes                        |
| :----------------- | :------ | :--------------------------- |
| Multi-file editing | Yes     | Cascade-coordinated          |
| Terminal access    | Yes     | Integrated terminal control  |
| Autonomous coding  | Yes     | "Flows" for multi-step tasks |
| File creation      | Yes     | Can create new files         |
| Git operations     | Yes     | Built-in git UI + AI commits |
| Web browsing       | Partial | Research mode [2026-VERIFY]  |

#### Windsurf Configuration System

```text
.windsurfrules                 # Project-level rules (Markdown)
.windsurf/
├── rules/                     # Modular rule files
├── settings.json              # IDE settings
└── cascade.json               # Cascade behavior config
```

**Configuration Discovery:**

1. IDE startup loads workspace configuration
2. `.windsurfrules` parsed at project open
3. Rules applied to all Cascade interactions
4. Hierarchical: workspace > user settings

**Example .windsurfrules:**

```markdown
# Project Rules

## Language
This is a TypeScript monorepo using pnpm workspaces.

## Style
- Use functional components with hooks
- Prefer Zod for validation
- No `any` types

## Commands
- Build: `pnpm build`
- Test: `pnpm test`
- Lint: `pnpm lint`

## Architecture
- Service layer in `/services`
- API routes in `/api`
- Shared types in `/types`
```

#### Windsurf Memory/Context Persistence

- **Cascade Memory:** Automatic memory formation from conversations
- **Scope:** Project-level persistence
- **Format:** Proprietary (not directly exportable)
- **Features:**
  - Learns coding patterns
  - Remembers architectural decisions
  - Tracks conversation context across sessions

#### Windsurf MCP Support

- **Status:** Native support [2026-VERIFY]
- **Configuration:** Settings JSON or UI
- **Capabilities:** Tools, Resources, Prompts

#### Windsurf Pricing Model [2026-VERIFY]

| Tier       | Price          | Features                            |
| :--------- | :------------- | :---------------------------------- |
| Free       | $0             | Limited completions, basic Cascade  |
| Pro        | $15/month      | Unlimited completions, full Cascade |
| Teams      | $19/user/month | Admin controls, shared settings     |
| Enterprise | Custom         | SSO, audit logs, SLA                |

#### Windsurf Unique Differentiators

1. **Cascade Engine:** Multi-model orchestration for complex tasks
2. **Flows:** Visual workflow representation for agentic tasks
3. **Speed:** Optimized for low-latency interactions
4. **Memory:** Automatic context retention
5. **Supercomplete:** Context-aware, multi-line completions

---

### 1.2 Antigravity

#### Antigravity Company/Origin

- **Company:** Unknown/Stealth [2026-VERIFY]
- **Status:** Emerged as a new entrant in the agentic IDE space
- **Positioning:** "Zero-latency" AI coding experience

#### What is Antigravity?

Based on available information, Antigravity appears to be a next-generation agentic IDE with several distinguishing characteristics:

1. **Speculative Execution:** Pre-computes likely code paths before user requests
2. **Local-First Architecture:** Emphasizes on-device processing to minimize latency
3. **Novel UI Paradigm:** May depart from traditional IDE layouts

**[2026-VERIFY]:** Antigravity is a newer entrant and detailed technical specifications may have emerged since May 2025. This section requires validation against current sources.

#### Antigravity Architecture (Speculative)

- **Base:** Unknown (possibly custom, not VS Code fork)
- **Focus:** Latency optimization through:
  - Predictive model execution
  - Edge computing integration
  - Optimized model architectures

#### Antigravity AI Model(s) Used

- Details not publicly available at time of research
- Likely custom models optimized for:
  - Inference speed
  - Code-specific tasks
  - Streaming responses

#### Antigravity Reported Features [2026-VERIFY]

| Capability              | Support | Notes              |
| :---------------------- | :------ | :----------------- |
| Multi-file editing      | Likely  | Industry standard  |
| Terminal access         | Unknown |                    |
| Autonomous coding       | Likely  | Key differentiator |
| Predictive editing      | Yes     | Core value prop    |
| Real-time collaboration | Unknown |                    |

#### Antigravity Configuration System

- Details not available
- Likely follows industry patterns (rules file + settings)

#### Antigravity Memory/Context Persistence

- Unknown implementation
- Likely includes some form of project memory

#### Antigravity MCP Support

- Unknown

#### Antigravity Pricing Model

- Not publicly announced

#### Antigravity Unique Differentiators (Claimed)

1. **Zero-Latency:** Emphasis on eliminating perceived AI response time
2. **Predictive Architecture:** Anticipates developer intent
3. **Novel Interaction Model:** May include new UI/UX paradigms

#### How it Differs from Windsurf/Cursor

| Aspect       | Windsurf      | Cursor       | Antigravity       |
| :----------- | :------------ | :----------- | :---------------- |
| Focus        | Flows/Cascade | Composer     | Latency           |
| Architecture | VS Code fork  | VS Code fork | Unknown (custom?) |
| Maturity     | Established   | Established  | Early             |
| Memory       | Automatic     | Limited      | Unknown           |

**Research Gap:** Antigravity requires additional research from 2025-2026 sources to provide comprehensive analysis.

---

### 1.3 Cursor IDE

#### Cursor Company/Origin

- **Company:** Anysphere Inc.
- **Founded:** 2022
- **Headquarters:** San Francisco
- **Funding:** $60M+ raised
- **Focus:** AI-first code editor

#### Cursor Architecture

- **Base:** VS Code fork (Electron-based)
- **Customizations:** Heavily modified for AI integration
- **Codebase Indexing:** Vector embeddings for semantic search
- **Deployment:** Desktop application (Windows, macOS, Linux)

#### Cursor AI Model(s) Used

- **Primary:** Claude 3.5 Sonnet (default for many operations)
- **Secondary:** GPT-4, GPT-4 Turbo
- **User Choice:** Model selection in settings
- **Cursor-small:** Proprietary model for fast operations [2026-VERIFY]

#### Cursor Agentic Capabilities

| Capability         | Support | Notes                |
| :----------------- | :------ | :------------------- |
| Multi-file editing | Yes     | Composer mode        |
| Terminal access    | Partial | Terminal integration |
| Autonomous coding  | Yes     | Agent mode           |
| File creation      | Yes     | Via Composer         |
| Git operations     | Partial | UI integration       |
| Web browsing       | Yes     | @web for search      |

#### Cursor Configuration System

```text
.cursorrules                   # Legacy: Project rules (Markdown/Text)
.cursor/
├── rules                      # Current: Project rules
├── settings.json              # Cursor-specific settings
└── prompts/                   # Custom prompt templates [2026-VERIFY]
~/.cursor/
└── rules                      # Global rules
```

**Configuration Discovery:**

1. Project load triggers config scan
2. `.cursorrules` or `.cursor/rules` loaded first
3. Global `~/.cursor/rules` applied as fallback
4. Settings merged (project > user)

**Example .cursorrules:**

```markdown
You are an expert TypeScript developer working on a Next.js 14 application.

## Rules
- Always use TypeScript strict mode
- Prefer server components unless interactivity needed
- Use Tailwind CSS for styling
- Use the app router exclusively

## Patterns
- Repository pattern for data access
- Zod for all runtime validation
- Error boundaries on all pages

## Avoid
- Never use `any` type
- Don't use inline styles
- Don't create pages/ directory files
```

#### Cursor Memory/Context Persistence

- **Codebase Index:** Vector embeddings of repository
- **Chat History:** Persisted per project
- **Memory:** Limited explicit memory (conversation-based)
- **RAG:** `@codebase` for semantic search across files

#### Cursor MCP Support

- **Status:** Native support
- **Configuration:** Via Cursor settings
- **Capabilities:** Tools, Resources

#### Cursor Pricing Model [2026-VERIFY]

| Tier     | Price          | Features                              |
| :------- | :------------- | :------------------------------------ |
| Free     | $0             | 2,000 completions/month, limited chat |
| Pro      | $20/month      | Unlimited, all models                 |
| Business | $40/user/month | Admin, SSO, audit                     |

#### Cursor Unique Differentiators

1. **Composer:** Multi-file, planned editing interface
2. **@-mentions:** Rich context system (`@file`, `@folder`, `@web`, `@codebase`)
3. **Inline Editing:** Cmd+K for quick inline changes
4. **Model Flexibility:** Easy model switching
5. **Tab Completion:** Predictive multi-line suggestions

---

### 1.4 Zed Editor

#### Zed Company/Origin

- **Company:** Zed Industries
- **Founded:** 2021
- **Team:** Former Atom core team members
- **Headquarters:** San Francisco
- **Focus:** High-performance collaborative editor

#### Zed Architecture

- **Base:** Custom (NOT VS Code fork)
- **Language:** Rust + GPUI (custom UI framework)
- **Performance:** Native performance, GPU-accelerated rendering
- **Deployment:** Desktop (macOS primary, Linux, Windows [2026-VERIFY])

#### Zed AI Model(s) Used

- **Integrated:** Claude (via Anthropic partnership)
- **User Choice:** Supports multiple providers via configuration
- **Local Models:** Support for Ollama and local LLMs [2026-VERIFY]

#### Zed Agentic Capabilities

| Capability         | Support | Notes               |
| :----------------- | :------ | :------------------ |
| Multi-file editing | Partial | Via assistant panel |
| Terminal access    | Yes     | Integrated terminal |
| Autonomous coding  | Limited | Assistant-based     |
| File creation      | Partial | Through assistant   |
| Git operations     | Yes     | Built-in git UI     |
| Real-time collab   | Yes     | Core feature        |

#### Zed Configuration System

```text
~/.config/zed/
├── settings.json              # Global settings
├── keymap.json                # Key bindings
└── themes/                    # Custom themes
.zed/
└── settings.json              # Project settings
```

**Example AI Configuration:**

```json
{
  "assistant": {
    "default_model": {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022"
    },
    "version": "2"
  },
  "features": {
    "inline_completion_provider": "copilot"
  }
}
```

#### Zed Memory/Context Persistence

- **Status:** Limited
- **Assistant Panel:** Conversation history
- **Project Context:** File-based via settings

#### Zed MCP Support

- **Status:** Native support [2026-VERIFY]
- **Configuration:** Via settings.json
- **Context:** Zed has been investing in MCP integration

#### Zed Pricing Model

| Tier    | Price     | Features                       |
| :------ | :-------- | :----------------------------- |
| Free    | $0        | Full editor, limited AI        |
| Zed Pro | $20/month | Full AI features [2026-VERIFY] |

#### Zed Unique Differentiators

1. **Performance:** Fastest editor in benchmarks (Rust + GPU)
2. **Collaboration:** Real-time multiplayer editing
3. **Native:** Not Electron - true native performance
4. **Modern Design:** Clean, minimal interface
5. **Open Source:** Community contributions

---

### 1.5 VS Code + AI Extensions

#### Overview

VS Code with AI extensions represents the "assemble your own" approach to agentic coding, offering flexibility but less integration.

#### Major AI Extensions

##### GitHub Copilot

- **Company:** GitHub (Microsoft)
- **Model:** OpenAI Codex, GPT-4
- **Features:**
  - Inline completions
  - Chat panel
  - Copilot Workspace (autonomous PR generation)
- **Configuration:** `.github/copilot-instructions.md`
- **MCP:** No native support
- **Pricing:** $10-39/month

##### Continue.dev

- **Company:** Continue (open source)
- **Model:** Any (Claude, GPT, local)
- **Features:**
  - Inline editing
  - Chat panel
  - Custom commands
  - Context providers
- **Configuration:** `~/.continue/config.json`, `.continuerc.json`
- **MCP:** Partial (via context providers)
- **Pricing:** Free (open source)

##### Cody (Sourcegraph)

- **Company:** Sourcegraph
- **Model:** Claude, StarCoder
- **Features:**
  - Code intelligence
  - Codebase-aware chat
  - Enterprise context
- **Configuration:** VS Code settings, `.cody/` [2026-VERIFY]
- **MCP:** Limited
- **Pricing:** Free tier + Enterprise

#### VS Code AI Configuration Comparison

#### VS Code AI Configuration Comparison

| Extension | Config File                       | Format   | Project-Level |
| :-------- | :-------------------------------- | :------- | :------------ |
| Copilot   | `copilot-instructions.md`         | Markdown | Yes           |
| Continue  | `config.json`, `.continuerc.json` | JSON     | Yes           |
| Cody      | VS Code settings                  | JSON     | Limited       |

#### Advantages

- Mix and match extensions
- Familiar VS Code environment
- Large extension ecosystem
- Open source (VS Code base)

#### Disadvantages

- Less integrated experience
- Extension conflicts possible
- No unified agentic workflow
- Multiple configuration systems

---

### 1.6 JetBrains IDEs + AI

#### JetBrains Company/Origin

- **Company:** JetBrains
- **Headquarters:** Prague, Czech Republic
- **Focus:** Language-specific IDEs (IntelliJ, PyCharm, WebStorm, etc.)

#### JetBrains Architecture

- **Base:** JetBrains Platform (Java-based)
- **AI Integration:** JetBrains AI Assistant plugin
- **Deployment:** Desktop (Windows, macOS, Linux)

#### JetBrains AI Model(s) Used

- **Primary:** JetBrains AI (proprietary) [2026-VERIFY]
- **Integration:** Claude, GPT-4 partnerships
- **Local:** Support for local models in some configurations

#### JetBrains AI Assistant Features

| Capability         | Support | Notes              |
| :----------------- | :------ | :----------------- |
| Code completion    | Yes     | AI-enhanced        |
| Chat               | Yes     | AI Assistant panel |
| Multi-file editing | Limited | Refactoring tools  |
| Terminal access    | Partial | Through IDE        |
| Documentation      | Yes     | AI-generated docs  |
| Test generation    | Yes     | AI-suggested tests |

#### JetBrains Configuration System

- **Location:** IDE settings (GUI-based)
- **Project Settings:** `.idea/` directory
- **AI Settings:** Plugin configuration

```text
.idea/
├── ai-assistant.xml           # AI configuration [2026-VERIFY]
└── workspace.xml              # Workspace settings
```

#### JetBrains MCP Support

- **Status:** Partial/Experimental [2026-VERIFY]
- **Focus:** Native JetBrains integrations

#### JetBrains Pricing Model

| Tier         | Price              | AI Included     |
| :----------- | :----------------- | :-------------- |
| Individual   | $149-249/year      | Optional add-on |
| Organization | $499-599/year/user | Optional add-on |
| AI Assistant | +$10-20/month      | AI features     |

#### JetBrains Unique Differentiators

1. **Language Intelligence:** Deep language-specific features
2. **Refactoring:** Best-in-class refactoring tools
3. **Integration:** Database, frameworks, deployment
4. **Quality:** Static analysis, inspections
5. **Ecosystem:** Mature plugin ecosystem

#### Limitations for Agentic Workflows

- GUI-first approach limits automation
- Less focus on autonomous multi-file editing
- Configuration not as portable
- Heavier resource usage

---

### 1.7 Emerging/Other Agentic IDEs

#### Void Editor

- **Status:** Early development [2026-VERIFY]
- **Focus:** Open-source Cursor alternative
- **Architecture:** VS Code fork
- **Differentiator:** Fully open source agentic IDE

#### Bolt.new / StackBlitz

- **Type:** Browser-based agentic development
- **Focus:** Full-stack app generation
- **Model:** Claude integration
- **Differentiator:** In-browser development environment

#### Replit Agent

- **Type:** Browser-based IDE with AI
- **Focus:** Collaborative coding + deployment
- **Model:** Multiple providers
- **Differentiator:** Integrated hosting/deployment

#### Aider (CLI)

- **Type:** Terminal-based coding agent
- **Focus:** Git-integrated AI coding
- **Model:** Any (Claude, GPT, local)
- **Differentiator:** CLI workflow, git-first design

#### Claude Code (CLI)

- **Type:** Terminal-based agentic coding
- **Focus:** Anthropic's official CLI agent
- **Model:** Claude
- **Differentiator:** Deep MCP integration, skills system

---

## 2. Comparison Matrix

### 2.1 Architecture Comparison

| IDE         | Base               | Language            | Performance | Open Source |
| :---------- | :----------------- | :------------------ | :---------- | :---------- |
| Windsurf    | VS Code fork       | TypeScript/Electron | Good        | No          |
| Cursor      | VS Code fork       | TypeScript/Electron | Good        | No          |
| Zed         | Custom             | Rust/GPUI           | Excellent   | Yes         |
| VS Code     | Original           | TypeScript/Electron | Good        | Yes         |
| JetBrains   | JetBrains Platform | Java/Kotlin         | Moderate    | No          |
| Antigravity | Unknown            | Unknown             | Unknown     | Unknown     |
| Claude Code | CLI                | Various             | Excellent   | No          |

### 2.2 AI Model Support

| IDE                | Claude | GPT-4   | Custom/Local | Model Choice |
| :----------------- | :----- | :------ | :----------- | :----------- |
| Windsurf           | Yes    | Yes     | Yes          | Yes          |
| Cursor             | Yes    | Yes     | No           | Yes          |
| Zed                | Yes    | Partial | Yes (Ollama) | Yes          |
| VS Code (Copilot)  | No     | Yes     | No           | Limited      |
| VS Code (Continue) | Yes    | Yes     | Yes          | Yes          |
| JetBrains          | Yes    | Yes     | Limited      | Limited      |
| Claude Code        | Yes    | No      | No           | No           |

### 2.3 Agentic Capabilities

| IDE                  | Multi-File | Terminal | Autonomous      | Memory    | MCP     |
| :------------------- | :--------- | :------- | :-------------- | :-------- | :------ |
| Windsurf             | Full       | Full     | Full (Cascade)  | Automatic | Yes     |
| Cursor               | Full       | Partial  | Full (Composer) | Limited   | Yes     |
| Zed                  | Partial    | Full     | Partial         | Limited   | Yes     |
| VS Code + Extensions | Varies     | Varies   | Varies          | Varies    | Varies  |
| JetBrains            | Limited    | Partial  | Limited         | No        | Partial |
| Claude Code          | Full       | Full     | Full            | Explicit  | Full    |

### 2.4 Configuration Systems

| IDE         | Config File               | Format   | Hierarchical | Portable |
| :---------- | :------------------------ | :------- | :----------- | :------- |
| Windsurf    | `.windsurfrules`          | Markdown | Limited      | Medium   |
| Cursor      | `.cursorrules`            | Markdown | Limited      | Medium   |
| Zed         | `.zed/settings.json`      | JSON     | No           | Low      |
| Copilot     | `copilot-instructions.md` | Markdown | Partial      | High     |
| Continue    | `.continuerc.json`        | JSON     | Yes          | Medium   |
| JetBrains   | `.idea/*.xml`             | XML      | No           | Low      |
| Claude Code | `CLAUDE.md`               | Markdown | Yes          | High     |

### 2.5 Enterprise Readiness

| IDE         | SSO | Audit Logs | Admin Console | Compliance         | Pricing         |
| :---------- | :-- | :--------- | :------------ | :----------------- | :-------------- |
| Windsurf    | Yes | Yes        | Yes           | SOC2 [2026-VERIFY] | Enterprise tier |
| Cursor      | Yes | Yes        | Yes           | SOC2               | Business tier   |
| Zed         | Ltd | No         | No            | Limited            | Pro tier        |
| Copilot     | Yes | Yes        | Yes           | SOC2               | Enterprise      |
| JetBrains   | Yes | Yes        | Yes           | Various            | Enterprise      |
| Claude Code | No  | No         | No            | Via API            | API pricing     |

---

## 3. What Makes an IDE "Agentic"?

### 3.1 Agentic vs AI-Assisted Spectrum

```text
AI-Assisted                                             Agentic
    |                                                       |
    v                                                       v
[Autocomplete] -> [Chat] -> [Edit] -> [Multi-file] -> [Autonomous]
    |              |          |            |                |
   Basic       Context    Directed     Planned         Self-directed
   completion   Q&A      changes     changes          execution
```

### 3.2 Characteristics of Agentic IDEs

| Characteristic | AI-Assisted      | Agentic              |
| :------------- | :--------------- | :------------------- |
| **User Role**  | Driver           | Supervisor           |
| **Scope**      | Single file/line | Project-wide         |
| **Planning**   | None             | Multi-step reasoning |
| **Execution**  | User applies     | AI executes          |
| **Memory**     | Session only     | Persistent           |
| **Tool Use**   | Limited          | Extensive            |
| **Autonomy**   | None             | Configurable         |

### 3.3 Required Capabilities for "Agentic" Status

1. **Multi-step Reasoning:** Can plan and execute complex tasks
2. **Multi-file Editing:** Can modify multiple files in coordinated ways
3. **Tool Invocation:** Can run terminal commands, tests, etc.
4. **Memory/Context:** Maintains context across interactions
5. **Self-correction:** Can detect and fix errors in its work
6. **Autonomous Execution:** Can work with minimal user intervention

---

## 4. Configuration Discovery and Loading

### 4.1 How IDEs Discover Agentic Configuration

#### Windsurf

```text
1. IDE starts
2. Open workspace/folder
3. Scan for .windsurfrules in root
4. Scan .windsurf/ directory
5. Load user settings (~/.windsurf/)
6. Merge: project > user > defaults
7. Initialize Cascade with merged config
```

#### Cursor

```text
1. IDE starts
2. Open workspace/folder
3. Scan for .cursorrules (legacy) or .cursor/rules
4. Load user rules (~/.cursor/rules)
5. Merge: project > user > defaults
6. Index codebase for RAG
7. Initialize AI with rules context
```

#### Claude Code

```text
1. CLI starts in directory
2. Walk up directory tree
3. Collect all CLAUDE.md files
4. Load ~/.claude/CLAUDE.md
5. Merge: child > parent > user > system
6. Load MCP servers from settings
7. Initialize tools and skills
```

### 4.2 Configuration Priority Patterns

**Most Specific Wins:**

```text
Project-specific > Directory-specific > User > System/Default
```

**Merge vs Override:**

| IDE         | Strategy | Conflict Resolution |
| :---------- | :------- | :------------------ |
| Windsurf    | Merge    | Later wins          |
| Cursor      | Override | Project wins        |
| Claude Code | Merge    | Child wins          |
| Copilot     | Merge    | Project wins        |

### 4.3 Integration Points for Orchestrator Tool

An orchestrator tool managing agentic configuration across IDEs should:

1. **Generate Tool-Specific Configs:**

   ```text
   .agentic/rules/common.md
        |
        v
   [Orchestrator]
        |
   +----+----+----+
   v    v    v    v
   CLAUDE.md  .cursorrules  .windsurfrules  copilot-instructions.md
   ```

2. **Watch for Changes:**
   - Monitor `.agentic/` for updates
   - Regenerate tool configs on change
   - Optionally: git hooks for sync

3. **Provide Conversion Utilities:**
   - Import: Tool-specific -> Common format
   - Export: Common format -> Tool-specific

4. **Handle MCP Configuration:**
   - Centralized MCP server registry
   - Generate tool-specific MCP configs

---

## 5. Market Positioning and Target Users

### 5.1 Market Map

```text
                    Enterprise-Focused
                          |
                    JetBrains
                    Copilot Enterprise
                          |
    Individual <----------+----------> Team
        |                 |              |
      Aider         Cursor/Windsurf    Zed
      Claude Code                       |
        |                              |
        v                              v
    Developer-Focused          Collaboration-Focused
```

### 5.2 Target User Profiles

| IDE               | Primary User            | Use Case                | Team Size         |
| :---------------- | :---------------------- | :---------------------- | :---------------- |
| Windsurf          | Full-stack developer    | Rapid prototyping       | Individual-Small  |
| Cursor            | TypeScript/JS developer | Feature development     | Individual-Medium |
| Zed               | Performance-focused dev | Real-time collaboration | Small-Medium      |
| VS Code + Copilot | General developer       | Wide applicability      | Any               |
| JetBrains         | Enterprise Java/.NET    | Large codebases         | Medium-Large      |
| Claude Code       | Terminal-native dev     | CLI automation          | Individual        |

### 5.3 Positioning Statements

**Windsurf:** "The AI IDE that thinks ahead" - Cascade flows, automatic memory
**Cursor:** "The AI-first code editor" - Composer, @-mentions, seamless AI
**Zed:** "Code at the speed of thought" - Performance, collaboration
**Copilot:** "Your AI pair programmer" - Ubiquitous, integrated
**JetBrains:** "The IDE for professionals" - Deep language support, enterprise

---

## 6. Configuration Portability Analysis

### 6.1 What Can Be Ported

| Configuration Type | Portability | Notes                     |
| :----------------- | :---------- | :------------------------ |
| Code style rules   | High        | Markdown works everywhere |
| Architecture docs  | High        | Universal format          |
| Command references | High        | Shell commands universal  |
| File patterns      | High        | Glob syntax standard      |
| Model preferences  | Medium      | Different defaults        |
| MCP servers        | Medium      | For MCP-supporting tools  |
| Memory/context     | Low         | Proprietary formats       |
| Skills/workflows   | Low         | Tool-specific             |

### 6.2 Portable Configuration Template

```markdown
# Universal Agentic Configuration

## Project Overview
[Description works in all tools]

## Tech Stack
- Language: TypeScript
- Framework: Next.js 14
- Database: PostgreSQL
- Testing: Jest + Playwright

## Code Style
- Use strict TypeScript
- Functional components with hooks
- Tailwind for styling
- Zod for validation

## Architecture
- App router (src/app/)
- Server components by default
- API routes in src/app/api/
- Shared types in src/types/

## Commands
- Build: `pnpm build`
- Test: `pnpm test`
- Lint: `pnpm lint`
- Dev: `pnpm dev`

## Patterns to Follow
- Repository pattern for data
- Service layer for business logic
- Error boundaries on all pages

## Patterns to Avoid
- No `any` types
- No inline styles
- No pages/ directory (app router only)
```

### 6.3 Tool-Specific Additions

Each tool may need additions:

**Claude Code additions:**

```markdown
## Claude-Specific

### Skills
Reference: .claude/skills/deploy.md for deployment

### MCP Servers
- context7: Documentation lookup
- postgres: Database queries
```

**Cursor additions:**

```markdown
## Cursor-Specific

### Context Commands
- @codebase for semantic search
- @web for documentation lookup
```

---

## 7. Future Directions for Interoperability

The evolution of agentic IDEs suggests a growing need for standardized configuration and context management. Future research should focus on how these tools might converge on shared protocols like MCP and how cross-platform portability of rules and memory could be achieved through community-driven standards.

---

## 8. Key Insights and Conclusions

### 8.1 Market Trends

1. **VS Code Dominance:** Most agentic IDEs build on VS Code
2. **Convergence on Markdown:** Rules files standardizing on Markdown
3. **MCP Momentum:** Growing adoption of MCP for tool integration
4. **Memory Competition:** Automatic memory becoming a differentiator
5. **Performance Race:** Latency optimization is a key battleground

### 8.2 Configuration Landscape

1. **No Universal Standard:** Each tool has unique configuration
2. **Markdown Common Ground:** Text-based rules work across tools
3. **MCP as Bridge:** Best hope for tool integration standardization
4. **Memory Not Portable:** Biggest gap in interoperability
5. **Skills Tool-Specific:** No portable skill/workflow format

### 8.3 Recommendations

1. **Bet on MCP:** Best available standardization effort
2. **Use Markdown Rules:** Maximum portability today
3. **Build Abstraction Layer:** Generate tool configs from common source
4. **Accept Some Lock-in:** Tool-specific features will exist
5. **Track Antigravity:** May introduce new paradigms
6. **Monitor Zed:** Non-VS-Code approach gaining traction

### 8.4 Research Gaps

Items requiring current research to validate:

- [ ] Antigravity full feature set and architecture
- [ ] Current pricing for all tools
- [ ] Latest MCP support status
- [ ] JetBrains AI Assistant current capabilities
- [ ] Zed Windows support status
- [ ] Windsurf memory export capabilities
- [ ] Void Editor development status

---

## Appendix A: Configuration File Quick Reference

| IDE         | Primary Config            | Location        | Format   |
| :---------- | :------------------------ | :-------------- | :------- |
| Windsurf    | `.windsurfrules`          | Root            | Markdown |
| Cursor      | `.cursorrules`            | Root            | Markdown |
| Zed         | `settings.json`           | `.zed/`         | JSON     |
| Copilot     | `copilot-instructions.md` | `.github/`      | Markdown |
| Continue    | `config.json`             | `~/.continue/`  | JSON     |
| JetBrains   | Various                   | `.idea/`        | XML      |
| Claude Code | `CLAUDE.md`               | Root, hierarchy | Markdown |
| Aider       | `.aider.conf.yml`        | Root            | YAML     |

## Appendix B: MCP Configuration Examples

### Claude Code

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "."]
    }
  }
}
```

### Cursor

```json
{
  "mcpServers": {
    "context7": {
      "command": "npx",
      "args": ["-y", "@context7/mcp-server"]
    }
  }
}
```

### Windsurf

Similar to Cursor; configured via settings UI or JSON.

## Appendix C: Feature Checklist for Agentic IDE Evaluation

```markdown
## Evaluation Checklist

### Core Agentic Features
- [ ] Multi-file editing
- [ ] Terminal command execution
- [ ] File creation/deletion
- [ ] Git operations
- [ ] Test execution
- [ ] Build/deploy commands

### Intelligence
- [ ] Multi-step planning
- [ ] Error detection and correction
- [ ] Context awareness
- [ ] Memory persistence
- [ ] RAG/semantic search

### Configuration
- [ ] Project-level rules
- [ ] User-level rules
- [ ] Hierarchical inheritance
- [ ] MCP support
- [ ] Skill/workflow definitions

### Enterprise
- [ ] SSO/SAML
- [ ] Audit logging
- [ ] Admin console
- [ ] Content policies
- [ ] Compliance certifications

### Developer Experience
- [ ] Low latency
- [ ] Inline completions
- [ ] Chat interface
- [ ] Visual diffs
- [ ] Undo/rollback
```

---

*Research compiled: January 2026*
*Last updated: 2026-01-23*
*Branch: research-docs*
*Status: Comprehensive analysis - [2026-VERIFY] items need current validation*
