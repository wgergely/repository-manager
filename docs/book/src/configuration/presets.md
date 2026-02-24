# Presets

Presets are named bundles of rules and settings for a specific language or stack. Applying a preset populates your project with sensible defaults without you having to write every rule from scratch.

## Built-in Presets

Repository Manager ships with built-in presets for common environments:

| Preset   | Description                                        |
|----------|----------------------------------------------------|
| `rust`   | Rust coding guidelines, error handling conventions |
| `python` | Python style (PEP 8), type hints, project layout   |
| `node`   | Node.js/TypeScript conventions, package management |

## Applying a Preset

```bash
repo add-preset rust
```

This records `rust` in the `[presets]` section of `config.toml` and makes its rules available during the next sync.

## Removing a Preset

```bash
repo remove-preset rust
```

This removes the preset from `config.toml`. Rules contributed by the preset will no longer appear in generated tool configs after the next sync.

## Listing Available Presets

```bash
repo list-presets
```

## How Presets Work

A preset contributes a set of rules and can also adjust tool-specific settings. When you run `repo sync` with a preset active:

1. The preset's rules are merged with your project's own rules.
2. The combined rule set is injected into each tool's configuration file.
3. Preset rules are tagged with the preset name so they can be identified and removed cleanly if the preset is later disabled.

Preset rules follow the same Markdown format as project-level rules and appear in `.repository/rules/` (or are loaded from the preset bundle directly).

## Multiple Presets

You can apply more than one preset to a project:

```toml
[presets]
enabled = ["rust", "node"]
```

Or via CLI:

```bash
repo add-preset rust
repo add-preset node
```

When multiple presets are active, their rules are merged. If two presets provide conflicting guidance, you can override either by adding your own rule with a higher priority.

## Preset Priority

Rules contributed by presets have a default priority. Your project-level rules are applied after preset rules, so your project's instructions take precedence. You can also set an explicit priority in a rule's frontmatter:

```markdown
---
id: my-rule
priority: 80
---

My rule content here.
```

Higher numbers are applied later (and therefore take precedence in most tools).
