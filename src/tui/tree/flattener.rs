use crate::tui::models::node::TreeNode;

/// Represents a flattened tree item with display text, indent level, and entry status
pub type FlatTreeItem = (String, usize, bool);

pub struct TreeFlattener;

impl TreeFlattener {
    /// Flattens a tree structure into a linear list suitable for display
    pub fn flatten(nodes: &[TreeNode]) -> Vec<FlatTreeItem> {
        let mut flat_items = Vec::new();

        for (i, node) in nodes.iter().enumerate() {
            let is_last = i == nodes.len() - 1;
            let prefix = String::new();
            // recursively
        }

        flat_items
    }

    fn flatten_node_recursive(
        node: &TreeNode,
        prefix: &str,
        is_last: bool,
        flat_items: &mut Vec<FlatTreeItem>,
    ) {
        let display_text = Self::build_display_text(node, prefix, is_last);
        let indent_level = Self::calculate_indent_level(prefix);

        flat_items.push((display_text, indent_level, node.is_entry));

        // Process children if node is expanded
        if node.is_expanded && !node.children.is_empty() {
            let child_prefix = Self::build_child_prefix(prefix, is_last);

            for (i, child) in node.children.iter().enumerate() {
                let child_is_last = i == node.children.len() - 1;
                Self::flatten_node_recursive(child, &child_prefix, child_is_last, flat_items);
            }
        }
    }

    fn build_display_text(node: &TreeNode, prefix: &str, is_last: bool) -> String {
        let connector = if is_last { "└─ " } else { "├─ " };
        let expansion_indicator = Self::get_expansion_indicator(node);

        format!(
            "{}{}{}{}",
            prefix, connector, expansion_indicator, node.name
        )
    }

    fn get_expansion_indicator(node: &TreeNode) -> &'static str {
        if node.is_entry {
            ""
        } else if node.is_expanded {
            "[-] "
        } else {
            "[+] "
        }
    }

    fn calculate_indent_level(prefix: &str) -> usize {
        prefix.chars().filter(|&c| c == '|' || c == ' ').count() / 4
    }

    fn build_child_prefix(prefix: &str, is_last: bool) -> String {
        if is_last {
            format!("{}    ", prefix) // 4 spaces for last node
        } else {
            format!("{}│   ", prefix) // pipe + 3 spaces for continuing branch
        }
    }
}
