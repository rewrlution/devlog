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

// fn execute_list(storage: &Storage) -> Result<()> {
//     let entries = storage.list_entries()?;

//     println!("Recent entries (last 20)\n");

//     for entry_id in entries.iter().take(20) {
//         // Load the entry to get its content
//         let preview = match storage.load_entry(&entry_id) {
//             Ok(entry) => {
//                 // Get the first line of content and truncate based on display width
//                 let first_line = entry.content.lines().next().unwrap_or("").trim();

//             }
//         }
//     }
// }
