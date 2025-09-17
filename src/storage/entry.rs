use crate::utils::date::format_entry_date;
use chrono::{Datelike, NaiveDate};
use color_eyre::{Result, eyre::Ok};
use std::path::PathBuf;

/// Represents a journal entry
#[derive(Debug, Clone)]
pub struct Entry {
    pub date: NaiveDate,
    pub path: PathBuf,
    pub content: String,
}

impl Entry {
    pub fn new(date: NaiveDate) -> Self {
        let path = Self::generate_file_path(&date);
        Self {
            date,
            path,
            content: String::new(),
        }
    }

    pub fn with_content(date: NaiveDate, content: String) -> Self {
        let path = Self::generate_file_path(&date);
        Self {
            date,
            path,
            content,
        }
    }

    pub fn load_from_file(date: NaiveDate) -> Result<Self> {
        let path = Self::generate_file_path(&date);
        let content = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };
        Ok(Self {
            date,
            path,
            content,
        })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, &self.content)?;
        Ok(())
    }

    fn generate_file_path(date: &NaiveDate) -> PathBuf {
        let year = date.year();
        let month = date.month();
        let date_str = format_entry_date(date);

        PathBuf::from("entries")
            .join(format!("{}", year))
            .join(format!("{:02}", month))
            .join(format!("{}.md", date_str))
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_entry_creation() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let entry = Entry::new(date);

        assert_eq!(entry.date, date);
        assert!(entry.path.ends_with("20250315.md"));
        assert!(entry.content.is_empty());
    }

    #[test]
    fn test_entry_with_content() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let content = "Today I learned Rust!".to_string();
        let entry = Entry::with_content(date, content.clone());

        assert_eq!(entry.content, content);
    }

    #[test]
    fn test_path_generation() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let entry = Entry::new(date);

        let path_str = entry.path.to_string_lossy();
        assert!(path_str.contains("entries"));
        assert!(path_str.contains("2025"));
        assert!(path_str.contains("03"));
        assert!(path_str.ends_with("20250315.md"));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
    }
}
