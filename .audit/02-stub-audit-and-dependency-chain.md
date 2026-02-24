# Stub Audit & Dependency Chain Analysis

**Date:** 2026-02-23
**Scope:** All stubs, manifest dependency chains, version management, ADR compliance

---

## 1. Stub Inventory

### 1.1 Extension Lifecycle Stubs (CLI Layer)

| # | Function | File | Line | Status |
|---|----------|------|------|--------|
| 1 | `handle_extension_install()` | `crates/repo-cli/src/commands/extension.rs` | 14 | Returns `CliError::user("not yet implemented")` |
| 2 | `handle_extension_add()` | `crates/repo-cli/src/commands/extension.rs` | 21 | Returns `CliError::user("not yet implemented")` |
| 3 | `handle_extension_init()` | `crates/repo-cli/src/commands/extension.rs` | 28 | Returns `CliError::user("not yet implemented")` |
| 4 | `handle_extension_remove()` | `crates/repo-cli/src/commands/extension.rs` | 35 | Returns `CliError::user("not yet implemented")` |

**Note:** `handle_extension_list()` (line 45) IS implemented and functional.

### 1.2 Extension Lifecycle Stubs (MCP Layer)

| # | Function | File | Status |
|---|----------|------|--------|
| 5 | `handle_extension_install()` | `crates/repo-mcp/src/handlers.rs` | Returns `Error::NotImplemented` |
| 6 | `handle_extension_add()` | `crates/repo-mcp/src/handlers.rs` | Returns `Error::NotImplemented` |
| 7 | `handle_extension_init()` | `crates/repo-mcp/src/handlers.rs` | Returns `Error::NotImplemented` |
| 8 | `handle_extension_remove()` | `crates/repo-mcp/src/handlers.rs` | Returns `Error::NotImplemented` |

### 1.3 Stale MCP Tool Descriptions

| # | Tool | File | Line | Issue |
|---|------|------|------|-------|
| 9 | `git_push` | `crates/repo-mcp/src/tools.rs` | 200 | Description says "[Not implemented]" but handler IS implemented |
| 10 | `git_pull` | `crates/repo-mcp/src/tools.rs` | 217 | Description says "[Not implemented]" but handler IS implemented |
| 11 | `git_merge` | `crates/repo-mcp/src/tools.rs` | 234 | Description says "[Not implemented]" but handler IS implemented |

Evidence: `crates/repo-mcp/tests/git_and_init_tests.rs` lines 131-150 explicitly test that these handlers no longer return `NotImplemented`.

### 1.4 Detection-Only Preset Providers

| # | Provider | File | Behavior |
|---|----------|------|----------|
| 12 | `RustProvider::apply()` | `crates/repo-presets/src/rust/rust_provider.rs` | Returns `ApplyReport::detection_only()` |
| 13 | `NodeProvider::apply()` | `crates/repo-presets/src/node/node_provider.rs` | Returns `ApplyReport::detection_only()` |

These are **intentionally detection-only** (Rust/Node install is user-managed). Not bugs.

---

## 2. ADR Compliance Matrix

### ADR-001: Extension System Architecture

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Extensions as fourth entity type (peer to tools, rules, presets) | PARTIAL | Manifest schema exists, registry exists, but lifecycle commands are stubs |
| TOML manifests (`repo_extension.toml`) | DONE | `crates/repo-extensions/src/manifest.rs` - full parse/validate/serialize |
| Git repo + local path distribution | DESIGNED | `ExtensionConfig.source` field exists, no fetch/clone logic |
| `[outputs]` section mapping | DONE | `Outputs` struct in manifest |
| Ledger tracking with `ext:{name}` intent IDs | NOT STARTED | SyncEngine does not create extension intents |
| Extension state at `.repository/extensions/{name}/` | PARTIAL | Path convention used in MCP resolution, no install creates this |

### ADR-002: Extension Lifecycle & Dependencies

| Requirement | Status | Evidence |
|-------------|--------|----------|
| `repo extension install <url>` | STUB | Returns error in both CLI and MCP layers |
| `repo extension add <name>` | STUB | Returns error in both CLI and MCP layers |
| `repo extension init <name>` | STUB | Returns error in both CLI and MCP layers |
| `repo extension remove <name>` | STUB | Returns error in both CLI and MCP layers |
| `repo extension list` | DONE | Shows registry entries |
| PEP 440 version constraints for Python | NOT STARTED | `PythonRequirement.version` is stored as raw `String`, never parsed/validated |
| `.repository/extensions.lock` file | NOT STARTED | No lock file types or I/O |
| Dependency chain resolution | NOT STARTED | No topological sort, no ordering logic |

### ADR-003: Python Runtime Management

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Per-extension venv at `.repository/extensions/{name}/.venv/` | PARTIAL | `find_extension_python()` checks this path, but no code creates it |
| `uv` as primary Python discovery | DONE | `UvProvider` uses `uv venv --python` |
| PATH fallback if uv unavailable | PARTIAL | `VenvProvider` exists as alternative, but no automatic fallback |
| PEP 440 constraint satisfaction | NOT STARTED | Version string passed raw to `uv`; no constraint parsing |
| Install flow: fetch -> deps -> venv -> install -> activate | NOT STARTED | All install stubs return errors |

---

## 3. Disconnected Subsystems

### 3.1 Preset Providers vs SyncEngine

**Gap:** `PresetProvider.check()` and `PresetProvider.apply()` are never called from `SyncEngine`.

- `SyncEngine::check()` validates ledger projections against filesystem checksums
- `SyncEngine::sync()` writes tool configs and rules
- Neither invokes preset providers to verify/create runtime environments
- Preset add/remove only writes to manifest TOML; no provider invocation

**Impact:** A user can add `env:python` preset, sync, and the Python venv is never created.

### 3.2 Extension Manifest `[requires]` vs Preset System

**Gap:** `ExtensionManifest.requires.python.version` is parsed and stored but never validated against:
- The system's actual Python version
- The preset provider's configured version
- PEP 440 constraint semantics

**Impact:** An extension declaring `requires.python.version = ">=3.13"` will not fail if Python 3.10 is available.

### 3.3 Extension MCP Resolution vs Extension Install

**Gap:** `SyncEngine::resolve_extension_mcp_configs()` reads extension manifests from `.repository/extensions/{name}/` but no code ever populates that directory.

**Impact:** MCP resolution code works correctly but has no data to operate on.

### 3.4 PresetDefinition.requires vs Provider Ordering

**Gap:** `repo-meta` defines `PresetRequires { tools, presets }` for dependency ordering, but:
- No topological sort implementation exists
- No `PresetOrchestrator` coordinates provider execution order
- Providers are standalone; calling one doesn't trigger its dependencies

---

## 4. Version Management Analysis

### 4.1 Extension Versions (Semver)

**Status:** WORKING. `ExtensionManifest` validates version strings via `semver::Version::parse()`.

```rust
// crates/repo-extensions/src/manifest.rs:266
semver::Version::parse(&self.extension.version).map_err(|e| Error::InvalidVersion { ... })?;
```

**Gap:** Only validates that the version is valid semver. No range/constraint comparison (e.g., checking if installed version satisfies `>=0.2.0`).

### 4.2 Python Version Constraints

**Status:** NOT IMPLEMENTED. Raw strings only.

- Stored: `PythonRequirement { version: String }` (e.g., `">=3.13"`)
- Used: `Context::python_version()` returns `"3.12"` (simple version, not constraint)
- Passed: `uv venv --python 3.12` (no constraint check)

**Required:** PEP 440 parsing to validate `">=3.13"` against actual Python `"3.12"`.

### 4.3 Rust Version Constraints

**Status:** NOT APPLICABLE. `RustProvider` is detection-only. No version constraints declared in manifest schema.

### 4.4 Node Version Constraints

**Status:** NOT APPLICABLE. `NodeProvider` is detection-only. No version constraints in manifest schema.

---

## 5. Dependency Chain Architecture (Required by ADRs)

### 5.1 Current State

No dependency chain exists. The expected flow per ADR-002 and ADR-003:

```
Extension Install Request
    |
    v
Parse repo_extension.toml
    |
    v
Check [requires.python] constraints  <-- NOT IMPLEMENTED
    |
    v
Resolve preset dependencies           <-- NOT IMPLEMENTED
(Python ext -> needs env:python preset -> needs venv)
    |
    v
Topological sort dependencies          <-- NOT IMPLEMENTED
    |
    v
Execute dependency chain:
  1. Check/create Python venv          <-- UvProvider exists but not called
  2. pip install extension             <-- NOT IMPLEMENTED
  3. Register in extensions.lock       <-- NOT IMPLEMENTED
  4. Add to manifest config            <-- NOT IMPLEMENTED
  5. Resolve MCP configs               <-- IMPLEMENTED (in SyncEngine)
```

### 5.2 Required Components

1. **`DependencyGraph`** - Topological sort over preset/extension dependency edges
2. **`VersionConstraint`** - PEP 440 parser + semver range comparison
3. **`ExtensionInstaller`** - Fetch, validate, create venv, pip install, register
4. **`PresetOrchestrator`** - Coordinate provider check/apply in dependency order
5. **`LockFile`** - Serialize/deserialize `.repository/extensions.lock`

---

## 6. Recommendations

### Priority 1: Fix Stale Descriptions
- Remove "[Not implemented]" from `git_push`, `git_pull`, `git_merge` tool descriptions (they ARE implemented)

### Priority 2: Version Constraint Validation
- Add `VersionConstraint` type to `repo-extensions` with PEP 440 parsing
- Validate `requires.python.version` against available Python at install time
- Add semver range support for extension version pinning

### Priority 3: Dependency Chain Types
- Add `DependencyGraph` with topological sort to `repo-extensions`
- Define implicit dependency: `runtime.type = "python"` -> requires `env:python` preset
- Wire into extension install flow

### Priority 4: Extension Lifecycle Implementation
- Implement `extension install`: fetch source, parse manifest, validate deps, create venv, pip install
- Implement `extension add`: re-enable from registry
- Implement `extension init`: scaffold `repo_extension.toml`
- Implement `extension remove`: deactivate, clean up venv

### Priority 5: Preset-Sync Integration
- Add `PresetOrchestrator` that runs providers during sync
- Report preset health in `check` output
- Create venvs during `fix` when presets are missing
