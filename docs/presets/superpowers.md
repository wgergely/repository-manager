# Superpowers Preset

The superpowers preset installs the [superpowers](https://github.com/obra/superpowers) Claude Code plugin, providing agentic skills for TDD, debugging, planning, and collaboration workflows.

## Installation

```bash
repo superpowers install
```

Or with a specific version:

```bash
repo superpowers install --version v4.1.1
```

## Usage

After installation, superpowers skills are available in Claude Code:

- `/superpowers:brainstorming` - Refine ideas through collaborative dialogue
- `/superpowers:writing-plans` - Create detailed implementation plans
- `/superpowers:test-driven-development` - Enforce TDD workflow
- `/superpowers:systematic-debugging` - Structured debugging methodology

## Status

Check installation status:

```bash
repo superpowers status
```

## Uninstall

```bash
repo superpowers uninstall
```

## How It Works

1. Clones superpowers from GitHub to `~/.claude/plugins/cache/git/superpowers/{version}/`
2. Enables the plugin in `~/.claude/settings.json`
3. Skills become available in Claude Code sessions

## Troubleshooting

### Plugin not showing in Claude Code

Ensure Claude Code is restarted after installation. The plugin is enabled via settings.json but requires a session restart.

### Network errors during install

The install requires network access to clone from GitHub. Ensure you have internet connectivity and can access github.com.
