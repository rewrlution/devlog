use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};

use crate::data::Entry;

/// Represents a month in the entry tree
#[derive(Debug, Clone)]
pub struct Month {
    pub month: u32,
    pub entries: BTreeMap<u32, Entry>,
}

/// Represents a year in the entry tree
#[derive(Debug, Clone)]
pub struct Year {
    pub year: u32,
    pub months: BTreeMap<u32, Month>,
}

/// Main tree structure for organizing entries
#[derive(Debug, Clone)]
pub struct EntryTree {
    pub years: BTreeMap<u32, Year>,
}

impl Month {
    pub fn new(month: u32) -> Self {
        Self {
            month,
            entries: BTreeMap::new(),
        }
    }

    /// Add an entry to this month
    pub fn add_entry(&mut self, entry: Entry) {
        let day = entry.date.day();
        self.entries.insert(day, entry);
    }

    /// Get entry for a specific day
    pub fn get_entry(&self, day: u32) -> Option<&Entry> {
        self.entries.get(&day)
    }

    /// check if this month has any entries
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Year {
    pub fn new(year: u32) -> Self {
        Self {
            year,
            months: BTreeMap::new(),
        }
    }

    /// Add an entry to this year
    pub fn add_entry(&mut self, entry: Entry) {
        let month_num = entry.date.month();
        let month = self
            .months
            .entry(month_num)
            .or_insert_with(|| Month::new(month_num));
        month.add_entry(entry);
    }

    /// Get all entries in this year
    pub fn get_all_entries(&self) -> Vec<&Entry> {
        self.months
            .values()
            .flat_map(|month| month.entries.values())
            .collect()
    }

    /// Check if this year has any entries
    pub fn is_empty(&self) -> bool {
        self.months.is_empty()
    }
}

impl EntryTree {
    pub fn new() -> Self {
        Self {
            years: BTreeMap::new(),
        }
    }

    /// Add an entry to the tree
    pub fn add_entry(&mut self, entry: Entry) {
        let year_num = entry.date.year() as u32;
        let year = self
            .years
            .entry(year_num)
            .or_insert_with(|| Year::new(year_num));
        year.add_entry(entry);
    }

    /// Get an entry by date
    pub fn get_entry(&self, date: &NaiveDate) -> Option<&Entry> {
        let year_num = date.year() as u32;
        let month_num = date.month();
        let day_num = date.day();

        self.years
            .get(&year_num)?
            .months
            .get(&month_num)?
            .entries
            .get(&day_num)
    }

    /// Get all entries in chronological order
    pub fn get_all_entries(&self) -> Vec<&Entry> {
        self.years
            .values()
            .flat_map(|year| year.get_all_entries())
            .collect()
    }

    /// Get all entry dates in chronological order
    pub fn get_all_dates(&self) -> Vec<NaiveDate> {
        let mut dates = Vec::new();

        for year in self.years.values() {
            for month in year.months.values() {
                for entry in month.entries.values() {
                    dates.push(entry.date);
                }
            }
        }

        dates.sort();
        dates
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.years.is_empty()
    }

    /// Get the latest entry date
    pub fn get_latest_date(&self) -> Option<NaiveDate> {
        self.get_all_dates().into_iter().last()
    }

    /// Get the earliest entry date
    pub fn get_earliest_date(&self) -> Option<NaiveDate> {
        self.get_all_dates().into_iter().next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entry(year: i32, month: u32, day: u32, content: &str) -> Entry {
        let date = NaiveDate::from_ymd_opt(year, month, day).unwrap();
        Entry::with_content(date, content.to_string())
    }

    #[test]
    fn test_empty_tree() {
        let tree = EntryTree::new();
        assert!(tree.is_empty());
        assert!(tree.get_latest_date().is_none());
        assert!(tree.get_earliest_date().is_none());
    }

    #[test]
    fn test_add_single_entry() {
        let mut tree = EntryTree::new();
        let entry = create_test_entry(2025, 3, 15, "Test entry");
        let date = entry.date;

        tree.add_entry(entry);

        assert!(!tree.is_empty());
        assert!(tree.get_entry(&date).is_some());
    }

    #[test]
    fn test_multiple_entries() {
        let mut tree = EntryTree::new();

        tree.add_entry(create_test_entry(2025, 3, 15, "Entry 1"));
        tree.add_entry(create_test_entry(2025, 3, 16, "Entry 2"));
        tree.add_entry(create_test_entry(2025, 4, 1, "Entry 3"));
        tree.add_entry(create_test_entry(2024, 12, 31, "Entry 4"));

        let all_dates = tree.get_all_dates();
        assert_eq!(all_dates.len(), 4);

        // Should be in chronological order
        assert_eq!(all_dates[0], NaiveDate::from_ymd_opt(2024, 12, 31).unwrap());
        assert_eq!(all_dates[1], NaiveDate::from_ymd_opt(2025, 3, 15).unwrap());
        assert_eq!(all_dates[2], NaiveDate::from_ymd_opt(2025, 3, 16).unwrap());
        assert_eq!(all_dates[3], NaiveDate::from_ymd_opt(2025, 4, 1).unwrap());
    }

    #[test]
    fn test_latest_and_earliest() {
        let mut tree = EntryTree::new();

        tree.add_entry(create_test_entry(2025, 3, 15, "Middle"));
        tree.add_entry(create_test_entry(2025, 4, 1, "Latest"));
        tree.add_entry(create_test_entry(2024, 12, 31, "Earliest"));

        assert_eq!(
            tree.get_earliest_date(),
            Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap())
        );
        assert_eq!(
            tree.get_latest_date(),
            Some(NaiveDate::from_ymd_opt(2025, 4, 1).unwrap())
        );
    }
}
