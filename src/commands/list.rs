use color_eyre::eyre::Result;

use crate::storage::Storage;
use crate::tui::app::launch_tui;

pub fn execute(storage: &Storage, interactive: bool) -> Result<()> {
    if interactive {
        launch_tui()?;
    } else {
        display_list(storage)?;
    }

    Ok(())
}

fn display_list(storage: &Storage) -> Result<()> {
    let entries = storage.list_entries()?;

    println!("Recent entries (last 20)\n");

    for entry_id in entries.iter().take(20) {
        // Load the entry to get its content
        let preview = match storage.load_entry(&entry_id) {
            Ok(entry) => entry.preview(),
            Err(_) => "(error reading entry)".to_string(),
        };

        println!("{}  {}", entry_id, preview);
    }

    Ok(())
}
