# Project Architecture

## Overview

This document outlines the architecture for the Engineering Journal app, designed to be simple and beginner-friendly while following Rust best practices.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                            │
│                    (Application Entry)                     │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                        App                                  │
│              (Main Application State)                      │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐ │
│  │    UI       │ Navigation  │   Editor    │  Storage    │ │
│  │  Module     │   Module    │   Module    │   Module    │ │
│  └─────────────┴─────────────┴─────────────┴─────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Module Structure

### File Organization

```
src/
├── main.rs              # Entry point
├── app.rs               # Main application state and logic
├── ui/
│   ├── mod.rs          # UI module exports
│   ├── layout.rs       # Layout and rendering
│   └── events.rs       # Event handling
├── navigation/
│   ├── mod.rs          # Navigation module exports
│   ├── tree.rs         # Tree structure and navigation
│   └── state.rs        # Navigation state management
├── editor/
│   ├── mod.rs          # Editor module exports
│   ├── text.rs         # Text editing logic
│   └── modes.rs        # Navigation vs Edit modes
├── storage/
│   ├── mod.rs          # Storage module exports
│   ├── filesystem.rs   # File operations
│   └── entry.rs        # Entry data structure
└── utils/
    ├── mod.rs          # Utility module exports
    └── date.rs         # Date formatting and parsing
```

## Core Data Structures

### 1. Application State

```rust
pub struct App {
    // Application modes
    pub mode: AppMode,
    pub should_quit: bool,

    // Core components
    pub navigation: NavigationState,
    pub editor: EditorState,
    pub storage: Storage,

    // UI state
    pub current_view: ViewMode,
}

pub enum AppMode {
    Navigation,
    Edit,
    Prompt(PromptType),
}

pub enum PromptType {
    CreateEntry,
    DeleteConfirmation,
}
```

### 2. Navigation State

```rust
pub struct NavigationState {
    pub tree: EntryTree,
    pub selected_path: Option<EntryPath>,
    pub expanded_paths: HashSet<String>,
}

pub struct EntryTree {
    pub years: BTreeMap<u32, Year>,
}

pub struct Year {
    pub year: u32,
    pub months: BTreeMap<u32, Month>,
}

pub struct Month {
    pub month: u32,
    pub entries: BTreeMap<u32, Entry>,
}
```

### 3. Entry Data

```rust
pub struct Entry {
    pub date: NaiveDate,
    pub path: PathBuf,
    pub content: Option<String>, // Lazy loaded
}

pub struct EntryPath {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}
```

### 4. Editor State

```rust
pub struct EditorState {
    pub content: Vec<String>, // Lines of text
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub dirty: bool, // Has unsaved changes
    pub current_entry: Option<EntryPath>,
}
```

## Key Design Principles

### 1. Separation of Concerns

- **UI**: Only handles rendering and layout
- **Navigation**: Manages tree state and selection
- **Editor**: Handles text editing logic
- **Storage**: Manages file I/O operations

### 2. Error Handling Strategy

```rust
// Use color-eyre for all error handling
type Result<T> = color_eyre::Result<T>;

// Example usage
pub fn load_entry(path: &EntryPath) -> Result<Entry> {
    // Implementation with automatic error propagation
}
```

### 3. State Management

- **Single source of truth**: All state in `App` struct
- **Immutable updates**: State changes through methods
- **Event-driven**: UI events trigger state changes

## Communication Patterns

### Event Flow

```
User Input → UI Events → App State Update → UI Re-render
     ↑                                           ↓
     └─────────── Storage Operations ←───────────┘
```

### Example Event Handling

```rust
impl App {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            AppMode::Navigation => self.handle_navigation_key(key),
            AppMode::Edit => self.handle_editor_key(key),
            AppMode::Prompt(prompt_type) => self.handle_prompt_key(key, prompt_type),
        }
    }
}
```

## Rust Learning Opportunities

### Concepts You'll Practice

1. **Ownership**: Passing data between modules
2. **Borrowing**: Accessing app state without moving
3. **Pattern Matching**: Handling different modes and events
4. **Error Handling**: Using Result<T, E> throughout
5. **Traits**: Implementing custom behaviors
6. **Modules**: Organizing code logically

### Starting Simple

- Begin with basic structs and enums
- Add complexity gradually
- Use `#[derive(Debug)]` for easy debugging
- Start with synchronous file I/O

## Dependencies Interaction

### ratatui Integration

```rust
// UI rendering with ratatui
impl Widget for NavigationTree {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render tree using ratatui widgets
    }
}
```

### chrono Integration

```rust
// Date handling with chrono
pub fn parse_entry_date(date_str: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y%m%d")
        .map_err(|e| eyre::eyre!("Invalid date format: {}", e))
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_path_creation() {
        let path = EntryPath::new(2025, 3, 15);
        assert_eq!(path.to_string(), "20250315");
    }
}
```

### Integration Tests

- Test file operations with temporary directories
- Test UI components with mock events
- Test complete workflows

## Next Steps

1. Set up basic project structure
2. Implement core data structures
3. Start with navigation module (see 03-implementation-phases.md)
4. Add UI layer progressively
