# Step 1: Project Setup and Dependencies

## Overview

Set up the basic Rust project structure and add necessary dependencies for the MVP.

## Dependencies to Add

Update `Cargo.toml` with these essential crates:

```toml
[dependencies]
# Command line argument parsing
clap = { version = "4.4", features = ["derive"] }

# Date/time handling
chrono = { version = "0.4", features = ["serde"] }

# YAML frontmatter parsing
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"

# Terminal UI for interactive mode
crossterm = "0.27"
ratatui = "0.24"

# File system operations
walkdir = "2.4"

# Error handling with beautiful colored output
color-eyre = "0.6"

# Regex for parsing annotations
regex = "1.10"
```

## Why These Libraries?

- **clap**: The most popular Rust CLI framework with derive macros (simple to use)
- **chrono**: Standard for date/time in Rust
- **serde + serde_yaml**: For parsing YAML frontmatter in markdown files
- **crossterm + ratatui**: Modern, cross-platform terminal UI libraries
- **walkdir**: Simple recursive directory traversal
- **color-eyre**: Enhanced error handling with beautiful colored output and detailed error reports
- **regex**: For parsing @mentions, ::projects, and +tags

## Project Structure to Create

```
src/
├── main.rs              # Entry point and CLI setup
├── lib.rs               # Library root
├── commands/            # Command implementations
│   ├── mod.rs
│   ├── new.rs           # devlog new
│   ├── edit.rs          # devlog edit
│   ├── list.rs          # devlog list
│   ├── show.rs          # devlog show
│   └── insight.rs       # devlog insight
├── storage/             # File operations
│   ├── mod.rs
│   └── entry.rs         # Entry struct and file operations
├── tui/                 # Terminal UI components
│   ├── mod.rs
│   └── app.rs           # Interactive TUI application
└── utils/               # Utility functions
    ├── mod.rs
    ├── editor.rs        # Vim integration
    └── parser.rs        # Annotation parsing
```

## Implementation Tasks

1. **Update Cargo.toml** with the dependencies above
2. **Create the directory structure** in `src/`
3. **Set up basic module declarations** in each `mod.rs` file

## Key Rust Concepts to Understand

- **Crates**: External libraries (like npm packages)
- **Modules**: Organize code within your project using `mod.rs` files
- **Features**: Optional functionality in crates (like `["derive"]` for clap)
- **color_eyre::Result**: Enhanced error handling - use `color_eyre::Result<T>` instead of `Result<T, SomeErrorType>`
- **eyre::Context**: Adds context to errors with `.wrap_err()` and `.wrap_err_with()`

## Next Steps

After completing this setup, move to Step 2: Data Structures and Storage.

## Implementation Order

For this project, follow this recommended order:

1. **Step 1**: Project Setup (this step)
2. **Step 2**: Data Structures and Storage
3. **Step 3**: Basic CLI Commands
4. **Step 5**: Interactive TUI ⭐ (implement before annotation parsing)
5. **Step 4**: Annotation Parsing and Insights (can be implemented later)
6. **Step 6**: Configuration and Polish

**Why TUI before Annotations?** The TUI provides immediate visual feedback and is more engaging for users, while annotation parsing is a nice-to-have feature that can be added once the core functionality works.
