//! Shared types for agent operations

use serde::{Deserialize, Serialize};

/// Information about a discovered agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent name (e.g., "researcher", "coder")
    pub name: String,
    /// Agent tier (e.g., "orchestrator", "specialist", "worker")
    pub tier: String,
    /// Provider used by this agent (e.g., "claude", "gemini")
    pub provider: String,
    /// Whether the agent is currently available
    pub available: bool,
}

/// Status of a dispatched task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task is queued but not yet started
    Pending,
    /// Task is currently running
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
}

/// Health report for the agent subsystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Whether the agent subsystem is fully available
    pub available: bool,
    /// Path to the Python interpreter, if found
    pub python_path: Option<String>,
    /// Python version string, if found
    pub python_version: Option<String>,
    /// Path to the vaultspec framework directory, if found
    pub vaultspec_path: Option<String>,
    /// Number of agents discovered
    pub agent_count: usize,
    /// Human-readable status messages
    pub messages: Vec<String>,
}

impl HealthReport {
    /// Create a report indicating the agent subsystem is unavailable
    pub fn unavailable(messages: Vec<String>) -> Self {
        Self {
            available: false,
            python_path: None,
            python_version: None,
            vaultspec_path: None,
            agent_count: 0,
            messages,
        }
    }
}
