use clap::{Parser, Subcommand};
use color_eyre::Result;

mod commands;
mod storage;
mod utils;
mod tui;
mod sync;

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
    /// Sync entries with cloud storage
    Sync {
        #[command(subcommand)]
        command: commands::sync::SyncCommands,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;
    
    let cli = Cli::parse();

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

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
        Commands::Sync { command } => {
            rt.block_on(commands::sync::handle_sync_command(command))
        }
    }
}