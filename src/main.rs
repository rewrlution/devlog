use clap::{Parser, Subcommand};
use color_eyre::Result;

mod commands;
mod storage;
mod utils;
mod tui;

#[derive(Parser)]
#[command(name = "devlog")]
#[command(about = "A simple development log CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new entry
    New,
    /// Edit an existing entry
    Edit { id: String },
    /// Show an entry
    Show { id: String },
    /// List entries
    List {
        /// Launch interactive TUI mode
        #[arg(long)]
        interactive: bool,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    
    let cli = Cli::parse();

    match cli.command {
        Commands::New => commands::new::execute(),
        Commands::Edit { id } => commands::edit::execute(id),
        Commands::Show { id } => commands::show::execute(id),
        Commands::List { interactive } => {
            if interactive {
                commands::list::execute_interactive()
            } else {
                commands::list::execute()
            }
        }
    }
}