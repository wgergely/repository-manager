# Multi-Format Block System Design

## Overview

This document defines how managed blocks work across different file formats.
Each format requires a format-specific approach to marking and managing blocks.

## Core Principle

The rule UUID from the registry becomes the block marker in all formats,
enabling bidirectional traceability.

## Format-Specific Strategies

| Format | Block Approach | User Content | Managed Content |
|--------|---------------|--------------|-----------------|
| Markdown/HTML | XML comments | Preserved | Between `<!-- repo:block:UUID -->` markers |
| JSON | Reserved key | All other keys | Under `__repo_managed__[UUID]` |
| YAML | Comments | All content | Between `# repo:block:UUID` comment markers |
| TOML | Section | All other sections | Under `[repo_managed.UUID]` |
| JavaScript/CSS | Block comments | All other code | Between `/* repo:block:UUID */` markers |
| Plain text | Hash comments | All content | Between `# repo:block:UUID` markers |

## Trait Interface

```rust
use uuid::Uuid;

/// A parsed managed block from a file
#[derive(Debug, Clone)]
pub struct ManagedBlock {
    /// The rule UUID this block belongs to
    pub uuid: Uuid,
    /// The content inside the block
    pub content: String,
    /// Start position in the original content (bytes)
    pub start: usize,
    /// End position in the original content (bytes)
    pub end: usize,
}

/// Handler for format-specific managed block operations
pub trait FormatHandler: Send + Sync {
    /// Parse all managed blocks from file content
    fn parse_blocks(&self, content: &str) -> Vec<ManagedBlock>;

    /// Write or update a managed block in the content
    /// Returns the new file content with the block added/updated
    fn write_block(&self, content: &str, uuid: Uuid, block_content: &str) -> String;

    /// Remove a managed block from the content
    /// Returns the new file content with the block removed
    fn remove_block(&self, content: &str, uuid: Uuid) -> String;

    /// Check if a block with this UUID exists
    fn has_block(&self, content: &str, uuid: Uuid) -> bool {
        self.parse_blocks(content).iter().any(|b| b.uuid == uuid)
    }

    /// Get block content by UUID
    fn get_block(&self, content: &str, uuid: Uuid) -> Option<String> {
        self.parse_blocks(content)
            .into_iter()
            .find(|b| b.uuid == uuid)
            .map(|b| b.content)
    }
}
```

## Format Details

### Markdown/HTML/XML

Uses XML comment syntax. Works for `.md`, `.html`, `.xml`, `.mdc` files.

```markdown
# User content here

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
## Managed Rule

This content is managed by repository-manager.
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->

More user content...
```

**Parser Strategy:**
- Regex: `<!-- repo:block:([0-9a-f-]+) -->(.*?)<!-- /repo:block:\1 -->`
- Multiline, dotall mode

### JSON

Uses a reserved `__repo_managed__` key. All managed content goes under UUID sub-keys.

```json
{
    "editor.formatOnSave": true,
    "python.linting.enabled": true,
    "__repo_managed__": {
        "550e8400-e29b-41d4-a716-446655440000": {
            "python.defaultInterpreterPath": ".venv/bin/python"
        },
        "6ba7b810-9dad-11d1-80b4-00c04fd430c8": {
            "editor.tabSize": 4
        }
    }
}
```

**Parser Strategy:**
- Parse JSON
- Look for `__repo_managed__` object
- Each key is a UUID, value is the managed content

**Write Strategy:**
- Parse existing JSON
- Preserve all keys except `__repo_managed__`
- Merge managed blocks into `__repo_managed__`
- Pretty-print with consistent formatting

### YAML

Uses comment-based markers. Works for `.yml`, `.yaml` files.

```yaml
# User content
name: my-project
version: 1.0.0

# repo:block:550e8400-e29b-41d4-a716-446655440000
python:
  interpreter: .venv/bin/python
  version: "3.11"
# /repo:block:550e8400-e29b-41d4-a716-446655440000

# More user content
dependencies:
  - requests
```

**Parser Strategy:**
- Regex: `# repo:block:([0-9a-f-]+)\n(.*?)# /repo:block:\1`
- Multiline mode

**Limitations:**
- Block content is technically part of the YAML structure
- Must ensure valid YAML after insertion
- Consider using a dedicated `repo_managed:` key similar to JSON approach

### TOML

Uses section-based approach with `[repo_managed.UUID]` tables.

```toml
# User content
[project]
name = "my-project"
version = "1.0.0"

[repo_managed."550e8400-e29b-41d4-a716-446655440000"]
python_path = ".venv/bin/python"
auto_activate = true

[repo_managed."6ba7b810-9dad-11d1-80b4-00c04fd430c8"]
editor_config = "strict"

# More user content
[dependencies]
requests = "2.28"
```

**Parser Strategy:**
- Parse TOML
- Look for `repo_managed` table
- Each sub-table key is a UUID

### JavaScript/CSS

Uses block comment markers.

```javascript
// User code
const userConfig = {
    theme: 'dark'
};

/* repo:block:550e8400-e29b-41d4-a716-446655440000 */
const managedConfig = {
    pythonPath: '.venv/bin/python'
};
/* /repo:block:550e8400-e29b-41d4-a716-446655440000 */

// More user code
export default { ...userConfig, ...managedConfig };
```

**Parser Strategy:**
- Regex: `/\* repo:block:([0-9a-f-]+) \*/(.*?)/\* \/repo:block:\1 \*/`

## Implementation Priority

1. **Markdown** - Already implemented, most common for rules files
2. **JSON** - Required for `.vscode/settings.json`, many config files
3. **TOML** - Required for `pyproject.toml`, Rust configs
4. **YAML** - Required for GitHub Actions, Docker Compose, etc.
5. **JavaScript** - Lower priority, edge cases

## Error Handling

- Invalid UUID in marker: Log warning, skip block
- Unclosed block: Log warning, treat as no block
- Malformed managed key (JSON/TOML): Preserve user content, reset managed section
- Conflicting UUIDs: Last write wins

## File Extension Mapping

```rust
fn handler_for_extension(ext: &str) -> Box<dyn FormatHandler> {
    match ext.to_lowercase().as_str() {
        "md" | "mdc" | "html" | "xml" => Box::new(MarkdownHandler),
        "json" | "jsonc" => Box::new(JsonHandler),
        "yaml" | "yml" => Box::new(YamlHandler),
        "toml" => Box::new(TomlHandler),
        "js" | "ts" | "jsx" | "tsx" | "css" => Box::new(JsHandler),
        _ => Box::new(PlainTextHandler), // Default: hash comments
    }
}
```
