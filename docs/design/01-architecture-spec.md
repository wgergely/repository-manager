# Agentic Repository Manager - Architecture Specification

*Version 1.1*
*Date: 2026-01-23*
*Status: Synthesized from Research*

## 1. Executive Summary

The **Agentic Repository Manager** (`agentic`) is a CLI tool designed to orchestrate configuration, rules, and context across multiple AI coding assistants (Claude Code, Cursor, Windsurf, etc.). It addresses the fragmentation in the agentic IDE landscape by providing a "Single Source of Truth" for development standards and automating the synchronization of these standards to tool-specific formats.

## 2. Core Concepts: The "Superpowers" Trinity

The architecture is built around three core entities that define an agentic environment:

1. **Rules**: Passive instructions and constraints (e.g., coding standards, architecture capabilities).
    * *Source*: Markdown files in `.agentic/rules/`.
    * *Target*: `CLAUDE.md`, `.cursorrules`, `.windsurfrules`.
2. **Skills**: Active capabilities and tools (e.g., `git commit`, `deploy`, `db-query`).
    * *Source*: Markdown/YAML definitions in `.agentic/skills/`.
    * *Target*: Tool-specific tool definitions or MCP servers.
3. **Workflows**: Process guides for complex tasks (e.g., "Onboarding", "Release").
    * *Source*: Markdown files in `.agentic/workflows/`.
    * *Target*: Context files or actionable checklists.

## 3. System Architecture

The system follows a modular architecture with a core orchestrator and pluggable providers.

### 3.1 High-Level Component Diagram

```text
[User] -> [CLI Interface] -> [Core Orchestrator]
                                    |
          +-------------------------+-------------------------+
          |                         |                         |
[Configuration Engine]      [Provider Adapters]       [Git Manager]
          |                         |                         |
    (Parses TOML/MD)        (Translates Config)     (Handle Worktrees)
          |                         |                         |
          v                         v                         v
    [.agentic config]       [Tool Config Files]      [Git Database]
```

### 3.2 Technology Stack

* **Language**: Rust (for performance, safety, and single-binary distribution).
* **CLI Framework**: `clap` (v4).
* **Config Parsing**: `toml` (configuration), `markdown` (content).
* **Async Runtime**: `tokio`.
* **Git Integration**: `git2` (libgit2 bindings) or CLI wrapper.

## 4. Key Modules

### 4.1 CLI Interface

Entry point for user interaction.

* `init`: Initialize a new agentic repository.
* `sync`: Synchronize `.agentic` config to tool-specific files.
* `check`: Validate configuration and detect drift.
* `provider`: Manage AI provider integrations.
* `worktree`: Manage git worktrees with shared config.

### 4.2 Configuration Engine

Responsible for reading, validating, and merging configuration from `.agentic/config.toml` and distributed Markdown files.

* Supports hierarchical configuration (Global > Organization > Project > User).
* Implements schema validation (see `08-orchestrator-api-schema.md`).

### 4.3 Provider System

Abstracts the differences between agentic tools.

* **Claude Provider**: Generates `CLAUDE.md`, manages `.claude/`.
* **Cursor Provider**: Generates `.cursorrules`.
* **Windsurf Provider**: Generates `.windsurfrules`.
* **Generic Provider**: basic config files for other tools.

### 4.4 Git Worktree Manager

Handles the complexity of the "Container Pattern" for worktrees (see `03-worktree-patterns.md`).

* Creates sibling worktree directories.
* Ensures configuration visibility across worktrees (via symlinks or copying).

## 5. Integration Strategy

### 5.1 MCP (Model Context Protocol) Support

The Repository Manager will act as an MCP Server to serve configuration to MCP-aware tools.

* **Resource**: `agentic://config` (Current configuration state).
* **Resource**: `agentic://rules` (Merged rules set).
* **Tool**: `agentic_sync` (Trigger sync from within IDE).

### 5.2 CI/CD Integration

* Drift detection via `agentic check --ci`.
* Automated rule generation in build pipelines.

## 6. Directory Structure

### Standard Repository

```text
repo-root/
├── .agentic/
│   ├── config.toml
│   ├── rules/
│   ├── skills/
│   └── workflows/
├── CLAUDE.md (Generated)
├── .cursorrules (Generated)
└── src/
```

### Worktree Container

```text
project-container/
├── .agentic/ (Shared Config)
├── .git/ (Bare-like)
├── main/ (Use Worktree)
├── feature-a/ (Worktree)
└── feature-b/ (Worktree)
```

## 7. Future Roadmap

1. **v1.0**: Core CLI, Claude/Cursor support, Basic Rule Sync.
2. **v1.1**: Git Worktree support, Windsurf provider.
3. **v1.2**: MCP Server implementation.
4. **v2.0**: Plugin system for community providers.
