//! Configuration management for DevLog
//!
//! This module handles reading and writing configuration files,

use anyhow::{Context, Ok, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevLogConfig {
    pub remote: RemoteConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoteConfig {
    pub provider: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncConfig {
    // Rust's type system prevents us from accidentally comparing `DateTime<Local>` to `DateTime<Utc>`
    // Therefore, we can still use `Local` for things like `entry ID`
    // and we can use `Utc` for features like syncing to prevent timezone-related sync issues.
    pub last_push_timestamp: Option<DateTime<Utc>>,
}

impl Default for DevLogConfig {
    fn default() -> Self {
        Self {
            remote: RemoteConfig {
                provider: "azure".to_string(),
                url: String::new(),
            },
            sync: SyncConfig {
                last_push_timestamp: None,
            },
        }
    }
}

impl DevLogConfig {
    /// Get the default config filepath: ~/.devlog/config.toml
    pub fn config_filepath() -> Result<PathBuf> {
        // Using `anyhow` makes managing errors very ergonomically!
        // Let's rewrite the <dyn std::error::Error> with context
        // TODO
        let home_dir = dirs::home_dir().context("Unable to determine home directory")?;
        let devlog_dir = home_dir.join(".devlog");
        let config_path = devlog_dir.join("config.toml");

        Ok(config_path)
    }

    /// Load configuration from file. Create default if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_filepath()?;

        if !config_path.exists() {
            // Create default config and save it
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: DevLogConfig = toml::from_str(&config_content)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_filepath()?;

        // Ensure the .devlog directory exists
        // .context() vs .with_context()
        // .context() for static message, .with_content() for dynamic message
        // .with_context() uses a `closure` and that's only executed if an error occurs.
        // This is more performant
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let config_content =
            toml::to_string_pretty(self).context("Failed to serialize configuration")?;

        std::fs::write(&config_path, config_content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// Update the last push timestamp and save
    pub fn update_last_push_timestamp(&mut self) -> Result<()> {
        self.sync.last_push_timestamp = Some(Utc::now());
        self.save()
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.remote.url.is_empty() {
            anyhow::bail!(
                "Remote URL is not configured. Please set the URL in ~/.devlog/config.toml"
            );
        }

        if !self.remote.url.starts_with("https://") {
            anyhow::bail!("Remote URL must use HTTPS protocol");
        }

        if self.remote.provider != "azure" {
            anyhow::bail!("Only 'azure' provider is currently supported")
        }

        Ok(())
    }
}
