use crate::tui::{models::node::TreeNode, tree::flattener::FlatTreeItem};

#[derive(PartialEq, Debug)]
pub enum Panel {
    Nav,
    Content,
}

#[derive(Debug)]
pub struct AppState {
    /// Hierarchical tree structure organizing entries by year/month/day
    /// This represents the logical organization of journal entries
    pub tree_nodes: Vec<TreeNode>,

    /// Flattened list representation for UI rendering
    /// Contains the current view state including which nodes are expanded/collapsed
    /// Directly coupled to ratatui's ListState and ListItem components
    pub flat_items: Vec<FlatTreeItem>,

    /// Currently active panel (navigation or content view)
    pub current_panel: Panel,

    /// Content of the currently selected journal entry
    pub selected_entry_content: String,

    /// Vertical scroll position within the content panel
    pub content_scroll: u16,

    /// Forces a complete UI redraw on next render cycle
    pub needs_redraw: bool,

    /// Signals the application to terminate gracefully
    pub should_quit: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tree_nodes: Vec::new(),
            flat_items: Vec::new(),
            current_panel: Panel::Nav,
            selected_entry_content: String::new(),
            content_scroll: 0,
            should_quit: false,
            needs_redraw: false,
        }
    }

    pub fn scroll_content_up(&mut self) {
        // `saturating_sub()` performs saturating subtraction, which means "saturates" at min value
        self.content_scroll = self.content_scroll.saturating_sub(1);
    }

    pub fn scroll_content_down(&mut self, max_scroll: u16) {
        if self.content_scroll < max_scroll {
            self.content_scroll += 1;
        }
    }

    pub fn reset_content_scroll(&mut self) {
        self.content_scroll = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_scroll_up() {
        let mut state = AppState::new();
        state.content_scroll = 5;
        state.scroll_content_up();
        assert_eq!(state.content_scroll, 4);

        // Test saturating behavior
        state.content_scroll = 0;
        state.scroll_content_up();
        assert_eq!(state.content_scroll, 0);
    }

    #[test]
    fn test_app_state_scroll_down() {
        let mut state = AppState::new();
        state.scroll_content_down(10);
        assert_eq!(state.content_scroll, 1);

        // Test max limit
        state.content_scroll = 10;
        state.scroll_content_down(10);
        assert_eq!(state.content_scroll, 10);
    }
}
