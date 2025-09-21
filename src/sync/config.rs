use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for sync feature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    pub provider: String, // "local" or "azure"
    pub local: Option<LocalConfig>,
    pub azure: Option<AzureConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalConfig {
    pub sync_dir: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureConfig {
    pub connection_string: String,
    pub container_name: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            local: Some(LocalConfig {
                sync_dir: "~/.devlog/sync".to_string(),
            }),
            azure: None,
        }
    }
}

/// Simple config manager for MVP
pub struct ConfigManager {
    pub sync_config: Option<SyncConfig>,
}

impl ConfigManager {
    /// Load config from .devlog/config.toml if it exists
    pub fn load() -> Result<Self> {
        // Try home directory first
        if let Some(home_dir) = dirs::home_dir() {
            let config_path = home_dir.join(".devlog").join("config.toml");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                let config: SyncConfig =
                    toml::from_str(&content).map_err(|e| eyre!("Failed to parse config: {}", e))?;
                return Ok(ConfigManager {
                    sync_config: Some(config),
                });
            }
        }

        // Fallback to local directory
        let config_path = PathBuf::from(".devlog/config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: SyncConfig =
                toml::from_str(&content).map_err(|e| eyre!("Failed to parse config: {}", e))?;
            return Ok(ConfigManager {
                sync_config: Some(config),
            });
        }

        Ok(ConfigManager { sync_config: None })
    }

    /// Create config for specific provider
    pub fn create_config_for_provider(provider: &str) -> Result<()> {
        let config_dir = if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".devlog")
        } else {
            PathBuf::from(".devlog")
        };

        std::fs::create_dir_all(&config_dir)?;

        let config = match provider {
            "local" => SyncConfig::default(),
            "azure" => SyncConfig {
                provider: "azure".to_string(),
                local: None,
                azure: Some(AzureConfig {
                    connection_string: "REPLACE_WITH_YOUR_AZURE_CONNECTION_STRING".to_string(),
                    container_name: "devlog-entries".to_string(),
                }),
            },
            _ => return Err(eyre!("Unknown provider: {}", provider)),
        };

        let content = toml::to_string_pretty(&config)?;

        let config_path = config_dir.join("config.toml");
        std::fs::write(&config_path, content)?;
        println!("Created {} config at {}", provider, config_path.display());

        if provider == "azure" {
            println!("\n📝 Next steps:");
            println!("1. Replace REPLACE_WITH_YOUR_AZURE_CONNECTION_STRING with your actual connection string");
            println!("2. Update container_name if needed (default: devlog-entries)");
            println!("3. Run 'devlog sync status' to verify configuration");
        }

        Ok(())
    }
}
