# Rust Learning Guide

## Overview

This guide is specifically tailored for implementing the Engineering Journal app. It focuses on the Rust concepts you'll encounter and provides practical examples from our project.

## Core Rust Concepts for This Project

### 1. Ownership and Borrowing

#### The Problem

Rust ensures memory safety without garbage collection through ownership rules.

#### In Our Project

```rust
// ❌ This won't work - value moved
let app = App::new();
handle_ui(app);
handle_events(app); // Error: app was moved

// ✅ This works - borrowing
let mut app = App::new();
handle_ui(&app);        // Immutable borrow
handle_events(&mut app); // Mutable borrow
```

#### Practical Example: App State

```rust
impl App {
    // Takes ownership of self, returns new state
    pub fn transition_to_edit_mode(mut self, entry_path: EntryPath) -> Self {
        self.mode = AppMode::Edit;
        self.editor.load_entry(entry_path);
        self
    }

    // Borrows self mutably to modify in place
    pub fn handle_navigation(&mut self, direction: NavDirection) {
        self.navigation.move_selection(direction);
    }

    // Borrows self immutably to read data
    pub fn current_entry(&self) -> Option<&Entry> {
        self.navigation.selected_entry()
    }
}
```

#### Learning Strategy

1. **Start with cloning**: Use `.clone()` liberally at first
2. **Understand the borrow checker**: Read error messages carefully
3. **Refactor gradually**: Remove unnecessary clones as you learn

### 2. Error Handling

#### The Rust Way

Rust uses `Result<T, E>` for recoverable errors and `panic!` for unrecoverable ones.

#### In Our Project

```rust
use color_eyre::Result;  // This is Result<T, color_eyre::Report>

// File operations
pub fn load_entry(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .map_err(|e| eyre::eyre!("Failed to load entry: {}", e))
}

// Using the ? operator for error propagation
pub fn save_entry(entry: &Entry) -> Result<()> {
    let dir = entry.path.parent().unwrap();
    std::fs::create_dir_all(dir)?;  // Propagates error if fails
    std::fs::write(&entry.path, &entry.content)?;  // Propagates error if fails
    Ok(())  // Success
}

// Handling errors in main application
pub fn handle_save_request(&mut self) -> Result<()> {
    match self.editor.current_entry() {
        Some(entry) => {
            self.storage.save_entry(entry)?;
            self.editor.mark_clean();
            Ok(())
        }
        None => Err(eyre::eyre!("No entry to save")),
    }
}
```

#### Common Patterns

```rust
// Converting between error types
let date = NaiveDate::parse_from_str(date_str, "%Y%m%d")
    .map_err(|e| eyre::eyre!("Invalid date format: {}", e))?;

// Providing context
std::fs::create_dir_all(&dir)
    .with_context(|| format!("Failed to create directory: {:?}", dir))?;

// Handling Options
let entry = self.entries.get(&date)
    .ok_or_else(|| eyre::eyre!("Entry not found for date: {}", date))?;
```

### 3. Pattern Matching

#### Basic Matching

```rust
// Handling different key events
match key.code {
    KeyCode::Char('q') => self.should_quit = true,
    KeyCode::Char('j') | KeyCode::Down => self.navigation.move_down(),
    KeyCode::Char('k') | KeyCode::Up => self.navigation.move_up(),
    KeyCode::Enter => self.open_selected_entry()?,
    _ => {} // Do nothing for other keys
}

// Matching on app modes
match self.mode {
    AppMode::Navigation => self.handle_navigation_key(key),
    AppMode::Edit => self.handle_editor_key(key),
    AppMode::Prompt(PromptType::CreateEntry) => self.handle_create_prompt(key),
    AppMode::Prompt(PromptType::DeleteConfirmation) => self.handle_delete_prompt(key),
}
```

#### Advanced Patterns

```rust
// Matching with guards
match (self.mode, key.code) {
    (AppMode::Edit, KeyCode::Esc) => {
        if self.editor.is_dirty() {
            self.show_unsaved_changes_prompt();
        } else {
            self.mode = AppMode::Navigation;
        }
    }
    (AppMode::Navigation, KeyCode::Char('d')) if self.navigation.has_selection() => {
        self.show_delete_confirmation();
    }
    _ => {}
}

// Destructuring structs
match &self.navigation.selected_entry() {
    Some(Entry { date, path, .. }) => {
        println!("Selected entry: {} at {:?}", date, path);
    }
    None => println!("No entry selected"),
}
```

### 4. Traits

#### Using Existing Traits

```rust
// Debug trait for easy printing
#[derive(Debug)]
pub struct Entry {
    pub date: NaiveDate,
    pub content: String,
}

// Clone trait for duplicating data
#[derive(Clone)]
pub struct NavigationState {
    pub selected: Option<EntryPath>,
}

// Default trait for initial states
#[derive(Default)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
}
```

#### Implementing Custom Traits

```rust
// ratatui Widget trait
use ratatui::{widgets::Widget, buffer::Buffer, layout::Rect};

impl Widget for EntryList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for (i, entry) in self.entries.iter().enumerate() {
            let y = area.y + i as u16;
            if y < area.bottom() {
                buf.set_string(area.x, y, &entry.title, Style::default());
            }
        }
    }
}

// Custom trait for our navigation
trait Navigable {
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn select_current(&self) -> Option<&Entry>;
}

impl Navigable for EntryTree {
    fn move_up(&mut self) {
        // Implementation
    }
    // ... other methods
}
```

### 5. Collections

#### Common Collections in Our Project

```rust
use std::collections::{HashMap, BTreeMap, HashSet};

pub struct NavigationState {
    // BTreeMap for sorted years/months
    pub tree: BTreeMap<u32, BTreeMap<u32, Vec<Entry>>>,

    // HashSet for tracking expanded nodes
    pub expanded: HashSet<String>,

    // Vec for ordered lists
    pub navigation_stack: Vec<EntryPath>,
}

// Working with collections
impl NavigationState {
    pub fn add_entry(&mut self, entry: Entry) {
        let year = entry.date.year() as u32;
        let month = entry.date.month();

        self.tree
            .entry(year)
            .or_insert_with(BTreeMap::new)
            .entry(month)
            .or_insert_with(Vec::new)
            .push(entry);
    }

    pub fn find_entries_in_month(&self, year: u32, month: u32) -> Option<&Vec<Entry>> {
        self.tree.get(&year)?.get(&month)
    }
}
```

#### Iterator Patterns

```rust
// Functional programming with iterators
impl EntryTree {
    pub fn all_entries(&self) -> impl Iterator<Item = &Entry> {
        self.tree
            .values()                    // Get all years
            .flat_map(|year| year.values()) // Get all months
            .flat_map(|entries| entries.iter()) // Get all entries
    }

    pub fn entries_in_year(&self, year: u32) -> impl Iterator<Item = &Entry> {
        self.tree
            .get(&year)
            .into_iter()
            .flat_map(|months| months.values())
            .flat_map(|entries| entries.iter())
    }

    pub fn find_entry(&self, date: &NaiveDate) -> Option<&Entry> {
        self.all_entries()
            .find(|entry| entry.date == *date)
    }
}
```

### 6. Modules and Visibility

#### Module Structure

```rust
// src/lib.rs or src/main.rs
pub mod app;
pub mod ui;
pub mod navigation;
pub mod editor;
pub mod storage;

// src/navigation/mod.rs
pub mod tree;
pub mod state;

pub use tree::EntryTree;
pub use state::NavigationState;

// src/navigation/tree.rs
use crate::storage::Entry;  // Use from another module

pub struct EntryTree {
    // pub means public to users of this module
    pub years: BTreeMap<u32, Year>,

    // private field, only accessible within this module
    cache: HashMap<String, Entry>,
}

impl EntryTree {
    // Public method
    pub fn new() -> Self { /* ... */ }

    // Private method
    fn rebuild_cache(&mut self) { /* ... */ }
}
```

### 7. Lifetimes (You'll encounter these with ratatui)

#### Basic Lifetime Annotations

```rust
// ratatui widgets often need lifetimes
pub struct EntryWidget<'a> {
    entry: &'a Entry,
}

impl<'a> Widget for EntryWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // self.entry is valid for the lifetime 'a
        buf.set_string(area.x, area.y, &self.entry.title, Style::default());
    }
}

// Using the widget
let entry = Entry::new(/* ... */);
let widget = EntryWidget { entry: &entry };
// widget can only be used while entry is alive
```

## Practical Learning Exercises

### Exercise 1: Basic Entry Management

```rust
// Create a simple entry manager
struct EntryManager {
    entries: Vec<Entry>,
}

impl EntryManager {
    pub fn new() -> Self { /* implement */ }
    pub fn add_entry(&mut self, entry: Entry) { /* implement */ }
    pub fn find_entry(&self, date: &NaiveDate) -> Option<&Entry> { /* implement */ }
    pub fn remove_entry(&mut self, date: &NaiveDate) -> Option<Entry> { /* implement */ }
}

// Test your implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_management() {
        let mut manager = EntryManager::new();
        let entry = Entry::new(NaiveDate::from_ymd(2025, 3, 15));

        manager.add_entry(entry);
        assert!(manager.find_entry(&NaiveDate::from_ymd(2025, 3, 15)).is_some());
    }
}
```

### Exercise 2: Event Handling

```rust
#[derive(Debug)]
pub enum AppEvent {
    KeyPress(char),
    NavigateUp,
    NavigateDown,
    EnterEditMode,
    SaveEntry,
    Quit,
}

pub struct EventHandler {
    pub should_quit: bool,
}

impl EventHandler {
    pub fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => self.should_quit = true,
            AppEvent::KeyPress('q') => self.should_quit = true,
            // Implement other events
            _ => {}
        }
        Ok(())
    }
}
```

## Common Pitfalls and Solutions

### 1. Fighting the Borrow Checker

```rust
// ❌ Borrow checker error
let entry = &mut self.entries[0];
self.update_status(); // Error: can't borrow self while entry is borrowed

// ✅ Solution: limit scope of borrow
{
    let entry = &mut self.entries[0];
    entry.modify();
} // borrow ends here
self.update_status(); // Now this works
```

### 2. String vs &str

```rust
// Use String for owned data
pub struct Entry {
    pub content: String,  // Owned
}

// Use &str for borrowed data
pub fn process_content(content: &str) { // Borrowed
    // Work with content
}

// Converting between them
let owned: String = "hello".to_string();
let borrowed: &str = &owned;
let owned_again: String = borrowed.to_owned();
```

### 3. Option and Result Handling

```rust
// Chaining operations
let result = self.navigation
    .selected_entry()        // Returns Option<&Entry>
    .ok_or_else(|| eyre::eyre!("No entry selected"))?  // Convert to Result
    .load_content()?;        // Returns Result<String>

// Using map and and_then
let content_length = self.navigation
    .selected_entry()
    .map(|entry| entry.content.len())
    .unwrap_or(0);
```

## Debugging Tips

### 1. Use Debug Trait

```rust
#[derive(Debug)]
pub struct Entry {
    pub date: NaiveDate,
    pub content: String,
}

// Then you can:
println!("{:?}", entry);
dbg!(&entry);  // Even better for debugging
```

### 2. Use the Compiler

```rust
// Let the compiler infer types, then ask it what they are
let result = some_complex_operation();
let _: () = result;  // Compiler will tell you the actual type
```

### 3. Start Simple

```rust
// Instead of this complex version immediately:
pub fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
    match (self.mode, event.code, event.modifiers) {
        // Complex pattern matching...
    }
}

// Start with this:
pub fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
    match event.code {
        KeyCode::Char('q') => self.should_quit = true,
        _ => {}
    }
    Ok(())
}
```

## Next Steps

1. **Read through this guide** to get familiar with concepts
2. **Start with Phase 1** of the implementation plan
3. **Don't worry about perfection** - focus on getting things working
4. **Use the Rust compiler** as your learning partner - its error messages are excellent
5. **Test frequently** - small, working pieces are better than complex, broken ones

Remember: Every Rust developer has fought the borrow checker. It gets easier with practice!
