//! Superpowers plugin management commands

use crate::cli::SuperpowersAction;
use crate::error::{CliError, Result};
use repo_fs::{LayoutMode, NormalizedPath, WorkspaceLayout};
use repo_presets::{Context, PresetProvider, PresetStatus, SuperpowersProvider};
use std::collections::HashMap;

pub async fn handle_superpowers(action: SuperpowersAction) -> Result<()> {
    // Create a minimal context (superpowers doesn't use project root)
    let current_dir = std::env::current_dir()?;
    let layout = WorkspaceLayout {
        root: NormalizedPath::new(&current_dir),
        active_context: NormalizedPath::new(&current_dir),
        mode: LayoutMode::Classic,
    };
    let context = Context::new(layout, HashMap::new());

    match action {
        SuperpowersAction::Install { version } => {
            let provider = SuperpowersProvider::new().with_version(&version);

            println!("Checking superpowers status...");
            let check = provider.check(&context).await?;

            if check.status == PresetStatus::Healthy {
                println!("Superpowers {} is already installed and enabled.", version);
                return Ok(());
            }

            println!("Installing superpowers {}...", version);
            let report = provider.apply(&context).await?;

            for action in &report.actions_taken {
                println!("  {}", action);
            }

            if report.success {
                println!("Superpowers {} installed successfully!", version);
            } else {
                for err in &report.errors {
                    eprintln!("Error: {}", err);
                }
                return Err(CliError::user("Installation failed"));
            }
        }

        SuperpowersAction::Status => {
            let provider = SuperpowersProvider::new();
            let check = provider.check(&context).await?;

            println!("Superpowers status: {:?}", check.status);
            for detail in &check.details {
                println!("  {}", detail);
            }
        }

        SuperpowersAction::Uninstall { version } => {
            let provider = SuperpowersProvider::new().with_version(&version);

            println!("Uninstalling superpowers {}...", version);
            let report = provider.uninstall(&context).await?;

            for action in &report.actions_taken {
                println!("  {}", action);
            }

            if report.success {
                println!("Superpowers {} uninstalled.", version);
            } else {
                for err in &report.errors {
                    eprintln!("Error: {}", err);
                }
            }
        }
    }

    Ok(())
}
