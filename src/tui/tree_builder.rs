use crate::storage::Storage;
use crate::tui::data::TreeNode;
use color_eyre::Result;
use std::collections::HashMap;

pub struct TreeBuilder {
    storage: Storage,
}

impl TreeBuilder {
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        Ok(Self { storage })
    }

    pub fn build_tree(&self) -> Result<Vec<TreeNode>> {
        let entry_ids = self.storage.list_entries()?;

        // Build year -> month -> day hierarchy
        let mut year_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        for entry_id in entry_ids {
            if entry_id.len() == 8 {
                // YYYYMMDD format
                let year = entry_id[0..4].to_string();
                let month = entry_id[4..6].to_string();

                year_map
                    .entry(year)
                    .or_default()
                    .entry(month)
                    .or_default()
                    .push(entry_id);
            }
        }

        // Convert to tree structure
        let mut tree_nodes = Vec::new();
        let mut years: Vec<_> = year_map.keys().collect();
        years.sort_by(|a, b| b.cmp(a)); // Newest first

        for year in years {
            let year_months = &year_map[year];
            let mut months: Vec<_> = year_months.keys().collect();
            months.sort_by(|a, b| b.cmp(a)); // Newest first

            let mut month_nodes = Vec::new();
            for month in months {
                let month_days = &year_months[month];
                let mut days = month_days.clone();
                days.sort_by(|a, b| b.cmp(a)); // Newest first

                let day_nodes: Vec<TreeNode> = days
                    .into_iter()
                    .map(|day| TreeNode::new_entry(format_entry_display(&day)))
                    .collect();

                month_nodes.push(TreeNode {
                    display_name: format!("{}-{}", year, month),
                    children: day_nodes,
                    is_expanded: false,
                    is_entry: false,
                });
            }

            tree_nodes.push(TreeNode {
                display_name: year.clone(),
                children: month_nodes,
                is_expanded: false,
                is_entry: false,
            });
        }

        Ok(tree_nodes)
    }
}

fn format_entry_display(entry_id: &str) -> String {
    if entry_id.len() == 8 {
        format!(
            "{}-{}-{}",
            &entry_id[0..4],
            &entry_id[4..6],
            &entry_id[6..8]
        )
    } else {
        entry_id.to_string()
    }
}

pub fn flatten_tree(nodes: &[TreeNode]) -> Vec<(String, usize, bool)> {
    let mut flat_items = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let is_last = i == nodes.len() - 1;
        let prefix = String::new();
        flatten_node_with_tree_art(node, &prefix, is_last, &mut flat_items);
    }
    flat_items
}

fn flatten_node_with_tree_art(
    node: &TreeNode,
    prefix: &str,
    is_last: bool,
    flat_items: &mut Vec<(String, usize, bool)>,
) {
    // Build the tree art prefix for this node
    let connector = if is_last { "└─ " } else { "├─ " };
    let expansion_indicator = if node.is_entry {
        ""
    } else if node.is_expanded {
        "[-] "
    } else {
        "[+] "
    };

    let display_text = format!(
        "{}{}{}{}",
        prefix, connector, expansion_indicator, node.display_name
    );

    // Calculate indent level by counting tree characters (for styling)
    let indent_level = prefix.chars().filter(|&c| c == '|' || c == ' ').count() / 4;
    flat_items.push((display_text, indent_level, node.is_entry));

    // Add children if expanded
    if node.is_expanded && !node.children.is_empty() {
        let child_prefix = if is_last {
            format!("{}    ", prefix) // 4 spaces for last node
        } else {
            format!("{}│   ", prefix) // pipe + 3 spaces for continuing branch
        };

        for (i, child) in node.children.iter().enumerate() {
            let child_is_last = i == node.children.len() - 1;
            flatten_node_with_tree_art(child, &child_prefix, child_is_last, flat_items);
        }
    }
}
