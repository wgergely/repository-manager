//! Plugin management commands

use crate::cli::PluginsAction;
use crate::error::{CliError, Result};
use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::{Context, PluginsProvider, PresetProvider, PresetStatus};
use std::collections::HashMap;

pub async fn handle_plugins(action: PluginsAction) -> Result<()> {
    // Create a minimal context (plugins doesn't use project root)
    let current_dir = std::env::current_dir()?;
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(&current_dir),
        active_context: NormalizedPath::new(&current_dir),
        mode: LayoutMode::Classic,
    };
    let context = Context::new(layout, HashMap::new());

    match action {
        PluginsAction::Install { version } => {
            let provider = PluginsProvider::new().with_version(&version);

            println!("Checking plugin status...");
            let check = provider.check(&context).await?;

            if check.status == PresetStatus::Healthy {
                println!("Plugin {} is already installed and enabled.", version);
                return Ok(());
            }

            println!("Installing plugin {}...", version);
            let report = provider.apply(&context).await?;

            for action in &report.actions_taken {
                println!("  {}", action);
            }

            if report.success {
                println!("Plugin {} installed successfully!", version);
            } else {
                for err in &report.errors {
                    eprintln!("Error: {}", err);
                }
                return Err(CliError::user("Installation failed"));
            }
        }

        PluginsAction::Status => {
            let provider = PluginsProvider::new();
            let check = provider.check(&context).await?;

            println!("Plugin status: {:?}", check.status);
            for detail in &check.details {
                println!("  {}", detail);
            }
        }

        PluginsAction::Uninstall { version } => {
            let provider = PluginsProvider::new().with_version(&version);

            println!("Uninstalling plugin {}...", version);
            let report = provider.uninstall(&context).await?;

            for action in &report.actions_taken {
                println!("  {}", action);
            }

            if report.success {
                println!("Plugin {} uninstalled.", version);
            } else {
                for err in &report.errors {
                    eprintln!("Error: {}", err);
                }
            }
        }
    }

    Ok(())
}
