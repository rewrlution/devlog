use crate::models::entry::Entry;

use chrono::Utc;
use color_eyre::eyre::{Context, ContextCompat, Ok, Result};
use std::{fs, path::PathBuf};

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
            r#"---"
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
