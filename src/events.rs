use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Events that can happen to an entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
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
        tags: Vec<String>,
        people: Vec<String>,
        projects: Vec<String>,
        timestamp: DateTime<Local>,
    },
}

impl EntryEvent {
    /// Get the timestamp of when this event occured
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn entry_id(&self) -> Option<&str> {
        match self {
            EntryEvent::Created { id, .. } => Some(id),
            _ => None, // Other events don't carry the ID directly
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_timestamp() -> DateTime<Local> {
        Local.with_ymd_and_hms(2025, 9, 6, 10, 30, 0).unwrap()
    }

    #[test]
    fn test_created_event_creation() {
        let timestamp = create_test_timestamp();
        let event = EntryEvent::Created {
            id: "20250906".to_string(),
            content: "Test content".to_string(),
            timestamp,
        };

        assert_eq!(event.timestamp(), timestamp);
        assert_eq!(event.entry_id(), Some("20250906"));
    }

    #[test]
    fn test_content_updated_event() {
        let timestamp = create_test_timestamp();
        let event = EntryEvent::ContentUpdated {
            content: "Updated content".to_string(),
            timestamp,
        };

        assert_eq!(event.timestamp(), timestamp);
        assert_eq!(event.entry_id(), None); // Update event itself doesn't have event id
    }

    #[test]
    fn test_serialized_format_has_type_tag() {
        let timestamp = create_test_timestamp();
        let event = EntryEvent::Created {
            id: "test".to_string(),
            content: "content".to_string(),
            timestamp,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"Created\""));
    }

    #[test]
    fn test_all_event_types_timestamp_method() {
        let timestamp1 = create_test_timestamp();
        let timestamp2 = Local.with_ymd_and_hms(2025, 9, 7, 15, 45, 30).unwrap();
        let timestamp3 = Local.with_ymd_and_hms(2025, 9, 8, 9, 15, 45).unwrap();

        let created = EntryEvent::Created {
            id: "20250906".to_string(),
            content: "content".to_string(),
            timestamp: timestamp1,
        };

        let updated = EntryEvent::ContentUpdated {
            content: "new content".to_string(),
            timestamp: timestamp2,
        };

        let parsed = EntryEvent::AnnotationParsed {
            tags: vec!["awesome".to_string()],
            people: vec!["alice".to_string()],
            projects: vec!["devlog".to_string()],
            timestamp: timestamp3,
        };

        assert_eq!(created.timestamp(), timestamp1);
        assert_eq!(updated.timestamp(), timestamp2);
        assert_eq!(parsed.timestamp(), timestamp3);
    }
}
