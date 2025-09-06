# DevLog Event Sourcing Architecture

## Overview

DevLog is a developer journal CLI tool built on **event sourcing principles**. The system captures developer activities, parses annotations from natural language, and maintains complete historical state through immutable events stored in append-only logs.

## Core Event Sourcing Design

### Events as Source of Truth

All state changes are captured as immutable events:

```rust
enum EntryEvent {
    Created { id, content, timestamp },
    ContentUpdated { content, timestamp },
    AnnotationParsed { tags, people, projects, timestamp },
}
```

**Key Properties:**

- **Immutable**: Events never change once written
- **Timestamped**: Complete chronological ordering
- **Serializable**: JSON Lines format for storage
- **Append-only**: New events added, old events preserved

### Uniform Event Processing

Both new entries and loaded entries use identical event replay logic:

```rust
// Creation: applies events uniformly
let mut entry = Entry::new("content");

// Loading: reconstructs state from events
let entry = Entry::from_events(stored_events);
```

This ensures **consistent behavior** and **single source of truth** for state transitions.

## Architecture Components

### 1. Entry Aggregate (`src/entry.rs`)

Central business logic component managing events and state:

```rust
struct Entry {
    events: Vec<EntryEvent>,        // Complete event history
    state: EntryState,              // Current derived state
    annotation_parser: AnnotationParser,
}
```

**Responsibilities:**

- Generates events for all state changes
- Maintains current state via `apply_event()`
- Automatic annotation parsing on content changes
- Provides clean API for business operations

### 2. Dual Storage Strategy (`src/storage.rs`)

```
~/.devlog/
├── events/20250905.jsonl    # Event sourcing (append-only)
└── entries/20250905.md      # Current state (overwritten)
```

**Technical Choices:**

- **JSONL Format**: One JSON event per line for streaming/recovery
- **PathBuf**: Cross-platform file path handling
- **Dual Persistence**: Events for history, markdown for user convenience
- **Automatic Cleanup**: Events enable state reconstruction if markdown corrupted

### 3. Annotation System (`src/annotations.rs`)

Regex-based parsing extracting structured metadata from natural text:

```
@alice     → people: ["alice"]
::project  → projects: ["project"]
+rust      → tags: ["rust"]
```

**Implementation Details:**

- **Vec<String>**: Preserves order and allows duplicates (vs HashSet)
- **Generic Extraction**: Single `extract_with_regex()` for all annotation types
- **DRY Principle**: Eliminates code duplication across parsers

## Key Technical Decisions

### Why Vec<String> over HashSet<String>?

```rust
// Vec preserves order and frequency
"Met @alice then @bob then @alice" → ["alice", "bob", "alice"]

// HashSet would lose information
"Met @alice then @bob then @alice" → ["alice", "bob"]
```

**Benefits:**

- Order preservation (first mention vs later mentions)
- Frequency tracking (how often someone is mentioned)
- Simpler serialization (direct JSON arrays)

### Why Event Sourcing?

1. **Complete Audit Trail**: Every change tracked with timestamps
2. **Data Recovery**: State can always be rebuilt from events
3. **Future Analytics**: Rich historical data for insights
4. **Scalability**: Append-only writes are fast and scalable

### Error Handling Strategy

```rust
Result<T, Box<dyn std::error::Error>>
```

- **Flexible**: Handles any error type implementing `std::error::Error`
- **Composable**: File I/O, JSON parsing, and business logic errors
- **Ergonomic**: `?` operator for clean error propagation

## Event Flow Example

### Creating New Entry

```rust
Entry::new("Worked with @alice on ::search using +rust")
```

1. **Created Event**: `{ content: "...", timestamp: now }`
2. **Apply Event**: Update state with content and metadata
3. **Parse Annotations**: Extract @alice, ::search, +rust
4. **AnnotationParsed Event**: `{ people: ["alice"], ... }`
5. **Persist**: Save events to `.jsonl`, state to `.md`

### Loading Existing Entry

```rust
Entry::load("20250905", storage)
```

1. **Load Events**: Read from `events/20250905.jsonl`
2. **Replay Events**: Apply each event in chronological order
3. **Reconstruct State**: Final state matches original creation
4. **Return Entry**: Ready for further operations

## Performance Characteristics

### Storage

- **Write Performance**: O(1) append operations
- **Read Performance**: O(n) event replay (cached in memory)
- **Space Efficiency**: Events are compact JSON, markdown is human-readable

### Memory Usage

- **Current State**: Kept in memory for fast access
- **Event History**: Stored on disk, loaded on demand
- **Annotation Parsing**: Compiled regexes cached per entry

## Extensibility Points

### New Event Types

```rust
// Easy to add new events
EntryEvent::InsightsGenerated { insights, timestamp }
EntryEvent::TagsUpdated { added, removed, timestamp }
```

### New Annotation Types

```rust
// Add location mentions: #san-francisco
locations_regex: Regex::new(r"#([\w-]+)")
```

### Storage Backends

```rust
// Interface allows database, cloud storage
trait EventStorage {
    fn save_events(&self, date: &str, events: &[EntryEvent]) -> Result<()>;
    fn load_events(&self, date: &str) -> Result<Vec<EntryEvent>>;
}
```

## Testing Strategy

### Event Sourcing Validation

- **Consistency Tests**: `new()` and `from_events()` produce identical state
- **Replay Tests**: Complex event sequences reconstruct correctly
- **Edge Cases**: Empty events, malformed data, missing files

### Isolation with TempDir

```rust
let temp_dir = TempDir::new()?;
let storage = EntryStorage::new(Some(temp_dir.path().to_path_buf()))?;
```

- **Test Isolation**: Each test gets clean directory
- **Automatic Cleanup**: No test artifacts left behind
- **Parallel Safety**: Tests can run concurrently

## Future Enhancements Enabled

### Advanced Querying

- "Show all entries mentioning @alice in the last month"
- "What projects used +rust this quarter?"
- Cross-entry pattern analysis

### Real-time Features

- Event streaming for live collaboration
- WebSocket updates for team dashboards
- Conflict resolution through event ordering

### AI Integration

- `InsightsGenerated` events from LLM analysis
- Automated tagging based on content patterns
- Trend analysis across historical data

## Summary

This event sourcing implementation provides:

- **Data Integrity**: Immutable events prevent corruption
- **Complete History**: Every change preserved with context
- **Clean Architecture**: Clear separation of concerns
- **Extensible Foundation**: Easy to add new features
- **Rust Safety**: Ownership system prevents data races

The system demonstrates production-ready event sourcing while solving the practical problem of developer activity tracking with structured, searchable, and historically complete data.
