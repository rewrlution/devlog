use crate::entry::Entry;
use crate::storage::EntryStorage;
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use std::process;

/// DevLog - a journal CLI tool for developers
#[derive(Parser)]
#[command(name = "devlog")]
#[command(about = "A journal CLI tool for developers")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new entry
    New {
        /// Inline message for the entry
        #[arg(short, long)]
        message: Option<String>,
        /// Optional ID for the entry (format: YYYMMDD)
        #[arg(long, value_name = "YYYYMMDD")]
        id: Option<String>,
    },
    /// Edit an existing entry
    Edit {
        /// Entry ID to edit (format: YYYYMMDD)
        #[arg(long, value_name = "YYYYMMDD")]
        id: String,
    },
}

impl Cli {
    /// Run the CLI application
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();
        // TODO: read user defined storage path
        // For now, we use the default `base_dir`, which is `~/.devlog`
        let storage = EntryStorage::new(None)?;

        match cli.command {
            Commands::New { message, id } => {
                Self::handle_new_command(message, id, &storage)?;
            }
            Commands::Edit { id } => {
                Self::handle_edit_command(id, &storage)?;
            }
        }

        Ok(())
    }

    /// Handle the new subcommand
    fn handle_new_command(
        message: Option<String>,
        custom_id: Option<String>,
        storage: &EntryStorage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate custom ID format if provided
        if let Some(ref id) = custom_id {
            Self::validate_id_format(id)?;
        }
        // Generate ID: use custom ID if provided, otherwise use current date
        let id = custom_id.unwrap_or_else(|| format!("{}", Local::now().format("%Y%m%d")));

        let content = match message {
            Some(msg) => msg,
            None => Self::open_editor_for_content(None)?,
        };

        if content.trim().is_empty() {
            eprintln!("Entry content cannot be empty.");
            process::exit(1);
        }

        // Create new entry with mandatory ID
        let entry = Entry::new(content, id);

        // Save the entry
        entry.save(storage)?;

        let state = entry.current_state();
        println!("Created new entry: {}", state.id);

        Ok(())
    }

    /// Handle the edit subcommand
    fn handle_edit_command(
        id: String,
        storage: &EntryStorage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Load existing entry
        let mut entry = match Entry::load(&id, storage)? {
            Some(entry) => entry,
            None => {
                eprintln!("Entry with ID '{}' not found.", id);
                process::exit(1);
            }
        };

        // Get current content and open editor with it
        let current_content = entry.current_state().content.clone();
        let new_content = Self::open_editor_for_content(Some(&current_content))?;

        if new_content.trim().is_empty() {
            eprintln!("Entry content cannot be empty.");
            process::exit(1);
        }

        // Update the entry
        entry.update_content(new_content);

        // Save the updated entry
        entry.save(storage)?;

        println!("Updated entry: {}", id);

        Ok(())
    }

    /// Validate that the ID is in YYYYMMDD format
    fn validate_id_format(id: &str) -> Result<(), Box<dyn std::error::Error>> {
        NaiveDate::parse_from_str(id, "%Y%m%d")
            .map(|_| format!("Invalid date format '{}'. Expected YYYYMMDD formate", id))?;
        Ok(())
    }

    /// Open a text editor for the user to write content
    fn open_editor_for_content(
        existing_content: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Create a temporary file for editing
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path();

        // Write initial content with instructions
        let init_content = match existing_content {
            Some(content) => format!("{}\n{}", content, Self::get_template()),
            None => Self::get_template(),
        };
        std::fs::write(temp_path, init_content)?;

        // Open the editor
        let editor = Self::find_available_editor();
        let status = process::Command::new(&editor).arg(temp_path).status()?;

        if !status.success() {
            return Err(format!("Editor '{}' exited with non-zero status", editor).into());
        }

        // Read content back from temp file
        let content = std::fs::read_to_string(temp_path)?;

        // Clean the content by removing comment lines
        let processed_content = Self::clean_content(content);
        Ok(processed_content)
    }

    /// Find the first available editor
    fn find_available_editor() -> String {
        let editors = ["vi", "nano"];

        for editor in &editors {
            if process::Command::new(editor)
                .arg("--version")
                .output()
                .is_ok()
            {
                return editor.to_string();
            }
        }

        // Fallback to vi (should be available on most Unix systems)
        "vi".to_string()
    }

    /// Get the initial template for new entries
    fn get_template() -> String {
        r#"

# Enter your journal entry above this line
# Lines starting with # are comments and will be ignored
# You can use annotations:
#   @person    - to mention people
#   ::project  - to reference projects  
#   +tag       - to add tags
#
# Save and exit to create the entry (:wq in vim)
# Exit without saving to cancel (ZQ in vim or Ctrl+C)
"#
        .to_string()
    }

    /// Clean content by removing comment lines and tempy lines at the beginning
    fn clean_content(content: String) -> String {
        let lines: Vec<&str> = content
            .lines()
            .filter(|line| !line.trim().starts_with('#'))
            .collect();
        lines.join("\n").trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template() {
        let template = Cli::get_template();
        assert!(template.contains("# Enter your journal entry"));
        assert!(template.contains("@person"));
        assert!(template.contains("::project"));
        assert!(template.contains("+tag"));
    }

    #[test]
    fn test_find_available_editor() {
        let editor = Cli::find_available_editor();
        // Should return one of our supported editors or fallback to vi
        assert!(editor == "vi" || editor == "nano");
    }
}
