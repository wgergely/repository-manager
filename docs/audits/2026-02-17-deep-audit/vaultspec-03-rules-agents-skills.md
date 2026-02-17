# Vaultspec Declarative Systems Audit: Rules, Agents, Skills, Templates, and RAG

**Auditor:** Auditor3
**Date:** 2026-02-17
**Scope:** Deep audit of vaultspec's declarative configuration systems (`.vaultspec/` directory)

---

## Executive Summary

Vaultspec implements a **markdown-based declarative configuration system** for AI-assisted development. All governance structures—rules, agent definitions, skills, templates, system prompts—are defined as markdown files in `.vaultspec/` with YAML frontmatter. A Python runtime (`cli.py`, `subagent.py`) loads these definitions and orchestrates multi-agent workflows backed by a **RAG (Retrieval-Augmented Generation) module** for semantic search over documentation artifacts.

**Key Finding:** The system is **highly modular and extensible**. New agents, skills, or workflow rules can be added by creating markdown files following documented patterns. The RAG module provides GPU-accelerated semantic search using `nomic-embed-text-v1.5` on CUDA, stored in LanceDB with hybrid BM25+ANN retrieval.

---

## Summary Table

| Category | Count | Format | Key Features |
|----------|-------|--------|--------------|
| **Rules** | 2 | Markdown | Workflow enforcement (sub-agents, skills) |
| **Agents** | 9 | Markdown + YAML frontmatter | Tiered (HIGH/MEDIUM/LOW), role-based, tool constraints |
| **Skills** | 12 | Markdown + YAML frontmatter | User-invocable workflows (research, plan, execute, review) |
| **Templates** | 8 | Markdown + YAML frontmatter | Standardized document schemas (ADR, plan, exec-step, etc.) |
| **System Prompts** | 4 | Markdown | Composable prompt fragments (base, gemini, operations, workflow) |
| **RAG Module** | 5 core files + tests | Python | GPU-accelerated semantic search (nomic-embed-text, LanceDB, hybrid search) |

---

## 1. Rules System (`Y:/code/task-worktrees/main/.vaultspec/rules/`)

### 1.1 Purpose
Enforce project-level development workflow policies that mandate sub-agent-based development and ADR-backed plans.

### 1.2 Files

#### `vaultspec-skills.builtin.md`
- **Enforces:** Spec-driven development pipeline (Research → ADR → Plan → Execute → Verify)
- **Mandates:**
  - Documentation persistence in `.vault/` (research, adr, plan, reference, exec logs, summaries)
  - Use of high-level skills: `vaultspec-research`, `vaultspec-adr`, `vaultspec-reference`, `vaultspec-write`, `vaultspec-execute`
  - Sub-agent dispatch via `vaultspec-subagent` skill

#### `vaultspec-subagents.builtin.md`
- **Declares:** Sub-agents as the de facto standard for meaningful work
- **Mechanism:**
  - CLI dispatch: `python .vaultspec/lib/scripts/subagent.py run --agent <name> --goal "<task>"`
  - MCP server dispatch (preferred): `vs-subagent-mcp` with `list_agents` and `dispatch_agent` tools
  - Agents loaded from `.vaultspec/agents/` markdown files
  - Session logs persisted to `.vaultspec/logs/yyyy-mm-dd-<session_id>.log`

### 1.3 Extensibility
- Add new workflow policies by creating markdown files in `rules/`
- No Python code changes required
- Files are loaded and injected into system prompts via `cli.py config sync`

---

## 2. Agent Definitions (`Y:/code/task-worktrees/main/.vaultspec/agents/`)

### 2.1 Purpose
Define specialized AI agents with role-based capabilities, tool constraints, and operational mandates.

### 2.2 Catalog (9 Agents)

| Agent Name | Tier | Mode | Tools | Role |
|------------|------|------|-------|------|
| **vaultspec-adr-researcher** | HIGH | read-only | Glob, Grep, Read, WebFetch, WebSearch, Bash | Lead Technical Researcher. Conducts research, synthesizes pathways, formalizes ADRs. |
| **vaultspec-code-reviewer** | HIGH | read-only | Glob, Grep, Read, Bash | Lead Code Reviewer & Safety Officer. Audits safety, intent, and quality. |
| **vaultspec-complex-executor** | HIGH | read-write | Glob, Grep, Read, Write, Edit, Bash | Lead Implementation Engineer (Hard-Tier). Handles complex refactors, core logic, advanced Rust features. |
| **vaultspec-docs-curator** | MEDIUM | read-write | Glob, Grep, Read, Write, Edit, Bash | Documentation Vault Curator. Enforces frontmatter, wiki-links, naming conventions, template compliance. |
| **vaultspec-reference-auditor** | MEDIUM | read-only | Glob, Grep, Read, Bash | Reference Codebase Specialist. Audits reference implementations (e.g., Zed) for patterns. |
| **vaultspec-researcher** | MEDIUM | read-only | Glob, Grep, Read, WebFetch, WebSearch, Bash | General-purpose research agent. Info gathering, analysis, synthesis. |
| **vaultspec-simple-executor** | LOW | read-write | Glob, Grep, Read, Write, Edit, Bash | Lead Implementation Engineer (Simple-Tier). Straightforward edits, docs, low-risk logic. |
| **vaultspec-standard-executor** | MEDIUM | read-write | Glob, Grep, Read, Write, Edit, Bash | Lead Implementation Engineer (Standard-Tier). Typical features, component work, standard logic. |
| **vaultspec-writer** | HIGH | read-write | Glob, Grep, Read, Write, Edit, Bash | Senior Task Orchestrator & Delegator. Converts ADRs into implementation plans. |

### 2.3 Structure
Each agent file contains:
- **YAML frontmatter:**
  - `description`: One-line summary
  - `tier`: HIGH/MEDIUM/LOW (determines model/cost)
  - `mode`: read-only or read-write
  - `tools`: Comma-separated list of allowed tools
- **Markdown body:**
  - Persona definition
  - Core mandates
  - Workflow rules
  - Document persistence rules (templates, frontmatter schema, wiki-links)
  - Tooling strategy (fd, rg, sd, sg, cargo-*, etc.)

### 2.4 Extensibility
- Add new agents by creating markdown files in `agents/` with frontmatter schema
- No Python code changes required
- Agents are loaded dynamically by `subagent.py` based on `--agent <name>` argument

### 2.5 Integration with Code
- `subagent.py` parses agent frontmatter and constructs prompts
- Tools list restricts what the agent can invoke
- Mode (read-only/read-write) enforces filesystem permissions
- Tier determines model selection and cost budgeting

---

## 3. Skills System (`Y:/code/task-worktrees/main/.vaultspec/skills/`)

### 3.1 Purpose
Define user-invocable workflows that orchestrate multi-agent tasks and enforce documentation standards.

### 3.2 Catalog (12 Skills)

| Skill Name | Invocation | Purpose |
|------------|------------|---------|
| **vaultspec-adr** | User explicit | Persist ADRs after research. Mandates linking to `<Research>`. Template: `.vaultspec/templates/adr.md`. |
| **vaultspec-curate** | Post-feature / periodic | Audit `.vault/` for frontmatter, wiki-links, naming, template compliance. Dispatches `vaultspec-docs-curator`. |
| **vaultspec-execute** | Plan execution | Execute implementation plans. Delegates to executors (simple/standard/complex). Mandates code review. |
| **vaultspec-fd** | Utility | File discovery using `fd` CLI. Preferred over `find`/`ls`. |
| **vaultspec-reference** | ADR/Plan phase | Audit reference codebases (e.g., Zed) for implementation patterns. Dispatches `vaultspec-reference-auditor`. |
| **vaultspec-research** | Feature kickoff | Conduct structured research. Dispatches `vaultspec-adr-researcher`. Template: `.vaultspec/templates/research.md`. |
| **vaultspec-review** | Post-execute (mandatory) | Code review for safety, intent, quality. Dispatches `vaultspec-code-reviewer`. Template: `.vaultspec/templates/code-review.md`. |
| **vaultspec-rg** | Utility | High-performance search using `rg` (ripgrep). Preferred over `grep`. |
| **vaultspec-sd** | Utility | Find-and-replace using `sd`. Preferred for text manipulation. |
| **vaultspec-sg** | Utility | AST-based code manipulation using `sg` (ast-grep). For structural refactoring. |
| **vaultspec-subagent** | Internal | Dispatch sub-agents. CLI: `subagent.py run`. MCP: `vs-subagent-mcp`. **Not user-invoked directly**. |
| **vaultspec-write** | Post-ADR | Write implementation plans. Dispatches `vaultspec-writer`. Template: `.vaultspec/templates/plan.md`. |

### 3.3 Structure
Each skill file contains:
- **YAML frontmatter:**
  - `description`: One-line summary
- **Markdown body:**
  - When to use
  - Workflow steps (often "dispatch agent X, persist to Y")
  - Template requirements
  - Frontmatter & tagging mandates (EXACTLY TWO tags: directory + feature)
  - Linking rules (quoted wiki-links, no `@ref`)

### 3.4 Extensibility
- Add skills by creating markdown files in `skills/`
- Skills invoke agents via `vaultspec-subagent` skill
- No Python changes required
- Skills are referenced in system prompts and agent instructions

---

## 4. Templates (`Y:/code/task-worktrees/main/.vaultspec/templates/`)

### 4.1 Purpose
Provide standardized schemas for documentation artifacts persisted to `.vault/`.

### 4.2 Catalog (8 Templates)

| Template File | Purpose | Persisted To |
|---------------|---------|--------------|
| **readme.md** | Master rulebook for `.vault/` hierarchy, tag taxonomy, frontmatter schema, placeholder naming conventions. | N/A (reference doc) |
| **research.md** | Research findings template. | `.vault/research/yyyy-mm-dd-<feature>-<phase>-research.md` |
| **adr.md** | ADR schema (Problem, Considerations, Constraints, Implementation, Rationale, Consequences). | `.vault/adr/yyyy-mm-dd-<feature>-<phase>-adr.md` |
| **plan.md** | Implementation plan schema (Proposed Changes, Tasks, Parallelization, Verification). | `.vault/plan/yyyy-mm-dd-<feature>-<phase>-plan.md` |
| **exec-step.md** | Execution step record (Modified files, Description, Tests). | `.vault/exec/yyyy-mm-dd-<feature>/yyyy-mm-dd-<feature>-<phase>-<step>.md` |
| **exec-summary.md** | Phase summary (Overall progress, Modified files, Tests). | `.vault/exec/yyyy-mm-dd-<feature>/yyyy-mm-dd-<feature>-<phase>-summary.md` |
| **code-review.md** | Code review report (Status: PASS/FAIL/REVISION REQUIRED, Findings by severity, Recommendations). | `.vault/exec/yyyy-mm-dd-<feature>/yyyy-mm-dd-<feature>-review.md` |
| **ref-audit.md** | Reference audit findings (Sources, Findings). | `.vault/reference/yyyy-mm-dd-<feature>-reference.md` |

### 4.3 Frontmatter Schema (Universal)
All templates enforce:
```yaml
---
# ALLOWED TAGS - DO NOT REMOVE - REFERENCE: #adr #exec #plan #reference #research #<feature>
tags:
  - "#<directory-tag>"  # ONE OF: adr, exec, plan, reference, research
  - "#<feature-tag>"    # kebab-case feature identifier
date: yyyy-mm-dd
related:
  - "[[wiki-link-1]]"   # Quoted wiki-links only
  - "[[wiki-link-2]]"
---
```

**Critical Rules:**
- **EXACTLY TWO TAGS:** Directory tag + feature tag
- **No structural tags:** No `#step`, `#phase`, `#summary`, etc.
- **Quoted wiki-links:** `"[[file-name]]"` in YAML (prevents parse errors)
- **No relative paths:** Assume flat namespace or vault-root resolution

### 4.4 Placeholder Conventions
- `<feature>`: lowercase, kebab-case (e.g., `editor-demo`)
- `<phase>`: lowercase, kebab-case (e.g., `phase-1`)
- `<step>`: lowercase, kebab-case (e.g., `task-1-window-setup`)
- `<yyyy-mm-dd>`: ISO 8601 date
- All placeholders must be replaced; no `<...>` should remain in committed docs

### 4.5 Extensibility
- Add templates by creating markdown files in `templates/`
- Update `readme.md` to document new template rules
- No Python changes required

---

## 5. System Prompts (`Y:/code/task-worktrees/main/.vaultspec/system/`)

### 5.1 Purpose
Composable system prompt fragments assembled into tool-specific configurations.

### 5.2 Files

| File | Purpose | Target Tools |
|------|---------|-------------|
| **base.md** | Core mandates (conventions, libraries, style, comments, proactiveness, skill guidance). | All tools |
| **gemini.md** | Tool-specific: shell (pwsh), CLI tools (fd, rg, sd, sg), hook context, sandboxing reminder. | Gemini CLI |
| **operations.md** | Operational guidelines (token efficiency, tone/style, security, tool usage, git workflow). | All tools |
| **workflow.md** | Primary workflows (software tasks, new apps). Mandates ADR-driven development. | All tools |

### 5.3 Assembly Process
- `cli.py system sync` assembles parts from `system/` into `SYSTEM.md`
- Syncs to tool destinations (`.gemini/SYSTEM.md`, etc.)
- Allows tool-specific customization while maintaining shared base

### 5.4 Extensibility
- Add new system prompt parts by creating markdown files in `system/`
- Update `cli.py` to include new parts in assembly
- Override per-tool by creating tool-specific variants

---

## 6. Framework Documentation

### 6.1 Files

| File | Purpose |
|------|---------|
| **FRAMEWORK.md** | Core framework & mission. Read-only. Synced to `AGENTS.md`, `CLAUDE.md`, `GEMINI.md` via `cli.py config sync`. |
| **PROJECT.md** | Project-specific context (currently empty placeholder). User-editable. Appended to generated config files. |
| **README.md** | User manual. Workflow overview (Research → Specify → Plan → Execute → Verify), agent reference table, context management rules. |

### 6.2 Context Management
- **FRAMEWORK.md** stored in YAML frontmatter (`system_framework` key) in generated files for syntactic stability
- **PROJECT.md** appended verbatim to tool configs
- `cli.py config sync` generates:
  - `./AGENTS.md` (root-level AI entry point)
  - `.claude/CLAUDE.md` (Claude Code config)
  - `.gemini/GEMINI.md` (Gemini CLI config)

---

## 7. RAG Module (`Y:/code/task-worktrees/main/.vaultspec/lib/src/rag/`)

### 7.1 Purpose
Provide GPU-accelerated semantic search over vault documentation artifacts using embedding-based retrieval + hybrid BM25+ANN search.

### 7.2 Architecture

**Core Components:**

| File | Purpose |
|------|---------|
| **api.py** | Public facade. Tier 1 (filesystem-only): `list_documents`, `get_document`, `get_related`, `get_status`. Tier 2 (RAG): `index`, `search`. Singleton `VaultRAG` engine. |
| **embeddings.py** | Embedding model wrapper. Uses `nomic-ai/nomic-embed-text-v1.5` (768-dim) on CUDA. **CPU NOT SUPPORTED**. Batch encoding, LRU query cache. |
| **indexer.py** | Indexing pipeline. Scans `.vault/`, parses metadata, embeds documents, stores in LanceDB. Supports full and incremental indexing (mtime-based). |
| **search.py** | Retrieval pipeline. Query parsing (filter extraction: `type:adr`, `feature:rag`), hybrid search (BM25 + ANN), graph-aware re-ranking (authority, neighborhood, recency). |
| **store.py** | LanceDB vector store. Table: `vault_docs` (768-dim vectors, full markdown content). Hybrid search via RRF reranker. SQL injection prevention via escaping. |

**Tests:** Unit tests in `tests/` for embeddings, indexer, search, store, query parsing.

### 7.3 Technical Details

#### 7.3.1 Embedding Model
- **Model:** `nomic-ai/nomic-embed-text-v1.5` (768 dimensions)
- **Backend:** `sentence-transformers` on CUDA (PyTorch)
- **Prefixes:**
  - Documents: `"search_document: " + text`
  - Queries: `"search_query: " + text`
- **Batch Encoding:** Sorts by length before batching to minimize padding waste
- **Truncation:** Documents truncated to 8000 chars (8192 tokens) to avoid massive padding overhead
- **Cache:** Query embeddings LRU-cached (128 entries)

#### 7.3.2 Vector Store (LanceDB)
- **Storage:** `{root_dir}/.lance/vault_docs` table
- **Schema:**
  - `id` (string): Document stem
  - `path` (string): Relative path
  - `doc_type` (string): adr, plan, exec, research, reference
  - `feature` (string): Feature tag without `#`
  - `date` (string): ISO date from frontmatter
  - `tags` (string): JSON-serialized list
  - `related` (string): JSON-serialized wiki-links
  - `title` (string): H1 heading
  - `content` (string): Full markdown body (for BM25)
  - `vector` (list[float, 768]): Embedding

#### 7.3.3 Hybrid Search
- **BM25:** Full-text search on `content` via Tantivy FTS index
- **ANN:** Vector search using LanceDB's IVF index
- **Reranking:** RRF (Reciprocal Rank Fusion) reranker combines BM25 and ANN results
- **Filters:** SQL WHERE clauses for `doc_type`, `feature`, `date` (prefix match)
- **SQL Injection Prevention:** Filter values sanitized (escape single quotes, strip control chars)

#### 7.3.4 Graph-Aware Re-ranking
- **Authority Boost:** `score *= (1 + 0.1 * min(in_link_count, 10))`
- **Neighborhood Boost:** `score *= 1.15` if wiki-link neighbors share feature tag
- **Recency Boost:** `score *= (1 + 0.02 * rank)` (most recent gets highest rank)

#### 7.3.5 Indexing
- **Full Index:** Scans all docs, embeds, replaces entire store
- **Incremental Index:** Compares mtimes against metadata, re-indexes new/modified, deletes removed
- **Metadata:** Sidecar JSON (`{root_dir}/.lance/index_metadata.json`) tracks file mtimes
- **Concurrency:** Parallel I/O for file reading, serial embedding (GPU batch encoding)

### 7.4 Dependencies
- **torch** (CUDA 12.4+)
- **sentence-transformers**
- **lancedb**
- **pyarrow**
- Install: `pip install -e '.[rag]'`

### 7.5 Integration with Code
- **CLI:** `cli.py rag index`, `cli.py rag search "<query>"`
- **MCP Server:** (Planned) `vs-rag-mcp` with `index_vault`, `search_vault` tools
- **API:** Imported by other modules for document lookup, feature listing, status checks

### 7.6 Extensibility
- **New Embedding Models:** Update `EmbeddingModel.MODEL_NAME` in `embeddings.py`
- **New Vector Stores:** Replace `VaultStore` implementation (requires implementing `upsert`, `delete`, `hybrid_search`)
- **New Rerankers:** Replace RRF with custom reranker in `store.py`
- **New Filters:** Add SQL filter logic in `store.py::_build_where()`

---

## 8. Integration Assessment: Declaratives → Runtime

### 8.1 How Markdown Definitions Connect to Python Runtime

| Declarative File | Runtime Loader | Mechanism |
|------------------|----------------|-----------|
| **Rules** (`rules/*.md`) | `cli.py config sync` | Injected into system prompts (FRAMEWORK.md, CLAUDE.md, GEMINI.md). Loaded as context by AI tools. |
| **Agents** (`agents/*.md`) | `subagent.py` | Parses YAML frontmatter for tier, mode, tools. Injects markdown body into agent prompt. Tool list restricts invocable tools. |
| **Skills** (`skills/*.md`) | System prompts | Referenced in agent instructions. No explicit Python loader (AI follows skill instructions). |
| **Templates** (`templates/*.md`) | Agents/Skills | Agents read templates via `Read` tool, populate placeholders, persist to `.vault/`. |
| **System Prompts** (`system/*.md`) | `cli.py system sync` | Assembled into `SYSTEM.md`, synced to tool destinations (`.gemini/SYSTEM.md`). |
| **RAG** (`lib/src/rag/*.py`) | `api.py::get_engine()` | Singleton engine loads embedding model, store, indexer, searcher on first access. Lazy init. |

### 8.2 Key Observations
- **No code generation:** Markdown files are not compiled; they are loaded as text and injected into prompts
- **Hot-reloadable:** Changes to markdown files take effect immediately (no restart required)
- **Type safety:** YAML frontmatter provides schema validation (tier, mode, tools)
- **Governance enforcement:** Rules and templates enforce documentation standards via natural language constraints (agents read and follow them)

---

## 9. Extensibility Analysis

### 9.1 Adding a New Rule
1. Create markdown file in `.vaultspec/rules/<rule-name>.md`
2. Document the rule (what it enforces, when it applies)
3. Run `cli.py config sync` to inject into system prompts
4. No Python changes required

### 9.2 Adding a New Agent
1. Create markdown file in `.vaultspec/agents/vaultspec-<agent-name>.md`
2. Add YAML frontmatter:
   ```yaml
   ---
   description: "One-line summary"
   tier: HIGH | MEDIUM | LOW
   mode: read-only | read-write
   tools: Glob, Grep, Read, Write, Edit, Bash
   ---
   ```
3. Define persona, mandates, workflow in markdown body
4. Invoke via `python .vaultspec/lib/scripts/subagent.py run --agent vaultspec-<agent-name> --goal "<task>"`
5. No Python changes required

### 9.3 Adding a New Skill
1. Create markdown file in `.vaultspec/skills/vaultspec-<skill-name>.md`
2. Add YAML frontmatter:
   ```yaml
   ---
   description: "One-line summary"
   ---
   ```
3. Document: when to use, workflow steps, template requirements, frontmatter schema
4. Reference skill in agent instructions or system prompts
5. No Python changes required

### 9.4 Adding a New Template
1. Create markdown file in `.vaultspec/templates/<template-name>.md`
2. Define YAML frontmatter schema, placeholders, body structure
3. Update `.vaultspec/templates/readme.md` to document template usage
4. Reference template in agent/skill instructions
5. No Python changes required

### 9.5 Extending RAG
- **New Embedding Model:** Update `EmbeddingModel.MODEL_NAME` in `embeddings.py`, reindex
- **New Vector Store:** Implement `VaultStore` interface, update `api.py::VaultRAG`
- **New Reranker:** Replace `RRFReranker` in `store.py::hybrid_search()`
- **New Metadata Fields:** Add columns to LanceDB schema, update `VaultDocument` dataclass

---

## 10. Critical Integration Insights for `repo agent` Wrapper

### 10.1 What Makes Vaultspec Viable for Integration?

**Strengths:**
1. **Pure Markdown Governance:** All rules, agents, skills are markdown. Can be versioned, diffed, reviewed like code.
2. **No Hard-Coded Workflows:** Workflow logic lives in markdown, not Python. Easy to customize without forking.
3. **Modular Agent System:** Agents are isolated, composable. Can invoke individually or chain together.
4. **RAG-Backed Context:** Semantic search over documentation ensures agents have relevant context without token bloat.
5. **Hot-Reloadable:** Changes to rules/agents/skills take effect immediately.

**Challenges:**
1. **GPU Dependency:** RAG requires CUDA GPU. No CPU fallback. Limits deployment environments.
2. **Tool Coupling:** Skills reference specific CLI tools (`fd`, `rg`, `sd`, `sg`). Must be installed.
3. **Prompt Injection Risk:** System prompts are assembled from markdown. Malicious markdown could inject arbitrary instructions.
4. **No Access Control:** Agents can invoke any sub-agent. No RBAC or permission system.

### 10.2 Integration Strategy for `repo agent`

**Option A: Vaultspec as Embedded Library**
- `repo agent` invokes `subagent.py` directly
- RAG module provides context retrieval
- Markdown configs bundled with `repo-manager`
- **Pros:** Full control, no external dependencies
- **Cons:** GPU requirement limits deployment

**Option B: Vaultspec as External Service**
- `repo agent` calls vaultspec MCP server (`vs-subagent-mcp`, `vs-rag-mcp`)
- Vaultspec runs on GPU-enabled server
- **Pros:** GPU not required on client
- **Cons:** Network latency, auth/security overhead

**Option C: Hybrid (Recommended)**
- **Tier 1 (No RAG):** `repo agent` uses vaultspec markdown configs, invokes agents via `subagent.py`, no GPU required
- **Tier 2 (RAG):** Optional RAG server for semantic search (MCP server)
- **Pros:** Works without GPU, optional RAG enhancement
- **Cons:** Complexity of managing two tiers

### 10.3 Key Takeaways

1. **Vaultspec is a governance framework, not a code generator.** It enforces discipline via natural language rules and agent personas.
2. **The RAG module is optional.** Filesystem-based vault tools (`list_documents`, `get_document`, `get_related`) work without GPU.
3. **Extensibility is excellent.** New rules/agents/skills/templates require zero Python changes.
4. **Integration surface is clean.** `subagent.py` is the primary entry point. MCP server provides optional RPC interface.

---

## 11. Recommendations

### 11.1 For Vaultspec Maintainers
- [ ] Add CPU fallback for RAG (even if slower/lower quality)
- [ ] Implement RBAC for agent invocation
- [ ] Sanitize markdown configs to prevent prompt injection
- [ ] Document MCP server protocol in FRAMEWORK.md

### 11.2 For `repo agent` Integration
- [ ] Start with Tier 1 (no RAG) integration
- [ ] Validate markdown parsing robustness (handle malformed frontmatter)
- [ ] Implement agent sandboxing (restrict file access, network, etc.)
- [ ] Build test suite for agent invocation (mock `subagent.py` responses)
- [ ] Prototype MCP client for optional RAG server

---

## Appendix A: Workflow Diagram (from README.md)

The vaultspec workflow is:

```
Research (vaultspec-research + vaultspec-adr-researcher)
  ↓
ADR (vaultspec-adr + vaultspec-writer)
  ↓
Plan (vaultspec-write + vaultspec-writer)
  ↓
Execute (vaultspec-execute + vaultspec-complex/standard/simple-executor)
  ↓
Review (vaultspec-review + vaultspec-code-reviewer)
  ↓ (if pass)
Commit
```

Each step persists artifacts to `.vault/` using standardized templates. The `vaultspec-docs-curator` agent periodically audits `.vault/` for compliance.

---

## Appendix B: RAG Query Examples

**Query with filters:**
```
type:adr feature:rag vector database
```
Parses to:
- Text: `"vector database"`
- Filters: `{"doc_type": "adr", "feature": "rag"}`

**Hybrid search:**
1. BM25 matches: `"vector database"` in document content
2. ANN matches: Semantic similarity to query embedding
3. RRF reranker combines rankings
4. Graph re-ranking boosts authoritative docs (high in-link count)

---

**End of Audit**
