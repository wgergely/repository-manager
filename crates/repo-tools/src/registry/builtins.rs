//! Built-in tool registrations - SINGLE SOURCE OF TRUTH
//!
//! This module defines all built-in tool registrations in one place,
//! eliminating the 3-location duplication that previously existed in
//! the dispatcher (get_integration, has_tool, list_available).

use super::{ToolCategory, ToolRegistration};
use crate::{
    aider, amazonq, antigravity, claude, claude_desktop, cline, copilot, cursor, gemini, jetbrains,
    roo, vscode, windsurf, zed,
};

/// Number of built-in tools.
pub const BUILTIN_COUNT: usize = 14;

/// Returns all built-in tool registrations.
///
/// This is the SINGLE SOURCE OF TRUTH for built-in tools.
/// All tool listing, lookup, and dispatch should ultimately
/// derive from this function.
pub fn builtin_registrations() -> Vec<ToolRegistration> {
    vec![
        // IDEs (6 tools)
        ToolRegistration::new(
            "vscode",
            "VS Code",
            ToolCategory::Ide,
            vscode::vscode_definition(),
        ),
        ToolRegistration::new(
            "cursor",
            "Cursor",
            ToolCategory::Ide,
            cursor::cursor_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "zed",
            "Zed",
            ToolCategory::Ide,
            zed::zed_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "jetbrains",
            "JetBrains",
            ToolCategory::Ide,
            jetbrains::jetbrains_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "windsurf",
            "Windsurf",
            ToolCategory::Ide,
            windsurf::windsurf_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "antigravity",
            "Antigravity",
            ToolCategory::Ide,
            antigravity::antigravity_integration().definition().clone(),
        ),
        // CLI Agents (4 tools)
        ToolRegistration::new(
            "claude",
            "Claude Code",
            ToolCategory::CliAgent,
            claude::claude_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "claude_desktop",
            "Claude Desktop",
            ToolCategory::CliAgent,
            claude_desktop::claude_desktop_integration()
                .definition()
                .clone(),
        ),
        ToolRegistration::new(
            "aider",
            "Aider",
            ToolCategory::CliAgent,
            aider::aider_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "gemini",
            "Gemini CLI",
            ToolCategory::CliAgent,
            gemini::gemini_integration().definition().clone(),
        ),
        // Autonomous Agents (2 tools)
        ToolRegistration::new(
            "cline",
            "Cline",
            ToolCategory::Autonomous,
            cline::cline_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "roo",
            "Roo",
            ToolCategory::Autonomous,
            roo::roo_integration().definition().clone(),
        ),
        // Copilots (2 tools)
        ToolRegistration::new(
            "copilot",
            "GitHub Copilot",
            ToolCategory::Copilot,
            copilot::copilot_integration().definition().clone(),
        ),
        ToolRegistration::new(
            "amazonq",
            "Amazon Q",
            ToolCategory::Copilot,
            amazonq::amazonq_integration().definition().clone(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_builtin_count() {
        assert_eq!(builtin_registrations().len(), BUILTIN_COUNT);
    }

    #[test]
    fn test_no_duplicate_slugs() {
        let regs = builtin_registrations();
        let slugs: HashSet<_> = regs.iter().map(|r| &r.slug).collect();
        assert_eq!(slugs.len(), BUILTIN_COUNT, "Duplicate slugs found");
    }

    #[test]
    fn test_all_expected_tools_present() {
        let regs = builtin_registrations();
        let slugs: HashSet<_> = regs.iter().map(|r| r.slug.as_str()).collect();

        // IDEs
        assert!(slugs.contains("vscode"));
        assert!(slugs.contains("cursor"));
        assert!(slugs.contains("zed"));
        assert!(slugs.contains("jetbrains"));
        assert!(slugs.contains("windsurf"));
        assert!(slugs.contains("antigravity"));

        // CLI Agents
        assert!(slugs.contains("claude"));
        assert!(slugs.contains("claude_desktop"));
        assert!(slugs.contains("aider"));
        assert!(slugs.contains("gemini"));

        // Autonomous
        assert!(slugs.contains("cline"));
        assert!(slugs.contains("roo"));

        // Copilots
        assert!(slugs.contains("copilot"));
        assert!(slugs.contains("amazonq"));
    }

    #[test]
    fn test_category_counts() {
        let regs = builtin_registrations();

        let ide_count = regs
            .iter()
            .filter(|r| r.category == ToolCategory::Ide)
            .count();
        let cli_count = regs
            .iter()
            .filter(|r| r.category == ToolCategory::CliAgent)
            .count();
        let auto_count = regs
            .iter()
            .filter(|r| r.category == ToolCategory::Autonomous)
            .count();
        let copilot_count = regs
            .iter()
            .filter(|r| r.category == ToolCategory::Copilot)
            .count();

        assert_eq!(ide_count, 6);
        assert_eq!(cli_count, 4);
        assert_eq!(auto_count, 2);
        assert_eq!(copilot_count, 2);
    }

    #[test]
    fn test_all_have_definitions() {
        let regs = builtin_registrations();
        for reg in regs {
            // Verify definition slug matches registration slug
            assert_eq!(
                reg.definition.meta.slug, reg.slug,
                "Definition slug mismatch for {}",
                reg.slug
            );
        }
    }
}
