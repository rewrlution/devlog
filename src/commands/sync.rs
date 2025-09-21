use clap::Subcommand;
use color_eyre::Result;

use crate::sync::{config::ConfigManager, engine::{SyncEngine, LocalProvider}};

#[derive(Subcommand)]
pub enum SyncCommands {
    /// Initialize sync configuration
    Init,
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
        SyncCommands::Init => {
            ConfigManager::create_default()?;
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
                    if let Some(sync_dir) = &config.sync_dir {
                        println!("  Sync directory: {}", sync_dir);
                        let path = std::path::Path::new(sync_dir);
                        println!("  Remote exists: {}", path.exists());
                    }
                }
                None => {
                    println!("  No sync configuration found. Run 'devlog sync init' to get started.");
                }
            }
            Ok(())
        }
    }
}

async fn create_sync_engine() -> Result<SyncEngine> {
    let config_manager = ConfigManager::load()?;
    let config = config_manager.sync_config
        .unwrap_or_default();
    
    // MVP: Only support local provider
    let sync_dir = config.sync_dir.unwrap_or_else(|| "~/.devlog/sync".to_string());
    
    // Expand ~ in sync_dir path
    let sync_dir = if sync_dir.starts_with("~/") {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
        home_dir.join(&sync_dir[2..])
    } else {
        std::path::PathBuf::from(sync_dir)
    };
    
    let provider = Box::new(LocalProvider::new(sync_dir)?);
    
    // Use ~/.devlog/entries as the local entries directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;
    let entries_dir = home_dir.join(".devlog").join("entries");
    
    Ok(SyncEngine::new(provider, entries_dir))
}
