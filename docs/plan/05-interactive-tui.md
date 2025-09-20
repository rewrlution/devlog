# Step 5: Interactive TUI (Terminal User Interface)

## Overview

Implement the interactive terminal interface with tree navigation for `devlog list --interactive`. This step is broken down into manageable chunks: data structures, UI components, event handling, and application logic.

## Part A: Core Data Structures (`src/tui/data.rs`)

First, let's define the core data structures for our TUI:

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub display_name: String,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
    pub is_entry: bool, // true if this is an actual entry file
}

impl TreeNode {
    pub fn new_folder(id: String, display_name: String) -> Self {
        Self {
            id,
            display_name,
            children: Vec::new(),
            is_expanded: false,
            is_entry: false,
        }
    }

    pub fn new_entry(id: String, display_name: String) -> Self {
        Self {
            id,
            display_name,
            children: Vec::new(),
            is_expanded: false,
            is_entry: true,
        }
    }
}

#[derive(PartialEq)]
pub enum Panel {
    Tree,
    Content,
}

pub struct AppState {
    pub tree_nodes: Vec<TreeNode>,
    pub flat_items: Vec<(String, usize, bool)>, // (display_text, indent_level, is_entry)
    pub current_panel: Panel,
    pub selected_entry_content: String,
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tree_nodes: Vec::new(),
            flat_items: Vec::new(),
            current_panel: Panel::Tree,
            selected_entry_content: String::new(),
            should_quit: false,
        }
    }
}
```

## Part B: Data Loading and Tree Building (`src/tui/tree_builder.rs`)

Handle loading entries and building the tree structure:

```rust
use crate::storage::Storage;
use crate::tui::data::{TreeNode, AppState};
use color_eyre::Result;
use std::collections::HashMap;

pub struct TreeBuilder {
    storage: Storage,
}

impl TreeBuilder {
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        Ok(Self { storage })
    }

    pub fn build_tree(&self) -> Result<Vec<TreeNode>> {
        let entry_ids = self.storage.list_entries()?;

        // Build year -> month -> day hierarchy
        let mut year_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        for entry_id in entry_ids {
            if entry_id.len() == 8 { // YYYYMMDD format
                let year = entry_id[0..4].to_string();
                let month = entry_id[4..6].to_string();
                let day = entry_id.clone();

                year_map
                    .entry(year)
                    .or_insert_with(HashMap::new)
                    .entry(month)
                    .or_insert_with(Vec::new)
                    .push(day);
            }
        }

        // Convert to tree structure
        let mut tree_nodes = Vec::new();
        let mut years: Vec<_> = year_map.keys().collect();
        years.sort_by(|a, b| b.cmp(a)); // Newest first

        for year in years {
            let year_months = &year_map[year];
            let mut months: Vec<_> = year_months.keys().collect();
            months.sort_by(|a, b| b.cmp(a)); // Newest first

            let mut month_nodes = Vec::new();
            for month in months {
                let month_days = &year_months[month];
                let mut days = month_days.clone();
                days.sort_by(|a, b| b.cmp(a)); // Newest first

                let day_nodes: Vec<TreeNode> = days
                    .into_iter()
                    .map(|day| TreeNode::new_entry(day.clone(), format_entry_display(&day)))
                    .collect();

                month_nodes.push(TreeNode {
                    id: format!("{}{}", year, month),
                    display_name: format!("{}-{}", year, month),
                    children: day_nodes,
                    is_expanded: false,
                    is_entry: false,
                });
            }

            tree_nodes.push(TreeNode {
                id: year.clone(),
                display_name: year.clone(),
                children: month_nodes,
                is_expanded: false,
                is_entry: false,
            });
        }

        Ok(tree_nodes)
    }

    pub fn get_storage(&self) -> &Storage {
        &self.storage
    }
}

fn format_entry_display(entry_id: &str) -> String {
    if entry_id.len() == 8 {
        format!("{}-{}-{}", &entry_id[0..4], &entry_id[4..6], &entry_id[6..8])
    } else {
        entry_id.to_string()
    }
}

pub fn flatten_tree(nodes: &[TreeNode]) -> Vec<(String, usize, bool)> {
    let mut flat_items = Vec::new();
    for node in nodes {
        flatten_node(node, 0, &mut flat_items);
    }
    flat_items
}

fn flatten_node(node: &TreeNode, indent: usize, flat_items: &mut Vec<(String, usize, bool)>) {
    let display_text = if node.is_entry {
        node.display_name.clone()
    } else if node.is_expanded {
        format!("📂 {}", node.display_name)
    } else {
        format!("📁 {}", node.display_name)
    };

    flat_items.push((display_text, indent, node.is_entry));

    if node.is_expanded {
        for child in &node.children {
            flatten_node(child, indent + 1, flat_items);
        }
    }
}
```

## Part C: UI Components (`src/tui/ui.rs`)

Handle the rendering of UI components:

```rust
use crate::tui::data::{AppState, Panel};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub struct UIRenderer;

impl UIRenderer {
    pub fn render(app_state: &AppState, tree_state: &mut ListState, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(f.size());

        Self::render_tree_panel(app_state, tree_state, f, chunks[0]);
        Self::render_content_panel(app_state, f, chunks[1]);
    }

    fn render_tree_panel(app_state: &AppState, tree_state: &mut ListState, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = app_state
            .flat_items
            .iter()
            .map(|(text, indent, is_entry)| {
                let indent_str = "  ".repeat(*indent);
                let style = if *is_entry {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Yellow)
                };

                ListItem::new(Line::from(Span::styled(
                    format!("{}{}", indent_str, text),
                    style,
                )))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Entries")
                    .border_style(if app_state.current_panel == Panel::Tree {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::LightBlue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, tree_state);
    }

    fn render_content_panel(app_state: &AppState, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(app_state.selected_entry_content.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Content")
                    .border_style(if app_state.current_panel == Panel::Content {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}
```

## Part D: Event Handling (`src/tui/events.rs`)

Handle keyboard input and events:

```rust
use crate::storage::Storage;
use crate::tui::data::{AppState, Panel, TreeNode};
use crate::tui::tree_builder::{flatten_tree};
use color_eyre::Result;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::widgets::ListState;
use std::io;

pub struct EventHandler {
    storage: Storage,
}

impl EventHandler {
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        Ok(Self { storage })
    }

    pub fn handle_key_event(
        &self,
        key_code: KeyCode,
        app_state: &mut AppState,
        tree_state: &mut ListState
    ) -> Result<()> {
        match key_code {
            KeyCode::Char('q') => app_state.should_quit = true,
            KeyCode::Tab => {
                app_state.current_panel = match app_state.current_panel {
                    Panel::Tree => Panel::Content,
                    Panel::Content => Panel::Tree,
                };
            }
            _ => {
                if app_state.current_panel == Panel::Tree {
                    self.handle_tree_navigation(key_code, app_state, tree_state)?;
                }
            }
        }
        Ok(())
    }

    fn handle_tree_navigation(
        &self,
        key_code: KeyCode,
        app_state: &mut AppState,
        tree_state: &mut ListState,
    ) -> Result<()> {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => {
                let selected = tree_state.selected().unwrap_or(0);
                if selected > 0 {
                    tree_state.select(Some(selected - 1));
                    self.update_content_panel(app_state, tree_state)?;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let selected = tree_state.selected().unwrap_or(0);
                if selected < app_state.flat_items.len().saturating_sub(1) {
                    tree_state.select(Some(selected + 1));
                    self.update_content_panel(app_state, tree_state)?;
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                self.toggle_node(app_state, tree_state)?;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.collapse_node(app_state, tree_state)?;
            }
            KeyCode::Char('e') => {
                self.edit_current_entry(app_state, tree_state)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn toggle_node(&self, app_state: &mut AppState, tree_state: &mut ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if selected < app_state.flat_items.len() {
                let (_, _, is_entry) = &app_state.flat_items[selected];
                if !is_entry {
                    // Find and toggle the corresponding tree node
                    self.toggle_node_recursive(&mut app_state.tree_nodes, selected, &mut 0)?;
                    app_state.flat_items = flatten_tree(&app_state.tree_nodes);
                } else {
                    // It's an entry, load its content
                    self.update_content_panel(app_state, tree_state)?;
                }
            }
        }
        Ok(())
    }

    fn toggle_node_recursive(
        &self,
        nodes: &mut Vec<TreeNode>,
        target_index: usize,
        current_index: &mut usize,
    ) -> Result<bool> {
        for node in nodes {
            if *current_index == target_index && !node.is_entry {
                node.is_expanded = !node.is_expanded;
                return Ok(true);
            }
            *current_index += 1;

            if node.is_expanded {
                if self.toggle_node_recursive(&mut node.children, target_index, current_index)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn collapse_node(&self, app_state: &mut AppState, tree_state: &mut ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if selected < app_state.flat_items.len() {
                let (_, _, is_entry) = &app_state.flat_items[selected];
                if !is_entry {
                    let mut current_index = 0;
                    self.collapse_node_recursive(&mut app_state.tree_nodes, selected, &mut current_index)?;
                    app_state.flat_items = flatten_tree(&app_state.tree_nodes);
                }
            }
        }
        Ok(())
    }

    fn collapse_node_recursive(
        &self,
        nodes: &mut Vec<TreeNode>,
        target_index: usize,
        current_index: &mut usize,
    ) -> Result<bool> {
        for node in nodes {
            if *current_index == target_index && !node.is_entry && node.is_expanded {
                node.is_expanded = false;
                return Ok(true);
            }
            *current_index += 1;

            if node.is_expanded {
                if self.collapse_node_recursive(&mut node.children, target_index, current_index)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn update_content_panel(&self, app_state: &mut AppState, tree_state: &ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if selected < app_state.flat_items.len() {
                let (display_text, _, is_entry) = &app_state.flat_items[selected];
                if *is_entry {
                    // Extract entry ID from display text - convert back to YYYYMMDD format
                    let entry_id = display_text.replace("-", "");
                    if let Ok(entry) = self.storage.load_entry(&entry_id) {
                        app_state.selected_entry_content = format!(
                            "Entry: {}\nCreated: {}\nUpdated: {}\n\n---\n\n{}",
                            entry.id,
                            entry.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                            entry.updated_at.format("%Y-%m-%d %H:%M:%S UTC"),
                            entry.content
                        );
                    } else {
                        app_state.selected_entry_content = "Error loading entry".to_string();
                    }
                } else {
                    app_state.selected_entry_content = format!(
                        "Selected: {}\n\nPress 'l' or Enter to expand/collapse",
                        display_text
                    );
                }
            }
        }
        Ok(())
    }

    fn edit_current_entry(&self, app_state: &AppState, tree_state: &ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if selected < app_state.flat_items.len() {
                let (display_text, _, is_entry) = &app_state.flat_items[selected];
                if *is_entry {
                    // Convert display format back to YYYYMMDD
                    let entry_id = display_text.replace("-", "");

                    // Exit TUI mode temporarily
                    disable_raw_mode()?;
                    execute!(io::stdout(), LeaveAlternateScreen)?;

                    // Launch edit command
                    println!("Launching editor for entry: {}", entry_id);
                    crate::commands::edit::execute(entry_id)?;

                    // Re-enter TUI mode
                    enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen)?;
                }
            }
        }
        Ok(())
    }
}
```

## Part E: Main Application (`src/tui/app.rs`)

Tie everything together:

```rust
use crate::tui::data::AppState;
use crate::tui::events::EventHandler;
use crate::tui::tree_builder::{TreeBuilder, flatten_tree};
use crate::tui::ui::UIRenderer;
use color_eyre::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::ListState,
    Terminal,
};
use std::io;

pub struct App {
    app_state: AppState,
    tree_state: ListState,
    event_handler: EventHandler,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app_state = AppState::new();
        let tree_builder = TreeBuilder::new()?;

        // Build the tree
        app_state.tree_nodes = tree_builder.build_tree()?;
        app_state.flat_items = flatten_tree(&app_state.tree_nodes);

        let mut tree_state = ListState::default();

        // Select first item
        if !app_state.flat_items.is_empty() {
            tree_state.select(Some(0));
        }

        let event_handler = EventHandler::new()?;

        Ok(Self {
            app_state,
            tree_state,
            event_handler,
        })
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| UIRenderer::render(&self.app_state, &mut self.tree_state, f))?;

            if self.app_state.should_quit {
                break;
            }

            if let Event::Key(key) = event::read()? {
                self.event_handler.handle_key_event(
                    key.code,
                    &mut self.app_state,
                    &mut self.tree_state
                )?;
            }
        }
        Ok(())
    }
}

// Helper function to launch TUI
pub fn launch_tui() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = app.run(&mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
```

## Update TUI Module Declaration (`src/tui/mod.rs`)

```rust
pub mod app;
pub mod data;
pub mod events;
pub mod tree_builder;
pub mod ui;
```

## Update Commands Module (`src/commands/list.rs`)

```rust
use crate::storage::Storage;
use color_eyre::Result;

pub fn execute() -> Result<()> {
    let storage = Storage::new()?;
    let entries = storage.list_entries()?;

    if entries.is_empty() {
        println!("No entries found. Create your first entry with 'devlog new'");
        return Ok(());
    }

    println!("Recent entries (last 20):");
    println!();

    for (i, entry_id) in entries.iter().take(20).enumerate() {
        // Format YYYYMMDD as YYYY-MM-DD for display
        let formatted_id = if entry_id.len() == 8 {
            format!("{}-{}-{}", &entry_id[0..4], &entry_id[4..6], &entry_id[6..8])
        } else {
            entry_id.clone()
        };
        println!("  {}. {} ({})", i + 1, formatted_id, entry_id);
    }

    println!();
    println!("Commands:");
    println!("  devlog show <id>              - View an entry (use YYYYMMDD format)");
    println!("  devlog edit <id>              - Edit an entry (use YYYYMMDD format)");
    println!("  devlog list --interactive     - Launch TUI mode");

    Ok(())
}

pub fn execute_interactive() -> Result<()> {
    crate::tui::app::launch_tui()
}
```

## Update Main Library (`src/lib.rs`)

```rust
pub mod commands;
pub mod storage;
pub mod utils;
pub mod tui;
```

## Implementation Strategy

### Phase 1: Data Structures (Start Here)

1. **Create `src/tui/data.rs`** with core data structures
2. **Create `src/tui/tree_builder.rs`** with tree building logic
3. **Test tree building** with simple println debugging

### Phase 2: UI Components

1. **Create `src/tui/ui.rs`** with rendering logic
2. **Create basic TUI structure** without events
3. **Test UI rendering** with static data

### Phase 3: Event Handling

1. **Create `src/tui/events.rs`** with keyboard handling
2. **Implement navigation** and node toggling
3. **Test all TUI interactions**

### Phase 4: Integration

1. **Create `src/tui/app.rs`** to tie everything together
2. **Update commands and module declarations**
3. **Test complete TUI functionality**

## Key Rust Concepts Explained

- **Modules**: Each file is a separate module with specific responsibilities
- **Separation of Concerns**: UI, events, data, and logic are separate
- **State Management**: AppState holds all mutable application state
- **Event-Driven**: UI updates in response to keyboard events
- **Result Propagation**: Errors bubble up through the ? operator

## TUI Controls

- **Navigation**:
  - `↑`/`k` - Move up in tree
  - `↓`/`j` - Move down in tree
  - `→`/`l`/`Enter` - Expand folder or select entry
  - `←`/`h` - Collapse folder
  - `Tab` - Switch between tree and content panels
  - `e` - Edit current entry (launches editor)
  - `q` - Quit TUI

## File Naming Convention

- **Storage**: Files are stored as `YYYYMMDD.md` (e.g., `20240920.md`)
- **Display**: TUI shows entries as `YYYY-MM-DD` for readability
- **Commands**: CLI commands accept `YYYYMMDD` format (e.g., `devlog show 20240920`)

## Implementation Tasks

1. **Create module structure**:

   - `src/tui/mod.rs`
   - `src/tui/data.rs`
   - `src/tui/tree_builder.rs`
   - `src/tui/ui.rs`
   - `src/tui/events.rs`
   - `src/tui/app.rs`

2. **Update existing files**:

   - `src/commands/list.rs` for TUI integration
   - `src/lib.rs` to include tui module

3. **Test incrementally** after each phase

## Testing Your Implementation

```bash
# Create a few entries first
cargo run new
# Add content with YYYYMMDD format

# Test regular list
cargo run list

# Launch TUI
cargo run list --interactive

# Try all TUI navigation:
# - Use j/k to move up/down
# - Use l/Enter to expand years/months
# - Select an entry to view content
# - Press 'e' to edit an entry
# - Press 'q' to quit
```

## Troubleshooting

- **Date Format Issues**: Make sure the tree builder correctly parses YYYYMMDD format
- **Display Problems**: Check terminal supports Unicode symbols (📁📂)
- **Navigation Issues**: Verify tree flattening logic maintains correct structure
- **Editor Problems**: Ensure editor exits cleanly before returning to TUI

## Benefits of This Structure

- **Maintainable**: Each component has a single responsibility
- **Testable**: Individual modules can be tested separately
- **Extensible**: Easy to add new features like search or filtering
- **Readable**: Clear separation makes code easier to understand
- **Debuggable**: Issues can be isolated to specific modules

## Next Steps

Move to Step 4: Annotation Parsing (implement this after TUI is working) or continue with Step 6: Configuration and Polish.
