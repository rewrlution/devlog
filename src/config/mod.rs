pub mod defaults;
pub mod interactive;
pub mod providers;

use color_eyre::eyre::{Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub base_path: PathBuf,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure: Option<providers::azure::AzureConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws: Option<providers::aws::AwsConfig>,
}

impl Config {
    /// Load configuration from file, or create default if not exists
    pub fn load_or_create_default() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Load configuration from file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .wrap_err_with(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .wrap_err("Failed to parse config file")?;
            
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .wrap_err("Failed to serialize config")?;
            
        fs::write(&config_path, content)
            .wrap_err_with(|| format!("Failed to write config file: {}", config_path.display()))?;
            
        Ok(())
    }

    /// Get the path to the configuration file
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
            .wrap_err("Could not find config directory")?;
            
        Ok(config_dir.join("devlog").join("config.toml"))
    }

    /// Get the expanded base path (resolves ~)
    pub fn expanded_base_path(&self) -> Result<PathBuf> {
        let path_str = self.base_path.to_string_lossy();
        if path_str.starts_with("~/") {
            let home_dir = dirs::home_dir()
                .wrap_err("Could not find home directory")?;
            Ok(home_dir.join(&path_str[2..]))
        } else if path_str == "~" {
            dirs::home_dir()
                .wrap_err("Could not find home directory")
        } else {
            Ok(self.base_path.clone())
        }
    }

    /// Reset to default configuration
    pub fn reset_to_default() -> Result<Self> {
        let config = Self::default();
        config.save()?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("~/.devlog"),
            sync: SyncConfig {
                enabled: false,
                azure: None,
                aws: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        
        assert!(toml_str.contains("base_path"));
        assert!(toml_str.contains("enabled = false"));
        assert!(toml_str.contains("[sync]"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
        base_path = "~/.devlog"
        
        [sync]
        enabled = false
        "#;
        
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.base_path, PathBuf::from("~/.devlog"));
        assert!(!config.sync.enabled);
    }

    #[test]
    fn test_path_expansion() {
        let config = Config {
            base_path: PathBuf::from("~/test"),
            ..Default::default()
        };
        
        let expanded = config.expanded_base_path().unwrap();
        assert!(!expanded.to_string_lossy().starts_with("~"));
    }
}