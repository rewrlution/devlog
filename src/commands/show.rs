use color_eyre::eyre::{Context, Ok, Result};

use crate::storage::Storage;

pub fn execute(storage: &Storage, id: String) -> Result<()> {
    println!("Showing entry {id}\n\n\n");

    let entry = storage
        .load_entry(&id)
        .wrap_err_with(|| format!("Entry '{}' not found", id))?;

    println!("{}", entry.content);

    Ok(())
}
