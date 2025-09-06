use crate::storage::EntryStorage;
use crate::{entry::Entry, storage};
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
            Commands::New { message } => {
                Self::handle_new_command(message, &storage);
            }
        }

        Ok(())
    }

    /// Handle the new subcommand
    fn handle_new_command(
        message: Option<String>,
        storage: &EntryStorage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = match message {
            Some(msg) => msg,
            None => Self::open_editor_for_content()?,
        };

        Ok(())
    }

    /// Open a text editor for the user to write content
    fn open_editor_for_content() -> Result<String, Box<dyn std::error::Error>> {
        // Create a temporary file for editing
        let temp_file = tempfile::NamedTempFile::new()?;
        let temp_path = temp_file.path();

        // Write initial content with instructions
        let init_content = Self::get_template();
        std::fs::write(temp_path, init_content);

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
