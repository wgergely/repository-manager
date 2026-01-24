# Configuration Strategy

## 1. The Repository Manifest Hierarchy

The configuration system acts as the "DNA" of the agentic repository. It uses a **Hierarchical Merge Strategy** to allow global defaults while permitting granular local control.

### 1.1 Data Model Overview

The configuration is not a flat list; it is a scoped declarations tree.

```text
Global Defaults (System Level)
└── Organization Standards (Shared Git Repo)
    └── Repository Configuration (Committable)
        └── Local Developer Overrides (Git-Ignored)
```

## 2. Configuration Schema Design

The schema is divided into "Namespaces" corresponding to different Provider domains.

### 2.1 The `[presets]` Declaration block

This is the primary interface for enabling capabilities.

```toml
# .repository/config.toml (Conceptual)

[presets]
# Syntax: "namespace:variant" = { properties... }

# Environment Definitions
"env:python" = { provider = "uv", version = "3.12" }
"env:node"   = { provider = "npm", version = "20" }

# Configuration Templates
"config:git" = { strict_ignore = true }
"config:editor" = { style = "vscode" }

# Tooling Injections
"tool:linter" = { use = "ruff", config_strategy = "managed" }
```

### 2.2 Dependency Modeling ("Preset Composition")

Presets form a directed acyclic graph. Presets can imply or require other presets. This allows for "Stacks".

- **Concept**: A `stack:backend` preset is a "Meta-Preset".
- **Resolution**: When the system sees `stack:backend`, it effectively expands it into:
  - `env:python`
  - `tool:linter`
  - `config:git`

### 2.3 Local Overrides Strategy

To support developer preference without breaking the repository standard:

- **Strict Fields**: Defined in the repo config, cannot be overridden (e.g., Python Version 3.12).
- **Loose Fields**: Can be overridden locally (e.g., Provider Preference: `conda` vs `uv`).

## 3. Metadata Exposure (The "Agent Interface")

The system projects resolved configuration into a Runtime Context JSON for agent consumption.

### 3.1 The Context Projection

This read-only view is generated after the configuration is fully resolved and installed.

```json
{
  "runtime": {
    "python": {
      "active_provider": "uv",
      "executable_path": "Y:/code/repo/.venv/Scripts/python.exe",
      "environment_root": "Y:/code/repo/.venv"
    },
    "node": {
      "package_manager": "npm"
    }
  },
  "capabilities": [
    "linting",
    "formatting",
    "testing"
  ]
}
```

*Purpose: An agent reads this JSON to know exactly which executable to call, removing ambiguity.*
