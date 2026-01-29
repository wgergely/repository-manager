# Phase 3: Config Writers

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create semantic-aware config writers that merge properly.

**Parent:** [2026-01-29-registry-architecture-index.md](2026-01-29-registry-architecture-index.md)
**Depends on:** [2026-01-29-phase2-capability-translator.md](2026-01-29-phase2-capability-translator.md)
**Next Phase:** [2026-01-29-phase4-integration.md](2026-01-29-phase4-integration.md)

---

## What This Solves

Current system uses `<!-- repo:block:X -->` markers for ad-hoc injection.

This phase creates proper format-aware writers:
- **JSON**: Semantic merge (preserve user keys)
- **Markdown**: Section-based merge
- **Text**: Full replacement (tool owns file)

---

## Task 3.1: Create ConfigWriter trait

**Files:**
- Create: `crates/repo-tools/src/writer/mod.rs`
- Create: `crates/repo-tools/src/writer/traits.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create traits.rs**

```rust
//! ConfigWriter trait

use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::NormalizedPath;

pub trait ConfigWriter: Send + Sync {
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        schema_keys: Option<&SchemaKeys>,
    ) -> Result<()>;

    fn can_handle(&self, path: &NormalizedPath) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct SchemaKeys {
    pub instruction_key: Option<String>,
    pub mcp_key: Option<String>,
    pub python_path_key: Option<String>,
}

impl From<&repo_meta::schema::ToolSchemaKeys> for SchemaKeys {
    fn from(k: &repo_meta::schema::ToolSchemaKeys) -> Self {
        Self {
            instruction_key: k.instruction_key.clone(),
            mcp_key: k.mcp_key.clone(),
            python_path_key: k.python_path_key.clone(),
        }
    }
}
```

**Step 2: Create mod.rs and export**

```bash
git add crates/repo-tools/src/writer/
git commit -m "feat(repo-tools): add ConfigWriter trait"
```

---

## Task 3.2: Create JsonWriter

**Files:**
- Create: `crates/repo-tools/src/writer/json.rs`
- Modify: `crates/repo-tools/src/writer/mod.rs`

**Step 1: Create json.rs**

```rust
//! JSON config writer with semantic merge

use super::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};
use serde_json::{json, Value};

pub struct JsonWriter;

impl JsonWriter {
    pub fn new() -> Self { Self }

    fn parse_existing(path: &NormalizedPath) -> Value {
        if !path.exists() { return json!({}); }
        io::read_text(path)
            .ok()
            .and_then(|c| serde_json::from_str(&c).ok())
            .unwrap_or(json!({}))
    }

    fn merge(existing: &mut Value, content: &TranslatedContent, keys: Option<&SchemaKeys>) {
        let obj = match existing.as_object_mut() {
            Some(o) => o,
            None => return,
        };

        if let (Some(instr), Some(k)) = (&content.instructions, keys) {
            if let Some(ref key) = k.instruction_key {
                obj.insert(key.clone(), json!(instr));
            }
        }

        if let (Some(mcp), Some(k)) = (&content.mcp_servers, keys) {
            if let Some(ref key) = k.mcp_key {
                obj.insert(key.clone(), mcp.clone());
            }
        }

        for (key, value) in &content.data {
            obj.insert(key.clone(), value.clone());
        }
    }
}

impl Default for JsonWriter {
    fn default() -> Self { Self::new() }
}

impl ConfigWriter for JsonWriter {
    fn write(&self, path: &NormalizedPath, content: &TranslatedContent, keys: Option<&SchemaKeys>) -> Result<()> {
        let mut existing = Self::parse_existing(path);
        if !existing.is_object() { existing = json!({}); }
        Self::merge(&mut existing, content, keys);
        io::write_text(path, &serde_json::to_string_pretty(&existing)?)?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        path.as_str().ends_with(".json")
    }
}

#[cfg(test)]
mod tests {
    // Test: write to new file
    // Test: preserves existing content
    // Test: uses schema keys
}
```

**Step 2: Update mod.rs and commit**

```bash
cargo test -p repo-tools writer::json
git add crates/repo-tools/src/writer/
git commit -m "feat(repo-tools): add JsonWriter with semantic merge"
```

---

## Task 3.3: Create MarkdownWriter

**Files:**
- Create: `crates/repo-tools/src/writer/markdown.rs`
- Modify: `crates/repo-tools/src/writer/mod.rs`

**Step 1: Create markdown.rs**

```rust
//! Markdown writer with section-based merge

use super::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};

const MANAGED_START: &str = "<!-- repo:managed:start -->";
const MANAGED_END: &str = "<!-- repo:managed:end -->";

pub struct MarkdownWriter;

impl MarkdownWriter {
    pub fn new() -> Self { Self }

    fn parse_existing(path: &NormalizedPath) -> (String, String) {
        if !path.exists() { return (String::new(), String::new()); }

        let content = match io::read_text(path) {
            Ok(c) => c,
            Err(_) => return (String::new(), String::new()),
        };

        if let (Some(start), Some(end)) = (content.find(MANAGED_START), content.find(MANAGED_END)) {
            let before = content[..start].trim_end();
            let after = content[end + MANAGED_END.len()..].trim_start();
            let user = if after.is_empty() { before.to_string() }
                       else { format!("{}\n\n{}", before, after) };
            (user, String::new())
        } else {
            (content, String::new())
        }
    }

    fn combine(user: &str, managed: &str) -> String {
        let mut out = String::new();
        if !user.is_empty() {
            out.push_str(user);
            out.push_str("\n\n");
        }
        out.push_str(MANAGED_START);
        out.push('\n');
        out.push_str(managed);
        out.push('\n');
        out.push_str(MANAGED_END);
        out.push('\n');
        out
    }
}

impl Default for MarkdownWriter {
    fn default() -> Self { Self::new() }
}

impl ConfigWriter for MarkdownWriter {
    fn write(&self, path: &NormalizedPath, content: &TranslatedContent, _: Option<&SchemaKeys>) -> Result<()> {
        let (user, _) = Self::parse_existing(path);
        let managed = content.instructions.as_deref().unwrap_or("");
        io::write_text(path, &Self::combine(&user, managed))?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        let p = path.as_str();
        p.ends_with(".md") || p.ends_with(".markdown")
    }
}

#[cfg(test)]
mod tests {
    // Test: write to new file
    // Test: preserves user content
    // Test: updates managed section
}
```

**Step 2: Commit**

```bash
cargo test -p repo-tools writer::markdown
git add crates/repo-tools/src/writer/
git commit -m "feat(repo-tools): add MarkdownWriter with section merge"
```

---

## Task 3.4: Create TextWriter and WriterRegistry

**Files:**
- Create: `crates/repo-tools/src/writer/text.rs`
- Create: `crates/repo-tools/src/writer/registry.rs`
- Modify: `crates/repo-tools/src/writer/mod.rs`

**Step 1: Create text.rs**

```rust
//! Plain text writer (full replacement)

use super::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};

pub struct TextWriter;

impl TextWriter {
    pub fn new() -> Self { Self }
}

impl Default for TextWriter {
    fn default() -> Self { Self::new() }
}

impl ConfigWriter for TextWriter {
    fn write(&self, path: &NormalizedPath, content: &TranslatedContent, _: Option<&SchemaKeys>) -> Result<()> {
        io::write_text(path, content.instructions.as_deref().unwrap_or(""))?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        let p = path.as_str();
        !p.ends_with(".json") && !p.ends_with(".yaml") && !p.ends_with(".yml")
            && !p.ends_with(".toml") && !p.ends_with(".md") && !p.ends_with(".markdown")
    }
}
```

**Step 2: Create registry.rs**

```rust
//! Writer registry

use super::{ConfigWriter, JsonWriter, MarkdownWriter, TextWriter};
use repo_meta::schema::ConfigType;

pub struct WriterRegistry {
    json: JsonWriter,
    markdown: MarkdownWriter,
    text: TextWriter,
}

impl WriterRegistry {
    pub fn new() -> Self {
        Self {
            json: JsonWriter::new(),
            markdown: MarkdownWriter::new(),
            text: TextWriter::new(),
        }
    }

    pub fn get_writer(&self, config_type: ConfigType) -> &dyn ConfigWriter {
        match config_type {
            ConfigType::Json => &self.json,
            ConfigType::Markdown => &self.markdown,
            ConfigType::Text | ConfigType::Yaml | ConfigType::Toml => &self.text,
        }
    }
}

impl Default for WriterRegistry {
    fn default() -> Self { Self::new() }
}
```

**Step 3: Update mod.rs exports**

```rust
mod json;
mod markdown;
mod registry;
mod text;
mod traits;

pub use json::JsonWriter;
pub use markdown::MarkdownWriter;
pub use registry::WriterRegistry;
pub use text::TextWriter;
pub use traits::{ConfigWriter, SchemaKeys};
```

**Step 4: Commit**

```bash
cargo test -p repo-tools writer
git add crates/repo-tools/src/writer/
git commit -m "feat(repo-tools): add TextWriter and WriterRegistry"
```

---

## Phase 3 Complete Checklist

- [ ] `ConfigWriter` trait created
- [ ] `JsonWriter` merges semantically
- [ ] `MarkdownWriter` uses section markers
- [ ] `TextWriter` does full replacement
- [ ] `WriterRegistry` selects by format
- [ ] All tests pass

**Next:** [Phase 4 - Integration](2026-01-29-phase4-integration.md)
