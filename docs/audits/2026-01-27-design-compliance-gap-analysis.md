# Design Compliance Gap Analysis

**Date:** 2026-01-27
**Analyst:** Claude Opus 4.5
**Scope:** Full codebase vs. design documentation

---

## Executive Summary

The Repository Manager implementation is **75% compliant** with design specifications. Core infrastructure (Layer 0 crates) is complete and robust. The orchestration layer (repo-core) and CLI (repo-cli) are implemented but have **incomplete functionality** in sync/fix operations. Major gaps exist in preset providers, tool integrations, and the MCP server (not started).

| Area | Compliance | Notes |
|------|------------|-------|
| Layer 0 Crates | 95% | Minor gaps in repo-presets, repo-tools |
| Orchestration (repo-core) | 70% | Structure complete, sync/fix partial |
| CLI (repo-cli) | 80% | Commands exist, some are stubs |
| MCP Server | 0% | Not started |

---

## 1. Architecture Compliance

### 1.1 Crate Structure

**Spec:** `docs/design/architecture-core.md`

| Specified Crate | Status | Notes |
|-----------------|--------|-------|
| repo-fs | ✅ Complete | NormalizedPath, atomic I/O, layouts |
| repo-git | ✅ Complete | Worktree operations, container layout |
| repo-content | ✅ Complete | Document, Format handlers, blocks |
| repo-blocks | ✅ Complete | Block parsing and writing |
| repo-meta | ✅ Complete | Config schema, registry |
| repo-presets | 🔶 Partial | Only UvProvider implemented |
| repo-tools | 🔶 Partial | VSCode, Cursor, Claude only |
| repo-core | 🔶 Partial | Structure done, sync/fix incomplete |
| repo-cli | 🔶 Partial | Commands exist, some incomplete |
| repo-mcp | ❌ Missing | Not started |

### 1.2 Mode Support

**Spec:** Two modes - Standard and Worktrees

| Feature | Standard Mode | Worktrees Mode |
|---------|---------------|----------------|
| Init | ✅ Implemented | ✅ Implemented |
| Branch create | ✅ Implemented | ✅ Implemented |
| Branch delete | ✅ Implemented | ✅ Implemented |
| Branch list | ✅ Implemented | ✅ Implemented |
| Config location | ✅ `.repository/` | ✅ Container `.repository/` |
| Default mode | N/A | ✅ Correct (worktrees) |

---

## 2. Ledger System Compliance

**Spec:** `docs/design/config-ledger.md`

### 2.1 Schema Compliance

| Spec Element | Implementation | Status |
|--------------|----------------|--------|
| `[meta]` section | `LedgerMeta` struct | ✅ Match |
| `[[intents]]` array | `Vec<Intent>` | ✅ Match |
| Intent.id | `String` | ✅ Match |
| Intent.uuid | `Uuid` | ✅ Match |
| Intent.timestamp | `DateTime<Utc>` | ✅ Match |
| Intent.args | `serde_json::Value` | ✅ Match |
| Intent.projections | `Vec<Projection>` | ✅ Match |
| Projection.tool | `String` | ✅ Match |
| Projection.file | `PathBuf` | ✅ Match |
| Projection.kind | `ProjectionKind` enum | ✅ Match |
| ProjectionKind::TextBlock | marker + checksum | ✅ Match |
| ProjectionKind::JsonKey | path + value | ✅ Match |
| ProjectionKind::FileManaged | checksum | ✅ Match |

### 2.2 Ledger Operations

| Operation | Spec | Implementation | Status |
|-----------|------|----------------|--------|
| Load from TOML | Required | `Ledger::load()` | ✅ |
| Save to TOML | Required | `Ledger::save()` | ✅ |
| Add intent | Required | `Ledger::add_intent()` | ✅ |
| Remove intent | Required | `Ledger::remove_intent()` | ✅ |
| Query by UUID | Required | `Ledger::get_intent()` | ✅ |
| Query by ID | Required | `Ledger::get_intents_by_id()` | ✅ |
| Query by file | Required | `Ledger::projections_for_file()` | ✅ |
| Query by tool | Required | `Ledger::projections_for_tool()` | ✅ |

---

## 3. CLI Compliance

**Spec:** `docs/design/spec-cli.md`

### 3.1 Commands

| Spec Command | Implementation | Status | Notes |
|--------------|----------------|--------|-------|
| `repo init` | `Commands::Init` | ✅ | All flags implemented |
| `repo check` | `Commands::Check` | ✅ | Working |
| `repo fix` | `Commands::Fix` | 🔶 | Stub - calls sync |
| `repo sync` | `Commands::Sync` | 🔶 | Partial - creates ledger only |
| `repo add-tool` | `Commands::AddTool` | 🔶 | Adds to config, no sync |
| `repo remove-tool` | `Commands::RemoveTool` | 🔶 | Removes from config, no cleanup |
| `repo add-preset` | `Commands::AddPreset` | 🔶 | Adds to config, no apply |
| `repo remove-preset` | `Commands::RemovePreset` | 🔶 | Removes from config, no cleanup |
| `repo branch add` | `BranchAction::Add` | ✅ | Working |
| `repo branch remove` | `BranchAction::Remove` | ✅ | Working |
| `repo branch list` | `BranchAction::List` | ✅ | Working |
| `repo push` | Not implemented | ❌ | Missing |
| `repo pull` | Not implemented | ❌ | Missing |
| `repo merge` | Not implemented | ❌ | Missing |

### 3.2 CLI Flags

| Flag | Spec | Implementation | Status |
|------|------|----------------|--------|
| `--verbose` / `-v` | Global | ✅ Implemented | Working |
| `--mode` | init | ✅ Implemented | Default = worktrees |
| `--tools` | init | ✅ Implemented | Multiple values |
| `--presets` | init | ✅ Implemented | Multiple values |
| `--dry-run` | sync/fix | ✅ Implemented | Working |
| `--base` | branch add | ✅ Implemented | Default = main |

---

## 4. Tools Subsystem Compliance

**Spec:** `docs/design/spec-tools.md`

### 4.1 ToolIntegration Trait

| Spec Method | Implementation | Status |
|-------------|----------------|--------|
| `name()` | Via `ToolId` enum | ✅ |
| `config_locations()` | `config_files()` method | ✅ |
| `sync()` | Not fully implemented | 🔶 |

### 4.2 Supported Tools

| Spec Tool | Implementation | Status |
|-----------|----------------|--------|
| VSCode | `VscodeIntegration` | ✅ Complete |
| Cursor | `CursorIntegration` | ✅ Complete |
| Claude Desktop/CLI | `ClaudeIntegration` | ✅ Complete |
| Antigravity | Not implemented | ❌ Missing |
| Windsurf | Not implemented | ❌ Missing |
| Gemini CLI | Not implemented | ❌ Missing |
| JetBrains | Not implemented | ❌ Missing |

---

## 5. Presets Subsystem Compliance

**Spec:** `docs/design/spec-presets.md`

### 5.1 PresetProvider Trait

| Spec Method | Implementation | Status |
|-------------|----------------|--------|
| `id()` | Implemented | ✅ |
| `check()` | Implemented | ✅ |
| `apply()` | Implemented | ✅ |

### 5.2 Built-in Providers

| Spec Provider | Implementation | Status |
|---------------|----------------|--------|
| `env:python` (UV) | `UvProvider` | ✅ Complete |
| `env:python` (venv) | Not implemented | ❌ Missing |
| `env:python` (conda) | Not implemented | ❌ Missing |
| `env:node` | Not implemented | ❌ Missing |
| `env:rust` | Not implemented | ❌ Missing |
| `config:editorconfig` | Not implemented | ❌ Missing |
| `config:gitignore` | Not implemented | ❌ Missing |
| `tool:ruff` | Not implemented | ❌ Missing |
| `tool:pytest` | Not implemented | ❌ Missing |

---

## 6. MCP Server Compliance

**Spec:** `docs/design/spec-mcp-server.md`

| Spec Component | Implementation | Status |
|----------------|----------------|--------|
| Crate structure | Not created | ❌ |
| Repository Lifecycle tools | Not implemented | ❌ |
| Branch Management tools | Not implemented | ❌ |
| Git Primitive tools | Not implemented | ❌ |
| Configuration tools | Not implemented | ❌ |
| Resources | Not implemented | ❌ |

**Gap:** The entire `repo-mcp` crate is missing. This is a **major gap** as it's the primary interface for agentic tools.

---

## 7. Critical Gaps Summary

### 7.1 Blocking Issues (Must Fix)

1. **Sync/Fix Operations Incomplete**
   - `sync()` only creates ledger, doesn't apply configurations
   - `fix()` is just a stub calling `sync()`
   - **Impact:** Core value proposition not delivered

2. **MCP Server Missing**
   - Entire crate not started
   - **Impact:** No agentic tool integration

3. **Preset Providers Missing**
   - Only UV provider exists
   - **Impact:** Limited language support

### 7.2 Important Gaps (Should Fix)

1. **Git Wrapper Commands Missing**
   - `repo push`, `repo pull`, `repo merge` not implemented
   - Workaround: Users can use git directly

2. **Tool Integrations Incomplete**
   - Antigravity, Windsurf, Gemini, JetBrains missing
   - Workaround: Use generic file handling

3. **Tool Add/Remove Don't Trigger Sync**
   - Config updated but files not generated
   - Workaround: Manual `repo sync`

### 7.3 Minor Gaps (Nice to Have)

1. **No validation warnings for unknown tools/presets**
2. **No migration between modes**
3. **No rollback system**

---

## 8. Compliance Score by Spec Document

| Document | Compliance |
|----------|------------|
| architecture-core.md | 80% |
| config-ledger.md | 95% |
| config-schema.md | 90% |
| config-strategy.md | 85% |
| spec-cli.md | 70% |
| spec-fs.md | 95% |
| spec-git.md | 90% |
| spec-metadata.md | 90% |
| spec-presets.md | 30% |
| spec-tools.md | 50% |
| spec-mcp-server.md | 0% |

---

## 9. Recommendations

### Immediate Priority (Phase A)
1. Fix symlink vulnerability in repo-fs
2. Add error injection tests
3. Add tool/preset validation warnings

### High Priority (Phase B)
1. Complete sync/fix implementation in repo-core
2. Wire tool add/remove to trigger sync
3. Implement rule add/modify/remove commands

### Medium Priority (Phase C)
1. Add Conda, Node, Rust preset providers
2. Add Windsurf, JetBrains tool integrations

### Lower Priority (Phase D)
1. Create repo-mcp crate
2. Implement MCP server tools and resources

### Future (Phase E)
1. Git wrapper commands (push, pull, merge)
2. Migration between modes
3. Rollback system
