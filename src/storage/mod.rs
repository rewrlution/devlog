pub mod entry;

use crate::storage::entry::Entry;
use chrono::Utc;
use color_eyre::{eyre::Context, eyre::ContextCompat, Result};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().wrap_err("Could not find home directory")?;
        let base_path = home.join(".devlog").join("entries");

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path).wrap_err("Failed to create devlog directory")?;

        Ok(Self { base_path })
    }

    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let file_path = self.base_path.join(format!("{}.md", entry.id));
        let content = self.serialize_entry(entry)?;
        fs::write(file_path, content).wrap_err("Failed to write entry file")?;
        Ok(())
    }

    pub fn load_entry(&self, id: &str) -> Result<Entry> {
        let file_path = self.base_path.join(format!("{}.md", id));
        let content = fs::read_to_string(file_path).wrap_err("Failed to read entry file")?;
        self.deserialize_entry(id, &content)
    }

    pub fn list_entries(&self) -> Result<Vec<String>> {
        let mut entries = Vec::new();

        for entry in WalkDir::new(&self.base_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        {
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

    fn serialize_entry(&self, entry: &Entry) -> Result<String> {
        let frontmatter = format!(
            "---\ncreated_at: \"{}\"\nupdated_at: \"{}\"\n---\n\n{}",
            entry.created_at.to_rfc3339(),
            entry.updated_at.to_rfc3339(),
            entry.content
        );
        Ok(frontmatter)
    }

    fn deserialize_entry(&self, id: &str, content: &str) -> Result<Entry> {
        // Simple frontmatter parsing
        if content.starts_with("---\n") {
            let parts: Vec<&str> = content.splitn(3, "---\n").collect();
            if parts.len() >= 3 {
                let yaml_content = parts[1];
                let md_content = parts[2].trim_start().to_string();

                // Parse YAML frontmatter
                let frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_content)
                    .wrap_err("Failed to parse YAML frontmatter")?;

                let created_at = frontmatter["created_at"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(Utc::now);

                let updated_at = frontmatter["updated_at"]
                    .as_str()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(Utc::now);

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
            created_at: Utc::now(),
            updated_at: Utc::now(),
            content: content.to_string(),
        })
    }
}
