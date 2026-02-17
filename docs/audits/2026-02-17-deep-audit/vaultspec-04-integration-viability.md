# Vaultspec Integration Viability Report

**Date**: 2026-02-17
**Author**: ViabilitySpecialist
**Scope**: Concrete integration plan for making vaultspec an optional plugin powering `repo agent` in repository-manager
**Inputs**: Audit reports 01-03 (vaultspec), audit reports 00/02/05 (repo-manager), roadmap, source code review

---

## A. Viability Assessment

### Overall Viability Score: 8/10

**Justification**: The integration is highly viable. Vaultspec provides a mature, production-ready orchestration layer (9/10 core systems, protocol layer assessed as "HIGHLY VIABLE") with clean CLI boundaries and a well-structured MCP server. Repository-manager has a clean layered architecture (3.8/5 maturity) with a well-defined extension point in its CLI and MCP server. The primary deductions are:

- **-1**: Cross-language boundary (Rust/Python) introduces deployment complexity and a Python 3.13 runtime dependency
- **-1**: Vaultspec's CLI lacks a stable, versioned JSON output contract; several commands output human-readable text only

### Technical Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Python 3.13 runtime dependency on end-user machines | High | Graceful degradation: `repo agent` commands print "Install Python 3.13+ and vaultspec for agent features" when not found |
| Vaultspec CLI output format instability | Medium | Define a JSON output contract in vaultspec; version it; repo-manager parses only stable fields |
| Process management complexity on Windows | Medium | Use `std::process::Command` with proper signal handling; test on Windows CI |
| Startup latency for Python subprocess | Low | Keep subprocess alive for MCP server mode; accept ~500ms cold start for one-shot commands |
| Version compatibility drift between projects | Medium | Semver-based compatibility check at `repo agent --check`; pin minimum vaultspec version |

### Effort Estimate: 6-8 person-weeks

| Component | Effort |
|-----------|--------|
| Plugin discovery & health check | 0.5 weeks |
| `repo-agent` crate scaffolding + CLI surface | 1 week |
| Subprocess wrapper (spawn/capture/parse) | 1.5 weeks |
| MCP bridge (connect to vaultspec MCP server) | 1.5 weeks |
| Config schema additions (`.repository/config.toml`) | 0.5 weeks |
| Testing (unit + integration + E2E) | 1.5 weeks |
| Vaultspec-side JSON output stabilization | 1 week |

---

## B. Architecture Decision: How to Wrap

### Option 1: Subprocess Wrapping

`repo` invokes vaultspec CLI scripts (`cli.py`, `subagent.py`) as child processes, captures stdout/stderr, and parses JSON output.

**Pros**:
- Simplest implementation; proven pattern (git itself works this way)
- Complete isolation; vaultspec can be upgraded independently
- No shared memory or FFI complexity
- Works on all platforms where Python is available

**Cons**:
- ~500ms cold start per invocation (Python interpreter startup)
- Output parsing is fragile if vaultspec changes format
- No shared state between invocations (each call is independent)
- Error reporting requires parsing stderr

**Complexity**: Low
**Maintenance Burden**: Low (changes in vaultspec only affect repo-manager if the CLI contract changes)

### Option 2: MCP Bridge

`repo` connects to vaultspec's MCP server (`vs-subagent-mcp`) as a client over stdio, using the 5 well-defined MCP tools (`list_agents`, `dispatch_agent`, `get_task_status`, `cancel_task`, `get_locks`).

**Pros**:
- Structured, typed interface (JSON-RPC 2.0 with defined schemas)
- Persistent connection eliminates repeated startup cost
- Built-in task lifecycle management, advisory locking, TTL cleanup
- Dynamic agent discovery via MCP resources
- Vaultspec's MCP server is production-tested with comprehensive test coverage

**Cons**:
- Requires spawning and managing a long-running MCP server process
- MCP client implementation needed in Rust (or reuse existing `repo-mcp` patterns)
- Server process consumes memory even when idle
- More complex error handling (connection drops, server crashes)

**Complexity**: Medium
**Maintenance Burden**: Medium (MCP protocol is stable; vaultspec server changes are internal)

### Option 3: Hybrid (RECOMMENDED)

Subprocess wrapping for lifecycle/config commands; MCP bridge for structured orchestration queries.

```
repo agent list        -> subprocess: python subagent.py list
repo agent sync        -> subprocess: python cli.py sync-all
repo agent rules list  -> subprocess: python cli.py rules list

repo agent spawn       -> MCP: dispatch_agent
repo agent status      -> MCP: get_task_status
repo agent stop        -> MCP: cancel_task
repo agent locks       -> MCP: get_locks
```

**Pros**:
- Best of both approaches: simple subprocess for simple commands, structured MCP for orchestration
- MCP server only started when agent orchestration is needed (lazy spawn)
- One-shot commands (list, sync) avoid MCP server overhead
- Orchestration commands get full task lifecycle, advisory locks, and streaming

**Cons**:
- Two integration paths to maintain
- Slightly more code than either approach alone

**Complexity**: Medium
**Maintenance Burden**: Medium

### Option 4: PyO3 Embedding

Embed the Python runtime inside the Rust binary using PyO3, calling vaultspec's Python APIs directly.

**Pros**:
- Direct API access, no serialization overhead
- Shared memory, fastest possible integration
- Single binary distribution (if Python is bundled)

**Cons**:
- Dramatically increases build complexity (linking Python, managing GIL)
- Binary size bloat (embeds Python runtime)
- Platform-specific build issues (different Python installations)
- Debugging cross-language issues is extremely difficult
- Vaultspec uses async Python (`asyncio`), which is hard to bridge from Rust
- Python 3.13 requirement constrains PyO3 version compatibility

**Complexity**: Very High
**Maintenance Burden**: Very High

### Recommendation: Option 3 (Hybrid)

The hybrid approach provides the best balance of simplicity and capability. Subprocess wrapping handles the simple CRUD operations where cold start latency is acceptable (users expect `list` commands to take a moment). MCP bridging handles the orchestration commands where persistent state, task lifecycle, and streaming are essential.

---

## C. Concrete Command Mapping

### Subprocess-Wrapped Commands

```
repo agent list         -> python .vaultspec/lib/scripts/subagent.py --root . list
repo agent rules list   -> python .vaultspec/lib/scripts/cli.py rules list
repo agent rules sync   -> python .vaultspec/lib/scripts/cli.py rules sync
repo agent agents list  -> python .vaultspec/lib/scripts/cli.py agents list
repo agent agents add   -> python .vaultspec/lib/scripts/cli.py agents add --name <name> --tier <tier>
repo agent skills list  -> python .vaultspec/lib/scripts/cli.py skills list
repo agent sync         -> python .vaultspec/lib/scripts/cli.py sync-all
repo agent audit        -> python .vaultspec/lib/scripts/docs.py audit --summary --json
repo agent config show  -> python .vaultspec/lib/scripts/cli.py config show
repo agent run <name>   -> python .vaultspec/lib/scripts/subagent.py --root . run --agent <name> --goal "<task>"
```

### MCP-Bridged Commands (via vs-subagent-mcp)

```
repo agent spawn <name> --goal "..." -> MCP tool: dispatch_agent(agent=<name>, task="...")
repo agent status [task-id]          -> MCP tool: get_task_status(task_id=<id>)
repo agent stop <task-id>            -> MCP tool: cancel_task(task_id=<id>)
repo agent locks                     -> MCP tool: get_locks()
repo agent agents discover           -> MCP resource: list agents://{name}
```

### Direct (No Vaultspec) Commands

```
repo agent --check    -> Check vaultspec installation, version, Python version
repo agent config     -> Show/set agent configuration in .repository/config.toml
repo agent init       -> Bootstrap .vaultspec/ directory with defaults
```

---

## D. Changes Required in Each Project

### In Repository-Manager

#### New Crate: `repo-agent`

A new crate at `crates/repo-agent/` that encapsulates all vaultspec integration logic.

**Modules**:

| Module | Purpose |
|--------|---------|
| `lib.rs` | Public API: `AgentManager` struct |
| `discovery.rs` | Find vaultspec installation (`PATH`, configured location, `.vaultspec/` detection) |
| `subprocess.rs` | Spawn vaultspec CLI commands, capture output, parse JSON |
| `mcp_client.rs` | Connect to vaultspec MCP server, call tools, read resources |
| `config.rs` | Agent-related configuration schema |
| `error.rs` | Error types for agent operations |
| `types.rs` | Shared types (AgentInfo, TaskStatus, etc.) |

**Key type: `AgentManager`**:
```rust
pub struct AgentManager {
    root: NormalizedPath,
    vaultspec_path: Option<PathBuf>,  // path to .vaultspec/
    python_path: Option<PathBuf>,     // path to python3.13+
    mcp_process: Option<Child>,       // lazy-started MCP server
}

impl AgentManager {
    pub fn discover(root: &NormalizedPath) -> Result<Self>;
    pub fn is_available(&self) -> bool;
    pub fn health_check(&self) -> Result<HealthReport>;

    // Subprocess commands
    pub fn list_agents(&self) -> Result<Vec<AgentInfo>>;
    pub fn run_agent(&self, name: &str, goal: &str, opts: RunOptions) -> Result<RunResult>;
    pub fn sync_all(&self, dry_run: bool) -> Result<SyncReport>;

    // MCP commands (lazy-start server)
    pub async fn dispatch(&mut self, agent: &str, task: &str, opts: DispatchOptions) -> Result<TaskId>;
    pub async fn task_status(&self, task_id: &str) -> Result<TaskInfo>;
    pub async fn cancel_task(&self, task_id: &str) -> Result<()>;
    pub async fn get_locks(&self) -> Result<Vec<LockInfo>>;
}
```

#### CLI Additions (`repo-cli/src/cli.rs`)

Add to the `Commands` enum:

```rust
/// Manage AI agents (requires vaultspec)
Agent {
    #[command(subcommand)]
    action: AgentAction,
},
```

With `AgentAction` subcommands for `list`, `run`, `spawn`, `status`, `stop`, `sync`, `rules`, `config`, `--check`.

#### Config Schema Additions (`.repository/config.toml`)

```toml
[agent]
# Enable agent orchestration (requires vaultspec)
enabled = true

# Path to Python interpreter (auto-detected if not set)
# python_path = "/usr/bin/python3.13"

# Path to vaultspec framework directory
# framework_dir = ".vaultspec"

# Default provider for new agents
default_provider = "claude"

# Default permission mode
default_mode = "read-write"
```

#### MCP Server Extensions (`repo-mcp`)

Add agent-related MCP tools that proxy to vaultspec:

| Tool | Description |
|------|-------------|
| `agent_list` | List available agents |
| `agent_dispatch` | Dispatch an agent task |
| `agent_status` | Check task status |
| `agent_cancel` | Cancel a running task |

These would be registered only when vaultspec is detected.

#### Test Strategy

| Test Layer | Scope | Approach |
|------------|-------|----------|
| Unit tests | `repo-agent` crate | Mock subprocess output, test JSON parsing, test MCP message construction |
| Integration tests | Full `repo agent` commands | Use a test fixture with a minimal `.vaultspec/` directory, mock Python responses |
| E2E tests | Real vaultspec invocation | CI job with Python 3.13 + vaultspec installed, run actual agent list/sync commands |
| Negative tests | Missing vaultspec | Verify graceful degradation when Python/vaultspec not installed |

### In Vaultspec

#### CLI Contract Stabilization

**Priority: HIGH** - Without this, the integration is fragile.

1. **Add `--json` flag to ALL CLI commands** (currently only `docs.py audit` supports JSON output):
   - `cli.py rules list --json`
   - `cli.py agents list --json`
   - `cli.py skills list --json`
   - `cli.py sync-all --json`
   - `subagent.py list --json`

2. **Define a JSON output schema** for each command:
   ```json
   {
     "version": "1.0",
     "command": "agents.list",
     "data": [...],
     "errors": []
   }
   ```

3. **Versioned output contract**: Include a `version` field so repo-manager can detect breaking changes.

#### Missing Features for Integration

| Feature | Priority | Description |
|---------|----------|-------------|
| JSON output for all CLI commands | HIGH | Required for reliable subprocess parsing |
| Non-interactive mode for `agents add` | HIGH | Currently opens editor; needs `--no-edit` or stdin pipe mode |
| Health check endpoint | MEDIUM | `subagent.py health` that reports Python version, dependencies, GPU status |
| Exit codes standardization | MEDIUM | Document exit codes (0=success, 1=error, 2=not found, etc.) |
| `--quiet` flag for all commands | LOW | Suppress non-essential output for scripting |

#### Configuration Bridge Requirements

When running under `repo agent`, vaultspec should read additional configuration from `.repository/config.toml`:

1. Override `VAULTSPEC_ROOT_DIR` from repo-manager's detected root
2. Override `VAULTSPEC_FRAMEWORK_DIR` from `[agent] framework_dir`
3. Override `VAULTSPEC_DOCS_DIR` from `.repository/` if configured

This can be achieved by having repo-manager set environment variables before spawning vaultspec subprocess.

#### Output Format Standardization

All CLI commands should support a consistent JSON envelope:

```json
{
  "ok": true,
  "version": "1.0.0",
  "data": { ... },
  "warnings": [],
  "errors": []
}
```

Error responses:

```json
{
  "ok": false,
  "version": "1.0.0",
  "data": null,
  "errors": [{"code": "AGENT_NOT_FOUND", "message": "Agent 'foo' not found"}]
}
```

---

## E. Integration Phases (Detailed Implementation Plan)

### Phase 2A: Plugin Discovery and Health Check (0.5 weeks)

**Deliverable**: `repo agent --check` works, reports vaultspec status.

**Steps**:
1. Create `crates/repo-agent/` with `Cargo.toml` depending on `repo-fs`
2. Implement `discovery.rs`: search for Python 3.13+, search for `.vaultspec/lib/scripts/`, validate version
3. Implement `repo agent --check` command that reports:
   - Python found/version
   - Vaultspec found/version
   - Framework directory exists
   - Agent count
   - MCP server available
4. Implement graceful error message when vaultspec not found

**Dependencies**: None
**Testable**: `repo agent --check` returns structured report
**Shippable**: Yes (standalone utility)

### Phase 2B: Subprocess Wrapper - Read Commands (1 week)

**Deliverable**: `repo agent list`, `repo agent rules list`, `repo agent config show` work.

**Steps**:
1. Implement `subprocess.rs`: generic `run_vaultspec(script, args, json_mode) -> Result<Value>`
2. Add `--json` flag to vaultspec's `cli.py agents list`, `rules list`, `skills list` (vaultspec change)
3. Implement `repo agent list` -> parse JSON agent list, display in repo-manager format
4. Implement `repo agent rules list` -> parse JSON rules, display
5. Implement `repo agent config show` -> capture text output, display
6. Add config schema: `[agent]` section in `.repository/config.toml`

**Dependencies**: Phase 2A
**Testable**: All list commands return expected output
**Shippable**: Yes

### Phase 2C: Subprocess Wrapper - Write Commands (1 week)

**Deliverable**: `repo agent sync`, `repo agent run`, `repo agent agents add` work.

**Steps**:
1. Implement `repo agent sync` -> invokes `cli.py sync-all`, reports results
2. Implement `repo agent run <name> --goal "..."` -> invokes `subagent.py run`, streams output
3. Implement `repo agent agents add` -> invokes `cli.py agents add` with `--no-edit` mode
4. Add `--json` and `--no-edit` flags to vaultspec write commands (vaultspec change)
5. Handle long-running processes (streaming stdout for `run` command)

**Dependencies**: Phase 2B
**Testable**: Sync modifies files; run produces agent output
**Shippable**: Yes (basic agent orchestration works)

### Phase 2D: MCP Bridge - Server Management (1.5 weeks)

**Deliverable**: `repo agent spawn`, `repo agent status`, `repo agent stop` work via MCP.

**Steps**:
1. Implement `mcp_client.rs`: spawn `subagent.py serve --root .` as background process
2. Implement MCP client using JSON-RPC over stdio (reuse patterns from `repo-mcp`)
3. Implement lazy server startup: first `spawn` command starts the server
4. Implement `dispatch_agent` tool call -> returns task ID
5. Implement `get_task_status` tool call -> returns status JSON
6. Implement `cancel_task` tool call -> cancels running task
7. Implement server lifecycle: start, health check, restart on crash, graceful shutdown
8. Handle `repo agent locks` via `get_locks` tool

**Dependencies**: Phase 2B (for discovery), Phase 2C (for run validation)
**Testable**: Spawn/status/stop lifecycle works E2E
**Shippable**: Yes (full orchestration capability)

### Phase 2E: MCP Server Extensions (1 week)

**Deliverable**: `repo-mcp` exposes agent tools to AI assistants.

**Steps**:
1. Add conditional agent tools to `repo-mcp` (only when vaultspec detected)
2. Register `agent_list`, `agent_dispatch`, `agent_status`, `agent_cancel` tools
3. These proxy to vaultspec's MCP server via the `AgentManager`
4. Add agent resources: `agent://{name}` for agent metadata
5. Test with Claude Desktop and Cursor

**Dependencies**: Phase 2D
**Testable**: MCP tools callable from AI assistants
**Shippable**: Yes

### Phase 2F: Polish and Documentation (0.5 weeks)

**Deliverable**: Complete `repo agent` documentation and error handling.

**Steps**:
1. Add `repo agent` to CLI help and completions
2. Write user documentation for agent setup and usage
3. Improve error messages for common failure modes
4. Add `repo agent init` to bootstrap `.vaultspec/` from a template
5. Update roadmap to reflect completed integration

**Dependencies**: Phase 2E
**Testable**: Documentation review, UX testing
**Shippable**: Yes

---

## F. Risk Register

### Risk 1: Python 3.13 Availability

**Probability**: Medium (30%)
**Impact**: High (blocks all agent features for affected users)
**Description**: End users may not have Python 3.13+ installed. Vaultspec uses `StrEnum`, `|` type unions, and other 3.13 features.
**Mitigation**:
- Graceful degradation: `repo agent` prints clear installation instructions
- Consider supporting Python 3.12 by removing 3.13-specific syntax (vaultspec change)
- Long-term: bundle a minimal Python runtime or consider Nuitka compilation of vaultspec

### Risk 2: Subprocess Output Parsing Fragility

**Probability**: Medium (40%)
**Impact**: Medium (commands fail silently or show garbled output)
**Description**: Vaultspec CLI outputs are currently human-readable, not machine-parseable. Changes to output format break the integration.
**Mitigation**:
- Phase 2B adds `--json` flag to all vaultspec commands
- Version the JSON contract; repo-manager checks `version` field
- Integration tests verify JSON parsing against actual vaultspec output
- Pin minimum vaultspec version in repo-manager config

### Risk 3: MCP Server Process Management on Windows

**Probability**: Medium (35%)
**Impact**: Medium (agent orchestration unusable on Windows)
**Description**: Long-running subprocess management (spawn, signal, cleanup) behaves differently on Windows vs Unix. Vaultspec already has a Windows version enforcement for Gemini (`v0.9.0` minimum).
**Mitigation**:
- Use `std::process::Command` with proper Windows signal handling
- Test on Windows CI (the existing repo-manager already targets Windows)
- Implement a PID file + polling health check instead of relying on Unix signals
- Graceful fallback: subprocess-only mode if MCP server fails to start

### Risk 4: Two-Project Coordination Overhead

**Probability**: High (60%)
**Impact**: Low-Medium (feature delays, not breakage)
**Description**: Changes required in vaultspec (JSON output, non-interactive mode, health check) must be coordinated with repo-manager development. Both projects evolve independently.
**Mitigation**:
- Define the CLI contract as a shared document (interface specification)
- Vaultspec changes (Phase 2B) can be done in parallel with repo-manager scaffolding (Phase 2A)
- Use feature flags in repo-manager to ship incrementally
- Accept that initial release may use text parsing as a fallback before JSON is available

### Risk 5: GPU Dependency for RAG Features

**Probability**: Low (15% -- RAG is optional)
**Impact**: Low (only affects `repo agent search`)
**Description**: Vaultspec's RAG module requires CUDA GPU. Most developer machines have GPUs, but CI/CD and some laptops do not.
**Mitigation**:
- RAG features are explicitly optional (Tier 2 in the hybrid approach from audit 03)
- `repo agent search` checks for RAG availability and provides clear error
- Filesystem-based vault tools (`list_documents`, `get_document`) work without GPU
- Future: CPU fallback in vaultspec (recommended in audit 03)

---

## G. Cross-References

### Audit Reports

| Report | File | Key Findings Used |
|--------|------|-------------------|
| Vaultspec Core Systems | `vaultspec-01-core-systems.md` | CLI command reference, config override patterns, module dependency graph, viability score 9/10 |
| Vaultspec Protocol Layer | `vaultspec-02-protocol-layer.md` | MCP server 5 tools, A2A/ACP protocols, sandbox model, "HIGHLY VIABLE" verdict |
| Vaultspec Declarative Systems | `vaultspec-03-rules-agents-skills.md` | 9 agents, 12 skills, markdown governance model, RAG architecture, hybrid integration recommendation |
| Repo-Manager Audit Index | `00-audit-index.md` | Key metrics (1,078 tests, 13 tools, 3.8/5 maturity), critical finding: no agent orchestration |
| Repo-Manager Technical Audit | `02-technical-audit.md` | 10 crate architecture, CLI structure, MCP server implementation, stubbed features |
| Repo-Manager Feature Gaps | `05-feature-gaps-opportunities.md` | Agent orchestration as biggest gap, lightweight local orchestration as biggest opportunity |

### Roadmap

| Section | File | Relevance |
|---------|------|-----------|
| Phase 2: Vaultspec Integration | `2026-02-17-roadmap.md` | This report provides the detailed implementation plan for Phase 2 |

### Source Files Reviewed

**Vaultspec**:
- `Y:/code/task-worktrees/main/.vaultspec/lib/scripts/cli.py` -- Resource manager CLI, 1322 lines, all commands implemented
- `Y:/code/task-worktrees/main/.vaultspec/lib/scripts/subagent.py` -- Subagent launcher, 318 lines, run/serve/list/a2a-serve commands
- `Y:/code/task-worktrees/main/.vaultspec/lib/src/orchestration/task_engine.py` -- 5-state task lifecycle, thread-safe, TTL cleanup
- `Y:/code/task-worktrees/main/.vaultspec/lib/src/orchestration/subagent.py` -- Agent spawning via ACP, provider selection, process management
- `Y:/code/task-worktrees/main/.vaultspec/lib/src/protocol/a2a/server.py` -- A2A HTTP server (Starlette ASGI)
- `Y:/code/task-worktrees/main/.vaultspec/lib/src/protocol/acp/claude_bridge.py` -- ACP bridge wrapping claude-agent-sdk, 948 lines
- `Y:/code/task-worktrees/main/.vaultspec/lib/src/core/config.py` -- Centralized config with 30+ env vars, override support
- `Y:/code/task-worktrees/main/mcp.json` -- MCP server configuration (vs-subagent-mcp)

**Repository-Manager**:
- `Y:/code/repository-manager-worktrees/main/crates/repo-cli/src/cli.rs` -- Clap-derive CLI, 20+ commands, 679 lines
- `Y:/code/repository-manager-worktrees/main/crates/repo-cli/src/commands/mod.rs` -- Command module exports (11 modules)
- `Y:/code/repository-manager-worktrees/main/crates/repo-mcp/src/main.rs` -- MCP server entry point, JSON-RPC over stdio
- `Y:/code/repository-manager-worktrees/main/crates/repo-tools/src/integration.rs` -- ToolIntegration trait, SyncContext, ConfigLocation

---

## Summary

| Dimension | Assessment |
|-----------|-----------|
| **Overall Viability** | 8/10 -- Highly viable with minor deployment considerations |
| **Recommended Architecture** | Hybrid (subprocess + MCP bridge) |
| **Total Effort** | 6-8 person-weeks |
| **Biggest Risk** | Python 3.13 dependency on end-user machines |
| **Biggest Opportunity** | Immediate agent orchestration without rebuilding in Rust |
| **Critical Dependency** | Vaultspec JSON output stabilization |
| **First Shippable Milestone** | Phase 2B: `repo agent list` (1.5 weeks from start) |
| **Full Feature Milestone** | Phase 2E: MCP tools exposed to AI assistants (6 weeks from start) |

---

*Report generated: 2026-02-17*
*File: `Y:/code/repository-manager-worktrees/main/docs/audits/2026-02-17-deep-audit/vaultspec-04-integration-viability.md`*
