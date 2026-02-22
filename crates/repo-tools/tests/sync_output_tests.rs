//! Sync output tests for tool integrations
//!
//! Category: component
//! Tests that tool sync operations produce expected file output.

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
    fn test_cursor_creates_cursorrules_with_content() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("cursor").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("python-style", "Use snake_case for variables.")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        assert!(
            content.contains("Use snake_case for variables."),
            "Rule content must be written to file"
        );
        assert!(
            content.contains("<!-- repo:block:python-style -->"),
            "Must have block start marker"
        );
        assert!(
            content.contains("<!-- /repo:block:python-style -->"),
            "Must have block end marker"
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
    fn test_cursor_via_factory_function_writes_content() {
        let temp = TempDir::new().unwrap();

        let integration = cursor_integration();
        assert_eq!(integration.name(), "cursor");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        assert!(
            content.contains("Test content"),
            "Factory function integration must produce correct content"
        );
        assert!(
            content.contains("<!-- repo:block:test -->"),
            "Factory function integration must use managed blocks"
        );
    }
}

// ============================================================================
// Claude Tests
// ============================================================================

mod claude_tests {
    use super::*;

    #[test]
    fn test_claude_creates_claude_md_with_content() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("claude").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("style", "Use clear, concise code.")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert!(
            content.contains("Use clear, concise code."),
            "Rule content must be written"
        );
        assert!(
            content.contains("<!-- repo:block:style -->"),
            "Must have block start marker"
        );
        assert!(
            content.contains("<!-- /repo:block:style -->"),
            "Must have block end marker"
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
    fn test_claude_via_factory_function_writes_content() {
        let temp = TempDir::new().unwrap();

        let integration = claude_integration();
        assert_eq!(integration.name(), "claude");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert!(
            content.contains("Test content"),
            "Factory function integration must produce correct content"
        );
        assert!(
            content.contains("<!-- repo:block:test -->"),
            "Factory function integration must use managed blocks"
        );
    }
}

// ============================================================================
// Copilot Tests
// ============================================================================

mod copilot_tests {
    use super::*;

    #[test]
    fn test_copilot_creates_instructions_with_content() {
        let temp = TempDir::new().unwrap();

        // Create .github directory (copilot expects it to exist or creates it)
        fs::create_dir_all(temp.path().join(".github")).unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("copilot").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("style", "Follow conventions.")];

        integration.sync(&context, &rules).unwrap();

        let content =
            fs::read_to_string(temp.path().join(".github/copilot-instructions.md")).unwrap();
        assert!(
            content.contains("Follow conventions."),
            "Rule content must be written to copilot instructions"
        );
        // Format: managed block structure
        assert!(
            content.contains("<!-- repo:block:style -->"),
            "Must have block start marker"
        );
        assert!(
            content.contains("<!-- /repo:block:style -->"),
            "Must have block end marker"
        );
        let opens = content.matches("<!-- repo:block:").count();
        let closes = content.matches("<!-- /repo:block:").count();
        assert_eq!(opens, closes, "Block markers must be balanced");
    }

    #[test]
    fn test_copilot_via_factory_function_writes_content() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".github")).unwrap();

        let integration = copilot_integration();
        assert_eq!(integration.name(), "copilot");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        let content =
            fs::read_to_string(temp.path().join(".github/copilot-instructions.md")).unwrap();
        assert!(
            content.contains("Test content"),
            "Factory function integration must produce correct content"
        );
        assert!(
            content.contains("<!-- repo:block:test -->"),
            "Factory function integration must use managed blocks"
        );
    }
}

// ============================================================================
// VS Code Tests
// ============================================================================

mod vscode_tests {
    use super::*;

    #[test]
    fn test_vscode_creates_valid_settings_json() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("vscode").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![];

        integration.sync(&context, &rules).unwrap();

        // Verify valid JSON object is created
        let content = fs::read_to_string(temp.path().join(".vscode/settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(
            settings.is_object(),
            "settings.json must be a valid JSON object"
        );
    }

    #[test]
    fn test_vscode_settings_json_has_python_path() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("vscode").unwrap();

        let python_path = NormalizedPath::new("/usr/bin/python3");
        let context = SyncContext::new(NormalizedPath::new(temp.path())).with_python(python_path);

        integration.sync(&context, &[]).unwrap();

        let content = fs::read_to_string(temp.path().join(".vscode/settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify the JSON object has the python path as a string value
        assert!(settings.is_object(), "settings.json must be a JSON object");
        assert!(
            settings["python.defaultInterpreterPath"].is_string(),
            "python path must be a string type"
        );
        assert_eq!(
            settings["python.defaultInterpreterPath"], "/usr/bin/python3",
            "python path must match the provided value"
        );
    }

    #[test]
    fn test_vscode_via_struct_produces_valid_json() {
        let temp = TempDir::new().unwrap();

        let integration = VSCodeIntegration::new();
        assert_eq!(integration.name(), "vscode");

        let python_path = NormalizedPath::new("/test/python");
        let context = SyncContext::new(NormalizedPath::new(temp.path())).with_python(python_path);

        integration.sync(&context, &[]).unwrap();

        let content = fs::read_to_string(temp.path().join(".vscode/settings.json")).unwrap();
        let settings: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(
            settings.is_object(),
            "settings.json must be a valid JSON object"
        );
        assert_eq!(settings["python.defaultInterpreterPath"], "/test/python");
    }
}

// ============================================================================
// Windsurf Tests
// ============================================================================

mod windsurf_tests {
    use super::*;

    #[test]
    fn test_windsurf_creates_windsurfrules_with_content() {
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let integration = dispatcher.get_integration("windsurf").unwrap();

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("style", "Be concise.")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();
        assert!(
            content.contains("Be concise."),
            "Rule content must be written"
        );
        assert!(
            content.contains("<!-- repo:block:style -->"),
            "Must have block start marker"
        );
        assert!(
            content.contains("<!-- /repo:block:style -->"),
            "Must have block end marker"
        );
    }

    #[test]
    fn test_windsurf_via_factory_function_writes_content() {
        let temp = TempDir::new().unwrap();

        let integration = windsurf_integration();
        assert_eq!(integration.name(), "windsurf");

        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test", "Test content")];

        integration.sync(&context, &rules).unwrap();

        let content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();
        assert!(
            content.contains("Test content"),
            "Factory function integration must produce correct content"
        );
        assert!(
            content.contains("<!-- repo:block:test -->"),
            "Factory function integration must use managed blocks"
        );
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
    fn test_dispatcher_sync_all_tools_writes_content() {
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

        // Verify files contain the shared rule content
        let cursor_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        assert!(
            cursor_content.contains("Shared rule content"),
            "Cursor must have shared rule"
        );

        let claude_content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert!(
            claude_content.contains("Shared rule content"),
            "Claude must have shared rule"
        );

        let windsurf_content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();
        assert!(
            windsurf_content.contains("Shared rule content"),
            "Windsurf must have shared rule"
        );

        // Format: all files must have balanced block markers
        for (name, content) in [("cursor", &cursor_content), ("claude", &claude_content), ("windsurf", &windsurf_content)] {
            let opens = content.matches("<!-- repo:block:").count();
            let closes = content.matches("<!-- /repo:block:").count();
            assert_eq!(opens, closes, "{name} must have balanced block markers");
            assert!(opens > 0, "{name} must have at least one block marker pair");
        }
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

        // Format: all tools must produce balanced managed blocks
        for (name, content) in [("cursor", &cursor_content), ("claude", &claude_content), ("windsurf", &windsurf_content)] {
            let opens = content.matches("<!-- repo:block:").count();
            let closes = content.matches("<!-- /repo:block:").count();
            assert_eq!(opens, closes, "{name} must have balanced block markers");
        }
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

    #[test]
    fn test_cross_tool_sync_does_not_interfere() {
        // Sync claude, then cursor, then claude again - verify they don't interfere
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let context = SyncContext::new(NormalizedPath::new(temp.path()));

        let rules_v1 = vec![create_rule("shared", "Version 1 content")];
        let rules_v2 = vec![create_rule("shared", "Version 2 content")];

        // Step 1: Sync claude with v1
        let claude = dispatcher.get_integration("claude").unwrap();
        claude.sync(&context, &rules_v1).unwrap();

        let claude_v1 = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert!(claude_v1.contains("Version 1 content"));

        // Step 2: Sync cursor (different file) with v1
        let cursor = dispatcher.get_integration("cursor").unwrap();
        cursor.sync(&context, &rules_v1).unwrap();

        // Claude's file should be UNTOUCHED by cursor sync
        let claude_after_cursor = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert_eq!(
            claude_v1, claude_after_cursor,
            "Syncing cursor must not modify CLAUDE.md"
        );

        // Step 3: Sync claude with v2
        claude.sync(&context, &rules_v2).unwrap();

        let claude_v2 = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert!(claude_v2.contains("Version 2 content"));
        assert!(!claude_v2.contains("Version 1 content"));

        // Cursor's file should be UNTOUCHED by claude re-sync
        let cursor_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        assert!(
            cursor_content.contains("Version 1 content"),
            "Cursor file must retain v1 content after claude re-sync"
        );
    }

    #[test]
    fn test_sync_all_idempotency() {
        // Run sync_all twice with same rules, verify identical output
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![
            create_rule("rule-a", "Content A"),
            create_rule("rule-b", "Content B"),
        ];
        let tools = vec![
            "cursor".to_string(),
            "claude".to_string(),
            "windsurf".to_string(),
        ];

        // First sync
        dispatcher.sync_all(&context, &tools, &rules).unwrap();
        let cursor_1 = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        let claude_1 = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        let windsurf_1 = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();

        // Second sync with identical rules
        dispatcher.sync_all(&context, &tools, &rules).unwrap();
        let cursor_2 = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        let claude_2 = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        let windsurf_2 = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();

        assert_eq!(
            cursor_1, cursor_2,
            "Cursor must be idempotent across sync_all"
        );
        assert_eq!(
            claude_1, claude_2,
            "Claude must be idempotent across sync_all"
        );
        assert_eq!(
            windsurf_1, windsurf_2,
            "Windsurf must be idempotent across sync_all"
        );
    }

    #[test]
    fn test_sync_all_skips_unknown_tools_without_affecting_others() {
        // Verify unknown tools are silently skipped, and other tools still sync correctly
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let context = SyncContext::new(NormalizedPath::new(temp.path()));
        let rules = vec![create_rule("test-rule", "Test content for partial sync")];

        let synced = dispatcher
            .sync_all(
                &context,
                &[
                    "cursor".to_string(),
                    "nonexistent-tool-xyz".to_string(),
                    "claude".to_string(),
                ],
                &rules,
            )
            .unwrap();

        // Unknown tools skipped, known tools synced
        assert_eq!(synced.len(), 2);
        assert!(synced.contains(&"cursor".to_string()));
        assert!(synced.contains(&"claude".to_string()));

        // Verify content was actually written for the known tools
        let cursor_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        assert!(cursor_content.contains("Test content for partial sync"));

        let claude_content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert!(claude_content.contains("Test content for partial sync"));

        // Format: synced tools must have managed block structure
        assert!(cursor_content.contains("<!-- repo:block:test-rule -->"), "Cursor must have block marker");
        assert!(claude_content.contains("<!-- repo:block:test-rule -->"), "Claude must have block marker");
    }

    #[test]
    fn test_tools_write_to_separate_files() {
        // Verify that cursor, claude, and windsurf each write to their own file
        // and don't share or corrupt each other's files
        let temp = TempDir::new().unwrap();

        let dispatcher = ToolDispatcher::new();
        let context = SyncContext::new(NormalizedPath::new(temp.path()));

        // Give each tool a DIFFERENT rule so we can distinguish
        let cursor = dispatcher.get_integration("cursor").unwrap();
        cursor
            .sync(
                &context,
                &[create_rule("tool-specific", "CURSOR-ONLY-CONTENT")],
            )
            .unwrap();

        let claude = dispatcher.get_integration("claude").unwrap();
        claude
            .sync(
                &context,
                &[create_rule("tool-specific", "CLAUDE-ONLY-CONTENT")],
            )
            .unwrap();

        let windsurf = dispatcher.get_integration("windsurf").unwrap();
        windsurf
            .sync(
                &context,
                &[create_rule("tool-specific", "WINDSURF-ONLY-CONTENT")],
            )
            .unwrap();

        // Verify each file has ONLY its own content
        let cursor_content = fs::read_to_string(temp.path().join(".cursorrules")).unwrap();
        assert!(cursor_content.contains("CURSOR-ONLY-CONTENT"));
        assert!(!cursor_content.contains("CLAUDE-ONLY-CONTENT"));
        assert!(!cursor_content.contains("WINDSURF-ONLY-CONTENT"));

        let claude_content = fs::read_to_string(temp.path().join("CLAUDE.md")).unwrap();
        assert!(claude_content.contains("CLAUDE-ONLY-CONTENT"));
        assert!(!claude_content.contains("CURSOR-ONLY-CONTENT"));
        assert!(!claude_content.contains("WINDSURF-ONLY-CONTENT"));

        let windsurf_content = fs::read_to_string(temp.path().join(".windsurfrules")).unwrap();
        assert!(windsurf_content.contains("WINDSURF-ONLY-CONTENT"));
        assert!(!windsurf_content.contains("CURSOR-ONLY-CONTENT"));
        assert!(!windsurf_content.contains("CLAUDE-ONLY-CONTENT"));
    }
}
