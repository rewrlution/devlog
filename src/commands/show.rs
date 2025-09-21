use crate::storage::Storage;
use color_eyre::{eyre::Context, Result};

pub fn execute(id: String) -> Result<()> {
    let storage = Storage::new()?;

    let entry = storage
        .load_entry(&id)
        .wrap_err_with(|| format!("Entry '{}' not found", id))?;

    // Display entry with metadata
    println!("# Entry: {}", entry.id);
    println!(
        "Created: {}",
        entry.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "Updated: {}",
        entry.updated_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("---");
    println!("{}", entry.content);

    Ok(())
}
