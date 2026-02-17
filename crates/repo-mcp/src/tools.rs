//! MCP Tool implementations
//!
//! This module contains the tool handlers for the MCP server.
//! Tools are the primary way agents interact with Repository Manager.
//!
//! # Tool Categories
//!
//! ## Repository Lifecycle
//! - `repo_init` - Initialize a new repository configuration
//! - `repo_check` - Check configuration validity and consistency
//! - `repo_fix` - Repair inconsistencies
//! - `repo_sync` - Regenerate tool configurations
//!
//! ## Branch Management
//! - `branch_create` - Create a new branch (with worktree in worktrees mode)
//! - `branch_delete` - Remove a branch and its worktree
//! - `branch_list` - List active branches
//!
//! ## Git Primitives (Not Yet Implemented)
//! - `git_push` - Push current branch (returns NotImplemented)
//! - `git_pull` - Pull updates (returns NotImplemented)
//! - `git_merge` - Merge target branch (returns NotImplemented)
//!
//! ## Configuration Management
//! - `tool_add` - Enable a tool
//! - `tool_remove` - Disable a tool
//! - `rule_add` - Add a custom rule
//! - `rule_remove` - Delete a rule
//!
//! ## Preset Management
//! - `preset_list` - List configured presets
//! - `preset_add` - Add a preset to configuration
//! - `preset_remove` - Remove a preset from configuration
//!
//! ## Superpowers Plugin
//! - `superpowers_install` - Install the superpowers Claude Code plugin
//! - `superpowers_status` - Check superpowers installation status
//! - `superpowers_uninstall` - Uninstall the superpowers plugin

use serde::{Deserialize, Serialize};

/// Tool definition for MCP protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Result from a tool invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content types for tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
}

impl ToolResult {
    /// Create a successful text result
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text {
                text: content.into(),
            }],
            is_error: None,
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text {
                text: message.into(),
            }],
            is_error: Some(true),
        }
    }
}

/// Get all available tool definitions
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        // Repository Lifecycle
        ToolDefinition {
            name: "repo_init".to_string(),
            description: "Initialize a new repository configuration".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "mode": {
                        "type": "string",
                        "enum": ["standard", "worktrees"],
                        "description": "Repository mode"
                    },
                    "tools": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tools to enable"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "repo_check".to_string(),
            description: "Check configuration validity and consistency".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "repo_sync".to_string(),
            description: "Regenerate tool configurations from rules".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dry_run": {
                        "type": "boolean",
                        "description": "Preview changes without applying"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "repo_fix".to_string(),
            description: "Repair configuration inconsistencies".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dry_run": {
                        "type": "boolean",
                        "description": "Preview fixes without applying"
                    }
                }
            }),
        },
        // Branch Management
        ToolDefinition {
            name: "branch_create".to_string(),
            description: "Create a new branch (with worktree in worktrees mode)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Branch name"
                    },
                    "base": {
                        "type": "string",
                        "description": "Base branch (defaults to main)"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "branch_delete".to_string(),
            description: "Remove a branch and its worktree".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Branch name to delete"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "branch_list".to_string(),
            description: "List active branches".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Git Primitives (not yet implemented - will return NotImplemented error)
        ToolDefinition {
            name: "git_push".to_string(),
            description: "[Not implemented] Push current branch to remote".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "remote": {
                        "type": "string",
                        "description": "Remote name (defaults to origin)"
                    },
                    "branch": {
                        "type": "string",
                        "description": "Branch to push (defaults to current)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "git_pull".to_string(),
            description: "[Not implemented] Pull updates from remote".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "remote": {
                        "type": "string",
                        "description": "Remote name (defaults to origin)"
                    },
                    "branch": {
                        "type": "string",
                        "description": "Branch to pull (defaults to current)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "git_merge".to_string(),
            description: "[Not implemented] Merge target branch into current branch".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "Branch to merge from"
                    }
                },
                "required": ["source"]
            }),
        },
        // Configuration Management
        ToolDefinition {
            name: "tool_add".to_string(),
            description: "Enable a tool for this repository".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Tool name (e.g., vscode, cursor, claude)"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "tool_remove".to_string(),
            description: "Disable a tool for this repository".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Tool name to remove"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "rule_add".to_string(),
            description: "Add a custom rule to the repository".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Rule identifier"
                    },
                    "content": {
                        "type": "string",
                        "description": "Rule content/instructions"
                    }
                },
                "required": ["id", "content"]
            }),
        },
        ToolDefinition {
            name: "rule_remove".to_string(),
            description: "Delete a rule from the repository".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Rule ID to remove"
                    }
                },
                "required": ["id"]
            }),
        },
        // Preset Management
        ToolDefinition {
            name: "preset_list".to_string(),
            description: "List configured presets and available preset types".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "preset_add".to_string(),
            description: "Add a preset to the repository configuration".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Preset name (e.g., env:python, env:node, env:rust)"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "preset_remove".to_string(),
            description: "Remove a preset from the repository configuration".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Preset name to remove"
                    }
                },
                "required": ["name"]
            }),
        },
        // Agent Orchestration
        ToolDefinition {
            name: "agent_check".to_string(),
            description: "Check agent subsystem prerequisites (Python 3.13+, vaultspec)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "agent_list".to_string(),
            description: "List available agent definitions from vaultspec".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "agent_spawn".to_string(),
            description: "Spawn an agent to work on a task in a worktree".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Agent name or type (e.g., 'researcher', 'coder')"
                    },
                    "goal": {
                        "type": "string",
                        "description": "Goal or task description for the agent"
                    },
                    "worktree": {
                        "type": "string",
                        "description": "Worktree to run the agent in"
                    }
                },
                "required": ["name"]
            }),
        },
        ToolDefinition {
            name: "agent_status".to_string(),
            description: "Check status of running agents and tasks".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "Specific task ID to check (shows all if omitted)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "agent_stop".to_string(),
            description: "Stop a running agent by task ID".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "Task ID or agent ID to stop"
                    }
                },
                "required": ["task_id"]
            }),
        },
        // Superpowers Plugin
        ToolDefinition {
            name: "superpowers_install".to_string(),
            description: "Install the superpowers Claude Code plugin".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "version": {
                        "type": "string",
                        "description": "Version tag to install (defaults to latest)"
                    }
                }
            }),
        },
        ToolDefinition {
            name: "superpowers_status".to_string(),
            description: "Check superpowers plugin installation status".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        ToolDefinition {
            name: "superpowers_uninstall".to_string(),
            description: "Uninstall the superpowers Claude Code plugin".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "version": {
                        "type": "string",
                        "description": "Version tag to uninstall (defaults to latest)"
                    }
                }
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tool_definitions() {
        let tools = get_tool_definitions();
        assert!(!tools.is_empty());

        // Verify expected tools exist
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"repo_init"));
        assert!(names.contains(&"repo_check"));
        assert!(names.contains(&"repo_sync"));
        assert!(names.contains(&"repo_fix"));
        assert!(names.contains(&"git_push"));
        assert!(names.contains(&"git_pull"));
        assert!(names.contains(&"git_merge"));
        assert!(names.contains(&"branch_create"));
        assert!(names.contains(&"branch_delete"));
        assert!(names.contains(&"branch_list"));
        assert!(names.contains(&"tool_add"));
        assert!(names.contains(&"tool_remove"));
        assert!(names.contains(&"rule_add"));
        assert!(names.contains(&"rule_remove"));
        assert!(names.contains(&"preset_list"));
        assert!(names.contains(&"preset_add"));
        assert!(names.contains(&"preset_remove"));
        assert!(names.contains(&"agent_check"));
        assert!(names.contains(&"agent_list"));
        assert!(names.contains(&"agent_spawn"));
        assert!(names.contains(&"agent_status"));
        assert!(names.contains(&"agent_stop"));
        assert!(names.contains(&"superpowers_install"));
        assert!(names.contains(&"superpowers_status"));
        assert!(names.contains(&"superpowers_uninstall"));
    }

    #[test]
    fn test_tool_definitions_count() {
        let tools = get_tool_definitions();
        // 4 repo lifecycle + 3 branch + 3 git + 4 config + 3 preset + 5 agent + 3 superpowers = 25 tools
        assert_eq!(tools.len(), 25);
    }

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("Success");
        assert!(result.is_error.is_none());
        assert_eq!(result.content.len(), 1);

        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Success"),
        }
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Failed");
        assert_eq!(result.is_error, Some(true));
        assert_eq!(result.content.len(), 1);

        match &result.content[0] {
            ToolContent::Text { text } => assert_eq!(text, "Failed"),
        }
    }

    #[test]
    fn test_tool_definitions_serialize() {
        let tools = get_tool_definitions();
        let json = serde_json::to_string(&tools).unwrap();
        assert!(json.contains("repo_init"));
        assert!(json.contains("git_push"));
        assert!(json.contains("branch_create"));
        assert!(json.contains("tool_add"));
        assert!(json.contains("rule_add"));
    }

    #[test]
    fn test_tool_result_serialize() {
        let result = ToolResult::text("Hello, world!");
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("text"));
        // is_error should be skipped when None
        assert!(!json.contains("is_error"));

        let error_result = ToolResult::error("Something went wrong");
        let error_json = serde_json::to_string(&error_result).unwrap();
        assert!(error_json.contains("is_error"));
        assert!(error_json.contains("true"));
    }

    #[test]
    fn test_tool_definition_deserialize() {
        let json = r#"{
            "name": "test_tool",
            "description": "A test tool",
            "input_schema": {"type": "object"}
        }"#;
        let tool: ToolDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
    }

    #[test]
    fn test_tool_result_deserialize() {
        let json = r#"{
            "content": [{"type": "text", "text": "Result text"}],
            "is_error": false
        }"#;
        let result: ToolResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_each_tool_has_valid_schema() {
        let tools = get_tool_definitions();
        for tool in &tools {
            // Each tool should have an object schema
            assert!(
                tool.input_schema.is_object(),
                "Tool {} should have object schema",
                tool.name
            );
            let schema = tool.input_schema.as_object().unwrap();
            assert_eq!(
                schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Tool {} schema type should be 'object'",
                tool.name
            );
        }
    }

    #[test]
    fn test_tools_with_required_fields() {
        let tools = get_tool_definitions();

        // Check repo_init requires "name"
        let repo_init = tools.iter().find(|t| t.name == "repo_init").unwrap();
        let required = repo_init
            .input_schema
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("name")));

        // Check branch_create requires "name"
        let branch_create = tools.iter().find(|t| t.name == "branch_create").unwrap();
        let required = branch_create
            .input_schema
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("name")));

        // Check git_merge requires "source"
        let git_merge = tools.iter().find(|t| t.name == "git_merge").unwrap();
        let required = git_merge
            .input_schema
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("source")));

        // Check rule_add requires "id" and "content"
        let rule_add = tools.iter().find(|t| t.name == "rule_add").unwrap();
        let required = rule_add
            .input_schema
            .get("required")
            .unwrap()
            .as_array()
            .unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("id")));
        assert!(required.iter().any(|v| v.as_str() == Some("content")));
    }
}
