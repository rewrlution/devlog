use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Events that can happen to an entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="type", content="data")]
pub enum EntryEvent {
    Created {
        id: String,
        content: String,
        timestamp: DateTime<Local>,
    },
    ContentUpdated {
        content: String,
        timestamp: DateTime<Local>,
    },
    AnnotationParsed {
        tags: HashSet<String>,
        people: HashSet<String>,
        projects: HashSet<String>,
        timestamp: DateTime<Local>,
    },
}

impl EntryEvent {
    /// Get the timestamp of when this event occured
    pub fn timestamp(&self) -> DateTime<Local> {
        match self {
            // * is the dereference operator in Rust
            // Since `DateTime<Local>` implements the `Copy` trait
            // dereferencing with `*` creates a copy of the value
            EntryEvent::Created { timestamp, .. } => *timestamp,
            EntryEvent::ContentUpdated { timestamp, .. } => *timestamp,
            EntryEvent::AnnotationParsed { timestamp, .. } => *timestamp,
        }
    }

    /// Get the entry ID this event belongs to
    pub fn entry_id(&self) -> Option<&str> {
        match self {
            EntryEvent::Created { id, .. } => Some(id),
            _ => None, // Other events don't carry the ID directly
        }
    }
}
