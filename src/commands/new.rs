use crate::models::entry::Entry;
use crate::storage::Storage;
use crate::utils::editor;

use chrono::Local;
use color_eyre::eyre::{Ok, Result};

pub fn execute(storage: &Storage, id: Option<String>) -> Result<()> {
    println!("Creating new entry...");

    let entry_id = match id {
        Some(id) => id,
        None => Local::now().format("%Y%m%d").to_string(),
    };

    if storage.load_entry(&entry_id).is_ok() {
        println!(
            "Entry for {} already exists. Use 'devlog edit --id {}' to modify it.",
            entry_id, entry_id
        );
        return Ok(());
    }

    // Launch editor with template
    let content = editor::launch_editor(None)?;

    // Create and save entry
    let entry = Entry::new(entry_id.clone(), content);
    storage.save_entry(&entry)?;

    println!("Entry created successfully: {}", entry_id);
    Ok(())
}
