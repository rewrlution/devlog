use std::path::PathBuf;
use std::time::Instant;

use super::types::{AppMode, Focus, TreeNode};

pub struct App {
    // Source files and derived tree
    pub files: Vec<String>,
    pub tree_root: Vec<TreeNode>,
    // Flattened visible nodes for rendering and selection
    pub flat_nodes: Vec<(usize, Vec<usize>)>, // (indent, path indices)
    pub selected_index: Option<usize>,
    // Currently open file (full path) and its content
    pub current_path: Option<PathBuf>,
    pub content: String,
    // Editor state
    pub cursor_row: usize,
    pub cursor_col: usize,
    // View state
    pub focus: Focus,
    pub view_scroll: usize,
    pub dirty: bool,
    pub mode: AppMode,
    // Date prompt state
    pub date_input: String,
    pub date_error: Option<String>,
    // Save prompt state: 0=Yes,1=No,2=Cancel
    pub save_choice: usize,
    // Timing
    pub last_tick: Instant,
}
