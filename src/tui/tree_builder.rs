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
