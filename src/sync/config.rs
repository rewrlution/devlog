use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use color_eyre::{Result, eyre::eyre};

/// Configuration for sync feature
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    pub provider: String, // MVP: Just store as string, future: enum
    pub sync_dir: Option<String>, // MVP: Local directory for testing
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: "local".to_string(),
            sync_dir: Some("~/.devlog/sync".to_string()),
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
                let config: SyncConfig = toml::from_str(&content)
                    .map_err(|e| eyre!("Failed to parse config: {}", e))?;
                return Ok(ConfigManager {
                    sync_config: Some(config),
                });
            }
        }
        
        // Fallback to local directory
        let config_path = PathBuf::from(".devlog/config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: SyncConfig = toml::from_str(&content)
                .map_err(|e| eyre!("Failed to parse config: {}", e))?;
            return Ok(ConfigManager {
                sync_config: Some(config),
            });
        }
        
        Ok(ConfigManager { sync_config: None })
    }
    
    /// Create a default config file
    pub fn create_default() -> Result<()> {
        let config_dir = if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(".devlog")
        } else {
            PathBuf::from(".devlog")
        };
        
        std::fs::create_dir_all(&config_dir)?;
        
        let config = SyncConfig::default();
        let content = toml::to_string_pretty(&config)?;
        
        let config_path = config_dir.join("config.toml");
        std::fs::write(&config_path, content)?;
        println!("Created default config at {}", config_path.display());
        
        Ok(())
    }
}
