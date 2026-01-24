# Provider Ecosystem Reference

## 1. Provider Design: The Python Ecosystem

The Python ecosystem demonstrates Provider Architecture through `uv` and `conda` abstraction.

### 1.1 The "Python Environment" Abstract Provider

This provider implements the lifecycle for `env:python`.

**Responsibilities:**

1. **Detection**: Determine if `uv` or `conda` is available/requested.
2. **Creation**: Initialize the virtual environment.
3. **Synchronization**: Ensure dependencies (`pyproject.toml` or `environment.yml`) are synced.

#### Strategy A: The `uv` Provider

- **Philosophy**: "Speed & Standards".
- **Mechanism**: Wraps `uv venv` and `uv sync`.
- **State Tracking**: Relies on `uv.lock` as the source of truth.
- **Agent View**: Exposes standard `VIRTUAL_ENV` paths.

#### Strategy B: The `conda` Provider

- **Philosophy**: "Scientific Isolation".
- **Mechanism**: Wraps `micromamba` or `conda env create`.
- **State Tracking**: Relies on `environment.yml`.
- **Agent View**: Must expose the *activated* path constructs, which is harder. It creates a wrapper script for execution.

## 2. Provider Design: The Configuration Provider

The `config:*` namespace handles static file generation. This is crucial for enforcing repository standards.

### 2.1 The Template Engine

Instead of hardcoding files, this provider uses a generic template system.

- **Inputs**: The context (Project Name, Authors, Enabled Tools).
- **Templates**: `.gitignore.hbs`, `.editorconfig.hbs`.
- **Behavior**:
  - *Strict Mode*: Overwrites file on every check.
  - *Merge Mode*: Parses existing file (e.g., reads `.gitignore`) and appends missing critical lines only.

## 3. Provider Design: The Tooling Provider

The `tool:*` namespace manages binary distribution.

- **Problem**: Ensuring every developer has `ruff` or `cargo-nextest` installed.
- **Solution**: A "Version Manager" provider.
  - Checks if binary exists in `.bin/` or `PATH`.
  - If missing, downloads the precompiled asset (or runs `cargo install`).
  - Updates PATH for the shell integration.

### 3.1 Inter-Provider Dependency

The Tool Provider often depends on the Environment Provider.
*Example*: "Install `black`" might technically mean "Ensure `env:python` exists, then run `pip install black` inside it." The Orchestrator resolves this ordering.
