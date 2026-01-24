---
name: superpowers-git-worktrees
description: Use when starting feature work that needs isolation from current workspace or before executing implementation plans - creates isolated git worktrees with smart directory selection and verification
---

# Superpowers Git Worktrees

## Overview

Git worktrees create isolated workspaces sharing the same repository, allowing work on multiple branches simultaneously without switching.

**Core principle:** Systematic directory selection + safety verification = reliable isolation.

**Announce at start:** "I'm using the superpowers-git-worktrees skill to set up an isolated workspace."

## Directory Selection Process

Follow this priority order:

### 1. Check for Worktree Container (Sibling Pattern)

```python
# If parent folder ends with -worktrees or Worktrees → sibling pattern
parent = Path(repo_root).parent
if parent.name.endswith(('-worktrees', 'Worktrees', '-Worktrees')):
    # Worktrees are siblings in the container folder
    worktree_path = parent / branch_name
```

**Example:**
```
y:\code\MyProject-Worktrees\    ← container folder (detected)
├── main\                       ← main branch
├── feature-auth\               ← worktree sibling
├── feature-api\                ← worktree sibling
```

### 2. Check Existing Nested Directories

```powershell
# Check in priority order
Test-Path .worktrees   # Preferred (hidden)
Test-Path worktrees    # Alternative
```

**If found:** Use that directory (verify it's git-ignored first).

### 3. Default to Nested Pattern

If no container detected and no existing directory:
- Create `.worktrees/` inside the repository
- Add to `.gitignore` if not already ignored

## Safety Verification

### For Nested Directories (.worktrees or worktrees)

**MUST verify directory is ignored before creating worktree:**

```powershell
git check-ignore -q .worktrees
# Exit code 0 = ignored (safe)
# Exit code 1 = NOT ignored (fix first!)
```

**If NOT ignored:** Add to `.gitignore` and commit before proceeding.

### For Sibling Pattern (Container Folders)

No `.gitignore` verification needed - worktrees are outside the repository.

## Usage

Use the provided helper script to create and manage worktrees:

### Create Worktree

```powershell
python .agent/skills/superpowers-git-worktrees/scripts/manage_worktree.py create <branch-name>
```

This command will:

1. **Detect pattern**: Check if parent is a worktree container
2. **Find/create directory**: Use sibling or nested pattern
3. **Verify ignore**: Ensures nested directories are git-ignored
4. **Create worktree**: Runs `git worktree add`
5. **Setup project**: Runs `npm install`, `cargo build`, etc. based on detected files
6. **Verify baseline**: Runs tests to ensure a clean start

## Quick Reference

| Situation | Action |
|-----------|--------|
| Parent ends with `-Worktrees` | Use sibling pattern |
| `.worktrees/` exists | Use it (verify ignored) |
| `worktrees/` exists | Use it (verify ignored) |
| Neither exists | Create `.worktrees/` (add to .gitignore) |
| Directory not ignored | Add to .gitignore + commit |
| Tests fail during baseline | Report failures + ask |

## Common Mistakes

### Skipping ignore verification (nested pattern)
- **Problem:** Worktree contents get tracked, pollute git status
- **Fix:** Always use `git check-ignore` before creating nested worktree

### Assuming directory location
- **Problem:** Creates inconsistency, violates project conventions
- **Fix:** Follow priority: container → existing → create new

### Proceeding with failing tests
- **Problem:** Can't distinguish new bugs from pre-existing issues
- **Fix:** Report failures, get explicit permission to proceed

## Manual Fallback

If the script fails, you can do this manually:

### 1. Detect Pattern

```powershell
# Check if parent is a worktree container
$parent = (Get-Item ..).Name
if ($parent -match '(-[Ww]orktrees|Worktrees)$') {
    Write-Host "Sibling pattern detected"
}
```

### 2. Create Worktree

```powershell
# Sibling pattern
git worktree add ../feature-branch -b feature-branch

# Nested pattern
git worktree add .worktrees/feature-branch -b feature-branch
```

### 3. Setup & Verify

Run your project's install and test commands.

## Integration

**Called by:**
- **superpowers-brainstorm** (when design is approved)
- Any skill needing isolated workspace

**Pairs with:**
- **superpowers-finish** - Cleanup after work complete
- **superpowers-execute-plan** - Work happens in this worktree
