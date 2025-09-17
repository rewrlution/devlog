use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Entry {
    pub date: NaiveDate,
    pub content: String,
}

impl Entry {
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            content: String::new(),
        }
    }

    pub fn with_content(date: NaiveDate, content: String) -> Self {
        Self { date, content }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_creation() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let entry = Entry::new(date);

        assert_eq!(entry.date, date);
        assert!(entry.content.is_empty());
    }

    #[test]
    fn test_entry_with_content() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let content = "Today I learned Rust!".to_string();
        let entry = Entry::with_content(date, content.clone());

        assert_eq!(entry.content, content);
    }
}
