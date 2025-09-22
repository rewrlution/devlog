use chrono::{DateTime, Utc};
use std::fmt;

pub struct Entry {
    pub id: String, // YYYYMMDD format
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content: String, // Markdown content
}

impl Entry {
    /// Create a new entry with id and content
    pub fn new(id: String, content: String) -> Self {
        let now = Utc::now();

        Self {
            id,
            created_at: now,
            updated_at: now,
            content,
        }
    }

    /// Update the content and timestamp
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Id: {}\nCreated: {}\nUpdated: {}\n---\n\n{}",
            self.id,
            self.created_at.to_rfc3339(),
            self.updated_at.to_rfc3339(),
            self.content
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entry_with_id() {
        let content = "Test content".to_string();
        let id = "20250910".to_string();

        let entry = Entry::new(id.clone(), content.clone());

        assert_eq!(entry.id, id);
        assert_eq!(entry.content, content);
    }

    #[test]
    fn test_update_content() {
        let id = "20250920".to_string();
        let content = "Test content".to_string();
        let mut entry = Entry::new(id, content);
        let original_created = entry.created_at;

        std::thread::sleep(std::time::Duration::from_millis(1));

        let updated_content = "Update content".to_string();

        entry.update_content(updated_content.clone());

        assert_eq!(entry.content, updated_content);
        assert_eq!(entry.created_at, original_created);
        assert!(entry.updated_at > entry.created_at);
    }

    #[test]
    fn test_display_format() {
        let id = "20250921".to_string();
        let content = "# Test Entry\n\nThis is a test entry.".to_string();
        let entry = Entry::new(id.clone(), content.clone());

        let display_output = format!("{}", entry);

        // Check that the display output contains expected parts
        assert!(display_output.contains(&format!("Id: {}", id)));
        assert!(display_output.contains("Created:"));
        assert!(display_output.contains("Updated:"));
        assert!(display_output.contains("---"));
        assert!(display_output.contains(&content));
    }
}
