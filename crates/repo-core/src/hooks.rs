//! Lifecycle hooks for repository events
//!
//! Provides pre/post hooks for branch creation, deletion, and sync
//! events. Hooks are configured in config.toml as `[[hooks]]`
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

    /// Create context for a sync event
    pub fn for_sync() -> Self {
        let mut vars = HashMap::new();
        vars.insert("HOOK_EVENT_TYPE".to_string(), "sync".to_string());
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

        if failed {
            // Include stderr in the error message for actionable diagnostics
            let stderr_snippet = result.stderr.trim();
            let message = if stderr_snippet.is_empty() {
                format!(
                    "Hook exited with non-zero status (exit code: {:?})",
                    result.exit_code
                )
            } else {
                format!(
                    "Hook exited with non-zero status (exit code: {:?}): {}",
                    result.exit_code, stderr_snippet
                )
            };
            results.push(result);
            return Err(Error::HookFailed {
                event: event.to_string(),
                command: hook.command.clone(),
                message,
            });
        }

        results.push(result);
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

    // Validate working_dir is within the repository root (default_dir) to prevent
    // hooks from executing in arbitrary directories
    if let Some(ref custom_dir) = hook.working_dir
        && let (Ok(canon_custom), Ok(canon_default)) =
            (custom_dir.canonicalize(), default_dir.canonicalize())
        && !canon_custom.starts_with(&canon_default) {
            return Err(Error::HookFailed {
                event: hook.event.to_string(),
                command: hook.command.clone(),
                message: format!(
                    "Hook working_dir {:?} is outside the repository root {:?}",
                    custom_dir, default_dir
                ),
            });
        }
        // If canonicalize fails (directory doesn't exist yet), allow it — the
        // Command::new call will fail with a clear OS error.

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
            HookEvent::parse("post-sync"),
            Some(HookEvent::PostSync)
        );
        assert_eq!(HookEvent::parse("invalid"), None);
        // Agent events should no longer parse
        assert_eq!(HookEvent::parse("pre-agent-complete"), None);
        assert_eq!(HookEvent::parse("post-agent-complete"), None);
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
    fn test_hook_context_for_sync() {
        let ctx = HookContext::for_sync();
        assert_eq!(ctx.vars["HOOK_EVENT_TYPE"], "sync");
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

    /// Verify HookEvent has exactly 6 variants (pre/post for branch-create,
    /// branch-delete, sync). This catches unwired events being added without
    /// updating all_names() and the rest of the matching infrastructure.
    #[test]
    fn test_hook_event_enum_has_no_agent_events() {
        let names = HookEvent::all_names();
        assert_eq!(
            names.len(),
            6,
            "Expected exactly 6 hook events (pre/post for branch-create, branch-delete, sync), \
             found {}. If you added a new event, make sure it is wired to a call site.",
            names.len()
        );

        // Verify the exact set of expected events
        let expected = [
            "pre-branch-create",
            "post-branch-create",
            "pre-branch-delete",
            "post-branch-delete",
            "pre-sync",
            "post-sync",
        ];
        for name in &expected {
            assert!(
                names.contains(name),
                "Expected event '{}' not found in all_names()",
                name
            );
        }
    }

    /// Verify PreSync serializes to "pre-sync" in kebab-case via serde
    #[test]
    fn test_pre_sync_event_serializes_correctly() {
        let json = serde_json::to_string(&HookEvent::PreSync).unwrap();
        assert_eq!(json, "\"pre-sync\"");
        // Also verify Display
        assert_eq!(HookEvent::PreSync.to_string(), "pre-sync");
        // And round-trip parse
        assert_eq!(HookEvent::parse("pre-sync"), Some(HookEvent::PreSync));
    }

    /// Verify PostSync serializes to "post-sync" in kebab-case via serde
    #[test]
    fn test_post_sync_event_serializes_correctly() {
        let json = serde_json::to_string(&HookEvent::PostSync).unwrap();
        assert_eq!(json, "\"post-sync\"");
        // Also verify Display
        assert_eq!(HookEvent::PostSync.to_string(), "post-sync");
        // And round-trip parse
        assert_eq!(HookEvent::parse("post-sync"), Some(HookEvent::PostSync));
    }

    /// Verify that run_hooks executes a matching hook by checking for a
    /// marker file side effect. This is NOT a return-value-only test.
    #[test]
    fn test_run_hooks_executes_matching_event() {
        let temp = tempfile::TempDir::new().unwrap();
        let marker_path = temp.path().join("pre-sync-marker.txt");

        // Create a hook that touches a marker file when PreSync fires
        let hooks = vec![HookConfig {
            event: HookEvent::PreSync,
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                format!("echo 'hook ran' > '{}'", marker_path.display()),
            ],
            working_dir: None,
        }];

        let ctx = HookContext::for_sync();
        let results = run_hooks(&hooks, HookEvent::PreSync, &ctx, temp.path()).unwrap();

        // Verify the hook actually executed via side effect
        assert!(
            marker_path.exists(),
            "Marker file should exist — the pre-sync hook must have actually executed"
        );
        let content = std::fs::read_to_string(&marker_path).unwrap();
        assert!(
            content.contains("hook ran"),
            "Marker file should contain 'hook ran', got: {:?}",
            content
        );

        // Also verify the result metadata
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert_eq!(results[0].event, HookEvent::PreSync);
    }

    /// Verify that run_hooks does NOT execute a hook when the event does not
    /// match. This is the required negative test.
    #[test]
    fn test_run_hooks_skips_non_matching_event() {
        let temp = tempfile::TempDir::new().unwrap();
        let marker_path = temp.path().join("should-not-exist.txt");

        // Configure a hook for PreSync only
        let hooks = vec![HookConfig {
            event: HookEvent::PreSync,
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                format!("echo 'oops' > '{}'", marker_path.display()),
            ],
            working_dir: None,
        }];

        let ctx = HookContext::for_sync();
        // Fire PostSync — the PreSync hook should NOT run
        let results = run_hooks(&hooks, HookEvent::PostSync, &ctx, temp.path()).unwrap();

        assert!(
            results.is_empty(),
            "No hooks should have matched PostSync event"
        );
        assert!(
            !marker_path.exists(),
            "Marker file should NOT exist — the hook must not fire for a non-matching event"
        );
    }

    /// Verify that run_hooks returns an error when a hook script exits with
    /// a non-zero exit code.
    #[test]
    fn test_run_hooks_returns_error_on_hook_failure() {
        let temp = tempfile::TempDir::new().unwrap();

        // Create a hook that deliberately fails with exit code 1
        let hooks = vec![HookConfig {
            event: HookEvent::PreSync,
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'failing on purpose' >&2; exit 1".to_string(),
            ],
            working_dir: None,
        }];

        let ctx = HookContext::for_sync();
        let result = run_hooks(&hooks, HookEvent::PreSync, &ctx, temp.path());

        assert!(
            result.is_err(),
            "run_hooks should return Err when a hook exits with non-zero status"
        );

        // Verify the error message contains actionable information
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("pre-sync"),
            "Error should mention the event name, got: {:?}",
            err_msg
        );
        assert!(
            err_msg.contains("failing on purpose"),
            "Error should include stderr from the hook, got: {:?}",
            err_msg
        );
    }
}
