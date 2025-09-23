use chrono::Local;
use color_eyre::eyre::{Ok, Result};

use crate::storage::Storage;

pub fn execute(storage: &Storage, id: Option<String>) -> Result<()> {
    println!("Creating new entry...");

    let entry_id = match id {
        Some(id) => id,
        None => Local::now().format("%Y%m%d").to_string(),
    };

    if storage.load_entry(&entry_id).is_ok() {
        println!(
            "Entry for {} already exists. Use 'devlog edit {}' to modify it.",
            entry_id, entry_id
        );
        return Ok(());
    }

    // Launch editor with template

    Ok(())
}
