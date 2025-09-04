mod entry;
mod cli;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.handle_command() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}