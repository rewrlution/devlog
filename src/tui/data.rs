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
