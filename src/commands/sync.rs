use clap::Subcommand;
use color_eyre::Result;

use crate::sync::{
    config::ConfigManager,
    engine::{LocalProvider, SyncEngine},
    providers::AzureProvider,
};

#[derive(Subcommand)]
pub enum SyncCommands {
    /// Initialize sync configuration
    Init {
        /// Cloud provider (local or azure)
        #[arg(default_value = "local")]
        provider: String,
    },
    /// Push local changes to remote
    Push,
    /// Pull remote changes to local
    Pull,
    /// Bidirectional sync (push + pull)
    Sync,
    /// Show sync status
    Status,
}

pub async fn handle_sync_command(command: SyncCommands) -> Result<()> {
    match command {
        SyncCommands::Init { provider } => {
            ConfigManager::create_config_for_provider(&provider)?;
            Ok(())
        }
        SyncCommands::Push => {
            let engine = create_sync_engine().await?;
            let result = engine.push().await?;
            result.print_summary();
            Ok(())
        }
        SyncCommands::Pull => {
            let engine = create_sync_engine().await?;
            let result = engine.pull().await?;
            result.print_summary();
            Ok(())
        }
        SyncCommands::Sync => {
            let engine = create_sync_engine().await?;
            let result = engine.sync().await?;
            result.print_summary();
            Ok(())
        }
        SyncCommands::Status => {
            println!("📊 Sync Status:");
            let config_manager = ConfigManager::load()?;
            match config_manager.sync_config {
                Some(config) => {
                    println!("  Provider: {}", config.provider);
                    match config.provider.as_str() {
                        "local" => {
                            if let Some(local_config) = &config.local {
                                println!("  Sync directory: {}", local_config.sync_dir);
                                let path = std::path::Path::new(&local_config.sync_dir);
                                println!("  Remote exists: {}", path.exists());
                            }
                        }
                        "azure" => {
                            if let Some(azure_config) = &config.azure {
                                println!("  Container: {}", azure_config.container_name);
                                if azure_config.connection_string.contains("REPLACE_WITH") {
                                    println!("  ⚠️  Connection string not configured");
                                } else {
                                    println!("  ✅ Connection string configured");
                                }
                            }
                        }
                        _ => {
                            println!("  ⚠️  Unknown provider: {}", config.provider);
                        }
                    }
                }
                None => {
                    println!(
                        "  No sync configuration found. Run 'devlog sync init' to get started."
                    );
                }
            }
            Ok(())
        }
    }
}

async fn create_sync_engine() -> Result<SyncEngine> {
    let config_manager = ConfigManager::load()?;
    let config = config_manager.sync_config.ok_or_else(|| {
        color_eyre::eyre::eyre!("No sync configuration found. Run 'devlog sync init' first.")
    })?;

    // Create provider based on config
    let provider: Box<dyn crate::sync::CloudStorage> = match config.provider.as_str() {
        "local" => {
            let local_config = config
                .local
                .ok_or_else(|| color_eyre::eyre::eyre!("Local config missing"))?;

            // Expand ~ in sync_dir path
            let sync_dir = if local_config.sync_dir.starts_with("~/") {
                let home_dir = dirs::home_dir()
                    .ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
                home_dir.join(&local_config.sync_dir[2..])
            } else {
                std::path::PathBuf::from(local_config.sync_dir)
            };

            Box::new(LocalProvider::new(sync_dir)?)
        }
        "azure" => {
            let azure_config = config
                .azure
                .ok_or_else(|| color_eyre::eyre::eyre!("Azure config missing"))?;

            if azure_config.connection_string.contains("REPLACE_WITH") {
                return Err(color_eyre::eyre::eyre!(
                    "Azure connection string not configured. Please update ~/.devlog/config.toml"
                ));
            }

            Box::new(AzureProvider::new(
                &azure_config.connection_string,
                &azure_config.container_name,
            )?)
        }
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "Unknown provider: {}",
                config.provider
            ));
        }
    };

    // Use ~/.devlog/entries as the local entries directory
    let home_dir =
        dirs::home_dir().ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
    let entries_dir = home_dir.join(".devlog").join("entries");

    Ok(SyncEngine::new(provider, entries_dir))
}
