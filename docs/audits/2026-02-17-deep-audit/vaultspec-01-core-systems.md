# Vaultspec Core Systems Audit

**Date**: 2026-02-17
**Auditor**: Auditor1
**Scope**: Orchestration, Vault, Core Config, CLI, Graph, Metrics, Verification
**Repository**: `Y:/code/task-worktrees/main/.vaultspec`

---

## Executive Summary

Vaultspec is a **governed development framework for AI agents** implemented in Python 3.13. It provides structured documentation management (the "vault"), subagent orchestration, and CLI tools for managing rules/agents/skills across multiple AI platforms.

**Overall Assessment**: The core systems are **production-ready** with comprehensive implementation, good separation of concerns, and robust testing coverage.

### Key Strengths
- ✅ **Complete Implementation**: All core modules fully functional (no stubs)
- ✅ **Comprehensive Testing**: Unit tests for task engine, vault, graph, verification
- ✅ **Type Safety**: Full type hints with TYPE_CHECKING guards
- ✅ **Modular Design**: Clean separation between vault/orchestration/protocol layers
- ✅ **Configuration System**: Centralized, environment-aware config with validation

### Integration Viability for `repo agent`
**Recommendation**: **VIABLE** with minor considerations

The system is well-architected for integration as a Rust plugin:
- Clean CLI interface (`cli.py`, `docs.py`, `subagent.py`)
- Well-defined Python package structure
- No hard dependencies on specific runtime environments
- Configuration can be overridden programmatically

---

## Module Summary Table

| Module | Purpose | Status | Test Coverage | Public API |
|--------|---------|--------|---------------|------------|
| **orchestration/task_engine.py** | Task lifecycle manager | ✅ Complete | ✅ Excellent | `TaskEngine`, `LockManager` |
| **orchestration/subagent.py** | Subagent spawning/ACP client | ✅ Complete | ⚠️ Integration only | `run_subagent()` |
| **vault/parser.py** | Frontmatter YAML parser | ✅ Complete | ✅ Good | `parse_frontmatter()`, `parse_vault_metadata()` |
| **vault/scanner.py** | Vault filesystem scanner | ✅ Complete | ✅ Excellent | `scan_vault()`, `get_doc_type()` |
| **vault/hydration.py** | Template hydration | ✅ Complete | ⚠️ Basic | `hydrate_template()`, `get_template_path()` |
| **vault/links.py** | Wiki-link extraction | ✅ Complete | ✅ Good | `extract_wiki_links()` |
| **vault/models.py** | Data models + validation | ✅ Complete | ✅ Excellent | `DocumentMetadata`, `VaultConstants` |
| **core/config.py** | Centralized configuration | ✅ Complete | ✅ Good | `get_config()`, `VaultSpecConfig` |
| **graph/api.py** | Document graph analysis | ✅ Complete | ✅ Excellent | `VaultGraph` |
| **metrics/api.py** | Vault statistics | ✅ Complete | ✅ Good | `get_vault_metrics()` |
| **verification/api.py** | Vault compliance checks | ✅ Complete | ✅ Good | `verify_vault_structure()`, `get_malformed()` |
| **scripts/cli.py** | Resource manager CLI | ✅ Complete | ⚠️ Manual testing | Full CLI interface |
| **scripts/docs.py** | Vault audit CLI | ✅ Complete | ⚠️ Manual testing | `audit`, `create`, `index`, `search` |
| **scripts/subagent.py** | Subagent launcher CLI | ✅ Complete | ⚠️ Manual testing | `run`, `serve`, `a2a-serve`, `list` |

**Legend**:
- ✅ Complete: Fully implemented and working
- ⚠️ Partial: Some test coverage or basic implementation
- ❌ Stub: TODO or placeholder

---

## 1. Orchestration (`Y:/code/task-worktrees/main/.vaultspec/lib/src/orchestration/`)

### 1.1 `task_engine.py` - Task Lifecycle Manager

**Purpose**: Standalone 5-state task lifecycle manager for background subagent execution.

**State Machine**:
```
WORKING ──┬──> COMPLETED
          ├──> INPUT_REQUIRED ──┬──> WORKING (resume)
          │                     └──> CANCELLED
          ├──> FAILED
          └──> CANCELLED
```

**Public API**:
```python
class TaskEngine:
    def create_task(agent: str, *, model: str | None, mode: str, task_id: str | None) -> SubagentTask
    def get_task(task_id: str) -> SubagentTask | None
    def update_status(task_id: str, status: TaskStatus, status_message: str | None) -> SubagentTask
    def complete_task(task_id: str, result: dict) -> SubagentTask
    def fail_task(task_id: str, error: str) -> SubagentTask
    def cancel_task(task_id: str) -> SubagentTask
    def list_tasks() -> list[SubagentTask]
    async def wait_for_update(task_id: str, timeout: float | None) -> None

class LockManager:
    def acquire_lock(task_id: str, paths: set[str], mode: str) -> tuple[FileLock, list[str]]
    def release_lock(task_id: str) -> bool
    def check_conflicts(paths: set[str]) -> list[str]
    @staticmethod
    def validate_readonly_paths(paths: set[str]) -> list[str]
```

**Key Features**:
- Thread-safe with internal locking
- TTL-based task expiration (default 3600s)
- Advisory file locking for workspace coordination
- Async wait/notify for status changes
- Read-only mode validation (`.vault/` paths only)

**Implementation Status**: ✅ **Fully working**
- State transition validation
- TTL cleanup on get/create
- Automatic lock release on terminal states
- Event notification for async waiters

**Test Coverage**: ✅ **Excellent** (Y:/code/task-worktrees/main/.vaultspec/lib/src/orchestration/tests/test_task_engine.py)
- Lock acquisition/release
- Task creation/retrieval
- State transitions (complete/fail/cancel)
- TTL eviction
- Error cases (invalid transitions, duplicate IDs)

**Integration Notes**:
- Designed for MCP server context (see audit #2)
- Stateless (in-memory only, no persistence)
- Can be wrapped by Rust with Python FFI or spawned as subprocess

---

### 1.2 `subagent.py` - Subagent Orchestration

**Purpose**: Spawns and manages AI agent processes using the Agent Communication Protocol (ACP).

**Public API**:
```python
async def run_subagent(
    agent_name: str,
    root_dir: pathlib.Path,
    initial_task: str = "",
    context_files: list[pathlib.Path] | None = None,
    plan_file: pathlib.Path | None = None,
    model_override: str | None = None,
    provider_override: str | None = None,
    interactive: bool = False,
    mode: str = "read-write",
    resume_session_id: str | None = None,
    max_turns: int | None = None,
    budget: float | None = None,
    effort: str | None = None,
    output_format: str | None = None,
) -> SubagentResult

def load_agent(agent_name: str, root_dir: pathlib.Path, provider_name: str | None) -> tuple[dict[str, str], str]
def get_provider_for_model(model_name: str | None) -> AgentProvider
```

**Key Design Decisions**:
1. **Provider Selection**: Model name patterns (`claude-*`, `gemini-*`) route to appropriate provider
2. **Fallback Strategy**: Tries provider-specific agent definitions first, then canonical
3. **Task Context Building**: Assembles prompt from plan file + context files + goal
4. **Process Management**: Uses ACP `spawn_agent_process()` with stderr consumption
5. **Permission Modes**: `read-only` restricts file writes to `.vault/` directory

**Implementation Status**: ✅ **Fully working**
- ACP handshake (protocolVersion=1)
- Session creation with MCP server support
- Interactive loop for multi-turn conversations
- Automatic cleanup of temp files
- Garbage collection for callback cycles

**Dependencies**:
- `acp.spawn_agent_process` (Agent Communication Protocol)
- `protocol.providers.{claude,gemini}` (provider implementations)
- `protocol.acp.client.SubagentClient` (ACP client)

**Integration Notes**:
- Main entry point for `repo agent run <agent-name>`
- Returns `SubagentResult` with session_id, response_text, written_files
- Async interface requires event loop (handled by CLI)

---

### 1.3 `constants.py` - Shared Constants

**Purpose**: Shared orchestration constants.

**Contents**:
```python
READONLY_PERMISSION_PROMPT = """
PERMISSION MODE: READ-ONLY
You MUST only write files within the `.vault/` directory.
Do not modify any source code files.
"""
```

**Implementation Status**: ✅ **Complete** (minimal but sufficient)

---

### 1.4 `utils.py` - Security Utilities

**Purpose**: Workspace boundary enforcement.

**Public API**:
```python
def find_project_root() -> pathlib.Path
def safe_read_text(path: pathlib.Path, root_dir: pathlib.Path) -> str
```

**Key Features**:
- Git repository detection (walks up looking for `.git/`)
- Path traversal prevention (`resolve()` + `is_relative_to()`)
- Raises `SecurityError` on boundary violations

**Implementation Status**: ✅ **Fully working**

---

## 2. Vault (`Y:/code/task-worktrees/main/.vaultspec/lib/src/vault/`)

### 2.1 `models.py` - Data Models

**Purpose**: Rigid schema definitions for vault documents.

**Public API**:
```python
class DocType(StrEnum):
    ADR = "adr"
    EXEC = "exec"
    PLAN = "plan"
    REFERENCE = "reference"
    RESEARCH = "research"

    @property
    def tag(self) -> str  # Returns "#adr", "#plan", etc.
    @classmethod
    def from_tag(cls, tag: str) -> DocType | None

@dataclass
class DocumentMetadata:
    tags: list[str]
    date: str | None
    related: list[str]

    def validate(self) -> list[str]  # Returns validation errors

class VaultConstants:
    DOCS_DIR = ".vault"
    SUPPORTED_DIRECTORIES: ClassVar[set[str]]
    SUPPORTED_TAGS: ClassVar[set[str]]

    @classmethod
    def validate_vault_structure(cls, root_dir: Path) -> list[str]
    @classmethod
    def validate_filename(cls, filename: str, doc_type: DocType | None) -> list[str]
```

**Rigid Rules** (enforced by validation):
1. **Rule of Two**: Exactly 2 tags required (1 directory tag + 1 feature tag)
2. **Directory Tag**: Must be one of `#adr`, `#exec`, `#plan`, `#reference`, `#research`
3. **Feature Tag**: Must be kebab-case (`#editor-demo`, `#vault-spec`)
4. **Date Format**: `YYYY-MM-DD`
5. **Filename Pattern**: `yyyy-mm-dd-<feature>-<type>.md`
6. **Related Links**: Must be `[[wiki-links]]`

**Implementation Status**: ✅ **Fully working**
- Comprehensive validation with clear error messages
- Regex patterns for filename/tag validation
- Supports `.obsidian` directory exception

**Test Coverage**: ✅ **Excellent** (Y:/code/task-worktrees/main/.vaultspec/lib/src/vault/tests/test_types.py)

---

### 2.2 `parser.py` - Frontmatter Parser

**Purpose**: Extract YAML frontmatter from markdown files.

**Public API**:
```python
def parse_frontmatter(content: str) -> tuple[dict[str, Any], str]
def parse_vault_metadata(content: str) -> tuple[DocumentMetadata, str]
```

**Key Design Decisions**:
1. **PyYAML Optional**: Falls back to simple key-value parser
2. **Resilient Parsing**: Handles unquoted colons in values
3. **List Parsing**: Supports both inline `["#a", "#b"]` and bulleted lists

**Implementation Status**: ✅ **Fully working**
- Handles YAML errors gracefully
- Preserves colons in values (e.g., `description: A test: with colons`)

---

### 2.3 `scanner.py` - Filesystem Scanner

**Purpose**: Discover all markdown files in the vault.

**Public API**:
```python
def scan_vault(root_dir: pathlib.Path) -> Iterator[pathlib.Path]
def get_doc_type(path: pathlib.Path, root_dir: pathlib.Path) -> DocType | None
```

**Key Features**:
- Recursive `.vault/**/*.md` scan
- Skips `.obsidian` directories
- Determines DocType from parent directory

**Implementation Status**: ✅ **Fully working**

**Test Coverage**: ✅ **Excellent** (Y:/code/task-worktrees/main/.vaultspec/lib/src/vault/tests/test_scanner.py)
- Verifies >80 docs found in test project
- Confirms `.obsidian` exclusion
- Tests all DocType detection

---

### 2.4 `hydration.py` - Template Hydration

**Purpose**: Replace placeholders in document templates.

**Public API**:
```python
def hydrate_template(template_content: str, feature: str, date: str, title: str | None) -> str
def get_template_path(root_dir: pathlib.Path, doc_type: DocType) -> pathlib.Path | None
```

**Supported Placeholders**:
- `<feature>` → feature name
- `<yyyy-mm-dd>` → date
- `<title>` → optional title

**Template Mapping**:
- ADR → `adr.md`
- PLAN → `plan.md`
- RESEARCH → `research.md`
- REFERENCE → `ref-audit.md`
- EXEC → `exec-step.md`

**Implementation Status**: ✅ **Fully working** (basic string replacement)

**Note**: Could be enhanced with Jinja2 for more complex templates

---

### 2.5 `links.py` - Link Extraction

**Purpose**: Extract wiki-links from markdown content.

**Public API**:
```python
def extract_wiki_links(content: str) -> set[str]
def extract_related_links(related: list[str]) -> set[str]
```

**Supported Formats**:
- `[[Link Name]]`
- `[[Link Name|Display Name]]`

**Implementation Status**: ✅ **Fully working**

**Test Coverage**: ✅ **Good** (Y:/code/task-worktrees/main/.vaultspec/lib/src/vault/tests/test_links.py)

---

## 3. Core Config (`Y:/code/task-worktrees/main/.vaultspec/lib/src/core/config.py`)

**Purpose**: Centralized configuration system with environment variable support.

**Public API**:
```python
@dataclass
class VaultSpecConfig:
    # Agent settings
    root_dir: Path
    agent_mode: str  # "read-write" | "read-only"
    system_prompt: str | None
    max_turns: int | None
    budget_usd: float | None
    allowed_tools: list[str]
    disallowed_tools: list[str]

    # MCP settings
    mcp_root_dir: Path | None
    mcp_port: int
    mcp_host: str
    mcp_ttl_seconds: float

    # Storage paths
    docs_dir: str  # ".vault"
    framework_dir: str  # ".vaultspec"
    lance_dir: str  # ".lance"

    # Tool directories
    claude_dir: str  # ".claude"
    gemini_dir: str  # ".gemini"
    antigravity_dir: str  # ".antigravity"

    # RAG settings
    graph_ttl_seconds: float
    embedding_batch_size: int
    max_embed_chars: int
    embedding_model: str

    @classmethod
    def from_environment(cls, overrides: dict[str, Any] | None) -> VaultSpecConfig

def get_config(overrides: dict[str, Any] | None = None) -> VaultSpecConfig
def reset_config() -> None
```

**Configuration Resolution Order**:
1. Explicit `overrides` dict (for testing/DI)
2. `VAULTSPEC_*` environment variables
3. Dataclass defaults

**Key Features**:
- Centralized registry (`CONFIG_REGISTRY`)
- Type-safe parsing (int, float, Path, CSV lists)
- Validation (range checks, allowed options)
- Module-level singleton (`get_config()`)

**Environment Variables** (46 total):
```bash
VAULTSPEC_ROOT_DIR
VAULTSPEC_AGENT_MODE
VAULTSPEC_MAX_TURNS
VAULTSPEC_BUDGET_USD
VAULTSPEC_MCP_PORT
VAULTSPEC_DOCS_DIR
VAULTSPEC_FRAMEWORK_DIR
# ... and 39 more
```

**Implementation Status**: ✅ **Fully working**
- No third-party dependencies (stdlib only)
- Comprehensive error handling
- Reset mechanism for testing

**Test Coverage**: ✅ **Good** (Y:/code/task-worktrees/main/.vaultspec/lib/src/core/tests/test_config.py)

**Integration Notes**:
- **Critical for Rust plugin**: Override `root_dir`, `docs_dir`, `framework_dir` via `overrides` dict
- No need to set environment variables if using programmatic config

---

## 4. Graph (`Y:/code/task-worktrees/main/.vaultspec/lib/src/graph/api.py`)

**Purpose**: Directed graph analysis of vault documents (wiki-links + related fields).

**Public API**:
```python
@dataclass
class DocNode:
    path: pathlib.Path
    name: str
    doc_type: DocType | None
    tags: set[str]
    out_links: set[str]
    in_links: set[str]

class VaultGraph:
    def __init__(self, root_dir: pathlib.Path) -> None

    def get_hotspots(limit: int, doc_type: DocType | None, feature: str | None) -> list[tuple[str, int]]
    def get_feature_rankings(limit: int) -> list[tuple[str, int]]
    def get_orphaned() -> list[str]
    def get_invalid_links() -> list[tuple[str, str]]
```

**Graph Construction** (2-pass):
1. **Pass 1**: Create nodes with metadata (tags, doc_type)
2. **Pass 2**: Extract links and build bidirectional references

**Analysis Capabilities**:
- **Hotspots**: Documents with most incoming links (filter by type/feature)
- **Feature Rankings**: Features ranked by cumulative link count
- **Orphaned Docs**: No incoming links (excludes "readme")
- **Invalid Links**: Links to non-existent documents

**Implementation Status**: ✅ **Fully working**

**Test Coverage**: ✅ **Excellent** (Y:/code/task-worktrees/main/.vaultspec/lib/src/graph/tests/test_graph.py)
- Graph construction (>80 nodes)
- Link extraction (out_links, in_links)
- Filtering by type/feature
- Edge cases (orphans, invalid links)

**Use Cases**:
- Identify important documentation
- Find unused documents
- Detect broken links
- Analyze feature coverage

---

## 5. Metrics (`Y:/code/task-worktrees/main/.vaultspec/lib/src/metrics/api.py`)

**Purpose**: Aggregate vault statistics.

**Public API**:
```python
@dataclass
class VaultSummary:
    total_docs: int
    counts_by_type: dict[DocType, int]
    total_features: int

def get_vault_metrics(root_dir: pathlib.Path) -> VaultSummary
```

**Metrics Collected**:
- Total document count
- Breakdown by DocType (ADR, PLAN, RESEARCH, REFERENCE, EXEC)
- Unique feature count

**Implementation Status**: ✅ **Fully working**
- Uses `scan_vault()` for filesystem enumeration
- Delegates feature extraction to `verification.api.list_features()`

**Test Coverage**: ✅ **Good** (Y:/code/task-worktrees/main/.vaultspec/lib/src/metrics/tests/test_metrics.py)

---

## 6. Verification (`Y:/code/task-worktrees/main/.vaultspec/lib/src/verification/api.py`)

**Purpose**: Vault compliance checking against rigid schema rules.

**Public API**:
```python
class VerificationError:
    path: pathlib.Path
    message: str

def verify_vault_structure(root_dir: pathlib.Path) -> list[VerificationError]
def verify_file(path: pathlib.Path, root_dir: pathlib.Path) -> list[VerificationError]
def get_malformed(root_dir: pathlib.Path) -> list[VerificationError]
def list_features(root_dir: pathlib.Path) -> set[str]
def verify_vertical_integrity(root_dir: pathlib.Path) -> list[VerificationError]
```

**Verification Checks**:
1. **Structure**: No unsupported directories in `.vault/`
2. **Filename**: Matches `yyyy-mm-dd-<feature>-<type>.md` pattern
3. **Metadata**: Rule of Two, date format, tag format
4. **Directory Tag**: Matches parent directory
5. **Vertical Integrity**: Every feature has a plan document

**Implementation Status**: ✅ **Fully working**
- Uses `VaultConstants.validate_*()` methods
- Aggregates errors from all files
- Clear error messages with file paths

**Test Coverage**: ✅ **Good** (Y:/code/task-worktrees/main/.vaultspec/lib/src/verification/tests/test_verification.py)

**Use Cases**:
- CI/CD validation
- Pre-commit hooks
- `vaultspec docs audit --verify`

---

## 7. CLI Scripts (`Y:/code/task-worktrees/main/.vaultspec/lib/scripts/`)

### 7.1 `cli.py` - Resource Manager

**Purpose**: Multi-resource manager for rules, agents, skills, config, system prompts.

**Full Command Reference**:

#### Rules Commands
```bash
python cli.py rules list                      # List all rules
python cli.py rules add --name <name>         # Add custom rule (opens editor)
python cli.py rules add --name <name> --content "..."  # Add with inline content
python cli.py rules sync [--prune] [--dry-run]  # Sync to .claude/.gemini
```

#### Agents Commands
```bash
python cli.py agents list                     # List agents with tier/model info
python cli.py agents add --name <name> --tier {LOW|MEDIUM|HIGH}
python cli.py agents set-tier <name> --tier {LOW|MEDIUM|HIGH}
python cli.py agents sync [--prune] [--dry-run]
```

#### Skills Commands
```bash
python cli.py skills list                     # List managed skills
python cli.py skills add --name <name>        # Creates vaultspec-<name>.md
python cli.py skills sync [--prune] [--dry-run]
```

#### Config Commands
```bash
python cli.py config show                     # Display FRAMEWORK.md + PROJECT.md
python cli.py config sync [--force] [--dry-run]  # Generate CLAUDE.md, GEMINI.md
```

#### System Commands
```bash
python cli.py system show                     # Display system prompt parts
python cli.py system sync [--force] [--dry-run]  # Assemble SYSTEM.md
```

#### Sync All
```bash
python cli.py sync-all [--prune] [--force] [--dry-run]  # Sync everything
```

#### Test Runner
```bash
python cli.py test [all|unit|api|search|index|quality]
python cli.py test --module {cli|rag|vault|protocol|orchestration|subagent}
python cli.py test -- -v -k test_name  # Pass extra pytest args
```

**Global Options**:
```bash
--root <path>      # Override workspace root
--verbose / -v     # INFO level logging
--debug            # DEBUG level logging
```

**Key Design Patterns**:
1. **Source of Truth**: `.vaultspec/{rules,agents,skills}/*`
2. **Multi-Destination Sync**: One source → multiple tool-specific destinations
3. **Format Transformation**: Frontmatter adaptation per tool
4. **Tier Resolution**: `{LOW,MEDIUM,HIGH}` → tool-specific model IDs
5. **Safety Guards**: `--force` required to overwrite non-CLI-managed files
6. **Atomic Writes**: Temp file + rename to prevent corruption

**Protected Skills**: `fd`, `rg`, `sg`, `sd` (never pruned)

**Tool Configurations**:
```python
TOOL_CONFIGS = {
    "claude": {
        rules_dir: .claude/rules
        agents_dir: .claude/agents
        skills_dir: .claude/skills
        config_file: .claude/CLAUDE.md
    },
    "gemini": {
        rules_dir: .gemini/rules
        agents_dir: .gemini/agents
        skills_dir: .gemini/skills
        config_file: .gemini/GEMINI.md
        system_file: .gemini/SYSTEM.md
    },
    "antigravity": {
        rules_dir: .agent/rules
        skills_dir: .agent/skills
    },
    "agents": {
        config_file: AGENTS.md
    }
}
```

**Backward Compatibility**:
- Warns if `INTERNAL.md` → should rename to `FRAMEWORK.md`
- Warns if `CUSTOM.md` → should rename to `PROJECT.md`

**Implementation Status**: ✅ **Fully working**
- All commands implemented
- Dry-run support
- YAML parsing with PyYAML optional fallback
- Model tier resolution via provider API

**Test Coverage**: ⚠️ **Manual testing** (no unit tests for CLI commands)

**Integration Notes**:
- **Critical for `repo agent init`**: Use `cli.py sync-all` to bootstrap workspace
- Can be invoked as subprocess or imported directly

---

### 7.2 `docs.py` - Vault Audit CLI

**Purpose**: Audit, create, index, and search vault documents.

**Full Command Reference**:

#### Audit Command
```bash
python docs.py audit --summary                # Vault statistics
python docs.py audit --features               # List all features
python docs.py audit --verify                 # Run compliance checks
python docs.py audit --graph                  # Show hotspots
python docs.py audit --graph --type adr       # Filter by DocType
python docs.py audit --graph --feature editor-demo  # Filter by feature
python docs.py audit --limit 20               # Limit results
python docs.py audit --json                   # JSON output
```

**Combined Flags**:
```bash
python docs.py audit --summary --features --verify --graph
```

#### Create Command
```bash
python docs.py create --type {adr|plan|research|reference|exec} \
                      --feature <name> \
                      [--title "Title"]
```

**Behavior**:
- Hydrates template from `.vaultspec/templates/<type>.md`
- Generates filename: `yyyy-mm-dd-<feature>-<type>.md`
- Writes to `.vault/<type>/`

#### Index Command (RAG)
```bash
python docs.py index [--full] [--json]
```

**Requires**: `pip install -e '.[rag]'`

**Behavior**:
- Full: Re-index all documents
- Incremental: Update changed files only
- Uses GPU if available (reports VRAM)

#### Search Command (RAG)
```bash
python docs.py search "semantic query" [--limit 5] [--json]
```

**Output**:
- Semantic search results with scores
- Snippet preview
- Feature/type metadata

**Global Options**:
```bash
--root <path>      # Override vault root
--verbose / -v     # INFO level logging
--debug            # DEBUG level logging
```

**Implementation Status**: ✅ **Fully working**
- All commands functional
- JSON output support
- Graceful handling of missing RAG dependencies

**Test Coverage**: ⚠️ **Manual testing**

**Integration Notes**:
- `audit --json` → machine-readable for `repo agent audit`
- `create` → template for `repo agent new <type>`

---

### 7.3 `subagent.py` - Subagent Launcher

**Purpose**: CLI interface for running and serving subagents.

**Full Command Reference**:

#### Run Command
```bash
python subagent.py --root <path> run --agent <name> --goal "..."
python subagent.py --root <path> run --agent <name> --task "..."  # Legacy
python subagent.py --root <path> run --agent <name> --plan plan.md
python subagent.py --root <path> run --agent <name> --context adr.md --context research.md
python subagent.py --root <path> run --agent <name> --task-file task.md  # Legacy

# Options
--model <model-id>                    # Override default model
--provider {gemini|claude}            # Force specific provider
--mode {read-write|read-only}         # Permission mode (default: read-write)
--interactive / -i                    # Multi-turn conversation
--verbose / -v                        # INFO logging
--debug                               # DEBUG logging
```

**Task Context Assembly**:
```
# CURRENT PLAN
<plan-file-content>

# CONTEXT FILES
## File: adr.md
<content>

## File: research.md
<content>

# TASK
<goal>
```

#### Serve Command (MCP Server)
```bash
python subagent.py --root <path> serve
```

**Starts**: Subagent MCP server on `VAULTSPEC_MCP_PORT` (default 10010)

#### A2A Serve Command
```bash
python subagent.py --root <path> a2a-serve \
    --executor {claude|gemini} \
    --agent <name> \
    --port 10010 \
    --model <model-id> \
    --mode {read-write|read-only}
```

**Starts**: Agent-to-Agent HTTP server (A2A protocol)

#### List Command
```bash
python subagent.py --root <path> list
```

**Output**: All agents in `.vaultspec/agents/`

**Implementation Status**: ✅ **Fully working**
- All commands implemented
- Agent file auto-discovery
- Graceful error handling
- Read-only permission prompt injection

**Test Coverage**: ⚠️ **Manual testing**

**Integration Notes**:
- **Main entry point for `repo agent run`**
- Returns response text for capture
- Async execution (uses `asyncio.run_until_complete()`)

---

### 7.4 `_paths.py` - Path Bootstrap

**Purpose**: Shared path resolution for all scripts.

**Exports**:
```python
ROOT_DIR: Path         # Workspace root (4 levels up)
LIB_SRC_DIR: Path      # .vaultspec/lib/src/
```

**Key Behavior**:
- Adds `LIB_SRC_DIR` to `sys.path` for direct imports
- Consistent path references across all scripts

**Implementation Status**: ✅ **Complete**

---

## Integration Architecture for `repo agent`

### Recommended Approach: **Subprocess Invocation**

```rust
use std::process::Command;

fn run_vaultspec_cli(args: &[&str]) -> Result<String> {
    let output = Command::new("python")
        .arg(".vaultspec/lib/scripts/cli.py")
        .args(args)
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Usage
run_vaultspec_cli(&["agents", "list"])?;
run_vaultspec_cli(&["sync-all"])?;
```

### Alternative: **Python FFI via PyO3**

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

fn get_vault_metrics() -> PyResult<HashMap<String, i32>> {
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        sys.getattr("path")?.call_method1("insert", (0, ".vaultspec/lib/src"))?;

        let metrics = py.import("metrics.api")?;
        let result = metrics.getattr("get_vault_metrics")?.call1((Path::new("."),))?;

        // Extract data
        Ok(...)
    })
}
```

**Pros/Cons**:

| Approach | Pros | Cons |
|----------|------|------|
| **Subprocess** | ✅ Simple, no FFI complexity<br>✅ JSON output available<br>✅ Isolated Python runtime | ⚠️ Startup overhead<br>⚠️ No shared state |
| **PyO3 FFI** | ✅ Direct Python API access<br>✅ Shared memory | ⚠️ Complex build<br>⚠️ Python runtime dependency |

**Recommendation**: Start with **subprocess**, migrate to PyO3 only if performance is critical.

---

## CLI Command Mapping for `repo agent`

```bash
# Vaultspec                          # Repository Manager
python cli.py agents list            → repo agent list
python cli.py agents add             → repo agent new <name>
python cli.py sync-all               → repo agent sync
python docs.py audit --summary       → repo agent audit
python docs.py create --type adr     → repo agent new adr <feature>
python subagent.py run --agent X     → repo agent run <name>
```

---

## Key Findings for Integration

### ✅ Strengths
1. **Clean Separation**: Orchestration/Vault/Protocol are independent
2. **Well-Tested Core**: Task engine, vault scanner, graph all have unit tests
3. **Flexible Configuration**: Override via dict (no env vars needed)
4. **Type Safety**: Full type hints, no dynamic typing risks
5. **No Hard Dependencies**: Optional PyYAML, optional RAG

### ⚠️ Considerations
1. **Python 3.13 Required**: Uses modern type syntax (`|`, `StrEnum`)
2. **No Persistence**: Task engine is in-memory only
3. **CLI Tests Missing**: Manual testing only for CLI commands
4. **RAG Optional**: `pip install -e '.[rag]'` for search/index

### 🔧 Recommended Changes (Optional)
1. Add unit tests for CLI commands (priority: LOW)
2. Document config override patterns (priority: MEDIUM)
3. Add JSON output to all CLI commands (priority: HIGH for integration)
4. Consider persistent task storage for long-running agents (priority: LOW)

---

## Appendix: Module Dependencies

```
orchestration/
├── task_engine.py (standalone, thread-safe)
├── subagent.py
│   ├── acp (Agent Communication Protocol)
│   ├── protocol.providers.{claude,gemini}
│   ├── protocol.acp.client
│   └── vault.parser
├── utils.py (standalone)
└── constants.py (standalone)

vault/
├── models.py (standalone)
├── parser.py
│   └── vault.models
├── scanner.py
│   ├── vault.models
│   └── core.config
├── hydration.py
│   ├── vault.models
│   └── core.config
└── links.py (standalone)

core/
└── config.py (standalone, stdlib only)

graph/
└── api.py
    ├── vault.{scanner,parser,links,models}
    └── core.config

metrics/
└── api.py
    ├── vault.{scanner,models}
    ├── verification.api
    └── core.config

verification/
└── api.py
    ├── vault.{scanner,parser,models}
    └── core.config

scripts/
├── cli.py
│   ├── vault.parser
│   ├── protocol.providers.{claude,gemini}
│   └── core.config
├── docs.py
│   ├── vault.*
│   ├── graph.api
│   ├── metrics.api
│   ├── verification.api
│   └── rag.api (optional)
├── subagent.py
│   ├── orchestration.subagent
│   ├── subagent_server.server
│   └── protocol.*
└── _paths.py (standalone)
```

---

## Conclusion

**Overall Assessment**: The vaultspec core systems are **production-ready** and suitable for integration into a Rust CLI as a Python plugin.

**Viability Score**: **9/10**
- Deduction for missing CLI tests and lack of persistence layer

**Next Steps**:
1. Audit #2: Protocol layer (A2A, ACP, providers, sandbox, MCP server)
2. Audit #3: Rules, agents, skills, RAG, documentation
3. Integration plan: Design `repo agent` command structure and Python invocation strategy

---

**End of Audit Report**
Generated: 2026-02-17
File: `Y:/code/repository-manager-worktrees/main/docs/audits/2026-02-17-deep-audit/vaultspec-01-core-systems.md`
