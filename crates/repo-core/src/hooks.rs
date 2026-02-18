//! Lifecycle hooks for repository events
//!
//! Provides pre/post hooks for branch creation, deletion, sync, and agent
//! completion events. Hooks are configured in config.toml as `[[hooks]]`
//! entries and executed as subprocesses.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Events that can trigger hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HookEvent {
    /// Before a branch/worktree is created
    PreBranchCreate,
    /// After a branch/worktree is created
    PostBranchCreate,
    /// Before a branch/worktree is deleted
    PreBranchDelete,
    /// After a branch/worktree is deleted
    PostBranchDelete,
    /// Before an agent task completes
    PreAgentComplete,
    /// After an agent task completes
    PostAgentComplete,
    /// Before sync runs
    PreSync,
    /// After sync runs
    PostSync,
}

impl fmt::Display for HookEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PreBranchCreate => write!(f, "pre-branch-create"),
            Self::PostBranchCreate => write!(f, "post-branch-create"),
            Self::PreBranchDelete => write!(f, "pre-branch-delete"),
            Self::PostBranchDelete => write!(f, "post-branch-delete"),
            Self::PreAgentComplete => write!(f, "pre-agent-complete"),
            Self::PostAgentComplete => write!(f, "post-agent-complete"),
            Self::PreSync => write!(f, "pre-sync"),
            Self::PostSync => write!(f, "post-sync"),
        }
    }
}

impl HookEvent {
    /// Parse a hook event from a string
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pre-branch-create" => Some(Self::PreBranchCreate),
            "post-branch-create" => Some(Self::PostBranchCreate),
            "pre-branch-delete" => Some(Self::PreBranchDelete),
            "post-branch-delete" => Some(Self::PostBranchDelete),
            "pre-agent-complete" => Some(Self::PreAgentComplete),
            "post-agent-complete" => Some(Self::PostAgentComplete),
            "pre-sync" => Some(Self::PreSync),
            "post-sync" => Some(Self::PostSync),
            _ => None,
        }
    }

    /// List all valid event names
    pub fn all_names() -> &'static [&'static str] {
        &[
            "pre-branch-create",
            "post-branch-create",
            "pre-branch-delete",
            "post-branch-delete",
            "pre-agent-complete",
            "post-agent-complete",
            "pre-sync",
            "post-sync",
        ]
    }
}

/// Configuration for a single hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// The event that triggers this hook
    pub event: HookEvent,
    /// The command to execute
    pub command: String,
    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory override (defaults to repository root)
    pub working_dir: Option<PathBuf>,
}

/// Context variables available to hooks during execution
#[derive(Debug, Clone, Default)]
pub struct HookContext {
    /// Variables available for substitution in hook args
    pub vars: HashMap<String, String>,
}

impl HookContext {
    /// Create context for a branch event
    pub fn for_branch(branch_name: &str, worktree_path: Option<&Path>) -> Self {
        let mut vars = HashMap::new();
        vars.insert("BRANCH_NAME".to_string(), branch_name.to_string());
        if let Some(path) = worktree_path {
            vars.insert("WORKTREE_PATH".to_string(), path.display().to_string());
        }
        Self { vars }
    }

    /// Create context for an agent event
    pub fn for_agent(agent_name: &str, task_id: &str) -> Self {
        let mut vars = HashMap::new();
        vars.insert("AGENT_NAME".to_string(), agent_name.to_string());
        vars.insert("TASK_ID".to_string(), task_id.to_string());
        Self { vars }
    }
}

/// Result of running a single hook
#[derive(Debug)]
pub struct HookResult {
    /// The hook that was run
    pub event: HookEvent,
    /// The command that was run
    pub command: String,
    /// Whether the hook succeeded
    pub success: bool,
    /// Captured stdout
    pub stdout: String,
    /// Captured stderr
    pub stderr: String,
    /// Exit code
    pub exit_code: Option<i32>,
}

/// Run all hooks matching the given event
///
/// Hooks are executed in order. If a hook fails (non-zero exit), execution
/// stops and an error is returned (fail-fast behavior).
pub fn run_hooks(
    hooks: &[HookConfig],
    event: HookEvent,
    context: &HookContext,
    default_dir: &Path,
) -> Result<Vec<HookResult>> {
    let matching: Vec<&HookConfig> = hooks.iter().filter(|h| h.event == event).collect();

    let mut results = Vec::new();

    for hook in matching {
        let result = execute_hook(hook, context, default_dir)?;
        let failed = !result.success;
        results.push(result);

        if failed {
            return Err(Error::HookFailed {
                event: event.to_string(),
                command: hook.command.clone(),
                message: "Hook exited with non-zero status".to_string(),
            });
        }
    }

    Ok(results)
}

/// Execute a single hook as a subprocess
fn execute_hook(
    hook: &HookConfig,
    context: &HookContext,
    default_dir: &Path,
) -> Result<HookResult> {
    let work_dir = hook.working_dir.as_deref().unwrap_or(default_dir);

    // Substitute context variables in args
    let args: Vec<String> = hook
        .args
        .iter()
        .map(|arg| substitute_vars(arg, &context.vars))
        .collect();

    let output = Command::new(&hook.command)
        .args(&args)
        .current_dir(work_dir)
        .envs(&context.vars)
        .output()
        .map_err(Error::Io)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(HookResult {
        event: hook.event,
        command: hook.command.clone(),
        success: output.status.success(),
        stdout,
        stderr,
        exit_code: output.status.code(),
    })
}

/// Substitute ${VAR_NAME} patterns in a string with context variables
fn substitute_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        let pattern = format!("${{{}}}", key);
        result = result.replace(&pattern, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_event_display() {
        assert_eq!(HookEvent::PreBranchCreate.to_string(), "pre-branch-create");
        assert_eq!(HookEvent::PostSync.to_string(), "post-sync");
    }

    #[test]
    fn test_hook_event_parse() {
        assert_eq!(
            HookEvent::parse("pre-branch-create"),
            Some(HookEvent::PreBranchCreate)
        );
        assert_eq!(
            HookEvent::parse("post-agent-complete"),
            Some(HookEvent::PostAgentComplete)
        );
        assert_eq!(HookEvent::parse("invalid"), None);
    }

    #[test]
    fn test_hook_event_roundtrip() {
        for name in HookEvent::all_names() {
            let event = HookEvent::parse(name).unwrap();
            assert_eq!(event.to_string(), *name);
        }
    }

    #[test]
    fn test_hook_config_serialize() {
        let hook = HookConfig {
            event: HookEvent::PostBranchCreate,
            command: "npm".to_string(),
            args: vec!["install".to_string()],
            working_dir: None,
        };

        let json = serde_json::to_string(&hook).unwrap();
        let deserialized: HookConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event, HookEvent::PostBranchCreate);
        assert_eq!(deserialized.command, "npm");
        assert_eq!(deserialized.args, vec!["install"]);
    }

    #[test]
    fn test_hook_context_for_branch() {
        let ctx = HookContext::for_branch("feature-x", Some(Path::new("/repo/feature-x")));
        assert_eq!(ctx.vars["BRANCH_NAME"], "feature-x");
        assert_eq!(ctx.vars["WORKTREE_PATH"], "/repo/feature-x");
    }

    #[test]
    fn test_hook_context_for_agent() {
        let ctx = HookContext::for_agent("coder", "task-123");
        assert_eq!(ctx.vars["AGENT_NAME"], "coder");
        assert_eq!(ctx.vars["TASK_ID"], "task-123");
    }

    #[test]
    fn test_substitute_vars() {
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "feature-x".to_string());
        vars.insert("PATH".to_string(), "/repo".to_string());

        assert_eq!(
            substitute_vars("branch: ${NAME} at ${PATH}", &vars),
            "branch: feature-x at /repo"
        );
        assert_eq!(substitute_vars("no vars here", &vars), "no vars here");
    }

    #[test]
    fn test_run_hooks_no_matching() {
        let hooks = vec![HookConfig {
            event: HookEvent::PreSync,
            command: "echo".to_string(),
            args: vec!["sync".to_string()],
            working_dir: None,
        }];

        let ctx = HookContext::default();
        let temp = tempfile::TempDir::new().unwrap();
        let results = run_hooks(&hooks, HookEvent::PostSync, &ctx, temp.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_run_hooks_echo() {
        let hooks = vec![HookConfig {
            event: HookEvent::PostBranchCreate,
            command: if cfg!(windows) {
                "cmd".to_string()
            } else {
                "echo".to_string()
            },
            args: if cfg!(windows) {
                vec!["/C".to_string(), "echo".to_string(), "hello".to_string()]
            } else {
                vec!["hello".to_string()]
            },
            working_dir: None,
        }];

        let ctx = HookContext::default();
        let temp = tempfile::TempDir::new().unwrap();
        let results = run_hooks(&hooks, HookEvent::PostBranchCreate, &ctx, temp.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(results[0].stdout.trim().contains("hello"));
    }

    #[test]
    fn test_run_hooks_failure() {
        let hooks = vec![HookConfig {
            event: HookEvent::PreBranchCreate,
            command: if cfg!(windows) {
                "cmd".to_string()
            } else {
                "false".to_string()
            },
            args: if cfg!(windows) {
                vec!["/C".to_string(), "exit".to_string(), "1".to_string()]
            } else {
                vec![]
            },
            working_dir: None,
        }];

        let ctx = HookContext::default();
        let temp = tempfile::TempDir::new().unwrap();
        let result = run_hooks(&hooks, HookEvent::PreBranchCreate, &ctx, temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_hook_event_serde_roundtrip() {
        let event = HookEvent::PostBranchCreate;
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(json, "\"post-branch-create\"");
        let parsed: HookEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, event);
    }

    #[test]
    fn test_hook_config_toml_roundtrip() {
        let toml_str = r#"
event = "post-branch-create"
command = "npm"
args = ["install"]
"#;
        let hook: HookConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(hook.event, HookEvent::PostBranchCreate);
        assert_eq!(hook.command, "npm");
        assert_eq!(hook.args, vec!["install"]);
    }
}
