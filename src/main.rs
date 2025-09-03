use chrono::{DateTime, Utc};
use std::collections::HashSet;
use regex::Regex;

pub struct Entry {
    pub id: String,
    pub date: DateTime<Utc>,
    pub content: String,
    pub tags: HashSet<String>,
    pub people: HashSet<String>,
    pub projects: HashSet<String>,
}

impl Entry {
    /// Create a new entry with the given content
    pub fn new(content: String) -> Self {
        let now = Utc::now();
        let id = format!("{}", now.format("%Y%m%d"));

        Self {
            id,
            date: now,
            content,
            tags: HashSet::new(),
            people: HashSet::new(),
            projects: HashSet::new(),
        }
    }

    /// Parse annotations from content and populate metadata
    pub fn parse_annotations(&mut self) {
        // Extract @mentions (people)
        self.extract_people();
    }

    fn extract_people(&mut self) {
        // Regex pattern: @ followed by one or more word characters (letters, digits, underscore)
        let re = Regex::new(r"@(\w+)").unwrap();

        // Find all matches in the content
        for captures in re.captures_iter(&self.content) {
            // captures[0] is the full match (@alice)
            // captures[1] is the first capture group (alice)
            if let Some(person_match) = captures.get(1) {
                let person = person_match.as_str().to_string();
                self.people.insert(person);
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_person() {
        let mut entry = Entry::new("Met with @alice today".to_string());
        entry.parse_annotations();

        assert_eq!(entry.people.len(), 1);
        assert!(entry.people.contains("alice"));
    }
}