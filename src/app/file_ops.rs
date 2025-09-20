use std::fs::File;
use std::io::{self, Read, Write};
use chrono::NaiveDate;

use super::state::App;
use super::types::AppMode;
use crate::utils::{devlog_path, list_existing_devlog_files};

impl App {
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

    pub fn validate_date(input: &str) -> Result<(), &'static str> {
        if input.len() != 8 || !input.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid date. Use YYYYMMDD.");
        }
        if NaiveDate::parse_from_str(input, "%Y%m%d").is_err() {
            return Err("Invalid calendar date.");
        }
        Ok(())
    }
}
