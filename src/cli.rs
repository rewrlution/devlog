use clap::{Parser, Subcommand};
use crate::entry::Entry;

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
            None => {
                // TODO - open a real text editor here
                // For now, let's use a simple prompt
                println!("Enter your journal entry (press Ctrl+D when finished):");
                use std::io::{self, Read};
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer.trim().to_string()
            }
        };

        if content.is_empty() {
            println!("No content provided. Entry not created.");
            return Ok(());
        }

        let mut entry = Entry::new(content);
        entry.parse_annotations();

        // For now, just print the entry details
        println!("{}", entry.to_markdown());
        
        Ok(())
    }
}