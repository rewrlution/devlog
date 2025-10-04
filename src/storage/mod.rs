use color_eyre::eyre::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub mod entry;
mod platform;

use platform::{get_xdg_directory, XdgDirectoryType};

#[derive(Clone)]
pub struct Storage {
    /// Path for application data (entries, events, etc.) - NOT including the 'entries' subdirectory
    data_path: PathBuf,
    /// Path for configuration files
    config_path: PathBuf,
    /// Path for cache files
    cache_path: PathBuf,
    /// Path for state files (logs, history)
    state_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance using XDG directories
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_dir()?;
        let data_path = Self::get_data_dir()?;
        let cache_path = Self::get_cache_dir()?;
        let state_path = Self::get_state_dir()?;

        Ok(Self {
            config_path,
            data_path,
            cache_path,
            state_path,
        })
    }

    /// Create a new Storage instance for testing with custom base directory
    #[cfg(test)]
    pub fn new_with_base_dir(base_dir: &Path) -> Result<Self> {
        let config_path = base_dir.join("config");
        let data_path = base_dir.join("data"); // No 'entries' subdirectory here
        let cache_path = base_dir.join("cache");
        let state_path = base_dir.join("state");

        // Create all necessary directories
        for dir in [&config_path, &data_path, &cache_path, &state_path] {
            fs::create_dir_all(dir)
                .wrap_err_with(|| format!("Failed to create directory: {}", dir.display()))?;
        }

        Ok(Self {
            config_path,
            data_path,
            cache_path,
            state_path,
        })
    }

    /// Get XDG config directory with platform-specific fallbacks
    fn get_config_dir() -> Result<PathBuf> {
        get_xdg_directory(XdgDirectoryType::Config, "devlog", dirs::config_dir)
    }

    /// Get XDG data directory with platform-specific fallbacks
    fn get_data_dir() -> Result<PathBuf> {
        get_xdg_directory(XdgDirectoryType::Data, "devlog", dirs::data_dir)
    }

    /// Get XDG cache directory with platform-specific fallbacks
    fn get_cache_dir() -> Result<PathBuf> {
        get_xdg_directory(XdgDirectoryType::Cache, "devlog", dirs::cache_dir)
    }

    /// Get XDG state directory with platform-specific fallbacks
    fn get_state_dir() -> Result<PathBuf> {
        get_xdg_directory(XdgDirectoryType::State, "devlog", dirs::state_dir)
    }

    /// Get the data directory path (where entries are stored)
    pub fn data_path(&self) -> &Path {
        &self.data_path
    }

    /// Get the config directory path
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Get the cache directory path
    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }

    /// Get the state directory path
    pub fn state_path(&self) -> &Path {
        &self.state_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a test storage instance in a temporary directory
    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage =
            Storage::new_with_base_dir(temp_dir.path()).expect("Failed to create storage");
        (storage, temp_dir)
    }

    #[test]
    fn test_xdg_directory_structure() {
        let (storage, temp_dir) = create_test_storage();

        // Verify that all XDG directories are created
        assert!(temp_dir.path().join("config").exists());
        assert!(temp_dir.path().join("data").exists()); // Data dir, not data/entries
        assert!(temp_dir.path().join("cache").exists());
        assert!(temp_dir.path().join("state").exists());

        // Verify that the storage has the correct paths
        assert_eq!(storage.config_path(), temp_dir.path().join("config"));
        assert_eq!(storage.data_path(), temp_dir.path().join("data")); // Data dir, not data/entries
        assert_eq!(storage.cache_path(), temp_dir.path().join("cache"));
        assert_eq!(storage.state_path(), temp_dir.path().join("state"));
    }

    #[test]
    fn test_entry_operations() {
        use crate::models::entry::Entry;
        let (storage, temp_dir) = create_test_storage();

        // Test that entry operations create the entries directory
        let entry = Entry::new("20250920".to_string(), "Test content".to_string());
        storage.save_entry(&entry).expect("Failed to save entry");

        // Verify that entries directory is created within data directory
        assert!(temp_dir.path().join("data").join("entries").exists());
        assert!(temp_dir
            .path()
            .join("data")
            .join("entries")
            .join("20250920.md")
            .exists());

        // Test loading the entry
        let loaded_entry = storage
            .load_entry("20250920")
            .expect("Failed to load entry");
        assert_eq!(loaded_entry.id, "20250920");
        assert_eq!(loaded_entry.content, "Test content");
    }
}
