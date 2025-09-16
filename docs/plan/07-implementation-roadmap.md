# Implementation Roadmap

## Overview

This roadmap provides a week-by-week implementation plan for building the Engineering Journal app. It's designed for someone learning Rust while building a real application.

## Pre-Implementation (Day 0)

### Setup and Preparation

- [ ] Read through all planning documents
- [ ] Set up development environment
- [ ] Create initial Git repository
- [ ] Bookmark key resources

**Resources to Bookmark:**

- [The Rust Book](https://doc.rust-lang.org/book/)
- [ratatui Documentation](https://docs.rs/ratatui/)
- [color-eyre Documentation](https://docs.rs/color-eyre/)
- [chrono Documentation](https://docs.rs/chrono/)

## Week 1: Foundation (Days 1-7)

### Day 1: Project Setup

**Goal**: Get basic project structure working

**Tasks**:

- [ ] Update Cargo.toml with dependencies
- [ ] Create basic module structure
- [ ] Set up color-eyre error handling
- [ ] Create "Hello World" with basic ratatui

**Expected Output**: A program that opens a terminal UI and displays "Hello World"

**Code Milestone**:

```rust
// main.rs should successfully run and show a basic TUI
use color_eyre::Result;
use ratatui::prelude::*;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Engineering Journal - Setup Complete!");
    Ok(())
}
```

### Day 2-3: Basic Data Structures

**Goal**: Define core data types

**Tasks**:

- [ ] Create `Entry` struct with basic fields
- [ ] Implement date parsing with chrono
- [ ] Create `EntryPath` for YYYYMMDD format
- [ ] Add basic file path generation
- [ ] Write unit tests for data structures

**Code Milestone**:

```rust
// Should pass these tests
#[test]
fn test_entry_creation() {
    let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
    let entry = Entry::new(date);
    assert_eq!(entry.date, date);
}

#[test]
fn test_date_parsing() {
    let result = parse_entry_date("20250315");
    assert!(result.is_ok());
}
```

### Day 4-5: File Operations

**Goal**: Basic CRUD for entries

**Tasks**:

- [ ] Implement entry saving to filesystem
- [ ] Implement entry loading from filesystem
- [ ] Create directory structure automatically
- [ ] Handle file I/O errors with color-eyre
- [ ] Test with temporary directories

**Code Milestone**:

```bash
# Should be able to run these CLI commands
cargo run -- create 20250315 "My first entry"
cargo run -- read 20250315
cargo run -- list
```

### Day 6-7: Basic CLI Interface

**Goal**: Command-line interface working

**Tasks**:

- [ ] Parse command line arguments
- [ ] Implement basic CRUD commands
- [ ] Add help text and error messages
- [ ] Test all CLI operations
- [ ] Prepare for TUI implementation

**Learning Focus This Week**:

- Basic Rust syntax and ownership
- Working with external crates
- File I/O and error handling
- Unit testing in Rust

## Week 2: Basic TUI (Days 8-14)

### Day 8-9: Terminal Setup

**Goal**: Basic ratatui application loop

**Tasks**:

- [ ] Set up terminal initialization/cleanup
- [ ] Create basic event loop
- [ ] Handle keyboard input
- [ ] Implement graceful shutdown (q key)
- [ ] Basic layout (split screen)

**Code Milestone**:

```rust
// Should have working TUI that:
// - Starts up cleanly
// - Responds to 'q' key to quit
// - Shows basic layout with two panes
```

### Day 10-11: App State Management

**Goal**: Core application state

**Tasks**:

- [ ] Create `App` struct with modes
- [ ] Implement mode switching (Navigation/Edit)
- [ ] Basic event routing based on mode
- [ ] Simple status bar showing current mode
- [ ] Handle ESC key for mode switching

**Code Milestone**:

```rust
// Should be able to switch between modes
// - Start in Navigation mode
// - Press Enter → Edit mode
// - Press ESC → Navigation mode
// - Status bar shows current mode
```

### Day 12-14: Simple List View

**Goal**: Display and navigate entries

**Tasks**:

- [ ] Create simple list widget
- [ ] Load and display existing entries
- [ ] Implement basic navigation (j/k keys)
- [ ] Show selected entry content in right pane
- [ ] Add visual selection indicator

**Code Milestone**:

```rust
// Should have working navigation:
// - List shows all entries
// - j/k keys move selection
// - Selected entry content displays
// - Visual feedback for selection
```

**Learning Focus This Week**:

- ratatui basics (layouts, widgets, events)
- Pattern matching on events
- Borrowing and mutability with app state

## Week 3: Navigation System (Days 15-21)

### Day 15-16: Tree Data Structure

**Goal**: Hierarchical entry organization

**Tasks**:

- [ ] Design tree structure (Year/Month/Entry)
- [ ] Build tree from entry list
- [ ] Implement tree traversal
- [ ] Add expand/collapse state tracking
- [ ] Test tree operations

**Code Milestone**:

```rust
// Should have working tree structure:
let tree = EntryTree::from_entries(entries);
assert_eq!(tree.years.len(), 2); // 2024, 2025
assert_eq!(tree.years[&2025].months.len(), 3); // Jan, Mar, Apr
```

### Day 17-18: Tree Navigation Logic

**Goal**: Navigate through tree hierarchy

**Tasks**:

- [ ] Implement hjkl navigation
- [ ] Add left/right for expand/collapse
- [ ] Handle navigation between levels
- [ ] Implement selection state management
- [ ] Add breadcrumb display

**Code Milestone**:

```rust
// Navigation should work:
// h/l: expand/collapse or move between levels
// j/k: move within current level
// Breadcrumb shows: 2025 > March > 15
```

### Day 19-21: Tree UI Implementation

**Goal**: Visual tree in left pane

**Tasks**:

- [ ] Render tree with proper indentation
- [ ] Add expand/collapse indicators (▼▶)
- [ ] Implement visual selection
- [ ] Show entry indicators (●•)
- [ ] Handle window resizing

**Code Milestone**:

```
Tree should display like:
▼ 2025
  ▼ March
    • 01
    ● 15  (selected)
    • 28
  ▶ April
▶ 2024
```

**Learning Focus This Week**:

- Complex data structures (BTreeMap, HashSet)
- Working with collections and iterators
- State management patterns

## Week 4: Editor Integration (Days 22-28)

### Day 22-23: Basic Text Editor

**Goal**: Simple text editing capability

**Tasks**:

- [ ] Create `EditorState` struct
- [ ] Implement cursor positioning
- [ ] Add character insertion/deletion
- [ ] Handle line operations
- [ ] Basic cursor movement (arrow keys)

**Code Milestone**:

```rust
// Should be able to:
// - Insert characters at cursor
// - Delete characters with backspace
// - Move cursor with arrow keys
// - Handle newlines
```

### Day 24-25: Editor UI

**Goal**: Visual text editor in right pane

**Tasks**:

- [ ] Render text with cursor
- [ ] Show cursor position visually
- [ ] Handle text scrolling
- [ ] Display line numbers (optional)
- [ ] Show dirty state indicator

**Code Milestone**:

```
Editor should show:
┌─ Editor - 20250315.md * ─┐
│Line 1 of content█        │
│Line 2 of content         │
│                          │
└──────────────────────────┘
```

### Day 26-28: Mode Integration

**Goal**: Smooth navigation ↔ edit transitions

**Tasks**:

- [ ] Load entry content into editor
- [ ] Save editor content to file (Ctrl+S)
- [ ] Handle unsaved changes warnings
- [ ] Return to navigation on ESC
- [ ] Update tree view after saves

**Code Milestone**:

```
Complete workflow should work:
1. Navigate to entry (or create new)
2. Press Enter → Edit mode with content
3. Edit text, see cursor
4. Ctrl+S to save
5. ESC to return to navigation
```

**Learning Focus This Week**:

- String manipulation in Rust
- Working with mutable state
- File I/O integration with UI

## Week 5: Entry Management (Days 29-35)

### Day 29-30: Entry Creation

**Goal**: Create entries from UI

**Tasks**:

- [ ] Implement 'n' key for today's entry
- [ ] Add 'c' key with date prompt
- [ ] Create prompt input widget
- [ ] Validate date input
- [ ] Create files and update tree

**Code Milestone**:

```
Entry creation should work:
- Press 'n' → creates today's entry, enters edit mode
- Press 'c' → shows prompt for date
- Type "20250315" → creates entry for that date
- Invalid dates show error message
```

### Day 31-32: Entry Deletion

**Goal**: Delete entries safely

**Tasks**:

- [ ] Add 'd' key for deletion
- [ ] Implement confirmation prompt
- [ ] Delete file and update tree
- [ ] Handle deletion errors gracefully
- [ ] Update navigation after deletion

**Code Milestone**:

```
Deletion should work:
- Press 'd' on selected entry
- Shows: "Delete entry for 20250315? (y/N)"
- Press 'y' → deletes file and updates tree
- Press 'n' or ESC → cancels operation
```

### Day 33-35: Prompt System

**Goal**: Reusable prompt widget

**Tasks**:

- [ ] Create generic prompt widget
- [ ] Handle text input in prompts
- [ ] Add input validation
- [ ] Support different prompt types
- [ ] Polish prompt UI

**Code Milestone**:

```
Prompt system should handle:
- Date input with validation
- Yes/No confirmations
- Error messages
- Clean transitions back to main UI
```

**Learning Focus This Week**:

- User input handling
- State machine patterns
- Input validation techniques

## Week 6: Polish and Testing (Days 36-42)

### Day 36-37: Error Handling

**Goal**: Robust error handling

**Tasks**:

- [ ] Improve error messages throughout
- [ ] Handle edge cases gracefully
- [ ] Add user-friendly error display
- [ ] Test error scenarios
- [ ] Add recovery mechanisms

### Day 38-39: Performance

**Goal**: Smooth performance

**Tasks**:

- [ ] Optimize tree operations
- [ ] Implement lazy loading for large content
- [ ] Profile performance with many entries
- [ ] Optimize UI rendering
- [ ] Add loading indicators if needed

### Day 40-42: Final Testing

**Goal**: Production-ready application

**Tasks**:

- [ ] Comprehensive manual testing
- [ ] Test with real data (100+ entries)
- [ ] Test edge cases and error conditions
- [ ] Performance testing
- [ ] Documentation and cleanup

**Final Milestone**:

```
Complete application should:
✅ Navigate smoothly through hundreds of entries
✅ Create/edit/delete entries reliably
✅ Handle errors gracefully
✅ Start up quickly
✅ Feel responsive and polished
```

## Daily Development Routine

### Morning Routine (15 minutes)

```bash
# Pull latest changes
git pull

# Check everything still works
cargo check
cargo test

# Review today's goals
# Read relevant docs for today's tasks
```

### Development Session

1. **Focus on one task at a time**
2. **Write tests before implementation** (when possible)
3. **Commit frequently** with descriptive messages
4. **Test changes immediately**

### End of Day (10 minutes)

```bash
# Clean up code
cargo fmt
cargo clippy

# Run full test suite
cargo test

# Commit progress
git add .
git commit -m "Day X: Implemented Y feature"

# Plan tomorrow's work
```

## Weekly Reviews

### End of Each Week

1. **Demo the current progress** to yourself
2. **Review what you learned** about Rust
3. **Identify any struggles** and plan how to address them
4. **Adjust timeline** if needed (it's okay to take longer!)
5. **Celebrate progress** - building a real app is a big achievement!

## Troubleshooting Guide

### When Stuck on Rust Concepts

1. **Read the compiler error carefully** - Rust's compiler is very helpful
2. **Start with .clone()** if ownership is confusing, optimize later
3. **Break the problem down** into smaller pieces
4. **Use println! debugging** liberally
5. **Ask for help** in Rust community forums

### When Stuck on ratatui

1. **Check the examples** in the ratatui repository
2. **Start with simple widgets** before complex layouts
3. **Use TestBackend** for testing UI components
4. **Debug rendering** by checking buffer contents

### When Stuck on Architecture

1. **Keep it simple** - don't over-engineer
2. **Make it work first**, then make it pretty
3. **Copy patterns** from the examples in this plan
4. **Refactor gradually** as you learn better patterns

## Success Metrics

### Technical Metrics

- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code is formatted (cargo fmt)
- [ ] App starts in <1 second
- [ ] Can handle 100+ entries smoothly

### Learning Metrics

- [ ] Comfortable with basic Rust ownership
- [ ] Can read and fix compiler errors
- [ ] Understanding of ratatui patterns
- [ ] Can write simple tests
- [ ] Knows how to debug Rust programs

### User Experience Metrics

- [ ] All key bindings work as expected
- [ ] No crashes during normal use
- [ ] Error messages are helpful
- [ ] UI feels responsive
- [ ] Navigation is intuitive

## Beyond Week 6: Future Enhancements

Once you have the MVP working, consider these additions:

- [ ] Search functionality across entries
- [ ] Export entries to different formats
- [ ] Configuration file for customization
- [ ] Vim-like command mode
- [ ] Entry templates
- [ ] Tags and categorization
- [ ] Statistics and analytics

Remember: The goal is to learn Rust while building something useful. Don't worry about building the perfect application - focus on learning and making steady progress!
