use std::{fs, process};

use color_eyre::eyre::{bail, Context, Result};

/// Open a text editor for users to write content
pub fn launch_editor(existing_content: Option<&str>) -> Result<String> {
    // Create a temporary file
    let temp_path = std::env::temp_dir().join("devlog_temp.md");

    let init_content = match existing_content {
        Some(content) => format!("{}\n{}", content, get_template()),
        None => get_template(),
    };

    fs::write(&temp_path, init_content).wrap_err("Failed to create temporary file")?;

    // Get editor from environment or default to vim
    let editor = find_available_editor();

    // Launch editor
    let status = process::Command::new(&editor)
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

    // Clean the content by removing comment lines
    let processed_content = clean_content(content);

    // Clean up
    let _ = fs::remove_file(temp_path);

    Ok(processed_content)
}

/// Find the first available editor
fn find_available_editor() -> String {
    let editors = ["vi", "nano"];

    for editor in editors {
        if process::Command::new(editor)
            .arg("--version")
            .output()
            .is_ok()
        {
            return editor.to_string();
        }
    }

    // Fallback to vi (should be available on most unix system)
    "vi".to_string()
}

/// Get the initial template for new entries
fn get_template() -> String {
    r#"

# Enter your journal entry above this line
# Lines starting with # are comments and will be ignored
# You can use annotations:
#   @person    - to mention people
#   ::project  - to reference projects  
#   +tag       - to add tags
#
# Save and exit to create the entry (:wq in vim)
# Exit without saving to cancel (ZQ in vim or Ctrl+C)
"#
    .to_string()
}

/// Clean content by removing comment lines and empty lines at the beginning
fn clean_content(content: String) -> String {
    let lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim().starts_with('#'))
        .collect();
    lines.join("\n").trim().to_string()
}
