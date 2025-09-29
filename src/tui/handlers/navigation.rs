use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::widgets::ListState;

use crate::{
    storage::Storage,
    tui::{models::state::AppState, tree::flattener::FlatTreeItem},
};

pub struct TreeNavigator {
    storage: Storage,
}

impl TreeNavigator {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn handle_navigation(
        &self,
        key_code: KeyCode,
        app_state: &mut AppState,
        tree_state: &mut ListState,
        flat_items: &mut Vec<FlatTreeItem>,
    ) -> Result<()> {
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
}
