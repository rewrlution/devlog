mod annotations;
mod cli;
mod config;
mod entry;
mod events;
mod remote;
mod storage;
mod sync;

use cli::Cli;
use std::process;

#[tokio::main]
async fn main() {
    if let Err(e) = Cli::run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
