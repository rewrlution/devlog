use std::{fs, path::PathBuf};

use color_eyre::eyre::{Context, ContextCompat, Ok, Result};

pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().wrap_err("Could not find home directory")?;
        let base_path = home.join(".devlog").join("entries");

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path).wrap_err("Failed to create devlog directory")?;

        Ok(Self { base_path })
    }
}
