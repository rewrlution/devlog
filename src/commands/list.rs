use crate::storage::Storage;
use color_eyre::Result;

pub fn execute() -> Result<()> {
    let storage = Storage::new()?;
    let entries = storage.list_entries()?;

    if entries.is_empty() {
        println!("No entries found. Create your first entry with 'devlog new'");
        return Ok(());
    }

    println!("Recent entries (last 20):");
    println!();

    for (i, entry_id) in entries.iter().take(20).enumerate() {
        // Format YYYYMMDD as YYYY-MM-DD for display
        let formatted_id = if entry_id.len() == 8 {
            format!("{}-{}-{}", &entry_id[0..4], &entry_id[4..6], &entry_id[6..8])
        } else {
            entry_id.clone()
        };
        println!("  {}. {} ({})", i + 1, formatted_id, entry_id);
    }

    println!();
    println!("Commands:");
    println!("  devlog show <id>              - View an entry (use YYYYMMDD format)");
    println!("  devlog edit <id>              - Edit an entry (use YYYYMMDD format)");
    println!("  devlog list --interactive     - Launch TUI mode");

    Ok(())
}

pub fn execute_interactive() -> Result<()> {
    crate::tui::app::launch_tui()
}
