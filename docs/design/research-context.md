# Skill and Context Management in 2026: A Design Study

## 1. Introduction

As of 2026, the landscape of AI coding tools has bifurcated into **CLI-based Agents** (Claude Code, Aider) and **Agentic IDEs** (Cursor, Windsurf, Zed). While they share common goals—autonomous coding, context awareness, and tool use—their approaches to configuration and capability management differ significantly.

This document analyzes these approaches to inform the design of a unified `repository-manager` that can seamlessly orchestrate context and skills across these diverse environments.

## 2. The Landscape of Agentic Tools

### 2.1 CLI Agents

Tools like **Claude Code** and **Aider** operate in the terminal. They rely heavily on explicit configuration files found in the working directory to establish context and behavior.

* **Context**: Explicitly loaded via commands (`/add src/main.rs`) or implicitly via file system scanning.
* **Skills**: Defined as Markdown files (SOPs) or MCP servers (executable tools).
* **Philosophy**: "The filesystem is the source of truth."

### 2.2 Agentic IDEs

Tools like **Cursor** and **Windsurf** integrate the agent directly into the editor flow. They leverage the editor's state (open tabs, cursor position) and indexed codebase embeddings (RAG) for context.

* **Context**: Dynamic and implicit (Open Tabs, `@codebase`, `@file`).
* **Rules**: Project-specific Markdown files (`.cursorrules`, `.windsurfrules`) that act as "system prompts" for the agent.
* **Philosophy**: "The editor state is the source of truth."

### 2.3 Antigravity (The "Superpowers" Framework)

**Antigravity** represents a structural framework or "overlay" that organizes agent capabilities into a standardized `.agent` directory. This approach seeks to make skills and workflows portable and structured, independent of the underlying LLM driver.

* **Structure**: `.agent/skills/`, `.agent/workflows/`, `.agent/rules/`.
* **Philosophy**: "Structured capabilities are the source of truth."

## 3. Core Concepts and Implementation Strategies

### 3.1 Managing "Rules" (Behavior & Style)

**Standard**: Markdown is the universal standard for defining behavioral rules.

| Tool | Location | Format | Feature |
| :--- | :--- | :--- | :--- |
| **Claude Code** | `CLAUDE.md` | Markdown | Hierarchical merging (folder > root > global). |
| **Cursor** | `.cursorrules` | Markdown | Single project-level file. |
| **Windsurf** | `.windsurfrules` | Markdown | Includes "Cascade" behavioral instructions. |
| **Antigravity** | `.agent/rules/*.md` | Markdown | Modular rules (e.g., `superpowers.md`). |

**Design Implication**: The orchestrator should maintain a central set of Markdown rules (e.g., in `.repository/rules/`) and *compile* them into the specific formats required by each tool (`CLAUDE.md`, `.cursorrules`).

### 3.2 Managing "Skills" (Capabilities & SOPs)

A "Skill" in 2026 is often a hybrid of a **Standard Operating Procedure (SOP)** (text instructions) and **Executable Tools** (scripts/MCP).

* **Claude Code**: Skills are Markdown files in `.claude/skills/`. They act as complex prompts that can invoke CLI commands.
* **Antigravity**: Skills are **directories** in `.agent/skills/` (e.g., `.agent/skills/superpowers-plan/`).
  * **`SKILL.md`**: Contains the definition, overview, and instructions.
  * **Auxiliary Files**: Scripts, templates, or resources needed by the skill.
* **MCP (Model Context Protocol)**: The emerging standard for *executable* tools (database access, git operations).

**Design Implication**:

* **Soft Skills (SOPs)**: Use the Antigravity structure (Markdown defined steps). These map to Claude's skills.
* **Hard Skills (Tools)**: Use MCP servers. The orchestrator should manage the `mcpServers` configuration in `settings.json` for all tools.

### 3.3 Managing "Workflows" (Orchestration)

Workflows are higher-order sequences that string together multiple skills or steps.

* **Antigravity**: Explicit `.md` files in `.agent/workflows/`. They use YAML frontmatter for metadata and define steps (e.g., "1. Brainstorm", "2. Plan", "3. Execute").
* **Plugins**: Often essentially "packaged workflows" or "skills with code".

### 3.4 Managing "Context"

Context is the data the agent works with.

* **Explicit**: "Read file X", "Search for Y".
* **Implicit (RAG)**: "Understanding the codebase". IDEs excel here via embeddings.
* **Protocol (MCP)**: Connecting to external contexts (Postgres, Slack).

## 4. Design Proposal: The Unified `.repository` Schema

To satisfy the user's request for a tool that manages this complexity, we propose a `.repository` schema that abstracts these concepts.

### 4.1 Directory Structure

```text
.repository/
├── config.toml          # Meta-configuration (which providers are active?)
├── rules/               # Modular Markdown rules
│   ├── style.md         # "Use strict types..."
│   └── architecture.md  # "MVVM pattern..."
├── skills/              # Capabilities
│   └── deploy-app/
│       ├── SKILL.md
│       └── script.sh
├── workflows/           # Orchestrated sequences
│   └── release-cycle.md
└── mcp/                 # MCP Server configurations
    └── postgres.json
```

### 4.2 The "Compiler" (Orchestrator)

The CLI tool will read this structure and **unroll** it to the target environments:

1. **Sync to Claude**:
    * Concatenate `rules/*.md` -> `CLAUDE.md`
    * Symlink/Copy `skills/*/SKILL.md` -> `.claude/skills/`
    * Generate `.claude/settings.json` from `mcp/` configs.

2. **Sync to Cursor**:
    * Concatenate `rules/*.md` -> `.cursorrules`
    * (Cursor currently lacks a native "skill" folder concept equivalent to Claude's, so skills might be appended to rules or managed via external scripts).

3. **Sync to Antigravity**:
    * Antigravity *is* the source structure (`.agent`), so this is a passthrough or verification step.

## 5. Summary of Overlaps

| Feature | Agents (Claude) | IDEs (Cursor) | Framework (Antigravity) |
| :--- | :--- | :--- | :--- |
| **Config Format** | Markdown + JSON | Markdown + JSON | Markdown + YAML + Scripts |
| **Tool Protocol** | MCP (Native) | MCP (Native/Extension) | Wraps MCP + Scripts |
| **SOP Definition** | `skills/*.md` | via Prompt/Rules | `skills/*/SKILL.md` |
| **Orchestration** | Manual / Chain | "Composer" / "Flows" | `workflows/*.md` |

**Conclusion**: The industry is converging on **Markdown for human instruction** and **MCP for machine interaction**. A tool that manages these two primitives and projects them into tool-specific locations will provide the "Superpowers" the user seeks.
