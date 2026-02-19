# ADR-002: Extension Lifecycle and Dependencies

**Status:** Approved (decisions confirmed)
**Date:** 2026-02-19
**Context:** Defining the extension install/add/init/remove lifecycle and dependency resolution

---

## Context

Extensions need a clear lifecycle with separate install, activation, and initialization phases. They must also declare dependencies on host capabilities (like Python runtimes) that the repo manager can verify and provision.

## Decisions

### 2.1 Lifecycle Commands

**Decision: Combined install with opt-out.**

| Command | Semantics |
|---------|-----------|
| `repo extension install <url>` | Fetch, set up runtime, activate in config. Use `--no-activate` to skip activation. |
| `repo extension add <name>` | Re-enable a previously installed extension |
| `repo extension init <name>` | Run extension's initialization logic |
| `repo extension remove <name>` | Deactivate + uninstall |
| `repo extension list` | Show installed/active extensions and status |

**Rationale:** Most users want install-and-go. Power users get `--no-activate`. Mirrors `cargo add` convention.

### 2.2 Dependency Declaration Format

**Decision: Structured requirements.**

```toml
[requires.python]
version = ">=3.10,<3.13"
```

Typed requirements with constraint parameters per capability. Resolved against preset providers.

**Rejected alternatives:**
- Flat strings: can't carry version constraints
- dependsOn: over-engineered for current scope

### 2.3 Version Constraint Format

**Decision: PEP 440 for Python, semver for extensions.**

- Python version constraints use PEP 440 (`>=3.10,<3.13`) - it's the Python standard
- Extension versioning uses semver - it's the Rust/general convention
- Available crates: `pep440_rs`, `semver`

### 2.4 Lock File

**Decision: Separate `extensions.lock`.**

New file at `.repository/extensions.lock`. The ledger tracks sync state (intents/projections). The lock file tracks resolved dependency versions. Different concerns, different files.

## Consequences

- `repo extension` nested CLI command group (matches `branch`/`config`/`hooks` pattern)
- Extension install flow: fetch -> check deps -> create venv -> install deps -> activate
- Version constraints parsed at install time and recorded in lock file
- Lock file committed to version control for reproducibility
- 5 new MCP tools: `extension_install`, `extension_add`, `extension_init`, `extension_remove`, `extension_list`
