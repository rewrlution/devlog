use chrono::{DateTime, Utc};

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
}
