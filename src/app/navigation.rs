use super::state::App;
use super::types::{NodeKind, TreeNode};

impl App {
    pub fn is_last_child(&self, path: &[usize]) -> bool {
        if path.is_empty() {
            return false;
        }
        let parent_path = &path[..path.len() - 1];
        let child_index = path[path.len() - 1];

        if let Some(parent) = self.node_by_path_slice(parent_path) {
            child_index == parent.children.len() - 1
        } else if parent_path.is_empty() {
            // Root level
            child_index == self.tree_root.len() - 1
        } else {
            false
        }
    }

    pub fn node_by_path(&self, path: &Vec<usize>) -> Option<&TreeNode> {
        self.node_by_path_slice(path.as_slice())
    }

    pub fn node_by_path_slice(&self, path: &[usize]) -> Option<&TreeNode> {
        let mut cur: Option<&TreeNode> = None;
        for (depth, &idx) in path.iter().enumerate() {
            if depth == 0 {
                cur = self.tree_root.get(idx);
            } else {
                cur = cur.and_then(|n| n.children.get(idx));
            }
        }
        cur
    }

    pub fn node_by_path_mut(&mut self, path: &Vec<usize>) -> Option<&mut TreeNode> {
        self.node_by_path_mut_slice(path.as_slice())
    }

    pub fn node_by_path_mut_slice(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        if path.is_empty() {
            return None;
        }
        let (first, rest) = (path[0], &path[1..]);
        let node = self.tree_root.get_mut(first)?;
        if rest.is_empty() {
            return Some(node);
        }
        let mut cur = node;
        for &idx in rest {
            cur = cur.children.get_mut(idx)?;
        }
        Some(cur)
    }

    pub fn toggle_expand_at_selected(&mut self, expand: bool) {
        if let Some(sel) = self.selected_index {
            if let Some((_indent, path)) = self.flat_nodes.get(sel).cloned() {
                if let Some(node) = self.node_by_path_mut(&path) {
                    if !matches!(node.kind, NodeKind::Day { .. }) {
                        node.expanded = expand;
                        self.recompute_flat_nodes();
                    }
                }
            }
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.flat_nodes.is_empty() {
            self.selected_index = None;
            return;
        }
        let cur = self.selected_index.unwrap_or(0) as isize;
        let len = self.flat_nodes.len() as isize;
        let mut next = cur + delta;
        if next < 0 {
            next = 0;
        }
        if next >= len {
            next = len - 1;
        }
        self.selected_index = Some(next as usize);
        // Only auto-open files if we're navigating to a day node
        if let Some(sel) = self.selected_index {
            if let Some((_indent, path)) = self.flat_nodes.get(sel) {
                if let Some(node) = self.node_by_path(path) {
                    if let NodeKind::Day { filename } = &node.kind {
                        let filename_clone = filename.clone();
                        let _ = self.open_file_by_name(&filename_clone);
                    }
                }
            }
        }
    }

    pub fn select_day_by_filename(&mut self, filename: &str) {
        for (i, (_indent, path)) in self.flat_nodes.iter().enumerate() {
            if let Some(node) = self.node_by_path(path) {
                if let NodeKind::Day { filename: f } = &node.kind {
                    if f == filename {
                        self.selected_index = Some(i);
                        return;
                    }
                }
            }
        }
    }

    pub fn selected_filename(&self) -> Option<&str> {
        if let Some(sel) = self.selected_index {
            if let Some((_indent, path)) = self.flat_nodes.get(sel) {
                if let Some(node) = self.node_by_path(path) {
                    if let NodeKind::Day { filename } = &node.kind {
                        return Some(filename.as_str());
                    }
                }
            }
        }
        None
    }
}
