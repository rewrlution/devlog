use color_eyre::eyre::{Context, Result};

use crate::storage::Storage;
use crate::utils::editor;

pub fn execute(storage: &Storage, id: String) -> Result<()> {
    // load existing entry
    let mut entry = storage
        .load_entry(&id)
        .wrap_err_with(|| format!("Entry '{}' not found", id))?;

    println!("Editing entry {id}");

    // Launch editor with existing content
    let new_content = editor::launch_editor(Some(&entry.content))?;

    // Update entry
    entry.update_content(new_content);
    storage.save_entry(&entry)?;

    println!("Entry updated successfully: {}", id);
    Ok(())
}
