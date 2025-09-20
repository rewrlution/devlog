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
                    self.toggle_node_recursive(&mut app_state.tree_nodes, selected, &mut current_index)?;
                    app_state.flat_items = flatten_tree(&app_state.tree_nodes);
                }
            }
        }
        Ok(())
    }

    fn toggle_node_recursive(
        &self,
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

            if node.is_expanded && self.toggle_node_recursive(&mut node.children, target_index, current_index)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn collapse_node(&self, app_state: &mut AppState, tree_state: &mut ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            let mut current_index = 0;
            self.collapse_node_recursive(&mut app_state.tree_nodes, selected, &mut current_index)?;
            app_state.flat_items = flatten_tree(&app_state.tree_nodes);
        }
        Ok(())
    }

    fn collapse_node_recursive(
        &self,
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

            if node.is_expanded && self.collapse_node_recursive(&mut node.children, target_index, current_index)? {
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
                            }
                            Err(_) => {
                                app_state.selected_entry_content = "Error loading entry".to_string();
                            }
                        }
                    }
                } else {
                    app_state.selected_entry_content = "Select an entry to view its content".to_string();
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
