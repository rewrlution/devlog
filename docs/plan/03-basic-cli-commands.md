# Step 3: Basic CLI Commands

## Overview

Implement the core CLI commands: `new`, `edit`, `show`, and `list` (non-interactive mode).

## CLI Structure with Clap

Create/update `src/main.rs`:

```rust
use clap::{Parser, Subcommand};
use color_eyre::Result;

mod commands;
mod storage;
mod utils;

#[derive(Parser)]
#[command(name = "devlog")]
#[command(about = "A simple development log CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new entry
    New,
    /// Edit an existing entry
    Edit { id: String },
    /// Show an entry
    Show { id: String },
    /// List entries
    List {
        /// Launch interactive TUI mode
        #[arg(long)]
        interactive: bool,
    },
    /// Generate insights from entries
    Insight,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New => commands::new::execute(),
        Commands::Edit { id } => commands::edit::execute(id),
        Commands::Show { id } => commands::show::execute(id),
        Commands::List { interactive } => {
            if interactive {
                commands::list::execute_interactive()
            } else {
                commands::list::execute()
            }
        }
        Commands::Insight => commands::insight::execute(),
    }
}
```

## Command Implementations

### 1. New Command (`src/commands/new.rs`)

```rust
use crate::storage::{Storage, entry::Entry};
use crate::utils::editor;
use color_eyre::Result;

pub fn execute() -> Result<()> {
    println!("Creating new entry...");

    // Get current date as entry ID
    let today = chrono::Utc::now().format("%Y%m%d").to_string();

    // Check if entry already exists for today
    let storage = Storage::new()?;
    if let Ok(_) = storage.load_entry(&today) {
        println!("Entry for {} already exists. Use 'devlog edit {}' to modify it.", today, today);
        return Ok(());
    }

    // Launch editor with template
    let template = format!(
        "# Development Log - {}\n\n## What I worked on today\n\n\n## What I learned\n\n\n## Next steps\n\n",
        today
    );

    let content = editor::launch_editor(&template)?;

    // Create and save entry
    let entry = Entry::new(content);
    storage.save_entry(&entry)?;

    println!("Entry created successfully: {}", today);
    Ok(())
}
```

### 2. Edit Command (`src/commands/edit.rs`)

```rust
use crate::storage::Storage;
use crate::utils::editor;
use color_eyre::{eyre::Context, Result};

pub fn execute(id: String) -> Result<()> {
    let storage = Storage::new()?;

    // Load existing entry
    let mut entry = storage.load_entry(&id)
        .wrap_err_with(|| format!("Entry '{}' not found", id))?;

    println!("Editing entry: {}", id);

    // Launch editor with existing content
    let new_content = editor::launch_editor(&entry.content)?;

    // Update entry
    entry.update_content(new_content);
    storage.save_entry(&entry)?;

    println!("Entry updated successfully: {}", id);
    Ok(())
}
```

### 3. Show Command (`src/commands/show.rs`)

```rust
use crate::storage::Storage;
use color_eyre::{eyre::Context, Result};

pub fn execute(id: String) -> Result<()> {
    let storage = Storage::new()?;

    let entry = storage.load_entry(&id)
        .wrap_err_with(|| format!("Entry '{}' not found", id))?;

    // Display entry with metadata
    println!("# Entry: {}", entry.id);
    println!("Created: {}", entry.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("Updated: {}", entry.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("---");
    println!("{}", entry.content);

    Ok(())
}
```

### 4. List Command (`src/commands/list.rs`)

```rust
use crate::storage::Storage;
use color_eyre::Result;

pub fn execute() -> Result<()> {
    let storage = Storage::new()?;
    let entries = storage.list_entries()?;

    if entries.is_empty() {
        println!("No entries found. Create your first entry with 'devlog new'");
        return Ok(());
    }

    println!("Recent entries (last 20):");
    println!();

    for (i, entry_id) in entries.iter().take(20).enumerate() {
        println!("  {}. {}", i + 1, entry_id);
    }

    println!();
    println!("Commands:");
    println!("  devlog show <id>     - View an entry");
    println!("  devlog edit <id>     - Edit an entry");
    println!("  devlog list --interactive - Launch TUI mode");

    Ok(())
}

pub fn execute_interactive() -> Result<()> {
    // Placeholder for TUI mode (will implement in Step 5)
    println!("Interactive TUI mode coming in Step 5!");
    println!("For now, use: devlog list");
    Ok(())
}
```

## Editor Integration (`src/utils/editor.rs`)

```rust
use color_eyre::{eyre::Context, Result};
use std::fs;
use std::process::Command;

pub fn launch_editor(initial_content: &str) -> Result<String> {
    // Create temporary file
    let temp_path = std::env::temp_dir().join("devlog_temp.md");
    fs::write(&temp_path, initial_content)
        .wrap_err("Failed to create temporary file")?;

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
    let content = fs::read_to_string(&temp_path)
        .wrap_err("Failed to read temporary file")?;

    // Clean up
    let _ = fs::remove_file(&temp_path);

    Ok(content)
}
```

## Module Declarations

Update `src/lib.rs`:

```rust
pub mod commands;
pub mod storage;
pub mod utils;
```

Create `src/commands/mod.rs`:

```rust
pub mod new;
pub mod edit;
pub mod show;
pub mod list;
pub mod insight;
```

Create `src/utils/mod.rs`:

```rust
pub mod editor;
pub mod parser;
```

## Key Rust Concepts Explained

- **Subcommands**: Clap's way of handling `git commit`, `git push` style commands
- **match**: Like switch/case but more powerful - handles all possible enum variants
- **?**: The "try" operator - if Result is Err, return early with that error
- **wrap_err()**: Adds more helpful error messages (color-eyre version of anyhow's context)
- **eyre::bail!**: Early return with an error message (color-eyre version of anyhow::bail!)

## Implementation Tasks

1. **Update `src/main.rs`** with the CLI structure
2. **Create all command modules** in `src/commands/`
3. **Create `src/utils/editor.rs`** for Vim integration
4. **Update module declarations** in `mod.rs` files
5. **Test basic commands** by running `cargo run new`, `cargo run list`, etc.

## Testing Your Implementation

```bash
# Create a new entry
cargo run new

# List entries
cargo run list

# Show an entry
cargo run show 20240920

# Edit an entry
cargo run edit 20240920
```

## Next Steps

Move to Step 4: Annotation Parsing and Insights to implement the `devlog insight` command.
