# Task 05: CLI Command Structure Implementation

**Estimated Time**: 1-2 hours  
**Difficulty**: ⭐⭐ Beginner-Intermediate  
**Prerequisites**: Tasks 01, 02, 03, and 04 completed

## Objective

Add the `push` command to the existing CLI interface, integrating it with the current clap-based command structure.

## What You'll Learn

- Working with clap derive macros
- Command enumeration and argument parsing
- Integration with existing CLI patterns
- Basic command validation and error handling

## Tasks

### 1. Define Push Mode Enum

First, add the `PushMode` enum to `src/cli.rs`. Add this near the top of the file after the existing imports:

```rust
/// Push mode for uploading files
#[derive(Debug, Clone)]
pub enum PushMode {
    /// Upload only changed files since last push
    Incremental,
    /// Upload all files regardless of changes
    All,
}

impl std::str::FromStr for PushMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "incremental" => Ok(PushMode::Incremental),
            "all" => Ok(PushMode::All),
            _ => Err(format!(
                "Invalid push mode: '{}'. Valid options: 'incremental', 'all'",
                s
            )),
        }
    }
}

impl std::fmt::Display for PushMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PushMode::Incremental => write!(f, "incremental"),
            PushMode::All => write!(f, "all"),
        }
    }
}
```

### 2. Add Push Command to Commands Enum

Update the `Commands` enum in `src/cli.rs` to include the new Push command:

```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Create a new entry
    New {
        /// Inline message for the entry
        #[arg(short, long)]
        message: Option<String>,
        /// Optional ID for the entry (format: YYYMMDD)
        #[arg(long, value_name = "YYYYMMDD")]
        id: Option<String>,
    },
    /// Edit an existing entry
    Edit {
        /// Entry ID to edit (format: YYYYMMDD)
        #[arg(long, value_name = "YYYYMMDD")]
        id: String,
    },
    /// Show a specific entry
    Show {
        /// Entry ID to display (format: YYYYMMDD)
        #[arg(value_name = "YYYYMMDD")]
        id: String,
        /// Display human-readable format instead of raw markdown content
        #[arg(long)]
        formatted: bool,
    },
    /// List all entries
    List,
    /// Push local changes to remote storage
    Push {
        /// Push mode: 'incremental' (default) or 'all'
        #[arg(long, default_value = "incremental")]
        mode: PushMode,
        /// Force push even if there are no changes
        #[arg(long)]
        force: bool,
        /// Dry run - show what would be uploaded without actually uploading
        #[arg(long)]
        dry_run: bool,
    },
}
```

### 3. Add Push Command Handler to the Match Statement

Find the match statement in the `Cli::run()` method and add the push case:

```rust
match cli.command {
    Commands::New { message, id } => {
        Self::handle_new_command(message, id, &storage)?;
    }
    Commands::Edit { id } => {
        Self::handle_edit_command(id, &storage)?;
    }
    Commands::Show { id, formatted } => {
        Self::handle_show_command(id, formatted, &storage)?;
    }
    Commands::List => {
        Self::handle_list_command(&storage)?;
    }
    Commands::Push { mode, force, dry_run } => {
        Self::handle_push_command(mode, force, dry_run)?;
    }
}
```

### 4. Implement Push Command Handler

Add the push command handler method to the `impl Cli` block:

```rust
/// Handle the push command
fn handle_push_command(
    mode: PushMode,
    force: bool,
    dry_run: bool
) -> Result<(), Box<dyn std::error::Error>> {
    // Import the config module at the top of the file if not already done
    use crate::config::DevLogConfig;

    println!("🔄 DevLog Push Command");
    println!("Mode: {}", mode);

    if dry_run {
        println!("🔍 Dry run mode - no files will actually be uploaded");
    }

    if force {
        println!("💪 Force mode - will upload even if no changes detected");
    }

    // Step 1: Load and validate configuration
    print!("📋 Loading configuration... ");
    let config = match DevLogConfig::load() {
        Ok(config) => {
            println!("✅");
            config
        }
        Err(e) => {
            println!("❌");
            eprintln!("Failed to load configuration: {}", e);
            eprintln!();
            eprintln!("💡 To set up remote storage configuration:");
            eprintln!("   Create ~/.devlog/config.toml with your Azure storage details");
            eprintln!("   Example:");
            eprintln!("   [remote]");
            eprintln!("   provider = \"azure\"");
            eprintln!("   url = \"https://account.blob.core.windows.net/container\"");
            return Err(e);
        }
    };

    // Step 2: Validate configuration
    print!("🔍 Validating configuration... ");
    if let Err(e) = config.validate() {
        println!("❌");
        eprintln!("Configuration validation failed: {}", e);
        eprintln!();
        eprintln!("💡 Please check your ~/.devlog/config.toml file");
        return Err(e.into());
    }
    println!("✅");

    println!("📡 Remote storage: {} ({})", config.remote.url, config.remote.provider);

    if let Some(last_push) = config.sync.last_push_timestamp {
        println!("🕒 Last push: {}", last_push.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("🆕 This will be the first push");
    }

    // For now, just show what we would do
    // The actual sync logic will be implemented in Task 07
    println!();
    println!("🚧 Sync implementation coming in Task 07: Sync Manager");
    println!("📝 Current status: Configuration validated successfully");

    if dry_run {
        println!("✨ Dry run completed - ready for actual implementation");
    }

    Ok(())
}
```

### 5. Add Required Module Imports

At the top of `src/cli.rs`, add the new module import:

```rust
use crate::entry::Entry;
use crate::storage::{EntryStorage, LocalEntryStorage};
use crate::config::DevLogConfig;  // Add this line
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use std::process;
```

### 6. Update main.rs for Async Support

Since the push command will eventually use async operations, update your `src/main.rs` to support async:

```rust
mod cli;
mod config;       // Add this
mod remote;       // Add this
mod local;        // Add this
mod sync;         // Add this (for future use)
mod entry;
mod events;
mod storage;
mod annotations;

#[tokio::main]  // Change this from regular main
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::Cli::run()
}
```

## Validation Steps

### 1. Compilation Test

```bash
# Build the project to ensure no compilation errors
cargo build

# Check for warnings
cargo clippy
```

### 2. Command Help Test

Test that the new command appears in help:

```bash
# Build and test help
cargo build
./target/debug/devlog --help

# Test push command help
./target/debug/devlog push --help
```

### 3. Basic Command Test

Test the push command with different arguments:

```bash
# Test default mode
./target/debug/devlog push

# Test explicit mode
./target/debug/devlog push --mode incremental

# Test all mode with dry run
./target/debug/devlog push --mode all --dry-run

# Test force mode
./target/debug/devlog push --force
```

### 4. Configuration Integration Test

Create a test configuration and verify it loads:

```bash
# Create test config directory
mkdir -p ~/.devlog

# Create test config file
cat > ~/.devlog/config.toml << EOF
[remote]
provider = "azure"
url = "https://testaccount.blob.core.windows.net/devlog"

[sync]
last_push_timestamp = "2025-09-11T10:30:00Z"
EOF

# Test push command
./target/debug/devlog push --dry-run
```

## Expected Outputs

After completing this task:

- ✅ `devlog push` command is available in CLI
- ✅ Command accepts `--mode`, `--force`, and `--dry-run` flags
- ✅ Help text displays correctly for the push command
- ✅ Configuration validation works and provides helpful error messages
- ✅ Dry run mode shows what would be done without actual upload
- ✅ All existing commands continue to work unchanged

### Sample Output

When running `devlog push --dry-run`, you should see something like:

```
🔄 DevLog Push Command
Mode: incremental
🔍 Dry run mode - no files will actually be uploaded
📋 Loading configuration... ✅
🔍 Validating configuration... ✅
📡 Remote storage: https://account.blob.core.windows.net/devlog (azure)
🆕 This will be the first push

🚧 Sync implementation coming in Task 07: Sync Manager
📝 Current status: Configuration validated successfully
✨ Dry run completed - ready for actual implementation
```

## Troubleshooting

**Common Issues**:

1. **Compilation Errors**: Make sure all module declarations are correct
2. **Import Errors**: Verify that the config module is properly imported
3. **Clap Errors**: Ensure the PushMode enum implements the required traits
4. **Async Errors**: Make sure main.rs uses `#[tokio::main]`

**Testing Commands**:

```bash
# Check compilation
cargo check

# Test specific command
cargo build && ./target/debug/devlog push --help

# Verify all commands still work
cargo build && ./target/debug/devlog --help
```

## Next Steps

Once this task is complete, proceed to **Task 06: Azure Storage Client** where we'll implement the actual Azure Blob Storage integration.

## Rust Learning Notes

**Key Concepts Introduced**:

- **Clap Derive Macros**: Using `#[derive(Subcommand)]` and argument attributes
- **String Parsing**: Implementing `FromStr` for custom types
- **Display Trait**: Implementing `Display` for custom formatting
- **Error Handling**: Integrating error handling across modules
- **Module Organization**: Adding new modules to the CLI structure

**Questions to Research**:

1. How do clap derive macros work under the hood?
2. What's the difference between `FromStr` and `TryFrom<&str>`?
3. How does `#[tokio::main]` transform the main function?
4. What are the best practices for CLI error messages?

<function_calls>
<invoke name="read_file">
<parameter name="filePath">/Users/huijinghuang/source/repos/devlog/src/cli.rs
