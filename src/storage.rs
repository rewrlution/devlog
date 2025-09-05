use crate::events::EntryEvent;
use chrono::{DateTime, Local};
use serde_json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Handle file storage for `entries` and `events`
pub struct EntryStorage {
    // `PathBuf` handles cross-platform path separators (`/` on Linux, `\` on Windows)
    // It also has built-in methods like `.join()` and `.exists()`
    base_dir: PathBuf,
}

impl EntryStorage {
    /// Create a new storage instance
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

    /// Append an event to the event log
    pub fn append_event(&self, date: &str, event: &EntryEvent) -> Result<(), Box<dyn std::error::Error>> {
        let events_path = self.events_path(date);
        let event_json = serde_json::to_string(event)?;

        let mut file = OpenOptions::new()
            .create(true)   // Create if doesn't exist
            .append(true)   // Append to the end of a file
            .open(&events_path)?;

        writeln!(file, "{}", event_json)?;
        Ok(())
    }

    /// Save markdown content (overwrites existing)
    pub fn save_markdown(&self, date: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let markdown_path = self.markdown_path(date);
        fs::write(&markdown_path, content)?;
        Ok(())
    }

    /// Load all events for a given date
    pub fn load_events(&self, date: &str) -> Result<Vec<EntryEvent>, Box<dyn std::error::Error>> {
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
    pub fn load_markdown(&self, date: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let markdown_path = self.markdown_path(date);

        if !markdown_path.exists() {
            return Ok(None)
        }

        let content = fs::read_to_string(&markdown_path)?;
        Ok(Some(content))
    }
}