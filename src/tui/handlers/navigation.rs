use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

use crate::tui::{
    models::{node::TreeNode, state::AppState},
    tree::flattener::TreeFlattener,
};

pub struct TreeNavigator {}

impl TreeNavigator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_navigation(
        &self,
        key_code: KeyCode,
        app_state: &mut AppState,
        tree_state: &mut ListState,
    ) -> Result<()> {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up(tree_state);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down(tree_state, app_state.flat_items.len());
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                self.toggle_node(app_state, tree_state)?;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.collapse_node(app_state, tree_state)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Move the selection up by one position in the list widget
    fn move_up(&self, tree_state: &mut ListState) {
        let selected = tree_state.selected().unwrap_or(0);
        if selected > 0 {
            tree_state.select(Some(selected - 1));
        }
    }

    fn move_down(&self, tree_state: &mut ListState, items_len: usize) {
        let selected = tree_state.selected().unwrap_or(0);
        if selected < items_len.saturating_sub(1) {
            tree_state.select(Some(selected + 1));
        }
    }

    fn toggle_node(&self, app_state: &mut AppState, tree_state: &mut ListState) -> Result<()> {
        if let Some(selected) = tree_state.selected() {
            if let Some((_, is_entry)) = app_state.flat_items.get(selected) {
                if !is_entry {
                    // It's a folder, toggle expansion
                    let mut current_index = 0;
                    Self::toggle_node_recursive(
                        &mut app_state.tree_nodes,
                        selected,
                        &mut current_index,
                    )?;
                    app_state.flat_items = TreeFlattener::flatten(&app_state.tree_nodes);
                }
            }
        }
        Ok(())
    }

    /// The function traverses a tree structure to find a node at a specific target_index
    /// and toggles its `is_expanded` state (collapsed ↔ expanded).
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
            app_state.flat_items = TreeFlattener::flatten(&app_state.tree_nodes);
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
}
