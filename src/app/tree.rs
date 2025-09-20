use chrono::{Datelike, NaiveDate};
use std::collections::BTreeMap;

use super::state::App;
use super::types::{NodeKind, TreeNode};

impl App {
    pub fn rebuild_tree(&mut self) {
        let mut root: Vec<TreeNode> = Vec::new();
        let mut year_map: BTreeMap<i32, BTreeMap<u32, Vec<String>>> = BTreeMap::new();

        // Find the latest entry filename
        let latest_file = self.files.iter().max().cloned();
        let (latest_year, latest_month) = if let Some(ref fname) = latest_file {
            if let Ok(date) = NaiveDate::parse_from_str(&fname[..8], "%Y%m%d") {
                (Some(date.year()), Some(date.month()))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Group files by year and month
        for fname in &self.files {
            if let Ok(date) = NaiveDate::parse_from_str(&fname[..8], "%Y%m%d") {
                let year = date.year();
                let month = date.month();
                year_map
                    .entry(year)
                    .or_default()
                    .entry(month)
                    .or_default()
                    .push(fname.clone());
            }
        }

        // Build tree structure - iterate years in descending order
        for (year, months) in year_map.into_iter().rev() {
            let mut year_node = TreeNode {
                label: format!("{}", year),
                kind: NodeKind::Year,
                children: Vec::new(),
                expanded: Some(year) == latest_year, // Only expand year containing latest entry
            };
            for (month, mut days) in months.into_iter().rev() {
                let mut month_node = TreeNode {
                    label: format!("{:04}-{:02}", year, month),
                    kind: NodeKind::Month,
                    children: Vec::new(),
                    expanded: Some(year) == latest_year && Some(month) == latest_month, // Only expand month containing latest entry
                };
                // Sort days in descending order (newest first)
                days.sort_by(|a, b| b.cmp(a));
                for fname in days {
                    let date = &fname[..8];
                    let label = match NaiveDate::parse_from_str(date, "%Y%m%d") {
                        Ok(d) => d.format("%Y-%m-%d").to_string(),
                        Err(_) => date.to_string(),
                    };
                    month_node.children.push(TreeNode {
                        label,
                        kind: NodeKind::Day { filename: fname },
                        children: Vec::new(),
                        expanded: false,
                    });
                }
                year_node.children.push(month_node);
            }
            root.push(year_node);
        }
        self.tree_root = root;
        self.recompute_flat_nodes();

        // Auto-select the latest entry
        if let Some(latest_file) = latest_file {
            self.select_day_by_filename(&latest_file);
            // Open the latest file
            let _ = self.open_file_by_name(&latest_file);
        } else if self.selected_index.is_none() && !self.flat_nodes.is_empty() {
            self.selected_index = Some(0);
        }
    }

    pub fn recompute_flat_nodes(&mut self) {
        self.flat_nodes.clear();
        let len = self.tree_root.len();
        for i in 0..len {
            let mut path = vec![i];
            self.flatten_from(&mut path, 0);
        }
        if let Some(sel) = self.selected_index {
            if self.flat_nodes.is_empty() {
                self.selected_index = None;
            } else if sel >= self.flat_nodes.len() {
                self.selected_index = Some(self.flat_nodes.len() - 1);
            }
        }
    }

    fn flatten_from(&mut self, path: &mut Vec<usize>, indent: usize) {
        self.flat_nodes.push((indent, path.clone()));
        if let Some(node) = self.node_by_path(path) {
            if node.expanded {
                for child_idx in 0..node.children.len() {
                    path.push(child_idx);
                    self.flatten_from(path, indent + 1);
                    path.pop();
                }
            }
        }
    }
}
