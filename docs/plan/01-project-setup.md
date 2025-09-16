# Project Setup and Dependencies

## Overview

This document outlines the initial project setup and dependencies needed for the Engineering Journal app. Since you're new to Rust, this includes explanations of why we choose each dependency and how they work together.

## Current Cargo.toml Analysis

```toml
[package]
name = "todos"
version = "0.1.0"
edition = "2024"

[dependencies]
color-eyre = "0.6.5"
ratatui = { version = "0.29.0", features = ["all-widgets"] }
```

## Recommended Dependencies

### Core Dependencies

#### 1. **ratatui** (Already included)

```toml
ratatui = { version = "0.29.0", features = ["all-widgets"] }
```

- **Purpose**: Terminal UI framework
- **Why**: Industry standard for Rust TUI applications
- **Learning**: Great for understanding Rust's widget patterns

#### 2. **color-eyre** (Already included)

```toml
color-eyre = "0.6.5"
```

- **Purpose**: Better error handling and reporting
- **Why**: Follows your preference for using libraries instead of custom errors
- **Learning**: Shows Rust's Result<T, E> pattern in action

#### 3. **crossterm** (Add this)

```toml
crossterm = "0.28"
```

- **Purpose**: Cross-platform terminal manipulation
- **Why**: ratatui recommends it for input handling
- **Learning**: Good for understanding Rust's cross-platform approach

#### 4. **chrono** (Add this)

```toml
chrono = { version = "0.4", features = ["serde"] }
```

- **Purpose**: Date and time handling
- **Why**: Essential for YYYYMMDD entry format
- **Learning**: Rust's approach to time handling

#### 5. **serde** (Add this)

```toml
serde = { version = "1.0", features = ["derive"] }
```

- **Purpose**: Serialization/deserialization
- **Why**: Useful for future config files or data formats
- **Learning**: Rust's powerful derive macros

#### 6. **dirs** (Add this)

```toml
dirs = "5.0"
```

- **Purpose**: Cross-platform directory paths
- **Why**: Finding ~/.config/engineering-journal directory
- **Learning**: Platform-specific path handling

#### 7. **tokio** (Optional for now)

```toml
tokio = { version = "1.0", features = ["full"] }
```

- **Purpose**: Async runtime
- **Why**: Might be useful for file I/O later
- **Learning**: Rust's async/await patterns

## Updated Cargo.toml

```toml
[package]
name = "engineering-journal"  # Rename from "todos"
version = "0.1.0"
edition = "2021"  # More stable than 2024 for learning

[dependencies]
# UI Framework
ratatui = { version = "0.29.0", features = ["all-widgets"] }
crossterm = "0.28"

# Error Handling
color-eyre = "0.6.5"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# File System
dirs = "5.0"

# Serialization (for future config)
serde = { version = "1.0", features = ["derive"] }

# Optional: Async runtime
# tokio = { version = "1.0", features = ["full"] }
```

## Learning Resources

### Rust Concepts You'll Learn

1. **Ownership & Borrowing**: Through file handling
2. **Pattern Matching**: With Result<T, E> and Option<T>
3. **Traits**: ratatui's Widget trait
4. **Error Handling**: color-eyre integration
5. **Modules**: Organizing code into logical units
6. **Lifetimes**: Working with ratatui's rendering

### Recommended Study Order

1. **Week 1**: Basic file I/O and error handling
2. **Week 2**: ratatui basics and event handling
3. **Week 3**: Data structures and navigation
4. **Week 4**: Integration and polish

## Development Environment Setup

### VS Code Extensions (Recommended)

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "vadimcn.vscode-lldb",
    "serayuzgur.crates"
  ]
}
```

### Useful Commands

```bash
# Update dependencies
cargo update

# Check for issues without building
cargo check

# Run with better error messages
RUST_BACKTRACE=1 cargo run

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Next Steps

1. Update Cargo.toml with new dependencies
2. Rename project from "todos" to "engineering-journal"
3. Set up basic project structure (see 02-architecture.md)
4. Start with simple "Hello World" ratatui app (see 03-implementation-phases.md)
