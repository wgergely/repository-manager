# Capability-Based Registry Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a capability-driven translation layer that generates tool configs from a canonical registry, replacing ad-hoc block injection with semantic-aware config generation.

**Architecture:** Three-layer system: (1) Registry Domain holds canonical rules, tools, and MCP servers; (2) Capability Translator maps registry content to tool-specific formats based on declared capabilities; (3) Config Writers handle AST-aware merging with existing configs while preserving user content.

**Tech Stack:** Rust, serde, serde_json, toml, tree-sitter (for AST parsing in later phases)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    REGISTRY DOMAIN (repo-meta)                       │
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐   │
│  │ Rules        │  │ MCP Servers  │  │ Tools                    │   │
│  │ - content    │  │ - command    │  │ - capabilities           │   │
│  │ - severity   │  │ - args       │  │ - schema_keys            │   │
│  │ - targets    │  │ - env        │  │ - config locations       │   │
│  │ - examples   │  │              │  │                          │   │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                CAPABILITY TRANSLATOR (repo-tools)                    │
│                                                                      │
│  For each (tool, capability):                                        │
│    supports_custom_instructions → RuleTranslator                     │
│    supports_mcp                 → MCPTranslator                      │
│    supports_rules_directory     → DirectoryRuleTranslator            │
│                                                                      │
│  Each translator produces: TranslatedContent { format, data }        │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   CONFIG WRITERS (repo-tools)                        │
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌────────────┐  │
│  │ JsonWriter  │  │ YamlWriter  │  │ TomlWriter  │  │ TextWriter │  │
│  │ - parse     │  │ - parse     │  │ - parse     │  │ - sections │  │
│  │ - merge     │  │ - merge     │  │ - merge     │  │ - blocks   │  │
│  │ - serialize │  │ - serialize │  │ - serialize │  │ - write    │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      TOOL CONFIG FILES                               │
│                                                                      │
│  .cursorrules, CLAUDE.md, .vscode/settings.json, etc.               │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Phased Approach

| Phase | Focus | Outcome |
|-------|-------|---------|
| **1** | Capability Translator Foundation | Capabilities actually control what gets generated |
| **2** | Semantic Rule Translation | Rules translated based on severity, targets, examples |
| **3** | Structured Config Writers | JSON/YAML/TOML parsed and merged semantically |
| **4** | MCP Server Support | Full MCP config generation for supporting tools |

---

## Phase 1: Capability Translator Foundation

### Goal
Make `ToolCapabilities` actually control what content gets generated for each tool.

### Current Problem
```rust
// ToolCapabilities exists but is NEVER CHECKED:
pub struct ToolCapabilities {
    pub supports_custom_instructions: bool,  // Ignored
    pub supports_mcp: bool,                  // Ignored
    pub supports_rules_directory: bool,      // Ignored
}

// GenericToolIntegration::sync() just writes everything regardless of capabilities
```

### Design
```rust
/// Translates registry content to tool-specific format based on capabilities
pub struct CapabilityTranslator;

impl CapabilityTranslator {
    /// Translate rules for a specific tool based on its capabilities
    pub fn translate_rules(
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> TranslatedContent {
        // Only translate if tool supports custom instructions
        if !tool.capabilities.supports_custom_instructions {
            return TranslatedContent::empty();
        }
        // ... translation logic
    }
}

/// Content ready to be written to a tool config
pub struct TranslatedContent {
    /// The format this content should be written as
    pub format: ConfigType,
    /// Main content (rules, instructions)
    pub instructions: Option<String>,
    /// MCP server configurations (if applicable)
    pub mcp_servers: Option<serde_json::Value>,
    /// Additional structured data
    pub metadata: HashMap<String, serde_json::Value>,
}
```

---

### Task 1.1: Create TranslatedContent type

**Files:**
- Create: `crates/repo-tools/src/translator/mod.rs`
- Create: `crates/repo-tools/src/translator/content.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/repo-tools/src/translator/content.rs`:
```rust
//! Translated content ready for config writing

use repo_meta::schema::ConfigType;
use serde_json::Value;
use std::collections::HashMap;

/// Content translated from registry, ready to be written to tool config
#[derive(Debug, Clone, Default)]
pub struct TranslatedContent {
    /// Target format for this content
    pub format: ConfigType,
    /// Translated instruction/rules content
    pub instructions: Option<String>,
    /// MCP server configurations
    pub mcp_servers: Option<Value>,
    /// Additional key-value data for structured configs
    pub data: HashMap<String, Value>,
}

impl TranslatedContent {
    /// Create empty content (tool doesn't support this capability)
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create content with instructions only
    pub fn with_instructions(format: ConfigType, instructions: String) -> Self {
        Self {
            format,
            instructions: Some(instructions),
            ..Default::default()
        }
    }

    /// Check if this content has anything to write
    pub fn is_empty(&self) -> bool {
        self.instructions.is_none()
            && self.mcp_servers.is_none()
            && self.data.is_empty()
    }

    /// Add a data field
    pub fn with_data(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }

    /// Set MCP servers
    pub fn with_mcp_servers(mut self, servers: Value) -> Self {
        self.mcp_servers = Some(servers);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_content() {
        let content = TranslatedContent::empty();
        assert!(content.is_empty());
        assert!(content.instructions.is_none());
        assert!(content.mcp_servers.is_none());
    }

    #[test]
    fn test_with_instructions() {
        let content = TranslatedContent::with_instructions(
            ConfigType::Markdown,
            "Use snake_case for variables".into(),
        );
        assert!(!content.is_empty());
        assert_eq!(content.format, ConfigType::Markdown);
        assert!(content.instructions.is_some());
    }

    #[test]
    fn test_with_data() {
        let content = TranslatedContent::empty()
            .with_data("python.path", serde_json::json!("/usr/bin/python3"));
        assert!(!content.is_empty());
        assert!(content.data.contains_key("python.path"));
    }

    #[test]
    fn test_builder_chain() {
        let content = TranslatedContent::with_instructions(
            ConfigType::Json,
            "instructions".into(),
        )
        .with_data("key", serde_json::json!("value"))
        .with_mcp_servers(serde_json::json!({"server": {}}));

        assert!(!content.is_empty());
        assert!(content.instructions.is_some());
        assert!(content.mcp_servers.is_some());
        assert!(!content.data.is_empty());
    }
}
```

**Step 2: Create module file**

Create `crates/repo-tools/src/translator/mod.rs`:
```rust
//! Capability-based translation from registry to tool configs
//!
//! This module translates canonical registry content (rules, MCP servers)
//! into tool-specific formats based on each tool's declared capabilities.

mod content;

pub use content::TranslatedContent;
```

**Step 3: Export from lib.rs**

Add to `crates/repo-tools/src/lib.rs`:
```rust
pub mod translator;
pub use translator::TranslatedContent;
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools translator::content`
Expected: All 4 tests pass

**Step 5: Commit**

```bash
git add crates/repo-tools/src/translator/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add TranslatedContent type for capability translation"
```

---

### Task 1.2: Create RuleTranslator for instruction generation

**Files:**
- Create: `crates/repo-tools/src/translator/rules.rs`
- Modify: `crates/repo-tools/src/translator/mod.rs`

**Step 1: Write the failing test**

Create `crates/repo-tools/src/translator/rules.rs`:
```rust
//! Rule translation based on tool capabilities

use super::TranslatedContent;
use repo_meta::schema::{ConfigType, RuleDefinition, Severity, ToolDefinition};

/// Translates rules to tool-specific instruction format
pub struct RuleTranslator;

impl RuleTranslator {
    /// Translate rules for a tool based on its capabilities
    ///
    /// Returns empty content if tool doesn't support custom instructions.
    pub fn translate(
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> TranslatedContent {
        // Check capability - this is the key change!
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

    /// Format rules into a string based on output format
    fn format_rules(rules: &[RuleDefinition], format: ConfigType) -> String {
        let mut output = String::new();

        // Sort by severity (mandatory first)
        let mut sorted: Vec<_> = rules.iter().collect();
        sorted.sort_by_key(|r| match r.meta.severity {
            Severity::Mandatory => 0,
            Severity::Suggestion => 1,
        });

        for rule in sorted {
            let formatted = Self::format_single_rule(rule, format);
            output.push_str(&formatted);
            output.push_str("\n\n");
        }

        output.trim_end().to_string()
    }

    /// Format a single rule based on output format
    fn format_single_rule(rule: &RuleDefinition, format: ConfigType) -> String {
        match format {
            ConfigType::Markdown | ConfigType::Text => {
                Self::format_rule_markdown(rule)
            }
            ConfigType::Json | ConfigType::Yaml | ConfigType::Toml => {
                // For structured formats, just return the instruction
                // The actual structure is handled by the config writer
                rule.content.instruction.clone()
            }
        }
    }

    /// Format rule as markdown with severity and examples
    fn format_rule_markdown(rule: &RuleDefinition) -> String {
        let mut output = String::new();

        // Header with severity indicator
        let severity_marker = match rule.meta.severity {
            Severity::Mandatory => "**[REQUIRED]**",
            Severity::Suggestion => "[Suggested]",
        };
        output.push_str(&format!("## {} {}\n\n", rule.meta.id, severity_marker));

        // Main instruction
        output.push_str(&rule.content.instruction);

        // Add examples if present
        if let Some(ref examples) = rule.examples {
            if !examples.positive.is_empty() || !examples.negative.is_empty() {
                output.push_str("\n\n### Examples\n");

                if !examples.positive.is_empty() {
                    output.push_str("\n**Good:**\n");
                    for ex in &examples.positive {
                        output.push_str(&format!("```\n{}\n```\n", ex));
                    }
                }

                if !examples.negative.is_empty() {
                    output.push_str("\n**Bad:**\n");
                    for ex in &examples.negative {
                        output.push_str(&format!("```\n{}\n```\n", ex));
                    }
                }
            }
        }

        // Add file targets if present
        if let Some(ref targets) = rule.targets {
            if !targets.file_patterns.is_empty() {
                output.push_str("\n\n**Applies to:** ");
                output.push_str(&targets.file_patterns.join(", "));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        RuleContent, RuleExamples, RuleMeta, RuleTargets,
        ToolCapabilities, ToolIntegrationConfig, ToolMeta,
    };

    fn tool_with_instructions() -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Test".into(),
                slug: "test".into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".test".into(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: true,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    fn tool_without_instructions() -> ToolDefinition {
        let mut tool = tool_with_instructions();
        tool.capabilities.supports_custom_instructions = false;
        tool
    }

    fn sample_rule() -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta {
                id: "snake-case".into(),
                severity: Severity::Mandatory,
                tags: vec!["style".into()],
            },
            content: RuleContent {
                instruction: "Use snake_case for variable names.".into(),
            },
            examples: Some(RuleExamples {
                positive: vec!["my_variable = 1".into()],
                negative: vec!["myVariable = 1".into()],
            }),
            targets: Some(RuleTargets {
                file_patterns: vec!["**/*.py".into()],
            }),
        }
    }

    #[test]
    fn test_translate_empty_when_no_capability() {
        let tool = tool_without_instructions();
        let rules = vec![sample_rule()];

        let content = RuleTranslator::translate(&tool, &rules);

        assert!(content.is_empty(), "Should return empty when tool doesn't support instructions");
    }

    #[test]
    fn test_translate_empty_when_no_rules() {
        let tool = tool_with_instructions();
        let rules: Vec<RuleDefinition> = vec![];

        let content = RuleTranslator::translate(&tool, &rules);

        assert!(content.is_empty(), "Should return empty when no rules");
    }

    #[test]
    fn test_translate_produces_content() {
        let tool = tool_with_instructions();
        let rules = vec![sample_rule()];

        let content = RuleTranslator::translate(&tool, &rules);

        assert!(!content.is_empty());
        assert!(content.instructions.is_some());

        let instructions = content.instructions.unwrap();
        assert!(instructions.contains("snake-case"));
        assert!(instructions.contains("**[REQUIRED]**"));
        assert!(instructions.contains("snake_case"));
    }

    #[test]
    fn test_translate_includes_examples() {
        let tool = tool_with_instructions();
        let rules = vec![sample_rule()];

        let content = RuleTranslator::translate(&tool, &rules);
        let instructions = content.instructions.unwrap();

        assert!(instructions.contains("my_variable = 1"));
        assert!(instructions.contains("myVariable = 1"));
        assert!(instructions.contains("**Good:**"));
        assert!(instructions.contains("**Bad:**"));
    }

    #[test]
    fn test_translate_includes_targets() {
        let tool = tool_with_instructions();
        let rules = vec![sample_rule()];

        let content = RuleTranslator::translate(&tool, &rules);
        let instructions = content.instructions.unwrap();

        assert!(instructions.contains("**/*.py"));
        assert!(instructions.contains("Applies to:"));
    }

    #[test]
    fn test_mandatory_rules_come_first() {
        let tool = tool_with_instructions();

        let mut suggestion = sample_rule();
        suggestion.meta.id = "suggestion-rule".into();
        suggestion.meta.severity = Severity::Suggestion;

        let mut mandatory = sample_rule();
        mandatory.meta.id = "mandatory-rule".into();
        mandatory.meta.severity = Severity::Mandatory;

        // Pass suggestion first, mandatory second
        let rules = vec![suggestion, mandatory];
        let content = RuleTranslator::translate(&tool, &rules);
        let instructions = content.instructions.unwrap();

        // Mandatory should appear before suggestion in output
        let mandatory_pos = instructions.find("mandatory-rule").unwrap();
        let suggestion_pos = instructions.find("suggestion-rule").unwrap();
        assert!(mandatory_pos < suggestion_pos, "Mandatory rules should come first");
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/translator/mod.rs`:
```rust
//! Capability-based translation from registry to tool configs
//!
//! This module translates canonical registry content (rules, MCP servers)
//! into tool-specific formats based on each tool's declared capabilities.

mod content;
mod rules;

pub use content::TranslatedContent;
pub use rules::RuleTranslator;
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools translator::rules`
Expected: All 6 tests pass

**Step 4: Commit**

```bash
git add crates/repo-tools/src/translator/
git commit -m "feat(repo-tools): add RuleTranslator with capability checking"
```

---

### Task 1.3: Create CapabilityTranslator orchestrator

**Files:**
- Create: `crates/repo-tools/src/translator/capability.rs`
- Modify: `crates/repo-tools/src/translator/mod.rs`

**Step 1: Write the failing test**

Create `crates/repo-tools/src/translator/capability.rs`:
```rust
//! Main capability translator that orchestrates all translations

use super::{RuleTranslator, TranslatedContent};
use repo_meta::schema::{RuleDefinition, ToolDefinition};

/// Orchestrates translation of registry content to tool-specific formats
///
/// This is the main entry point for the translation layer. It checks
/// tool capabilities and delegates to appropriate translators.
pub struct CapabilityTranslator;

impl CapabilityTranslator {
    /// Translate all applicable content for a tool
    ///
    /// Checks tool capabilities and translates:
    /// - Rules (if supports_custom_instructions)
    /// - MCP servers (if supports_mcp) - Phase 4
    /// - Directory rules (if supports_rules_directory) - Phase 2
    pub fn translate(
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> TranslatedContent {
        let mut content = TranslatedContent::empty();
        content.format = tool.integration.config_type;

        // Translate rules if supported
        if tool.capabilities.supports_custom_instructions {
            let rule_content = RuleTranslator::translate(tool, rules);
            if let Some(instructions) = rule_content.instructions {
                content.instructions = Some(instructions);
            }
        }

        // MCP translation will be added in Phase 4
        // Directory rules will be added in Phase 2

        content
    }

    /// Check if a tool has any capabilities that require translation
    pub fn has_translatable_capabilities(tool: &ToolDefinition) -> bool {
        tool.capabilities.supports_custom_instructions
            || tool.capabilities.supports_mcp
            || tool.capabilities.supports_rules_directory
    }

    /// Get list of capabilities a tool supports
    pub fn list_capabilities(tool: &ToolDefinition) -> Vec<&'static str> {
        let mut caps = Vec::new();
        if tool.capabilities.supports_custom_instructions {
            caps.push("custom_instructions");
        }
        if tool.capabilities.supports_mcp {
            caps.push("mcp");
        }
        if tool.capabilities.supports_rules_directory {
            caps.push("rules_directory");
        }
        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        ConfigType, RuleContent, RuleMeta, Severity,
        ToolCapabilities, ToolIntegrationConfig, ToolMeta,
    };

    fn make_tool(instructions: bool, mcp: bool, rules_dir: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: "Test".into(),
                slug: "test".into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: ".test".into(),
                config_type: ConfigType::Markdown,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: instructions,
                supports_mcp: mcp,
                supports_rules_directory: rules_dir,
            },
            schema_keys: None,
        }
    }

    fn make_rule(id: &str) -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta {
                id: id.into(),
                severity: Severity::Mandatory,
                tags: vec![],
            },
            content: RuleContent {
                instruction: format!("Rule {} instruction", id),
            },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_translate_with_instructions_capability() {
        let tool = make_tool(true, false, false);
        let rules = vec![make_rule("test-rule")];

        let content = CapabilityTranslator::translate(&tool, &rules);

        assert!(!content.is_empty());
        assert!(content.instructions.is_some());
        assert!(content.instructions.unwrap().contains("test-rule"));
    }

    #[test]
    fn test_translate_without_capabilities() {
        let tool = make_tool(false, false, false);
        let rules = vec![make_rule("test-rule")];

        let content = CapabilityTranslator::translate(&tool, &rules);

        assert!(content.is_empty());
    }

    #[test]
    fn test_has_translatable_capabilities() {
        assert!(CapabilityTranslator::has_translatable_capabilities(
            &make_tool(true, false, false)
        ));
        assert!(CapabilityTranslator::has_translatable_capabilities(
            &make_tool(false, true, false)
        ));
        assert!(CapabilityTranslator::has_translatable_capabilities(
            &make_tool(false, false, true)
        ));
        assert!(!CapabilityTranslator::has_translatable_capabilities(
            &make_tool(false, false, false)
        ));
    }

    #[test]
    fn test_list_capabilities() {
        let tool = make_tool(true, true, false);
        let caps = CapabilityTranslator::list_capabilities(&tool);

        assert!(caps.contains(&"custom_instructions"));
        assert!(caps.contains(&"mcp"));
        assert!(!caps.contains(&"rules_directory"));
    }

    #[test]
    fn test_content_has_correct_format() {
        let mut tool = make_tool(true, false, false);
        tool.integration.config_type = ConfigType::Json;
        let rules = vec![make_rule("test")];

        let content = CapabilityTranslator::translate(&tool, &rules);

        assert_eq!(content.format, ConfigType::Json);
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/translator/mod.rs`:
```rust
//! Capability-based translation from registry to tool configs
//!
//! This module translates canonical registry content (rules, MCP servers)
//! into tool-specific formats based on each tool's declared capabilities.
//!
//! # Architecture
//!
//! ```text
//! Registry (rules, MCP servers)
//!     │
//!     ▼
//! CapabilityTranslator (checks tool.capabilities)
//!     │
//!     ├── RuleTranslator (if supports_custom_instructions)
//!     ├── MCPTranslator (if supports_mcp) [Phase 4]
//!     └── DirectoryTranslator (if supports_rules_directory) [Phase 2]
//!     │
//!     ▼
//! TranslatedContent (ready for ConfigWriter)
//! ```

mod capability;
mod content;
mod rules;

pub use capability::CapabilityTranslator;
pub use content::TranslatedContent;
pub use rules::RuleTranslator;
```

**Step 3: Update lib.rs exports**

Update `crates/repo-tools/src/lib.rs`:
```rust
pub use translator::{CapabilityTranslator, RuleTranslator, TranslatedContent};
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools translator`
Expected: All tests pass (content: 4, rules: 6, capability: 5 = 15 tests)

**Step 5: Commit**

```bash
git add crates/repo-tools/src/translator/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add CapabilityTranslator orchestrator"
```

---

### Task 1.4: Create ConfigWriter trait and JsonWriter

**Files:**
- Create: `crates/repo-tools/src/writer/mod.rs`
- Create: `crates/repo-tools/src/writer/trait_def.rs`
- Create: `crates/repo-tools/src/writer/json.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create writer trait**

Create `crates/repo-tools/src/writer/trait_def.rs`:
```rust
//! ConfigWriter trait definition

use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::NormalizedPath;

/// Trait for writing translated content to config files
///
/// Each writer knows how to:
/// 1. Parse existing config (if any)
/// 2. Merge translated content with existing content
/// 3. Serialize and write the result
pub trait ConfigWriter: Send + Sync {
    /// Write translated content to a config file
    ///
    /// # Arguments
    /// * `path` - Path to the config file
    /// * `content` - Translated content to write
    /// * `schema_keys` - Optional keys for placing content in structured formats
    ///
    /// # Behavior
    /// - If file exists, parse and merge
    /// - If file doesn't exist, create new
    /// - Preserve user content not managed by the system
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        schema_keys: Option<&crate::SchemaKeys>,
    ) -> Result<()>;

    /// Check if this writer can handle the given file
    fn can_handle(&self, path: &NormalizedPath) -> bool;
}

/// Schema keys for placing content in structured configs
#[derive(Debug, Clone, Default)]
pub struct SchemaKeys {
    /// Key for instruction content (e.g., "customInstructions")
    pub instruction_key: Option<String>,
    /// Key for MCP servers (e.g., "mcpServers")
    pub mcp_key: Option<String>,
    /// Key for python path (e.g., "python.defaultInterpreterPath")
    pub python_path_key: Option<String>,
}

impl From<&repo_meta::schema::ToolSchemaKeys> for SchemaKeys {
    fn from(keys: &repo_meta::schema::ToolSchemaKeys) -> Self {
        Self {
            instruction_key: keys.instruction_key.clone(),
            mcp_key: keys.mcp_key.clone(),
            python_path_key: keys.python_path_key.clone(),
        }
    }
}
```

**Step 2: Create JSON writer**

Create `crates/repo-tools/src/writer/json.rs`:
```rust
//! JSON config writer with semantic merge

use super::trait_def::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};
use serde_json::{json, Map, Value};

/// Writer for JSON configuration files
///
/// Performs semantic merge: preserves existing keys not managed by the system.
pub struct JsonWriter;

impl JsonWriter {
    pub fn new() -> Self {
        Self
    }

    /// Parse existing JSON or return empty object
    fn parse_existing(path: &NormalizedPath) -> Value {
        if !path.exists() {
            return json!({});
        }

        match io::read_text(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or(json!({})),
            Err(_) => json!({}),
        }
    }

    /// Merge translated content into existing JSON
    fn merge(
        existing: &mut Value,
        content: &TranslatedContent,
        schema_keys: Option<&SchemaKeys>,
    ) {
        let obj = existing.as_object_mut().unwrap_or(&mut Map::new());

        // Apply instruction content if we have a key for it
        if let (Some(instructions), Some(keys)) = (&content.instructions, schema_keys) {
            if let Some(ref key) = keys.instruction_key {
                obj.insert(key.clone(), json!(instructions));
            }
        }

        // Apply MCP servers if we have a key for it
        if let (Some(servers), Some(keys)) = (&content.mcp_servers, schema_keys) {
            if let Some(ref key) = keys.mcp_key {
                obj.insert(key.clone(), servers.clone());
            }
        }

        // Apply additional data fields
        for (key, value) in &content.data {
            // Support nested keys like "python.defaultInterpreterPath"
            Self::set_nested(obj, key, value.clone());
        }
    }

    /// Set a potentially nested key (e.g., "python.path" -> {"python": {"path": value}})
    fn set_nested(obj: &mut Map<String, Value>, key: &str, value: Value) {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() == 1 {
            obj.insert(key.to_string(), value);
        } else {
            // Navigate/create nested structure
            let mut current = obj;
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    current.insert(part.to_string(), value.clone());
                } else {
                    current = current
                        .entry(part.to_string())
                        .or_insert_with(|| json!({}))
                        .as_object_mut()
                        .unwrap();
                }
            }
        }
    }
}

impl Default for JsonWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWriter for JsonWriter {
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        schema_keys: Option<&SchemaKeys>,
    ) -> Result<()> {
        let mut existing = Self::parse_existing(path);

        // Ensure it's an object
        if !existing.is_object() {
            existing = json!({});
        }

        Self::merge(&mut existing, content, schema_keys);

        let output = serde_json::to_string_pretty(&existing)?;
        io::write_text(path, &output)?;

        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        path.as_str().ends_with(".json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::ConfigType;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_to_new_file() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("config.json");

        let content = TranslatedContent::with_instructions(
            ConfigType::Json,
            "Test instructions".into(),
        );
        let keys = SchemaKeys {
            instruction_key: Some("customInstructions".into()),
            ..Default::default()
        };

        let writer = JsonWriter::new();
        writer.write(&path, &content, Some(&keys)).unwrap();

        let result: Value = serde_json::from_str(&fs::read_to_string(path.to_native()).unwrap()).unwrap();
        assert_eq!(result["customInstructions"], "Test instructions");
    }

    #[test]
    fn test_preserves_existing_content() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("config.json");

        // Create existing file with user content
        let existing = json!({
            "userSetting": true,
            "editor.fontSize": 14
        });
        fs::write(path.to_native(), serde_json::to_string(&existing).unwrap()).unwrap();

        // Write translated content
        let content = TranslatedContent::with_instructions(
            ConfigType::Json,
            "New instructions".into(),
        );
        let keys = SchemaKeys {
            instruction_key: Some("customInstructions".into()),
            ..Default::default()
        };

        let writer = JsonWriter::new();
        writer.write(&path, &content, Some(&keys)).unwrap();

        let result: Value = serde_json::from_str(&fs::read_to_string(path.to_native()).unwrap()).unwrap();

        // New content added
        assert_eq!(result["customInstructions"], "New instructions");
        // Existing content preserved
        assert_eq!(result["userSetting"], true);
        assert_eq!(result["editor.fontSize"], 14);
    }

    #[test]
    fn test_nested_key_insertion() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("config.json");

        let content = TranslatedContent::empty()
            .with_data("python.defaultInterpreterPath", json!("/usr/bin/python3"));

        let writer = JsonWriter::new();
        writer.write(&path, &content, None).unwrap();

        let result: Value = serde_json::from_str(&fs::read_to_string(path.to_native()).unwrap()).unwrap();
        assert_eq!(result["python"]["defaultInterpreterPath"], "/usr/bin/python3");
    }

    #[test]
    fn test_can_handle() {
        let writer = JsonWriter::new();
        assert!(writer.can_handle(&NormalizedPath::new("config.json")));
        assert!(writer.can_handle(&NormalizedPath::new(".vscode/settings.json")));
        assert!(!writer.can_handle(&NormalizedPath::new("config.toml")));
    }
}
```

**Step 3: Create mod.rs**

Create `crates/repo-tools/src/writer/mod.rs`:
```rust
//! Config writers for different file formats
//!
//! Writers handle semantic merge of translated content with existing files.

mod json;
mod trait_def;

pub use json::JsonWriter;
pub use trait_def::{ConfigWriter, SchemaKeys};
```

**Step 4: Update lib.rs**

Add to `crates/repo-tools/src/lib.rs`:
```rust
pub mod writer;
pub use writer::{ConfigWriter, JsonWriter, SchemaKeys};
```

**Step 5: Run tests**

Run: `cargo test -p repo-tools writer`
Expected: All 4 tests pass

**Step 6: Commit**

```bash
git add crates/repo-tools/src/writer/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add ConfigWriter trait and JsonWriter"
```

---

### Task 1.5: Create MarkdownWriter

**Files:**
- Create: `crates/repo-tools/src/writer/markdown.rs`
- Modify: `crates/repo-tools/src/writer/mod.rs`

**Step 1: Create markdown writer**

Create `crates/repo-tools/src/writer/markdown.rs`:
```rust
//! Markdown config writer with section-based merge

use super::trait_def::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};

/// Marker for managed content sections
const MANAGED_START: &str = "<!-- repo:managed:start -->";
const MANAGED_END: &str = "<!-- repo:managed:end -->";

/// Writer for Markdown configuration files (CLAUDE.md, etc.)
///
/// Uses section markers to separate managed content from user content.
pub struct MarkdownWriter;

impl MarkdownWriter {
    pub fn new() -> Self {
        Self
    }

    /// Parse existing file into user content and managed content
    fn parse_existing(path: &NormalizedPath) -> (String, String) {
        if !path.exists() {
            return (String::new(), String::new());
        }

        let content = match io::read_text(path) {
            Ok(c) => c,
            Err(_) => return (String::new(), String::new()),
        };

        // Find managed section
        if let (Some(start), Some(end)) = (
            content.find(MANAGED_START),
            content.find(MANAGED_END),
        ) {
            let user_before = content[..start].trim_end();
            let user_after = content[end + MANAGED_END.len()..].trim_start();

            let user_content = if user_after.is_empty() {
                user_before.to_string()
            } else {
                format!("{}\n\n{}", user_before, user_after)
            };

            (user_content, String::new()) // Managed content will be replaced
        } else {
            // No managed section - all content is user content
            (content, String::new())
        }
    }

    /// Combine user content with managed content
    fn combine(user_content: &str, managed_content: &str) -> String {
        let mut output = String::new();

        // User content first (if any)
        if !user_content.is_empty() {
            output.push_str(user_content);
            output.push_str("\n\n");
        }

        // Then managed section
        output.push_str(MANAGED_START);
        output.push('\n');
        output.push_str(managed_content);
        output.push('\n');
        output.push_str(MANAGED_END);
        output.push('\n');

        output
    }
}

impl Default for MarkdownWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWriter for MarkdownWriter {
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        _schema_keys: Option<&SchemaKeys>,
    ) -> Result<()> {
        let (user_content, _) = Self::parse_existing(path);

        let managed_content = content.instructions.as_deref().unwrap_or("");
        let output = Self::combine(&user_content, managed_content);

        io::write_text(path, &output)?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        let p = path.as_str();
        p.ends_with(".md") || p.ends_with(".markdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::ConfigType;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_to_new_file() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("CLAUDE.md");

        let content = TranslatedContent::with_instructions(
            ConfigType::Markdown,
            "## Rule 1\n\nDo this thing.".into(),
        );

        let writer = MarkdownWriter::new();
        writer.write(&path, &content, None).unwrap();

        let result = fs::read_to_string(path.to_native()).unwrap();
        assert!(result.contains(MANAGED_START));
        assert!(result.contains("## Rule 1"));
        assert!(result.contains(MANAGED_END));
    }

    #[test]
    fn test_preserves_user_content() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("CLAUDE.md");

        // Create file with user content
        let user_content = "# My Project\n\nThis is my custom documentation.\n";
        fs::write(path.to_native(), user_content).unwrap();

        // Write managed content
        let content = TranslatedContent::with_instructions(
            ConfigType::Markdown,
            "Managed rules here.".into(),
        );

        let writer = MarkdownWriter::new();
        writer.write(&path, &content, None).unwrap();

        let result = fs::read_to_string(path.to_native()).unwrap();

        // User content preserved
        assert!(result.contains("# My Project"));
        assert!(result.contains("This is my custom documentation."));

        // Managed content added
        assert!(result.contains("Managed rules here."));
        assert!(result.contains(MANAGED_START));
    }

    #[test]
    fn test_updates_managed_section() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join("CLAUDE.md");

        // Create file with existing managed section
        let existing = format!(
            "# User Header\n\n{}\nOld managed content\n{}\n",
            MANAGED_START, MANAGED_END
        );
        fs::write(path.to_native(), &existing).unwrap();

        // Write new managed content
        let content = TranslatedContent::with_instructions(
            ConfigType::Markdown,
            "New managed content".into(),
        );

        let writer = MarkdownWriter::new();
        writer.write(&path, &content, None).unwrap();

        let result = fs::read_to_string(path.to_native()).unwrap();

        // User content preserved
        assert!(result.contains("# User Header"));

        // Old managed content replaced
        assert!(!result.contains("Old managed content"));
        assert!(result.contains("New managed content"));

        // Only one managed section
        assert_eq!(result.matches(MANAGED_START).count(), 1);
    }

    #[test]
    fn test_can_handle() {
        let writer = MarkdownWriter::new();
        assert!(writer.can_handle(&NormalizedPath::new("CLAUDE.md")));
        assert!(writer.can_handle(&NormalizedPath::new("README.markdown")));
        assert!(!writer.can_handle(&NormalizedPath::new("config.json")));
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/writer/mod.rs`:
```rust
//! Config writers for different file formats
//!
//! Writers handle semantic merge of translated content with existing files.

mod json;
mod markdown;
mod trait_def;

pub use json::JsonWriter;
pub use markdown::MarkdownWriter;
pub use trait_def::{ConfigWriter, SchemaKeys};
```

**Step 3: Update lib.rs**

Update `crates/repo-tools/src/lib.rs`:
```rust
pub use writer::{ConfigWriter, JsonWriter, MarkdownWriter, SchemaKeys};
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools writer`
Expected: All 8 tests pass (json: 4, markdown: 4)

**Step 5: Commit**

```bash
git add crates/repo-tools/src/writer/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add MarkdownWriter with section-based merge"
```

---

### Task 1.6: Create TextWriter for plain text configs

**Files:**
- Create: `crates/repo-tools/src/writer/text.rs`
- Modify: `crates/repo-tools/src/writer/mod.rs`

**Step 1: Create text writer**

Create `crates/repo-tools/src/writer/text.rs`:
```rust
//! Plain text config writer (for .cursorrules, etc.)

use super::trait_def::{ConfigWriter, SchemaKeys};
use crate::error::Result;
use crate::translator::TranslatedContent;
use repo_fs::{io, NormalizedPath};

/// Writer for plain text configuration files (.cursorrules, etc.)
///
/// For plain text, the translated content IS the entire file.
/// These files are typically tool-specific and don't support user sections.
pub struct TextWriter;

impl TextWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigWriter for TextWriter {
    fn write(
        &self,
        path: &NormalizedPath,
        content: &TranslatedContent,
        _schema_keys: Option<&SchemaKeys>,
    ) -> Result<()> {
        let output = content.instructions.as_deref().unwrap_or("");
        io::write_text(path, output)?;
        Ok(())
    }

    fn can_handle(&self, path: &NormalizedPath) -> bool {
        // Text writer is the fallback - can handle anything
        // But should be used last in the chain
        let p = path.as_str();
        !p.ends_with(".json")
            && !p.ends_with(".yaml")
            && !p.ends_with(".yml")
            && !p.ends_with(".toml")
            && !p.ends_with(".md")
            && !p.ends_with(".markdown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::ConfigType;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_plain_text() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join(".cursorrules");

        let content = TranslatedContent::with_instructions(
            ConfigType::Text,
            "Rule 1: Do this\nRule 2: Do that".into(),
        );

        let writer = TextWriter::new();
        writer.write(&path, &content, None).unwrap();

        let result = fs::read_to_string(path.to_native()).unwrap();
        assert_eq!(result, "Rule 1: Do this\nRule 2: Do that");
    }

    #[test]
    fn test_overwrites_existing() {
        let temp = TempDir::new().unwrap();
        let path = NormalizedPath::new(temp.path()).join(".cursorrules");

        // Create existing file
        fs::write(path.to_native(), "Old content").unwrap();

        // Write new content
        let content = TranslatedContent::with_instructions(
            ConfigType::Text,
            "New content".into(),
        );

        let writer = TextWriter::new();
        writer.write(&path, &content, None).unwrap();

        let result = fs::read_to_string(path.to_native()).unwrap();
        assert_eq!(result, "New content");
        assert!(!result.contains("Old content"));
    }

    #[test]
    fn test_can_handle() {
        let writer = TextWriter::new();
        assert!(writer.can_handle(&NormalizedPath::new(".cursorrules")));
        assert!(writer.can_handle(&NormalizedPath::new(".windsurfrules")));
        assert!(!writer.can_handle(&NormalizedPath::new("config.json")));
        assert!(!writer.can_handle(&NormalizedPath::new("CLAUDE.md")));
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/writer/mod.rs`:
```rust
//! Config writers for different file formats

mod json;
mod markdown;
mod text;
mod trait_def;

pub use json::JsonWriter;
pub use markdown::MarkdownWriter;
pub use text::TextWriter;
pub use trait_def::{ConfigWriter, SchemaKeys};
```

**Step 3: Update lib.rs**

Update `crates/repo-tools/src/lib.rs`:
```rust
pub use writer::{ConfigWriter, JsonWriter, MarkdownWriter, SchemaKeys, TextWriter};
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools writer`
Expected: All 11 tests pass

**Step 5: Commit**

```bash
git add crates/repo-tools/src/writer/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add TextWriter for plain text configs"
```

---

### Task 1.7: Create WriterRegistry to select appropriate writer

**Files:**
- Create: `crates/repo-tools/src/writer/registry.rs`
- Modify: `crates/repo-tools/src/writer/mod.rs`

**Step 1: Create writer registry**

Create `crates/repo-tools/src/writer/registry.rs`:
```rust
//! Registry of config writers by format

use super::{ConfigWriter, JsonWriter, MarkdownWriter, TextWriter};
use repo_meta::schema::ConfigType;

/// Registry that selects the appropriate writer for a config type
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

    /// Get the appropriate writer for a config type
    pub fn get_writer(&self, config_type: ConfigType) -> &dyn ConfigWriter {
        match config_type {
            ConfigType::Json => &self.json,
            ConfigType::Markdown => &self.markdown,
            ConfigType::Text => &self.text,
            ConfigType::Yaml | ConfigType::Toml => {
                // TODO: Add YamlWriter and TomlWriter in Phase 3
                // For now, fall back to text
                &self.text
            }
        }
    }
}

impl Default for WriterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_fs::NormalizedPath;

    #[test]
    fn test_get_json_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Json);
        assert!(writer.can_handle(&NormalizedPath::new("test.json")));
    }

    #[test]
    fn test_get_markdown_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Markdown);
        assert!(writer.can_handle(&NormalizedPath::new("test.md")));
    }

    #[test]
    fn test_get_text_writer() {
        let registry = WriterRegistry::new();
        let writer = registry.get_writer(ConfigType::Text);
        assert!(writer.can_handle(&NormalizedPath::new(".cursorrules")));
    }
}
```

**Step 2: Update mod.rs**

Update `crates/repo-tools/src/writer/mod.rs`:
```rust
//! Config writers for different file formats

mod json;
mod markdown;
mod registry;
mod text;
mod trait_def;

pub use json::JsonWriter;
pub use markdown::MarkdownWriter;
pub use registry::WriterRegistry;
pub use text::TextWriter;
pub use trait_def::{ConfigWriter, SchemaKeys};
```

**Step 3: Update lib.rs**

Add `WriterRegistry` to exports in `crates/repo-tools/src/lib.rs`:
```rust
pub use writer::{ConfigWriter, JsonWriter, MarkdownWriter, SchemaKeys, TextWriter, WriterRegistry};
```

**Step 4: Run tests**

Run: `cargo test -p repo-tools writer`
Expected: All 14 tests pass

**Step 5: Commit**

```bash
git add crates/repo-tools/src/writer/
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add WriterRegistry for format selection"
```

---

### Task 1.8: Create ToolSyncer that ties everything together

**Files:**
- Create: `crates/repo-tools/src/syncer.rs`
- Modify: `crates/repo-tools/src/lib.rs`

**Step 1: Create ToolSyncer**

Create `crates/repo-tools/src/syncer.rs`:
```rust
//! ToolSyncer - the main entry point for capability-based config sync
//!
//! This replaces the ad-hoc sync logic with a proper capability-driven flow:
//! 1. Load rules from registry
//! 2. Translate based on tool capabilities
//! 3. Write using format-appropriate writer

use crate::error::Result;
use crate::translator::CapabilityTranslator;
use crate::writer::{SchemaKeys, WriterRegistry};
use repo_fs::NormalizedPath;
use repo_meta::schema::{RuleDefinition, ToolDefinition};

/// Syncs registry content to tool configuration files
///
/// This is the main entry point for the capability-based architecture.
pub struct ToolSyncer {
    writer_registry: WriterRegistry,
}

impl ToolSyncer {
    pub fn new() -> Self {
        Self {
            writer_registry: WriterRegistry::new(),
        }
    }

    /// Sync rules to a tool's configuration
    ///
    /// # Arguments
    /// * `root` - Repository root path
    /// * `tool` - Tool definition with capabilities
    /// * `rules` - Rules to sync
    ///
    /// # Returns
    /// * `Ok(true)` - Content was written
    /// * `Ok(false)` - No content to write (tool doesn't support capabilities)
    /// * `Err(_)` - Write failed
    pub fn sync(
        &self,
        root: &NormalizedPath,
        tool: &ToolDefinition,
        rules: &[RuleDefinition],
    ) -> Result<bool> {
        // Step 1: Check if tool has any translatable capabilities
        if !CapabilityTranslator::has_translatable_capabilities(tool) {
            return Ok(false);
        }

        // Step 2: Translate content based on capabilities
        let content = CapabilityTranslator::translate(tool, rules);

        if content.is_empty() {
            return Ok(false);
        }

        // Step 3: Get the appropriate writer
        let writer = self.writer_registry.get_writer(tool.integration.config_type);

        // Step 4: Build schema keys if available
        let schema_keys = tool.schema_keys.as_ref().map(SchemaKeys::from);

        // Step 5: Write to config file
        let config_path = root.join(&tool.integration.config_path);
        writer.write(&config_path, &content, schema_keys.as_ref())?;

        Ok(true)
    }

    /// Sync rules to multiple tools
    ///
    /// Returns list of tool slugs that were successfully synced.
    pub fn sync_all(
        &self,
        root: &NormalizedPath,
        tools: &[ToolDefinition],
        rules: &[RuleDefinition],
    ) -> Result<Vec<String>> {
        let mut synced = Vec::new();

        for tool in tools {
            if self.sync(root, tool, rules)? {
                synced.push(tool.meta.slug.clone());
            }
        }

        Ok(synced)
    }
}

impl Default for ToolSyncer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use repo_meta::schema::{
        ConfigType, RuleContent, RuleMeta, Severity,
        ToolCapabilities, ToolIntegrationConfig, ToolMeta, ToolSchemaKeys,
    };
    use std::fs;
    use tempfile::TempDir;

    fn make_tool(slug: &str, config_type: ConfigType, supports_instructions: bool) -> ToolDefinition {
        ToolDefinition {
            meta: ToolMeta {
                name: slug.into(),
                slug: slug.into(),
                description: None,
            },
            integration: ToolIntegrationConfig {
                config_path: format!(".{}", slug),
                config_type,
                additional_paths: vec![],
            },
            capabilities: ToolCapabilities {
                supports_custom_instructions: supports_instructions,
                supports_mcp: false,
                supports_rules_directory: false,
            },
            schema_keys: None,
        }
    }

    fn make_rule(id: &str) -> RuleDefinition {
        RuleDefinition {
            meta: RuleMeta {
                id: id.into(),
                severity: Severity::Mandatory,
                tags: vec![],
            },
            content: RuleContent {
                instruction: format!("{} instruction content", id),
            },
            examples: None,
            targets: None,
        }
    }

    #[test]
    fn test_sync_with_capable_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());

        let tool = make_tool("testrules", ConfigType::Text, true);
        let rules = vec![make_rule("test-rule")];

        let syncer = ToolSyncer::new();
        let result = syncer.sync(&root, &tool, &rules).unwrap();

        assert!(result, "Should return true when content is written");

        let config_path = temp.path().join(".testrules");
        assert!(config_path.exists(), "Config file should be created");

        let content = fs::read_to_string(config_path).unwrap();
        assert!(content.contains("test-rule"));
    }

    #[test]
    fn test_sync_with_incapable_tool() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());

        let tool = make_tool("notool", ConfigType::Text, false);
        let rules = vec![make_rule("test-rule")];

        let syncer = ToolSyncer::new();
        let result = syncer.sync(&root, &tool, &rules).unwrap();

        assert!(!result, "Should return false when tool has no capabilities");

        let config_path = temp.path().join(".notool");
        assert!(!config_path.exists(), "Config file should NOT be created");
    }

    #[test]
    fn test_sync_json_with_schema_keys() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());

        let mut tool = make_tool("config", ConfigType::Json, true);
        tool.integration.config_path = "config.json".into();
        tool.schema_keys = Some(ToolSchemaKeys {
            instruction_key: Some("customInstructions".into()),
            mcp_key: None,
            python_path_key: None,
        });

        let rules = vec![make_rule("json-rule")];

        let syncer = ToolSyncer::new();
        syncer.sync(&root, &tool, &rules).unwrap();

        let config_path = temp.path().join("config.json");
        let content = fs::read_to_string(config_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(json.get("customInstructions").is_some());
    }

    #[test]
    fn test_sync_all() {
        let temp = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp.path());

        let tools = vec![
            make_tool("tool1", ConfigType::Text, true),
            make_tool("tool2", ConfigType::Text, false), // No capability
            make_tool("tool3", ConfigType::Text, true),
        ];
        let rules = vec![make_rule("rule1")];

        let syncer = ToolSyncer::new();
        let synced = syncer.sync_all(&root, &tools, &rules).unwrap();

        assert_eq!(synced.len(), 2);
        assert!(synced.contains(&"tool1".to_string()));
        assert!(!synced.contains(&"tool2".to_string()));
        assert!(synced.contains(&"tool3".to_string()));
    }
}
```

**Step 2: Update lib.rs**

Add to `crates/repo-tools/src/lib.rs`:
```rust
mod syncer;
pub use syncer::ToolSyncer;
```

**Step 3: Run tests**

Run: `cargo test -p repo-tools syncer`
Expected: All 4 tests pass

**Step 4: Run full test suite**

Run: `cargo test -p repo-tools`
Expected: All tests pass

**Step 5: Commit**

```bash
git add crates/repo-tools/src/syncer.rs
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): add ToolSyncer for capability-based config sync

This completes Phase 1 of the capability-based registry architecture:
- TranslatedContent for holding translated data
- RuleTranslator that respects capabilities
- CapabilityTranslator orchestrator
- ConfigWriter trait with JSON, Markdown, Text implementations
- WriterRegistry for format selection
- ToolSyncer as the main entry point

Capabilities now actually control what gets generated."
```

---

## Phase 1 Complete - Summary

After Phase 1, we have:

| Component | Purpose |
|-----------|---------|
| `TranslatedContent` | Holds translated rules, MCP, and data |
| `RuleTranslator` | Translates rules using severity, examples, targets |
| `CapabilityTranslator` | Checks capabilities before translation |
| `ConfigWriter` | Trait for format-specific writing |
| `JsonWriter` | Semantic merge for JSON configs |
| `MarkdownWriter` | Section-based merge for MD files |
| `TextWriter` | Full replacement for plain text |
| `WriterRegistry` | Selects writer by format |
| `ToolSyncer` | Main entry point tying it all together |

**Key Achievement:** `ToolCapabilities` now actually controls what gets generated.

---

## Phase 2: Semantic Rule Translation (Future)

Tasks to add:
- DirectoryRuleTranslator for rules_directory capability
- Per-file rule targeting using RuleTargets.file_patterns
- Rule grouping by tags

## Phase 3: AST-Aware Config Writers (Future)

Tasks to add:
- YamlWriter with proper YAML parsing
- TomlWriter with proper TOML parsing
- Conflict detection and resolution

## Phase 4: MCP Server Support (Future)

Tasks to add:
- MCPServerDefinition schema in repo-meta
- MCPTranslator in translator module
- MCP config generation for supporting tools

---

## Success Criteria - Phase 1

- [ ] `cargo test -p repo-tools` - All tests pass
- [ ] `cargo clippy -p repo-tools` - No warnings
- [ ] `ToolCapabilities.supports_custom_instructions = false` → No rules in output
- [ ] `ToolCapabilities.supports_mcp = false` → No MCP in output (trivially true, not implemented)
- [ ] JSON configs preserve user content
- [ ] Markdown configs use section markers
- [ ] Existing dispatcher tests still pass

---

*Plan created: 2026-01-29*
*Phases: 1 of 4 detailed*
