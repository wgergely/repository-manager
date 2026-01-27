# Rule Registry Schema Design

> **Created:** 2026-01-27
> **Status:** Draft
> **Purpose:** Define the central rule registry that links rules to managed blocks via UUIDs

---

## Overview

The Rule Registry is the single source of truth for all rules in a repository. Each rule has a UUID that directly becomes the block marker in tool config files, enabling bidirectional traceability.

## Design Principles

1. **UUID = Block Marker**: Rule UUID is used directly as the managed block marker
2. **Single Registry**: One registry file at `.repository/rules/registry.toml`
3. **Content Hashing**: Track content hash for drift detection
4. **Tool Agnostic**: Rules are tool-independent; projection handles tool-specific formatting

---

## Registry Structure

### File Location

```
.repository/
├── config.toml           # Main config (tools, presets, mode)
├── ledger.toml           # Intent/projection tracking
└── rules/
    └── registry.toml     # Central rule registry (NEW)
```

### Registry Schema

```toml
# .repository/rules/registry.toml
version = "1.0"

[[rules]]
uuid = "550e8400-e29b-41d4-a716-446655440000"
id = "python-style"
content = """
Use snake_case for all variables and functions.
Use type hints on all public functions.
"""
created = "2026-01-27T10:00:00Z"
updated = "2026-01-27T10:00:00Z"
tags = ["python", "style"]
content_hash = "sha256:abc123..."

[[rules]]
uuid = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
id = "api-design"
content = """
All endpoints must return JSON with {data, error, meta} envelope.
Use HTTP status codes correctly.
"""
created = "2026-01-27T10:05:00Z"
updated = "2026-01-27T10:05:00Z"
tags = ["api", "http"]
content_hash = "sha256:def456..."
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `uuid` | UUID v4 | Yes | Unique identifier, used as block marker |
| `id` | String | Yes | Human-readable slug (unique within registry) |
| `content` | String | Yes | The rule text (Markdown) |
| `created` | DateTime | Yes | When rule was created |
| `updated` | DateTime | Yes | When rule was last modified |
| `tags` | Array<String> | No | Categories for filtering |
| `content_hash` | String | Yes | SHA-256 hash for drift detection |

---

## UUID Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Rule Creation                             │
├─────────────────────────────────────────────────────────────────┤
│  repo add-rule python-style --instruction "Use snake_case..."   │
│                              │                                   │
│                              ▼                                   │
│              ┌───────────────────────────────┐                  │
│              │  Generate UUID v4             │                  │
│              │  550e8400-e29b-41d4-...       │                  │
│              └───────────────────────────────┘                  │
│                              │                                   │
│                              ▼                                   │
│              ┌───────────────────────────────┐                  │
│              │  Store in registry.toml       │                  │
│              │  uuid = "550e8400-..."        │                  │
│              │  id = "python-style"          │                  │
│              └───────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                           Sync                                   │
├─────────────────────────────────────────────────────────────────┤
│  repo sync                                                       │
│                              │                                   │
│                              ▼                                   │
│              ┌───────────────────────────────┐                  │
│              │  Read registry.toml           │                  │
│              │  For each rule:               │                  │
│              │    - Get UUID                 │                  │
│              │    - Get content              │                  │
│              └───────────────────────────────┘                  │
│                              │                                   │
│                              ▼                                   │
│              ┌───────────────────────────────┐                  │
│              │  For each configured tool:    │                  │
│              │    - Format content for tool  │                  │
│              │    - Write managed block:     │                  │
│              │                               │                  │
│              │  <!-- repo:block:550e8400-... │                  │
│              │  Use snake_case...            │                  │
│              │  <!-- /repo:block:550e8400... │                  │
│              └───────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                        Drift Detection                           │
├─────────────────────────────────────────────────────────────────┤
│  repo check                                                      │
│                              │                                   │
│                              ▼                                   │
│              ┌───────────────────────────────┐                  │
│              │  For each tool config file:   │                  │
│              │    - Parse managed blocks     │                  │
│              │    - Extract UUID from marker │                  │
│              │    - Look up in registry      │                  │
│              │    - Compare content hash     │                  │
│              └───────────────────────────────┘                  │
│                              │                                   │
│                              ▼                                   │
│              ┌───────────────────────────────┐                  │
│              │  Report:                      │                  │
│              │  ✓ python-style: in sync      │                  │
│              │  ✗ api-design: drifted        │                  │
│              └───────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Integration with Existing Architecture

### Relationship to Ledger

The Ledger tracks **what has been projected** (Intents + Projections).
The Registry tracks **what rules exist** (source of truth).

```
Registry (rules/registry.toml)     Ledger (ledger.toml)
┌─────────────────────────────┐    ┌─────────────────────────────┐
│ Rule                        │    │ Intent                      │
│  uuid: 550e8400-...         │───▶│  id: "rule:python-style"    │
│  id: "python-style"         │    │  uuid: (different UUID)     │
│  content: "..."             │    │  projections:               │
│  content_hash: "..."        │    │    - tool: cursor           │
└─────────────────────────────┘    │      file: .cursorrules     │
                                   │      marker: 550e8400-...   │◀─┐
                                   └─────────────────────────────┘  │
                                                                    │
                                   The projection marker uses the   │
                                   RULE UUID, not the Intent UUID ──┘
```

### Key Decision: Rule UUID as Block Marker

Based on Task 0.2 research, we use the **Rule UUID** (not Intent UUID) as the block marker. This enables:

1. **Direct lookup**: Parse block marker → find rule in registry
2. **No Intent traversal**: Don't need to iterate all Intents to find owner
3. **Simpler architecture**: One UUID to track, not two

---

## API Design

### RuleRegistry

```rust
pub struct RuleRegistry {
    version: String,
    rules: Vec<Rule>,
    path: PathBuf,
}

impl RuleRegistry {
    pub fn new(path: PathBuf) -> Self;
    pub fn load(path: PathBuf) -> Result<Self>;
    pub fn save(&self) -> Result<()>;

    // CRUD
    pub fn add_rule(&mut self, id: &str, content: &str, tags: Vec<String>) -> Result<&Rule>;
    pub fn get_rule(&self, uuid: Uuid) -> Option<&Rule>;
    pub fn get_rule_by_id(&self, id: &str) -> Option<&Rule>;
    pub fn update_rule(&mut self, uuid: Uuid, content: &str) -> Result<()>;
    pub fn remove_rule(&mut self, uuid: Uuid) -> Result<Option<Rule>>;

    // Query
    pub fn all_rules(&self) -> &[Rule];
    pub fn rules_by_tag(&self, tag: &str) -> Vec<&Rule>;
}
```

### Rule

```rust
pub struct Rule {
    pub uuid: Uuid,
    pub id: String,
    pub content: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub tags: Vec<String>,
    pub content_hash: String,
}

impl Rule {
    pub fn new(id: impl Into<String>, content: impl Into<String>, tags: Vec<String>) -> Self;
    pub fn compute_hash(&self) -> String;
    pub fn has_drifted(&self, current_content: &str) -> bool;
}
```

---

## Migration Path

For existing repositories without a registry:

1. On first `repo sync` after upgrade:
   - Scan existing rule files in `.repository/rules/*.md`
   - Generate UUIDs for each
   - Create `registry.toml`
   - Update tool config files with new UUID markers

2. For existing managed blocks with old format:
   - Parse old markers
   - Map to new UUID-based markers
   - Preserve content, update markers

---

## Open Questions

1. **Should registry be human-editable?** Yes, but with tooling to validate.
2. **What happens on UUID collision?** Reject with error (UUID v4 collision is astronomically unlikely).
3. **Should we support rule inheritance/composition?** Future enhancement, not MVP.
