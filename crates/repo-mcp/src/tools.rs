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
//! - `branch_checkout` - Checkout or get worktree path
//!
//! ## Git Primitives
//! - `git_push` - Push current branch
//! - `git_pull` - Pull updates
//! - `git_merge` - Merge target branch
//!
//! ## Configuration Management
//! - `tool_add` - Enable a tool
//! - `tool_remove` - Disable a tool
//! - `preset_add` - Apply a preset
//! - `preset_remove` - Remove a preset
//! - `rule_add` - Add a custom rule
//! - `rule_modify` - Modify an existing rule
//! - `rule_remove` - Delete a rule

// TODO: Implement tool handlers
