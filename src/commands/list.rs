use color_eyre::eyre::Result;

pub fn execute(interactive: bool) -> Result<()> {
    if interactive {
        println!("Listing in interactive mode");
    } else {
        println!("List latest 10 entries");
    }

    Ok(())
}
