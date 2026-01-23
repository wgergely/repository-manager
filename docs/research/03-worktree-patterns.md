# Git Worktree Patterns for Agentic Development

*Research Document: Phase 2 - Worktree Directory Structures*
*Date: 2026-01-23*

## Executive Summary

This document analyzes git worktree patterns and evaluates three candidate solutions for organizing agentic development environments where configuration must be shared across multiple branch workspaces. The goal is to enable agentic tools (Claude Code, Cursor, Windsurf, etc.) to discover configuration files from a container directory while keeping git branches isolated in worktrees.

Three approaches are documented: Solution A (Centralized Git Database), Solution B (Orphaned Utility Branch), and Solution C (Hybrid/Submodule). Each has distinct trade-offs in terms of setup complexity, version control integration, and maintenance overhead.

---

## 1. Git Worktree Fundamentals

### 1.1 How Git Worktrees Work

Git worktrees allow multiple working directories to be attached to a single repository. Understanding the internals is crucial for designing agentic configuration patterns.

#### Directory Structure Internals

When you create a worktree, Git establishes a bidirectional link:

```text
main-repo/
├── .git/                           # Main git database
│   ├── config                      # Shared configuration
│   ├── objects/                    # Shared object store
│   ├── refs/                       # Shared refs
│   └── worktrees/                  # Per-worktree data
│       ├── feature-a/
│       │   ├── HEAD               # Worktree-specific HEAD
│       │   ├── gitdir             # Path back to worktree
│       │   ├── index              # Worktree-specific index
│       │   └── config.worktree    # Optional worktree-specific config
│       └── feature-b/
│           └── ...
└── (working files for main branch)

/other/path/feature-a/
├── .git                            # FILE (not directory) containing:
│                                   # gitdir: /main-repo/.git/worktrees/feature-a
└── (working files for feature-a)
```

#### Key Environment Variables

Git uses two environment variables to navigate worktree structures:

| Variable           | Points To                     | Purpose                                   |
| :----------------- | :---------------------------- | :---------------------------------------- |
| `$GIT_DIR`         | Worktree's private directory  | Per-worktree state (HEAD, index)          |
| `$GIT_COMMON_DIR`  | Main repository's .git        | Shared resources (objects, refs, config)  |

#### The .git File vs .git Directory

- **Main worktree**: Has a `.git/` directory containing the full database
- **Linked worktrees**: Have a `.git` **file** containing a `gitdir:` pointer

Example `.git` file content:

```text
gitdir: /path/to/main-repo/.git/worktrees/feature-a
```

### 1.2 Configuration Sharing Behavior

#### Default Behavior

The repository `config` file at `$GIT_COMMON_DIR/config` is shared across all worktrees.

#### Worktree-Specific Configuration

Git 2.20+ supports per-worktree configuration:

```bash
# Enable worktree-specific config
git config extensions.worktreeConfig true
```

With this enabled, configuration precedence becomes:

1. `$GIT_COMMON_DIR/config` (shared base)
2. `$GIT_DIR/config.worktree` (worktree-specific overrides)

**Warning:** Enabling `extensions.worktreeConfig` breaks compatibility with Git versions < 2.20.

### 1.3 Ref Sharing Rules

| Ref Type           | Location            | Shared?            |
| :----------------- | :------------------ | :----------------- |
| `refs/heads/*`     | `$GIT_COMMON_DIR`   | Yes                |
| `refs/tags/*`      | `$GIT_COMMON_DIR`   | Yes                |
| `refs/remotes/*`   | `$GIT_COMMON_DIR`   | Yes                |
| `HEAD`             | `$GIT_DIR`          | No (per-worktree)  |
| `refs/bisect/*`    | `$GIT_DIR`          | No (per-worktree)  |
| `refs/worktree/*`  | `$GIT_DIR`          | No (per-worktree)  |

---

## 2. Agentic Tool Configuration Discovery

### 2.1 Current Discovery Patterns

Most agentic coding tools use a directory-traversal algorithm to find configuration:

```text
Current directory discovery pattern:
1. Look for config in current working directory
2. Traverse up to parent directories
3. Stop at repository root (detected by .git)
4. Fall back to user-level config (~/.config/tool/)
```

**Problem with Worktrees:** When an agentic tool opens a linked worktree, it typically:

1. Finds the `.git` file (not directory)
2. Treats the worktree root as the repository root
3. Never discovers configuration in a parent container directory

### 2.2 Known Tool Behaviors

#### Claude Code

- Looks for `.claude/` in the working directory
- Walks up to git root (stops at `.git` file/directory)
- User-level fallback to `~/.claude/`
- **Worktree limitation:** Does not traverse beyond worktree root

#### Cursor

- Uses `.cursor/` and `.cursorrules` files
- Similar directory traversal pattern
- **Worktree limitation:** Configuration isolation per worktree

#### VS Code / Copilot

- Uses workspace-level and user-level settings
- `.vscode/` folder per workspace
- **Worktree limitation:** Each worktree is treated as separate workspace

### 2.3 Configuration Discovery Implications

For agentic tools to share configuration across worktrees, one of these must happen:

1. **Tool modification:** Tools must be updated to traverse beyond `.git` files
2. **Symlinks:** Configuration symlinked from each worktree to shared location
3. **Container pattern:** Configuration at container level that tools can discover
4. **Environment variables:** Tools respect env vars pointing to shared config

---

## 3. Candidate Solution Analysis

### 3.1 Solution A: Centralized Git Database

```text
container/
├── .git/                    # Centralized git database (bare-like)
├── .agentic/                # Shared agentic configuration
│   ├── claude/
│   │   ├── rules/
│   │   └── settings.json
│   ├── cursor/
│   │   └── .cursorrules
│   └── shared/
│       ├── coding-standards.md
│       └── context.md
├── main/                    # Main branch worktree
│   ├── .git                 # File: gitdir: ../../../.git/worktrees/main
│   └── (source code)
├── feature-a/               # Feature branch worktree
│   ├── .git
│   └── (source code)
├── feature-b/
    └── ...
```

#### Solution A: Implementation Concept

The implementation involves cloning the repository with a bare-like structure in the container's `.git` directory and using `git worktree add` to create individual workspace directories for each branch.

#### Solution A: Pros

| Advantage                  | Description                                 |
| :------------------------- | :------------------------------------------ |
| **Single source of truth** | All configuration in one `.agentic/` folder |
| **Native git structure**   | Uses standard git worktree mechanics        |
| **Easy navigation**        | All worktrees siblings in container         |
| **Atomic updates**         | Config changes immediately available        |
| **Version control ready**  | `.agentic/` can be committed to a branch    |

#### Solution A: Cons

| Disadvantage           | Description                                                |
| :--------------------- | :--------------------------------------------------------- |
| **Tool discovery**     | Most tools won't traverse up from worktree to find config  |
| **Manual setup**       | Requires intentional container structure                   |
| **Not cloneable**      | Container structure isn't part of the git history          |
| **Unfamiliar pattern** | Developers expect to clone directly into working directory |

#### Solution A: Edge Cases

1. **Moving worktrees:** If a worktree is moved, run `git worktree repair`
2. **Nested repositories:** Tools may misinterpret `.git` file as root
3. **IDE integration:** IDEs may not recognize the container as a workspace
4. **Submodule interaction:** Submodules within worktrees work normally
5. **Shallow clones:** Compatible, but limited history in worktrees

#### Solution A: Mitigation Strategies

Potential mitigations for tool discovery issues include using symlinks to link the shared configuration into each worktree or utilizing environment variables if supported by the specific agentic tool.

---

### 3.2 Solution B: Orphaned Utility Branch

In this pattern, configuration lives on a dedicated "orphaned" git branch that has no common history with the code branches. This branch is checked out into the container root, while development work happens in worktrees inside a subdirectory.

#### Solution B: Implementation Concept

Setting up this pattern requires creating an orphan branch with its own file structure and then managing worktrees within a dedicated `worktrees/` directory to avoid overlap with the configuration branch files.

#### Solution B: Pros

| Advantage                | Description                               |
| :----------------------- | :---------------------------------------- |
| **Version controlled**   | Configuration lives in git history        |
| **Cloneable**            | Orphan branch can be checked out on clone |
| **Explicit structure**   | Clear separation of config vs code        |
| **Branch-based updates** | Can PR changes to agentic config          |
| **Team shareable**       | Push orphan branch to share config        |

#### Solution B: Cons

| Disadvantage            | Description                                                |
| :---------------------- | :--------------------------------------------------------- |
| **Complexity**          | Orphan branches are unfamiliar to many developers          |
| **Merge conflicts**     | Switching between orphan and code branches is disorienting |
| **Tool support**        | Still requires symlinks or tool modifications              |
| **Dual history**        | Orphan branch creates disconnected commit history          |
| **CI/CD complications** | Build systems may struggle with orphan branches            |

#### Solution B: Edge Cases

1. **Accidental checkout:** Developer accidentally checks out orphan branch, loses work context
2. **Branch deletion:** Orphan branch could be accidentally deleted
3. **Merge mistakes:** Cannot merge orphan branch into code branches (different roots)
4. **Rebase confusion:** Rebasing doesn't work across disconnected histories
5. **Shallow clone issues:** `--depth=1` may not fetch orphan branch

#### Solution B: Mitigation Strategies

To manage the complexity of orphan branches, teams can use protected branch settings and custom refspecs to ensure the configuration branch is handled correctly during fetch and checkout operations.

---

### 3.3 Solution C: Hybrid/Submodule Approach

This approach separates configuration into its own repository, which is then managed either as a git submodule or as an independent sibling repository within a shared project container.

#### Solution C: Implementation Concept

Implementation can be achieved either by adding the configuration repository as a git submodule within the main project or by maintaining it as an independent repository within a shared project container.

#### Solution C: Pros

| Advantage                  | Description                                   |
| :------------------------- | :-------------------------------------------- |
| **Independent versioning** | Config and code evolve separately             |
| **Reusable**               | Same config repo across multiple projects     |
| **Team templates**         | Organization-wide agentic standards           |
| **Clear ownership**        | Config changes go through separate review     |
| **Tool agnostic**          | Works regardless of how tools discover config |

#### Solution C: Cons

| Disadvantage              | Description                                   |
| :------------------------ | :-------------------------------------------- |
| **Complexity**            | Multiple git operations required              |
| **Sync overhead**         | Must keep submodule updated across branches   |
| **Submodule pain points** | Detached HEAD, forgotten commits, etc.        |
| **Two repositories**      | More URLs, credentials, permissions to manage |
| **Clone complexity**      | `--recurse-submodules` required               |

#### Solution C: Edge Cases

1. **Submodule version drift:** Different branches may point to different config versions
2. **Detached HEAD in submodule:** Changes made in detached state can be lost
3. **CI/CD submodule fetch:** Build systems need explicit submodule initialization
4. **Nested worktrees:** Worktrees don't automatically include submodule checkouts
5. **Permission issues:** Submodule repo may have different access controls

#### Solution C: Mitigation Strategies

Submodule-specific challenges can be mitigated using automatic recursion settings for fetch and pull, and by pinning submodules to specific branches to simplify updates across many worktrees.

---

## 4. Technical Deep Dive

### 4.1 Git Internals: Worktree Reference Mechanism

#### The gitdir File

Each linked worktree has a corresponding entry in `$GIT_COMMON_DIR/worktrees/<name>/`:

```text
$GIT_COMMON_DIR/worktrees/feature-a/
├── HEAD          # Current commit for this worktree
├── ORIG_HEAD     # Previous HEAD (if applicable)
├── index         # Staging area for this worktree
├── gitdir        # Absolute path to worktree's .git file
├── locked        # Present if worktree is locked (contains reason)
└── config.worktree  # Worktree-specific config (if extensions.worktreeConfig)
```

The `gitdir` file contains the absolute path to the worktree directory, enabling bidirectional linking.

#### Path Resolution Algorithm

When Git resolves paths in a worktree:

```text
resolve_git_path(path):
    if path is per-worktree (HEAD, index, refs/worktree/*, etc.):
        return $GIT_DIR/path
    else:
        return $GIT_COMMON_DIR/path
```

### 4.2 Configuration File Discovery Algorithms

#### Typical Agentic Tool Algorithm

```python
def find_config(start_dir, config_names):
    """
    Standard directory-traversal config discovery.
    Most agentic tools use this pattern.
    """
    current = start_dir
    while current != root:
        for name in config_names:
            config_path = current / name
            if config_path.exists():
                return config_path

        # Stop at git boundary
        if (current / '.git').exists():  # File OR directory
            break

        current = current.parent

    # Fallback to user config
    return home / '.config' / tool_name / 'config'
```

#### Enhanced Algorithm for Worktree Support

```python
def find_config_worktree_aware(start_dir, config_names):
    """
    Enhanced discovery that traverses beyond worktree boundaries.
    """
    current = start_dir
    found_git = False

    while current != root:
        for name in config_names:
            config_path = current / name
            if config_path.exists():
                return config_path

        git_path = current / '.git'
        if git_path.exists():
            if git_path.is_file():
                # This is a worktree - continue searching up
                found_git = True
            else:
                # This is the main .git directory - stop
                break

        current = current.parent

    return home / '.config' / tool_name / 'config'
```

### 4.3 Symlink vs Gitfile Approaches

#### Symlink Approach

```bash
# Create symlinks in each worktree
cd worktree-a
ln -s ../.agentic/.claude .claude
```

**Pros:**

- Works with all tools without modification
- Transparent to git (symlinks are tracked)

**Cons:**

- Platform differences (Windows requires admin for symlinks)
- Must be recreated for each new worktree
- Clutters worktree with symlinks

#### Gitfile Approach (Custom Tool Support)

Some tools could be modified to read a `.tool-config` file similar to `.git`:

```text
# .claude-config file in worktree
configdir: /path/to/container/.agentic/claude
```

**Pros:**

- Single file instead of multiple symlinks
- Platform independent
- Explicit configuration

**Cons:**

- Requires tool modification
- Non-standard pattern
- Must be added to each worktree

---

## 5. Existing Patterns in the Wild

### 5.1 Monorepo Patterns

Large monorepos (Google, Meta, Microsoft) often use:

```text
monorepo/
├── .config/                 # Shared tooling configuration
├── tools/                   # Build and development tools
├── packages/
│   ├── app-a/
│   ├── app-b/
│   └── shared-lib/
└── .git/
```

**Relevance:** Configuration at root is discovered by tools in any subdirectory. Similar principle could apply to worktree containers.

### 5.2 Multi-Package Repository Patterns

Projects like Babel, Jest, and Lerna use:

```text
project/
├── lerna.json              # Root-level config
├── package.json            # Root package.json with workspaces
├── packages/
│   ├── package-a/
│   └── package-b/
└── .git/
```

**Relevance:** Tools like ESLint, TypeScript, and Jest implement "config inheritance" - looking up from subdirectories to find root config.

### 5.3 Sparse Checkout Patterns

Git sparse checkout (alternative to worktrees for large repos):

```bash
git clone --filter=blob:none --sparse <repo>
git sparse-checkout set src/feature-a
```

**Relevance:** Some teams use sparse checkout instead of worktrees, avoiding the multi-directory problem entirely.

### 5.4 VS Code Multi-Root Workspace Pattern

VS Code supports multi-root workspaces:

```json
// project.code-workspace
{
  "folders": [
    { "path": "worktrees/main" },
    { "path": "worktrees/feature-a" },
    { "path": ".agentic" }
  ],
  "settings": {
    "claude.configPath": "${workspaceFolder:2}"
  }
}
```

**Relevance:** Could provide a model for agentic tools to support multi-directory configurations.

---

## 6. Tool Maintainer Perspectives

### 6.1 Common Concerns

Based on tool documentation and community discussions:

1. **Security:** Configuration discovery must not traverse into untrusted directories
2. **Performance:** Traversing many directories slows startup
3. **Predictability:** Users expect consistent behavior
4. **Simplicity:** Complex discovery logic leads to debugging nightmares

### 6.2 Requested Features

Common feature requests related to worktrees:

- "Respect symlinks when finding config" (#frequent)
- "Allow environment variable to specify config path"
- "Support multiple config file locations"
- "Inherit config from parent directories beyond git root"

### 6.3 Implementation Barriers

- Breaking changes to existing behavior
- Cross-platform symlink handling
- Security implications of traversing outside repo
- Testing complexity for edge cases

---

## 7. Comparative Analysis

### 7.1 Solution Comparison Matrix

| Criterion               | Solution A (Centralized) | Solution B (Orphan Branch) | Solution C (Submodule) |
| :---------------------- | :----------------------- | :------------------------- | :--------------------- |
| **Setup complexity**    | Medium                   | High                       | High                   |
| **Clone simplicity**    | Low (manual setup)       | Medium (script needed)     | Medium (--recurse)     |
| **Version control**     | Optional                 | Built-in                   | Built-in               |
| **Team sharing**        | Manual                   | Git push                   | Git push               |
| **Tool compatibility**  | Requires symlinks        | Requires symlinks          | Requires symlinks      |
| **Maintenance burden**  | Low                      | Medium                     | High                   |
| **Familiar pattern**    | Somewhat                 | No                         | Somewhat               |
| **Cross-project reuse** | No                       | No                         | Yes                    |
| **Config isolation**    | None (shared)            | None (shared)              | Per-branch possible    |

### 7.2 Use Case Alignment

| Scenario                               | Relevant Solutions | Considerations                                               |
| :------------------------------------- | :----------------- | :----------------------------------------------------------- |
| Solo developer, single project         | Solution A         | Lower setup complexity, local-only configuration             |
| Team with standardized config          | Solution B         | Configuration lives in git history, shareable via push       |
| Organization with multiple projects    | Solution C         | Configuration can be reused across repositories              |
| Experimentation with different configs | Solution C         | Different branches can reference different submodule commits |
| CI/CD heavy workflow                   | Solution A         | Fewer dependencies for build systems to resolve              |
| Rapid feature branch creation          | Solution A         | No submodule initialization step required                    |

### 7.3 Risk Assessment

| Risk                 | Solution A        | Solution B        | Solution C                   |
| :------------------- | :---------------- | :---------------- | :--------------------------- |
| Config loss          | Low (local files) | Low (git tracked) | Low (separate repo)          |
| Sync issues          | N/A               | N/A               | High (submodule drift)       |
| Developer confusion  | Medium            | High              | Medium                       |
| Tool breakage        | Low               | Low               | Medium (submodule edge cases)|
| Migration difficulty | Low               | Medium            | High                         |

---

## 8. Implementation Considerations

### 8.1 Automation and Tooling

Successful adoption of these patterns typically requires some level of automation to streamline the creation of worktrees and the linking of shared configuration. Scripts can be developed to initialize the container structure, handle git cloning, and establish the necessary symlinks or environment variables.

### 8.2 Version-Controlled Configuration Options

teams should decide whether to track configuration within the same repository (e.g., via an orphan branch) or in an independent repository (e.g., via submodules) based on their requirements for cross-project reuse and organizational standards.

### 8.3 Long-Term Considerations

Factors that may affect solution viability over time:

1. **Documentation:** Container structures benefit from explanation for new team members
2. **Automation:** Setup scripts reduce manual steps for worktree + config linking
3. **Tool evolution:** Agentic tools may add improved worktree support
4. **Contribution opportunities:** Discovery algorithm improvements could be proposed to tool maintainers

### 8.4 Potential Tool Improvements

Features that would simplify worktree-based workflows if implemented by tool maintainers:

1. **Config path environment variables:** `CLAUDE_CONFIG_PATH`, `CURSOR_CONFIG_PATH`
2. **Worktree-aware discovery:** Optional traversal beyond `.git` files
3. **Config inheritance:** Similar to `.gitignore` inheritance patterns
4. **Multi-root support:** Explicit configuration for multi-directory setups

---

## 9. Appendix

### 9.1 Quick Reference: Git Worktree Commands

```bash
# Create worktree for existing branch
git worktree add <path> <branch>

# Create worktree with new branch
git worktree add -b <new-branch> <path> <start-point>

# List worktrees
git worktree list

# Remove worktree
git worktree remove <path>

# Repair worktree links after moving
git worktree repair

# Lock worktree (prevent pruning)
git worktree lock <path> --reason "description"

# Prune stale worktree info
git worktree prune
```

### 9.2 Configuration File Locations by Tool

| Tool        | Primary Config     | Secondary Config | User Config      |
| :---------- | :----------------- | :--------------- | :--------------- |
| Claude Code | `.claude/`         | `.claude.json`   | `~/.claude/`     |
| Cursor      | `.cursor/`         | `.cursorrules`   | `~/.cursor/`     |
| Windsurf    | `.windsurf/`       | -                | `~/.windsurf/`   |
| Continue    | `.continue/`       | `config.json`    | `~/.continue/`   |
| Copilot     | `.github/copilot/` | -                | VS Code settings |

#### Platform Considerations

Platform-specific limitations, such as symlink permission requirements on Windows or cross-filesystem constraints in WSL, should be evaluated when selecting a pattern. Teams may need to adapt their approach (e.g., using junctions on Windows) to ensure compatibility across different development environments.

---

## 10. References

1. Git Documentation: git-worktree - <https://git-scm.com/docs/git-worktree>
2. Git Documentation: git-submodule - <https://git-scm.com/book/en/v2/Git-Tools-Submodules>
3. Git Internals: Environment Variables - <https://git-scm.com/book/en/v2/Git-Internals-Environment-Variables>
4. Project Research Brief: `00-research-brief.md`

---

*Document Status: Complete*
*Next Steps: Test proposed structures with actual agentic tools*
*Related Documents: 00-research-brief.md, (pending) 04-implementation-guide.md*
