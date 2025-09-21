use crate::storage::Storage;
use crate::tui::data::{AppState, Panel, TreeNode};
use crate::tui::tree_builder::flatten_tree;
use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

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
        tree_state: &mut ListState,
    ) -> Result<()> {
        match key_code {
            KeyCode::Char('q') => app_state.should_quit = true,
            KeyCode::Tab => {
                app_state.current_panel = match app_state.current_panel {
                    Panel::Tree => Panel::Content,
                    Panel::Content => Panel::Tree,
                };
            }
            KeyCode::Char('e') => {
                if app_state.current_panel == Panel::Content {
                    self.edit_current_entry(app_state, tree_state)?;
                }
            }
            _ => match app_state.current_panel {
                Panel::Tree => {
                    self.handle_tree_navigation(key_code, app_state, tree_state)?;
                }
                Panel::Content => {
                    self.handle_content_navigation(key_code, app_state)?;
                }
            },
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
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let selected = tree_state.selected().unwrap_or(0);
                if selected < app_state.flat_items.len().saturating_sub(1) {
                    tree_state.select(Some(selected + 1));
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                self.toggle_node(app_state, tree_state)?;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.collapse_node(app_state, tree_state)?;
            }
            _ => {}
        }

        // Update content panel when selection changes
        self.update_content_panel(app_state, tree_state)?;
        Ok(())
    }

    fn toggle_node(&self, app_state: &mut AppState, tree_state: &mut ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if let Some((_, _, is_entry)) = app_state.flat_items.get(selected) {
                if !is_entry {
                    // It's a folder, toggle expansion
                    let mut current_index = 0;
                    Self::toggle_node_recursive(
                        &mut app_state.tree_nodes,
                        selected,
                        &mut current_index,
                    )?;
                    app_state.flat_items = flatten_tree(&app_state.tree_nodes);
                }
            }
        }
        Ok(())
    }

    fn toggle_node_recursive(
        nodes: &mut [TreeNode],
        target_index: usize,
        current_index: &mut usize,
    ) -> Result<bool> {
        for node in nodes {
            if *current_index == target_index {
                node.is_expanded = !node.is_expanded;
                return Ok(true);
            }
            *current_index += 1;

            if node.is_expanded
                && Self::toggle_node_recursive(&mut node.children, target_index, current_index)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn collapse_node(&self, app_state: &mut AppState, tree_state: &mut ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            let mut current_index = 0;
            Self::collapse_node_recursive(&mut app_state.tree_nodes, selected, &mut current_index)?;
            app_state.flat_items = flatten_tree(&app_state.tree_nodes);
        }
        Ok(())
    }

    fn collapse_node_recursive(
        nodes: &mut [TreeNode],
        target_index: usize,
        current_index: &mut usize,
    ) -> Result<bool> {
        for node in nodes {
            if *current_index == target_index {
                node.is_expanded = false;
                return Ok(true);
            }
            *current_index += 1;

            if node.is_expanded
                && Self::collapse_node_recursive(&mut node.children, target_index, current_index)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn update_content_panel(&self, app_state: &mut AppState, tree_state: &ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if let Some((text, _, is_entry)) = app_state.flat_items.get(selected) {
                if *is_entry {
                    // Extract entry ID from display text
                    if let Some(entry_id) = self.extract_entry_id(text) {
                        match self.storage.load_entry(&entry_id) {
                            Ok(entry) => {
                                app_state.selected_entry_content = entry.content;
                                app_state.reset_content_scroll(); // Reset scroll when loading new content
                            }
                            Err(_) => {
                                app_state.selected_entry_content =
                                    "Error loading entry".to_string();
                                app_state.reset_content_scroll();
                            }
                        }
                    }
                } else {
                    app_state.selected_entry_content =
                        "Select an entry to view its content".to_string();
                    app_state.reset_content_scroll();
                }
            }
        }
        Ok(())
    }

    fn handle_content_navigation(&self, key_code: KeyCode, app_state: &mut AppState) -> Result<()> {
        let content_lines = app_state.selected_entry_content.lines().count();
        let max_scroll = content_lines.saturating_sub(1) as u16;

        match key_code {
            KeyCode::Up | KeyCode::Char('k') => {
                app_state.scroll_content_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app_state.scroll_content_down(max_scroll);
            }
            KeyCode::Home => {
                app_state.reset_content_scroll();
            }
            KeyCode::End => {
                app_state.content_scroll = max_scroll;
            }
            KeyCode::PageUp => {
                for _ in 0..10 {
                    app_state.scroll_content_up();
                }
            }
            KeyCode::PageDown => {
                for _ in 0..10 {
                    app_state.scroll_content_down(max_scroll);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn edit_current_entry(&self, app_state: &mut AppState, tree_state: &ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if let Some((text, _, is_entry)) = app_state.flat_items.get(selected) {
                if *is_entry {
                    if let Some(entry_id) = self.extract_entry_id(text) {
                        // Temporarily exit raw mode and restore terminal
                        use crossterm::{
                            execute,
                            terminal::{
                                disable_raw_mode, enable_raw_mode, Clear, ClearType,
                                EnterAlternateScreen, LeaveAlternateScreen,
                            },
                        };
                        use std::io;

                        // Save current terminal state and exit TUI mode
                        disable_raw_mode()?;
                        execute!(io::stdout(), LeaveAlternateScreen)?;

                        // Launch editor
                        let result = {
                            use crate::utils::editor;
                            let mut entry = self.storage.load_entry(&entry_id)?;
                            let new_content = editor::launch_editor(&entry.content);
                            match new_content {
                                Ok(content) => {
                                    entry.update_content(content);
                                    self.storage.save_entry(&entry)
                                }
                                Err(e) => Err(e),
                            }
                        };

                        // Restore TUI mode with proper screen clearing
                        enable_raw_mode()?;
                        execute!(
                            io::stdout(),
                            EnterAlternateScreen,
                            Clear(ClearType::All),
                            crossterm::cursor::MoveTo(0, 0)
                        )?;

                        // Handle editor result
                        result?;

                        // Refresh the content in the TUI
                        self.update_content_panel(app_state, tree_state)?;

                        // Set flag to force full redraw on next render
                        app_state.needs_redraw = true;
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_entry_id(&self, display_text: &str) -> Option<String> {
        // Find the date pattern YYYY-MM-DD in the display text
        if let Some(start) = display_text.find(char::is_numeric) {
            let date_part = &display_text[start..];
            if date_part.len() >= 10 {
                let date_str = &date_part[0..10]; // YYYY-MM-DD
                if date_str.matches('-').count() == 2 {
                    // Convert YYYY-MM-DD to YYYYMMDD
                    return Some(date_str.replace('-', ""));
                }
            }
        }
        None
    }
}
