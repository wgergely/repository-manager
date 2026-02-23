# ADR-010: Venv Path Declaration in Extension Manifest

**Status:** Approved
**Date:** 2026-02-23
**Context:** Making the extension's expected venv location explicit and configurable

---

## Context

ADR-003 §3.1 decided "per-extension venv at `.repository/extensions/{name}/.venv/`." The repo
manager creates this path when running `uv venv`, and MCP config resolution in
`SyncEngine::resolve_extension_mcp_configs()` computes the Python binary path from it.

Two problems with the hardcoded `.venv` assumption:

1. **Extension-owned venvs**: Some extensions (like VaultSpec) already manage their own venv
   at a different path (e.g., `.vaultspec/.venv`). The repo manager currently overwrites
   or ignores this. `install = "uv sync"` in VaultSpec's case lets `uv` manage the venv
   itself — and uv defaults to `.venv` relative to the `pyproject.toml` location, which may
   not be `.repository/extensions/vaultspec/`.

2. **`.python-version` delegation**: uv respects `.python-version` files in the project
   directory. If the extension ships a `.python-version` file, `uv venv` picks up the version
   automatically — but only if the working directory is correct. The current code passes the
   version string from `[requires.python].version` raw to `uv venv --python`, which is a
   constraint (e.g., `>=3.12`) not a pinned version (e.g., `3.12.4`). uv rejects constraint
   strings; it expects a concrete version selector.

This ADR addresses both by making the venv path a first-class manifest field.

## Decisions

### 10.1 Add `venv_path` to `[runtime]`

**Decision: Optional path string in `RuntimeConfig`, relative to the extension source directory.**

```toml
[runtime]
type = "python"
package_manager = "uv"
venv_path = ".venv"          # optional; defaults to managed path
install = "uv sync"
```

Rust struct change:
```rust
pub struct RuntimeConfig {
    pub runtime_type: String,
    pub package_manager: Option<String>,
    pub venv_path: Option<String>,     // NEW
    pub install: Option<String>,
}
```

Resolution priority (highest to lowest):
1. `runtime.venv_path` in `repo_extension.toml` → interpreted as relative to extension source dir
2. `.repository/extensions/{name}/.venv/` (the ADR-003 default)

**Rationale:** Extensions that manage their own virtualenv (e.g., via `uv sync` or
`poetry install`) already have a venv at a known location. The repo manager must find that
venv to construct the Python binary path for MCP config injection. Requiring extensions to
always use the repo-manager-created venv conflicts with tools like uv that create the venv
as a side effect of running `uv sync`.

**Rejected alternatives:**
- Scan for any `.venv` in the extension directory: ambiguous when multiple `.venv` directories
  exist (nested projects); not deterministic
- Require the extension to use the repo-manager venv: forces extensions to use `pip install`
  instead of their native tool, defeating the purpose of ADR-007

### 10.2 `.python-version` Delegation

**Decision: When running `uv venv`, pass `--python` only if `[requires.python].version` is a
pinned version selector, not a range constraint.**

A pinned selector is one that uv accepts: `3.12`, `3.12.4`, `3.13.1`, `>=3.12` (uv interprets
`>=3.12` as "find a 3.12+"). A range constraint like `>=3.10,<3.14` is not a valid `--python`
argument.

Algorithm at install time when the repo manager creates the venv:
1. If `[requires.python].version` is a single-bound constraint (`>=X.Y`) → pass as `--python`
2. If it is a range (`>=X,<Y`) → do NOT pass `--python`; let uv read `.python-version` if present
3. If `.python-version` is absent and no `--python` is passed → uv uses the system default

**Rationale:** uv already handles `.python-version` natively. Trying to convert a range
constraint into a concrete `--python` argument in Rust would require resolving which Python
versions are available on the machine — that's what uv does internally. Delegating to uv's
own logic is the correct abstraction.

**Note:** When `[runtime].install` is set and the repo manager does NOT create the venv itself
(because the install command handles it), this point is moot. This only applies to the
`uv venv`-then-install flow defined in ADR-003.

### 10.3 Security: Venv Path Confinement

**Decision: Reject `venv_path` values that escape the extension source directory.**

```rust
// In ExtensionManifest::validate()
if let Some(ref vp) = runtime.venv_path {
    let path = Path::new(vp);
    if path.is_absolute() || path.components().any(|c| c == Component::ParentDir) {
        return Err(Error::InvalidVenvPath { path: vp.clone() });
    }
}
```

Absolute paths and `..` components in `venv_path` are rejected at parse time.

**Rationale:** A malicious manifest with `venv_path = "../../../../usr/bin"` could cause the
repo manager to read the Python binary path from an attacker-controlled location. The confinement
check mirrors the existing security check in `EntryPoints::resolve_one()`.

### 10.4 Venv Path Written to Lock File

**Decision: Record the resolved venv path in the lock entry.**

```toml
[[extensions]]
name = "vaultspec"
version = "0.1.0"
source = "git+https://..."
runtime_type = "python"
python_version = "3.13.1"
venv_path = ".vaultspec/.venv"    # NEW — resolved path relative to extension source
```

**Rationale:** MCP config resolution (`SyncEngine::resolve_extension_mcp_configs()`) currently
recomputes the venv path from the `Context` struct. Recording the resolved path in the lock
file eliminates the recomputation and ensures that MCP configs point to the same venv that was
used during install, even if the manifest changes between installs.

## Consequences

- `RuntimeConfig` gains `pub venv_path: Option<String>` in `manifest.rs`
- `LockedExtension` gains `pub venv_path: Option<String>` in `lock.rs`
- New `Error::InvalidVenvPath { path: String }` in `error.rs`
- `SyncEngine::resolve_extension_mcp_configs()` reads `venv_path` from lock file first,
  falls back to `Context.venv_path()` default
- `ExtensionManifest::validate()` runs the confinement check
- Existing manifests without `venv_path` continue to use the ADR-003 default (no breakage)
- `.python-version` delegation logic lives in the installer, not the manifest
