# Implementation Phases

## Overview

This document breaks down the implementation into manageable phases, designed for someone learning Rust. Each phase builds on the previous one and introduces new Rust concepts gradually.

## Phase 1: Foundation (Week 1-2)

**Goal**: Basic project structure and file operations
**Rust Concepts**: Structs, enums, basic error handling

### Milestones

#### 1.1 Project Setup

- [ ] Update Cargo.toml with dependencies
- [ ] Create basic module structure
- [ ] Set up main.rs with basic color-eyre setup
- [ ] Create simple "Hello World" with ratatui

**Code Example**:

```rust
// main.rs
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Engineering Journal v0.1.0");
    Ok(())
}
```

#### 1.2 Basic Data Structures

- [ ] Define `Entry`, `EntryPath` structs
- [ ] Implement basic date parsing with chrono
- [ ] Create directory structure utilities
- [ ] Test basic file operations

**Learning Focus**:

- Rust structs and implementation blocks
- Using external crates (chrono)
- Basic error handling with `?` operator

#### 1.3 File System Operations

- [ ] Create entry directory structure
- [ ] Implement basic CRUD operations for entries
- [ ] Handle file I/O errors gracefully
- [ ] Add unit tests for file operations

**Code Example**:

```rust
// storage/entry.rs
use chrono::NaiveDate;
use std::path::PathBuf;
use color_eyre::Result;

#[derive(Debug, Clone)]
pub struct Entry {
    pub date: NaiveDate,
    pub path: PathBuf,
    pub content: Option<String>,
}

impl Entry {
    pub fn new(date: NaiveDate) -> Self {
        let path = Self::path_for_date(&date);
        Self {
            date,
            path,
            content: None,
        }
    }

    pub fn load_content(&mut self) -> Result<&str> {
        if self.content.is_none() {
            self.content = Some(std::fs::read_to_string(&self.path)?);
        }
        Ok(self.content.as_ref().unwrap())
    }
}
```

**Deliverable**: CLI that can create, read, and list entries (no UI yet)

## Phase 2: Basic UI (Week 2-3)

**Goal**: Simple ratatui interface with basic navigation
**Rust Concepts**: Traits, pattern matching, lifetimes

### Milestones

#### 2.1 Basic ratatui Setup

- [ ] Create terminal initialization/cleanup
- [ ] Set up event loop
- [ ] Handle basic keyboard input
- [ ] Implement graceful shutdown

**Code Example**:

```rust
// main.rs
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
```

#### 2.2 Basic App State

- [ ] Create `App` struct with basic state
- [ ] Implement mode switching (Navigation vs Edit)
- [ ] Add basic event handling
- [ ] Create simple UI layout

**Learning Focus**:

- Ownership and borrowing with app state
- Pattern matching on events and modes
- Basic ratatui widget usage

#### 2.3 Simple List View

- [ ] Display list of existing entries
- [ ] Implement basic navigation (j/k or up/down)
- [ ] Show selected entry content
- [ ] Add status bar with key hints

**Deliverable**: Basic TUI that shows entries and allows navigation

## Phase 3: Navigation System (Week 3-4)

**Goal**: Hierarchical tree navigation
**Rust Concepts**: Collections, iterators, more complex state management

### Milestones

#### 3.1 Tree Data Structure

- [ ] Build hierarchical entry tree
- [ ] Implement tree traversal logic
- [ ] Add expand/collapse functionality
- [ ] Handle empty months/years gracefully

**Code Example**:

```rust
// navigation/tree.rs
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct EntryTree {
    pub years: BTreeMap<u32, Year>,
}

impl EntryTree {
    pub fn from_entries(entries: Vec<Entry>) -> Self {
        let mut tree = Self {
            years: BTreeMap::new(),
        };

        for entry in entries {
            tree.insert_entry(entry);
        }

        tree
    }

    pub fn navigate(&self, direction: NavDirection) -> Option<EntryPath> {
        // Tree navigation logic
    }
}
```

#### 3.2 Tree Navigation UI

- [ ] Render tree with expand/collapse indicators
- [ ] Implement hjkl navigation
- [ ] Add breadcrumb navigation
- [ ] Handle tree state updates

**Learning Focus**:

- Working with BTreeMap and other collections
- Iterator patterns in Rust
- Mutable vs immutable references

#### 3.3 Enhanced Key Bindings

- [ ] Add all vim-style navigation keys
- [ ] Implement arrow key alternatives
- [ ] Add space for expand/collapse
- [ ] Handle edge cases (empty tree, etc.)

**Deliverable**: Full tree navigation working with real file system

## Phase 4: Editor Integration (Week 4-5)

**Goal**: Built-in text editor
**Rust Concepts**: String manipulation, more complex state

### Milestones

#### 4.1 Basic Text Editor

- [ ] Create editor state management
- [ ] Implement cursor movement
- [ ] Add basic text insertion/deletion
- [ ] Handle line-based editing

**Code Example**:

```rust
// editor/text.rs
#[derive(Debug)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub dirty: bool,
}

impl EditorState {
    pub fn insert_char(&mut self, ch: char) {
        if self.cursor_line < self.lines.len() {
            let line = &mut self.lines[self.cursor_line];
            line.insert(self.cursor_col, ch);
            self.cursor_col += 1;
            self.dirty = true;
        }
    }

    pub fn delete_char(&mut self) {
        // Implementation
    }
}
```

#### 4.2 Mode Switching

- [ ] Implement smooth navigation ↔ edit mode switching
- [ ] Handle ESC key properly
- [ ] Save on Ctrl+S
- [ ] Show mode in status bar

#### 4.3 File Integration

- [ ] Load entry content into editor
- [ ] Save editor content to file
- [ ] Handle unsaved changes warnings
- [ ] Create new entries from editor

**Deliverable**: Working editor that can modify entry content

## Phase 5: Entry Management (Week 5-6)

**Goal**: Full CRUD operations
**Rust Concepts**: Advanced error handling, user input

### Milestones

#### 5.1 Entry Creation

- [ ] Implement `n` key for today's entry
- [ ] Add `c` key with date prompt
- [ ] Handle date input validation
- [ ] Create file and directory structure

#### 5.2 Entry Deletion

- [ ] Add `d` key for deletion
- [ ] Implement confirmation prompt
- [ ] Handle file deletion
- [ ] Update tree state after deletion

#### 5.3 Prompt System

- [ ] Create reusable prompt widget
- [ ] Handle text input in prompts
- [ ] Add validation and error messages
- [ ] Smooth prompt → navigation transitions

**Deliverable**: Complete CRUD functionality

## Phase 6: Polish and Features (Week 6+)

**Goal**: Production-ready application
**Rust Concepts**: Performance, error handling, testing

### Milestones

#### 6.1 Error Handling

- [ ] Improve error messages
- [ ] Handle edge cases gracefully
- [ ] Add logging for debugging
- [ ] Test error scenarios

#### 6.2 Performance

- [ ] Lazy load entry content
- [ ] Optimize tree operations
- [ ] Handle large numbers of entries
- [ ] Add benchmarks

#### 6.3 Testing

- [ ] Unit tests for all modules
- [ ] Integration tests
- [ ] Test with real data
- [ ] Performance tests

#### 6.4 Documentation

- [ ] Add inline documentation
- [ ] Create user manual
- [ ] Document architecture decisions
- [ ] Add examples

**Deliverable**: Production-ready application

## Learning Checkpoints

### After Phase 1

- [ ] Comfortable with basic Rust syntax
- [ ] Understanding ownership basics
- [ ] Can handle simple errors with `?`
- [ ] Know how to work with external crates

### After Phase 3

- [ ] Understand borrowing and lifetimes
- [ ] Can work with complex data structures
- [ ] Comfortable with pattern matching
- [ ] Understand module organization

### After Phase 6

- [ ] Proficient with Rust error handling
- [ ] Can design and implement complex applications
- [ ] Understand performance considerations
- [ ] Can write tests and documentation

## Daily Development Workflow

```bash
# Start each day
cargo check          # Quick syntax check
cargo clippy         # Linting
cargo test           # Run tests

# During development
cargo run            # Test your changes
RUST_BACKTRACE=1 cargo run  # Debug issues

# End of day
cargo fmt            # Format code
git add . && git commit -m "Phase X.Y: milestone description"
```

## Getting Help

### When Stuck

1. **Compiler errors**: Read them carefully, Rust's compiler is very helpful
2. **Ownership issues**: Start with cloning, optimize later
3. **ratatui questions**: Check the examples in their repository
4. **General Rust**: The Rust Book (https://doc.rust-lang.org/book/)

### Useful Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [ratatui examples](https://github.com/ratatui-org/ratatui/tree/main/examples)
- [color-eyre documentation](https://docs.rs/color-eyre/)

Remember: Don't try to make everything perfect in the first implementation. Get it working, then refactor!
