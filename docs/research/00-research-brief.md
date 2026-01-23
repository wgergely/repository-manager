# Agentic Coding Landscape Research Brief (2026)

## Project Overview

A comprehensive research investigation into the agentic coding landscape of 2026, focusing on how frontier agentic coding IDEs and tools manage context, configuration, and cross-platform interoperability.

## Research Phases

### Phase 1: Agentic Tool Configuration Landscape

**Core Questions:**

- How do different agentic coding tools (Claude Code, Cursor, Windsurf, Copilot, Gemini Code Assist, etc.) set and manage:
  - Context windows and retrieval
  - Repository structure conventions
  - Rules and behavioral guidelines
  - Skills/capabilities/plugins
  - Style guides and coding standards
  - Memory and persistent context

**Cross-Platform Interoperability:**

- Can Claude and Gemini use the same set of rules?
- Can memories be shared across platforms?
- Can skills be portable between tools?
- How can we ensure different agentic coding platforms access the same coding standards?
- How can behavioral drift be mitigated across tools?

**Research Areas:**

1. Developer chatter and community discussions
2. Tool documentation and configuration schemas
3. Emerging standards and conventions
4. Plugin/extension architectures
5. MCP (Model Context Protocol) and similar interop layers

### Phase 2: Git Worktree-Based Directory Structure Schema

**Chief Aim:** Enable agentic development tools to read control files from a repository container while keeping git branches isolated in worktrees.

**Candidate Solutions:**

#### Solution A: Centralized Git Database

Analyses of a centralized git database approach where the main git repository remains at a container level and worktrees are checked out as subdirectories.

#### Solution B: Orphaned Utility Branch

Evaluation of an orphaned branch pattern where configuration is stored on a dedicated branch, potentially allowing for cross-project sharing.

#### Solution C: Hybrid Approach

Exploration of a hybrid model using submodules or separate repositories for shared configuration.

**Research Questions:**

- What are the pros and cons of each approach?
- How do current tools handle multi-worktree setups?
- What are the edge cases and failure modes?
- How do different agentic tools discover configuration?
- What patterns exist in the wild?

### Phase 3: Orchestrator Tool Design (Implementation Target)

**Project Name:** `agentic` (working title)

**Vision:** A performant Rust-based utility library for managing agentic development across multiple tools (Claude, Gemini, Cursor, Copilot, etc.).

**Core Functionality Areas:**

- Initialization of provider-specific configurations
- Atomic management of rules, skills, and workflows
- Synchronization across multiple agentic platforms
- Git integration for automated rule application
- High-performance implementation in Rust

**Design Principles:**

1. **Fundamental Actions:** add/remove/modify rule/skill/workflow as atomic operations
2. **Provider Agnostic:** Central config translates to tool-specific formats
3. **Git Integration:** Rules can map to pre-commit hooks (format, lint, AI review)
4. **Plugin Architecture:** Registered plugins auto-update across all supported tools
5. **Performance:** Rust implementation for speed and reliability

**Adjacent Integrations:**

- Pre-commit hooks (husky, lefthook, pre-commit)
- Code formatters (prettier, black, rustfmt)
- Linters (eslint, clippy, golangci-lint)
- These traditional tools share mandate patterns with agentic rules

**Implementation Language:** Rust

## Research Methodology

1. **Documentation Analysis** - Official tool docs, GitHub repos, configuration schemas
2. **Community Research** - Discord, Reddit, HN, X/Twitter developer discussions
3. **Hands-on Testing** - Practical experiments with different tools
4. **Schema Comparison** - Side-by-side analysis of config formats
5. **Gap Analysis** - What's missing in current solutions
6. **Rust Ecosystem Survey** - CLI frameworks, git libraries, config crates

## Expected Deliverables

1. **Landscape Overview** - Current state of agentic coding tools (2026)
2. **Configuration Schema Comparison** - How each tool handles rules/skills/context
3. **Interoperability Analysis** - What can be shared, what can't, and why
4. **Worktree Schema Proposal** - Recommended directory structure with rationale
5. **Tool Support Matrix** - Feature comparison across all major tools
6. **Orchestrator API Design** - CLI commands, config schema, plugin system
7. **Rust Implementation Plan** - Recommended crates and architecture

## Status

### Phase 1: Agentic Tool Landscape

- [x] Research brief created
- [x] Tool configurations research (`01-tool-configurations.md`)
- [x] Cross-platform interoperability research (`02-cross-platform-interop.md`)
- [x] Emerging standards research (`04-emerging-standards.md`)

### Phase 2: Git Worktree Structure

- [x] Worktree patterns research (`03-worktree-patterns.md`)
- [ ] Schema proposal

### Phase 3: Orchestrator Implementation

- [x] Rust CLI ecosystem research (`05-rust-cli-ecosystem.md`)
- [x] Tool support matrix (`06-tool-support-matrix.md`)
- [x] Git hooks ecosystem research (`07-git-hooks-ecosystem.md`)
- [x] Orchestrator API schema design (`08-orchestrator-api-schema.md`)
- [ ] Final synthesis and recommendations

## Research Documents

| # | Document | Status |
|---|----------|--------|
| 00 | Research Brief | Complete |
| 01 | Tool Configurations | Complete |
| 02 | Cross-Platform Interop | In Progress |
| 03 | Worktree Patterns | Complete |
| 04 | Emerging Standards | In Progress |
| 05 | Rust CLI Ecosystem | Drafted |
| 06 | Tool Support Matrix | Drafted (Needs Verification) |
| 07 | Git Hooks Ecosystem | Drafted |
| 08 | Orchestrator API Schema | Drafted |
| 09 | Agentic IDE Landscape | Complete |

---

*Last updated: 2026-01-23*
*Branch: research-docs*
