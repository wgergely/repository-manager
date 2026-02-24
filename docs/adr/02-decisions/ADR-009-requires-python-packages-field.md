# ADR-009: `[requires.python]` Enrichment — `packages` Field

**Status:** Approved
**Date:** 2026-02-23
**Context:** Declarative Python dependency declaration for extensions without a pyproject.toml

---

## Context

ADR-003 §3.3 decided "every Python extension must have a proper `pyproject.toml` or
`requirements.txt`." But the VaultSpec audit found that VaultSpec's `pyproject.toml` is
misconfigured (empty `top_level.txt`, no `console_scripts`). The decision to "fix VaultSpec's
packaging" was reasonable in January but the gap has persisted.

The feedback from the field identifies a more fundamental gap: there is no way to declare Python
package dependencies *inside* `repo_extension.toml` itself. An extension author wanting to
declare `vaultspec @ git+https://github.com/org/vaultspec` as a dependency has no field to put
it in. They must maintain a separate `requirements.txt` or a correctly-configured `pyproject.toml`
and then write `install = "pip install -r requirements.txt"` — adding boilerplate that the
manifest system was supposed to eliminate.

The `packages` field provides a first-class way to declare pip-installable packages directly
in the manifest, covering the common case without requiring a separate packaging file.

## Decisions

### 9.1 Add `packages` Array to `[requires.python]`

**Decision: Optional list of pip requirement strings.**

```toml
[requires.python]
version = ">=3.12"
packages = [
    "vaultspec @ git+https://github.com/org/vaultspec",
    "httpx>=0.27",
    "pydantic>=2.0,<3.0",
]
```

Rust struct change:
```rust
pub struct PythonRequirement {
    pub version: String,
    #[serde(default)]
    pub packages: Vec<String>,   // NEW
}
```

Each string is a PEP 508 dependency specifier. The repo manager does not parse these; it
passes them directly to the package manager (`uv pip install`, `pip install`) at install time.

**Rationale:** PEP 508 is the universal Python dependency format — pip, uv, poetry, and
hatch all accept it. Passing specifiers through verbatim avoids implementing a PEP 508 parser
in Rust and keeps the manifest readable by Python developers who already know the format.

**Rejected alternatives:**
- Separate `[dependencies]` section: extra nesting for no benefit; `[requires.python]` is
  already the Python-specific section
- Require pyproject.toml: already rejected by the continued presence of misconfigured packages
  in the wild; the manifest should be self-sufficient for simple cases

### 9.2 `packages` Install Behavior

**Decision: When `packages` is non-empty and no explicit `[runtime].install` is set, synthesize
an install command.**

Synthesis logic:
```
package_manager = "uv" (or detected)  → "uv pip install {packages...}"
package_manager = "pip" (or fallback) → "pip install {packages...}"
```

If `[runtime].install` is also explicitly set, the explicit value wins and `packages` is
treated as documentation only (a warning is emitted). This prevents surprising behavior where
both the synthesized command and the explicit command run.

**Rationale:** The most common case is "I have three packages to install." Making this work
without requiring the user to write `install = "pip install httpx pydantic"` reduces boilerplate
and makes the intent clearer. The explicit-wins rule ensures backward compatibility when
`install` is already set.

### 9.3 `packages` Validation at Parse Time

**Decision: Validate that each entry is a non-empty string. Do not parse PEP 508 semantics.**

At `ExtensionManifest::validate()` time:
- Empty strings in the `packages` array → `Error::InvalidPackages { reason: "empty specifier" }`
- Strings containing shell metacharacters (`;`, `&`, `|`, `` ` ``) →
  `Error::InvalidPackages { reason: "shell metacharacter in package specifier" }`

The shell metacharacter check is a security boundary: since package specifiers are interpolated
into shell commands (§9.2), injection must be prevented at parse time.

**Rationale:** A full PEP 508 parser is out of scope. Shell injection is not. Blocking the
obvious injection vectors (`; rm -rf /`) at manifest load time prevents surprises when the
install command is assembled later.

### 9.4 Lock File Records Installed Packages

**Decision: The `LockedExtension` entry records the packages list that was installed.**

```toml
[[extensions]]
name = "my-ext"
version = "1.0.0"
source = "path:./my-ext"
runtime_type = "python"
python_version = "3.13.1"
packages = ["httpx>=0.27", "pydantic>=2.0,<3.0"]   # NEW lock field
```

This is the *requested* specifier list, not the resolved version. Resolved versions live in the
extension's own lock file (e.g., `uv.lock` or `requirements.lock` inside the extension dir).

**Rationale:** Recording the requested specifiers in `extensions.lock` makes `repo extension list`
able to show what was declared, and enables future `repo extension upgrade` to re-run with
updated specifiers.

## Consequences

- `PythonRequirement` gains `pub packages: Vec<String>` in `manifest.rs`
- `LockedExtension` gains `pub packages: Vec<String>` in `lock.rs`
- New `Error::InvalidPackages { reason: String }` in `error.rs`
- Install synthesis logic in `repo-extensions/src/installer.rs`
- `ExtensionManifest::validate()` runs the metacharacter check
- Extensions with `packages = []` (empty) behave identically to omitting the field
- Extensions with both `packages` and `install` log a warning but proceed with `install`
