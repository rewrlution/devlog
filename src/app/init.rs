use std::io;
use std::time::Instant;

use super::state::App;
use super::types::{AppMode, Focus};
use crate::utils::{list_existing_devlog_files, today_str};

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
}
