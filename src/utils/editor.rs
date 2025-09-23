use std::fs;
use std::process::Command;

use color_eyre::eyre::{bail, Context, Result};

pub fn launch_editor(init_content: &str) -> Result<String> {
    // Create a temporary file
    let temp_path = std::env::temp_dir().join("devlog_temp.md");
    fs::write(&temp_path, init_content).wrap_err("Failed to create temporary file")?;

    // Get editor from environment or default to vim
    let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());

    // Launch editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .wrap_err_with(|| format!("Failed to launch editor: {}", editor))?;

    if !status.success() {
        // bail!() macro immediately returns an error from the current function.
        // bail!("Something went wrong");
        // is equivalent to:
        // return Err(eyre!("Something went wrong"));
        bail!("Editor exited with error");
    }

    // Read modified content
    let content = fs::read_to_string(&temp_path).wrap_err("Failed to read temporary file")?;

    // Clean up
    let _ = fs::remove_file(temp_path);

    Ok(content)
}
