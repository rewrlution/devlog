use color_eyre::eyre::Result;

use crate::storage::Storage;

pub fn execute(storage: &Storage, interactive: bool) -> Result<()> {
    if interactive {
        println!("Listing in interactive mode");
    } else {
        println!("List latest 10 entries");
    }

    Ok(())
}

fn execute_list(storage: &Storage) {}
