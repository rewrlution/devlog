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

    pub fn reset_content_scroll(&mut self) {
        self.content_scroll = 0;
    }
}
