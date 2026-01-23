# Presets Subsystem

**Goal**: A full-featured capability provider system. Presets are dynamic providers that write/modify the repository configuration and files.

## Responsibilities

- **Provider Interface**: Traits for entities that manage specific repository aspects (e.g., `VenvProvider`, `GitIgnoreProvider`).
- **Execution**: Writing config files, running setup commands (pip install, etc.).
- **Sub-crates**:
  - `presets-core`: Interface definitions.
  - `presets-venv`: Python environment management.
  - `presets-node`: Node.js environment management.
