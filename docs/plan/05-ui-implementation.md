# UI/UX Implementation Guide

## Overview

This document focuses on the user interface implementation using ratatui, with practical examples and learning guidance for building the Engineering Journal's TUI.

## ratatui Fundamentals

### Core Concepts

#### 1. Terminal Setup

```rust
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

// Terminal setup and cleanup
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

pub fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
```

#### 2. Event Loop Pattern

```rust
pub fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, mut app: App) -> Result<()> {
    loop {
        // Render the UI
        terminal.draw(|f| ui::render(&mut app, f))?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key_event(key)?;

                if app.should_quit {
                    break;
                }
            }
        }
    }
    Ok(())
}
```

## Layout System

### Basic Layout

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    // Main layout: horizontal split
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Tree view (left)
            Constraint::Percentage(70), // Content view (right)
        ])
        .split(frame.size());

    // Render tree in left pane
    render_tree_view(app, frame, main_layout[0]);

    // Render content in right pane
    render_content_view(app, frame, main_layout[1]);

    // Status bar at bottom
    render_status_bar(app, frame);
}
```

### Advanced Layout with Status Bar

```rust
pub fn render(app: &mut App, frame: &mut Frame) {
    // Vertical split: main area + status bar
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),        // Main area (at least 3 lines)
            Constraint::Length(1),     // Status bar (exactly 1 line)
        ])
        .split(frame.size());

    // Horizontal split for main area
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Tree view
            Constraint::Percentage(70), // Content view
        ])
        .split(root_layout[0]);

    render_tree_view(app, frame, main_layout[0]);
    render_content_view(app, frame, main_layout[1]);
    render_status_bar(app, frame, root_layout[1]);
}
```

## Tree View Implementation

### Data Structure for UI

```rust
use ratatui::widgets::{List, ListItem, ListState};

pub struct TreeViewState {
    pub items: Vec<TreeItem>,
    pub list_state: ListState,
    pub expanded: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct TreeItem {
    pub label: String,
    pub path: String,
    pub level: usize,
    pub item_type: TreeItemType,
    pub has_children: bool,
}

#[derive(Debug, Clone)]
pub enum TreeItemType {
    Year(u32),
    Month(u32),
    Entry(NaiveDate),
}
```

### Building Tree Items

```rust
impl TreeViewState {
    pub fn from_entry_tree(tree: &EntryTree) -> Self {
        let mut items = Vec::new();
        let expanded = HashSet::new();

        for (year, months) in &tree.years {
            // Add year item
            items.push(TreeItem {
                label: format!("▼ {}", year),
                path: year.to_string(),
                level: 0,
                item_type: TreeItemType::Year(*year),
                has_children: !months.is_empty(),
            });

            // Add month items if year is expanded
            if expanded.contains(&year.to_string()) {
                for (month, entries) in months {
                    let month_name = month_name(*month);
                    let month_path = format!("{}/{}", year, month);

                    items.push(TreeItem {
                        label: format!("  ▼ {}", month_name),
                        path: month_path.clone(),
                        level: 1,
                        item_type: TreeItemType::Month(*month),
                        has_children: !entries.is_empty(),
                    });

                    // Add entry items if month is expanded
                    if expanded.contains(&month_path) {
                        for entry in entries {
                            items.push(TreeItem {
                                label: format!("    • {}", entry.date.day()),
                                path: format!("{}/{}", month_path, entry.date.day()),
                                level: 2,
                                item_type: TreeItemType::Entry(entry.date),
                                has_children: false,
                            });
                        }
                    }
                }
            }
        }

        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            items,
            list_state,
            expanded,
        }
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January", 2 => "February", 3 => "March",
        4 => "April", 5 => "May", 6 => "June",
        7 => "July", 8 => "August", 9 => "September",
        10 => "October", 11 => "November", 12 => "December",
        _ => "Unknown",
    }
}
```

### Rendering Tree View

```rust
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

pub fn render_tree_view(app: &mut App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app.tree_view
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if Some(i) == app.tree_view.list_state.selected() {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(&item.label, style)))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Navigation")
                .borders(Borders::ALL)
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        );

    frame.render_stateful_widget(list, area, &mut app.tree_view.list_state);
}
```

## Content View Implementation

### Text Editor Widget

```rust
use ratatui::{
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render_content_view(app: &mut App, frame: &mut Frame, area: Rect) {
    match app.mode {
        AppMode::Navigation => render_entry_preview(app, frame, area),
        AppMode::Edit => render_editor(app, frame, area),
        AppMode::Prompt(_) => render_prompt(app, frame, area),
    }
}

fn render_entry_preview(app: &App, frame: &mut Frame, area: Rect) {
    let content = if let Some(entry) = app.navigation.selected_entry() {
        entry.content.clone().unwrap_or_else(|| "Loading...".to_string())
    } else {
        "No entry selected".to_string()
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title("Entry Content")
                .borders(Borders::ALL)
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn render_editor(app: &App, frame: &mut Frame, area: Rect) {
    let title = format!(
        "Editor - {} {}",
        app.editor.current_entry
            .map(|e| e.to_string())
            .unwrap_or_else(|| "New Entry".to_string()),
        if app.editor.dirty { "*" } else { "" }
    );

    // Create lines with cursor indication
    let lines: Vec<Line> = app.editor.lines
        .iter()
        .enumerate()
        .map(|(line_idx, line_content)| {
            if line_idx == app.editor.cursor_line {
                // Show cursor position
                let (before, after) = line_content.split_at(
                    app.editor.cursor_col.min(line_content.len())
                );
                Line::from(vec![
                    Span::raw(before),
                    Span::styled("█", Style::default().bg(Color::White).fg(Color::Black)),
                    Span::raw(after),
                ])
            } else {
                Line::from(line_content.clone())
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
```

## Status Bar Implementation

### Dynamic Status Bar

```rust
pub fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let status_text = match app.mode {
        AppMode::Navigation => {
            "[hjkl/↑↓←→] Nav [Enter] Edit [n] New [c] Create [d] Del [q] Quit"
        }
        AppMode::Edit => {
            "[ESC] Nav Mode [Ctrl+S] Save [Ctrl+L] Refresh"
        }
        AppMode::Prompt(PromptType::CreateEntry) => {
            "Enter date (YYYYMMDD) or press ESC to cancel"
        }
        AppMode::Prompt(PromptType::DeleteConfirmation) => {
            "Delete entry? (y/N)"
        }
    };

    let paragraph = Paragraph::new(status_text)
        .style(Style::default().bg(Color::Blue).fg(Color::White));

    frame.render_widget(paragraph, area);
}
```

## Navigation Implementation

### Key Event Handling

```rust
impl App {
    pub fn handle_navigation_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.tree_view.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.tree_view.move_up(),
            KeyCode::Char('h') | KeyCode::Left => self.tree_view.collapse_current(),
            KeyCode::Char('l') | KeyCode::Right => self.tree_view.expand_current(),

            // Actions
            KeyCode::Enter => self.open_selected_entry()?,
            KeyCode::Char(' ') => self.tree_view.toggle_current(),
            KeyCode::Char('n') => self.create_today_entry()?,
            KeyCode::Char('c') => self.prompt_create_entry(),
            KeyCode::Char('d') => self.prompt_delete_entry(),
            KeyCode::Char('q') => self.should_quit = true,

            _ => {}
        }
        Ok(())
    }
}

impl TreeViewState {
    pub fn move_down(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn move_up(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn toggle_current(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(item) = self.items.get(i) {
                if item.has_children {
                    if self.expanded.contains(&item.path) {
                        self.expanded.remove(&item.path);
                    } else {
                        self.expanded.insert(item.path.clone());
                    }
                    // Rebuild items with new expanded state
                    // This would require access to the original tree data
                }
            }
        }
    }
}
```

## Prompt System Implementation

### Generic Prompt Widget

```rust
pub struct PromptState {
    pub message: String,
    pub input: String,
    pub cursor_pos: usize,
}

impl PromptState {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            input: String::new(),
            cursor_pos: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> PromptResult {
        match key.code {
            KeyCode::Enter => PromptResult::Submit(self.input.clone()),
            KeyCode::Esc => PromptResult::Cancel,
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.input.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
                PromptResult::Continue
            }
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                PromptResult::Continue
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                PromptResult::Continue
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
                PromptResult::Continue
            }
            _ => PromptResult::Continue,
        }
    }
}

pub enum PromptResult {
    Continue,
    Submit(String),
    Cancel,
}

fn render_prompt(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(prompt) = &app.prompt_state {
        // Center the prompt
        let popup_area = centered_rect(60, 20, area);

        // Clear background
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            popup_area,
        );

        // Render prompt
        let input_with_cursor = format!(
            "{}{}{}",
            &prompt.input[..prompt.cursor_pos],
            "█",
            &prompt.input[prompt.cursor_pos..]
        );

        let paragraph = Paragraph::new(vec![
            Line::from(prompt.message.clone()),
            Line::from(""),
            Line::from(input_with_cursor),
        ])
        .block(
            Block::default()
                .title("Input")
                .borders(Borders::ALL)
        );

        frame.render_widget(paragraph, popup_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

## Performance Considerations

### Efficient Rendering

```rust
// Only rebuild tree when data changes
impl App {
    pub fn refresh_tree_view(&mut self) {
        if self.tree_dirty {
            self.tree_view = TreeViewState::from_entry_tree(&self.storage.tree);
            self.tree_dirty = false;
        }
    }

    pub fn mark_tree_dirty(&mut self) {
        self.tree_dirty = true;
    }
}

// Lazy loading for large entries
impl Entry {
    pub fn load_preview(&mut self) -> Result<String> {
        if self.preview.is_none() {
            let content = std::fs::read_to_string(&self.path)?;
            self.preview = Some(content.lines().take(10).collect::<Vec<_>>().join("\n"));
        }
        Ok(self.preview.as_ref().unwrap().clone())
    }
}
```

## Testing UI Components

### Unit Testing Widgets

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_tree_view_navigation() {
        let mut tree_view = TreeViewState::new();
        tree_view.items = vec![
            TreeItem { /* ... */ },
            TreeItem { /* ... */ },
        ];

        tree_view.move_down();
        assert_eq!(tree_view.list_state.selected(), Some(1));

        tree_view.move_down();
        assert_eq!(tree_view.list_state.selected(), Some(0)); // Wraps around
    }

    #[test]
    fn test_prompt_input() {
        let mut prompt = PromptState::new("Enter date:");

        let result = prompt.handle_key(KeyEvent::from(KeyCode::Char('2')));
        assert!(matches!(result, PromptResult::Continue));
        assert_eq!(prompt.input, "2");

        let result = prompt.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(matches!(result, PromptResult::Submit(_)));
    }
}
```

## Common UI Patterns

### Modal Dialogs

```rust
pub fn render_with_modal(app: &App, frame: &mut Frame) {
    // Render main UI
    render_main_ui(app, frame);

    // Render modal on top if active
    if let Some(modal) = &app.active_modal {
        render_modal(modal, frame, frame.size());
    }
}

fn render_modal(modal: &Modal, frame: &mut Frame, area: Rect) {
    // Semi-transparent background
    let background = Block::default()
        .style(Style::default().bg(Color::Black));
    frame.render_widget(background, area);

    // Modal content
    let modal_area = centered_rect(50, 30, area);
    // ... render modal content
}
```

### Scrollable Content

```rust
pub struct ScrollableContent {
    pub content: Vec<String>,
    pub scroll_offset: usize,
    pub visible_lines: usize,
}

impl ScrollableContent {
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        let max_scroll = self.content.len().saturating_sub(self.visible_lines);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }

    pub fn visible_content(&self) -> &[String] {
        let start = self.scroll_offset;
        let end = (start + self.visible_lines).min(self.content.len());
        &self.content[start..end]
    }
}
```

This guide provides the foundation for implementing a polished TUI with ratatui. Focus on getting the basic layout and navigation working first, then add polish and advanced features gradually.
