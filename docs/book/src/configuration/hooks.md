# Hooks

Hooks are shell commands that run automatically at specific points in the Repository Manager lifecycle. Use them to automate tasks that should happen alongside branch creation, deletion, or sync operations.

## Available Hook Events

| Event                  | When It Fires                              |
|------------------------|--------------------------------------------|
| `pre-branch-create`    | Before a new branch/worktree is created    |
| `post-branch-create`   | After a new branch/worktree is created     |
| `pre-branch-delete`    | Before a branch/worktree is deleted        |
| `post-branch-delete`   | After a branch/worktree is deleted         |
| `pre-sync`             | Before `repo sync` runs                    |
| `post-sync`            | After `repo sync` completes                |

## Managing Hooks

### List Configured Hooks

```bash
repo hooks list
```

### Add a Hook

```bash
repo hooks add <event> <command> [args...]
```

Examples:

```bash
# Install npm dependencies after creating a branch
repo hooks add post-branch-create npm install

# Run tests before syncing
repo hooks add pre-sync cargo test

# Notify a webhook after sync
repo hooks add post-sync curl -- -X POST https://example.com/webhook
```

### Remove a Hook

Removes all hooks registered for the given event:

```bash
repo hooks remove post-branch-create
```

## Hook Behavior

- Hooks run in the context of the relevant worktree directory (for branch events) or the project root (for sync events).
- If a `pre-*` hook exits with a non-zero status, the associated operation is aborted.
- `post-*` hooks run regardless of whether the operation succeeded or failed.
- Multiple hooks on the same event run in the order they were added.

## Example: Auto-installing Dependencies

When using worktrees, you may want each new worktree to have its dependencies installed automatically:

```bash
# Node.js project
repo hooks add post-branch-create npm install

# Python project
repo hooks add post-branch-create python -- -m venv .venv
```

## Example: Lint Before Sync

Prevent sync from writing files if a lint check fails:

```bash
repo hooks add pre-sync cargo clippy -- --deny warnings
```

If `cargo clippy` returns a non-zero exit code, `repo sync` will not run.
