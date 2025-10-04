use crate::models::entry::Entry;
use crate::storage::Storage;
use chrono::Utc;
use color_eyre::eyre::{Context, Result};
use std::fs;
use walkdir::WalkDir;

impl Storage {
    /// Save an entry to disk
    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let entries_path = self.get_entries_path()?;
        let file_path = entries_path.join(format!("{}.md", entry.id));
        let content = self.serialize_entry(entry)?;

        fs::write(&file_path, content)
            .wrap_err_with(|| format!("Failed to save entry to {}", file_path.display()))?;
        Ok(())
    }

    /// Load an entry from disk
    pub fn load_entry(&self, id: &str) -> Result<Entry> {
        let entries_path = self.get_entries_path()?;
        let file_path = entries_path.join(format!("{}.md", id));
        let content = fs::read_to_string(&file_path)
            .wrap_err_with(|| format!("Failed to read entry from {}", file_path.display()))?;

        self.deserialize_entry(id, &content)
    }

    /// List all entries from disk
    pub fn list_entries(&self) -> Result<Vec<String>> {
        let entries_path = self.get_entries_path()?;
        let mut entries = Vec::new();

        let md_files = WalkDir::new(&entries_path)
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

    /// Get the entries directory path, creating it if it doesn't exist
    fn get_entries_path(&self) -> Result<std::path::PathBuf> {
        let entries_path = self.data_path.join("entries");
        
        // Create entries directory if it doesn't exist
        fs::create_dir_all(&entries_path).wrap_err_with(|| {
            format!(
                "Failed to create entries directory: {}",
                entries_path.display()
            )
        })?;

        Ok(entries_path)
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

    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = Storage::new_with_base_dir(temp_dir.path()).expect("Failed to create storage");
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
    fn test_list_entries() {
        let (storage, _temp_dir) = create_test_storage();

        // Create some test entries
        let entry1 = Entry::new("20250920".to_string(), "First entry".to_string());
        let entry2 = Entry::new("20250921".to_string(), "Second entry".to_string());
        let entry3 = Entry::new("20250919".to_string(), "Third entry".to_string());

        // Save entries
        storage.save_entry(&entry1).expect("Failed to save entry1");
        storage.save_entry(&entry2).expect("Failed to save entry2");
        storage.save_entry(&entry3).expect("Failed to save entry3");

        // List entries
        let entries = storage.list_entries().expect("Failed to list entries");

        // Should return 3 entries
        assert_eq!(entries.len(), 3);

        // Should be sorted by date (newest first - string comparison)
        assert_eq!(entries[0], "20250921"); // newest
        assert_eq!(entries[1], "20250920");
        assert_eq!(entries[2], "20250919"); // oldest
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
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
}
