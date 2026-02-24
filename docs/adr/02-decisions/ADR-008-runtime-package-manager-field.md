# ADR-008: `[runtime]` Enrichment — `package_manager` Field

**Status:** Approved
**Date:** 2026-02-23
**Context:** Making install orchestration explicit and verifiable

---

## Context

ADR-007 §7.1 decided to execute `[runtime].install` verbatim. That solves the immediate
problem but introduces a new one: the repo manager cannot verify that the required package
manager binary is available before trying to run the install string.

VaultSpec ships `install = "uv sync"`. If `uv` is not installed, the user gets a confusing
shell error instead of an actionable message. Similarly, a Node extension with
`install = "npm ci"` fails differently on a machine without npm.

Beyond availability checking, the `package_manager` field enables smarter pre-install
behavior:
- Auto-install uv if the machine has Python but not uv (uv can self-bootstrap)
- Use `uv pip install` path when `package_manager = "uv"` is declared alongside a
  `requirements.txt`-style install string
- Surface the exact tool in `extension list` output for diagnostics

## Decisions

### 8.1 Add `package_manager` to `[runtime]`

**Decision: Optional string field in `RuntimeConfig`.**

```toml
[runtime]
type = "python"
package_manager = "uv"          # optional; hints which binary is expected
install = "uv sync"
```

Accepted values (validated at parse time): `"uv"`, `"pip"`, `"npm"`, `"yarn"`, `"pnpm"`,
`"cargo"`, `"bun"`. Any other value is an `Error::InvalidPackageManager`.

Rust struct change:
```rust
pub struct RuntimeConfig {
    pub runtime_type: String,
    pub package_manager: Option<String>,  // NEW
    pub install: Option<String>,
}
```

**Rationale:** A free-form hint rather than an enum gives extension authors flexibility
(e.g., `"uv"` today, some future tool tomorrow) while still enabling the repo manager
to check availability. The optional nature preserves backward compatibility with existing
manifests that omit it.

**Rejected alternatives:**
- Infer from install string (parse `uv sync` → requires `uv`): fragile; breaks for
  multi-step installs like `pip install build && python -m build`
- Enum in TOML: requires a crate update every time a new tool is added; extension authors
  cannot use non-enumerated tools

### 8.2 Pre-Install Binary Check

**Decision: When `package_manager` is set, verify the binary is on PATH before executing install.**

```
repo extension install ./vaultspec
  → parse manifest → package_manager = "uv"
  → which uv → not found
  → Error: install requires 'uv' but it was not found on PATH.
    Install with: curl -LsSf https://astral.sh/uv/install.sh | sh
```

The error message includes a known install hint for `uv`, `npm`, `cargo`. For unknown
tools, the message is generic: "install requires '{tool}' but it was not found on PATH."

This check happens *after* the Python version check (ADR-007 §7.2) and *before* shelling
out.

**Rationale:** Binary-not-found errors from `sh -c "uv sync"` produce opaque OS errors
("No such file or directory" or "command not found"). Checking explicitly lets the repo
manager produce a targeted, helpful error.

### 8.3 `package_manager` Written to Lock File

**Decision: Record `package_manager` in the lock entry alongside `runtime_type`.**

```toml
[[extensions]]
name = "vaultspec"
version = "0.1.0"
source = "git+https://..."
runtime_type = "python"
package_manager = "uv"          # NEW lock field
python_version = "3.13.1"
```

**Rationale:** Enables future `repo extension reinstall` logic to use the same tool that
was used originally, without re-parsing the manifest.

### 8.4 `extension list` Shows Package Manager

**Decision: `repo extension list` and `extension_list` MCP tool surface `package_manager`.**

```
NAME        VERSION  RUNTIME  MANAGER  STATUS
vaultspec   0.1.0    python   uv       installed
```

When `package_manager` is absent from the lock file (legacy entry or not declared in
manifest), the column shows `—`.

## Consequences

- `RuntimeConfig` gains `pub package_manager: Option<String>` in `manifest.rs`
- `LockedExtension` gains `pub package_manager: Option<String>` in `lock.rs`
- New `Error::InvalidPackageManager { value: String }` in `error.rs`
- New `Error::PackageManagerNotFound { tool: String, hint: Option<String> }` in `error.rs`
- Binary check logic in a new `fn check_package_manager(tool: &str) -> Result<PathBuf>` helper
  in `repo-extensions/src/installer.rs` (new file, ADR-007 impl target)
- Manifest validation in `ExtensionManifest::validate()` checks the allowed values list
- Existing manifests without `package_manager` continue to work (field is optional)
