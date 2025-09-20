use chrono::{Datelike, NaiveDate};
use std::cmp::min;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use crate::utils::{devlog_path, list_existing_devlog_files, today_str};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMode {
    Preview,
    Edit,
    DatePrompt,
    SavePrompt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Tree,
    Content,
}

#[derive(Clone, Debug)]
pub enum NodeKind {
    Year,
    Month,
    Day { filename: String },
}

#[derive(Clone, Debug)]
pub struct TreeNode {
    pub label: String,
    pub kind: NodeKind,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

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

impl App {
    pub fn new() -> io::Result<Self> {
        let mut app = Self {
            files: list_existing_devlog_files()?,
            tree_root: Vec::new(),
            flat_nodes: Vec::new(),
            selected_index: None,
            current_path: None,
            content: String::new(),
            cursor_row: 0,
            cursor_col: 0,
            focus: Focus::Tree,
            view_scroll: 0,
            dirty: false,
            mode: AppMode::Preview,
            date_input: today_str(),
            date_error: None,
            save_choice: 0,
            last_tick: Instant::now(),
        };
        // Build tree and select most recent file if any
        app.rebuild_tree();
        if !app.files.is_empty() {
            if let Some(name) = app.files.last().cloned() {
                app.select_day_by_filename(&name);
                app.open_file_by_name(&name).ok();
            }
        }
        Ok(app)
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

    pub fn open_file_by_name(&mut self, name: &str) -> io::Result<()> {
        let path = devlog_path().join(name);
        let mut f = File::open(&path)?;
        let mut content = String::new();
        f.read_to_string(&mut content)?;
        self.current_path = Some(path);
        self.content = content;
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.view_scroll = 0; // reset scroll when changing file
        self.dirty = false;
        self.mode = AppMode::Preview;
        Ok(())
    }

    pub fn open_or_create_for_date(&mut self, yyyymmdd: &str) -> io::Result<()> {
        use std::fs;
        let name = format!("{}.md", yyyymmdd);
        let path = devlog_path().join(&name);
        fs::create_dir_all(devlog_path())?;
        if !path.exists() {
            File::create(&path)?; // create empty file
                                  // refresh file list and tree
            self.files = list_existing_devlog_files()?;
            self.rebuild_tree();
        }
        // select the day node in tree
        self.select_day_by_filename(&name);
        // open and switch to edit
        self.open_file_by_name(&name)?;
        self.mode = AppMode::Edit;
        self.move_cursor_to_end();
        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.current_path {
            let mut f = File::create(path)?;
            f.write_all(self.content.as_bytes())?;
            self.dirty = false;
        }
        Ok(())
    }

    // ---- Tree building and navigation helpers ----
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

    pub fn validate_date(input: &str) -> Result<(), &'static str> {
        if input.len() != 8 || !input.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid date. Use YYYYMMDD.");
        }
        if NaiveDate::parse_from_str(input, "%Y%m%d").is_err() {
            return Err("Invalid calendar date.");
        }
        Ok(())
    }

    pub fn move_cursor_to_end(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.is_empty() {
            self.cursor_row = 0;
            self.cursor_col = 0;
        } else {
            self.cursor_row = lines.len() - 1;
            // Use character count, not byte length
            self.cursor_col = lines.last().unwrap().chars().count();
        }
    }

    pub fn insert_char(&mut self, ch: char) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        // Convert to character-based indexing
        let line_chars: Vec<char> = line.chars().collect();
        let col = self.cursor_col.min(line_chars.len());

        // Insert character at the correct character position
        let mut new_chars = line_chars;
        new_chars.insert(col, ch);
        *line = new_chars.into_iter().collect();

        // Advance cursor by 1 character (not bytes)
        self.cursor_col = col + 1;
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn backspace(&mut self) {
        if self.cursor_row == 0 && self.cursor_col == 0 {
            return;
        }
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }
        let row = self.cursor_row;
        let col = self.cursor_col;
        if col > 0 {
            let line = &mut lines[row];
            // Convert to character-based indexing
            let mut line_chars: Vec<char> = line.chars().collect();
            if col <= line_chars.len() {
                let char_idx = col - 1;
                line_chars.remove(char_idx);
                *line = line_chars.into_iter().collect();
                self.cursor_col = char_idx;
            }
        } else if row > 0 {
            // Moving to previous line - use character count for cursor position
            let prev_line_chars = lines[row - 1].chars().count();
            let current = lines.remove(row);
            self.cursor_row -= 1;
            self.cursor_col = prev_line_chars;
            lines[self.cursor_row].push_str(&current);
        }
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn delete(&mut self) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return;
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];
        // Convert to character-based indexing
        let mut line_chars: Vec<char> = line.chars().collect();
        let line_char_len = line_chars.len();

        if self.cursor_col < line_char_len {
            // Delete character at cursor position
            line_chars.remove(self.cursor_col);
            *line = line_chars.into_iter().collect();
        } else if row + 1 < lines.len() {
            // Delete newline - merge with next line
            let next = lines.remove(row + 1);
            lines[row].push_str(&next);
        }
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let mut lines: Vec<String> = self.content.split('\n').map(|s| s.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let row = self.cursor_row.min(lines.len() - 1);
        let line = &mut lines[row];

        // Convert to character-based indexing
        let line_chars: Vec<char> = line.chars().collect();
        let col = self.cursor_col.min(line_chars.len());

        // Split line at character position
        let (left_chars, right_chars) = line_chars.split_at(col);
        *line = left_chars.iter().collect();
        let rest: String = right_chars.iter().collect();

        self.cursor_row = row + 1;
        self.cursor_col = 0;
        lines.insert(self.cursor_row, rest);
        self.content = lines.join("\n");
        self.dirty = true;
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Use character count, not byte length
            self.cursor_col = self
                .content
                .split('\n')
                .nth(self.cursor_row)
                .map(|s| s.chars().count())
                .unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if lines.is_empty() {
            return;
        }
        // Use character count, not byte length
        let line_char_len = lines[self.cursor_row.min(lines.len() - 1)].chars().count();
        if self.cursor_col < line_char_len {
            self.cursor_col += 1;
        } else if self.cursor_row + 1 < lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            // Use character count, not byte length
            let line_char_len = self
                .content
                .split('\n')
                .nth(self.cursor_row)
                .map(|s| s.chars().count())
                .unwrap_or(0);
            self.cursor_col = min(self.cursor_col, line_char_len);
        }
    }

    pub fn move_down(&mut self) {
        let lines: Vec<&str> = self.content.split('\n').collect();
        if self.cursor_row + 1 < lines.len() {
            self.cursor_row += 1;
            // Use character count, not byte length
            let line_char_len = lines[self.cursor_row].chars().count();
            self.cursor_col = min(self.cursor_col, line_char_len);
        }
    }
}
