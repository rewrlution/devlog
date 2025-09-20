use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,           // YYYYMMDD format
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content: String,      // Markdown content
}

impl Entry {
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        let id = now.format("%Y%m%d").to_string();

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
