use crate::{
    data::{Entry, Storage},
    navigation::EntryTree,
    utils::date::parse_entry_date,
};
use chrono::{Datelike, NaiveDate};
use color_eyre::Result;
use std::fs;
use walkdir::WalkDir;

/// Navigation state for the application
#[derive(Debug)]
pub struct NavigationState {
    pub tree: EntryTree,
    pub selected_date: Option<NaiveDate>,
    pub expanded_year: Option<u32>,
    pub expanded_month: Option<u32>,
}

impl NavigationState {
    /// Create a new navigation state
    pub fn new() -> Self {
        Self {
            tree: EntryTree::new(),
            selected_date: None,
            expanded_year: None,
            expanded_month: None,
        }
    }

    /// Load the navigation state from storage
    pub fn load_from_storage(storage: &Storage) -> Result<Self> {
        let mut state = Self::new();
        state.refresh_from_storage(storage);
        Ok(state)
    }

    /// Refresh the tree from storage when files actually change: create/update/delete
    pub fn refresh_from_storage(&mut self, storage: &Storage) -> Result<()> {
        // Clear existing tree
        self.tree = EntryTree::new();

        // Get the base directory path
        let base_path = storage.get_base_dir();
        if !base_path.exists() {
            return Ok(());
        }

        // Walk through all files recursively
        for entry in WalkDir::new(base_path)
            .into_iter()
            .filter_map(|e| e.ok()) // Skip errors, continue with valid entries
            .filter(|e| e.file_type().is_file()) // Only process files
            .filter(|e| {
                // Only process .md files
                e.path().extension().map_or(false, |ext| ext == "md")
            })
        {
            let file_path = entry.path();

            // Extract filename (YYYYMMDD)
            if let Some(file_stem) = file_path.file_stem() {
                if let Some(date_str) = file_stem.to_str() {
                    match parse_entry_date(date_str) {
                        Ok(date) => {
                            // Load the entry using storage
                            match storage.load_entry(date) {
                                Ok(entry) => {
                                    self.tree.add_entry(entry);
                                }
                                Err(e) => {
                                    // Log warning but continue processing other files
                                    eprintln!("Warning: Failed to load entry for {}: {}", date, e);
                                }
                            }
                        }
                        Err(_) => {
                            // Skip files with invalid date formats
                            eprintln!("Warning: Failed to process date str: {}", date_str);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Select a specific entry date
    pub fn select_date(&mut self, date: NaiveDate) {
        self.selected_date = Some(date);
    }

    /// Clear the selection
    pub fn clear_selection(&mut self) {
        self.selected_date = None;
    }

    /// Get the currently selected date
    pub fn get_selected_date(&self) -> Option<NaiveDate> {
        self.selected_date
    }

    /// Move selection to the next entry
    pub fn select_next(&mut self) {
        let dates = self.tree.get_all_dates();
        if dates.is_empty() {
            return;
        }

        match self.selected_date {
            None => self.selected_date = dates.first().copied(),
            Some(current) => {
                if let Some(pos) = dates.iter().position(|&d| d == current) {
                    if pos + 1 < dates.len() {
                        self.selected_date = Some(dates[pos + 1]);
                    }
                }
            }
        }
    }

    /// Move selection to the previous entry
    pub fn select_prev(&mut self) {
        let dates = self.tree.get_all_dates();
        if dates.is_empty() {
            return;
        }

        match self.selected_date {
            None => self.selected_date = dates.last().copied(),
            Some(current) => {
                if let Some(pos) = dates.iter().position(|&d| d == current) {
                    if pos > 0 {
                        self.selected_date = Some(dates[pos - 1]);
                    }
                }
            }
        }
    }

    /// Expand a specific year (collapses any other expanded year)
    pub fn expand_year(&mut self, year: u32) {
        self.expanded_year = Some(year);
        self.expanded_month = None; // Reset month expansion
    }

    /// Expand a specific month with the expanded year
    pub fn expand_month(&mut self, month: u32) {
        if self.expanded_year.is_some() {
            self.expanded_month = Some(month);
        }
    }

    /// Collapse the expanded year (and month)
    pub fn collapse_year(&mut self) {
        self.expanded_year = None;
        self.expanded_month = None;
    }

    /// Collapse the expanded month (keep year expanded)
    pub fn collapse_month(&mut self) {
        self.expanded_month = None;
    }

    /// Toggle year expansion
    pub fn toggle_year(&mut self, year: u32) {
        if self.expanded_year == Some(year) {
            self.collapse_year();
        } else {
            self.expand_year(year);
        }
    }

    /// Toggle month expansion
    pub fn toggle_month(&mut self, month: u32) {
        if self.expanded_year.is_some() {
            if self.expanded_month == Some(month) {
                self.collapse_month();
            } else {
                self.expand_month(month);
            }
        }
    }

    /// Check if a year is expanded
    pub fn is_year_expanded(&self, year: u32) -> bool {
        self.expanded_year == Some(year)
    }

    /// Check if a month is expanded
    pub fn is_month_expanded(&self, month: u32) -> bool {
        self.expanded_month == Some(month)
    }

    /// Get the currently expanded year
    pub fn get_expanded_year(&self) -> Option<u32> {
        self.expanded_year
    }

    /// Get the currently expanded month
    pub fn get_expanded_month(&self) -> Option<u32> {
        self.expanded_month
    }

    /// Add a new entry to the tree and optionally select it
    pub fn add_entry(&mut self, entry: Entry, select: bool) {
        let date = entry.date;
        self.tree.add_entry(entry);

        if select {
            self.selected_date = Some(date);
            // Auto-expand to show the new entry
            self.expand_year(date.year() as u32);
            self.expand_month(date.month());
        }
    }

    /// Get all available years in descending order
    pub fn get_years(&self) -> Vec<u32> {
        let mut years = self.tree.get_years();
        years.sort_by(|a, b| b.cmp(a));
        years
    }

    /// Get all months for the expanded year in descending order
    pub fn get_months_for_expanded_year(&self) -> Vec<u32> {
        match self.expanded_year {
            Some(year) => {
                let mut months = self.tree.get_months_for_year(year);
                months.sort_by(|a, b| b.cmp(a));
                months
            }
            None => Vec::new(),
        }
    }

    /// Get all days for the expanded month in descending order
    pub fn get_days_for_expanded_month(&self) -> Vec<u32> {
        match (self.expanded_year, self.expanded_month) {
            (Some(year), Some(month)) => {
                let mut days = self.tree.get_days_for_month(year, month);
                days.sort_by(|a, b| b.cmp(a));
                days
            }
            _ => Vec::new(),
        }
    }

    /// Get the currently selected entry
    pub fn get_selected_entry(&self) -> Option<&Entry> {
        self.selected_date
            .and_then(|date| self.tree.get_entry(&date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_entry(year: i32, month: u32, day: u32) -> Entry {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        Entry::with_content(date, "Test content".to_string())
    }

    #[test]
    fn test_new_navigation_state() {
        let state = NavigationState::new();
        assert!(state.tree.is_empty());
        assert!(state.selected_date.is_none());
        assert!(state.expanded_year.is_none());
        assert!(state.expanded_month.is_none());
    }

    #[test]
    fn test_year_expansion() {
        let mut state = NavigationState::new();

        // Initially nothing expanded
        assert!(!state.is_year_expanded(2025));

        // Expand 2025
        state.expand_year(2025);
        assert!(state.is_year_expanded(2025));
        assert!(!state.is_year_expanded(2024));

        // Expanding different year should collapse previous
        state.expand_year(2024);
        assert!(state.is_year_expanded(2024));
        assert!(!state.is_year_expanded(2025));

        // Toggle should collapse
        state.toggle_year(2024);
        assert!(!state.is_year_expanded(2024));
    }

    #[test]
    fn test_month_expansion() {
        let mut state = NavigationState::new();

        // Can't expand month without year
        state.expand_month(3);
        assert!(!state.is_month_expanded(3));

        // Expand year first
        state.expand_year(2025);
        state.expand_month(3);
        assert!(state.is_month_expanded(3));

        // Collapsing year should collapse month
        state.collapse_year();
        assert!(!state.is_year_expanded(2025));
        assert!(!state.is_month_expanded(3));
    }

    #[test]
    fn test_auto_expansion_on_add() {
        let mut state = NavigationState::new();
        let entry = create_test_entry(2025, 3, 15);

        state.add_entry(entry, true);

        // Should auto-expand to show the new entry
        assert!(state.is_year_expanded(2025));
        assert!(state.is_month_expanded(3));
        assert_eq!(
            state.get_selected_date(),
            Some(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap())
        );
    }

    #[test]
    fn test_expanded_year_months() {
        let mut state = NavigationState::new();

        state.add_entry(create_test_entry(2025, 3, 15), false);
        state.add_entry(create_test_entry(2025, 4, 10), false);
        state.add_entry(create_test_entry(2024, 12, 31), false);

        // No months when no year expanded
        assert!(state.get_months_for_expanded_year().is_empty());

        // Expand 2025, months are sorted in descending order
        state.expand_year(2025);
        let months = state.get_months_for_expanded_year();
        assert_eq!(months, vec![4, 3]);

        // Expand 2024
        state.expand_year(2024);
        let months = state.get_months_for_expanded_year();
        assert_eq!(months, vec![12]);
    }

    #[test]
    fn test_refresh_from_storage_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());
        let mut state = NavigationState::new();

        let result = state.refresh_from_storage(&storage);
        assert!(result.is_ok());
        assert!(state.tree.is_empty());
    }

    #[test]
    fn test_refresh_from_storage_with_entries() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());

        // Create some test entries using storage
        let entries = vec![
            create_test_entry(2025, 3, 15),
            create_test_entry(2025, 3, 16),
            create_test_entry(2025, 4, 1),
            create_test_entry(2024, 12, 31),
        ];

        // Save entries to create the directory structure
        for entry in &entries {
            storage.save_entry(entry).unwrap();
        }

        // Now refresh and check that all entries are loaded
        let mut state = NavigationState::new();
        let result = state.refresh_from_storage(&storage);

        assert!(result.is_ok());
        assert!(!state.tree.is_empty());

        // Check that all dates are loaded
        let loaded_dates = state.tree.get_all_dates();
        assert_eq!(loaded_dates.len(), 4);

        for entry in &entries {
            assert!(loaded_dates.contains(&entry.date));
            assert!(state.tree.get_entry(&entry.date).is_some());
        }
    }

    #[test]
    fn test_refresh_from_storage_ignores_invalid_filenames() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path().to_path_buf());

        // Create directory structure
        let year_dir = temp_dir.path().join("2025").join("03");
        fs::create_dir_all(&year_dir).unwrap();

        // Create files with invalid date formats (should be ignored)
        fs::write(year_dir.join("invalid.md"), "Invalid filename").unwrap();
        fs::write(year_dir.join("2025031.md"), "Too short").unwrap();
        fs::write(year_dir.join("202503155.md"), "Too long").unwrap();
        fs::write(year_dir.join("20250229.md"), "Invalid date").unwrap();
        fs::write(year_dir.join("readme.md"), "Not a date").unwrap();

        // Create valid files
        fs::write(year_dir.join("20250315.md"), "Valid entry 1").unwrap();
        fs::write(year_dir.join("20250316.md"), "Valid entry 2").unwrap();

        let mut state = NavigationState::new();
        let result = state.refresh_from_storage(&storage);

        assert!(result.is_ok());

        // Should only load the valid files, ignore invalid ones
        let loaded_dates = state.tree.get_all_dates();
        assert_eq!(loaded_dates.len(), 2);

        let expected_dates = vec![
            NaiveDate::from_ymd_opt(2025, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2025, 3, 16).unwrap(),
        ];

        for expected_date in expected_dates {
            assert!(loaded_dates.contains(&expected_date));
        }
    }
}
