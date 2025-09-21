use clap::{Parser, Subcommand};

mod commands;
mod models;
mod storage;

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
    New,
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New => commands::new::execute(),
        Commands::Edit { id } => commands::edit::execute(id),
        Commands::Show { id } => commands::show::execute(id),
        Commands::List { interactive } => commands::list::execute(interactive),
    }
}
