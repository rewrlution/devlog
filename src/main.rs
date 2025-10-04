use clap::{Parser, Subcommand};

use crate::{commands::config::ConfigSubcommand, storage::Storage};

mod commands;
mod models;
mod storage;
mod tui;
mod utils;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new entry
    New {
        /// Entry ID to create (format: YYYYMMDD)
        #[arg(long, value_name = "YYYYMMDD")]
        id: Option<String>,
    },
    /// Edit an existing entry
    Edit {
        /// Entry ID to edit (format: YYYYMMDD)
        #[arg(long, value_name = "YYYYMMDD")]
        id: String,
    },
    /// Show an entry
    Show {
        /// Entry ID to display (format: YYYYMMDD)
        #[arg(long, value_name = "YYYYMMDD")]
        id: String,
    },
    /// List entries
    List {
        /// Launch interactive TUI mode
        #[arg(short, long)]
        interactive: bool,
    },
    /// Configure Devlog settings
    Config {
        #[command(subcommand)]
        subcmd: Option<ConfigSubcommand>,
    },
}

fn main() {
    let cli = Cli::parse();
    let storage = Storage::new(None).unwrap_or_else(|e| {
        eprintln!("Failed to initialize storage: {}", e);
        std::process::exit(1);
    });

    if let Err(e) = match cli.command {
        Commands::New { id } => commands::new::execute(&storage, id),
        Commands::Edit { id } => commands::edit::execute(&storage, id),
        Commands::Show { id } => commands::show::execute(&storage, id),
        Commands::List { interactive } => commands::list::execute(&storage, interactive),
        Commands::Config { subcmd } => commands::config::execute(subcmd),
    } {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
