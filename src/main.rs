mod data;
mod navigation;
mod ui;
mod utils;

use color_eyre::Result;

fn main() -> Result<()> {
    // Set up error handling
    color_eyre::install()?;

    ui::run()?;

    Ok(())
}
