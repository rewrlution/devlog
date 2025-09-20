# Step 5: Interactive TUI (Terminal User Interface)

## Overview

Implement the interactive terminal interface with tree navigation for `devlog list --interactive`.

## TUI Application Structure (`src/tui/app.rs`)

```rust
use crate::storage::Storage;
use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub display_name: String,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
    pub is_entry: bool, // true if this is an actual entry file
}

pub struct App {
    storage: Storage,
    tree_nodes: Vec<TreeNode>,
    tree_state: ListState,
    flat_items: Vec<(String, usize, bool)>, // (display_text, indent_level, is_entry)
    current_panel: Panel,
    selected_entry_content: String,
    should_quit: bool,
}

#[derive(PartialEq)]
enum Panel {
    Tree,
    Content,
}

impl App {
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        let mut app = Self {
            storage,
            tree_nodes: Vec::new(),
            tree_state: ListState::default(),
            flat_items: Vec::new(),
            current_panel: Panel::Tree,
            selected_entry_content: String::new(),
            should_quit: false,
        };

        app.load_entries()?;
        app.flatten_tree();

        // Select first item
        if !app.flat_items.is_empty() {
            app.tree_state.select(Some(0));
        }

        Ok(app)
    }

    fn load_entries(&mut self) -> Result<()> {
        let entry_ids = self.storage.list_entries()?;

        // Build year -> month -> day hierarchy
        let mut year_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        for entry_id in entry_ids {
            let parts: Vec<&str> = entry_id.split('-').collect();
            if parts.len() == 3 {
                let year = parts[0].to_string();
                let month = format!("{}-{}", parts[0], parts[1]);
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
                    .map(|day| TreeNode {
                        id: day.clone(),
                        display_name: day,
                        children: Vec::new(),
                        is_expanded: false,
                        is_entry: true,
                    })
                    .collect();

                let month_display = month.split('-').nth(1).unwrap_or(month);
                month_nodes.push(TreeNode {
                    id: month.clone(),
                    display_name: format!("Month {}", month_display),
                    children: day_nodes,
                    is_expanded: false,
                    is_entry: false,
                });
            }

            self.tree_nodes.push(TreeNode {
                id: year.clone(),
                display_name: format!("Year {}", year),
                children: month_nodes,
                is_expanded: false,
                is_entry: false,
            });
        }

        Ok(())
    }

    fn flatten_tree(&mut self) {
        self.flat_items.clear();
        for node in &self.tree_nodes {
            self.flatten_node(node, 0);
        }
    }

    fn flatten_node(&mut self, node: &TreeNode, indent: usize) {
        let display_text = if node.is_entry {
            node.display_name.clone()
        } else if node.is_expanded {
            format!("📂 {}", node.display_name)
        } else {
            format!("📁 {}", node.display_name)
        };

        self.flat_items.push((display_text, indent, node.is_entry));

        if node.is_expanded {
            for child in &node.children {
                self.flatten_node(child, indent + 1);
            }
        }
    }

    pub fn run<B: ratatui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if self.should_quit {
                break;
            }

            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => {
                self.current_panel = match self.current_panel {
                    Panel::Tree => Panel::Content,
                    Panel::Content => Panel::Tree,
                };
            }
            _ => {
                if self.current_panel == Panel::Tree {
                    self.handle_tree_navigation(key)?;
                }
            }
        }
        Ok(())
    }

    fn handle_tree_navigation(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let selected = self.tree_state.selected().unwrap_or(0);
                if selected > 0 {
                    self.tree_state.select(Some(selected - 1));
                    self.update_content_panel()?;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let selected = self.tree_state.selected().unwrap_or(0);
                if selected < self.flat_items.len().saturating_sub(1) {
                    self.tree_state.select(Some(selected + 1));
                    self.update_content_panel()?;
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                self.toggle_node()?;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.collapse_node()?;
            }
            KeyCode::Char('e') => {
                self.edit_current_entry()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn toggle_node(&mut self) -> Result<()> {
        if let Some(selected) = self.tree_state.selected() {
            if selected < self.flat_items.len() {
                let (_, _, is_entry) = &self.flat_items[selected];
                if !is_entry {
                    // Find and toggle the corresponding tree node
                    self.toggle_node_recursive(&mut self.tree_nodes.clone(), selected, 0)?;
                    self.flatten_tree();
                } else {
                    // It's an entry, load its content
                    self.update_content_panel()?;
                }
            }
        }
        Ok(())
    }

    fn toggle_node_recursive(
        &mut self,
        nodes: &mut Vec<TreeNode>,
        target_index: usize,
        current_index: &mut usize,
    ) -> Result<bool> {
        for node in nodes {
            if *current_index == target_index && !node.is_entry {
                node.is_expanded = !node.is_expanded;

                // Update the actual tree
                self.update_tree_node(&node.id, node.is_expanded)?;
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

    fn update_tree_node(&mut self, node_id: &str, expanded: bool) -> Result<()> {
        self.update_tree_node_recursive(&mut self.tree_nodes, node_id, expanded)
    }

    fn update_tree_node_recursive(
        &mut self,
        nodes: &mut Vec<TreeNode>,
        node_id: &str,
        expanded: bool,
    ) -> Result<()> {
        for node in nodes {
            if node.id == node_id {
                node.is_expanded = expanded;
                return Ok(());
            }
            self.update_tree_node_recursive(&mut node.children, node_id, expanded)?;
        }
        Ok(())
    }

    fn collapse_node(&mut self) -> Result<()> {
        // Simple implementation: collapse current node if expanded
        if let Some(selected) = self.tree_state.selected() {
            if selected < self.flat_items.len() {
                let (_, _, is_entry) = &self.flat_items[selected];
                if !is_entry {
                    let mut current_index = 0;
                    self.collapse_node_recursive(&mut self.tree_nodes.clone(), selected, &mut current_index)?;
                    self.flatten_tree();
                }
            }
        }
        Ok(())
    }

    fn collapse_node_recursive(
        &mut self,
        nodes: &mut Vec<TreeNode>,
        target_index: usize,
        current_index: &mut usize,
    ) -> Result<bool> {
        for node in nodes {
            if *current_index == target_index && !node.is_entry && node.is_expanded {
                node.is_expanded = false;
                self.update_tree_node(&node.id, false)?;
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

    fn update_content_panel(&mut self) -> Result<()> {
        if let Some(selected) = self.tree_state.selected() {
            if selected < self.flat_items.len() {
                let (display_text, _, is_entry) = &self.flat_items[selected];
                if *is_entry {
                    // Extract entry ID from display text
                    let entry_id = display_text.clone();
                    if let Ok(entry) = self.storage.load_entry(&entry_id) {
                        self.selected_entry_content = format!(
                            "Entry: {}\nCreated: {}\nUpdated: {}\n\n---\n\n{}",
                            entry.id,
                            entry.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                            entry.updated_at.format("%Y-%m-%d %H:%M:%S UTC"),
                            entry.content
                        );
                    } else {
                        self.selected_entry_content = "Error loading entry".to_string();
                    }
                } else {
                    self.selected_entry_content = format!("Selected: {}\n\nPress 'l' or Enter to expand/collapse", display_text);
                }
            }
        }
        Ok(())
    }

    fn edit_current_entry(&mut self) -> Result<()> {
        if let Some(selected) = self.tree_state.selected() {
            if selected < self.flat_items.len() {
                let (display_text, _, is_entry) = &self.flat_items[selected];
                if *is_entry {
                    let entry_id = display_text.clone();

                    // Exit TUI mode temporarily
                    disable_raw_mode()?;
                    execute!(io::stdout(), LeaveAlternateScreen)?;

                    // Launch edit command
                    println!("Launching editor for entry: {}", entry_id);
                    crate::commands::edit::execute(entry_id)?;

                    // Re-enter TUI mode
                    enable_raw_mode()?;
                    execute!(io::stdout(), EnterAlternateScreen)?;

                    // Reload content
                    self.update_content_panel()?;
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(f.size());

        self.render_tree_panel(f, chunks[0]);
        self.render_content_panel(f, chunks[1]);
    }

    fn render_tree_panel(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
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
                    .border_style(if self.current_panel == Panel::Tree {
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

        f.render_stateful_widget(list, area, &mut self.tree_state);
    }

    fn render_content_panel(&self, f: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(self.selected_entry_content.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Content")
                    .border_style(if self.current_panel == Panel::Content {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    }),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
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
        println!("  {}. {}", i + 1, entry_id);
    }

    println!();
    println!("Commands:");
    println!("  devlog show <id>              - View an entry");
    println!("  devlog edit <id>              - Edit an entry");
    println!("  devlog list --interactive     - Launch TUI mode");

    Ok(())
}

pub fn execute_interactive() -> Result<()> {
    crate::tui::app::launch_tui()
}
```

## Update TUI Module Declaration (`src/tui/mod.rs`)

```rust
pub mod app;
```

## Update Main Library (`src/lib.rs`)

```rust
pub mod commands;
pub mod storage;
pub mod utils;
pub mod tui;
```

## Key Rust Concepts Explained

- **Enums**: Like unions but safer - `Panel::Tree` vs `Panel::Content`
- **Mutable references**: `&mut self` allows changing the struct
- **Pattern matching**: `match` handles all possible enum variants
- **Vec<T>**: Dynamic arrays (like ArrayList in Java)
- **Clone**: Make a copy of data (sometimes expensive, but simple)
- **Recursive functions**: Functions that call themselves (like tree traversal)

## TUI Controls

- **Navigation**:
  - `↑`/`k` - Move up
  - `↓`/`j` - Move down
  - `→`/`l`/`Enter` - Expand folder or select entry
  - `←`/`h` - Collapse folder
  - `Tab` - Switch between tree and content panels
  - `e` - Edit current entry (launches Vim)
  - `q` - Quit

## Implementation Tasks

1. **Create `src/tui/mod.rs`** and `src/tui/app.rs`
2. **Update `src/commands/list.rs`** to support TUI mode
3. **Update `src/lib.rs`** to include tui module
4. **Test the TUI** with `cargo run list --interactive`

## Testing Your TUI

```bash
# Create a few entries first
cargo run new
# Add content and save

# Launch TUI
cargo run list --interactive

# Try navigation:
# - Use j/k to move up/down
# - Use l/Enter to expand years/months
# - Select an entry to view content
# - Press 'e' to edit an entry
# - Press 'q' to quit
```

## Troubleshooting

- **Terminal issues**: Make sure your terminal supports ANSI colors
- **Vim not opening**: Check that vim is installed and in PATH
- **Display problems**: Try resizing terminal window
- **Crashes on edit**: Ensure temporary file permissions are correct

## Next Steps

Move to Step 6: Configuration and Polish to add config file support and final improvements.
