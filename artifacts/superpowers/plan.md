# Documentation Cleanup & Fix Plan

> **For Antigravity:** Use `/superpowers-execute-plan` to implement this plan.

**Goal:** Fix the errors from the previous reorganization:

1. Move the leftover "Skill context" research document to a more appropriate location (`repository-management`).
2. Clean up the `metadata.md` file to remove the inaccurate architecture graph and simplify it to a high-level correlation document.
3. Remove the redundant/unwanted `docs/design/README.md`.

**Architecture:** Use the existing folder structure.

---

## Task 1: Fix Leftover Files

**Files:**

- Move: `docs/design/01-skill-context-management-2026.md` -> `docs/design/repository-management/skill-context-management-2026.md`
- Remove: `docs/design/README.md` (User requested removal/didn't ask for it).
- Edit: `docs/design/metadata.md` (Update link for context management if needed, though no link existed).

**Step 1: Move file**
Run:

```powershell
Move-Item docs/design/01-skill-context-management-2026.md docs/design/repository-management/skill-context-management-2026.md
```

**Step 2: Remove README**
Run:

```powershell
Remove-Item docs/design/README.md
```

**Step 3: Commit**
Run:

```bash
git add docs/design/repository-management/skill-context-management-2026.md docs/design/README.md docs/design/01-skill-context-management-2026.md
git commit -m "docs: move leftover research doc and remove unwanted readme"
```

## Task 2: Fix Imprecise Metadata

**Files:**

- Edit: `docs/design/metadata.md`

**Step 1: Overwrite metadata.md**
Content for `docs/design/metadata.md`:

```markdown
# System Crystallization & Correlation

This document serves as the high-level map correlating the various design documents.

> **Status**: Early Design Phase. Architecture is evolving.

## Subsystem Index

*   **[CLI](cli/spec.md)**: Top-level command line tool implementation.
*   **[Tools](tools/spec.md)**: Definition and registration of external tools (coding agents, IDEs).
*   **[Presets](presets/spec.md)**: Capability provider system (venvs, gitignores, configs).
*   **[Repository Management](repository-management/architecture.md)**: Core logic for repository structure.
    *   *[Context Research](repository-management/skill-context-management-2026.md)*: Analysis of agentic context patterns.
*   **[Metadata System](metadata-system/spec.md)**: The `.repository` directory structure and registry.
*   **[File Management](file-management/spec.md)**: Robust I/O utilities.
*   **[Git Management](git-management/spec.md)**: Worktree and remote sync management.

## Correlation Goals

*   **Presets** provide the capabilities.
*   **Tools** consume the environment configured by Presets.
*   **Metadata System** connects them by registering which tools and presets are active in a repository.
*   **CLI** is the conductor that orchestrates these interactions.
```

**Step 2: Commit**
Run:

```bash
git add docs/design/metadata.md
git commit -m "docs: simplify metadata.md and remove inaccurate graph"
```

## Risks & Mitigations

- **Risk**: Deleting `README.md` leaves folder without index on GitHub.
  - **Mitigation**: `metadata.md` effectively acts as the index now, which is what the user seems to prefer ("wrap all that using a metadata.md").

## Rollback

```bash
git reset --hard HEAD~2
```
