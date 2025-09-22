use color_eyre::eyre::Result;

pub fn execute(id: String) -> Result<()> {
    println!("Editing entry {id}");

    Ok(())
}
