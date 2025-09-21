use color_eyre::{eyre::Context, Result};
use std::fs;
use std::process::Command;

pub fn launch_editor(initial_content: &str) -> Result<String> {
    // Create temporary file
    let temp_path = std::env::temp_dir().join("devlog_temp.md");
    fs::write(&temp_path, initial_content).wrap_err("Failed to create temporary file")?;

    // Get editor from environment or default to vim
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    // Launch editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .wrap_err_with(|| format!("Failed to launch editor: {}", editor))?;

    if !status.success() {
        color_eyre::eyre::bail!("Editor exited with error");
    }

    // Read modified content
    let content = fs::read_to_string(&temp_path).wrap_err("Failed to read temporary file")?;

    // Clean up
    let _ = fs::remove_file(&temp_path);

    Ok(content)
}
