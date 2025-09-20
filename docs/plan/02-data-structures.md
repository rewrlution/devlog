# Step 2: Data Structures and Storage

## Overview

Define the core data structures and implement basic file storage operations.

## Core Data Structure: Entry

Create `src/storage/entry.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use color_eyre::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,           // YYYY-MM-DD format
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content: String,      // Markdown content
}

impl Entry {
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        let id = now.format("%Y-%m-%d").to_string();

        Self {
            id,
            created_at: now,
            updated_at: now,
            content,
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}
```

## File Format Structure

Each entry will be stored as `~/.devlog/entries/YYYY-MM-DD.md`:

```markdown
---
created_at: "2024-09-20T10:30:00Z"
updated_at: "2024-09-20T15:45:00Z"
---

# Today's Progress

Worked on the devlog CLI tool. Made good progress on:

- Setting up the project structure
- @alice helped with the design decisions
- Working on ::search-service integration
- +rust +cli +productivity

## Next Steps

- Implement the TUI interface
- Add better error handling
```

## Storage Operations

Create `src/storage/mod.rs`:

```rust
pub mod entry;

use crate::storage::entry::Entry;
use color_eyre::{eyre::Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .wrap_err("Could not find home directory")?;
        let base_path = home.join(".devlog").join("entries");

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path)
            .wrap_err("Failed to create devlog directory")?;

        Ok(Self { base_path })
    }

    pub fn save_entry(&self, entry: &Entry) -> Result<()> {
        let file_path = self.base_path.join(format!("{}.md", entry.id));
        let content = self.serialize_entry(entry)?;
        fs::write(file_path, content)
            .wrap_err("Failed to write entry file")?;
        Ok(())
    }

    pub fn load_entry(&self, id: &str) -> Result<Entry> {
        let file_path = self.base_path.join(format!("{}.md", id));
        let content = fs::read_to_string(file_path)
            .wrap_err("Failed to read entry file")?;
        self.deserialize_entry(id, &content)
    }

    pub fn list_entries(&self) -> Result<Vec<String>> {
        let mut entries = Vec::new();

        for entry in WalkDir::new(&self.base_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension()
                .map_or(false, |ext| ext == "md"))
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
```

## Additional Dependency

Add to `Cargo.toml`:

```toml
# For finding home directory
dirs = "5.0"
```

## Key Rust Concepts Explained

- **Structs**: Like classes in other languages, but simpler
- **impl blocks**: Where you define methods for structs
- **Result<T>**: Rust's way of handling errors - either `Ok(value)` or `Err(error)`
- **PathBuf**: Like a string but for file paths, handles different OS path formats
- **&self vs self**: `&self` borrows the struct, `self` takes ownership
- **String vs &str**: `String` is owned, `&str` is borrowed text
- **wrap_err()**: color-eyre's method to add context to errors (like anyhow's context())

## Implementation Tasks

1. **Add the `dirs` dependency** to Cargo.toml
2. **Create `src/storage/entry.rs`** with the Entry struct
3. **Create `src/storage/mod.rs`** with the Storage implementation
4. **Update `src/lib.rs`** to include the storage module

## Next Steps

Move to Step 3: Basic CLI Commands to implement the command-line interface.
