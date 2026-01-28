# Modern AI Coding Tools Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add built-in integrations for 7 modern AI coding tools: GitHub Copilot, Cline, Roo Code, JetBrains AI, Zed, Aider, and Amazon Q.

**Architecture:** Each tool gets a dedicated module following the existing pattern (cursor.rs, windsurf.rs). Each module exports a factory function that returns a `GenericToolIntegration` configured with the tool's `ToolDefinition`. The `ToolDispatcher` is updated to route to these new integrations.

**Tech Stack:** Rust, repo-tools crate, repo-meta schema types, repo-blocks for managed blocks

---

## Reference: Existing Pattern

All integrations follow this pattern from `cursor.rs`:

```rust
pub fn tool_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta { name, slug, description },
        integration: ToolIntegrationConfig { config_path, config_type, additional_paths },
        capabilities: ToolCapabilities { ... },
        schema_keys: None,
    })
}
```

---

## Task 1: GitHub Copilot Integration

**Files:**
- Create: `crates/repo-tools/src/copilot.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Create copilot.rs

```rust
//! GitHub Copilot integration for Repository Manager.
//!
//! Manages `.github/copilot-instructions.md` file using managed blocks.
//! Also supports path-specific instructions in `.github/instructions/`.
//!
//! Reference: https://docs.github.com/copilot/customizing-copilot/adding-custom-instructions-for-github-copilot

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a GitHub Copilot integration.
///
/// Configuration files:
/// - `.github/copilot-instructions.md` - Main instructions file (Markdown)
/// - `.github/instructions/` - Directory for path-specific `.instructions.md` files
///
/// Format: Markdown with optional YAML frontmatter for path-specific files:
/// ```yaml
/// ---
/// applyTo: "**/*.py"
/// ---
/// ```
pub fn copilot_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "GitHub Copilot".into(),
            slug: "copilot".into(),
            description: Some("GitHub Copilot AI coding assistant".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".github/copilot-instructions.md".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![".github/instructions/".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{Rule, SyncContext, ToolIntegration};
    use repo_fs::NormalizedPath;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = copilot_integration();
        assert_eq!(integration.name(), "copilot");
    }

    #[test]
    fn test_config_locations() {
        let integration = copilot_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".github/copilot-instructions.md");
        assert!(!locations[0].is_directory);
        assert_eq!(locations[1].path, ".github/instructions/");
        assert!(locations[1].is_directory);
    }

    #[test]
    fn test_sync_creates_instructions() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        // Create .github directory
        fs::create_dir_all(temp_dir.path().join(".github")).unwrap();

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "python-style".to_string(),
            content: "Use type hints for all function parameters.".to_string(),
        }];

        let integration = copilot_integration();
        integration.sync(&context, &rules).unwrap();

        let path = temp_dir.path().join(".github/copilot-instructions.md");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("python-style"));
        assert!(content.contains("Use type hints"));
    }
}
```

### Step 2: Add to lib.rs exports

Add to `crates/repo-tools/src/lib.rs`:

```rust
pub mod copilot;
pub use copilot::copilot_integration;
```

### Step 3: Add to dispatcher.rs

In the `get_integration` match block, add:

```rust
"copilot" => return Some(Box::new(copilot_integration())),
```

In `has_tool` matches, add `"copilot"`.

In `list_available`, add `"copilot".to_string()`.

### Step 4: Run tests

```bash
cargo test -p repo-tools copilot
```

### Step 5: Commit

```bash
git add crates/repo-tools/src/copilot.rs crates/repo-tools/src/lib.rs crates/repo-tools/src/dispatcher.rs
git commit -m "feat(repo-tools): add GitHub Copilot integration

Adds support for .github/copilot-instructions.md and
.github/instructions/ directory for path-specific rules."
```

---

## Task 2: Cline Integration

**Files:**
- Create: `crates/repo-tools/src/cline.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Create cline.rs

```rust
//! Cline (VS Code) integration for Repository Manager.
//!
//! Manages `.clinerules` file or `.clinerules/` directory using managed blocks.
//!
//! Reference: https://docs.cline.bot/features/cline-rules

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a Cline integration.
///
/// Configuration files:
/// - `.clinerules` - Single rules file (Markdown/Text)
/// - `.clinerules/` - Directory of rule files (*.md)
///
/// Cline also reads `.cursorrules` and `AGENTS.md` as fallbacks.
/// Files in directory are processed alphabetically (use `01-`, `02-` prefixes).
pub fn cline_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Cline".into(),
            slug: "cline".into(),
            description: Some("Cline AI coding assistant for VS Code".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".clinerules".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![".clinerules/".into()],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
    .with_raw_content(true) // No headers, direct content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{Rule, SyncContext, ToolIntegration};
    use repo_fs::NormalizedPath;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = cline_integration();
        assert_eq!(integration.name(), "cline");
    }

    #[test]
    fn test_config_locations() {
        let integration = cline_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".clinerules");
        assert_eq!(locations[1].path, ".clinerules/");
        assert!(locations[1].is_directory);
    }

    #[test]
    fn test_sync_creates_clinerules() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "coding-style".to_string(),
            content: "Use TypeScript strict mode.".to_string(),
        }];

        let integration = cline_integration();
        integration.sync(&context, &rules).unwrap();

        let path = temp_dir.path().join(".clinerules");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Use TypeScript strict mode"));
        // Raw content mode - no headers
        assert!(!content.contains("## coding-style"));
    }
}
```

### Step 2: Add to lib.rs and dispatcher.rs (same pattern as Task 1)

### Step 3: Run tests and commit

```bash
cargo test -p repo-tools cline
git add crates/repo-tools/src/cline.rs crates/repo-tools/src/lib.rs crates/repo-tools/src/dispatcher.rs
git commit -m "feat(repo-tools): add Cline integration

Adds support for .clinerules file and .clinerules/ directory."
```

---

## Task 3: Roo Code Integration

**Files:**
- Create: `crates/repo-tools/src/roo.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Create roo.rs

```rust
//! Roo Code integration for Repository Manager.
//!
//! Manages `.roo/rules/` directory and `.roomodes` file.
//!
//! Reference: https://docs.roocode.com/features/custom-instructions

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a Roo Code integration.
///
/// Configuration files:
/// - `.roo/rules/` - Directory of instruction files (*.md, *.txt)
/// - `.roo/rules-{mode}/` - Mode-specific rules directories
/// - `.roomodes` - Custom modes configuration (YAML or JSON)
///
/// Files are loaded recursively in alphabetical order.
/// Workspace rules override global rules (~/.roo/rules/).
pub fn roo_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Roo Code".into(),
            slug: "roo".into(),
            description: Some("Roo Code AI assistant (fork of Cline)".into()),
        },
        integration: ToolIntegrationConfig {
            // Primary path is the rules directory
            config_path: ".roo/rules/".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![
                ".roomodes".into(),
            ],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::ToolIntegration;

    #[test]
    fn test_name() {
        let integration = roo_integration();
        assert_eq!(integration.name(), "roo");
    }

    #[test]
    fn test_config_locations() {
        let integration = roo_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".roo/rules/");
        assert!(locations[0].is_directory);
        assert_eq!(locations[1].path, ".roomodes");
    }
}
```

### Step 2-3: Add to lib.rs, dispatcher.rs, test, and commit

```bash
git commit -m "feat(repo-tools): add Roo Code integration

Adds support for .roo/rules/ directory and .roomodes configuration."
```

---

## Task 4: JetBrains AI Integration

**Files:**
- Create: `crates/repo-tools/src/jetbrains.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Create jetbrains.rs

```rust
//! JetBrains AI Assistant integration for Repository Manager.
//!
//! Manages `.aiassistant/rules/` directory for project-specific AI rules.
//!
//! Reference: https://www.jetbrains.com/help/ai-assistant/configure-project-rules.html

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a JetBrains AI Assistant integration.
///
/// Configuration files:
/// - `.aiassistant/rules/` - Directory of rule files (*.md)
/// - `.aiignore` - Files to exclude from AI (gitignore syntax)
/// - `.noai` - Empty file to disable AI for entire project
///
/// Rule types: Always, Manually (@rule:), By Model Decision, By File Patterns, Off
/// Also reads: .cursorignore, .codeiumignore, .aiexclude
pub fn jetbrains_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "JetBrains AI".into(),
            slug: "jetbrains".into(),
            description: Some("JetBrains AI Assistant for IntelliJ IDEs".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".aiassistant/rules/".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![
                ".aiignore".into(),
            ],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true, // Supports MCP servers
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::ToolIntegration;

    #[test]
    fn test_name() {
        let integration = jetbrains_integration();
        assert_eq!(integration.name(), "jetbrains");
    }

    #[test]
    fn test_config_locations() {
        let integration = jetbrains_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".aiassistant/rules/");
        assert!(locations[0].is_directory);
        assert_eq!(locations[1].path, ".aiignore");
    }
}
```

### Step 2-3: Add to lib.rs, dispatcher.rs, test, and commit

```bash
git commit -m "feat(repo-tools): add JetBrains AI Assistant integration

Adds support for .aiassistant/rules/ directory and .aiignore."
```

---

## Task 5: Zed Integration

**Files:**
- Create: `crates/repo-tools/src/zed.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Create zed.rs

```rust
//! Zed editor integration for Repository Manager.
//!
//! Manages `.rules` file for AI agent instructions.
//!
//! Reference: https://zed.dev/docs/ai/rules

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates a Zed editor integration.
///
/// Configuration files:
/// - `.rules` - Project rules file (highest priority)
/// - `.zed/settings.json` - Project settings (for AI model config)
///
/// Priority order: .rules > .cursorrules > .windsurfrules > .clinerules >
///   .github/copilot-instructions.md > AGENT.md > AGENTS.md > CLAUDE.md > GEMINI.md
///
/// Only the first matching file is loaded.
pub fn zed_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Zed".into(),
            slug: "zed".into(),
            description: Some("Zed code editor with AI agent".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".rules".into(),
            config_type: ConfigType::Text,
            additional_paths: vec![
                ".zed/settings.json".into(),
            ],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: true,
            supports_rules_directory: false,
        },
        schema_keys: None,
    })
    .with_raw_content(true) // Direct content, no headers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::{Rule, SyncContext, ToolIntegration};
    use repo_fs::NormalizedPath;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_name() {
        let integration = zed_integration();
        assert_eq!(integration.name(), "zed");
    }

    #[test]
    fn test_config_locations() {
        let integration = zed_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".rules");
        assert_eq!(locations[1].path, ".zed/settings.json");
    }

    #[test]
    fn test_sync_creates_rules() {
        let temp_dir = TempDir::new().unwrap();
        let root = NormalizedPath::new(temp_dir.path());

        let context = SyncContext::new(root);
        let rules = vec![Rule {
            id: "code-style".to_string(),
            content: "Use Rust best practices.".to_string(),
        }];

        let integration = zed_integration();
        integration.sync(&context, &rules).unwrap();

        let path = temp_dir.path().join(".rules");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Use Rust best practices"));
    }
}
```

### Step 2-3: Add to lib.rs, dispatcher.rs, test, and commit

```bash
git commit -m "feat(repo-tools): add Zed editor integration

Adds support for .rules file and .zed/settings.json."
```

---

## Task 6: Aider Integration

**Files:**
- Create: `crates/repo-tools/src/aider.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Create aider.rs

```rust
//! Aider integration for Repository Manager.
//!
//! Manages `.aider.conf.yml` configuration file.
//!
//! Reference: https://aider.chat/docs/config/aider_conf.html

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates an Aider integration.
///
/// Configuration files:
/// - `.aider.conf.yml` - Project configuration (YAML)
/// - `CONVENTIONS.md` - Coding conventions (loaded via `read:` config)
///
/// Config priority: home dir < git root < current dir (last wins)
/// Environment variables: AIDER_xxx
pub fn aider_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Aider".into(),
            slug: "aider".into(),
            description: Some("Aider AI pair programming CLI".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".aider.conf.yml".into(),
            config_type: ConfigType::Yaml,
            additional_paths: vec![
                "CONVENTIONS.md".into(),
            ],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: false,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::ToolIntegration;

    #[test]
    fn test_name() {
        let integration = aider_integration();
        assert_eq!(integration.name(), "aider");
    }

    #[test]
    fn test_config_locations() {
        let integration = aider_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 2);
        assert_eq!(locations[0].path, ".aider.conf.yml");
        assert_eq!(locations[1].path, "CONVENTIONS.md");
    }
}
```

### Step 2-3: Add to lib.rs, dispatcher.rs, test, and commit

```bash
git commit -m "feat(repo-tools): add Aider integration

Adds support for .aider.conf.yml and CONVENTIONS.md."
```

---

## Task 7: Amazon Q Integration

**Files:**
- Create: `crates/repo-tools/src/amazonq.rs`
- Modify: `crates/repo-tools/src/lib.rs`
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Create amazonq.rs

```rust
//! Amazon Q Developer integration for Repository Manager.
//!
//! Manages `.amazonq/rules/` directory for project-specific rules.
//!
//! Reference: https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/context-project-rules.html

use crate::generic::GenericToolIntegration;
use repo_meta::schema::{ConfigType, ToolCapabilities, ToolDefinition, ToolIntegrationConfig, ToolMeta};

/// Creates an Amazon Q Developer integration.
///
/// Configuration files:
/// - `.amazonq/rules/` - Directory of rule files (*.md)
///
/// Rules are automatically applied to all chat sessions.
/// Individual rules can be toggled on/off via the Rules button in chat.
pub fn amazonq_integration() -> GenericToolIntegration {
    GenericToolIntegration::new(ToolDefinition {
        meta: ToolMeta {
            name: "Amazon Q".into(),
            slug: "amazonq".into(),
            description: Some("Amazon Q Developer AI assistant".into()),
        },
        integration: ToolIntegrationConfig {
            config_path: ".amazonq/rules/".into(),
            config_type: ConfigType::Markdown,
            additional_paths: vec![],
        },
        capabilities: ToolCapabilities {
            supports_custom_instructions: true,
            supports_mcp: false,
            supports_rules_directory: true,
        },
        schema_keys: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::ToolIntegration;

    #[test]
    fn test_name() {
        let integration = amazonq_integration();
        assert_eq!(integration.name(), "amazonq");
    }

    #[test]
    fn test_config_locations() {
        let integration = amazonq_integration();
        let locations = integration.config_locations();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].path, ".amazonq/rules/");
        assert!(locations[0].is_directory);
    }
}
```

### Step 2-3: Add to lib.rs, dispatcher.rs, test, and commit

```bash
git commit -m "feat(repo-tools): add Amazon Q Developer integration

Adds support for .amazonq/rules/ directory."
```

---

## Task 8: Update Dispatcher with All New Tools

**Files:**
- Modify: `crates/repo-tools/src/dispatcher.rs`

### Step 1: Complete dispatcher.rs updates

Ensure all imports are added at the top:

```rust
use crate::aider::aider_integration;
use crate::amazonq::amazonq_integration;
use crate::cline::cline_integration;
use crate::copilot::copilot_integration;
use crate::jetbrains::jetbrains_integration;
use crate::roo::roo_integration;
use crate::zed::zed_integration;
```

Update `get_integration` match block:

```rust
pub fn get_integration(&self, tool_name: &str) -> Option<Box<dyn ToolIntegration>> {
    match tool_name {
        // Existing
        "vscode" => return Some(Box::new(VSCodeIntegration::new())),
        "cursor" => return Some(Box::new(cursor_integration())),
        "claude" => return Some(Box::new(claude_integration())),
        "windsurf" => return Some(Box::new(windsurf_integration())),
        "antigravity" => return Some(Box::new(antigravity_integration())),
        "gemini" => return Some(Box::new(gemini_integration())),
        // New integrations
        "copilot" => return Some(Box::new(copilot_integration())),
        "cline" => return Some(Box::new(cline_integration())),
        "roo" => return Some(Box::new(roo_integration())),
        "jetbrains" => return Some(Box::new(jetbrains_integration())),
        "zed" => return Some(Box::new(zed_integration())),
        "aider" => return Some(Box::new(aider_integration())),
        "amazonq" => return Some(Box::new(amazonq_integration())),
        _ => {}
    }
    // ... rest unchanged
}
```

Update `has_tool`:

```rust
pub fn has_tool(&self, tool_name: &str) -> bool {
    matches!(
        tool_name,
        "vscode" | "cursor" | "claude" | "windsurf" | "antigravity" | "gemini" |
        "copilot" | "cline" | "roo" | "jetbrains" | "zed" | "aider" | "amazonq"
    ) || self.schema_tools.contains_key(tool_name)
}
```

Update `list_available`:

```rust
pub fn list_available(&self) -> Vec<String> {
    let mut tools = vec![
        "aider".to_string(),
        "amazonq".to_string(),
        "antigravity".to_string(),
        "claude".to_string(),
        "cline".to_string(),
        "copilot".to_string(),
        "cursor".to_string(),
        "gemini".to_string(),
        "jetbrains".to_string(),
        "roo".to_string(),
        "vscode".to_string(),
        "windsurf".to_string(),
        "zed".to_string(),
    ];
    // ... rest unchanged
}
```

### Step 2: Run all tests

```bash
cargo test -p repo-tools
```

### Step 3: Commit

```bash
git add crates/repo-tools/src/dispatcher.rs
git commit -m "feat(repo-tools): register all new tool integrations in dispatcher

Adds copilot, cline, roo, jetbrains, zed, aider, amazonq to dispatcher."
```

---

## Task 9: Update lib.rs with All Exports

**Files:**
- Modify: `crates/repo-tools/src/lib.rs`

### Step 1: Add all module declarations and re-exports

```rust
// Add after existing module declarations
pub mod aider;
pub mod amazonq;
pub mod cline;
pub mod copilot;
pub mod jetbrains;
pub mod roo;
pub mod zed;

// Add to re-exports
pub use aider::aider_integration;
pub use amazonq::amazonq_integration;
pub use cline::cline_integration;
pub use copilot::copilot_integration;
pub use jetbrains::jetbrains_integration;
pub use roo::roo_integration;
pub use zed::zed_integration;
```

### Step 2: Run tests and commit

```bash
cargo test -p repo-tools
git add crates/repo-tools/src/lib.rs
git commit -m "feat(repo-tools): export all new tool integration modules"
```

---

## Task 10: Update GAP_TRACKING.md

**Files:**
- Modify: `docs/testing/GAP_TRACKING.md`

### Step 1: Mark GAP-009 (JetBrains) as closed and add new tools

Update the gap registry to reflect:
- GAP-009: JetBrains tool - CLOSED
- Add entries for new tools if not already tracked

### Step 2: Update dashboard

```
Production Readiness: 95%+
  - Tools: 13/13 implemented (all modern AI tools)
```

### Step 3: Commit

```bash
git add docs/testing/GAP_TRACKING.md
git commit -m "docs: update GAP_TRACKING for new tool integrations"
```

---

## Task 11: Run Full Verification

### Step 1: Run full test suite

```bash
cargo test --workspace
```

### Step 2: Run clippy

```bash
cargo clippy --workspace
```

### Step 3: Final commit if any fixes needed

---

## Summary

| Task | Tool | Config File(s) | Commits |
|------|------|----------------|---------|
| 1 | GitHub Copilot | `.github/copilot-instructions.md` | 1 |
| 2 | Cline | `.clinerules` | 1 |
| 3 | Roo Code | `.roo/rules/` | 1 |
| 4 | JetBrains AI | `.aiassistant/rules/` | 1 |
| 5 | Zed | `.rules` | 1 |
| 6 | Aider | `.aider.conf.yml` | 1 |
| 7 | Amazon Q | `.amazonq/rules/` | 1 |
| 8 | Dispatcher | - | 1 |
| 9 | lib.rs exports | - | 1 |
| 10 | GAP_TRACKING | - | 1 |
| 11 | Verification | - | 0-1 |

**Total: 10-11 commits**
**New tools added: 7**
**Total tools after: 13**
