# ADR-001: Extension System Architecture

**Status:** Approved (decisions confirmed)
**Date:** 2026-02-19
**Context:** Introducing extensions as a new entity type to the repository manager

---

## Context

The repository manager needs to support external packages (like VaultSpec) that provide rules, tool definitions, presets, MCP servers, and runtime services. Existing entity types (tools, rules, presets) don't capture this concept.

## Decisions

### 1.1 Entity Classification

**Decision: Fourth entity type.**

Extensions sit alongside tools, rules, and presets as a peer concept. A new `[extensions]` section in `config.toml` and a new `repo-extensions` crate.

**Rationale:** Presets are detection-oriented, tools are consumption-oriented. Extensions are provision-oriented - they provide content and services. Mixing them muddies existing clean abstractions.

**Rejected alternatives:**
- Preset superset: stretches preset concept beyond detection
- Source + provider split: two new concepts where one suffices

### 1.2 Extension Manifest Format

**Decision: TOML.**

Extension manifests are `repo_extension.toml` files in the extension's source repository. Consistent with all other repo manager configuration (config.toml, ledger.toml, tool definitions) and the `repo` prefix makes it immediately clear which system the file belongs to.

**Rejected alternatives:**
- JSON: inconsistent with ecosystem
- Dual support: unnecessary complexity

### 1.3 Distribution Model

**Decision: Git repos + local paths.**

Extensions are distributed as git repositories, pinned to tags or revisions. Local paths (`source.type = "local"`) supported for development and testing. A registry mapping short names to URLs can be added later without breaking changes.

**Rejected alternatives:**
- Git repos only: no development workflow
- Git + local + registry: premature for initial implementation

### 1.4 Extension Output Targets

**Decision: Extension-declared output map.**

The extension manifest declares an `[outputs]` section mapping content to target paths. The repo manager resolves these paths based on layout mode (worktree vs standard).

```toml
[outputs]
claude_dir = ".claude/"
gemini_dir = ".gemini/"
agent_dir = ".agent/"
```

The extension declares WHERE it wants to write. The repo manager resolves HOW based on layout.

**Open question:** VaultSpec's root discovery behavior and `.vaultspec` folder placement in worktree layouts requires further investigation. VaultSpec's `--root` flag and `VAULTSPEC_*` env vars provide override mechanisms. In worktree mode, the repo manager passes the container root.

### 1.5 Layout-Aware Path Resolution

**Decision: Deferred pending VaultSpec investigation.**

All tool configs currently go at the container root (worktree mode) or repo root (standard mode). VaultSpec's `--root` flag allows the repo manager to pass the correct root. This works for the immediate case but needs deeper design if extensions require per-worktree output.

## Consequences

- New `repo-extensions` crate required
- `Manifest` struct gains `extensions: HashMap<String, Value>` field
- New `ExtensionSyncer` alongside `ToolSyncer`/`RuleSyncer`
- Extension-provided content tracked in ledger with `"ext:{name}"` intent IDs
- `.repository/extensions/{name}/` directory for extension state
