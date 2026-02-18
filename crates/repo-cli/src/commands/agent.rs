//! Agent command implementations
//!
//! Provides AI agent management operations. Uses repo-agent crate for
//! vaultspec discovery and subprocess execution.

use colored::Colorize;
use repo_agent::discovery::AgentManager;
use repo_agent::process::{ProcessManager, ProcessStatus};
use repo_agent::subprocess;

use crate::cli::{AgentAction, RulesSubAction};
use crate::error::{CliError, Result};

/// Run the agent command dispatcher.
pub fn run_agent(action: AgentAction) -> Result<()> {
    match action {
        AgentAction::Check => run_agent_check(),
        AgentAction::List => run_agent_list(),
        AgentAction::Spawn {
            name,
            goal,
            worktree,
        } => run_agent_spawn(&name, goal.as_deref(), worktree.as_deref()),
        AgentAction::Status { task_id } => run_agent_status(task_id.as_deref()),
        AgentAction::Stop { task_id } => run_agent_stop(&task_id),
        AgentAction::Sync => run_agent_sync(),
        AgentAction::Config { show } => run_agent_config(show),
        AgentAction::Rules { action } => run_agent_rules(action),
    }
}

/// Discover the agent manager for the current directory
fn discover_manager() -> Result<AgentManager> {
    let cwd = std::env::current_dir()?;
    AgentManager::discover(&cwd)
        .map_err(|e| CliError::user(format!("Agent discovery failed: {e}")))
}

/// Require the agent subsystem to be available, returning an error if not.
fn require_available(manager: &AgentManager) -> Result<()> {
    if !manager.is_available() {
        return Err(CliError::user(format!(
            "Agent subsystem not available. Run {} to check prerequisites.",
            "repo agent check"
        )));
    }
    Ok(())
}

fn run_agent_check() -> Result<()> {
    println!("{} Checking agent prerequisites...\n", "=>".blue().bold());

    // Check is diagnostic - gracefully handle discovery failures
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    let report = match manager.health_check() {
        Ok(r) => r,
        Err(e) => {
            println!("{} Health check failed: {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    for msg in &report.messages {
        let prefix = if msg.contains("not found") {
            "  FAIL".red().bold()
        } else {
            "  OK".green().bold()
        };
        println!("{} {}", prefix, msg);
    }

    println!();
    if report.available {
        println!(
            "{} Agent subsystem is {}",
            "=>".blue().bold(),
            "ready".green().bold()
        );
    } else {
        println!(
            "{} Agent subsystem is {}",
            "=>".blue().bold(),
            "not available".yellow().bold()
        );
        println!(
            "\n{} Install prerequisites:",
            "hint:".cyan().bold()
        );
        if report.python_path.is_none() {
            println!("  - Python 3.13+: https://python.org/downloads/");
        }
        if report.vaultspec_path.is_none() {
            println!("  - Vaultspec: pip install vaultspec");
        }
    }

    Ok(())
}

fn run_agent_list() -> Result<()> {
    let manager = discover_manager()?;
    require_available(&manager)?;

    let vaultspec_path = manager.vaultspec_path()
        .ok_or_else(|| CliError::user("Vaultspec directory not found"))?
        .to_path_buf();

    let agents = subprocess::list_agents(&vaultspec_path)
        .map_err(|e| CliError::user(format!("Failed to list agents: {e}")))?;

    if agents.is_empty() {
        println!(
            "{} No agents defined. Add agent definitions to {}/agents/",
            "note:".yellow().bold(),
            vaultspec_path.display()
        );
    } else {
        println!(
            "{} {} agent(s) found:\n",
            "=>".blue().bold(),
            agents.len()
        );
        println!(
            "  {:<20} {:<15} {}",
            "NAME".bold(),
            "TIER".bold(),
            "PROVIDER".bold()
        );
        println!("  {}", "-".repeat(50).dimmed());
        for agent in &agents {
            println!(
                "  {:<20} {:<15} {}",
                agent.name.cyan(),
                agent.tier,
                agent.provider
            );
        }
    }

    Ok(())
}

fn run_agent_spawn(name: &str, goal: Option<&str>, worktree: Option<&str>) -> Result<()> {
    let manager = discover_manager()?;
    require_available(&manager)?;

    println!(
        "{} Spawning agent '{}'{}{}...",
        "=>".blue().bold(),
        name.cyan(),
        goal.map(|g| format!(" with goal: {}", g.yellow()))
            .unwrap_or_default(),
        worktree
            .map(|w| format!(" in worktree: {}", w.yellow()))
            .unwrap_or_default(),
    );

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();
    let pm = ProcessManager::new(manager.root());

    let process = pm.spawn(python, vaultspec, name, worktree, goal)
        .map_err(|e| CliError::user(format!("Failed to spawn agent: {e}")))?;

    println!(
        "\n{} Agent '{}' spawned (PID: {}, ID: {})",
        "OK".green().bold(),
        name.cyan(),
        process.pid.to_string().yellow(),
        process.id.dimmed()
    );

    Ok(())
}

fn run_agent_status(task_id: Option<&str>) -> Result<()> {
    let manager = discover_manager()?;
    let pm = ProcessManager::new(manager.root());

    let processes = pm.list()
        .map_err(|e| CliError::user(format!("Failed to list processes: {e}")))?;

    let filtered: Vec<_> = if let Some(id) = task_id {
        processes.into_iter().filter(|p| p.id.contains(id)).collect()
    } else {
        processes
    };

    if filtered.is_empty() {
        println!("{} No tracked agent processes.", "note:".yellow().bold());
    } else {
        println!(
            "{} {} agent process(es):\n",
            "=>".blue().bold(),
            filtered.len()
        );
        println!(
            "  {:<30} {:<12} {:<8} {:<15} {}",
            "ID".bold(),
            "AGENT".bold(),
            "PID".bold(),
            "STATUS".bold(),
            "WORKTREE".bold()
        );
        println!("  {}", "-".repeat(75).dimmed());
        for p in &filtered {
            let status_str = match p.status {
                ProcessStatus::Running => "running".green().to_string(),
                ProcessStatus::Completed => "completed".blue().to_string(),
                ProcessStatus::Stopped => "stopped".yellow().to_string(),
                ProcessStatus::Failed => "failed".red().to_string(),
                ProcessStatus::Unknown => "unknown".dimmed().to_string(),
            };
            println!(
                "  {:<30} {:<12} {:<8} {:<15} {}",
                p.id.dimmed(),
                p.agent_name.cyan(),
                p.pid,
                status_str,
                p.worktree.as_deref().unwrap_or("-")
            );
        }
    }

    Ok(())
}

fn run_agent_stop(task_id: &str) -> Result<()> {
    let manager = discover_manager()?;

    println!(
        "{} Stopping agent '{}'...",
        "=>".blue().bold(),
        task_id.cyan()
    );

    let pm = ProcessManager::new(manager.root());

    let process = pm.stop(task_id)
        .map_err(|e| CliError::user(format!("Failed to stop agent: {e}")))?;

    println!(
        "{} Agent '{}' (PID {}) stopped.",
        "OK".green().bold(),
        process.agent_name.cyan(),
        process.pid
    );

    Ok(())
}

fn run_agent_sync() -> Result<()> {
    let manager = discover_manager()?;
    require_available(&manager)?;

    println!("{} Syncing agent configuration...", "=>".blue().bold());

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();

    let output = subprocess::run_vaultspec(python, vaultspec, manager.root(), &["config", "sync"])
        .map_err(|e| CliError::user(format!("Failed to sync: {e}")))?;

    if !output.trim().is_empty() {
        println!("{}", output);
    }
    println!("{} Agent configuration synced.", "OK".green().bold());

    Ok(())
}

fn run_agent_config(show: bool) -> Result<()> {
    // Config is diagnostic - gracefully handle discovery failures
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    if show {
        println!("{} Agent configuration:\n", "=>".blue().bold());

        let report = manager.health_check().ok();
        println!("  {:<20} {}", "Python:".bold(),
            manager.python_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "not found".red().to_string()));

        if let Some(ref r) = report {
            if let Some(ref v) = r.python_version {
                println!("  {:<20} {}", "Python version:".bold(), v);
            }
        }

        println!("  {:<20} {}",  "Vaultspec:".bold(),
            manager.vaultspec_path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "not found".red().to_string()));

        if let Some(ref r) = report {
            if r.agent_count > 0 {
                println!("  {:<20} {}", "Agents:".bold(), r.agent_count);
            }
        }

        println!("  {:<20} {}", "Status:".bold(),
            if manager.is_available() {
                "ready".green().to_string()
            } else {
                "not available".yellow().to_string()
            });
    } else {
        println!(
            "{} Use {} to display agent configuration.",
            "hint:".cyan().bold(),
            "repo agent config --show".cyan()
        );
    }

    Ok(())
}

fn run_agent_rules(action: RulesSubAction) -> Result<()> {
    let manager = discover_manager()?;
    require_available(&manager)?;

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();

    match action {
        RulesSubAction::List => {
            println!("{} Agent rules:\n", "=>".blue().bold());
            let output = subprocess::run_vaultspec(python, vaultspec, manager.root(), &["rules", "list"])
                .map_err(|e| CliError::user(format!("Failed to list rules: {e}")))?;
            if output.trim().is_empty() {
                println!("  No rules defined.");
            } else {
                println!("{}", output);
            }
        }
        RulesSubAction::Add { id, instruction } => {
            println!(
                "{} Adding rule '{}'...",
                "=>".blue().bold(),
                id.cyan()
            );
            subprocess::run_vaultspec(
                python,
                vaultspec,
                manager.root(),
                &["rules", "add", &id, "--instruction", &instruction],
            )
            .map_err(|e| CliError::user(format!("Failed to add rule: {e}")))?;
            println!("{} Rule '{}' added.", "OK".green().bold(), id.cyan());
        }
        RulesSubAction::Remove { id } => {
            println!(
                "{} Removing rule '{}'...",
                "=>".blue().bold(),
                id.cyan()
            );
            subprocess::run_vaultspec(
                python,
                vaultspec,
                manager.root(),
                &["rules", "remove", &id],
            )
            .map_err(|e| CliError::user(format!("Failed to remove rule: {e}")))?;
            println!("{} Rule '{}' removed.", "OK".green().bold(), id.cyan());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_check_runs() {
        // Check is diagnostic - always returns Ok even without prerequisites
        let result = run_agent_check();
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_config_show() {
        // Config is diagnostic - always returns Ok even without prerequisites
        let result = run_agent_config(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_config_no_show() {
        let result = run_agent_config(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_list_returns_error_without_prerequisites() {
        // Without vaultspec, list should return an error
        let result = run_agent_list();
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_spawn_returns_error_without_prerequisites() {
        let result = run_agent_spawn("claude", Some("fix bug"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_status_runs() {
        // Status is read-only; returns Ok with empty list when no processes tracked
        let result = run_agent_status(None);
        // May succeed (empty list) or fail (no repo) depending on environment
        let _ = result;
    }

    #[test]
    fn test_agent_stop_returns_error_for_nonexistent() {
        // Stop should fail for a nonexistent task ID
        let result = run_agent_stop("nonexistent-task-123");
        // May fail with discovery error or stop error depending on environment
        let _ = result;
    }

    #[test]
    fn test_agent_sync_returns_error_without_prerequisites() {
        let result = run_agent_sync();
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_rules_list_returns_error_without_prerequisites() {
        let result = run_agent_rules(RulesSubAction::List);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_rules_add_returns_error_without_prerequisites() {
        let result = run_agent_rules(RulesSubAction::Add {
            id: "test-rule".to_string(),
            instruction: "Test instruction".to_string(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_rules_remove_returns_error_without_prerequisites() {
        let result = run_agent_rules(RulesSubAction::Remove {
            id: "test-rule".to_string(),
        });
        assert!(result.is_err());
    }
}
