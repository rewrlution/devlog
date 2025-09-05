use chrono::Utc;
use clap::Error;
use clap::{Parser, Subcommand};
use crate::entry::Entry;
use std::env;
use std::fs;
use std::path::PathBuf;
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
    },
    // Edit an existing journal entry
    // Edit {
    //     /// Entry ID to edit (format: YYYYMMDD)
    //     id: String,
    // }
}

impl Cli {
    /// Handle the parsed command
    pub fn handle_command(&self)  -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Commands::New { message} => {
                self.handle_new_command(message.clone())
            },
            // Commands::Edit { id } => {
            //     self.handle_edit_command(id.clone())
            // }
        }
    }

    /// Handle the edit command
    // fn handle_edit_command(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
    //     // Get the devlog directory
    //     let devlog_id = self.get_devlog_directory()?;

    //     // Construct the filepath
    //     let filename = format!("devlog-{}.md", id);
    //     let filepath = devlog_id.join(filename);

    //     // Check if the file exists
    //     if !filepath.exists() {
    //         return Err(format!("Entry with ID: '{}' not found at {}", id, filepath.display()).into());
    //     }

    //     // Read the existing file and content
    //     let existing_content = fs::read_to_string(&filepath)?;
    //     let content = self.extract_content_from_markdown(&existing_content)?;

    //     // Open editor with the existing content
    //     let new_content = self.open_editor_with_content(&content)?;
        
    //     if new_content.is_empty() {
    //         println!("No content provided. Entry not updated.");
    //         return Ok(());
    //     }

    //     // Create a new entry with the updated content but preserve the original ID and date

    // }

    /// Extract content form markdown file (skip frontmatter)
    fn extract_content_from_markdown(&self, markdown: &str) -> Result<String, Box<dyn std::error::Error>> {
        let lines: Vec<&str> = markdown.lines().collect();

        // Find the end of frontmatter (second ---)
        let mut frontmatter_end = None;
        let mut in_frontmatter = false;

        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "---" {
                if in_frontmatter {
                    frontmatter_end = Some(i);
                    break;
                } else {
                    in_frontmatter = true;
                }
            }
        }

        match frontmatter_end {
            Some(end_idx) => {
                // Extract content after frontmatter (skip the --- line and any empty lines)
                let content_lines = &lines[end_idx + 1..];
                let content = content_lines.join("\n").trim().to_string();
                Ok(content)
            },
            None => {
                // No frontmatter found, return entire content
                // Technically speaking, this condition won't exist
                // since every entry has a frontmatter.
                // It will only happen when there's some parsing error
                // I am not sure if I should make the logic super defensive
                Ok(markdown.to_string())
            }
        }
    }

    /// Handle the new command
    fn handle_new_command(&self, message: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        let content = match message {
            Some(msg) => msg,
            None => self.open_editor_with_content(&"".to_string())?
        };

        if content.is_empty() {
            println!("No content provided. Entry not created.");
            return Ok(());
        }

        // Create entry and parse annotations
        let mut entry = Entry::new(content);
        entry.parse_annotations();

        // Get or crete the devlog directory
        let devlog_dir = self.get_devlog_directory()?;
        fs::create_dir_all(&devlog_dir)?;

        // Save to local markdown file
        let now = Utc::now();
        let date_str = now.format("%Y%m%d");
        let filename = format!("devlog-{}.md", date_str);
        let filepath = devlog_dir.join(&filename);

        fs::write(&filepath, entry.to_markdown())?;
        println!("Entry saved to {}", filepath.display());
        
        Ok(())
    }

    /// Get the appropriate devlog directory based on the platform
    fn get_devlog_directory(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home_dir = dirs::home_dir()
            .ok_or("Could not find home directory")?;

        // All platforms use ~/Documents/devlog for user accessibility
        // macOS: ~/Documents/devlog
        // Windows: %USERPROFILE%\Documents\devlog
        // Linux: ~/Documents/devlog
        let devlog_dir = home_dir.join("Documents").join("devlog");

        Ok(devlog_dir)
    }

    /// Open a text editor for the user to write content
    /// When initial_content is not None, the text editor should have pre-filled content
    fn open_editor_with_content(&self, initial_content: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Create a temporary file
        let uuid = uuid::Uuid::new_v4();
        let temp_dir = env::temp_dir();
        println!("Temporary directory: {}", temp_dir.display());
        let temp_file = temp_dir.join(format!("devlog-{}.md", uuid));

        // Write initial content with instructions
        let content_with_instructions = format!(r#"{}

# Enter your journal entry above this line
# Lines starting with # are comments and will be ignored
# You can use annotations:
#   @person    - to mention people
#   ::project  - to reference projects  
#   +tag       - to add tags
#
# Save and exit to create the entry (:wq in vim)
# Exit without saving to cancel (ZQ in vim or Ctrl+C)
"#, initial_content);
        fs::write(&temp_file, content_with_instructions)?;

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