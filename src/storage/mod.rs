use crate::models::entry::Entry;

use chrono::Utc;
use color_eyre::eyre::{Context, ContextCompat, Ok, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    /// Create a new Storage instance
    pub fn new(base_dir: Option<&Path>) -> Result<Self> {
        let base_path = match base_dir {
            Some(dir) => dir.join("entries"),
            None => {
                let home_dir = dirs::home_dir().wrap_err("Could not find home directory")?;
                home_dir.join(".devlog").join("entries")
            }
        };

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path).wrap_err_with(|| {
            format!(
                "Failed to create storage directory: {}",
                base_path.display()
            )
        })?;

        Ok(Self { base_path })
    }

    /// Save an entry to disk
    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let file_path = self.base_path.join(format!("{}.md", entry.id));
        let content = self.serialize_entry(entry)?;

        fs::write(&file_path, content)
            .wrap_err_with(|| format!("Failed to save entry to {}", file_path.display()))?;
        Ok(())
    }

    /// Load an entry from disk
    pub fn load_entry(&self, id: &str) -> Result<Entry> {
        let file_path = self.base_path.join(format!("{}.md", id));
        let content = fs::read_to_string(&file_path)
            .wrap_err_with(|| format!("Failed to read entry from {}", file_path.display()))?;

        self.deserialize_entry(id, &content)
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
}
