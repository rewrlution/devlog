use crate::annotations::AnnotationParser;
use crate::events::EntryEvent;
use crate::storage::EntryStorage;
use chrono::{DateTime, Local};

/// Current state of an entry (derived from events)
#[derive(Debug, Clone)]
pub struct EntryState {
    pub id: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub content: String,
    pub tags: Vec<String>,
    pub people: Vec<String>,
    pub projects: Vec<String>,
}

impl Default for EntryState {
    fn default() -> Self {
        let now = Local::now();
        Self {
            id: String::new(),
            created_at: now,
            updated_at: now,
            content: String::new(),
            tags: Vec::new(),
            people: Vec::new(),
            projects: Vec::new(),
        }
    }
}

/// The main Entry aggregate that manages events and state
pub struct Entry {
    events: Vec<EntryEvent>,
    state: EntryState,
    annotation_parser: AnnotationParser,
}

impl Entry {
    /// Create a new entry with inital content
    pub fn new(content: String) -> Self {
        let now = Local::now();
        let id = format!("{}", now.format("%Y%m%d"));

        // Start with empty entry and apply events
        let mut entry = Entry {
            events: Vec::new(),
            state: EntryState::default(),
            annotation_parser: AnnotationParser::new(),
        };

        let event = EntryEvent::Created {
            id,
            content,
            timestamp: now,
        };
        entry.apply_event(event);
        entry.parse_annotations(); // Automatically parse annotations on creation
        entry
    }

    /// Update the content and automatically reparse annotations
    pub fn update_content(&mut self, new_content: String) {
        let event = EntryEvent::ContentUpdated {
            content: new_content.clone(),
            timestamp: Local::now(),
        };

        self.apply_event(event);
        self.parse_annotations(); // reparse annotations when content changes
    }

    /// Parse annotations and record the parsing event
    fn parse_annotations(&mut self) {
        let annotations = self.annotation_parser.parse(&self.state.content);

        let event = EntryEvent::AnnotationParsed {
            tags: annotations.tags,
            people: annotations.people,
            projects: annotations.projects,
            timestamp: Local::now(),
        };

        self.apply_event(event);
    }

    /// Apply an event to update the current state
    fn apply_event(&mut self, event: EntryEvent) {
        match &event {
            EntryEvent::Created {
                id,
                content,
                timestamp,
            } => {
                self.state.id = id.clone();
                self.state.content = content.clone();
                self.state.created_at = *timestamp;
                self.state.updated_at = *timestamp;
            }
            EntryEvent::ContentUpdated { content, timestamp } => {
                self.state.content = content.clone();
                self.state.updated_at = *timestamp;
            }
            EntryEvent::AnnotationParsed {
                tags,
                people,
                projects,
                timestamp,
            } => {
                self.state.tags = tags.clone();
                self.state.people = people.clone();
                self.state.projects = projects.clone();
                self.state.updated_at = *timestamp;
            }
        }
        self.events.push(event);
    }

    /// Get the current state (what user sees)
    pub fn current_state(&self) -> &EntryState {
        &self.state
    }

    /// Get all events (for storage or debugging)
    #[allow(dead_code)]
    pub fn events(&self) -> &[EntryEvent] {
        &self.events
    }

    /// Rebuild entry from events
    pub fn from_events(events: Vec<EntryEvent>) -> Option<Self> {
        if events.is_empty() {
            return None;
        }

        // Start with default state
        let mut entry = Entry {
            events: Vec::new(),
            state: EntryState::default(),
            annotation_parser: AnnotationParser::new(),
        };

        // Apply all events to the state
        for event in events {
            entry.apply_event(event);
        }

        Some(entry)
    }

    /// Convert current state to markdown content
    pub fn to_markdown(&self) -> String {
        format!(
            r#"---
id: {}
created_at: {}
updated_at: {}
tags: [{}]
people: [{}]
projects: [{}]
---

{}
"#,
            self.state.id,
            self.state.created_at.format("%Y-%m-%dT%H:%M:%S%:z"),
            self.state.updated_at.format("%Y-%m-%dT%H:%M:%S%:z"),
            self.state.tags.join(", "),
            self.state.people.join(", "),
            self.state.projects.join(", "),
            self.state.content,
        )
    }

    /// Save entry to storage
    pub fn save(&self, storage: &EntryStorage) -> Result<(), Box<dyn std::error::Error>> {
        let date = &self.state.id;

        // Save all events
        for event in &self.events {
            storage.append_event(date, event)?;
        }

        // Save current markdown
        let markdown = self.to_markdown();
        storage.save_markdown(date, &markdown)?;

        Ok(())
    }

    /// Load entry from storage
    pub fn load(
        date: &str,
        storage: &EntryStorage,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let events = storage.load_events(date)?;
        Ok(Entry::from_events(events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_timestamp() -> DateTime<Local> {
        Local.with_ymd_and_hms(2025, 9, 5, 14, 30, 0).unwrap()
    }

    #[test]
    fn test_new_entry_basic() {
        let entry = Entry::new("Test content".to_string());
        let state = entry.current_state();

        assert_eq!(state.content, "Test content");
        assert!(!state.id.is_empty());

        // Should have 2 events: Created and AnnotationParsed
        assert_eq!(entry.events.len(), 2);
    }

    #[test]
    fn test_new_entry_with_annotations() {
        let entry = Entry::new("Worked with @alice on ::search_engine using +rust".to_string());
        let state = entry.current_state();

        assert_eq!(
            state.content,
            "Worked with @alice on ::search_engine using +rust"
        );
        assert_eq!(state.people.len(), 1);
        assert_eq!(state.people[0], "alice");
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.projects[0], "search_engine");
        assert_eq!(state.tags.len(), 1);
        assert_eq!(state.tags[0], "rust");

        // Should have 2 events: Created and AnnotationParsed
        assert_eq!(entry.events().len(), 2);
    }

    #[test]
    fn test_new_entry_preserves_annotation_order() {
        let entry = Entry::new("Met @alice then @bob then @alice again".to_string());
        let state = entry.current_state();

        // Vec preserves order and allows duplicates
        assert_eq!(state.people.len(), 3);
        assert_eq!(state.people[0], "alice");
        assert_eq!(state.people[1], "bob");
        assert_eq!(state.people[2], "alice");
    }

    #[test]
    fn test_update_content() {
        let mut entry = Entry::new("Initial content".to_string());
        let initial_events = entry.events().len();

        entry.update_content("Updated with @bob and +learning".to_string());

        let state = entry.current_state();
        assert_eq!(state.content, "Updated with @bob and +learning");
        assert_eq!(state.people[0], "bob");
        assert_eq!(state.tags[0], "learning");

        // Should have added ContentUpdated and AnnotationParsed events
        assert_eq!(entry.events().len(), initial_events + 2);
    }

    #[test]
    fn test_multiple_content_updates() {
        let mut entry = Entry::new("Initial".to_string());

        entry.update_content("First update @alice".to_string());
        entry.update_content("Second update @bob +rust".to_string());

        let state = entry.current_state();
        assert_eq!(state.content, "Second update @bob +rust");
        assert_eq!(state.people[0], "bob");
        assert_eq!(state.tags[0], "rust");

        // Should have 6 events: Created, AnnotationParsed, ContentUpdated, AnnotationParsed, ContentUpdated, AnnotationParsed
        assert_eq!(entry.events().len(), 6);
    }

    #[test]
    fn test_from_events_empty() {
        let result = Entry::from_events(Vec::new());
        assert!(result.is_none());
    }

    #[test]
    fn test_from_events_single_created() {
        let timestamp = create_test_timestamp();
        let events = vec![EntryEvent::Created {
            id: "20250905".to_string(),
            content: "Test content".to_string(),
            timestamp,
        }];

        let entry = Entry::from_events(events).unwrap();
        let state = entry.current_state();

        assert_eq!(state.id, "20250905");
        assert_eq!(state.content, "Test content");
        assert_eq!(state.created_at, timestamp);
        assert_eq!(state.updated_at, timestamp);
        assert_eq!(entry.events().len(), 1);
    }

    #[test]
    fn test_from_events_with_annotations() {
        let timestamp = create_test_timestamp();
        let events = vec![
            EntryEvent::Created {
                id: "20250905".to_string(),
                content: "Test content @alice".to_string(),
                timestamp,
            },
            EntryEvent::AnnotationParsed {
                tags: Vec::new(),
                people: vec!["alice".to_string()],
                projects: Vec::new(),
                timestamp,
            },
        ];

        let entry = Entry::from_events(events).unwrap();
        let state = entry.current_state();

        assert_eq!(state.content, "Test content @alice");
        assert_eq!(state.people[0], "alice");
        assert_eq!(entry.events().len(), 2);
    }

    #[test]
    fn test_from_events_complex_sequence() {
        let timestamp = create_test_timestamp();
        let events = vec![
            EntryEvent::Created {
                id: "20250905".to_string(),
                content: "Initial content".to_string(),
                timestamp,
            },
            EntryEvent::AnnotationParsed {
                tags: Vec::new(),
                people: Vec::new(),
                projects: Vec::new(),
                timestamp,
            },
            EntryEvent::ContentUpdated {
                content: "Updated with @alice +rust".to_string(),
                timestamp,
            },
            EntryEvent::AnnotationParsed {
                tags: vec!["rust".to_string()],
                people: vec!["alice".to_string()],
                projects: Vec::new(),
                timestamp,
            },
        ];

        let entry = Entry::from_events(events).unwrap();
        let state = entry.current_state();

        assert_eq!(state.content, "Updated with @alice +rust");
        assert_eq!(state.people[0], "alice");
        assert_eq!(state.tags[0], "rust");
        assert_eq!(entry.events().len(), 4);
    }

    #[test]
    fn test_new_and_from_events_consistency() {
        // Create entry using new()
        let entry1 = Entry::new("Test content @alice +rust".to_string());

        // Create entry using from_events() with same events
        let events = entry1.events().to_vec();
        let entry2 = Entry::from_events(events).unwrap();

        // Both should have identical state
        assert_eq!(entry1.state.id, entry2.state.id);
        assert_eq!(entry1.state.content, entry2.state.content);
        assert_eq!(entry1.state.people, entry2.state.people);
        assert_eq!(entry1.state.tags, entry2.state.tags);
        assert_eq!(entry1.state.projects, entry2.state.projects);
        assert_eq!(entry1.events().len(), entry2.events().len());
    }

    #[test]
    fn test_to_markdown_basic() {
        let entry = Entry::new("Simple content".to_string());
        let markdown = entry.to_markdown();

        assert!(markdown.contains("---"));
        assert!(markdown.contains("id:"));
        assert!(markdown.contains("created_at:"));
        assert!(markdown.contains("updated_at:"));
        assert!(markdown.contains("tags: []"));
        assert!(markdown.contains("people: []"));
        assert!(markdown.contains("projects: []"));
        assert!(markdown.contains("Simple content"));
    }

    #[test]
    fn test_to_markdown_with_annotations() {
        let entry = Entry::new("Worked with @alice and @bob on ::project using +rust".to_string());
        let markdown = entry.to_markdown();

        assert!(markdown.contains("---"));
        assert!(markdown.contains("people: [alice, bob]"));
        assert!(markdown.contains("projects: [project]"));
        assert!(markdown.contains("tags: [rust]"));
        assert!(markdown.contains("Worked with @alice and @bob"));

        // Test ISO 8601 timestamp format
        assert!(markdown.contains("T") && (markdown.contains("+") || markdown.contains("-")));
    }

    #[test]
    fn test_to_markdown_empty_annotations() {
        let entry = Entry::new("No annotations here".to_string());
        let markdown = entry.to_markdown();

        assert!(markdown.contains("tags: []"));
        assert!(markdown.contains("people: []"));
        assert!(markdown.contains("projects: []"));
    }

    #[test]
    fn test_to_markdown_multiple_annotations() {
        let entry = Entry::new(
            "Complex: @alice @bob @charlie +rust +tokio +async ::project1 ::project2".to_string(),
        );
        let markdown = entry.to_markdown();

        assert!(markdown.contains("people: [alice, bob, charlie]"));
        assert!(markdown.contains("tags: [rust, tokio, async]"));
        assert!(markdown.contains("projects: [project1, project2]"));
    }
}
