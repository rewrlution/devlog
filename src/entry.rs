use crate::annotations::{self, AnnotationParser, ParsedAnnotations};
use crate::events::{self, EntryEvent};
use crate::storage::{self, EntryStorage};
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
