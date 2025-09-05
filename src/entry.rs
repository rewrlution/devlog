use crate::annotations::{self, AnnotationParser, ParsedAnnotations};
use crate::events::EntryEvent;
use crate::storage::EntryStorage;
use chrono::{DateTime, Local};
use std::collections::HashSet;

/// Current state of an entry (derived from events)
#[derive(Debug, Clone)]
pub struct EntryState {
    pub id: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    pub content: String,
    pub tags: HashSet<String>,
    pub people: HashSet<String>,
    pub projects: HashSet<String>,
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

        let created_event = EntryEvent::Created {
            id: id.clone(),
            content: content.clone(),
            timestamp: now,
        };

        let mut entry = Entry {
            events: vec![created_event],
            state: EntryState {
                id,
                created_at: now,
                updated_at: now,
                content,
                tags: HashSet::new(),
                people: HashSet::new(),
                projects: HashSet::new(),
            },
            annotation_parser: AnnotationParser::new(),
        };

        // Automatically parse annotations on creation
        entry.parse_annotations();
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
}
