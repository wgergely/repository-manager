//! Integration tests for tool sync operations
//!
//! Tests that sync produces correct output files for each tool.

use repo_fs::NormalizedPath;
use repo_tools::{
    Rule, SyncContext, ToolDispatcher, ToolIntegration, VSCodeIntegration, claude_integration,
    copilot_integration, cursor_integration, windsurf_integration,
};
use std::fs;
use tempfile::TempDir;

/// Create a rule
fn create_rule(id: &str, content: &str) -> Rule {
    Rule {
        id: id.to_string(),
        content: content.to_string(),
    }
}

// ============================================================================
// Cursor Tests
// ============================================================================

mod cursor_tests {
    use super::*;

    #[test]
    fn test_cursor_creates_cursorrules_file() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("cursor").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("python-style", "Use snake_case for variables.")];

        integration.sync(&context, &rules).unwrap();

        // .cursorrules file should exist
        let rules_file = temp.path().join(".cursorrules");
        assert!(rules_file.exists(), ".cursorrules should be created");
    }

    #[test]
    fn test_cursor_uses_managed_blocks() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("cursor").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule(
            "typescript-style",
            "Use strict TypeScript mode.",
        )];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        assert!(
            content.contains("<!-- repo:block:typescript-style -->"),
            "Should contain block marker"
        );
        assert!(
            content.contains("TypeScript"),
            "Should contain rule content"
        );
    }

    #[test]
    fn test_cursor_multiple_rules() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("cursor").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![
            create_rule("rule-a", "Rule A content"),
            create_rule("rule-b", "Rule B content"),
            create_rule("rule-c", "Rule C content"),
        ];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();

        // All rules should be present
        assert!(content.contains("Rule A content"));
        assert!(content.contains("Rule B content"));
        assert!(content.contains("Rule C content"));

        // Each rule has its own block
        assert_eq!(
            content.matches("<!-- repo:block:rule-a -->").count(),
            1,
            "Should have exactly one rule-a block"
        );
        assert_eq!(
            content.matches("<!-- repo:block:rule-b -->").count(),
            1,
            "Should have exactly one rule-b block"
        );
        assert_eq!(
            content.matches("<!-- repo:block:rule-c -->").count(),
            1,
            "Should have exactly one rule-c block"
        );
    }

    #[test]
    fn test_cursor_via_factory_function() {
        let temp = TempDir::new().unwrap();

        let integration = cursor_integration();
        assert_eq!(integration.name(), "cursor");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        assert!(temp.path().join(".cursorrules").exists());
    }
}

// ============================================================================
// Claude Tests
// ============================================================================

mod claude_tests {
    use super::*;

    #[test]
    fn test_claude_creates_claude_md() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("claude").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("style", "Use clear, concise code.")];

        integration.sync(&context, &rules).unwrap();

        let claude_md = temp.path().join("CLAUDE.md");
        assert!(claude_md.exists(), "CLAUDE.md should be created");
    }

    #[test]
    fn test_claude_md_has_managed_blocks() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("claude").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("coding-style", "Follow project conventions.")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();

        // Should have block markers
        assert!(
            content.contains("<!-- repo:block:coding-style -->"),
            "Should contain block start marker"
        );
        assert!(
            content.contains("<!-- /repo:block:coding-style -->"),
            "Should contain block end marker"
        );
        assert!(
            content.contains("Follow project conventions"),
            "Should contain rule content"
        );
    }

    #[test]
    fn test_claude_preserves_user_content() {
        let temp = TempDir::new().unwrap();

        // Create existing CLAUDE.md with user content
        fs::write(
            temp.path().join("CLAUDE.md"),
            r#"# My Project

This is my custom documentation that should be preserved.
"#,
        )
        .unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("claude").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test-rule", "Test instruction.")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();

        // User content should be preserved
        assert!(content.contains("My Project"), "Title should be preserved");
        // Rule should be added
        assert!(content.contains("Test instruction"), "Rule should be added");
    }

    #[test]
    fn test_claude_via_factory_function() {
        let temp = TempDir::new().unwrap();

        let integration = claude_integration();
        assert_eq!(integration.name(), "claude");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        assert!(temp.path().join("CLAUDE.md").exists());
    }
}

// ============================================================================
// Copilot Tests
// ============================================================================

mod copilot_tests {
    use super::*;

    #[test]
    fn test_copilot_creates_github_directory() {
        let temp = TempDir::new().unwrap();

        // Create .github directory (copilot expects it to exist or creates it)
        fs::create_dir_all(temp.path().join(".github")).unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("copilot").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("style", "Follow conventions.")];

        integration.sync(&context, &rules).unwrap();

        assert!(
            temp.path().join(".github").exists(),
            ".github directory should exist"
        );
    }

    #[test]
    fn test_copilot_creates_instructions_file() {
        let temp = TempDir::new().unwrap();

        // Create .github directory
        fs::create_dir_all(temp.path().join(".github")).unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("copilot").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("coding", "Use TypeScript strict mode.")];

        integration.sync(&context, &rules).unwrap();

        let instructions = temp.path().join(".github").join("copilot-instructions.md");
        assert!(
            instructions.exists(),
            "copilot-instructions.md should exist"
        );

        let content = fs::read_to_string(&instructions).unwrap();
        assert!(
            content.contains("TypeScript"),
            "Should contain rule content"
        );
    }

    #[test]
    fn test_copilot_via_factory_function() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".github")).unwrap();

        let integration = copilot_integration();
        assert_eq!(integration.name(), "copilot");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        assert!(temp.path().join(".github/copilot-instructions.md").exists());
    }
}

// ============================================================================
// VS Code Tests
// ============================================================================

mod vscode_tests {
    use super::*;

    #[test]
    fn test_vscode_creates_vscode_directory() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("vscode").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![];

        integration.sync(&context, &rules).unwrap();

        assert!(
            temp.path().join(".vscode").exists(),
            ".vscode directory should exist"
        );
    }

    #[test]
    fn test_vscode_creates_settings_json() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("vscode").unwrap();

        let python_path = NormalizedPath::new("/usr/bin/python3");
        let context = SyncContext::new(NormalizedPath::new(temp.path())).with_python(python_path);

        integration.sync(&context, &[]).unwrap();

        let settings_path = temp.path().join(".vscode/settings.json");
        assert!(settings_path.exists(), "settings.json should be created");

        let content = fs::read_to_string(&settings_path).unwrap();
        assert!(content.contains("python.defaultInterpreterPath"));
    }

    #[test]
    fn test_vscode_via_struct() {
        let temp = TempDir::new().unwrap();

        let integration = VSCodeIntegration::new();
        assert_eq!(integration.name(), "vscode");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));

        integration.sync(&context, &[]).unwrap();

        assert!(temp.path().join(".vscode/settings.json").exists());
    }
}

// ============================================================================
// Windsurf Tests
// ============================================================================

mod windsurf_tests {
    use super::*;

    #[test]
    fn test_windsurf_creates_windsurfrules() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("windsurf").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("style", "Be concise.")];

        integration.sync(&context, &rules).unwrap();

        let windsurf_rules = temp.path().join(".windsurfrules");
        assert!(windsurf_rules.exists(), ".windsurfrules should exist");
    }

    #[test]
    fn test_windsurf_rule_content() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("windsurf").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("python-practices", "Always use type hints.")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();
        assert!(
            content.contains("type hints"),
            "Should contain rule content"
        );
    }

    #[test]
    fn test_windsurf_uses_managed_blocks() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("windsurf").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test-block", "Block content here.")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();

        // Should use managed blocks
        assert!(
            content.contains("<!-- repo:block:test-block -->"),
            "Should have block start marker"
        );
        assert!(
            content.contains("<!-- /repo:block:test-block -->"),
            "Should have block end marker"
        );
    }

    #[test]
    fn test_windsurf_via_factory_function() {
        let temp = TempDir::new().unwrap();

        let integration = windsurf_integration();
        assert_eq!(integration.name(), "windsurf");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        assert!(temp.path().join(".windsurfrules").exists());
    }
}

// ============================================================================
// Dispatcher Tests
// ============================================================================

mod dispatcher_tests {
    use super::*;

    #[test]
    fn test_dispatcher_has_all_builtin_tools() {
        let dispatcher = ToolDispatcher::new();

        // Core tools
        assert!(
            dispatcher.get_integration("vscode").is_some(),
            "vscode should be available"
        );
        assert!(
            dispatcher.get_integration("cursor").is_some(),
            "cursor should be available"
        );
        assert!(
            dispatcher.get_integration("claude").is_some(),
            "claude should be available"
        );
        assert!(
            dispatcher.get_integration("windsurf").is_some(),
            "windsurf should be available"
        );
        assert!(
            dispatcher.get_integration("copilot").is_some(),
            "copilot should be available"
        );

        // Additional tools
        assert!(
            dispatcher.get_integration("cline").is_some(),
            "cline should be available"
        );
        assert!(
            dispatcher.get_integration("gemini").is_some(),
            "gemini should be available"
        );
        assert!(
            dispatcher.get_integration("antigravity").is_some(),
            "antigravity should be available"
        );
        assert!(
            dispatcher.get_integration("jetbrains").is_some(),
            "jetbrains should be available"
        );
        assert!(
            dispatcher.get_integration("zed").is_some(),
            "zed should be available"
        );
        assert!(
            dispatcher.get_integration("aider").is_some(),
            "aider should be available"
        );
        assert!(
            dispatcher.get_integration("amazonq").is_some(),
            "amazonq should be available"
        );
        assert!(
            dispatcher.get_integration("roo").is_some(),
            "roo should be available"
        );
    }

    #[test]
    fn test_dispatcher_unknown_tool_returns_none() {
        let dispatcher = ToolDispatcher::new();

        assert!(
            dispatcher.get_integration("unknown-tool").is_none(),
            "Unknown tool should return None"
        );
    }

    #[test]
    fn test_dispatcher_sync_all_tools() {
        let temp = TempDir::new().unwrap();

        // Create .github for copilot
        fs::create_dir_all(temp.path().join(".github")).unwrap();

        let dispatcher = ToolDispatcher::new();
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("shared-rule", "Shared rule content")];

        let synced = dispatcher
            .sync_all(
                &context,
                &[
                    "cursor".to_string(),
                    "claude".to_string(),
                    "windsurf".to_string(),
                ],
                &rules,
            )
            .unwrap();

        assert_eq!(synced.len(), 3);
        assert!(synced.contains(&"cursor".to_string()));
        assert!(synced.contains(&"claude".to_string()));
        assert!(synced.contains(&"windsurf".to_string()));

        // Verify files were created
        assert!(temp.path().join(".cursorrules").exists());
        assert!(temp.path().join("CLAUDE.md").exists());
        assert!(temp.path().join(".windsurfrules").exists());
    }

    #[test]
    fn test_dispatcher_list_available() {
        let dispatcher = ToolDispatcher::new();
        let available = dispatcher.list_available();

        assert!(available.contains(&"vscode".to_string()));
        assert!(available.contains(&"cursor".to_string()));
        assert!(available.contains(&"claude".to_string()));
        assert!(available.contains(&"windsurf".to_string()));
        assert!(available.contains(&"copilot".to_string()));
    }
}

// ============================================================================
// Cross-Tool Tests
// ============================================================================

mod cross_tool_tests {
    use super::*;

    #[test]
    fn test_same_rules_sync_to_multiple_tools() {
        let temp = TempDir::new().unwrap();

        // Create required directories
        fs::create_dir_all(temp.path().join(".github")).unwrap();

        let dispatcher = ToolDispatcher::new();
        let context = SyncContext::new(NormalizedPath::new(temp.path()));

        // Same rules for all tools
        let rules = vec![
            create_rule("style-guide", "Use consistent formatting."),
            create_rule("testing", "Write tests for all features."),
        ];

        // Sync to cursor
        let cursor = dispatcher.get_integration("cursor").unwrap();
        cursor.sync(&context, &rules).unwrap();

        // Sync to claude
        let claude = dispatcher.get_integration("claude").unwrap();
        claude.sync(&context, &rules).unwrap();

        // Sync to windsurf
        let windsurf = dispatcher.get_integration("windsurf").unwrap();
        windsurf.sync(&context, &rules).unwrap();

        // Verify all have the content
        let cursor_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        let claude_content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        let windsurf_content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();

        // All should contain the rules
        assert!(cursor_content.contains("Use consistent formatting"));
        assert!(cursor_content.contains("Write tests"));

        assert!(claude_content.contains("Use consistent formatting"));
        assert!(claude_content.contains("Write tests"));

        assert!(windsurf_content.contains("Use consistent formatting"));
        assert!(windsurf_content.contains("Write tests"));
    }

    #[test]
    fn test_tool_idempotency() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let cursor = dispatcher.get_integration("cursor").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("idempotent-rule", "This should be stable.")];

        // Sync multiple times
        cursor.sync(&context, &rules).unwrap();
        let first_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();

        cursor.sync(&context, &rules).unwrap();
        let second_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();

        cursor.sync(&context, &rules).unwrap();
        let third_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();

        // All syncs should produce identical output
        assert_eq!(
            first_content, second_content,
            "Second sync should match first"
        );
        assert_eq!(
            second_content, third_content,
            "Third sync should match second"
        );

        // Should only have one block, not duplicates
        assert_eq!(
            first_content
                .matches("<!-- repo:block:idempotent-rule -->")
                .count(),
            1
        );
    }
}
