# Git Hooks and Code Standards Ecosystem

*Research Document: Git Hooks and Quality Tooling*
*Date: 2026-01-23*

## Overview

This document catalogs the git hooks ecosystem and code quality tooling landscape, including pre-commit frameworks, code formatters, linters, and integration patterns with AI-driven development workflows.

Pre-commit frameworks offer standardized configuration, language-agnostic invocation, and established patterns for code quality enforcement. These characteristics make them relevant to agentic tool integration.

---

## 1. Pre-commit Frameworks Comparison

### 1.1 pre-commit (Python)

**Repository:** https://github.com/pre-commit/pre-commit
**Language:** Python
**Maturity:** Production-ready (2014+)

#### Configuration Format

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

  - repo: https://github.com/psf/black
    rev: 24.1.0
    hooks:
      - id: black
        language_version: python3.11

  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.13
    hooks:
      - id: ruff
        args: [--fix]

  - repo: local
    hooks:
      - id: custom-check
        name: Custom validation
        entry: ./scripts/validate.sh
        language: script
        types: [python]
```

#### Features

| Feature | Support | Notes |
|---------|---------|-------|
| Multi-language | Excellent | Language auto-detection, isolated environments |
| Hook sources | Remote repos, local scripts | Versioned dependencies |
| Caching | Yes | Cached hook installations |
| Parallel execution | Yes | Configurable |
| CI/CD integration | Excellent | `pre-commit run --all-files` |
| Auto-fix | Yes | Hooks can modify files |
| Staged files only | Default | Configurable with `--all-files` |

#### Extensibility

**Custom Hook Definition:**
```yaml
# In .pre-commit-hooks.yaml of a hook repository
- id: my-hook
  name: My Custom Hook
  entry: my-script
  language: python
  types: [python]
  require_serial: false
  minimum_pre_commit_version: '2.0.0'
```

**Supported Languages:**
- python, node, ruby, rust, golang, docker, dotnet, lua, perl, swift
- `script` (shell scripts), `system` (system-installed commands)
- `fail` (always fail), `pygrep`, `docker_image`

#### Strengths for Agentic Integration

1. **YAML configuration:** Easily parseable and generatable
2. **Isolated environments:** Each hook runs in its own environment
3. **Version pinning:** Reproducible hook execution
4. **Hook metadata:** Rich schema for describing hooks
5. **Local hooks:** Can invoke AI tools as local scripts

---

### 1.2 Husky (Node.js)

**Repository:** https://github.com/typicode/husky
**Language:** Node.js
**Maturity:** Production-ready (2015+)

#### Configuration Format

**Modern Setup (v9+):**
```bash
# Install
npm install --save-dev husky

# Initialize
npx husky init
```

Creates `.husky/` directory:
```
.husky/
├── _/
│   ├── husky.sh
│   └── h
├── pre-commit
├── commit-msg
└── pre-push
```

**Hook Script (`.husky/pre-commit`):**
```bash
#!/bin/sh
npm run lint
npm run test
```

**Integration with lint-staged:**
```json
// package.json
{
  "lint-staged": {
    "*.{js,jsx,ts,tsx}": [
      "eslint --fix",
      "prettier --write"
    ],
    "*.{json,md}": "prettier --write"
  },
  "scripts": {
    "prepare": "husky install"
  }
}
```

#### Features

| Feature | Support | Notes |
|---------|---------|-------|
| Multi-language | Limited | Primarily JS/TS ecosystem |
| Hook sources | Local only | Shell scripts in `.husky/` |
| Caching | No | Relies on npm scripts |
| Parallel execution | Via lint-staged | External dependency |
| CI/CD integration | Good | `CI=true` skips hooks |
| Auto-fix | Via tools | Not built-in |
| Zero-dependency | v9+ | No runtime dependencies |

#### Extensibility

**Custom Hook:**
```bash
# .husky/pre-commit
#!/bin/sh

# Run linting
npm run lint

# Run AI review
npx ai-code-review --staged

# Custom validation
./scripts/validate-architecture.sh
```

#### Strengths for Agentic Integration

1. **Simple shell scripts:** Easy to invoke any tool
2. **lint-staged integration:** Efficient staged-file processing
3. **Zero dependencies (v9):** Minimal attack surface
4. **npm ecosystem integration:** Seamless with Node.js projects

---

### 1.3 Lefthook (Go)

**Repository:** https://github.com/evilmartians/lefthook
**Language:** Go
**Maturity:** Production-ready (2019+)

#### Configuration Format

```yaml
# lefthook.yml
pre-commit:
  parallel: true
  commands:
    lint:
      glob: "*.{js,ts,jsx,tsx}"
      run: npx eslint {staged_files}

    format:
      glob: "*.{js,ts,jsx,tsx,json,md}"
      run: npx prettier --write {staged_files}
      stage_fixed: true

    test:
      glob: "*.{js,ts}"
      run: npm test -- --findRelatedTests {staged_files}

  scripts:
    "ai-review":
      runner: bash

commit-msg:
  commands:
    check:
      run: npx commitlint --edit {1}

pre-push:
  parallel: true
  commands:
    audit:
      run: npm audit
    typecheck:
      run: npx tsc --noEmit
```

**Scripts (`.lefthook/pre-commit/ai-review.sh`):**
```bash
#!/bin/bash
# Invoke AI review for staged changes
claude-code-cli review --staged
```

#### Features

| Feature | Support | Notes |
|---------|---------|-------|
| Multi-language | Excellent | Single binary, no runtime needed |
| Hook sources | Local configs | Scripts or inline commands |
| Caching | No | Fast enough without caching |
| Parallel execution | Yes | Built-in parallelization |
| CI/CD integration | Excellent | `lefthook run pre-commit` |
| Auto-fix | Yes | `stage_fixed: true` |
| Performance | Excellent | Go binary, fast startup |

#### Extensibility

**Advanced Configuration:**
```yaml
# lefthook.yml
pre-commit:
  commands:
    ai-review:
      run: |
        if [ -n "$CLAUDE_API_KEY" ]; then
          npx claude-review {staged_files}
        fi
      env:
        CLAUDE_REVIEW_LEVEL: "thorough"
      fail_text: "AI review found issues"
      interactive: false

# Extend from remote
remotes:
  - git_url: https://github.com/org/shared-hooks
    ref: main
    config: lefthook-partial.yml
```

#### Strengths for Agentic Integration

1. **Rich templating:** `{staged_files}`, `{all_files}`, `{push_files}`
2. **Environment variables:** Easy configuration injection
3. **Remote configs:** Shared hook definitions across repos
4. **Parallel by default:** Fast execution
5. **No runtime dependencies:** Single Go binary

---

### 1.4 Rusty-hook (Rust)

**Repository:** https://github.com/AaronO/rusty-hook
**Language:** Rust
**Maturity:** Stable but less active

#### Configuration Format

```toml
# .rusty-hook.toml
[hooks.pre-commit]
command = "cargo fmt -- --check && cargo clippy"
timeout = 30

[hooks.commit-msg]
command = "./scripts/check-commit-message.sh"

[hooks.pre-push]
command = "cargo test"
timeout = 300
```

#### Features

| Feature | Support | Notes |
|---------|---------|-------|
| Multi-language | Limited | Any command, but Rust-focused |
| Hook sources | Local only | Commands in TOML |
| Caching | No | Simple command execution |
| Parallel execution | No | Sequential by default |
| CI/CD integration | Basic | Manual script invocation |
| Performance | Excellent | Rust binary |

#### Limitations for Agentic Integration

- Less active development
- Simpler feature set compared to alternatives
- Limited templating and file selection

---

### 1.5 Framework Comparison Matrix

| Criterion | pre-commit | Husky | Lefthook | Rusty-hook |
|-----------|------------|-------|----------|------------|
| **Config Format** | YAML | Shell + JSON | YAML | TOML |
| **Language Support** | Excellent | Node.js focus | Excellent | Limited |
| **Hook Isolation** | Yes | No | No | No |
| **Version Pinning** | Yes | No | Partial | No |
| **Parallelization** | Yes | Via lint-staged | Yes | No |
| **Remote Configs** | Yes (repos) | No | Yes | No |
| **Startup Speed** | Medium | Fast | Very Fast | Very Fast |
| **Ecosystem** | Large | Large (npm) | Growing | Small |
| **Agentic Potential** | High | Medium | High | Low |

**Agentic Integration Notes:** `pre-commit` offers maximum flexibility and the largest ecosystem. `Lefthook` prioritizes performance and supports remote configuration sharing.

---

### 1.6 Other Noteworthy Tools

While `pre-commit`, `Husky`, and `Lefthook` are the most dominant players, several other tools are worth mentioning for specific ecosystems or novel approaches:

- **Hk (Rust):** A newer hook manager written in Rust that emphasizes performance and parallelism. It notably uses the `pkl` language for configuration, offering a more structured and programmable alternative to YAML.
- **cargo-husky (Rust):** Inspired by Husky, this tool is tailored specifically for the Rust ecosystem and integrates with `Cargo.toml`. It's a lightweight option for Rust-centric projects.
- **githook manager (Go):** A simple, interactive hook manager written in Go that provides a straightforward setup for managing local hook scripts.

These tools highlight a trend towards high-performance, compiled hook managers and language-specific solutions that provide tight integration with the project's ecosystem.

---

## 2. Code Formatters (Cross-Language)

### 2.1 Formatter Landscape

#### Prettier (JS/TS/CSS/HTML/MD/JSON/YAML)

**Configuration:**
```json
// .prettierrc
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100
}
```

**CLI Invocation:**
```bash
# Format
npx prettier --write "src/**/*.{js,ts,json,md}"

# Check
npx prettier --check "src/**/*.{js,ts,json,md}"

# Staged files
npx prettier --write $(git diff --cached --name-only --diff-filter=ACMR | grep -E '\.(js|ts|json|md)$')
```

#### Black/Ruff (Python)

**Black Configuration:**
```toml
# pyproject.toml
[tool.black]
line-length = 100
target-version = ['py311']
include = '\.pyi?$'
extend-exclude = '''
/(
  \.eggs
  | \.git
  | build
  | dist
)/
'''
```

**Ruff Configuration (includes formatting in v0.1.2+):**
```toml
# pyproject.toml
[tool.ruff]
line-length = 100
target-version = "py311"

[tool.ruff.format]
quote-style = "double"
indent-style = "space"
```

**CLI Invocation:**
```bash
# Black
black src/ tests/
black --check --diff src/

# Ruff format
ruff format src/ tests/
ruff format --check src/
```

#### rustfmt (Rust)

**Configuration:**
```toml
# rustfmt.toml
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
```

**CLI Invocation:**
```bash
# Format
cargo fmt

# Check
cargo fmt -- --check

# Specific files
rustfmt src/main.rs src/lib.rs
```

#### gofmt/goimports (Go)

**No configuration file** - Go enforces a single style.

**CLI Invocation:**
```bash
# Format
gofmt -w .
go fmt ./...

# With import organization
goimports -w .

# Check
gofmt -d . | grep -q '^' && echo "Needs formatting"
```

#### clang-format (C/C++/Java/etc.)

**Configuration:**
```yaml
# .clang-format
BasedOnStyle: Google
IndentWidth: 4
ColumnLimit: 100
BreakBeforeBraces: Attach
AllowShortFunctionsOnASingleLine: Inline
```

**CLI Invocation:**
```bash
# Format
clang-format -i src/*.cpp include/*.h

# Check
clang-format --dry-run -Werror src/*.cpp

# Format changed files
git diff --name-only --diff-filter=ACMR | grep -E '\.(cpp|h)$' | xargs clang-format -i
```

### 2.2 Uniform Invocation Pattern

**Common Interface Design:**
```bash
# Pattern: <formatter> [--check] [--write] <files/patterns>

formatter_invoke() {
    local tool=$1
    local mode=$2  # "check" or "write"
    shift 2
    local files="$@"

    case $tool in
        prettier)
            if [ "$mode" = "check" ]; then
                npx prettier --check $files
            else
                npx prettier --write $files
            fi
            ;;
        black)
            if [ "$mode" = "check" ]; then
                black --check --diff $files
            else
                black $files
            fi
            ;;
        ruff-format)
            if [ "$mode" = "check" ]; then
                ruff format --check $files
            else
                ruff format $files
            fi
            ;;
        rustfmt)
            if [ "$mode" = "check" ]; then
                rustfmt --check $files
            else
                rustfmt $files
            fi
            ;;
        gofmt)
            if [ "$mode" = "check" ]; then
                gofmt -d $files | grep -q '^' && exit 1
            else
                gofmt -w $files
            fi
            ;;
        clang-format)
            if [ "$mode" = "check" ]; then
                clang-format --dry-run -Werror $files
            else
                clang-format -i $files
            fi
            ;;
    esac
}
```

**Pre-commit Integration (Unified):**
```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/mirrors-prettier
    rev: v3.1.0
    hooks:
      - id: prettier
        types_or: [javascript, jsx, ts, tsx, json, yaml, markdown]

  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.13
    hooks:
      - id: ruff-format

  - repo: https://github.com/doublify/pre-commit-rust
    rev: v1.0
    hooks:
      - id: fmt
        args: ["--", "--check"]

  - repo: https://github.com/dnephin/pre-commit-golang
    rev: v0.5.1
    hooks:
      - id: go-fmt

  - repo: https://github.com/pocc/pre-commit-hooks
    rev: v1.3.5
    hooks:
      - id: clang-format
```

### 2.3 Configuration Storage Patterns

| Formatter | Config File(s) | Location(s) | Format |
|-----------|---------------|-------------|--------|
| Prettier | `.prettierrc`, `prettier.config.js`, `package.json` | Project root | JSON/JS/YAML |
| Black | `pyproject.toml` | Project root | TOML |
| Ruff | `pyproject.toml`, `ruff.toml` | Project root | TOML |
| rustfmt | `rustfmt.toml`, `.rustfmt.toml` | Project root | TOML |
| gofmt | None | N/A | N/A |
| clang-format | `.clang-format` | Project root or parent dirs | YAML |

---

## 3. Linters

### 3.1 Linter Landscape

#### ESLint / Biome (JS/TS)

**ESLint (Established):**
```javascript
// eslint.config.js (Flat config - ESLint 9+)
import js from '@eslint/js';
import typescript from '@typescript-eslint/eslint-plugin';

export default [
  js.configs.recommended,
  {
    files: ['**/*.ts', '**/*.tsx'],
    plugins: { '@typescript-eslint': typescript },
    rules: {
      '@typescript-eslint/no-unused-vars': 'error',
      'no-console': 'warn'
    }
  }
];
```

**Biome (Emerging - Faster):**
```json
// biome.json
{
  "$schema": "https://biomejs.dev/schemas/1.5.0/schema.json",
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "suspicious": {
        "noExplicitAny": "error"
      }
    }
  },
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentWidth": 2
  }
}
```

**CLI:**
```bash
# ESLint
npx eslint . --fix
npx eslint . --max-warnings 0

# Biome
npx @biomejs/biome lint .
npx @biomejs/biome check . --apply  # lint + format
```

#### Clippy (Rust)

**Configuration:**
```toml
# clippy.toml
too_many_arguments_threshold = 10
cognitive_complexity_threshold = 30
```

**Cargo.toml lint configuration:**
```toml
[lints.clippy]
pedantic = "warn"
nursery = "warn"
unwrap_used = "deny"
```

**CLI:**
```bash
cargo clippy
cargo clippy -- -D warnings  # Deny all warnings
cargo clippy --fix           # Auto-fix where possible
```

#### golangci-lint (Go)

**Configuration:**
```yaml
# .golangci.yml
run:
  timeout: 5m

linters:
  enable:
    - gofmt
    - goimports
    - govet
    - errcheck
    - staticcheck
    - gosec
    - ineffassign
    - unused

linters-settings:
  govet:
    check-shadowing: true
  gocyclo:
    min-complexity: 15

issues:
  exclude-rules:
    - path: _test\.go
      linters:
        - errcheck
```

**CLI:**
```bash
golangci-lint run
golangci-lint run --fix
golangci-lint run --new-from-rev=HEAD~1  # Only new issues
```

### 3.2 Linter Integration Patterns

**Pre-commit Integration:**
```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/pre-commit/mirrors-eslint
    rev: v8.56.0
    hooks:
      - id: eslint
        additional_dependencies:
          - eslint@8.56.0
          - '@typescript-eslint/eslint-plugin@6.19.0'
        args: ['--fix']

  - repo: local
    hooks:
      - id: clippy
        name: Clippy
        entry: cargo clippy -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      - id: golangci-lint
        name: golangci-lint
        entry: golangci-lint run --new-from-rev=HEAD
        language: system
        types: [go]
        pass_filenames: false
```

### 3.3 Linter Output Formats

Most linters support standardized output formats for tooling integration:

```bash
# JSON output
eslint . --format json > lint-results.json
golangci-lint run --out-format json > lint-results.json

# SARIF (Static Analysis Results Interchange Format)
eslint . --format @microsoft/eslint-formatter-sarif
golangci-lint run --out-format sarif

# Checkstyle XML
eslint . --format checkstyle
clippy-sarif  # Community tool
```

---

## 4. Agentic Mandate Connection

### 4.1 "Format Before Commit" to AI Instructions Mapping

**Traditional Rule:**
```yaml
# .pre-commit-config.yaml
- id: prettier
  stages: [commit]
```

**Agentic Equivalent (CLAUDE.md):**
```markdown
## Code Formatting

Before committing any code changes:
1. Run `npx prettier --write` on modified files
2. Run the project linter: `npm run lint`
3. Fix any linting errors before committing
4. If auto-fix is available, apply it

Do NOT commit code that fails formatting or linting checks.
```

**Bidirectional Mapping Table:**

| Pre-commit Hook | AI Instruction Equivalent |
|-----------------|---------------------------|
| `prettier --check` | "Ensure code is formatted with Prettier" |
| `eslint --fix` | "Fix ESLint violations automatically when possible" |
| `cargo clippy -D warnings` | "Address all Clippy warnings before committing" |
| `check-yaml` | "Validate YAML files are syntactically correct" |
| `no-commit-to-branch` | "Never commit directly to main branch" |
| `detect-secrets` | "Never commit secrets or API keys" |

### 4.2 AI as Pre-commit Hook

**Pattern 1: AI Review Hook**
```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: ai-review
        name: AI Code Review
        entry: ./scripts/ai-review.sh
        language: script
        stages: [pre-push]  # Before push, not commit (slower)
        pass_filenames: false
```

**Script (`scripts/ai-review.sh`):**
```bash
#!/bin/bash
set -e

# Get diff for staged/push changes
DIFF=$(git diff origin/main...HEAD)

# Skip if no changes
[ -z "$DIFF" ] && exit 0

# Invoke AI review
AI_RESPONSE=$(echo "$DIFF" | claude --prompt "Review this code diff for:
- Security vulnerabilities
- Performance issues
- Best practice violations
- Logic errors

Output: JSON with 'pass': boolean, 'issues': array")

# Parse response
PASS=$(echo "$AI_RESPONSE" | jq -r '.pass')

if [ "$PASS" != "true" ]; then
    echo "AI Review Found Issues:"
    echo "$AI_RESPONSE" | jq -r '.issues[]'
    exit 1
fi

echo "AI Review: Passed"
```

**Pattern 2: AI-Assisted Commit Message**
```yaml
# lefthook.yml
prepare-commit-msg:
  commands:
    ai-commit-message:
      run: |
        if [ -z "$2" ]; then  # Only if no message provided
          DIFF=$(git diff --cached)
          SUGGESTION=$(echo "$DIFF" | claude --prompt "Generate conventional commit message")
          echo "$SUGGESTION" > "$1"
        fi
```

**Pattern 3: AI Security Scan**
```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: ai-security-scan
        name: AI Security Analysis
        entry: python scripts/ai_security_scan.py
        language: python
        types: [python, javascript, typescript]
        stages: [pre-push]
```

```python
# scripts/ai_security_scan.py
import subprocess
import json
import sys

def get_staged_content(files):
    content = {}
    for f in files:
        result = subprocess.run(['git', 'show', f':' + f], capture_output=True, text=True)
        content[f] = result.stdout
    return content

def ai_security_review(content):
    # Invoke AI API for security review
    # Return structured findings
    pass

if __name__ == "__main__":
    files = sys.argv[1:]
    content = get_staged_content(files)
    findings = ai_security_review(content)

    if findings['critical']:
        print("SECURITY: Critical issues found")
        for issue in findings['critical']:
            print(f"  - {issue['file']}: {issue['description']}")
        sys.exit(1)

    sys.exit(0)
```

### 4.3 "AI Review Before Push" Pattern

**Complete Workflow:**

```
Developer writes code
         |
         v
    git add .
         |
         v
    git commit
         |
    [pre-commit hooks]
         |-- Format check (fast)
         |-- Lint check (fast)
         |-- Unit tests (optional)
         |
         v
    Commit created
         |
         v
    git push
         |
    [pre-push hooks]
         |-- Full test suite
         |-- AI Code Review <-- NEW
         |-- Security scan
         |
         v
    Push to remote
```

**Lefthook Implementation:**
```yaml
# lefthook.yml
pre-commit:
  parallel: true
  commands:
    format:
      glob: "*.{js,ts,py,rs,go}"
      run: ./scripts/format-check.sh {staged_files}

    lint:
      glob: "*.{js,ts,py,rs,go}"
      run: ./scripts/lint-check.sh {staged_files}

pre-push:
  parallel: false  # Sequential for AI review
  commands:
    test:
      run: npm test

    ai-review:
      run: ./scripts/ai-review-push.sh
      env:
        AI_REVIEW_STRICTNESS: "standard"
      fail_text: |
        AI Review blocked this push.
        Please address the issues above before pushing.
```

### 4.4 Configuration Synchronization

**Agentic-Aware Pre-commit Config:**
```yaml
# .pre-commit-config.yaml
repos:
  # Standard formatting
  - repo: https://github.com/pre-commit/mirrors-prettier
    rev: v3.1.0
    hooks:
      - id: prettier
        # Must match CLAUDE.md formatting rules

  # Linting
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.13
    hooks:
      - id: ruff
        args: [--config, pyproject.toml]
        # Config also referenced in CLAUDE.md

  # AI Integration
  - repo: local
    hooks:
      - id: ai-consistency-check
        name: AI Rule Consistency
        entry: python scripts/check_ai_rules_sync.py
        language: python
        files: (CLAUDE\.md|\.cursorrules|\.pre-commit-config\.yaml)
        pass_filenames: false
```

**Synchronization Script:**
```python
# scripts/check_ai_rules_sync.py
"""
Ensures AI rules and pre-commit hooks are synchronized.
"""
import yaml
import re
from pathlib import Path

def extract_formatters_from_precommit():
    """Extract formatter tools from pre-commit config."""
    config = yaml.safe_load(Path('.pre-commit-config.yaml').read_text())
    formatters = []
    for repo in config.get('repos', []):
        for hook in repo.get('hooks', []):
            if 'format' in hook.get('id', '').lower() or 'prettier' in hook.get('id', ''):
                formatters.append(hook['id'])
    return formatters

def extract_formatters_from_claude_md():
    """Extract mentioned formatters from CLAUDE.md."""
    content = Path('CLAUDE.md').read_text()
    mentioned = []
    for tool in ['prettier', 'black', 'ruff', 'rustfmt', 'gofmt', 'clang-format']:
        if tool.lower() in content.lower():
            mentioned.append(tool)
    return mentioned

def check_sync():
    precommit_tools = set(extract_formatters_from_precommit())
    claude_tools = set(extract_formatters_from_claude_md())

    missing_in_claude = precommit_tools - claude_tools
    missing_in_precommit = claude_tools - precommit_tools

    if missing_in_claude or missing_in_precommit:
        print("WARNING: Tool configuration mismatch")
        if missing_in_claude:
            print(f"  In pre-commit but not CLAUDE.md: {missing_in_claude}")
        if missing_in_precommit:
            print(f"  In CLAUDE.md but not pre-commit: {missing_in_precommit}")
        return 1
    return 0

if __name__ == "__main__":
    exit(check_sync())
```

---

## 5. Configuration Management

### 5.1 Configuration Storage Patterns

#### Pre-commit Frameworks

| Framework | Config File | Format | Programmatic Generation |
|-----------|------------|--------|------------------------|
| pre-commit | `.pre-commit-config.yaml` | YAML | Easy - standard YAML |
| Husky | `.husky/*`, `package.json` | Shell + JSON | Medium - shell scripts |
| Lefthook | `lefthook.yml` | YAML | Easy - standard YAML |
| Rusty-hook | `.rusty-hook.toml` | TOML | Easy - standard TOML |

#### Formatters/Linters

| Tool | Config Files | Formats | Programmatic |
|------|-------------|---------|--------------|
| Prettier | `.prettierrc`, `prettier.config.js` | JSON/JS/YAML | Easy |
| ESLint | `eslint.config.js`, `.eslintrc.*` | JS/JSON/YAML | Easy |
| Biome | `biome.json` | JSON | Easy |
| Ruff | `pyproject.toml`, `ruff.toml` | TOML | Easy |
| Black | `pyproject.toml` | TOML | Easy |
| Clippy | `clippy.toml`, `Cargo.toml` | TOML | Easy |
| golangci-lint | `.golangci.yml` | YAML | Easy |
| clang-format | `.clang-format` | YAML | Easy |

### 5.2 Programmatic Configuration Generation

**Schema-Driven Generation:**
```python
# generate_precommit_config.py
"""
Generate pre-commit configuration from project metadata.
"""
import yaml
from pathlib import Path

def detect_languages():
    """Detect languages used in project."""
    languages = set()
    patterns = {
        'python': ['*.py'],
        'javascript': ['*.js', '*.jsx'],
        'typescript': ['*.ts', '*.tsx'],
        'rust': ['*.rs', 'Cargo.toml'],
        'go': ['*.go', 'go.mod'],
        'cpp': ['*.cpp', '*.hpp', '*.c', '*.h'],
    }
    for lang, globs in patterns.items():
        for glob in globs:
            if list(Path('.').rglob(glob)):
                languages.add(lang)
                break
    return languages

def generate_hooks(languages):
    """Generate hook configuration for detected languages."""
    hooks = {
        'repos': [
            {
                'repo': 'https://github.com/pre-commit/pre-commit-hooks',
                'rev': 'v4.5.0',
                'hooks': [
                    {'id': 'trailing-whitespace'},
                    {'id': 'end-of-file-fixer'},
                    {'id': 'check-yaml'},
                ]
            }
        ]
    }

    if 'python' in languages:
        hooks['repos'].append({
            'repo': 'https://github.com/astral-sh/ruff-pre-commit',
            'rev': 'v0.1.13',
            'hooks': [
                {'id': 'ruff', 'args': ['--fix']},
                {'id': 'ruff-format'}
            ]
        })

    if 'javascript' in languages or 'typescript' in languages:
        hooks['repos'].append({
            'repo': 'https://github.com/pre-commit/mirrors-prettier',
            'rev': 'v3.1.0',
            'hooks': [{'id': 'prettier'}]
        })

    # Add more language-specific hooks...

    return hooks

def main():
    languages = detect_languages()
    config = generate_hooks(languages)

    with open('.pre-commit-config.yaml', 'w') as f:
        yaml.dump(config, f, default_flow_style=False, sort_keys=False)

    print(f"Generated config for: {', '.join(languages)}")

if __name__ == "__main__":
    main()
```

### 5.3 Configuration Generation Patterns

Configurations can be generated programmatically from unified sources:

**Generator Implementation:**
```python
# .agentic/generate-configs.py
"""
Generate all quality and AI tool configurations from unified source.
"""
import yaml
import json
from pathlib import Path

def load_agentic_config():
    """Load unified configuration."""
    return {
        'hooks': yaml.safe_load(Path('.agentic/quality/hooks.yaml').read_text()),
        'formatting': yaml.safe_load(Path('.agentic/quality/formatting.yaml').read_text()),
        'linting': yaml.safe_load(Path('.agentic/quality/linting.yaml').read_text()),
        'rules': Path('.agentic/rules/coding-standards.md').read_text()
    }

def generate_precommit_config(config):
    """Generate .pre-commit-config.yaml."""
    # Transform unified config to pre-commit format
    pass

def generate_prettier_config(config):
    """Generate .prettierrc."""
    prettier_settings = config['formatting'].get('prettier', {})
    Path('.prettierrc').write_text(json.dumps(prettier_settings, indent=2))

def generate_claude_md(config):
    """Generate/update CLAUDE.md with coding standards."""
    template = f"""# Project Instructions

## Coding Standards

{config['rules']}

## Quality Tools

This project uses the following quality tools:
- Pre-commit hooks for automated checks
- See `.pre-commit-config.yaml` for hook configuration

Always run `pre-commit run --all-files` before committing.
"""
    Path('CLAUDE.md').write_text(template)

def main():
    config = load_agentic_config()
    generate_precommit_config(config)
    generate_prettier_config(config)
    generate_claude_md(config)
    print("Configuration files generated successfully")

if __name__ == "__main__":
    main()
```

---

## 6. Integration Patterns

### 6.1 Architecture Overview

Integration between agentic orchestrators and git hooks can involve:
- AI review hooks at pre-push stage
- Rule synchronization between hook configs and AI instruction files
- Configuration generation from unified sources

### 6.2 Sample Complete Configuration

**`.pre-commit-config.yaml` (Full Example):**
```yaml
# Pre-commit configuration with AI integration
default_install_hook_types: [pre-commit, pre-push, commit-msg]

repos:
  # Basic hooks
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-json
      - id: check-added-large-files
        args: ['--maxkb=1000']
      - id: detect-private-key
      - id: check-merge-conflict

  # JavaScript/TypeScript
  - repo: https://github.com/pre-commit/mirrors-prettier
    rev: v3.1.0
    hooks:
      - id: prettier
        types_or: [javascript, jsx, ts, tsx, json, yaml, markdown, css]

  - repo: https://github.com/pre-commit/mirrors-eslint
    rev: v8.56.0
    hooks:
      - id: eslint
        args: ['--fix']
        additional_dependencies:
          - eslint@8.56.0
          - '@typescript-eslint/eslint-plugin@6.19.0'
          - '@typescript-eslint/parser@6.19.0'

  # Python
  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.13
    hooks:
      - id: ruff
        args: ['--fix']
      - id: ruff-format

  # Rust (if applicable)
  - repo: local
    hooks:
      - id: cargo-fmt
        name: Cargo Format Check
        entry: cargo fmt -- --check
        language: system
        types: [rust]
        pass_filenames: false

      - id: clippy
        name: Clippy
        entry: cargo clippy -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

  # Go (if applicable)
  - repo: local
    hooks:
      - id: go-fmt
        name: Go Format
        entry: gofmt -w
        language: system
        types: [go]

      - id: golangci-lint
        name: golangci-lint
        entry: golangci-lint run
        language: system
        types: [go]
        pass_filenames: false

  # Commit message validation
  - repo: https://github.com/commitizen-tools/commitizen
    rev: v3.13.0
    hooks:
      - id: commitizen
        stages: [commit-msg]

  # AI Integration (Pre-push)
  - repo: local
    hooks:
      - id: ai-code-review
        name: AI Code Review
        entry: ./scripts/ai-review.sh
        language: script
        stages: [pre-push]
        pass_filenames: false
        verbose: true

      - id: ai-security-scan
        name: AI Security Scan
        entry: ./scripts/ai-security.sh
        language: script
        stages: [pre-push]
        pass_filenames: false

  # Configuration consistency
  - repo: local
    hooks:
      - id: config-sync-check
        name: Check AI/Hook Config Sync
        entry: python scripts/check_config_sync.py
        language: python
        files: (CLAUDE\.md|\.cursorrules|\.pre-commit-config\.yaml|pyproject\.toml)
        pass_filenames: false
```

**Corresponding `CLAUDE.md`:**
```markdown
# Project Instructions

## Code Quality Requirements

This project enforces strict code quality through automated tools and AI review.

### Formatting

All code must be formatted before committing:
- **JavaScript/TypeScript:** Prettier
- **Python:** Ruff formatter
- **Rust:** rustfmt
- **Go:** gofmt

Run `pre-commit run --all-files` to format everything.

### Linting

Code must pass linting without errors:
- **JavaScript/TypeScript:** ESLint with TypeScript plugin
- **Python:** Ruff linter
- **Rust:** Clippy (deny warnings)
- **Go:** golangci-lint

### Commit Messages

Use conventional commit format:
- `feat(scope): description` - New features
- `fix(scope): description` - Bug fixes
- `docs(scope): description` - Documentation
- `refactor(scope): description` - Code refactoring

### Pre-push Review

Before pushing, code undergoes:
1. Full test suite
2. AI code review for quality and security
3. Configuration consistency check

Do NOT force push to bypass these checks.

### Commands

```bash
# Run all quality checks
pre-commit run --all-files

# Run tests
npm test  # or cargo test, go test, pytest

# Format code
npm run format  # or equivalent for your language
```
```

---

## 7. Summary

### 7.1 Observations

1. **Pre-commit frameworks** provide standardized configuration, multi-language support, and extensibility for agentic tool integration.

2. **Formatters and linters** have converged on standard configuration formats (YAML, TOML, JSON), enabling programmatic generation and management.

3. **AI integration points** in the git workflow include:
   - Pre-commit: Fast checks, formatting assistance
   - Commit-msg: AI-generated commit messages
   - Pre-push: Thorough AI review, security scanning

4. **Configuration synchronization** between pre-commit hooks and AI rules affects behavioral consistency.

### 7.2 Open Questions

1. **MCP Integration:** Whether pre-commit hooks could be exposed as MCP tools for AI orchestrators
2. **Standardization:** Whether industry standards will emerge for AI-integrated quality workflows
3. **Performance:** Approaches to caching and parallelization for AI review steps
4. **Security:** Methods for sandboxing AI review to prevent code exfiltration

---

## Appendix A: Quick Reference

### Pre-commit Framework Commands

| Framework | Install | Initialize | Run | Add Hook |
|-----------|---------|------------|-----|----------|
| pre-commit | `pip install pre-commit` | `pre-commit install` | `pre-commit run` | Edit YAML |
| Husky | `npm install husky -D` | `npx husky init` | `git commit` | Edit shell script |
| Lefthook | `brew install lefthook` | `lefthook install` | `lefthook run` | Edit YAML |

### Formatter Quick Reference

| Language | Formatter | Config | Check | Fix |
|----------|-----------|--------|-------|-----|
| JS/TS | Prettier | `.prettierrc` | `--check` | `--write` |
| Python | Ruff | `pyproject.toml` | `format --check` | `format` |
| Rust | rustfmt | `rustfmt.toml` | `--check` | (default) |
| Go | gofmt | None | `-d` | `-w` |
| C/C++ | clang-format | `.clang-format` | `--dry-run -Werror` | `-i` |

### Linter Quick Reference

| Language | Linter | Config | Auto-fix |
|----------|--------|--------|----------|
| JS/TS | ESLint | `eslint.config.js` | `--fix` |
| JS/TS | Biome | `biome.json` | `--apply` |
| Python | Ruff | `pyproject.toml` | `--fix` |
| Rust | Clippy | `clippy.toml` | `--fix` |
| Go | golangci-lint | `.golangci.yml` | `--fix` |

---

*Document Status: Complete*
*Related Documents: 01-tool-configurations.md, 02-cross-platform-interop.md, 04-emerging-standards.md*
*Branch: research-docs*
