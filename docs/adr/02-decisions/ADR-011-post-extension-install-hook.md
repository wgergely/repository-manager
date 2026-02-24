# ADR-011: `PostExtensionInstall` Hook Event

**Status:** Approved
**Date:** 2026-02-23
**Context:** Enabling post-install automation for extensions

---

## Context

`HookEvent` currently has 6 variants covering branch creation, deletion, and sync:

```
pre-branch-create  post-branch-create
pre-branch-delete  post-branch-delete
pre-sync           post-sync
```

The install lifecycle (ADR-007) is a significant event — it creates a venv, installs packages,
and makes a new tool available. Several real post-install scenarios are impossible to hook today:

- Running the extension's own setup wizard (`vaultspec --init`)
- Registering the extension's MCP server with Claude Desktop outside of the repo manager
- Notifying a CI system that a new extension is available
- Validating that the installed extension's CLI actually works (`vaultspec --version`)

The hook system's test `test_hook_event_enum_has_no_agent_events` explicitly asserts 6 variants
and documents that adding a new event requires wiring it to a call site. This ADR does that.

## Decisions

### 11.1 Add `PostExtensionInstall` to `HookEvent`

**Decision: New variant with kebab-case name `post-extension-install`.**

```rust
pub enum HookEvent {
    PreBranchCreate,
    PostBranchCreate,
    PreBranchDelete,
    PostBranchDelete,
    PreSync,
    PostSync,
    PostExtensionInstall,   // NEW
}
```

`Display`, `parse()`, `all_names()`, and the serde serialization are all extended to cover the
new variant. The enum count assertion in the test is updated from 6 to 7.

**Why no `PreExtensionInstall`?** The pre-install scenario (e.g., "back up current state") is
covered by the existing install validation sequence (ADR-007 §7.2) which already gates on
version and binary checks before anything is mutated. Adding a pre-install hook now would be
speculative.

**Rationale:** `PostExtensionInstall` is the only install-related hook needed today and is
directly motivated by the VaultSpec `--init` pattern.

### 11.2 Hook Context Variables for Extension Install

**Decision: `HookContext::for_extension_install()` provides extension-specific variables.**

```rust
impl HookContext {
    pub fn for_extension_install(
        name: &str,
        version: &str,
        source: &str,
        extension_dir: &Path,
        venv_path: Option<&Path>,
    ) -> Self {
        let mut vars = HashMap::new();
        vars.insert("EXTENSION_NAME".to_string(), name.to_string());
        vars.insert("EXTENSION_VERSION".to_string(), version.to_string());
        vars.insert("EXTENSION_SOURCE".to_string(), source.to_string());
        vars.insert("EXTENSION_DIR".to_string(), extension_dir.display().to_string());
        if let Some(venv) = venv_path {
            vars.insert("EXTENSION_VENV".to_string(), venv.display().to_string());
        }
        Self { vars }
    }
}
```

These variables are substituted via `${VAR_NAME}` in hook `args` (the existing substitution
mechanism in `hooks.rs::substitute_vars()`).

**Rationale:** Post-install hooks need to know where the extension lives and what Python
environment it uses. Without `EXTENSION_DIR` and `EXTENSION_VENV`, a hook script cannot
run the installed tool.

Example hook config:
```toml
[[hooks]]
event = "post-extension-install"
command = "${EXTENSION_VENV}/bin/python"
args = ["-m", "${EXTENSION_NAME}", "--init", "--repo-root", "${REPO_ROOT}"]
```

### 11.3 Call Site: `run_hooks` Invoked After Successful Install

**Decision: `handle_extension_install()` calls `run_hooks()` with `PostExtensionInstall`
after the lock file is written.**

```rust
// After lock file is written:
let hook_ctx = HookContext::for_extension_install(
    &manifest.extension.name,
    &manifest.extension.version,
    &source,
    &extension_dir,
    resolved_venv.as_deref(),
);
run_hooks(config.hooks.as_slice(), HookEvent::PostExtensionInstall, &hook_ctx, &repo_root)?;
```

Hook failures abort the install with `Error::HookFailed` (existing variant, same semantics as
other hook failure sites). The lock file entry is already written at this point; a failed
post-install hook leaves the extension in an installed-but-not-initialized state.

**Rationale:** Writing the lock file before running hooks means the extension is "installed"
even if the post-hook fails. This matches the semantics of `PostSync` — the sync has happened
before hooks run. The user can re-run `repo extension reinit <name>` to retry initialization
without re-installing.

### 11.4 `repo extension reinit` Is the Re-Hook Path

**Decision: `repo extension reinit <name>` re-runs `PostExtensionInstall` hooks for a named extension.**

```
repo extension reinit vaultspec
  → load config
  → find vaultspec in extensions.lock
  → build HookContext::for_extension_install(...)
  → run_hooks(..., PostExtensionInstall, ...)
```

**Why `reinit` and not `init`?** `repo extension init <name>` is reserved for scaffold
generation — it creates the directory layout and starter files for a new extension (the
equivalent of `cargo new` for extensions). Re-firing hooks is a distinct lifecycle operation,
so it uses `reinit` to avoid overloading `init` with two unrelated meanings.

**Rationale:** Decoupling initialization from installation means failed inits can be retried
cheaply. The hook config owns what "initialize" means for a given extension.

## Consequences

- `HookEvent` gains `PostExtensionInstall` variant in `repo-core/src/hooks.rs`
- `HookEvent::all_names()` grows to 7 entries
- `HookContext::for_extension_install()` added to `repo-core/src/hooks.rs`
- Test `test_hook_event_enum_has_no_agent_events` updated: count becomes 7, expected set grows
- `handle_extension_install()` in `repo-cli` calls `run_hooks()` after lock file write
- `handle_extension_reinit()` in `repo-cli` re-fires `PostExtensionInstall` hooks
- `HookEvent::Display` and `HookEvent::parse()` updated for `post-extension-install`
- Existing repos without `post-extension-install` hooks are unaffected (zero matching hooks
  → `run_hooks` returns empty vec with no side effects)
