# Git Hooks and Code Quality Integration

Pre-commit frameworks, formatters, linters, and AI integration patterns.

## Pre-commit Frameworks

### Comparison Matrix

| Framework | Language | Config | Parallelization | Remote Configs | Agentic Potential |
|-----------|----------|--------|-----------------|----------------|-------------------|
| pre-commit | Python | YAML | Yes | Yes (repos) | High |
| Husky | Node.js | Shell | Via lint-staged | No | Medium |
| Lefthook | Go | YAML | Yes | Yes | High |
| Rusty-hook | Rust | TOML | No | No | Low |

### pre-commit (Python)

**Best for**: Multi-language projects, extensive hook ecosystem

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files

  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.13
    hooks:
      - id: ruff
        args: [--fix]
      - id: ruff-format

  - repo: https://github.com/pre-commit/mirrors-prettier
    rev: v3.1.0
    hooks:
      - id: prettier
```

**Install**: `pip install pre-commit && pre-commit install`

### Husky (Node.js)

**Best for**: JavaScript/TypeScript projects

```bash
# .husky/pre-commit
#!/bin/sh
npm run lint
npm run test
```

**Install**: `npm install -D husky && npx husky init`

### Lefthook (Go)

**Best for**: Performance-critical, multi-language

```yaml
# lefthook.yml
pre-commit:
  parallel: true
  commands:
    lint:
      glob: "*.{js,ts}"
      run: npx eslint {staged_files}
    format:
      glob: "*.{js,ts,json,md}"
      run: npx prettier --write {staged_files}
      stage_fixed: true
```

**Install**: `brew install lefthook && lefthook install`

## Formatters by Language

| Language | Formatter | Config File | Check | Fix |
|----------|-----------|-------------|-------|-----|
| JS/TS | Prettier | `.prettierrc` | `--check` | `--write` |
| Python | Ruff | `pyproject.toml` | `format --check` | `format` |
| Rust | rustfmt | `rustfmt.toml` | `--check` | (default) |
| Go | gofmt | None | `-d` | `-w` |
| C/C++ | clang-format | `.clang-format` | `--dry-run -Werror` | `-i` |

### Formatter Configs

Default configs available at each tool's documentation:
- Prettier: [prettier.io/docs/configuration](https://prettier.io/docs/en/configuration.html)
- Ruff: [docs.astral.sh/ruff/configuration](https://docs.astral.sh/ruff/configuration/)
- rustfmt: [rust-lang.github.io/rustfmt](https://rust-lang.github.io/rustfmt/)

## Linters by Language

| Language | Linter | Config | Auto-fix |
|----------|--------|--------|----------|
| JS/TS | ESLint | `eslint.config.js` | `--fix` |
| JS/TS | Biome | `biome.json` | `--apply` |
| Python | Ruff | `pyproject.toml` | `--fix` |
| Rust | Clippy | `clippy.toml` | `--fix` |
| Go | golangci-lint | `.golangci.yml` | `--fix` |

## AI Integration Patterns

### Pattern 1: AI Review Hook (Pre-push)

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: ai-review
        name: AI Code Review
        entry: ./scripts/ai-review.sh
        language: script
        stages: [pre-push]
        pass_filenames: false
```

```bash
# scripts/ai-review.sh
#!/bin/bash
DIFF=$(git diff origin/main...HEAD)
[ -z "$DIFF" ] && exit 0

AI_RESPONSE=$(echo "$DIFF" | claude --prompt "Review for security and best practices. Output JSON with 'pass': boolean")

PASS=$(echo "$AI_RESPONSE" | jq -r '.pass')
[ "$PASS" != "true" ] && echo "AI Review Failed" && exit 1
exit 0
```

### Pattern 2: AI Commit Message

```yaml
# lefthook.yml
prepare-commit-msg:
  commands:
    ai-message:
      run: |
        if [ -z "$2" ]; then
          DIFF=$(git diff --cached)
          claude --prompt "Generate conventional commit message" <<< "$DIFF" > "$1"
        fi
```

### Pattern 3: AI Security Scan

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: ai-security
        name: AI Security Analysis
        entry: python scripts/ai_security.py
        language: python
        stages: [pre-push]
```

## Mapping Hooks to AI Instructions

| Pre-commit Hook | AI Rule Equivalent |
|-----------------|-------------------|
| `prettier --check` | "Ensure code is formatted with Prettier" |
| `eslint --fix` | "Fix ESLint violations automatically" |
| `cargo clippy -D warnings` | "Address all Clippy warnings" |
| `no-commit-to-branch` | "Never commit directly to main" |
| `detect-secrets` | "Never commit secrets or API keys" |

## Complete Example

**`.pre-commit-config.yaml`**:
```yaml
default_install_hook_types: [pre-commit, pre-push, commit-msg]

repos:
  # Basic checks
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: detect-private-key

  # JavaScript/TypeScript
  - repo: https://github.com/pre-commit/mirrors-prettier
    rev: v3.1.0
    hooks:
      - id: prettier

  - repo: https://github.com/pre-commit/mirrors-eslint
    rev: v8.56.0
    hooks:
      - id: eslint
        args: ['--fix']

  # Python
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.13
    hooks:
      - id: ruff
        args: ['--fix']
      - id: ruff-format

  # Commit messages
  - repo: https://github.com/commitizen-tools/commitizen
    rev: v3.13.0
    hooks:
      - id: commitizen
        stages: [commit-msg]

  # AI integration (pre-push)
  - repo: local
    hooks:
      - id: ai-review
        name: AI Code Review
        entry: ./scripts/ai-review.sh
        language: script
        stages: [pre-push]
        pass_filenames: false
```

**Corresponding `CLAUDE.md`**:
```markdown
## Code Quality

This project enforces strict code quality through automated tools.

### Formatting
- **JavaScript/TypeScript:** Prettier
- **Python:** Ruff formatter

Run `pre-commit run --all-files` to format everything.

### Linting
- **JavaScript/TypeScript:** ESLint
- **Python:** Ruff linter

### Commit Messages
Use conventional commit format:
- `feat(scope): description`
- `fix(scope): description`
- `docs(scope): description`

### Pre-push
Before pushing, code undergoes AI review. Do NOT force push to bypass.
```

## Workflow

```
Developer writes code
         |
    git add .
         |
    git commit
         |
    [pre-commit hooks]
    ├── Format check
    ├── Lint check
    └── Basic validation
         |
    Commit created
         |
    git push
         |
    [pre-push hooks]
    ├── Full test suite
    ├── AI code review
    └── Security scan
         |
    Push to remote
```

---

*Last updated: 2026-01-23*
*Status: Complete*
