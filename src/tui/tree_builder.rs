use std::collections::HashMap;

use color_eyre::eyre::Result;

use crate::{storage::Storage, tui::models::node::TreeNode};

pub struct TreeBuilder {
    storage: Storage,
}

impl TreeBuilder {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    fn build_map(&self) -> Result<HashMap<String, HashMap<String, Vec<String>>>> {
        let entry_ids = self.storage.list_entries()?;

        // Build year -> month -> day hierarchy
        let mut year_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

        for entry_id in entry_ids {
            // entry id format: YYYYMMDD
            let year = entry_id[0..4].to_string();
            let month = entry_id[4..6].to_string();

            year_map
                .entry(year)
                .or_default()
                .entry(month)
                .or_default()
                .push(entry_id);
        }

        Ok(year_map)
    }

    pub fn build_tree(&self) -> Result<Vec<TreeNode>> {
        let year_map = self.build_map()?;
        // Convert to tree structure
        let mut tree_nodes = Vec::new();
        let mut years: Vec<_> = year_map.keys().collect();
        years.sort_by(|a, b| b.cmp(a)); // Newest first

        for year in years {
            let year_months = &year_map[year];
            let mut months: Vec<_> = year_months.keys().collect();
            months.sort_by(|a, b| b.cmp(a));

            let mut month_nodes = Vec::new();
            for month in months {
                let month_days = &year_months[month];
                let mut days = month_days.clone();
                days.sort_by(|a, b| b.cmp(a));

                let day_nodes: Vec<TreeNode> =
                    days.into_iter().map(|d| TreeNode::new_entry(d)).collect();

                month_nodes.push(TreeNode {
                    name: month.to_string(),
                    children: day_nodes,
                    is_expanded: false,
                    is_entry: false,
                });
            }

            tree_nodes.push(TreeNode {
                name: year.to_string(),
                children: month_nodes,
                is_expanded: false,
                is_entry: false,
            });
        }

        Ok(tree_nodes)
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
        prefix, connector, expansion_indicator, node.name
    );

    // Calcualte indent level by counting tree characters (for styling)
    let indent_level = prefix.chars().filter(|&c| c == '|' || c == ' ').count() / 4;
    flat_items.push((display_text, indent_level, node.is_entry));

    // Add children if expanded
    if node.is_expanded && !node.children.is_empty() {
        let child_prefix = if is_last {
            format!("{}    ", prefix) // 4 spaces for last node
        } else {
            format!("{}|  ", prefix) // pipe + 3 spaces for continuing branch
        };

        for (i, child) in node.children.iter().enumerate() {
            let child_is_last = i == node.children.len() - 1;
            flatten_node_with_tree_art(child, &child_prefix, child_is_last, flat_items);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::entry::Entry, tui::tree_builder};
    use tempfile::TempDir;

    /// Create a test storage instance in a temporary directory
    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage = Storage::new(Some(temp_dir.path())).expect("Failed to create storage");
        (storage, temp_dir)
    }

    /// Helper function to create and save test entries
    fn create_test_entries(storage: &Storage, entry_ids: &[&str]) {
        for &id in entry_ids {
            let entry = Entry::new(id.to_string(), format!("Content for {}", id));
            storage
                .save_entry(&entry)
                .expect("Failed to save test entry");
        }
    }

    #[test]
    fn test_build_map_multiple_months_and_years() {
        let (storage, _temp_dir) = create_test_storage();
        create_test_entries(
            &storage,
            &[
                "20250920", // September 2025
                "20250821", // August 2025
                "20240715", // July 2024
                "20240716", // July 2024
            ],
        );

        let tree_builder = TreeBuilder::new(storage);
        let result = tree_builder.build_map().expect("Failed to build map");

        assert_eq!(result.len(), 2); // Two years
        assert!(result.contains_key("2025"));
        assert!(result.contains_key("2024"));

        // Check 2025
        let year_2025 = &result["2025"];
        assert_eq!(year_2025.len(), 2); // Two months
        assert!(year_2025.contains_key("09"));
        assert!(year_2025.contains_key("08"));

        // Check 2024
        let year_2024 = &result["2024"];
        assert_eq!(year_2024.len(), 1); // One month
        assert!(year_2024.contains_key("07"));

        let month_07_2024 = &year_2024["07"];
        assert_eq!(month_07_2024.len(), 2); // Two days
    }

    #[test]
    fn test_build_tree_empty() {
        let (storage, _temp_dir) = create_test_storage();
        let tree_builder = TreeBuilder::new(storage);

        let tree_nodes = tree_builder.build_tree().expect("Failed to build tree");
        assert!(tree_nodes.is_empty());
    }

    #[test]
    fn test_build_tree_single_entry() {
        let (storage, _temp_dir) = create_test_storage();
        create_test_entries(&storage, &["20250920"]);

        let tree_builder = TreeBuilder::new(storage);
        let result = tree_builder.build_tree().expect("Failed to build tree");

        assert_eq!(result.len(), 1); // One year node

        let year_node = &result[0];
        assert_eq!(year_node.name, "2025");
        assert!(!year_node.is_expanded);
        assert!(!year_node.is_entry);
        assert_eq!(year_node.children.len(), 1); // One month

        let month_node = &year_node.children[0];
        assert_eq!(month_node.name, "09");
        assert!(!month_node.is_expanded);
        assert!(!month_node.is_entry);
        assert_eq!(month_node.children.len(), 1); // One day

        let day_node = &month_node.children[0];
        assert_eq!(day_node.name, "20250920");
        assert!(!day_node.is_expanded);
        assert!(day_node.is_entry);
        assert!(day_node.children.is_empty());
    }

    #[test]
    fn test_build_tree_sorting_newest_first() {
        let (storage, _temp_dir) = create_test_storage();
        create_test_entries(
            &storage,
            &[
                "20240715", // July 2024 (oldest)
                "20250821", // August 2025
                "20250920", // September 2025 (newest)
            ],
        );

        let tree_builder = TreeBuilder::new(storage);
        let result = tree_builder.build_tree().expect("Failed to build tree");

        assert_eq!(result.len(), 2); // Two years

        // Years should be sorted newest first
        assert_eq!(result[0].name, "2025");
        assert_eq!(result[1].name, "2024");

        // Months within 2025 should be sorted newest first
        let year_2025 = &result[0];
        assert_eq!(year_2025.children.len(), 2);
        assert_eq!(year_2025.children[0].name, "09"); // September
        assert_eq!(year_2025.children[1].name, "08"); // August
    }

    #[test]
    fn test_build_tree_multiple_days_same_month_sorting() {
        let (storage, _temp_dir) = create_test_storage();
        create_test_entries(
            &storage,
            &[
                "20250918", // 18th (oldest)
                "20250920", // 20th (newest)
                "20250919", // 19th (middle)
            ],
        );

        let tree_builder = TreeBuilder::new(storage);
        let result = tree_builder.build_tree().expect("Failed to build tree");

        let year_node = &result[0];
        let month_node = &year_node.children[0];

        // Days should be sorted newest first
        assert_eq!(month_node.children.len(), 3);
        assert_eq!(month_node.children[0].name, "20250920"); // newest
        assert_eq!(month_node.children[1].name, "20250919"); // middle
        assert_eq!(month_node.children[2].name, "20250918"); // oldest

        // All day nodes should be marked as entries
        for day_node in &month_node.children {
            assert!(day_node.is_entry);
            assert!(day_node.children.is_empty());
        }
    }

    #[test]
    fn test_build_tree_structure_properties() {
        let (storage, _temp_dir) = create_test_storage();
        create_test_entries(&storage, &["20250920", "20240715"]);

        let tree_builder = TreeBuilder::new(storage);
        let result = tree_builder.build_tree().expect("Failed to build tree");

        // Verify the structure properties for all nodes
        for year_node in &result {
            assert!(!year_node.is_entry);
            assert!(!year_node.is_expanded);

            for month_node in &year_node.children {
                assert!(!month_node.is_entry);
                assert!(!month_node.is_expanded);

                for day_node in &month_node.children {
                    assert!(day_node.is_entry);
                    assert!(!day_node.is_expanded);
                    assert!(day_node.children.is_empty());
                }
            }
        }
    }
}
