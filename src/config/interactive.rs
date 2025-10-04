use super::{
    defaults::{validate_base_path, validate_container_name, DEFAULT_AZURE_CONTAINER, DEFAULT_BASE_PATH},
    providers::azure::AzureConfig,
    Config, StorageConfig, StorageProvider,
};
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
    
    // Configure storage
    config.storage = configure_storage(&config.storage)?;
    
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

pub fn configure_storage_provider(provider: &str) -> Result<()> {
    let mut config = Config::load_or_create_default()?;
    
    match provider.to_lowercase().as_str() {
        "azure" => {
            println!("{}", style("Configuring Azure Blob Storage...").bold());
            config.storage = configure_azure_storage()?;
        }
        "local" => {
            config.storage = StorageConfig {
                provider: StorageProvider::Local,
                azure: None,
            };
            println!("{}", style("✓ Storage provider set to local").green());
        }
        _ => {
            return Err(color_eyre::eyre::eyre!(
                "Unsupported storage provider: {}. Supported providers: local, azure",
                provider
            ));
        }
    }
    
    config.save()?;
    println!("{}", style("✓ Storage configuration updated!").green());
    Ok(())
}

pub fn show_config() -> Result<()> {
    let config = Config::load_or_create_default()?;
    
    println!("{}", style("DevLog Configuration:").bold().underlined());
    println!("  {}: {}", style("Base path").bold(), config.base_path.display());
    
    match config.storage.provider {
        StorageProvider::Local => {
            println!("  {}: {}", style("Storage").bold(), "Local filesystem");
        }
        StorageProvider::Azure => {
            println!("  {}: {}", style("Storage").bold(), "Azure Blob Storage");
            if let Some(azure_config) = &config.storage.azure {
                println!("    {}: {}", style("Container").dim(), azure_config.container_name);
                println!("    {}: {}", style("Status").dim(), style("✓ Configured").green());
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
    
    // Try to determine the user's preferred editor
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                "notepad".to_string()
            } else {
                "nano".to_string()
            }
        });
    
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

fn configure_storage(current_storage: &StorageConfig) -> Result<StorageConfig> {
    let enable_cloud = Confirm::new()
        .with_prompt("Enable cloud storage?")
        .default(matches!(current_storage.provider, StorageProvider::Azure))
        .interact()?;
    
    if !enable_cloud {
        println!("{} Storage set to: {}", style("✓").green(), style("Local filesystem").cyan());
        return Ok(StorageConfig {
            provider: StorageProvider::Local,
            azure: None,
        });
    }
    
    // For now, only Azure is supported
    let providers = vec!["Azure Blob Storage"];
    let selection = Select::new()
        .with_prompt("Select cloud provider")
        .items(&providers)
        .default(0)
        .interact()?;
    
    match selection {
        0 => configure_azure_storage(),
        _ => unreachable!(),
    }
}

fn configure_azure_storage() -> Result<StorageConfig> {
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
                
                println!("{} Azure storage configured", style("✓").green());
                
                return Ok(StorageConfig {
                    provider: StorageProvider::Azure,
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