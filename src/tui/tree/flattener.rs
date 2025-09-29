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
            Self::flatten_node_recursive(node, &prefix, is_last, &mut flat_items);
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
        prefix.chars().filter(|&c| c == '│' || c == ' ').count() / 4
    }

    fn build_child_prefix(prefix: &str, is_last: bool) -> String {
        if is_last {
            format!("{}    ", prefix) // 4 spaces for last node
        } else {
            format!("{}│   ", prefix) // pipe + 3 spaces for continuing branch
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a folder node
    fn create_folder_node(name: &str, children: Vec<TreeNode>, is_expanded: bool) -> TreeNode {
        TreeNode {
            name: name.to_string(),
            children,
            is_expanded,
            is_entry: false,
        }
    }

    /// Helper function to create an entry node
    fn create_entry_node(name: &str) -> TreeNode {
        TreeNode::new_entry(name.to_string())
    }

    #[test]
    fn test_flatten_empty_tree() {
        let nodes = vec![];
        let result = TreeFlattener::flatten(&nodes);
        assert!(result.is_empty());
    }

    #[test]
    fn test_flatten_single_entry_node() {
        let nodes = vec![create_entry_node("20250920")];
        let result = TreeFlattener::flatten(&nodes);

        assert_eq!(result.len(), 1);
        let (display_text, indent_level, is_entry) = &result[0];
        assert_eq!(display_text, "└─ 20250920");
        assert_eq!(*indent_level, 0);
        assert!(*is_entry);
    }

    #[test]
    fn test_flatten_single_collapsed_folder() {
        let nodes = vec![create_folder_node("2025", vec![], false)];
        let result = TreeFlattener::flatten(&nodes);

        assert_eq!(result.len(), 1);
        let (display_text, indent_level, is_entry) = &result[0];
        assert_eq!(display_text, "└─ [+] 2025");
        assert_eq!(*indent_level, 0);
        assert!(!*is_entry);
    }

    #[test]
    fn test_flatten_single_expanded_empty_folder() {
        let nodes = vec![create_folder_node("2025", vec![], true)];
        let result = TreeFlattener::flatten(&nodes);

        assert_eq!(result.len(), 1);
        let (display_text, indent_level, is_entry) = &result[0];
        assert_eq!(display_text, "└─ [-] 2025");
        assert_eq!(*indent_level, 0);
        assert!(!*is_entry);
    }

    #[test]
    fn test_flatten_multiple_entries_same_level() {
        let nodes = vec![
            create_entry_node("20250920"),
            create_entry_node("20250919"),
            create_entry_node("20250918"),
        ];
        let result = TreeFlattener::flatten(&nodes);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "├─ 20250920");
        assert_eq!(result[1].0, "├─ 20250919");
        assert_eq!(result[2].0, "└─ 20250918");

        // All should have same indent level and be entries
        for (_, indent_level, is_entry) in &result {
            assert_eq!(*indent_level, 0);
            assert!(*is_entry);
        }
    }

    #[test]
    fn test_flatten_expanded_folder_with_children() {
        let children = vec![create_entry_node("20250920"), create_entry_node("20250919")];
        let nodes = vec![create_folder_node("09", children, true)];
        let result = TreeFlattener::flatten(&nodes);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "└─ [-] 09");
        assert_eq!(result[0].1, 0); // indent level
        assert!(!result[0].2); // not an entry

        assert_eq!(result[1].0, "    ├─ 20250920");
        assert_eq!(result[1].1, 1); // indent level 1
        assert!(result[1].2); // is an entry

        assert_eq!(result[2].0, "    └─ 20250919");
        assert_eq!(result[2].1, 1); // indent level 1
        assert!(result[2].2); // is an entry
    }

    #[test]
    fn test_flatten_collapsed_folder_with_children() {
        let children = vec![create_entry_node("20250920")];
        let nodes = vec![create_folder_node("09", children, false)];
        let result = TreeFlattener::flatten(&nodes);

        // Only the folder should be shown, children should be hidden
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "└─ [+] 09");
        assert_eq!(result[0].1, 0);
        assert!(!result[0].2);
    }

    #[test]
    fn test_flatten_complex_hierarchy() {
        let days = vec![create_entry_node("20250920"), create_entry_node("20250919")];
        let months = vec![
            create_folder_node("09", days, true),
            create_folder_node("08", vec![create_entry_node("20250815")], false),
        ];
        let nodes = vec![create_folder_node("2025", months, true)];

        let result = TreeFlattener::flatten(&nodes);

        assert_eq!(result.len(), 5);

        // Year node
        assert_eq!(result[0].0, "└─ [-] 2025");
        assert_eq!(result[0].1, 0);
        assert!(!result[0].2);

        // September (expanded)
        assert_eq!(result[1].0, "    ├─ [-] 09");
        assert_eq!(result[1].1, 1);
        assert!(!result[1].2);

        // September entries
        assert_eq!(result[2].0, "    │   ├─ 20250920");
        assert_eq!(result[2].1, 2);
        assert!(result[2].2);

        assert_eq!(result[3].0, "    │   └─ 20250919");
        assert_eq!(result[3].1, 2);
        assert!(result[3].2);

        // August (collapsed)
        assert_eq!(result[4].0, "    └─ [+] 08");
        assert_eq!(result[4].1, 1);
        assert!(!result[4].2);
    }

    #[test]
    fn test_get_expansion_indicator() {
        assert_eq!(
            TreeFlattener::get_expansion_indicator(&create_entry_node("test")),
            ""
        );
        assert_eq!(
            TreeFlattener::get_expansion_indicator(&create_folder_node("test", vec![], false)),
            "[+] "
        );
        assert_eq!(
            TreeFlattener::get_expansion_indicator(&create_folder_node("test", vec![], true)),
            "[-] "
        );
    }

    #[test]
    fn test_calculate_indent_level() {
        assert_eq!(TreeFlattener::calculate_indent_level(""), 0);
        assert_eq!(TreeFlattener::calculate_indent_level("    "), 1);
        assert_eq!(TreeFlattener::calculate_indent_level("│   "), 1);
        assert_eq!(TreeFlattener::calculate_indent_level("    │   "), 2);
        assert_eq!(TreeFlattener::calculate_indent_level("        "), 2);
    }

    #[test]
    fn test_build_child_prefix() {
        assert_eq!(TreeFlattener::build_child_prefix("", true), "    ");
        assert_eq!(TreeFlattener::build_child_prefix("", false), "│   ");
        assert_eq!(TreeFlattener::build_child_prefix("    ", true), "        ");
        assert_eq!(TreeFlattener::build_child_prefix("    ", false), "    │   ");
    }

    #[test]
    fn test_build_display_text() {
        let entry_node = create_entry_node("20250920");
        let collapsed_folder = create_folder_node("09", vec![], false);
        let expanded_folder = create_folder_node("09", vec![], true);

        // Test with no prefix
        assert_eq!(
            TreeFlattener::build_display_text(&entry_node, "", true),
            "└─ 20250920"
        );
        assert_eq!(
            TreeFlattener::build_display_text(&entry_node, "", false),
            "├─ 20250920"
        );

        // Test folders
        assert_eq!(
            TreeFlattener::build_display_text(&collapsed_folder, "", true),
            "└─ [+] 09"
        );
        assert_eq!(
            TreeFlattener::build_display_text(&expanded_folder, "", true),
            "└─ [-] 09"
        );

        // Test with prefix
        assert_eq!(
            TreeFlattener::build_display_text(&entry_node, "    ", true),
            "    └─ 20250920"
        );
        assert_eq!(
            TreeFlattener::build_display_text(&entry_node, "│   ", false),
            "│   ├─ 20250920"
        );
    }
}
