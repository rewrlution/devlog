use clap::{Parser, Subcommand};

use crate::{commands::config::ConfigSubcommand, storage::Storage};

mod commands;
mod config;
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
    /// Configure DevLog settings
    Config {
        #[command(subcommand)]
        subcmd: Option<ConfigSubcommand>,
    },
}

fn main() {
    // Initialize color-eyre for better error reporting
    color_eyre::install().expect("Failed to install color-eyre");

    let cli = Cli::parse();

    // Handle config command separately since it doesn't need storage
    if let Commands::Config { subcmd } = cli.command {
        if let Err(e) = commands::config::execute(subcmd) {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Initialize storage from configuration
    let storage = Storage::from_config().unwrap_or_else(|e| {
        eprintln!("Failed to initialize storage: {}", e);
        eprintln!("Try running 'devlog config' to set up your configuration.");
        std::process::exit(1);
    });

    if let Err(e) = match cli.command {
        Commands::New { id } => commands::new::execute(&storage, id),
        Commands::Edit { id } => commands::edit::execute(&storage, id),
        Commands::Show { id } => commands::show::execute(&storage, id),
        Commands::List { interactive } => commands::list::execute(&storage, interactive),
        Commands::Config { .. } => unreachable!(), // Already handled above
    } {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
