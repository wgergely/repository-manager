# Research: Prior Art - Plugin/Extension Systems in Developer Tools

**Date:** 2026-02-19
**Researcher:** extension-prior-art (Opus agent)
**Source:** Web research

---

## 1. VSCode Extension Manifest

### Manifest Format (package.json)
```json
{
  "name": "my-extension",
  "version": "1.0.0",
  "engines": { "vscode": "^1.0.0" },
  "main": "./out/extension.js",
  "activationEvents": ["onLanguage:python", "workspaceContains:**/Cargo.toml"],
  "extensionDependencies": ["ms-python.python"],
  "extensionPack": ["ms-python.vscode-pylance"],
  "contributes": {
    "commands": [{ "command": "ext.hello", "title": "Hello" }],
    "configuration": { "properties": { "myext.enabled": { "type": "boolean" } } }
  }
}
```

### Key Patterns
- **32 contribution point types** (commands, config, menus, keybindings, etc.)
- **Lazy activation** via `activationEvents` - extensions load only when needed
- **`extensionDependencies`** (hard) vs **`extensionPack`** (soft bundling)
- **`engines.vscode`** constrains host platform version
- Extensions run in shared extension host process

### Lessons
1. Declarative `contributes` model - host discovers capabilities from manifest
2. Lazy activation keeps startup fast
3. Hard vs soft dependency distinction is valuable
4. `extensionKind` (UI vs workspace) could translate to "config-time vs runtime"

---

## 2. Pre-commit Framework

### Consumer Config (`.pre-commit-config.yaml`)
```yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
        language_version: python3.11
        additional_dependencies: ['click>=7.0']
```

### Hook Provider Manifest (`.pre-commit-hooks.yaml`)
```yaml
- id: trailing-whitespace
  name: Trim Trailing Whitespace
  entry: trailing-whitespace-fixer
  language: python
  types: [text]
```

### Key Patterns
- **Git repos as plugin source**, pinned to tags/revs
- **Per-hook isolated environments** bootstrapped automatically by language
- **Language-aware**: Python -> venv, Node -> nodeenv, Rust -> cargo install
- **`language_version`** overridable per-hook
- **Consumer-side overrides** for args, files, language_version, additional_dependencies
- **No inter-hook dependencies** - simplifies model

### Lessons
1. Git repos + tag pinning = simple, no registry needed
2. Per-language environment isolation is gold standard
3. `language` field as backend selector
4. Cached environments avoid re-downloading
5. Content-addressable cache keyed on version + deps hash

---

## 3. Devcontainer Features

### Feature Manifest (`devcontainer-feature.json`)
```json
{
  "id": "python",
  "version": "1.2.3",
  "options": {
    "version": { "type": "string", "default": "latest" }
  },
  "dependsOn": { "ghcr.io/devcontainers/features/common-utils:2": {} },
  "installsAfter": ["ghcr.io/devcontainers/features/common-utils"]
}
```

### Key Patterns
- **OCI artifacts** for distribution (container registries)
- **`dependsOn`** (hard, auto-install) vs **`installsAfter`** (soft ordering)
- **Options as env vars** to `install.sh` entry point
- **Topological sort** for installation order
- Features baked into container at build time

### Lessons
1. Two-tier dependency model (hard + soft) well-designed
2. Shell script entry point is simplest interface
3. Feature equality by digest + options prevents duplicates
4. Options as environment variables is language-agnostic

---

## 4. mise (formerly rtx) / asdf Plugin System

### asdf Plugin Convention
Git repos with `bin/` directory:
- `bin/list-all` (required) - outputs versions
- `bin/download` (required) - downloads to ASDF_DOWNLOAD_PATH
- `bin/install` (required) - installs to ASDF_INSTALL_PATH
- `bin/latest-stable`, `bin/exec-env`, `bin/uninstall` (optional)

### mise Evolution
Multiple backend types behind unified `Backend` trait:
- Core tools (native Rust): Node, Python, Ruby
- Package managers: npm, pipx, cargo, gem
- Universal: aqua (registry compiled into binary), GitHub releases
- Legacy: asdf plugins

### Key Patterns
- **Convention over configuration** - directory structure IS the API
- **Multiple backends** behind unified trait
- **Registry compiled into binary** - no network for resolution
- **PATH manipulation over shims** (faster)

### Lessons
1. Start simple (convention), evolve to richer manifests
2. Multiple backend architecture provides flexibility
3. Compiled-in registry for common tools is very fast
4. Plugin extension commands via `lib/commands/`

---

## 5. Terraform Provider Registry

### Provider Requirements
```hcl
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}
```

### Key Patterns
- **Source address**: `hostname/namespace/type` triple
- **Version constraints**: `=`, `!=`, `>`, `>=`, `<`, `<=`, `~>` (pessimistic)
- **Lock file** (`.terraform.lock.hcl`) for reproducibility
- **Providers are standalone Go binaries** communicating via gRPC
- **No inter-provider dependencies**
- **Registry protocol**: 2 REST endpoints (list versions, get download URL)
- **GPG signing** for integrity

### Lessons
1. Source address as identity is clean
2. Lock file separating "range" from "resolved" is essential
3. Process isolation via separate binaries + protocol
4. Simple registry protocol is easy to self-host

---

## 6. Cargo Plugin Convention

### No Formal Manifest
Any binary named `cargo-{subcommand}` in PATH becomes `cargo {subcommand}`.

### Key Patterns
- Zero configuration, pure convention
- PATH-based discovery
- `cargo metadata` JSON output for plugins to query project info
- Install via `cargo install`

### Lessons (Mostly Anti-patterns)
1. No update mechanism
2. No compatibility declarations
3. No dependency management between plugins
4. Convention-only works for simple cases, breaks for complex ones

---

## Cross-Cutting Recommendations

| Aspect | Best Prior Art | Recommendation |
|---|---|---|
| Manifest format | VSCode (package.json) + Devcontainer (feature.json) | Declarative TOML with `contributes`-style provides section |
| Distribution | Pre-commit (git repos + tags) | Git repos + local paths, registry later |
| Dependencies | Devcontainer (`dependsOn` + `installsAfter`) | Hard deps with auto-provision, soft ordering |
| Runtime isolation | Pre-commit (per-hook envs) | Per-extension venv, cached |
| Version constraints | Terraform (`~>`, `>=`, `<`) | PEP 440 for Python, semver for extensions |
| Lock file | Terraform (`.terraform.lock.hcl`) | Separate `extensions.lock` |
| Activation | VSCode (activation events) | Consider lazy activation for future |
