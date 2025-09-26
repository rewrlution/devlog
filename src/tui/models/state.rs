use crate::tui::models::node::TreeNode;

#[derive(PartialEq)]
pub enum Panel {
    Nav,
    Content,
}

pub struct AppState {
    pub tree_nodes: Vec<TreeNode>,
    pub current_panel: Panel,
    pub selected_entry: String,
    pub content_scroll: u16, // Current scroll position in content
    pub should_quit: bool,
    pub needs_redraw: bool, // Flag to force full redraw after editor quit
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
            current_panel: Panel::Nav,
            selected_entry: String::new(),
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
