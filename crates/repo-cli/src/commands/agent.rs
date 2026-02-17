//! Agent command implementations
//!
//! Provides AI agent management operations. Uses repo-agent crate for
//! vaultspec discovery and subprocess execution.

use colored::Colorize;
use repo_agent::discovery::AgentManager;
use repo_agent::subprocess;

use crate::cli::{AgentAction, RulesSubAction};
use crate::error::Result;

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
fn discover_manager() -> std::result::Result<AgentManager, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("Cannot determine working directory: {e}"))?;
    AgentManager::discover(&cwd).map_err(|e| format!("Agent discovery failed: {e}"))
}

fn run_agent_check() -> Result<()> {
    println!("{} Checking agent prerequisites...\n", "=>".blue().bold());

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
            "  ✗".red().bold()
        } else {
            "  ✓".green().bold()
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
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    if !manager.is_available() {
        println!(
            "{} Agent subsystem not available. Run {} to check prerequisites.",
            "note:".yellow().bold(),
            "repo agent check".cyan()
        );
        return Ok(());
    }

    let vaultspec_path = match manager.vaultspec_path() {
        Some(p) => p.to_path_buf(),
        None => {
            println!("{} Vaultspec directory not found.", "error:".red().bold());
            return Ok(());
        }
    };

    match subprocess::list_agents(&vaultspec_path) {
        Ok(agents) if agents.is_empty() => {
            println!(
                "{} No agents defined. Add agent definitions to {}/agents/",
                "note:".yellow().bold(),
                vaultspec_path.display()
            );
        }
        Ok(agents) => {
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
            println!("  {}", "─".repeat(50).dimmed());
            for agent in &agents {
                println!(
                    "  {:<20} {:<15} {}",
                    agent.name.cyan(),
                    agent.tier,
                    agent.provider
                );
            }
        }
        Err(e) => {
            println!("{} Failed to list agents: {}", "error:".red().bold(), e);
        }
    }

    Ok(())
}

fn run_agent_spawn(name: &str, goal: Option<&str>, worktree: Option<&str>) -> Result<()> {
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    if !manager.is_available() {
        println!(
            "{} Agent subsystem not available. Run {} to check prerequisites.",
            "note:".yellow().bold(),
            "repo agent check".cyan()
        );
        return Ok(());
    }

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

    // Determine working directory (worktree or current)
    let work_dir = if let Some(wt) = worktree {
        let wt_path = manager.root().join(wt);
        if !wt_path.is_dir() {
            println!(
                "{} Worktree '{}' not found at {}",
                "error:".red().bold(),
                wt,
                wt_path.display()
            );
            return Ok(());
        }
        wt_path
    } else {
        manager.root().to_path_buf()
    };

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();

    // Build vaultspec spawn command
    let mut args = vec!["agent", "run", name];
    if let Some(g) = goal {
        args.push("--goal");
        args.push(g);
    }

    match subprocess::run_vaultspec(python, vaultspec, &work_dir, &args) {
        Ok(output) => {
            if !output.trim().is_empty() {
                println!("{}", output);
            }
            println!(
                "\n{} Agent '{}' spawned successfully.",
                "✓".green().bold(),
                name.cyan()
            );
        }
        Err(e) => {
            println!("{} Failed to spawn agent: {}", "error:".red().bold(), e);
        }
    }

    Ok(())
}

fn run_agent_status(task_id: Option<&str>) -> Result<()> {
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    if !manager.is_available() {
        println!(
            "{} Agent subsystem not available. Run {} to check prerequisites.",
            "note:".yellow().bold(),
            "repo agent check".cyan()
        );
        return Ok(());
    }

    match task_id {
        Some(id) => {
            println!(
                "{} Checking status of task '{}'...",
                "=>".blue().bold(),
                id.cyan()
            );
        }
        None => {
            println!("{} Checking status of all agents...", "=>".blue().bold());
        }
    }

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();

    let mut args = vec!["task", "status"];
    if let Some(id) = task_id {
        args.push(id);
    }

    match subprocess::run_vaultspec(python, vaultspec, manager.root(), &args) {
        Ok(output) => {
            if !output.trim().is_empty() {
                println!("{}", output);
            } else {
                println!("{} No active tasks.", "note:".yellow().bold());
            }
        }
        Err(e) => {
            println!("{} Failed to get status: {}", "error:".red().bold(), e);
        }
    }

    Ok(())
}

fn run_agent_stop(task_id: &str) -> Result<()> {
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    if !manager.is_available() {
        println!(
            "{} Agent subsystem not available. Run {} to check prerequisites.",
            "note:".yellow().bold(),
            "repo agent check".cyan()
        );
        return Ok(());
    }

    println!(
        "{} Stopping agent task '{}'...",
        "=>".blue().bold(),
        task_id.cyan()
    );

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();

    match subprocess::run_vaultspec(python, vaultspec, manager.root(), &["task", "stop", task_id]) {
        Ok(output) => {
            if !output.trim().is_empty() {
                println!("{}", output);
            }
            println!(
                "{} Task '{}' stopped.",
                "✓".green().bold(),
                task_id.cyan()
            );
        }
        Err(e) => {
            println!("{} Failed to stop task: {}", "error:".red().bold(), e);
        }
    }

    Ok(())
}

fn run_agent_sync() -> Result<()> {
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    if !manager.is_available() {
        println!(
            "{} Agent subsystem not available. Run {} to check prerequisites.",
            "note:".yellow().bold(),
            "repo agent check".cyan()
        );
        return Ok(());
    }

    println!("{} Syncing agent configuration...", "=>".blue().bold());

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();

    match subprocess::run_vaultspec(python, vaultspec, manager.root(), &["config", "sync"]) {
        Ok(output) => {
            if !output.trim().is_empty() {
                println!("{}", output);
            }
            println!("{} Agent configuration synced.", "✓".green().bold());
        }
        Err(e) => {
            println!("{} Failed to sync: {}", "error:".red().bold(), e);
        }
    }

    Ok(())
}

fn run_agent_config(show: bool) -> Result<()> {
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
    let manager = match discover_manager() {
        Ok(m) => m,
        Err(e) => {
            println!("{} {}", "error:".red().bold(), e);
            return Ok(());
        }
    };

    if !manager.is_available() {
        println!(
            "{} Agent subsystem not available. Run {} to check prerequisites.",
            "note:".yellow().bold(),
            "repo agent check".cyan()
        );
        return Ok(());
    }

    let python = manager.python_path().unwrap();
    let vaultspec = manager.vaultspec_path().unwrap();

    match action {
        RulesSubAction::List => {
            println!("{} Agent rules:\n", "=>".blue().bold());
            match subprocess::run_vaultspec(python, vaultspec, manager.root(), &["rules", "list"]) {
                Ok(output) => {
                    if output.trim().is_empty() {
                        println!("  No rules defined.");
                    } else {
                        println!("{}", output);
                    }
                }
                Err(e) => println!("{} Failed to list rules: {}", "error:".red().bold(), e),
            }
        }
        RulesSubAction::Add { id, instruction } => {
            println!(
                "{} Adding rule '{}'...",
                "=>".blue().bold(),
                id.cyan()
            );
            match subprocess::run_vaultspec(
                python,
                vaultspec,
                manager.root(),
                &["rules", "add", &id, "--instruction", &instruction],
            ) {
                Ok(_) => println!("{} Rule '{}' added.", "✓".green().bold(), id.cyan()),
                Err(e) => println!("{} Failed to add rule: {}", "error:".red().bold(), e),
            }
        }
        RulesSubAction::Remove { id } => {
            println!(
                "{} Removing rule '{}'...",
                "=>".blue().bold(),
                id.cyan()
            );
            match subprocess::run_vaultspec(
                python,
                vaultspec,
                manager.root(),
                &["rules", "remove", &id],
            ) {
                Ok(_) => println!("{} Rule '{}' removed.", "✓".green().bold(), id.cyan()),
                Err(e) => println!("{} Failed to remove rule: {}", "error:".red().bold(), e),
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_check_runs() {
        // This test exercises the real discovery path - it should
        // not fail regardless of whether vaultspec is installed
        let result = run_agent_check();
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_list_runs() {
        let result = run_agent_list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_spawn_runs() {
        let result = run_agent_spawn("claude", Some("fix bug"), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_status_all() {
        let result = run_agent_status(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_status_specific() {
        let result = run_agent_status(Some("task-123"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_stop() {
        let result = run_agent_stop("task-123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_sync() {
        let result = run_agent_sync();
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_config_show() {
        let result = run_agent_config(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_config_no_show() {
        let result = run_agent_config(false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_rules_list() {
        let result = run_agent_rules(RulesSubAction::List);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_rules_add() {
        let result = run_agent_rules(RulesSubAction::Add {
            id: "test-rule".to_string(),
            instruction: "Test instruction".to_string(),
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_rules_remove() {
        let result = run_agent_rules(RulesSubAction::Remove {
            id: "test-rule".to_string(),
        });
        assert!(result.is_ok());
    }
}
