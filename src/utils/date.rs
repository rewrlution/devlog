use chrono::{Local, NaiveDate};
use color_eyre::{Result, eyre};

/// Parse YYYYMMDD format into NaiveDate
pub fn parse_entry_date(date_str: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y%m%d")
        .map_err(|e| eyre::eyre!("Invalid date format '{}': {}", date_str, e))
}

/// Get today's date
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Format date as YYYYMMDD
pub fn format_entry_date(date: &NaiveDate) -> String {
    date.format("%Y%m%d").to_string()
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;

    #[test]
    fn test_parse_entry_date() {
        let date = parse_entry_date("20250315").unwrap();
        assert_eq!(date.year(), 2025);
        assert_eq!(date.month(), 3);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_parse_invalid_date() {
        assert!(parse_entry_date("invalid").is_err());
        assert!(parse_entry_date("20251301").is_err()); // invalid month
        assert!(parse_entry_date("20250230").is_err()); // invalid day
    }

    #[test]
    fn test_format_entry_date() {
        let date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        assert_eq!(format_entry_date(&date), "20250315");
    }
}
