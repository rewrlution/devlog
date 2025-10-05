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

    /// Builds a hierarchical map of entries organized by year -> month -> days
    fn build_entry_map(&self) -> Result<HashMap<String, HashMap<String, Vec<String>>>> {
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

    /// Builds the complete tree structure from storage
    pub fn build_tree(&self) -> Result<Vec<TreeNode>> {
        let year_map = self.build_entry_map()?;
        let mut tree_nodes = Vec::new();

        // Sort years newest first
        let mut years: Vec<_> = year_map.keys().collect();
        years.sort_by(|a, b| b.cmp(a));

        for year in years {
            let year_node = self.build_year_node(year, &year_map[year]);
            tree_nodes.push(year_node);
        }

        Ok(tree_nodes)
    }

    fn build_year_node(&self, year: &str, months: &HashMap<String, Vec<String>>) -> TreeNode {
        let mut month_nodes = Vec::new();

        // Sort months newest first
        let mut sorted_months: Vec<_> = months.keys().collect();
        sorted_months.sort_by(|a, b| b.cmp(a));

        for month in sorted_months {
            let month_node = self.build_month_node(month, &months[month]);
            month_nodes.push(month_node);
        }

        TreeNode {
            name: year.to_string(),
            children: month_nodes,
            is_expanded: false,
            is_entry: false,
        }
    }

    fn build_month_node(&self, month: &str, days: &[String]) -> TreeNode {
        // Sort days newest first
        let mut sorted_days = days.to_vec();
        sorted_days.sort_by(|a, b| b.cmp(a));

        let day_nodes: Vec<TreeNode> = sorted_days.into_iter().map(TreeNode::new_entry).collect();

        TreeNode {
            name: month.to_string(),
            children: day_nodes,
            is_expanded: false,
            is_entry: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entry::Entry;
    use tempfile::TempDir;

    /// Create a test storage instance in a temporary directory
    fn create_test_storage() -> (Storage, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let storage =
            Storage::new_with_base_dir(temp_dir.path()).expect("Failed to create storage");
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
        let result = tree_builder.build_entry_map().expect("Failed to build map");

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
