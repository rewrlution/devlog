use super::{
    defaults::{validate_base_path, validate_container_name, DEFAULT_AZURE_CONTAINER, DEFAULT_BASE_PATH},
    providers::azure::AzureConfig,
    Config, SyncConfig, SyncProvider,
};
use crate::utils::editor::find_available_editor;
use color_eyre::eyre::{Context, Result};
use console::style;
use dialoguer::{Confirm, Input, Select};
use std::path::PathBuf;

pub fn run_interactive_config() -> Result<()> {
    println!("{}", style("Welcome to DevLog configuration!").bold().green());
    println!();

    // Load existing config or create default
    let mut config = Config::load_or_create_default()
        .wrap_err("Failed to load existing configuration")?;

    // Configure base path
    config.base_path = configure_base_path(&config.base_path)?;
    
    // Configure sync
    config.sync = configure_sync(&config.sync)?;
    
    // Save configuration
    config.save().wrap_err("Failed to save configuration")?;
    
    println!();
    println!("{}", style("✓ Configuration saved successfully!").bold().green());
    println!(
        "Config file location: {}",
        style(Config::config_file_path()?.display()).dim()
    );
    println!();
    println!("Run 'devlog config --show' to view your current configuration.");
    
    Ok(())
}

pub fn configure_path() -> Result<()> {
    let mut config = Config::load_or_create_default()?;
    
    println!("Current base path: {}", style(&config.base_path.display()).cyan());
    
    let new_path = configure_base_path(&config.base_path)?;
    config.base_path = new_path;
    
    config.save()?;
    
    println!("{}", style("✓ Base path updated successfully!").green());
    Ok(())
}

pub fn configure_sync_provider(provider: Option<&str>) -> Result<()> {
    let mut config = Config::load_or_create_default()?;
    
    match provider {
        Some(provider_name) => {
            match provider_name.to_lowercase().as_str() {
                "azure" => {
                    println!("{}", style("Configuring Azure Blob Storage...").bold());
                    config.sync = Some(configure_azure_sync()?);
                }
                "aws" => {
                    return Err(color_eyre::eyre::eyre!(
                        "AWS sync is not yet supported. Currently supported: azure"
                    ));
                }
                "gcp" => {
                    return Err(color_eyre::eyre::eyre!(
                        "Google Cloud sync is not yet supported. Currently supported: azure"
                    ));
                }
                _ => {
                    return Err(color_eyre::eyre::eyre!(
                        "Unsupported sync provider: {}. Supported providers: azure (aws, gcp coming soon)",
                        provider_name
                    ));
                }
            }
        }
        None => {
            // Interactive mode - let user choose from available providers
            config.sync = configure_sync_interactive()?;
        }
    }
    
    config.save()?;
    println!("{}", style("✓ Sync configuration updated!").green());
    Ok(())
}

pub fn show_config() -> Result<()> {
    let config = Config::load_or_create_default()?;
    
    println!("{}", style("DevLog Configuration:").bold().underlined());
    println!("  {}: {}", style("Base path").bold(), config.base_path.display());
    
    match &config.sync {
        None => {
            println!("  {}: {}", style("Cloud Sync").bold(), style("Disabled").dim());
        }
        Some(sync_config) => {
            match sync_config.provider {
                SyncProvider::Azure => {
                    println!("  {}: {}", style("Cloud Sync").bold(), "Azure Blob Storage");
                    if let Some(azure_config) = &sync_config.azure {
                        println!("    {}: {}", style("Container").dim(), azure_config.container_name);
                        println!("    {}: {}", style("Status").dim(), style("✓ Configured").green());
                    }
                }
                SyncProvider::Aws => {
                    println!("  {}: {}", style("Cloud Sync").bold(), "AWS S3 (not yet supported)");
                }
                SyncProvider::Gcp => {
                    println!("  {}: {}", style("Cloud Sync").bold(), "Google Cloud Storage (not yet supported)");
                }
            }
        }
    }
    
    println!();
    println!(
        "Config file: {}",
        style(Config::config_file_path()?.display()).dim()
    );
    
    Ok(())
}

pub fn reset_config() -> Result<()> {
    let confirm = Confirm::new()
        .with_prompt("Are you sure you want to reset configuration to defaults?")
        .default(false)
        .interact()?;
        
    if confirm {
        Config::reset_to_default()?;
        println!("{}", style("✓ Configuration reset to defaults").green());
    } else {
        println!("Configuration reset cancelled.");
    }
    
    Ok(())
}

pub fn edit_config() -> Result<()> {
    let config_path = Config::config_file_path()?;
    
    // Ensure config file exists
    if !config_path.exists() {
        let config = Config::default();
        config.save()?;
    }
    
    // Use the same editor finding strategy as the main editor utility
    let editor = find_available_editor();
    
    println!(
        "Opening config file with {}: {}",
        style(&editor).bold(),
        style(config_path.display()).dim()
    );
    
    let status = std::process::Command::new(&editor)
        .arg(&config_path)
        .status()
        .wrap_err_with(|| format!("Failed to launch editor: {}", editor))?;
    
    if !status.success() {
        return Err(color_eyre::eyre::eyre!("Editor exited with error"));
    }
    
    // Validate the edited config
    match Config::load_from_file(&config_path) {
        Ok(_) => println!("{}", style("✓ Configuration file is valid").green()),
        Err(e) => {
            println!("{}", style("⚠ Configuration file has errors:").red());
            println!("  {}", e);
            println!("Please fix the errors and try again.");
        }
    }
    
    Ok(())
}

fn configure_base_path(current_path: &PathBuf) -> Result<PathBuf> {
    let current_display = if current_path == &PathBuf::from(DEFAULT_BASE_PATH) {
        DEFAULT_BASE_PATH.to_string()
    } else {
        current_path.display().to_string()
    };
    
    loop {
        let input: String = Input::new()
            .with_prompt("Enter base path for your dev logs")
            .default(current_display.clone())
            .interact_text()?;
        
        match validate_base_path(&input) {
            Ok(path) => {
                println!("{} Base path set to: {}", style("✓").green(), style(path.display()).cyan());
                return Ok(path);
            }
            Err(e) => {
                println!("{} {}", style("✗").red(), e);
                continue;
            }
        }
    }
}

fn configure_sync(current_sync: &Option<SyncConfig>) -> Result<Option<SyncConfig>> {
    let enable_cloud = Confirm::new()
        .with_prompt("Enable cloud sync?")
        .default(current_sync.is_some())
        .interact()?;
    
    if !enable_cloud {
        println!("{} Cloud sync: {}", style("✓").green(), style("Disabled").cyan());
        return Ok(None);
    }
    
    configure_sync_interactive()
}

fn configure_sync_interactive() -> Result<Option<SyncConfig>> {
    // Show available providers with support status
    let providers = vec![
        "Azure Blob Storage (supported)",
        "AWS S3 (coming soon)",
        "Google Cloud Storage (coming soon)"
    ];
    
    println!();
    println!("{}", style("Available cloud sync providers:").bold());
    
    let selection = Select::new()
        .with_prompt("Select cloud provider")
        .items(&providers)
        .default(0)
        .interact()?;
    
    match selection {
        0 => Ok(Some(configure_azure_sync()?)),
        1 => {
            println!("{}", style("AWS S3 support is coming soon!").yellow());
            Err(color_eyre::eyre::eyre!("AWS S3 is not yet supported"))
        }
        2 => {
            println!("{}", style("Google Cloud Storage support is coming soon!").yellow());
            Err(color_eyre::eyre::eyre!("Google Cloud Storage is not yet supported"))
        }
        _ => unreachable!(),
    }
}

fn configure_azure_sync() -> Result<SyncConfig> {
    println!();
    println!("{}", style("Azure Blob Storage Configuration:").bold());
    
    // Get connection string
    let connection_string: String = Input::new()
        .with_prompt("Connection string")
        .interact_text()?;
    
    if connection_string.trim().is_empty() {
        return Err(color_eyre::eyre::eyre!("Connection string cannot be empty"));
    }
    
    // Get container name
    loop {
        let container_name: String = Input::new()
            .with_prompt("Container name")
            .default(DEFAULT_AZURE_CONTAINER.to_string())
            .interact_text()?;
        
        match validate_container_name(&container_name) {
            Ok(name) => {
                let azure_config = AzureConfig::new(connection_string.trim().to_string(), name);
                
                // Validate the configuration
                if let Err(e) = azure_config.validate() {
                    println!("{} {}", style("✗").red(), e);
                    continue;
                }
                
                println!("{} Azure sync configured", style("✓").green());
                
                return Ok(SyncConfig {
                    provider: SyncProvider::Azure,
                    azure: Some(azure_config),
                });
            }
            Err(e) => {
                println!("{} {}", style("✗").red(), e);
                continue;
            }
        }
    }
}