use crate::models::entry::Entry;

use chrono::Utc;
use color_eyre::eyre::{Context, Ok, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct Storage {
    /// Path for application data (entries, events, etc.)
    data_path: PathBuf,
    /// Path for configuration files
    config_path: PathBuf,
    /// Path for cache files
    cache_path: PathBuf,
    /// Path for state files (logs, history)
    state_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance
    pub fn new(base_dir: Option<&Path>) -> Result<Self> {
        let (config_path, data_path, cache_path, state_path) = match base_dir {
            Some(dir) => {
                // Test mode: use provided directory structure
                (
                    dir.join("config"),
                    dir.join("data").join("entries"),
                    dir.join("cache"),
                    dir.join("state"),
                )
            }
            None => {
                // Production: use XDG directories with fallbacks and migration
                let config = Self::get_config_dir()?;
                let data = Self::get_data_dir_with_migration()?;
                let cache = Self::get_cache_dir()?;
                let state = Self::get_state_dir()?;
                (config, data, cache, state)
            }
        };

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
        dirs::config_dir()
            .or_else(|| {
                // Manual XDG fallback for Linux/Unix
                if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
                    dirs::home_dir().map(|home| home.join(".config"))
                } else if cfg!(target_os = "macos") {
                    dirs::home_dir().map(|home| home.join("Library").join("Application Support"))
                } else if cfg!(target_os = "windows") {
                    std::env::var("APPDATA")
                        .ok()
                        .map(PathBuf::from)
                        .or_else(|| {
                            dirs::home_dir().map(|home| home.join("AppData").join("Roaming"))
                        })
                } else {
                    dirs::home_dir().map(|home| home.join(".config"))
                }
            })
            .map(|dir| dir.join("devlog"))
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))
    }

    /// Get XDG data directory with migration from legacy ~/.devlog
    fn get_data_dir_with_migration() -> Result<PathBuf> {
        let xdg_data_dir = Self::get_data_dir()?;
        let legacy_dir = dirs::home_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?
            .join(".devlog")
            .join("entries");

        // If legacy directory exists and has entries, use it and warn user
        if legacy_dir.exists() && Self::directory_has_entries(&legacy_dir)? {
            eprintln!(
                "Warning: Using legacy data directory: {}",
                legacy_dir.display()
            );
            eprintln!(
                "Consider migrating to XDG-compliant directory: {}",
                xdg_data_dir.display()
            );
            eprintln!("Run 'devlog config migrate' to migrate your data");
            Ok(legacy_dir)
        } else {
            Ok(xdg_data_dir)
        }
    }

    /// Get XDG data directory with platform-specific fallbacks
    fn get_data_dir() -> Result<PathBuf> {
        dirs::data_dir()
            .or_else(|| {
                // Manual XDG fallback
                if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
                    dirs::home_dir().map(|home| home.join(".local").join("share"))
                } else if cfg!(target_os = "macos") {
                    dirs::home_dir().map(|home| home.join("Library").join("Application Support"))
                } else if cfg!(target_os = "windows") {
                    std::env::var("APPDATA")
                        .ok()
                        .map(PathBuf::from)
                        .or_else(|| {
                            dirs::home_dir().map(|home| home.join("AppData").join("Roaming"))
                        })
                } else {
                    dirs::home_dir().map(|home| home.join(".local").join("share"))
                }
            })
            .map(|dir| dir.join("devlog").join("entries"))
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine data directory"))
    }

    /// Get XDG cache directory with platform-specific fallbacks
    fn get_cache_dir() -> Result<PathBuf> {
        dirs::cache_dir()
            .or_else(|| {
                if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
                    dirs::home_dir().map(|home| home.join(".cache"))
                } else if cfg!(target_os = "macos") {
                    dirs::home_dir().map(|home| home.join("Library").join("Caches"))
                } else if cfg!(target_os = "windows") {
                    std::env::var("LOCALAPPDATA")
                        .ok()
                        .map(PathBuf::from)
                        .or_else(|| dirs::home_dir().map(|home| home.join("AppData").join("Local")))
                } else {
                    dirs::home_dir().map(|home| home.join(".cache"))
                }
            })
            .map(|dir| dir.join("devlog"))
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine cache directory"))
    }

    /// Get XDG state directory with platform-specific fallbacks
    fn get_state_dir() -> Result<PathBuf> {
        dirs::state_dir()
            .or_else(|| {
                if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
                    dirs::home_dir().map(|home| home.join(".local").join("state"))
                } else {
                    // macOS and Windows fall back to data directory
                    Self::get_data_dir()
                        .ok()
                        .map(|data_dir| data_dir.parent().unwrap().to_path_buf())
                }
            })
            .map(|dir| dir.join("devlog"))
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine state directory"))
    }

    /// Check if a directory contains any .md entry files
    fn directory_has_entries(dir: &Path) -> Result<bool> {
        if !dir.exists() {
            return Ok(false);
        }

        let entries = WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .take(1); // Just check if at least one exists

        Ok(entries.count() > 0)
    }

    /// Save an entry to disk
    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let file_path = self.data_path.join(format!("{}.md", entry.id));
        let content = self.serialize_entry(entry)?;

        fs::write(&file_path, content)
            .wrap_err_with(|| format!("Failed to save entry to {}", file_path.display()))?;
        Ok(())
    }

    /// Load an entry from disk
    pub fn load_entry(&self, id: &str) -> Result<Entry> {
        let file_path = self.data_path.join(format!("{}.md", id));
        let content = fs::read_to_string(&file_path)
            .wrap_err_with(|| format!("Failed to read entry from {}", file_path.display()))?;

        self.deserialize_entry(id, &content)
    }

    /// List all entries from disk
    pub fn list_entries(&self) -> Result<Vec<String>> {
        let mut entries = Vec::new();

        let md_files = WalkDir::new(&self.data_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"));

        for entry in md_files {
            if let Some(stem) = entry.path().file_stem() {
                if let Some(id) = stem.to_str() {
                    entries.push(id.to_string());
                }
            }
        }

        // Sort by date (newest first)
        entries.sort_by(|a, b| b.cmp(a));
        Ok(entries)
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

    /// Serialize entry to markdown with YAML frontmatter
    fn serialize_entry(&self, entry: &Entry) -> Result<String> {
        let frontmatter = format!(
            r#"---
id: {}
created_at: {}
updated_at: {}
---

{}"#,
            entry.id, entry.created_at, entry.updated_at, entry.content
        );
        Ok(frontmatter)
    }

    /// Deserialize entry from markdown with YAML frontmatter
    fn deserialize_entry(&self, id: &str, content: &str) -> Result<Entry> {
        let now = Utc::now();

        // Simple frontmatter parsing
        if content.starts_with("---\n") {
            let parts: Vec<&str> = content.split("---").collect();
            if parts.len() >= 3 {
                let yaml_content = parts[1];
                let md_content = parts[2].trim_start().to_string();

                // Parse YAML frontmatter
                let frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_content)
                    .wrap_err("Failed to parse YAML frontmatter")?;

                let created_at = frontmatter["created_at"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(now);

                let updated_at = frontmatter["updated_at"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(now);

                return Ok(Entry {
                    id: id.to_string(),
                    created_at,
                    updated_at,
                    content: md_content,
                });
            }
        }

        // Fallback: treat entire content as markdown
        Ok(Entry {
            id: id.to_string(),
            created_at: now,
            updated_at: now,
            content: content.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Create a test storage instance in a temporary directory
    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = Storage::new(Some(temp_dir.path())).expect("Failed to create storage");
        (storage, temp_dir)
    }

    #[test]
    fn test_save_and_load_entry() {
        let (storage, _temp_dir) = create_test_storage();

        let id = "20250920".to_string();
        let content = "#Test entry\n\nThis is a test.".to_string();
        let entry = Entry::new(id, content);

        // Save entry
        storage.save_entry(&entry).expect("Failed to save entry");

        // Load entry
        let loaded_entry = storage.load_entry(&entry.id).expect("Failed to load entry");

        assert_eq!(loaded_entry.id, entry.id);
        assert_eq!(loaded_entry.content, entry.content);
    }

    #[test]
    fn test_load_nonexistent_entry() {
        let (storage, _temp_dir) = create_test_storage();

        let result = storage.load_entry("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_entry() {
        let (storage, temp_dir) = create_test_storage();
        let entry = Entry::new("20250920".to_string(), "Content".to_string());
        storage
            .save_entry(&entry)
            .expect("Failed to save the entry");

        let path = temp_dir
            .path()
            .join("data")
            .join("entries")
            .join("20250920.md");
        assert!(path.exists());
    }

    #[test]
    fn test_list_entries() {
        let (storage, temp_dir) = create_test_storage();

        // Create some test entries
        let entry1 = Entry::new("20250920".to_string(), "First entry".to_string());
        let entry2 = Entry::new("20250921".to_string(), "Second entry".to_string());
        let entry3 = Entry::new("20250919".to_string(), "Third entry".to_string());

        // Save entries
        storage.save_entry(&entry1).expect("Failed to save entry1");
        storage.save_entry(&entry2).expect("Failed to save entry2");
        storage.save_entry(&entry3).expect("Failed to save entry3");

        // Create a non-markdown file that should be ignored
        let entries_dir = temp_dir.path().join("data").join("entries");
        std::fs::write(entries_dir.join("readme.txt"), "This should be ignored").unwrap();

        // List entries
        let entries = storage.list_entries().expect("Failed to list entries");

        // Should return 3 entries (ignoring the .txt file)
        assert_eq!(entries.len(), 3);

        // Should be sorted by date (newest first - string comparison)
        assert_eq!(entries[0], "20250921"); // newest
        assert_eq!(entries[1], "20250920");
        assert_eq!(entries[2], "20250919"); // oldest
    }

    #[test]
    fn test_deserialize_entry_without_frontmatter() {
        let (storage, _temp_dir) = create_test_storage();
        let content = "#Simple markdown\n\nContent";
        let entry = storage
            .deserialize_entry("20250920", content)
            .expect("Failed to deserialize the entry");

        assert_eq!(entry.id, "20250920");
        assert_eq!(entry.content, content);
    }

    #[test]
    fn test_deserialize_entry_with_invalid_frontmatter() {
        let (storage, _temp_dir) = create_test_storage();

        let content = "---\ninvalid: yaml: content:\n---\n\n# Content here";
        let result = storage.deserialize_entry("20250920", content);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_entry_format() {
        let (storage, _temp_dir) = create_test_storage();
        let entry = Entry::new("20250920".to_string(), "# Test\n\nContent".to_string());
        let serialized = storage
            .serialize_entry(&entry)
            .expect("Failed to serialize the entry");

        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("id: 20250920"));
        assert!(serialized.contains("created_at:"));
        assert!(serialized.contains("updated_at"));
        assert!(serialized.contains("# Test\n\nContent"));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let (storage, _temp_dir) = create_test_storage();

        let original_entry = Entry::new(
            "20250920".to_string(),
            "# Original\n\nSome content.".to_string(),
        );

        // Serialize then deserialize
        let serialized = storage
            .serialize_entry(&original_entry)
            .expect("Failed to serialize the entry.");
        let deserialized = storage
            .deserialize_entry(&original_entry.id, &serialized)
            .expect("Failed to deserialize the entry.");

        assert_eq!(deserialized.id, original_entry.id);
        assert_eq!(deserialized.content, original_entry.content);
    }

    #[test]
    fn test_xdg_directory_structure() {
        let (storage, temp_dir) = create_test_storage();

        // Verify that all XDG directories are created
        assert!(temp_dir.path().join("config").exists());
        assert!(temp_dir.path().join("data").join("entries").exists());
        assert!(temp_dir.path().join("cache").exists());
        assert!(temp_dir.path().join("state").exists());

        // Verify that the storage has the correct paths
        assert_eq!(storage.config_path(), temp_dir.path().join("config"));
        assert_eq!(
            storage.data_path(),
            temp_dir.path().join("data").join("entries")
        );
        assert_eq!(storage.cache_path(), temp_dir.path().join("cache"));
        assert_eq!(storage.state_path(), temp_dir.path().join("state"));
    }

    #[test]
    fn test_directory_has_entries() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let entries_dir = temp_dir.path().join("entries");

        // Empty directory should return false
        fs::create_dir_all(&entries_dir).unwrap();
        assert!(!Storage::directory_has_entries(&entries_dir).unwrap());

        // Directory with .md file should return true
        fs::write(entries_dir.join("20250920.md"), "content").unwrap();
        assert!(Storage::directory_has_entries(&entries_dir).unwrap());

        // Directory with non-.md file should return false
        let other_dir = temp_dir.path().join("other");
        fs::create_dir_all(&other_dir).unwrap();
        fs::write(other_dir.join("readme.txt"), "content").unwrap();
        assert!(!Storage::directory_has_entries(&other_dir).unwrap());
    }
}
