use chrono::{DateTime, Utc};
use std::fmt;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

    /// Get a preview of the entry content
    pub fn preview(&self) -> String {
        // Get the first line of content
        let first_line = self.content.lines().next().unwrap_or("").trim();

        // Target visual width of 60 characters
        const MAX_WIDTH: usize = 60;
        const ELIPSIS_WIDTH: usize = 3; // "..." width

        // If the line fits within the limit, return it as is
        if first_line.width() <= MAX_WIDTH {
            return first_line.to_string();
        }

        let mut current_width = 0;
        let mut truncate_at = 0;

        for (idx, ch) in first_line.char_indices() {
            let char_width = ch.width().unwrap_or(0);

            if current_width + char_width + ELIPSIS_WIDTH > MAX_WIDTH {
                break;
            }

            current_width += char_width;
            truncate_at = idx + ch.len_utf8();
        }

        format!("{}...", &first_line[..truncate_at])
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

    #[test]
    fn test_preview_short_content() {
        let id = "20250925".to_string();
        let content = "Short content".to_string();
        let entry = Entry::new(id, content.clone());

        let preview = entry.preview();
        assert_eq!(preview, content);
    }

    #[test]
    fn test_preview_long_ascii_content() {
        let id = "20250925".to_string();
        let content = "This is a very long line that exceeds sixty characters and should be truncated properly".to_string();
        let entry = Entry::new(id, content);

        let preview = entry.preview();
        assert!(preview.width() <= 60);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn test_preview_multiline_content() {
        let id = "20250925".to_string();
        let content = "First line\nSecond line\nThird line".to_string();
        let entry = Entry::new(id, content);

        let preview = entry.preview();
        assert_eq!(preview, "First line");
    }

    #[test]
    fn test_preview_chinese_characters() {
        let id = "20250925".to_string();
        // Chinese characters typically have width of 2 each
        let content =
            "这是一个包含中文字符的测试内容，用来测试预览功能是否能正确处理Unicode字符".to_string();
        let entry = Entry::new(id, content);

        let preview = entry.preview();
        assert!(preview.width() <= 60);
        assert!(preview.ends_with("..."));
    }
}
