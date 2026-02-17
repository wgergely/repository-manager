//! Agent command implementations
//!
//! Provides AI agent management operations. Requires vaultspec for full functionality.

use colored::Colorize;

use crate::cli::{AgentAction, RulesSubAction};
use crate::error::Result;

const VAULTSPEC_MSG: &str = "Agent features require vaultspec. Install with: pip install vaultspec";

/// Run the agent command dispatcher.
pub fn run_agent(action: AgentAction) -> Result<()> {
    match action {
        AgentAction::Check => run_agent_check(),
        AgentAction::List => run_agent_list(),
        AgentAction::Spawn { name, goal, worktree } => run_agent_spawn(&name, goal.as_deref(), worktree.as_deref()),
        AgentAction::Status { task_id } => run_agent_status(task_id.as_deref()),
        AgentAction::Stop { task_id } => run_agent_stop(&task_id),
        AgentAction::Sync => run_agent_sync(),
        AgentAction::Config { show } => run_agent_config(show),
        AgentAction::Rules { action } => run_agent_rules(action),
    }
}

fn run_agent_check() -> Result<()> {
    println!("{} Checking agent prerequisites...", "=>".blue().bold());
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

fn run_agent_list() -> Result<()> {
    println!("{} Listing available agents...", "=>".blue().bold());
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

fn run_agent_spawn(name: &str, goal: Option<&str>, worktree: Option<&str>) -> Result<()> {
    println!(
        "{} Spawning agent '{}'{}{}...",
        "=>".blue().bold(),
        name.cyan(),
        goal.map(|g| format!(" with goal: {}", g.yellow())).unwrap_or_default(),
        worktree.map(|w| format!(" in worktree: {}", w.yellow())).unwrap_or_default(),
    );
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

fn run_agent_status(task_id: Option<&str>) -> Result<()> {
    match task_id {
        Some(id) => println!("{} Checking status of task '{}'...", "=>".blue().bold(), id.cyan()),
        None => println!("{} Checking status of all agents...", "=>".blue().bold()),
    }
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

fn run_agent_stop(task_id: &str) -> Result<()> {
    println!("{} Stopping agent task '{}'...", "=>".blue().bold(), task_id.cyan());
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

fn run_agent_sync() -> Result<()> {
    println!("{} Syncing agent configuration...", "=>".blue().bold());
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

fn run_agent_config(show: bool) -> Result<()> {
    if show {
        println!("{} Agent configuration:", "=>".blue().bold());
    } else {
        println!("{} Agent configuration (use --show to display):", "=>".blue().bold());
    }
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

fn run_agent_rules(action: RulesSubAction) -> Result<()> {
    match action {
        RulesSubAction::List => {
            println!("{} Listing agent rules...", "=>".blue().bold());
        }
        RulesSubAction::Add { id, instruction } => {
            println!(
                "{} Adding agent rule '{}': {}",
                "=>".blue().bold(),
                id.cyan(),
                instruction.dimmed()
            );
        }
        RulesSubAction::Remove { id } => {
            println!("{} Removing agent rule '{}'...", "=>".blue().bold(), id.cyan());
        }
    }
    println!("{} {}", "note:".yellow().bold(), VAULTSPEC_MSG);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_check() {
        let result = run_agent_check();
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_list() {
        let result = run_agent_list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_spawn() {
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
