# Extension Install Implementation Plan

**Date:** 2026-02-23
**ADRs:** ADR-007, ADR-008, ADR-009, ADR-010, ADR-011
**Branch:** `claude/audit-repository-stubs-eaW3V` (this branch)

---

## Overview

This plan implements the five gaps identified in the Python package installation feedback.
Work is ordered by dependency: the minimum viable install path (ADR-007) must land first
because all subsequent work builds on it.

**End state:** `repo extension install ./vaultspec` fetches the manifest, checks the Python
version, runs `uv sync` (or the declared install command), fires a `post-extension-install`
hook, and writes a fully-populated `extensions.lock` entry.

---

## Phase 1 — Minimum Viable Install (ADR-007)

**Deliverable:** `repo extension install <path>` actually runs the install command.

### 1.1 Add `Error::InstallFailed` to `repo-extensions/src/error.rs`

```rust
/// The extension's install command failed.
#[error("install failed for '{name}': command '{command}' exited with {exit_code:?}\n{stderr}")]
InstallFailed {
    name: String,
    command: String,
    exit_code: Option<i32>,
    stderr: String,
},
```

### 1.2 Create `repo-extensions/src/installer.rs`

New file. Contains:

```rust
/// Execute the extension's install command.
pub fn run_install(
    name: &str,
    install_cmd: &str,
    working_dir: &Path,
    repo_root: &Path,
) -> Result<()>
```

- On Unix: `Command::new("sh").arg("-c").arg(install_cmd)`
- On Windows: `Command::new("cmd").args(["/C", install_cmd])`
- Working dir: `working_dir` (the extension source directory)
- Environment: inherit parent + inject `REPO_EXTENSION_NAME`, `REPO_EXTENSION_VERSION`,
  `REPO_ROOT`
- Stdout/stderr: `inherit()` (stream to terminal, not captured)
- Non-zero exit → `Error::InstallFailed { name, command, exit_code, stderr }`

Also expose:

```rust
/// Verify a binary is on PATH. Returns the resolved path or Error::PackageManagerNotFound.
pub fn check_binary_on_path(tool: &str) -> Result<PathBuf>
```

### 1.3 Export `installer` from `repo-extensions/src/lib.rs`

```rust
pub mod installer;
pub use installer::{run_install, check_binary_on_path};
```

### 1.4 Update `handle_extension_install()` in `repo-cli/src/commands/extension.rs`

Replace the current stub that reports dependency info with:

1. Resolve `source` arg to an extension directory path
2. `ExtensionManifest::from_path(dir / "repo_extension.toml")`
3. If `[requires.python].version` set: query Python version via `python3 --version` (or
   `uv python find --quiet`), call `manifest.python_version_satisfied()`
4. If `[runtime].package_manager` set: `check_binary_on_path(tool)?`
5. If `[runtime].install` set: `run_install(name, install_cmd, &extension_dir, &repo_root)?`
6. Load or create `.repository/extensions.lock`
7. Upsert `LockedExtension` entry with all resolved fields
8. Save lock file
9. Print success: `✓ Installed {name} v{version}`

### 1.5 Query Python Version (helper)

```rust
/// Returns the active Python version string (e.g., "3.13.1").
pub fn query_python_version(python_cmd: Option<&str>) -> Result<String>
```

Tries in order:
1. `uv python find --quiet` (returns path; run `{path} --version` to get version)
2. `python3 --version`
3. `python --version`

Parses `"Python 3.13.1"` → `"3.13.1"`.

### 1.6 Tests

- `test_run_install_no_op_when_no_install_command` — manifest without `[runtime].install` succeeds
- `test_run_install_exits_on_nonzero` — `install = "exit 1"` returns `Error::InstallFailed`
- `test_lock_file_written_after_install` — upsert creates the expected entry
- `test_python_version_parse` — `"Python 3.13.1"` → `"3.13.1"`

---

## Phase 2 — Runtime Enrichment (ADR-008)

**Deliverable:** `package_manager` field in manifest + binary check.

### 2.1 Update `RuntimeConfig` in `manifest.rs`

```rust
pub struct RuntimeConfig {
    #[serde(rename = "type")]
    pub runtime_type: String,
    #[serde(default)]
    pub package_manager: Option<String>,    // NEW
    #[serde(default)]
    pub install: Option<String>,
}
```

### 2.2 Add Validation in `ExtensionManifest::validate()`

```rust
const KNOWN_PACKAGE_MANAGERS: &[&str] = &[
    "uv", "pip", "npm", "yarn", "pnpm", "cargo", "bun",
];

if let Some(ref rt) = self.runtime {
    if let Some(ref pm) = rt.package_manager {
        if !KNOWN_PACKAGE_MANAGERS.contains(&pm.as_str()) {
            return Err(Error::InvalidPackageManager { value: pm.clone() });
        }
    }
}
```

### 2.3 Add `Error::InvalidPackageManager` and `Error::PackageManagerNotFound`

```rust
#[error("unknown package_manager '{value}'; known values: uv, pip, npm, yarn, pnpm, cargo, bun")]
InvalidPackageManager { value: String },

#[error("install requires '{tool}' but it was not found on PATH{hint}")]
PackageManagerNotFound { tool: String, hint: String },
```

Where `hint` is populated from a static map:
```rust
fn install_hint(tool: &str) -> &'static str {
    match tool {
        "uv" => "\n  Install: curl -LsSf https://astral.sh/uv/install.sh | sh",
        "npm" => "\n  Install: https://nodejs.org",
        "cargo" => "\n  Install: https://rustup.rs",
        _ => "",
    }
}
```

### 2.4 Update `LockedExtension` in `lock.rs`

```rust
pub struct LockedExtension {
    pub name: String,
    pub version: String,
    pub source: String,
    pub resolved_ref: Option<String>,
    pub runtime_type: Option<String>,
    pub python_version: Option<String>,
    pub package_manager: Option<String>,   // NEW
}
```

### 2.5 Wire into Phase 1 install sequence

After step 3 (version check) in §1.4, insert:
```
4. If manifest.runtime.package_manager is Some(tool):
       check_binary_on_path(tool)?
```

### 2.6 Tests

- `test_invalid_package_manager_rejected` — `package_manager = "poetry"` fails validation
- `test_known_package_managers_accepted` — all 7 known values pass
- `test_package_manager_not_found_error` — `check_binary_on_path("nonexistent_tool_xyz")` → Err

---

## Phase 3 — Python Packages Field (ADR-009)

**Deliverable:** `packages` array in `[requires.python]` synthesizes install command.

### 3.1 Update `PythonRequirement` in `manifest.rs`

```rust
pub struct PythonRequirement {
    pub version: String,
    #[serde(default)]
    pub packages: Vec<String>,    // NEW
}
```

### 3.2 Add `Error::InvalidPackages` and shell metacharacter check

```rust
#[error("invalid packages declaration: {reason}")]
InvalidPackages { reason: String },
```

In `validate()`:
```rust
const SHELL_METACHARACTERS: &[char] = &[';', '&', '|', '`', '$', '(', ')'];

for pkg in &python_req.packages {
    if pkg.is_empty() {
        return Err(Error::InvalidPackages { reason: "empty specifier".to_string() });
    }
    if pkg.chars().any(|c| SHELL_METACHARACTERS.contains(&c)) {
        return Err(Error::InvalidPackages {
            reason: format!("shell metacharacter in package specifier: {:?}", pkg),
        });
    }
}
```

### 3.3 Add `synthesize_install_command()` to `installer.rs`

```rust
/// Synthesize a pip install command from a packages list.
/// Returns None if packages is empty.
pub fn synthesize_install_command(
    packages: &[String],
    package_manager: Option<&str>,
) -> Option<String> {
    if packages.is_empty() {
        return None;
    }
    let pkg_list = packages.join(" ");
    match package_manager {
        Some("uv") => Some(format!("uv pip install {}", pkg_list)),
        _ => Some(format!("pip install {}", pkg_list)),
    }
}
```

### 3.4 Wire into install sequence

After step 4 in §1.4, before step 5:
```
4b. Determine effective install command:
    - explicit: manifest.runtime.install (wins if set; warn if packages also set)
    - synthesized: synthesize_install_command(packages, package_manager)
    - none: skip install step
```

### 3.5 Update `LockedExtension`

```rust
pub packages: Vec<String>,    // NEW
```

### 3.6 Tests

- `test_packages_metacharacter_rejected` — `packages = ["httpx; rm -rf /"]` fails
- `test_packages_empty_string_rejected`
- `test_synthesize_uv_command` — `["httpx>=0.27"]` + `"uv"` → `"uv pip install httpx>=0.27"`
- `test_synthesize_pip_fallback` — no package_manager → `"pip install ..."`
- `test_explicit_install_wins_over_packages`

---

## Phase 4 — Venv Path Declaration (ADR-010)

**Deliverable:** `venv_path` field in manifest, confinement check, lock file recording.

### 4.1 Update `RuntimeConfig` in `manifest.rs`

```rust
pub struct RuntimeConfig {
    #[serde(rename = "type")]
    pub runtime_type: String,
    #[serde(default)]
    pub package_manager: Option<String>,
    #[serde(default)]
    pub venv_path: Option<String>,          // NEW
    #[serde(default)]
    pub install: Option<String>,
}
```

### 4.2 Add `Error::InvalidVenvPath` and confinement check in `validate()`

```rust
#[error("invalid venv_path '{path}': must be a relative path within the extension directory")]
InvalidVenvPath { path: String },
```

```rust
if let Some(ref vp) = rt.venv_path {
    let path = Path::new(vp);
    if path.is_absolute() || path.components().any(|c| c == Component::ParentDir) {
        return Err(Error::InvalidVenvPath { path: vp.clone() });
    }
}
```

### 4.3 Update `LockedExtension`

```rust
pub venv_path: Option<String>,    // NEW — resolved relative to extension source dir
```

### 4.4 Update `SyncEngine::resolve_extension_mcp_configs()`

When building the Python binary path:
1. Check `LockedExtension.venv_path` first
2. Fall back to `Context.venv_path()` (the `.repository/extensions/{name}/.venv` default)

### 4.5 `.python-version` delegation

In the venv creation code (currently in `UvProvider::apply()`):
- If `[requires.python].version` is a range (contains `,`) → do NOT pass `--python` flag;
  uv picks up `.python-version` from the working directory or falls back to its own defaults.
- If it is a single specifier like `>=3.12` → pass the full specifier as-is: `--python >=3.12`;
  uv accepts PEP 440 version specifiers directly and resolves the best matching interpreter.
- If `.python-version` exists in the extension dir → uv picks it up automatically (range case)

### 4.6 Tests

- `test_absolute_venv_path_rejected`
- `test_parent_dir_venv_path_rejected` — `venv_path = "../other"` fails
- `test_valid_venv_path_accepted` — `venv_path = ".vaultspec/.venv"` passes
- `test_resolve_extension_mcp_uses_lock_venv_path` — SyncEngine reads from lock entry

---

## Phase 5 — Post-Extension-Install Hook (ADR-011)

**Deliverable:** `PostExtensionInstall` hook event fires after successful install.

### 5.1 Update `HookEvent` in `repo-core/src/hooks.rs`

Add variant:
```rust
PostExtensionInstall,
```

Update `Display`, `parse()`, `all_names()`.

### 5.2 Add `HookContext::for_extension_install()`

```rust
pub fn for_extension_install(
    name: &str,
    version: &str,
    source: &str,
    extension_dir: &Path,
    venv_path: Option<&Path>,
) -> Self
```

Variables: `EXTENSION_NAME`, `EXTENSION_VERSION`, `EXTENSION_SOURCE`, `EXTENSION_DIR`,
`EXTENSION_VENV` (optional).

### 5.3 Update hook test assertion count: 6 → 7

`test_hook_event_enum_has_no_agent_events`: update expected count and expected names set.

### 5.4 Wire into `handle_extension_install()`

After lock file write (step 8 in §1.4):
```rust
let hook_ctx = HookContext::for_extension_install(...);
run_hooks(&config.hooks, HookEvent::PostExtensionInstall, &hook_ctx, &repo_root)?;
```

### 5.5 Wire `handle_extension_reinit()` to re-fire hooks

```rust
pub fn handle_extension_reinit(name: &str, config: &Config, repo_root: &Path) -> Result<()> {
    let lock = LockFile::load(&repo_root.join(".repository/extensions.lock"))?;
    let entry = lock.get(name).ok_or(Error::ExtensionNotInstalled(name.to_string()))?;
    let hook_ctx = HookContext::for_extension_install(
        &entry.name, &entry.version, &entry.source,
        &extension_dir_from_lock(entry, repo_root),
        entry.venv_path.as_deref().map(Path::new),
    );
    run_hooks(&config.hooks, HookEvent::PostExtensionInstall, &hook_ctx, repo_root)?;
    Ok(())
}
```

### 5.6 Tests

- `test_post_extension_install_hook_fires` — hook with marker file side effect
- `test_post_extension_install_not_fired_on_failure` — install failure before hook
- `test_extension_reinit_refires_hooks`

---

## Implementation Order

```
Phase 1 (ADR-007) → Phase 2 (ADR-008) → Phase 3 (ADR-009) → Phase 4 (ADR-010) → Phase 5 (ADR-011)
```

Phases 2–4 can be parallelized once Phase 1 is complete (they are independent struct fields).
Phase 5 can be started in parallel with 2–4 since the hook infrastructure is in a different crate.

## Files Modified

| File | Phases |
|------|--------|
| `crates/repo-extensions/src/error.rs` | 1, 2, 3, 4 |
| `crates/repo-extensions/src/installer.rs` | 1 (new file), 2, 3 |
| `crates/repo-extensions/src/manifest.rs` | 2, 3, 4 |
| `crates/repo-extensions/src/lock.rs` | 2, 3, 4 |
| `crates/repo-extensions/src/lib.rs` | 1 |
| `crates/repo-cli/src/commands/extension.rs` | 1, 5 |
| `crates/repo-core/src/hooks.rs` | 5 |
| `crates/repo-core/src/sync/engine.rs` | 4 |

## Acceptance Criteria

- [ ] `repo extension install ./vaultspec` runs `uv sync` in the extension directory
- [ ] Version constraint mismatch prints actionable error before any install
- [ ] Missing `uv` binary prints actionable error with install hint
- [ ] `packages = ["httpx>=0.27"]` without `install` installs via `uv pip install httpx>=0.27`
- [ ] Shell metacharacters in `packages` are rejected at manifest parse time
- [ ] `venv_path = ".vaultspec/.venv"` is validated and recorded in the lock file
- [ ] MCP config resolution uses the lock-recorded venv path
- [ ] `post-extension-install` hooks fire after successful install
- [ ] `repo extension reinit <name>` re-fires `post-extension-install` hooks
- [ ] All existing tests continue to pass
