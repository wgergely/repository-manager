//! Process management for long-running agent sessions
//!
//! Tracks spawned agent processes, persists PID state to disk,
//! and handles graceful shutdown via signal forwarding.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::error::{AgentError, Result};

/// Directory name for agent state files within `.repository/`
const AGENT_STATE_DIR: &str = "agents";
/// State file name
const STATE_FILE: &str = "running.json";

/// Record of a spawned agent process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProcess {
    /// Unique identifier for this agent session
    pub id: String,
    /// Agent name/type (e.g., "researcher", "coder")
    pub agent_name: String,
    /// Process ID of the spawned process
    pub pid: u32,
    /// Worktree the agent is working in
    pub worktree: Option<String>,
    /// Goal/task description
    pub goal: Option<String>,
    /// When the agent was spawned (Unix timestamp)
    pub started_at: u64,
    /// Current status
    pub status: ProcessStatus,
}

/// Status of a tracked agent process
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
    /// Process is running
    Running,
    /// Process exited normally
    Completed,
    /// Process was stopped by user
    Stopped,
    /// Process exited with error
    Failed,
    /// Process state unknown (e.g., stale PID)
    Unknown,
}

/// Manages agent process lifecycle
#[derive(Debug)]
pub struct ProcessManager {
    /// Root directory containing `.repository/`
    root: PathBuf,
    /// Path to the state file
    state_path: PathBuf,
}

impl ProcessManager {
    /// Create a new process manager for the given repository root
    pub fn new(root: &Path) -> Self {
        let state_dir = root.join(".repository").join(AGENT_STATE_DIR);
        let state_path = state_dir.join(STATE_FILE);
        Self {
            root: root.to_path_buf(),
            state_path,
        }
    }

    /// Spawn an agent process and track it
    ///
    /// Invokes `python -m vaultspec agent run <name>` in the target directory
    /// and records the PID for later management.
    pub fn spawn(
        &self,
        python_path: &Path,
        vaultspec_path: &Path,
        agent_name: &str,
        worktree: Option<&str>,
        goal: Option<&str>,
    ) -> Result<AgentProcess> {
        let work_dir = if let Some(wt) = worktree {
            let wt_path = self.root.join(wt);
            if !wt_path.is_dir() {
                return Err(AgentError::ParseError(format!(
                    "Worktree '{}' not found at {}",
                    wt,
                    wt_path.display()
                )));
            }
            wt_path
        } else {
            self.root.clone()
        };

        let mut cmd = Command::new(python_path);
        cmd.current_dir(&work_dir)
            .env("VAULTSPEC_HOME", vaultspec_path)
            .arg("-m")
            .arg("vaultspec")
            .arg("agent")
            .arg("run")
            .arg(agent_name)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        if let Some(g) = goal {
            cmd.arg("--goal").arg(g);
        }

        let child: Child = cmd.spawn().map_err(AgentError::Io)?;
        let pid = child.id();

        let id = format!(
            "{}-{}-{}",
            agent_name,
            worktree.unwrap_or("root"),
            pid
        );

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let process = AgentProcess {
            id: id.clone(),
            agent_name: agent_name.to_string(),
            pid,
            worktree: worktree.map(|s| s.to_string()),
            goal: goal.map(|s| s.to_string()),
            started_at: now,
            status: ProcessStatus::Running,
        };

        // Save to state file
        let mut state = self.load_state()?;
        state.insert(id, process.clone());
        self.save_state(&state)?;

        // The Child handle is dropped here. Since stdout/stderr are set to
        // Stdio::null(), dropping the handle does not affect the child process
        // which continues running independently. We track it by PID.
        drop(child);

        Ok(process)
    }

    /// List all tracked agent processes, updating their status
    pub fn list(&self) -> Result<Vec<AgentProcess>> {
        let mut state = self.load_state()?;
        let mut changed = false;

        for process in state.values_mut() {
            if process.status == ProcessStatus::Running
                && !is_process_alive(process.pid)
            {
                process.status = ProcessStatus::Unknown;
                changed = true;
            }
        }

        if changed {
            self.save_state(&state)?;
        }

        let mut processes: Vec<_> = state.into_values().collect();
        processes.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(processes)
    }

    /// Stop an agent process by ID or PID
    pub fn stop(&self, id_or_pid: &str) -> Result<AgentProcess> {
        let mut state = self.load_state()?;

        // Find by ID first, then by PID
        let key = if state.contains_key(id_or_pid) {
            id_or_pid.to_string()
        } else if let Ok(pid) = id_or_pid.parse::<u32>() {
            state
                .iter()
                .find(|(_, p)| p.pid == pid)
                .map(|(k, _)| k.clone())
                .ok_or_else(|| {
                    AgentError::ParseError(format!("No agent found with PID {}", pid))
                })?
        } else {
            return Err(AgentError::ParseError(format!(
                "No agent found with ID '{}'",
                id_or_pid
            )));
        };

        let process = state
            .get_mut(&key)
            .ok_or_else(|| AgentError::ParseError(format!("Agent '{}' not found", key)))?;

        if process.status == ProcessStatus::Running {
            kill_process(process.pid);
            process.status = ProcessStatus::Stopped;
        }

        let result = process.clone();
        self.save_state(&state)?;
        Ok(result)
    }

    /// Clean up state file by removing completed/stopped/unknown entries
    pub fn cleanup(&self) -> Result<usize> {
        let mut state = self.load_state()?;
        let before = state.len();

        state.retain(|_, p| p.status == ProcessStatus::Running);

        let removed = before - state.len();
        if removed > 0 {
            self.save_state(&state)?;
        }
        Ok(removed)
    }

    /// Load state from disk
    fn load_state(&self) -> Result<HashMap<String, AgentProcess>> {
        if !self.state_path.exists() {
            return Ok(HashMap::new());
        }

        let content = std::fs::read_to_string(&self.state_path).map_err(AgentError::Io)?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }

        serde_json::from_str(&content)
            .map_err(|e| AgentError::ParseError(format!("Invalid state file: {}", e)))
    }

    /// Save state to disk
    fn save_state(&self, state: &HashMap<String, AgentProcess>) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.state_path.parent() {
            std::fs::create_dir_all(parent).map_err(AgentError::Io)?;
        }

        let content = serde_json::to_string_pretty(state)
            .map_err(|e| AgentError::ParseError(format!("Failed to serialize state: {}", e)))?;
        std::fs::write(&self.state_path, content).map_err(AgentError::Io)?;
        Ok(())
    }
}

/// Check if a process is still alive by PID
fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // Use kill -0 to check if process exists (no signal sent)
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|o| {
                let out = String::from_utf8_lossy(&o.stdout);
                out.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        false
    }
}

/// Kill a process by PID
fn kill_process(pid: u32) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output();
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string()])
            .output();
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, ProcessManager) {
        let temp = TempDir::new().unwrap();
        let pm = ProcessManager::new(temp.path());
        (temp, pm)
    }

    #[test]
    fn test_new_process_manager() {
        let temp = TempDir::new().unwrap();
        let pm = ProcessManager::new(temp.path());
        assert_eq!(pm.root, temp.path());
    }

    #[test]
    fn test_load_empty_state() {
        let (_temp, pm) = setup();
        let state = pm.load_state().unwrap();
        assert!(state.is_empty());
    }

    #[test]
    fn test_save_and_load_state() {
        let (_temp, pm) = setup();

        let mut state = HashMap::new();
        state.insert(
            "test-1".to_string(),
            AgentProcess {
                id: "test-1".to_string(),
                agent_name: "researcher".to_string(),
                pid: 12345,
                worktree: Some("feature-x".to_string()),
                goal: Some("research APIs".to_string()),
                started_at: 1000,
                status: ProcessStatus::Running,
            },
        );

        pm.save_state(&state).unwrap();
        let loaded = pm.load_state().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded["test-1"].agent_name, "researcher");
        assert_eq!(loaded["test-1"].pid, 12345);
    }

    #[test]
    fn test_list_empty() {
        let (_temp, pm) = setup();
        let list = pm.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_with_entries() {
        let (_temp, pm) = setup();

        let mut state = HashMap::new();
        state.insert(
            "a-1".to_string(),
            AgentProcess {
                id: "a-1".to_string(),
                agent_name: "coder".to_string(),
                pid: 99999, // Non-existent PID
                worktree: None,
                goal: None,
                started_at: 2000,
                status: ProcessStatus::Running,
            },
        );
        state.insert(
            "b-2".to_string(),
            AgentProcess {
                id: "b-2".to_string(),
                agent_name: "tester".to_string(),
                pid: 88888,
                worktree: None,
                goal: None,
                started_at: 1000,
                status: ProcessStatus::Completed,
            },
        );
        pm.save_state(&state).unwrap();

        let list = pm.list().unwrap();
        assert_eq!(list.len(), 2);
        // Sorted by started_at descending
        assert_eq!(list[0].id, "a-1");
        assert_eq!(list[1].id, "b-2");
    }

    #[test]
    fn test_stop_nonexistent() {
        let (_temp, pm) = setup();
        let result = pm.stop("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_stop_by_id() {
        let (_temp, pm) = setup();

        let mut state = HashMap::new();
        state.insert(
            "test-stop".to_string(),
            AgentProcess {
                id: "test-stop".to_string(),
                agent_name: "worker".to_string(),
                pid: 77777, // Non-existent PID
                worktree: None,
                goal: None,
                started_at: 3000,
                status: ProcessStatus::Running,
            },
        );
        pm.save_state(&state).unwrap();

        let stopped = pm.stop("test-stop").unwrap();
        assert_eq!(stopped.status, ProcessStatus::Stopped);
    }

    #[test]
    fn test_cleanup() {
        let (_temp, pm) = setup();

        let mut state = HashMap::new();
        state.insert(
            "running-1".to_string(),
            AgentProcess {
                id: "running-1".to_string(),
                agent_name: "a".to_string(),
                pid: 11111,
                worktree: None,
                goal: None,
                started_at: 1000,
                status: ProcessStatus::Running,
            },
        );
        state.insert(
            "done-1".to_string(),
            AgentProcess {
                id: "done-1".to_string(),
                agent_name: "b".to_string(),
                pid: 22222,
                worktree: None,
                goal: None,
                started_at: 900,
                status: ProcessStatus::Completed,
            },
        );
        state.insert(
            "stopped-1".to_string(),
            AgentProcess {
                id: "stopped-1".to_string(),
                agent_name: "c".to_string(),
                pid: 33333,
                worktree: None,
                goal: None,
                started_at: 800,
                status: ProcessStatus::Stopped,
            },
        );
        pm.save_state(&state).unwrap();

        let removed = pm.cleanup().unwrap();
        assert_eq!(removed, 2);

        let remaining = pm.load_state().unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(remaining.contains_key("running-1"));
    }

    #[test]
    fn test_process_status_serialization() {
        let json = serde_json::to_string(&ProcessStatus::Running).unwrap();
        assert_eq!(json, "\"running\"");

        let status: ProcessStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(status, ProcessStatus::Completed);
    }

    #[test]
    fn test_agent_process_serialization() {
        let process = AgentProcess {
            id: "test-1".to_string(),
            agent_name: "coder".to_string(),
            pid: 42,
            worktree: Some("feature".to_string()),
            goal: Some("fix bugs".to_string()),
            started_at: 1234567890,
            status: ProcessStatus::Running,
        };

        let json = serde_json::to_string(&process).unwrap();
        let deserialized: AgentProcess = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-1");
        assert_eq!(deserialized.pid, 42);
        assert_eq!(deserialized.status, ProcessStatus::Running);
    }
}
