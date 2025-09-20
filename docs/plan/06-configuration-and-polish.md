# Step 6: Configuration and Final Polish

## Overview

Add configuration file support, improve error handling, and add final polish features for the MVP.

## Configuration Structure (`src/config.rs`)

```rust
use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub editor: String,
    pub entries_path: Option<String>,
    pub known_people: Vec<String>,
    pub known_projects: Vec<String>,
    pub known_tags: Vec<String>,
    pub templates: Templates,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Templates {
    pub daily: String,
    pub weekly: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string()),
            entries_path: None, // Will use ~/.devlog/entries by default
            known_people: vec![
                "alice".to_string(),
                "bob".to_string(),
                "charlie".to_string(),
            ],
            known_projects: vec![
                "search-service".to_string(),
                "auth-system".to_string(),
                "devlog".to_string(),
            ],
            known_tags: vec![
                "rust".to_string(),
                "productivity".to_string(),
                "meeting".to_string(),
                "learning".to_string(),
            ],
            templates: Templates {
                daily: "# Development Log - {date}\n\n## What I worked on today\n\n\n## What I learned\n\n\n## Next steps\n\n".to_string(),
                weekly: "# Weekly Review - {date}\n\n## Accomplishments this week\n\n\n## Challenges faced\n\n\n## Goals for next week\n\n".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .wrap_err("Failed to read config file")?;
            let config: Config = serde_yaml::from_str(&content)
                .wrap_err("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .wrap_err("Failed to create config directory")?;
        }

        let content = serde_yaml::to_string(self)
            .wrap_err("Failed to serialize config")?;
        fs::write(&config_path, content)
            .wrap_err("Failed to write config file")?;

        println!("Config saved to: {}", config_path.display());
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .wrap_err("Could not find home directory")?;
        Ok(home.join(".devlog").join("config.yml"))
    }

    pub fn get_entries_path(&self) -> Result<PathBuf> {
        if let Some(custom_path) = &self.entries_path {
            Ok(PathBuf::from(custom_path))
        } else {
            let home = dirs::home_dir()
                .wrap_err("Could not find home directory")?;
            Ok(home.join(".devlog").join("entries"))
        }
    }

    pub fn format_template(&self, template_name: &str, date: &str) -> String {
        let template = match template_name {
            "weekly" => &self.templates.weekly,
            _ => &self.templates.daily,
        };

        template.replace("{date}", date)
    }
}
```

## Enhanced Storage with Config (`src/storage/mod.rs`)

```rust
pub mod entry;

use crate::config::Config;
use crate::storage::entry::Entry;
use color_eyre::{eyre::Context, Result};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct Storage {
    base_path: PathBuf,
    config: Config,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let base_path = config.get_entries_path()?;

        // Create directory if it doesn't exist
        fs::create_dir_all(&base_path)
            .wrap_err("Failed to create devlog directory")?;

        Ok(Self { base_path, config })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    // ... rest of the Storage implementation from Step 2 ...
    // (copy the save_entry, load_entry, list_entries, serialize_entry, deserialize_entry methods)
}
```

## Enhanced Commands with Templates

### Update New Command (`src/commands/new.rs`)

```rust
use crate::storage::{Storage, entry::Entry};
use crate::utils::editor;
use color_eyre::Result;
use clap::Args;

#[derive(Args)]
pub struct NewArgs {
    /// Use weekly template instead of daily
    #[arg(long)]
    weekly: bool,
}

pub fn execute(args: Option<NewArgs>) -> Result<()> {
    let storage = Storage::new()?;
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Check if entry already exists for today
    if let Ok(_) = storage.load_entry(&today) {
        println!("Entry for {} already exists. Use 'devlog edit {}' to modify it.", today, today);
        return Ok(());
    }

    // Determine template
    let template_name = if args.as_ref().map_or(false, |a| a.weekly) {
        "weekly"
    } else {
        "daily"
    };

    let template = storage.config().format_template(template_name, &today);

    println!("Creating new {} entry...", template_name);

    // Launch editor with template
    let content = editor::launch_editor(&template, storage.config())?;

    if content.trim().is_empty() {
        println!("Empty entry, not saving.");
        return Ok(());
    }

    // Create and save entry
    let entry = Entry::new(content);
    storage.save_entry(&entry)?;

    println!("Entry created successfully: {}", today);
    Ok(())
}
```

### Enhanced Editor Integration (`src/utils/editor.rs`)

```rust
use crate::config::Config;
use color_eyre::{eyre::Context, Result};
use std::fs;
use std::process::Command;

pub fn launch_editor(initial_content: &str, config: &Config) -> Result<String> {
    // Create temporary file
    let temp_path = std::env::temp_dir().join("devlog_temp.md");
    fs::write(&temp_path, initial_content)
        .wrap_err("Failed to create temporary file")?;

    // Use configured editor
    let editor = &config.editor;

    println!("Launching editor: {}", editor);

    // Launch editor
    let status = Command::new(editor)
        .arg(&temp_path)
        .status()
        .wrap_err_with(|| format!("Failed to launch editor: {}. Check your config or set EDITOR environment variable.", editor))?;

    if !status.success() {
        color_eyre::eyre::bail!("Editor exited with error code: {:?}", status.code());
    }

    // Read modified content
    let content = fs::read_to_string(&temp_path)
        .wrap_err("Failed to read temporary file")?;

    // Clean up
    let _ = fs::remove_file(&temp_path);

    Ok(content)
}

// Backwards compatibility
pub fn launch_editor_simple(initial_content: &str) -> Result<String> {
    let config = Config::default();
    launch_editor(initial_content, &config)
}
```

## Configuration Command

Add to `src/commands/mod.rs`:

```rust
pub mod new;
pub mod edit;
pub mod show;
pub mod list;
pub mod insight;
pub mod config;
```

Create `src/commands/config.rs`:

```rust
use crate::config::Config;
use color_eyre::Result;
use clap::Args;

#[derive(Args)]
pub struct ConfigArgs {
    /// Show current configuration
    #[arg(long)]
    show: bool,

    /// Reset to default configuration
    #[arg(long)]
    reset: bool,

    /// Set editor
    #[arg(long)]
    editor: Option<String>,

    /// Add known person
    #[arg(long)]
    add_person: Option<String>,

    /// Add known project
    #[arg(long)]
    add_project: Option<String>,

    /// Add known tag
    #[arg(long)]
    add_tag: Option<String>,
}

pub fn execute(args: ConfigArgs) -> Result<()> {
    if args.reset {
        let config = Config::default();
        config.save()?;
        println!("Configuration reset to defaults.");
        return Ok(());
    }

    let mut config = Config::load()?;
    let mut changed = false;

    if let Some(editor) = args.editor {
        config.editor = editor;
        changed = true;
        println!("Editor set to: {}", config.editor);
    }

    if let Some(person) = args.add_person {
        if !config.known_people.contains(&person) {
            config.known_people.push(person.clone());
            changed = true;
            println!("Added person: @{}", person);
        } else {
            println!("Person @{} already in config", person);
        }
    }

    if let Some(project) = args.add_project {
        if !config.known_projects.contains(&project) {
            config.known_projects.push(project.clone());
            changed = true;
            println!("Added project: ::{}", project);
        } else {
            println!("Project ::{} already in config", project);
        }
    }

    if let Some(tag) = args.add_tag {
        if !config.known_tags.contains(&tag) {
            config.known_tags.push(tag.clone());
            changed = true;
            println!("Added tag: +{}", tag);
        } else {
            println!("Tag +{} already in config", tag);
        }
    }

    if changed {
        config.save()?;
    }

    if args.show || (!changed && !args.reset) {
        show_config(&config);
    }

    Ok(())
}

fn show_config(config: &Config) {
    println!("📝 Devlog Configuration");
    println!("========================");
    println!();
    println!("Editor: {}", config.editor);

    if let Some(path) = &config.entries_path {
        println!("Entries path: {}", path);
    } else {
        println!("Entries path: ~/.devlog/entries (default)");
    }

    println!();
    println!("Known people ({}):", config.known_people.len());
    for person in &config.known_people {
        println!("  @{}", person);
    }

    println!();
    println!("Known projects ({}):", config.known_projects.len());
    for project in &config.known_projects {
        println!("  ::{}", project);
    }

    println!();
    println!("Known tags ({}):", config.known_tags.len());
    for tag in &config.known_tags {
        println!("  +{}", tag);
    }

    println!();
    println!("Templates:");
    println!("  Daily: {} characters", config.templates.daily.len());
    println!("  Weekly: {} characters", config.templates.weekly.len());
}
```

## Updated Main CLI (`src/main.rs`)

```rust
use clap::{Parser, Subcommand};
use color_eyre::Result;

mod commands;
mod storage;
mod utils;
mod config;

#[derive(Parser)]
#[command(name = "devlog")]
#[command(about = "A simple development log CLI tool")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new entry
    New {
        #[command(flatten)]
        args: Option<commands::new::NewArgs>,
    },
    /// Edit an existing entry
    Edit {
        id: String
    },
    /// Show an entry
    Show {
        id: String
    },
    /// List entries
    List {
        /// Launch interactive TUI mode
        #[arg(long)]
        interactive: bool,
    },
    /// Generate insights from entries
    Insight,
    /// Manage configuration
    Config {
        #[command(flatten)]
        args: commands::config::ConfigArgs,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { args } => commands::new::execute(args),
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
        Commands::Config { args } => commands::config::execute(args),
    }
}
```

## Enhanced Error Messages

Create `src/utils/errors.rs`:

```rust
use color_eyre::Result;

pub fn setup_error_handling() -> Result<()> {
    color_eyre::install()?;
    Ok(())
}

pub fn handle_error(error: color_eyre::Report) {
    eprintln!("❌ Error: {}", error);

    // Provide helpful suggestions based on error type
    let error_str = error.to_string().to_lowercase();

    if error_str.contains("no such file or directory") {
        eprintln!("💡 Tip: Make sure the entry ID exists. Use 'devlog list' to see available entries.");
    } else if error_str.contains("editor") {
        eprintln!("💡 Tip: Set your preferred editor with 'devlog config --editor <editor>' or set the EDITOR environment variable.");
    } else if error_str.contains("permission denied") {
        eprintln!("💡 Tip: Check file permissions for ~/.devlog/ directory.");
    }
}

// Update main.rs to use this:
fn main() {
    if let Err(error) = setup_error_handling().and_then(|_| run()) {
        handle_error(error);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { args } => commands::new::execute(args),
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
        Commands::Config { args } => commands::config::execute(args),
    }
}
```

## Final Implementation Tasks

1. **Create `src/config.rs`** with configuration management
2. **Update `src/storage/mod.rs`** to use config
3. **Update commands** to use config and templates
4. **Create config command** for management
5. **Add better error handling** and user messages
6. **Update `src/lib.rs`** to include config module

## Testing the Complete Application

```bash
# Initialize config
cargo run config --show

# Create entry with weekly template
cargo run new --weekly

# Add some known people/projects
cargo run config --add-person alice
cargo run config --add-project devlog

# Test all features
cargo run new
cargo run list
cargo run list --interactive
cargo run insight
cargo run show 2024-09-20

# Test configuration
cargo run config --editor nano
cargo run config --show
```

## Final File Structure

```
src/
├── main.rs
├── lib.rs
├── config.rs
├── commands/
│   ├── mod.rs
│   ├── new.rs
│   ├── edit.rs
│   ├── list.rs
│   ├── show.rs
│   ├── insight.rs
│   └── config.rs
├── storage/
│   ├── mod.rs
│   └── entry.rs
├── tui/
│   ├── mod.rs
│   └── app.rs
└── utils/
    ├── mod.rs
    ├── editor.rs
    ├── parser.rs
    └── errors.rs
```

## Key Features Completed ✅

- ✅ Basic entry management (`new`, `edit`, `show`, `list`)
- ✅ Interactive TUI with tree navigation
- ✅ Mechanical annotation parsing (`@`, `::`, `+`)
- ✅ Insights and analytics
- ✅ Configuration management
- ✅ Templates for daily/weekly entries
- ✅ Vim integration
- ✅ Local file storage

## Bonus Features to Consider (Future)

- Search functionality within TUI
- Export to different formats (HTML, PDF)
- Git integration for versioning
- Backup/sync commands
- Better Markdown rendering in TUI
- Tab completion for known annotations

Your MVP is now complete! 🎉
