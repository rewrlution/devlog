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
    /// @alice -> people
    /// ::search-engine -> projects
    /// +motivation -> tags
    pub fn parse_annotations(&mut self) {
        self.extract_people();
        self.extract_projects();
        self.extract_tags();
    }

    fn extract_people(&mut self) {
        // Regex pattern: @ followed by one or more word characters (letters, digits, underscore)
        let re = Regex::new(r"@(\w+)").unwrap();

        // Find all matches in the content
        for captures in re.captures_iter(&self.content) {
            // captures[0] is the full match (@alice)
            // captures[1] is the first capture group (alice)
            if let Some(person) = captures.get(1) {
                let person = person.as_str().to_string();
                self.people.insert(person);
            }
        }
    }

    fn extract_projects(&mut self) {
        let re = Regex::new(r"::(\w+)").unwrap();
        for captures in re.captures_iter(&self.content) {
            if let Some(project) = captures.get(1) {
                let project = project.as_str().to_string();
                self.projects.insert(project);
            }
        }
    }

    fn extract_tags(&mut self) {
        let re = Regex::new(r"\+(\w+)").unwrap();
        for captures in re.captures_iter(&self.content) {
            if let Some(tag) = captures.get(1) {
                let tag = tag.as_str().to_string();
                self.tags.insert(tag);
            }
        }
    }

    /// Convert entry to frontmatter + markdown format
    /// 
    /// This method demonstrates:
    /// - String formatting with `format!` macro
    /// - Iterator methods (`join`) for clean collection formatting
    /// - YAML frontmatter generation
    pub fn to_markdown(&self) -> String {
        let tags: Vec<&str> = self.tags.iter().map(|s| s.as_str()).collect();
        let people: Vec<&str> = self.people.iter().map(|s| s.as_str()).collect();
        let projects: Vec<&str> = self.projects.iter().map(|s| s.as_str()).collect();

        format!(
            r#"---
date: {}
tags: [{}]
people: [{}]
projects: [{}]
---

{}
"#,
            self.date.format("%Y-%m-%d UTC"),
            tags.join(", "),
            people.join(", "),
            projects.join(", "),
            self.content
        )
    }
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

    #[test]
    fn test_extract_multiple_people() {
        let mut entry = Entry::new("Worked with @alice, @bob, and @charlie".to_string());
        entry.parse_annotations();

        assert_eq!(entry.people.len(), 3);
        assert!(entry.people.contains("alice"));
        assert!(entry.people.contains("bob"));
        assert!(entry.people.contains("charlie"));
    }

    #[test]
    fn test_extract_people_with_punctuation() {
        let mut entry = Entry::new("Talked to @alice! Then @bob? Finally @charlie".to_string());
        entry.parse_annotations();

        assert_eq!(entry.people.len(), 3);
        assert!(entry.people.contains("alice"));
        assert!(entry.people.contains("bob"));
        assert!(entry.people.contains("charlie"));
    }

    #[test]
    fn test_duplicate_people_are_deduplicated() {
        let mut entry = Entry::new("@alice helped, then @alice helped again, and @alice was great!".to_string());
        entry.parse_annotations();
        
        // Should only have one "alice" despite being mentioned 3 times
        assert_eq!(entry.people.len(), 1);
        assert!(entry.people.contains("alice"));
    }

    #[test]
    fn test_no_people_mentioned() {
        let mut entry = Entry::new("Just worked alone today on some code".to_string());
        entry.parse_annotations();
        
        assert!(entry.people.is_empty());
    }

    #[test]
    fn test_ignore_standalone_at_symbol() {
        let mut entry = Entry::new("The @ symbol alone or @ with space should be ignored".to_string());
        entry.parse_annotations();
        
        assert!(entry.people.is_empty());
    }

    #[test]
    fn test_people_with_numbers_and_underscores() {
        let mut entry = Entry::new("Worked with @alice_smith and @bob123 today".to_string());
        entry.parse_annotations();
        
        assert_eq!(entry.people.len(), 2);
        assert!(entry.people.contains("alice_smith"));
        assert!(entry.people.contains("bob123"));
    }
    
    #[test]
    fn test_mixed_content() {
        let content = r#"
        Today was productive! Met with @sarah in the morning about the project.
        @mike joined us later, and we brainstormed solutions.
        Email from @jennifer_parker arrived at 3pm.
        Need to follow up with @sam_w tomorrow.
        "#.to_string();
        
        let mut entry = Entry::new(content);
        entry.parse_annotations();
        
        assert_eq!(entry.people.len(), 4);
        assert!(entry.people.contains("sarah"));
        assert!(entry.people.contains("mike"));
        assert!(entry.people.contains("jennifer_parker"));
        assert!(entry.people.contains("sam_w"));
    }

    #[test]
    fn test_parse_project_annotations() {
        let content = "Working on ::search_engine and ::auth_platform services".to_string();
        let mut entry = Entry::new(content);
        entry.parse_annotations();

        assert_eq!(entry.projects.len(), 2);
        assert!(entry.projects.contains("search_engine"));
        assert!(entry.projects.contains("auth_platform"));
    }

    #[test]
    fn test_parse_tag_annotations() {
        let content = "Learned +rust and +azure_ai today. I feel very +excited!".to_string();
        let mut entry = Entry::new(content);

        entry.parse_annotations();

        assert_eq!(entry.tags.len(), 3);
        assert!(entry.tags.contains("rust"));
        assert!(entry.tags.contains("azure_ai"));
        assert!(entry.tags.contains("excited"));
    }

    #[test]
    fn test_parse_mixed_annotations() {
        let content = "Worked with @alice on ::search_service using +rust and +tokio".to_string();
        let mut entry = Entry::new(content);
        
        entry.parse_annotations();
        
        assert_eq!(entry.people.len(), 1);
        assert!(entry.people.contains("alice"));
        
        assert_eq!(entry.projects.len(), 1);
        assert!(entry.projects.contains("search_service"));
        
        assert_eq!(entry.tags.len(), 2);
        assert!(entry.tags.contains("rust"));
        assert!(entry.tags.contains("tokio"));
    }

    #[test]
    fn test_to_markdown() {
        let mut entry = Entry::new("Worked with @alice and @bob on ::search_engine using +rust".to_string());
        entry.parse_annotations();

        let markdown = entry.to_markdown();

        println!("{}", markdown);

        // Just test that it contains the expected parts
        assert!(markdown.contains("---"));
        assert!(markdown.contains("date:"));
        assert!(markdown.contains("tags: [rust]") || markdown.contains("tags: [rust]"));
        assert!(markdown.contains("people: [alice, bob]") || markdown.contains("people: [bob, alice]"));
        assert!(markdown.contains("projects: [search_engine]"));
        assert!(markdown.contains("Worked with @alice and @bob on ::search_engine using +rust"));
    }
}