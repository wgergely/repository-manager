# Agentic Coding Landscape Research (2026)

Research documentation for the `repo-manager` project - a Rust CLI tool for orchestrating agentic development environments across multiple AI coding platforms.

## Research Scope

This research covers:

- **Agentic coding tools** - Claude Code, Cursor, Windsurf, Copilot, Gemini, Amazon Q, Zed, and others
- **Emerging standards** - AGENTS.md (universal rules), MCP (tool integration protocol)
- **Development patterns** - Git worktrees, hooks integration, cross-platform interoperability
- **Rust ecosystem** - CLI frameworks, git libraries, configuration management for implementation

## Document Index

### Standards & Protocols

| Document | Description |
|----------|-------------|
| [standard-agents-md.md](standard-agents-md.md) | AGENTS.md universal rules standard (Linux Foundation) |
| [standard-mcp.md](standard-mcp.md) | Model Context Protocol specification and adoption |
| [standard-other-protocols.md](standard-other-protocols.md) | OpenAI tools, Semantic Kernel, LangChain |

### Tool Deep-Dives

| Document | Description |
|----------|-------------|
| [tool-claude-code.md](tool-claude-code.md) | Anthropic's CLI agentic coding tool |
| [tool-claude-desktop.md](tool-claude-desktop.md) | Anthropic's desktop app with MCP support |
| [tool-cursor.md](tool-cursor.md) | Anysphere's AI-first VS Code fork |
| [tool-windsurf.md](tool-windsurf.md) | Codeium's Cascade-powered IDE (VS Code fork) |
| [tool-antigravity.md](tool-antigravity.md) | Google's agent-first IDE (VS Code fork) |
| [tool-openai-codex.md](tool-openai-codex.md) | OpenAI's CLI agent (AGENTS.md originator) |
| [tool-copilot.md](tool-copilot.md) | GitHub Copilot and Copilot Workspace |
| [tool-gemini-code-assist.md](tool-gemini-code-assist.md) | Google's AI coding assistant |
| [tool-amazon-q.md](tool-amazon-q.md) | AWS's AI developer tool |
| [tool-zed.md](tool-zed.md) | Rust-native high-performance editor |
| [tool-others.md](tool-others.md) | Aider, Continue.dev, Cody, JetBrains AI, etc. |

### Patterns & Architecture

| Document | Description |
|----------|-------------|
| [pattern-git-worktrees.md](pattern-git-worktrees.md) | Worktree-based development with shared config |
| [pattern-git-hooks.md](pattern-git-hooks.md) | Pre-commit frameworks and AI integration |
| [pattern-interoperability.md](pattern-interoperability.md) | Cross-tool rules, memory, and skill sharing |
| [pattern-claude-antigravity-comparison.md](pattern-claude-antigravity-comparison.md) | Direct comparison of Claude Code and Antigravity configs |

### Comparison Matrices

| Document | Description |
|----------|-------------|
| [matrix-feature-support.md](matrix-feature-support.md) | Feature comparison across all tools |
| [matrix-config-locations.md](matrix-config-locations.md) | Configuration file locations by tool |

### Rust Implementation

| Document | Description |
|----------|-------------|
| [rust-cli-frameworks.md](rust-cli-frameworks.md) | clap, argh, dialoguer, indicatif |
| [rust-config-libraries.md](rust-config-libraries.md) | figment, config-rs, serde patterns |
| [rust-git-libraries.md](rust-git-libraries.md) | gix (gitoxide) and git2 comparison |
| [rust-testing-2026.md](2026-01-28-rust-testing-standards.md) | Testing strategies and crates |
| [2026-01-29-rust-registry-patterns.md](2026-01-29-rust-registry-patterns.md) | Plugin/registry patterns (inventory, linkme, enum_dispatch) |

## Key Findings (January 2026)

### Two Standards Have Emerged

1. **AGENTS.md** - Universal rules format
   - Backed by: Google, OpenAI, Cursor, GitHub
   - Governance: Agentic AI Foundation (Linux Foundation)
   - Adoption: 20,000+ repositories on GitHub

2. **MCP (Model Context Protocol)** - Tool integration protocol
   - Adoption: OpenAI, Google DeepMind, Claude, Cursor, Windsurf, Zed, Amazon Q
   - Ecosystem: 100+ servers, SDKs for Python/TypeScript/Rust/Go
   - Current version: 2025-11-25

### Architecture Implications

- Support **AGENTS.md** as first-class citizen (read/write/sync)
- Integrate with **MCP** for tool/resource capabilities
- Generate tool-specific configs from universal sources
- Memory/context portability remains unsolved - potential differentiation

## Related Design Documents

- [../design/architecture-presets.md](../design/architecture-presets.md) - Preset meta-system architecture
- [../design/spec-api-schema.md](../design/spec-api-schema.md) - CLI commands and configuration schema

---

*Last updated: 2026-01-29*
*Branch: research-docs*
