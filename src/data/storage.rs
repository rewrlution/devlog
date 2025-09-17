use crate::data::entry::Entry;
use crate::utils::date::format_entry_date;
use chrono::{Datelike, NaiveDate};
use color_eyre::{Result, eyre::Ok};
use std::path::{Path, PathBuf};

/// Handels file system operations for journal entries
pub struct Storage {
    base_dir: PathBuf,
}

impl Storage {
    /// Creates a storage instance with custom base directory (absolute path)
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Creates a storage instance with default base directory in user's home
    pub fn default() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not find home directory"))?;

        let base_dir = home_dir.join(".devlog").join("entries");
        Ok(Self::new(base_dir))
    }

    /// Load an entry for a specific date
    pub fn load_entry(&self, date: NaiveDate) -> Result<Entry> {
        let path = self.generate_file_path(&date);
        let content = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };
        Ok(Entry::with_content(date, content))
    }

    /// Save the entry to the storage layer
    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let path = self.generate_file_path(&entry.date);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &entry.content)?;
        Ok(())
    }

    /// Get the base directory path (for directory scanning)
    pub fn get_base_dir(&self) -> &Path {
        &self.base_dir
    }

    fn generate_file_path(&self, date: &NaiveDate) -> PathBuf {
        let year = date.year();
        let month = date.month();
        let date_str = format_entry_date(date);

        self.base_dir
            .join(format!("{}", year))
            .join(format!("{:02}", month))
            .join(format!("{}.md", date_str))
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());
        assert_eq!(storage.base_dir, temp_dir.path());
    }

    #[test]
    fn test_path_generation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let path = storage.generate_file_path(&date);

        let path_str = path.to_string_lossy();
        assert!(path_str.contains("2025"));
        assert!(path_str.contains("03"));
        assert!(path_str.ends_with("20250315.md"));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let content = "Test content".to_string();

        let entry = Entry::with_content(date, content.clone());
        storage.save_entry(&entry).unwrap();

        let loaded_entry = storage.load_entry(date).unwrap();
        assert_eq!(loaded_entry.content, content);
        assert_eq!(loaded_entry.date, date);
    }
}
