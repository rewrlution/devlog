use chrono::Utc;
use clap::{Parser, Subcommand};
use crate::entry::Entry;
use std::env;
use std::fs;
use std::process::Command;

/// DevLog - A journal CLI tool for developers
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
    /// Create a new journal entry
    New {
        /// Inline message for quick entry
        #[arg(short, long)]
        message: Option<String>,
    }
}

impl Cli {
    /// Handle the parsed command
    pub fn handle_command(&self)  -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::New { message} => {
                self.handle_new_command(message.clone())
            }
        }
    }

    /// Handle the new command
    fn handle_new_command(&self, message: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let content = match message {
            Some(msg) => msg,
            None => self.open_editor_for_content()?
        };

        if content.is_empty() {
            println!("No content provided. Entry not created.");
            return Ok(());
        }

        // Create entry and parse annotations
        let mut entry = Entry::new(content);
        entry.parse_annotations();

        // Save to local markdown file
        let now = Utc::now();
        let date_str = now.format("%Y%m%d");
        let filename = format!("devlog-{}.md", date_str);

        fs::write(&filename, entry.to_markdown())?;
        println!("Entry saved to {}", filename);
        
        Ok(())
    }

    /// Open a text editor for the user to write content
    fn open_editor_for_content(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Create a temporary file
        let uuid = uuid::Uuid::new_v4();
        let temp_dir = env::temp_dir();
        println!("Temporary directory: {}", temp_dir.display());
        let temp_file = temp_dir.join(format!("devlog-{}.md", uuid));

        // Write initial content with instructions
        let initial_content = r#"

# Enter your journal entry above this line
# Lines starting with # are comments and will be ignored
# You can use annotations:
#   @person    - to mention people
#   ::project  - to reference projects  
#   +tag       - to add tags
#
# Save and exit to create the entry (:wq in vim)
# Exit without saving to cancel (ZQ in vim or Ctrl+C)
"#;
        fs::write(&temp_file, initial_content)?;

        // Get editor (only support `vi` at the moment)
        // Support `nano` and other visual editors in the future
        let editor = "vi".to_string();

        // Open the editor
        // Open the editor
        let status = Command::new(&editor)
            .arg(&temp_file)
            .status()?;

        if !status.success() {
            fs::remove_file(&temp_file).ok();
            return Err("Editor was cancelled or exited with error".into());
        }

        // Read the content back
        let content = fs::read_to_string(&temp_file)?;

        // Clean up the temporary file
        fs::remove_file(&temp_file).ok();

        // Process content - remove comment lines and trim
        let processed_content = content
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect::<Vec<&str>>()
            .join("\n")
            .trim()
            .to_string();

        Ok(processed_content)
    }
}