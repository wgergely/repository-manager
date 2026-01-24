# Git Worktree Patterns for Agentic Development

Patterns for using git worktrees with shared agentic tool configuration.

## The Problem

When using git worktrees, agentic tools (Claude Code, Cursor, etc.) treat each worktree as an isolated repository. Configuration in a parent container directory is not discovered.

```
container/
├── .agentic/           # Shared config - NOT FOUND by tools
├── main/               # Worktree
│   └── .git            # File pointing to ../git
├── feature-a/          # Worktree
│   └── .git
└── feature-b/          # Worktree
    └── .git
```

## Git Worktree Internals

### Structure

```
main-repo/
├── .git/                           # Main git database
│   ├── objects/                    # Shared object store
│   ├── refs/                       # Shared refs
│   └── worktrees/                  # Per-worktree data
│       ├── feature-a/
│       │   ├── HEAD
│       │   ├── index
│       │   └── gitdir
│       └── feature-b/
│           └── ...
└── (main branch files)

/path/to/feature-a/
├── .git                            # FILE containing: gitdir: /main-repo/.git/worktrees/feature-a
└── (feature-a files)
```

### Key Concepts

| Variable | Points To | Purpose |
|----------|-----------|---------|
| `$GIT_DIR` | Worktree's private dir | Per-worktree state |
| `$GIT_COMMON_DIR` | Main .git | Shared resources |

### What's Shared vs Isolated

| Resource | Shared | Isolated |
|----------|--------|----------|
| Object store | Yes | - |
| Refs (branches/tags) | Yes | - |
| HEAD | - | Yes |
| Index (staging) | - | Yes |
| Config (with worktreeConfig) | Base yes, overrides no | - |

## Tool Discovery Behavior

Most tools stop at `.git` (file or directory):

```python
# Typical discovery algorithm
def find_config(start_dir):
    current = start_dir
    while current != root:
        if config_exists(current):
            return config
        if (current / '.git').exists():  # Stops here!
            break
        current = current.parent
```

**Problem**: Worktrees have `.git` files, so tools stop there and never see the container.

## Solution Patterns

### Pattern A: Centralized Container

```
container/
├── .git/                    # Centralized database (bare-like)
├── .agentic/                # Shared configuration
│   ├── claude/rules/
│   ├── cursor/.cursorrules
│   └── shared/coding-standards.md
├── main/                    # git worktree add main main
│   ├── .git                 # File pointing up
│   └── (source)
└── feature-x/               # git worktree add feature-x feature-x
    ├── .git
    └── (source)
```

**Pros**: Single source of truth, native git structure
**Cons**: Tools don't discover config, requires symlinks

### Pattern B: Symlink Configuration

```bash
# In each worktree, symlink to shared config
cd feature-x
ln -s ../.agentic/.claude .claude
ln -s ../.agentic/.cursorrules .cursorrules
```

**Pros**: Works with all tools, transparent
**Cons**: Must repeat for each worktree, Windows symlink issues

### Pattern C: Orphan Branch

Configuration on a dedicated orphan branch:

```bash
git checkout --orphan agentic-config
# Add config files
git commit -m "Agentic configuration"

# Main worktree stays on code branch
# Config branch provides shared settings
```

**Pros**: Version controlled, shareable via push
**Cons**: Complex mental model, checkout confusion risk

### Pattern D: Submodule

Shared config as a separate repository:

```
project/
├── .agentic/              # git submodule
│   └── (shared configs)
├── src/
└── .git/
```

**Pros**: Cross-project reuse, independent versioning
**Cons**: Submodule complexity, sync overhead

## Recommended Approach

For repo-manager, **Pattern A + B** (Centralized + Symlinks):

```
container/
├── .agentic/                    # Source of truth
│   ├── rules/common.md
│   ├── claude/settings.json
│   ├── cursor/.cursorrules
│   └── sync.yaml                # Defines what syncs where
├── main/
│   ├── .claude -> ../.agentic/claude
│   ├── .cursorrules -> ../.agentic/cursor/.cursorrules
│   └── (source)
└── feature-x/
    ├── .claude -> ../.agentic/claude
    ├── .cursorrules -> ../.agentic/cursor/.cursorrules
    └── (source)
```

**repo-manager responsibilities**:
1. Initialize container structure
2. Create worktrees with correct symlinks
3. Keep symlinks in sync when config changes
4. Handle Windows junction points

## Automation Script

```bash
#!/bin/bash
# repo-manager worktree add <branch> <path>

BRANCH=$1
PATH=$2
CONTAINER=$(dirname $(git rev-parse --git-common-dir))

# Create worktree
git worktree add "$PATH" "$BRANCH"

# Create symlinks to shared config
cd "$PATH"
ln -sf "../.agentic/.claude" ".claude"
ln -sf "../.agentic/.cursorrules" ".cursorrules"
ln -sf "../.agentic/CLAUDE.md" "CLAUDE.md"

echo "Worktree created with agentic config linked"
```

## Platform Considerations

### Windows

- Symlinks require admin or Developer Mode
- Use junctions for directories: `mklink /J .claude ..\.agentic\.claude`
- Consider hard links for files

### WSL

- Cross-filesystem symlinks may not work
- Keep container on same filesystem

### macOS/Linux

- Standard symlinks work
- No special permissions needed

## Tool-Specific Notes

### Claude Code
- Follows symlinks for `.claude/` and `CLAUDE.md`
- Hierarchical rules still work within symlinked dirs

### Cursor
- Follows symlinks for `.cursorrules`
- May need IDE restart after symlink creation

### Windsurf
- Follows symlinks for `.windsurf/`
- Cascade memory is per-workspace (not synced)

## Quick Commands

```bash
# Create worktree
git worktree add ../feature-x feature-x

# List worktrees
git worktree list

# Remove worktree
git worktree remove ../feature-x

# Repair broken links
git worktree repair
```

---

*Last updated: 2026-01-23*
*Status: Complete*
