# ADR-003: Python Runtime Management

**Status:** Approved (decisions confirmed)
**Date:** 2026-02-19
**Context:** Managing Python runtimes for Python-based extensions

---

## Context

Python-based extensions (like VaultSpec) require a Python interpreter and isolated environment to run their CLI and MCP servers. The repo manager (written in Rust) must discover, provision, and invoke Python runtimes.

## Decisions

### 3.1 Venv Ownership

**Decision: Repo manager owns the venv.**

Per-extension venv at `.repository/extensions/{name}/.venv/`. The repo manager creates, manages, and destroys it. Per-extension isolation prevents dependency conflicts.

**Rejected alternatives:**
- Extension owns: less control, more coordination problems
- Shared venv: risks dependency conflicts

### 3.2 Python Discovery Strategy

**Decision: Require uv, fallback to PATH.**

Use `uv` as primary tool for Python discovery and venv creation (`uv python find`, `uv venv`). If uv isn't available, fall back to `python3` on PATH.

**Rationale:** uv is a single static binary that handles all cross-platform complexity (PEP 514 on Windows, pyenv shims, etc.). Avoids reimplementing Python discovery in Rust.

**Rejected alternatives:**
- Full multi-source discovery in Rust: significant effort, uv already solves this
- PATH-only: misses Windows py launcher, pyenv, etc.

### 3.3 Dependency Installation Strategy

**Decision: Always pip install.**

Every Python extension must have a proper `pyproject.toml` or `requirements.txt`. Dependencies are installed into the managed venv via `pip install` (or `uv pip install`).

**Note:** This decision was made before the VaultSpec audit revealed that VaultSpec's pyproject.toml is misconfigured (empty top_level.txt, no console_scripts). VaultSpec will need its packaging fixed to be a proper extension citizen. At minimum, a working `requirements.txt` for dependency installation.

**Rejected alternatives:**
- Dual strategy (install vs embedded): adds complexity; fixing VaultSpec's packaging is the cleaner path
- Extension-defined install command: least standardized

### 3.4 Preset Provider Evolution

**Decision: Build orchestrator on top.**

Keep the `PresetProvider` trait as-is. Create a new `PresetOrchestrator` that:
1. Reads resolved config
2. Maps preset IDs to provider instances via the Registry
3. Constructs `Context` objects from config values
4. Calls `check()` then `apply()` if needed
5. Handles dependency ordering (via `PresetDefinition.requires.presets`)

**Rationale:** `apply()` already provisions for UvProvider and VenvProvider. What's missing is the wiring, not the capability. No trait changes needed.

## Consequences

- `uv` becomes a soft dependency (recommended, not required)
- Per-extension `.venv/` in `.repository/extensions/{name}/`
- VaultSpec needs `pyproject.toml` or `requirements.txt` fixes
- New `PresetOrchestrator` in `repo-presets` or `repo-core`
- Python path resolved and passed to MCP config: `{venv}/bin/python` (Unix) or `{venv}/Scripts/python.exe` (Windows)
