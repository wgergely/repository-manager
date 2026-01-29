# Phase 2: Capability Translator

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create translation layer that respects tool capabilities.

**Parent:** [2026-01-29-registry-architecture-index.md](2026-01-29-registry-architecture-index.md)
**Depends on:** [2026-01-29-phase1-tool-registry.md](2026-01-29-phase1-tool-registry.md)
**Next Phase:** [2026-01-29-phase3-config-writers.md](2026-01-29-phase3-config-writers.md)

---

## What This Solves

Currently `ToolCapabilities` is declared but NEVER USED:
```rust
pub struct ToolCapabilities {
    pub supports_custom_instructions: bool,  // IGNORED
    pub supports_mcp: bool,                  // IGNORED
    pub supports_rules_directory: bool,      // IGNORED
}
```

This phase makes capabilities actually control what gets generated.

---

## Task 2.1: Create TranslatedContent type

**Files:**
- Create: `crates/repo-tools/src/translator/mod.rs`
- Create: `crates/repo-tools/src/translator/content.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create content.rs**

```rust
//! Translated content ready for config writing

use repo_meta::schema::ConfigType;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TranslatedContent {
    pub format: ConfigType,
    pub instructions: Option<String>,
    pub mcp_servers: Option<Value>,
    pub data: HashMap<String, Value>,
}

impl TranslatedContent {
    pub fn empty() -> Self { Self::default() }

    pub fn with_instructions(format: ConfigType, instructions: String) -> Self {
        Self { format, instructions: Some(instructions), ..Default::default() }
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_none() && self.mcp_servers.is_none() && self.data.is_empty()
    }

    pub fn with_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert!(TranslatedContent::empty().is_empty());
    }

    #[test]
    fn test_with_instructions() {
        let c = TranslatedContent::with_instructions(ConfigType::Markdown, "x".into());
        assert!(!c.is_empty());
    }
}
```

**Step 2: Create mod.rs, export from lib.rs, commit**

```bash
cargo test -p repo-tools translator
git add crates/repo-tools/src/translator/ crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add TranslatedContent type"
```

---

## Task 2.2: Create RuleTranslator

**Files:**
- Create: `crates/repo-tools/src/translator/rules.rs`
- Modify: `crates/repo-tools/src/translator/mod.rs`

**Step 1: Create rules.rs**

```rust
//! Rule translation with semantic formatting

use super::TranslatedContent;
use repo_meta::schema::{ConfigType, RuleDefinition, Severity, ToolDefinition};

pub struct RuleTranslator;

impl RuleTranslator {
    pub fn translate(tool: &ToolDefinition, rules: &[RuleDefinition]) -> TranslatedContent {
        // KEY CHANGE: Actually check the capability!
        if !tool.capabilities.supports_custom_instructions {
            return TranslatedContent::empty();
        }

        if rules.is_empty() {
            return TranslatedContent::empty();
        }

        let format = tool.integration.config_type;
        let instructions = Self::format_rules(rules, format);
        TranslatedContent::with_instructions(format, instructions)
    }

    fn format_rules(rules: &[RuleDefinition], format: ConfigType) -> String {
        let mut sorted: Vec<_> = rules.iter().collect();
        sorted.sort_by_key(|r| match r.meta.severity {
            Severity::Mandatory => 0,
            Severity::Suggestion => 1,
        });

        sorted.iter()
            .map(|r| Self::format_rule(r, format))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn format_rule(rule: &RuleDefinition, format: ConfigType) -> String {
        match format {
            ConfigType::Markdown | ConfigType::Text => Self::format_md(rule),
            _ => rule.content.instruction.clone(),
        }
    }

    fn format_md(rule: &RuleDefinition) -> String {
        let marker = match rule.meta.severity {
            Severity::Mandatory => "**[REQUIRED]**",
            Severity::Suggestion => "[Suggested]",
        };
        let mut out = format!("## {} {}\n\n{}", rule.meta.id, marker, rule.content.instruction);

        if let Some(ref ex) = rule.examples {
            if !ex.positive.is_empty() {
                out.push_str("\n\n**Good:**\n");
                for e in &ex.positive { out.push_str(&format!("```\n{}\n```\n", e)); }
            }
            if !ex.negative.is_empty() {
                out.push_str("\n**Bad:**\n");
                for e in &ex.negative { out.push_str(&format!("```\n{}\n```\n", e)); }
            }
        }

        if let Some(ref t) = rule.targets {
            if !t.file_patterns.is_empty() {
                out.push_str(&format!("\n\n**Applies to:** {}", t.file_patterns.join(", ")));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    // Test: empty when no capability
    // Test: translates when capable
    // Test: mandatory rules first
}
```

**Step 2: Update mod.rs and commit**

```bash
cargo test -p repo-tools translator::rules
git add crates/repo-tools/src/translator/
git commit -m "feat(repo-tools): add RuleTranslator with capability checking"
```

---

## Task 2.3: Create CapabilityTranslator orchestrator

**Files:**
- Create: `crates/repo-tools/src/translator/capability.rs`
- Modify: `crates/repo-tools/src/translator/mod.rs`

**Step 1: Create capability.rs**

```rust
//! Main capability translator

use super::{RuleTranslator, TranslatedContent};
use repo_meta::schema::{RuleDefinition, ToolDefinition};

pub struct CapabilityTranslator;

impl CapabilityTranslator {
    pub fn translate(tool: &ToolDefinition, rules: &[RuleDefinition]) -> TranslatedContent {
        let mut content = TranslatedContent::empty();
        content.format = tool.integration.config_type;

        if tool.capabilities.supports_custom_instructions {
            let rc = RuleTranslator::translate(tool, rules);
            content.instructions = rc.instructions;
        }

        // MCP: Phase 4
        // Rules directory: Future

        content
    }

    pub fn has_capabilities(tool: &ToolDefinition) -> bool {
        tool.capabilities.supports_custom_instructions
            || tool.capabilities.supports_mcp
            || tool.capabilities.supports_rules_directory
    }
}

#[cfg(test)]
mod tests {
    // Test: translate with capability
    // Test: translate without capability
    // Test: has_capabilities
}
```

**Step 2: Update exports and commit**

```rust
// translator/mod.rs
mod capability;
mod content;
mod rules;

pub use capability::CapabilityTranslator;
pub use content::TranslatedContent;
pub use rules::RuleTranslator;
```

```bash
cargo test -p repo-tools translator
git add crates/repo-tools/src/translator/ crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add CapabilityTranslator orchestrator"
```

---

## Phase 2 Complete Checklist

- [ ] `TranslatedContent` type created
- [ ] `RuleTranslator` checks `supports_custom_instructions`
- [ ] `CapabilityTranslator` orchestrates translation
- [ ] Capabilities now actually control output
- [ ] All tests pass

**Next:** [Phase 3 - Config Writers](2026-01-29-phase3-config-writers.md)
