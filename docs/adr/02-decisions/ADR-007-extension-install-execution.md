# ADR-007: Extension Install Execution — Minimum Viable Path

**Status:** Approved
**Date:** 2026-02-23
**Context:** Bridging the gap between manifest declaration and actual package installation

---

## Context

ADR-002 defined the install lifecycle and ADR-003 defined Python runtime management, but neither
specified *how* `[runtime].install` is actually executed. The current codebase has:

- `RuntimeConfig.install: Option<String>` declared in the manifest struct
- `repo extension install` command that parses the manifest and checks version constraints
- No code that ever shells out to run the install command

The VaultSpec audit (docs/adr/01-vaultspec-audit/) revealed the concrete minimum needed: an
extension ships `install = "uv sync"` in its `repo_extension.toml` alongside its own
`pyproject.toml`. The repo manager must execute that string inside the extension's source
directory with the correct environment.

This ADR defines the minimum viable execution contract.

## Decisions

### 7.1 Execute `[runtime].install` as a Structured Subprocess

**Decision: Execute the install string verbatim via the system shell, scoped to the extension directory.**

```toml
[runtime]
type = "python"
install = "uv sync"
```

Execution contract:
- Working directory: `.repository/extensions/{name}/` (the extension source root)
- Shell: `sh -c "{install}"` on Unix; `cmd /C "{install}"` on Windows
- Environment: inherit parent env, inject `REPO_EXTENSION_NAME`, `REPO_EXTENSION_VERSION`,
  `REPO_ROOT` so install scripts can self-orient
- Stdout/stderr: streamed to the user (not captured silently) so install failures are visible
- Exit code: non-zero exit aborts install with `Error::InstallFailed`

**Rationale:** Extensions know their own dependency tool better than the repo manager does.
`uv sync`, `pip install -e .`, `npm ci`, `cargo build --release` — these are extension concerns.
The repo manager's job is to run the command in the right place with the right env, then verify
the result. Shelling out avoids implementing separate logic for every package manager.

**Rejected alternatives:**
- Call `pip install` directly: forces all Python extensions to use pip; blocks `uv sync`-style
  lockfile-based installs
- Parse the install string and dispatch by tool name: unnecessary complexity; string execution
  covers all cases without special-casing

### 7.2 Pre-Install Dependency Check Gates Execution

**Decision: Check `[requires.python]` version constraint before executing install.**

Install sequence:
1. Parse `repo_extension.toml` → `ExtensionManifest`
2. If `[requires.python].version` is set, run `python3 --version` (or `uv python find`) and
   verify the active Python satisfies the constraint via `ExtensionManifest::python_version_satisfied()`
3. If not satisfied → `Error::VersionConstraintNotSatisfied { constraint, actual }` (user-visible)
4. If satisfied (or no constraint) → execute `[runtime].install`

**Rationale:** Failing fast at the version check gives the user an actionable error
("requires Python >=3.12, found 3.10.4") before any packages are downloaded or partially
installed. Partial installs are harder to recover from than pre-flight failures.

### 7.3 Install Result Written to Lock File

**Decision: On successful install, upsert a `LockedExtension` entry in `extensions.lock`.**

```toml
# .repository/extensions.lock
[[extensions]]
name = "vaultspec"
version = "0.1.0"
source = "git+https://github.com/..."
runtime_type = "python"
python_version = "3.13.1"
```

The `python_version` field is populated by querying `python3 --version` after the install
completes. This records the exact interpreter used for reproducibility.

**Rationale:** The lock file (defined in ADR-002 §2.4) is only useful if it is actually
populated. The install step is the correct time to write the resolved state.

### 7.4 `repo extension install` Is the Entry Point

**Decision: All install logic lives in `handle_extension_install()` in `repo-cli`.**

The function signature evolves from returning stub output to:
1. Resolving the extension directory from the `source` argument
2. Calling `ExtensionManifest::from_path()`
3. Running the pre-flight version check
4. Shelling out to the install command
5. Writing the lock file entry
6. Printing a success summary

The MCP `extension_install` tool delegates to the same core logic via a shared function in
`repo-extensions` or `repo-core`.

## Consequences

- `RuntimeConfig` in `manifest.rs` gains no new fields (this ADR; see ADR-008 for enrichment)
- `Error::InstallFailed { name, command, exit_code, stderr }` added to `repo-extensions/error.rs`
- `HookContext::for_extension_install()` added (but no `PostExtensionInstall` event yet — see ADR-011)
- Install is idempotent: running it twice updates the lock entry but does not error
- Extensions without `[runtime].install` succeed install silently (nothing to run)
