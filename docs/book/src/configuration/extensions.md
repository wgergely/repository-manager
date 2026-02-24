# Extensions

Extensions let you package and distribute custom Repository Manager functionality — additional tools, rules, presets, or hooks — as a reusable unit that can be installed by name or URL.

## Installing an Extension

Install an extension from a Git URL:

```bash
repo extension install https://github.com/example/my-extension.git
```

Install from a local path:

```bash
repo extension install ./path/to/my-extension
```

By default, the extension is activated immediately after installation. To install without activating:

```bash
repo extension install https://github.com/example/my-extension.git --no-activate
```

## Adding a Known Extension

If an extension is registered by name in the known-extensions registry:

```bash
repo extension add my-extension
```

## Listing Extensions

Show all installed and known extensions:

```bash
repo extension list
```

For machine-readable output:

```bash
repo extension list --json
```

## Removing an Extension

```bash
repo extension remove my-extension
```

## Reinitializing an Extension

Re-run the post-install setup for an already-installed extension (useful if you want to re-apply its defaults after a configuration change):

```bash
repo extension reinit my-extension
```

## Creating an Extension

Scaffold a new extension:

```bash
repo extension init my-extension
```

This creates a directory named `my-extension` with the required structure. An extension can provide:

- **Rules** — Markdown rule files that are made available when the extension is active.
- **Presets** — Named preset bundles.
- **Hooks** — Shell commands that run at lifecycle events.
- **Custom tools** — Tool definitions for integrations not built into Repository Manager.

## Command Alias

`extension` can be shortened to `ext`:

```bash
repo ext list
repo ext install https://github.com/example/ext.git
```
