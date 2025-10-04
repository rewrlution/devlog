use std::path::PathBuf;

use color_eyre::eyre::{ContextCompat, Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
}

impl Config {
    /// Get the path to the configuration file (Should move to the storage)
    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
            .wrap_err("Could not find config directory")?;

        Ok(config_dir.join("devlog").join("config.toml"))
    }
}
