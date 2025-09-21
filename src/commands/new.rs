use crate::storage::{entry::Entry, Storage};
use crate::utils::editor;
use chrono::Utc;
use color_eyre::Result;

pub fn execute() -> Result<()> {
    println!("Creating new entry...");

    // Get current date as entry ID
    let today = Utc::now().format("%Y%m%d").to_string();

    // Check if entry already exists for today
    let storage = Storage::new()?;
    if storage.load_entry(&today).is_ok() {
        println!(
            "Entry for {} already exists. Use 'devlog edit {}' to modify it.",
            today, today
        );
        return Ok(());
    }

    // Launch editor with template
    let template = format!(
        "# Development Log - {}\n\n## What I worked on today\n\n\n## What I learned\n\n\n## Next steps\n\n",
        today
    );

    let content = editor::launch_editor(&template)?;

    // Create and save entry
    let entry = Entry::new(content);
    storage.save_entry(&entry)?;

    println!("Entry created successfully: {}", today);
    Ok(())
}
