use crate::storage::Storage;
use color_eyre::Result;
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};

pub fn execute() -> Result<()> {
    let storage = Storage::new()?;
    let entries = storage.list_entries()?;

    if entries.is_empty() {
        println!("No entries found. Create your first entry with 'devlog new'");
        return Ok(());
    }

    println!("Recent entries (last 20):");
    println!();

    for entry_id in entries.iter().take(20) {
        // Load the entry to get its content
        let preview = match storage.load_entry(entry_id) {
            Ok(entry) => {
                // Get the first line of content and truncate based on display width
                let first_line = entry.content
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim();
                
                // Target display width of 60 characters (accounting for visual width)
                let target_width = 60;
                let ellipsis_width = 3; // "..." width
                
                let truncated = if first_line.width() > target_width {
                    // Find the longest substring that fits within the target width
                    let mut current_width = 0;
                    let mut char_boundary = 0;
                    
                    for (idx, ch) in first_line.char_indices() {
                        let char_width = ch.width().unwrap_or(0);
                        if current_width + char_width + ellipsis_width > target_width {
                            break;
                        }
                        current_width += char_width;
                        char_boundary = idx + ch.len_utf8();
                    }
                    
                    format!("{}...", &first_line[..char_boundary])
                } else {
                    first_line.to_string()
                };
                
                if truncated.is_empty() {
                    "(empty)".to_string()
                } else {
                    truncated
                }
            }
            Err(_) => "(error reading entry)".to_string(),
        };
        
        println!("{}  {}", entry_id, preview);
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
