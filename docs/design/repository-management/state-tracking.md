# Repository State & Capability Tracking Schema

## Overview

To manage complex, overlapping tool capabilities (e.g., Antigravity using both `.agent` and `.vscode` folders), we move beyond simple configuration files to a **Ledger-Based State System**.

The Repository Manager maintains a helper file (the "Ledger") that maps **Abstract Intents** (e.g., "Enforce Snake Case") to **Concrete Realizations** (specific file edits, JSON keys, files created). This allows for precise "Unrolling" (removal) and updates, even when a single rule fans out to multiple locations.

## The Ledger (`.repository/ledger.toml`)

This file is the machine-readable record of all active modifications performed by the manager. It allows the CLI to answer: *"Which lines in `.cursorrules` belong to the Python Style rule?"*

### Schema Design

The schema tracks the **Intent** (high-level rule) and its **Projections** (low-level artifacts).

```toml
[meta]
version = "1.0"
updated_at = "2024-01-23T14:30:00Z"

# Array of active capabilities/rules
[[intents]]
# 1. Identity & Definition
id = "rule:python/style/snake-case"  # Canonical ID of the rule definition
uuid = "550e8400-e29b-41d4-a716-446655440000" # Unique Instance ID for this specific application
timestamp = "2024-01-23T14:30:00Z"

# 2. Configuration used (Snapshot)
# We store the args used to generate the content, so we can re-generate or validte
args = { severity = "error", exceptions = ["constants"] }

# 3. Projections (The Look-up Table for Unrolling)
# A single intent can modify multiple files across multiple tools.

    # Projection A: Cursor Rules (Targeting a Text File)
    [[intents.projections]]
    tool = "cursor"
    file = ".cursorrules"
    backend = "text_block"
    # The UUID is embedded in the file using markers:
    # <!-- repo:550e8400... --> content <!-- /repo:550e8400... -->
    marker_uuid = "550e8400-e29b-41d4-a716-446655440000" 
    checksum = "sha256:abc12345..." # To detect if user manually edited inside the block

    # Projection B: VSCode Settings (Targeting a JSON Key)
    [[intents.projections]]
    tool = "vscode"
    file = ".vscode/settings.json"
    backend = "json_key"
    key_path = "python.analysis.typeCheckingMode"
    value_snapshot = "strict" # What we set it to (to differentiate from user changes)

    # Projection C: Antigravity IDE (Targeting a File Creation)
    [[intents.projections]]
    tool = "antigravity"
    file = ".agent/rules/python-style.md"
    backend = "file_managed" # We own this entire file
    file_uuid = "550e8400-e29b-41d4-a716-446655440000"
```

## Embedding Strategies

To ensure we can reliably "Unroll" (remove) content, we use different embedding strategies based on the target file format.

### 1. The Marker Strategy (Text/Markdown/Code)

For unstructured files (cursorrules, system prompts, source code), we rely on **UUID-tagged Delimiters**.

**Format:**
`{COMMENT_START} repo:block:{UUID} {COMMENT_END}`

**Example (.cursorrules):**

```markdown
# User content...
Always be helpful.

<!-- repo:block:550e8400-e29b-41d4-a716-446655440000 -->
## Python Style
* Ensure all variables are snake_case.
<!-- /repo:block:550e8400-e29b-41d4-a716-446655440000 -->
```

* **Lookup:** To remove, the Manager scans the file for the matching UUID markers and deletes the range.
* **Drift Detection:** If the markers are missing or broken, the Manager reports a "Broken Link" status in `repo check`.

### 2. The Key-Ownership Strategy (JSON/TOML/YAML)

For structured files (VSCode settings, global config), we track the specific **Keys** we modified.

**Example (.vscode/settings.json):**

```json
{
    "editor.fontSize": 14,
    "python.linting.pylintEnabled": true
}
```

* **Ledger Record:** `key_path = "python.linting.pylintEnabled"`
* **Unroll:**
    1. Read the JSON.
    2. Check if `python.linting.pylintEnabled` matches our `value_snapshot`.
    3. If yes, delete the key (or revert to default).
    4. If no (User changed it to `false`), we prompt: "User modified this setting. Keep or Force Overwrite?"

### 3. The Sidecar Strategy (Binary/Opaque)

If we cannot embed metadata (e.g., binary files or strictly validated schemas that don't allow comments), we rely purely on the Ledger's `checksum` and `file_path`.

* **Unroll:** Hash the current file. If it matches our `checksum`, delete/revert. If not, warn user.

## Data Structures (Rust)

```rust
#[derive(Serialize, Deserialize)]
pub struct Ledger {
    pub intents: Vec<Intent>,
}

#[derive(Serialize, Deserialize)]
pub struct Intent {
    pub id: String,         // "rule:snake-case"
    pub uuid: Uuid,         // Randomly generated on 'add'
    pub args: Value,        // Changes meaning based on rule definition
    pub projections: Vec<Projection>,
}

#[derive(Serialize, Deserialize)]
pub struct Projection {
    pub tool: String,       // "cursor"
    pub file: PathBuf,      // ".cursorrules"
    pub kind: ProjectionKind, 
}

#[derive(Serialize, Deserialize)]
pub enum ProjectionKind {
    TextBlock { marker: Uuid, checksum: String },
    JsonKey { path: String, value: Value },
    FileManaged { checksum: String },
}
```

## Workflow Example: "Add Pytest Rule"

1. **User Command**: `repo rule add python-testing --framework pytest`
2. **Manager Action**:
    * Generates `UUID-1`.
    * Consults Registry: "python-testing" rule implies:
        * VSCode: Set `python.testing.pytestEnabled = true`
        * Antigravity: Create `.agent/rules/testing.md`
    * **Writes to VSCode**: Updates `settings.json`.
    * **Writes to Antigravity**: Creates file with header `<!-- repo:file:UUID-1 -->`.
    * **Updates Ledger**:
        * Adds `Intent(UUID-1)` entries for both projections.
3. **User Command**: `repo rule remove python-testing`
4. **Manager Action**:
    * Finds `Intent` with `id="python-testing"`.
    * Iterates Projections:
        * **VSCode**: Removes json key.
        * **Antigravity**: Deletes `.agent/rules/testing.md`.
    * Removes `Intent` from Ledger.
