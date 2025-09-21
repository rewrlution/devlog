use clap::{Parser, Subcommand};

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
    Edit {id: String},
    /// Show an entry
    Show {id: String},
    /// List entries
    List {
        /// Launch interactive TUI mode
        #[arg(short, long)]
        interactive: bool,
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New => handle_new(),
        Commands::Edit { id } => handle_edit(id),
        Commands::Show { id } => hanlde_show(id),   
        Commands::List { interactive } => handle_list(interactive),
    }
}

fn handle_new() {
    println!("Creating new entry...");
}

fn handle_edit(id: String) {
    println!("Editing entry {id}");
}

fn hanlde_show(id: String) {
    println!("Showing entry {id}");
}

fn handle_list(interactive: bool) {
    if interactive {
        println!("Listing in interactive mode");
    } else {
        println!("List latest 10 entries");
    }
}