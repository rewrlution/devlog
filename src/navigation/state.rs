use chrono::{Datelike, NaiveDate};
use color_eyre::{Result, eyre::Ok};

use crate::{
    data::{Entry, Storage},
    navigation::EntryTree,
};

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
        state.refresh_from_Storage(storage);
        Ok(state)
    }

    /// Refresh the tree from storage when files actually change: create/update/delete
    pub fn refresh_from_Storage(&mut self, storage: &Storage) -> Result<()> {
        // TODO: implement directory scanning
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
                let mut days = self.tree.get_days_for_mont(year, month);
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
