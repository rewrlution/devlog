use crate::entry::Entry;
use crate::storage::{EntryStorage, LocalEntryStorage};
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
    /// Show a specific entry
    Show {
        /// Entry ID to display (format: YYYYMMDD)
        #[arg(value_name = "YYYYMMDD")]
        id: String,
    },
}

impl Cli {
    /// Run the CLI application
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();
        // TODO: read user defined storage path
        // For now, we use the default `base_dir`, which is `~/.devlog`
        let storage = LocalEntryStorage::new(None)?;

        match cli.command {
            Commands::New { message, id } => {
                Self::handle_new_command(message, id, &storage)?;
            }
            Commands::Edit { id } => {
                Self::handle_edit_command(id, &storage)?;
            }
            Commands::Show { id } => {
                Self::handle_show_command(id, &storage)?;
            }
        }

        Ok(())
    }

    /// Handle the new subcommand
    fn handle_new_command(
        message: Option<String>,
        custom_id: Option<String>,
        storage: &dyn EntryStorage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate custom ID format if provided
        if let Some(ref id) = custom_id {
            Self::validate_id_format(id)?;
        }
        // Generate ID: use custom ID if provided, otherwise use current date
        let id = custom_id.unwrap_or_else(|| format!("{}", Local::now().format("%Y%m%d")));

        // Check if entry already exists to prevent data loss
        if Entry::load(&id, storage)?.is_some() {
            eprintln!("Entry with ID '{}' already exists.", id);
            eprintln!("To edit the existing entry, use: devlog edit --id {}", id);
            process::exit(1);
        }

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
        storage: &dyn EntryStorage,
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

    /// Handle the show subcommand
    fn handle_show_command(
        id: String,
        storage: &EntryStorage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate ID format
        Self::validate_id_format(&id)?;

        // Load the entry
        let entry = match Entry::load(&id, storage)? {
            Some(entry) => entry,
            None => {
                eprintln!("Entry with ID '{}' not found.", id);
                process::exit(1);
            }
        };

        Self::display_default_format(&entry);

        Ok(())
    }

    /// Display entry in default human-readable format
    fn display_default_format(entry: &Entry) {
        let state = entry.current_state();

        println!("Entry: {}", state.id);
        println!("Created: {}", state.created_at.format("%Y-%m-%d %H:%M:%S"));
        println!("Updated: {}", state.updated_at.format("%Y-%m-%d %H:%M:%S"));
        println!();

        // Display metadata if present
        if !state.people.is_empty() || !state.projects.is_empty() || !state.tags.is_empty() {
            println!("Metadata:");
            if !state.people.is_empty() {
                println!("  People: {}", state.people.join(", "));
            }
            if !state.projects.is_empty() {
                println!("  Projects: {}", state.projects.join(", "));
            }
            if !state.tags.is_empty() {
                println!("  Tags: {}", state.tags.join(", "));
            }
            println!();
        }

        println!("Content:");
        println!("{}", Self::highlight_annotations(&state.content));
    }

    /// Highlight annotations in content for better readability
    fn highlight_annotations(content: &str) -> String {
        // For now, just return the content as-is
        // In the future, we could add color highlighting for @person, ::project, +tag
        content.to_string()
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
    use tempfile::TempDir;

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

    #[test]
    fn test_validate_id_format_valid() {
        assert!(Cli::validate_id_format("20250905").is_ok());
        assert!(Cli::validate_id_format("20231231").is_ok());
        assert!(Cli::validate_id_format("20240229").is_ok()); // Leap year
    }

    #[test]
    fn test_validate_id_format_invalid() {
        assert!(Cli::validate_id_format("2025905").is_err()); // Too short
        assert!(Cli::validate_id_format("202509055").is_err()); // Too long
        assert!(Cli::validate_id_format("20250932").is_err()); // Invalid day
        assert!(Cli::validate_id_format("20251301").is_err()); // Invalid month
        assert!(Cli::validate_id_format("abcd1234").is_err()); // Non-numeric
        assert!(Cli::validate_id_format("").is_err()); // Empty
    }

    #[test]
    fn test_highlight_annotations() {
        let content = "Worked with @alice on ::project using +rust";
        let result = Cli::highlight_annotations(content);
        // For now, it should just return the content as-is
        assert_eq!(result, content);
    }

    #[test]
    fn test_show_command_with_valid_entry() {
        let temp_dir = TempDir::new().unwrap();
        let storage = EntryStorage::new(Some(temp_dir.path().to_path_buf())).unwrap();

        // Create a test entry
        let entry = Entry::new(
            "Test content with @alice and +rust".to_string(),
            "20250905".to_string(),
        );
        entry.save(&storage).unwrap();

        // Test show command - should not panic
        let result = Cli::handle_show_command("20250905".to_string(), &storage);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_command_with_invalid_id_format() {
        let temp_dir = TempDir::new().unwrap();
        let storage = EntryStorage::new(Some(temp_dir.path().to_path_buf())).unwrap();

        // Test with invalid ID format
        let result = Cli::handle_show_command("invalid".to_string(), &storage);
        assert!(result.is_err());
    }

    #[test]
    fn test_display_function_with_empty_annotations() {
        // Create a test entry with no annotations
        let entry = Entry::new(
            "Simple content with no annotations".to_string(),
            "20250905".to_string(),
        );

        // These functions should handle empty annotations gracefully
        Cli::display_default_format(&entry);
    }
}
