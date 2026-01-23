# Project Overview

**Repository Manager** is a Rust-based CLI tool designed to orchestrate the "Agentic Workspace." It acts as a bridge between human developers and artificial intelligence coding agents (Claude Code, Gemini CLI, Windsurf, Cursor), ensuring that all participants in the codebase adhere to the same rules, use the same tools, and share a consistent context.

## 1. Intent & Purpose

In 2026, development environments are fragmented. A human uses VSCode; one agent uses a CLI interface; another agent lives inside the IDE. Each has its own way of defining "good code" (linters, formatters) and "how to work" (tasks, scripts).

**The Repository Manager solves this by providing a unified Control Plane.**

It establishes a **Single Source of Truth** structure (`.repository`) that abstracts away the specific configuration formats of individual tools. You declare your intent ("Use Python 3.12 with strict typing"), and the Repository Manager "unrolls" this intent into:

* `.vscode/settings.json` for the human.
* `.cursorrules` for the Cursor AI.
* `.claude/config.json` for the Claude CLI.
* `pyproject.toml` for the build system.

## 2. Top-Level Capabilities

### 🛡️ Single Source of Truth

Define rules, tools, and presets in a central, agnostic schema. The manager handles the translation and synchronization to ensure no tool drifts out of alignment.

### 🌳 Workspace Virtualization

Support distinct operational **Modes**:

* **Standard Mode**: Classic single-directory Git repository.
* **Worktree Mode**: Advanced container-based layout where branches exist as parallel sibling directories (`main/`, `feature-a/`, `feature-b/`), allowing simultaneous context switching for human and agents without file thrashing.

### 📦 The Preset System

Capabilities are modularized into **Presets**. Instead of manually configuring `git`, `python`, `ruff`, and `pytest` individually, you apply the `env:python` and `tool:agentic-quality` presets. The system automatically installs the necessary binaries, creates virtual environments, and configures the paths.

### 🤖 Agentic Orchestration

Treats AI Agents as first-class citizens. The Manager can:

* Inject system prompts and project rules into agent contexts.
* Register "Skills" (MCP servers, scripts) that agents can invoke.
* Validate that agents have not hallucinated changes to read-only configuration.

## 3. Reference Architecture

The system is built as a suite of modular Rust crates, emphasizing performance, safety, and strict separation of concerns.

* `repo-cli`: The user-facing command line interface.
* `repo-core`: The orchestration logic and configuration resolution.
* `repo-fs`: Robust file handling and atomic I/O.
* `repo-git`: High-level Git and Worktree abstractions.
* `repo-tools`: Integration logic for third-party tools (VSCode, Claude, etc.).

> **Next Steps**: See the [Design Documentation](design/_index.md) and [Research Documentation](research/_index.md) for background context.
