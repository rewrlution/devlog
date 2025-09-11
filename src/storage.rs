use crate::events::EntryEvent;
use serde_json;
use std::fs;
use std::path::PathBuf;

/// Trait for handling file storage for `entries` and `events`
pub trait EntryStorage {
    /// Save all events for a given date (overwrites existing events)
    fn save_events(
        &self,
        date: &str,
        events: &[EntryEvent],
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Save markdown content (overwrites existing markdown content)
    fn save_markdown(&self, date: &str, content: &str) -> Result<(), Box<dyn std::error::Error>>;

    /// Load all events for a given date
    fn load_events(&self, date: &str) -> Result<Vec<EntryEvent>, Box<dyn std::error::Error>>;

    /// Load markdown content
    #[allow(dead_code)]
    fn load_markdown(&self, date: &str) -> Result<Option<String>, Box<dyn std::error::Error>>;

    /// List all entry IDs sorted in descending order (newest first)
    fn list_entry_ids(&self) -> Result<Vec<String>, Box<dyn std::error::Error>>;
}

/// Local file system implementation of entry storage
pub struct LocalEntryStorage {
    // `PathBuf` handles cross-platform path separators (`/` on Linux, `\` on Windows)
    // It also has built-in methods like `.join()` and `.exists()`
    base_dir: PathBuf,
}

impl LocalEntryStorage {
    /// Create a new local storage instance
    pub fn new(base_dir: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        // The `Box` error type is convinient to capture any error type that implements `std::error::Error`
        // Examples:
        // fs::create_dir_all(path)?;       // std::io::Error
        // serde_json::to_String(event)?;   // serde_json::Error
        // dirs::home_dir().expect(...);    // Option -> panic (but could be Result)

        // default storage path: `~/.devlog`
        // user custom path: `/custom/path`
        let base_dir = base_dir.unwrap_or_else(|| {
            dirs::home_dir()
                .expect("Could not find home directory")
                .join(".devlog")
        });

        // Ensure base directories exist
        fs::create_dir_all(base_dir.join("events"))?;
        fs::create_dir_all(base_dir.join("entries"))?;

        Ok(Self { base_dir })
    }

    /// Get the event file path for a given date
    fn events_path(&self, date: &str) -> PathBuf {
        self.base_dir.join("events").join(format!("{}.jsonl", date))
    }

    /// Get the markdown file path for a given date
    fn markdown_path(&self, date: &str) -> PathBuf {
        self.base_dir.join("entries").join(format!("{}.md", date))
    }
}

impl EntryStorage for LocalEntryStorage {
    /// Save all events for a given date (overwrites existing events)
    fn save_events(
        &self,
        date: &str,
        events: &[EntryEvent],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let events_path = self.events_path(date);

        let mut content = String::new();
        for event in events {
            let event_json = serde_json::to_string(event)?;
            content.push_str(&event_json);
            content.push('\n');
        }

        fs::write(&events_path, content)?;
        Ok(())
    }

    /// Save markdown content (overwrites existing markdown content)
    fn save_markdown(&self, date: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let markdown_path = self.markdown_path(date);
        fs::write(&markdown_path, content)?;
        Ok(())
    }

    /// Load all events for a given date
    fn load_events(&self, date: &str) -> Result<Vec<EntryEvent>, Box<dyn std::error::Error>> {
        let events_path = self.events_path(date);

        if !events_path.exists() {
            // Return empty vector for events for a new date
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&events_path)?;
        let mut events = Vec::new();

        for line in content.lines() {
            let event: EntryEvent = serde_json::from_str(line)?;
            events.push(event);
        }

        Ok(events)
    }

    /// Load markdown content
    fn load_markdown(&self, date: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let markdown_path = self.markdown_path(date);

        if !markdown_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&markdown_path)?;
        Ok(Some(content))
    }

    /// List all entry IDs sorted in descending order (newest first)
    fn list_entry_ids(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let entries_dir = self.base_dir.join("entries");

        if !entries_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entry_ids = Vec::new();

        for entry in fs::read_dir(entries_dir)? {
            // Entry is Result<DirEntry, Error>, not DirEntry
            // Each individual file/directory read operation could fail due to permission, corrupted filesystem, etc.
            let entry = entry?;

            // Get the file name
            let file_name = entry.file_name();
            // Convert OsString to String
            if let Some(file_name_str) = file_name.to_str() {
                // Remove the .md extension
                if file_name_str.ends_with(".md") {
                    let entry_id = file_name_str.strip_suffix(".md").unwrap().to_string();
                    entry_ids.push(entry_id);
                }
            }
        }

        // Sort entry IDs in descending order (newest first)
        entry_ids.sort_by(|a, b| b.cmp(a));

        Ok(entry_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use tempfile::TempDir;

    #[test]
    fn test_storage_operations() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let storage = LocalEntryStorage::new(Some(temp_dir.path().to_path_buf()))?;

        let now = Local::now();
        let date = format!("{}", now.format("%Y%m%d"));

        // Test event storage with save_events
        let events = vec![EntryEvent::Created {
            id: date.to_string(),
            content: "Test content".to_string(),
            timestamp: now,
        }];

        storage.save_events(&date, &events)?;

        // Test event loading
        let loaded_events = storage.load_events(&date)?;
        assert_eq!(loaded_events.len(), 1);

        // Test markdown storage
        let markdown = "# Test Entry\n\nTest content";
        storage.save_markdown(&date, markdown)?;

        // Test markdown loading
        let loaded_markdown = storage.load_markdown(&date)?;
        assert_eq!(loaded_markdown, Some(markdown.to_string()));

        Ok(())
    }

    #[test]
    fn test_save_events_overwrites() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let storage = LocalEntryStorage::new(Some(temp_dir.path().to_path_buf()))?;

        let now = Local::now();
        let date = format!("{}", now.format("%Y%m%d"));

        // First save some events
        let events1 = vec![
            EntryEvent::Created {
                id: date.to_string(),
                content: "First content".to_string(),
                timestamp: now,
            },
            EntryEvent::AnnotationParsed {
                tags: vec!["first".to_string()],
                people: Vec::new(),
                projects: Vec::new(),
                timestamp: now,
            },
        ];

        storage.save_events(&date, &events1)?;
        let loaded = storage.load_events(&date)?;
        assert_eq!(loaded.len(), 2);

        // Now save different events (should overwrite)
        let events2 = vec![
            EntryEvent::Created {
                id: date.to_string(),
                content: "Second content".to_string(),
                timestamp: now,
            },
            EntryEvent::AnnotationParsed {
                tags: vec!["second".to_string()],
                people: Vec::new(),
                projects: Vec::new(),
                timestamp: now,
            },
            EntryEvent::ContentUpdated {
                content: "Updated content".to_string(),
                timestamp: now,
            },
        ];

        storage.save_events(&date, &events2)?;
        let loaded = storage.load_events(&date)?;
        assert_eq!(loaded.len(), 3); // Should have 3 events, not 5

        Ok(())
    }

    #[test]
    fn test_save_empty_events() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let storage = LocalEntryStorage::new(Some(temp_dir.path().to_path_buf()))?;

        let date = "20250906";

        // Save empty events list
        storage.save_events(date, &[])?;

        // Should load empty list
        let loaded = storage.load_events(date)?;
        assert_eq!(loaded.len(), 0);

        Ok(())
    }
}
