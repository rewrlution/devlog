# Task 01: Project Setup and Dependencies

**Estimated Time**: 1-2 hours  
**Difficulty**: ⭐ Beginner  
**Prerequisites**: Basic Rust project knowledge

## Objective

Set up the necessary dependencies and basic project structure to support the push command implementation.

## What You'll Learn

- How to manage Rust dependencies with Cargo.toml
- Basic project organization for a CLI application
- Understanding async Rust ecosystem

## Tasks

### 1. Update Cargo.toml Dependencies

Add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
# Existing dependencies (keep these)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4.0", features = ["derive"] }

# New dependencies for push command
toml = "0.8"                                           # TOML configuration parsing
sha2 = "0.10"                                         # File hashing for change detection
reqwest = { version = "0.11", features = ["json"] }  # HTTP client for Azure REST API
tokio = { version = "1.0", features = ["full"] }     # Async runtime
anyhow = "1.0"                                        # Enhanced error handling
thiserror = "1.0"                                     # Custom error types

[dev-dependencies]
tempfile = "3.0"                                      # Temporary files for testing
```

### 2. Create Module Structure

Create the following new files in the `src/` directory:

```
src/
├── config.rs          # Configuration management
├── remote/
│   ├── mod.rs         # Remote storage module
│   └── azure.rs       # Azure-specific implementation
├── sync.rs            # Sync manager and logic
└── lib.rs             # Library exports (optional)
```

### 3. Update main.rs for Async Support

Since we'll be using async operations for HTTP requests, update your `main.rs`:

```rust
// Add this attribute to your main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your existing main function code
    // (we'll modify this more in later tasks)
}
```

### 4. Create Basic Module Files

Create empty module files with basic structure:

**src/config.rs**:

```rust
//! Configuration management for DevLog
//!
//! This module handles reading and writing configuration files,
//! particularly for remote storage settings.

// We'll implement this in Task 02
```

**src/remote/mod.rs**:

```rust
//! Remote storage abstractions and implementations

pub mod azure;

// We'll define the RemoteStorage trait here in Task 03
```

**src/remote/azure.rs**:

```rust
//! Azure Blob Storage implementation

// We'll implement Azure storage client in Task 06
```

**src/sync.rs**:

```rust
//! Synchronization manager and logic

// We'll implement sync logic in Task 07
```

### 5. Update Module Declarations

In your `src/main.rs` or wherever you declare modules, add:

```rust
mod config;
mod remote;
mod sync;
```

## Validation Steps

1. **Compile Check**: Run `cargo check` to ensure all dependencies resolve correctly
2. **Build Test**: Run `cargo build` to verify the project structure
3. **Dependency Verification**: Run `cargo tree` to see the dependency graph

## Expected Outputs

After completing this task:

- ✅ All new dependencies are added to Cargo.toml
- ✅ Project compiles without errors
- ✅ Module structure is in place
- ✅ main.rs is set up for async operations

## Troubleshooting

**Common Issues**:

1. **Compilation Errors**: Make sure all module files exist (even if empty)
2. **Dependency Conflicts**: Run `cargo update` if you see version conflicts
3. **Async Errors**: Ensure `#[tokio::main]` is added to your main function

**Getting Help**:

- Check Rust documentation: https://doc.rust-lang.org/book/
- Tokio documentation: https://tokio.rs/
- Ask about any Rust-specific concepts you don't understand

## Next Steps

Once this task is complete, proceed to **Task 02: Configuration System** where we'll implement the TOML-based configuration management.

## Rust Learning Notes

**Key Concepts Introduced**:

- **Cargo.toml**: Rust's package manager and build system
- **Module System**: How Rust organizes code into modules
- **Async/Await**: Rust's approach to asynchronous programming
- **Dependencies**: External crates (libraries) and feature flags

**Questions to Research**:

1. What does the `features = ["derive"]` mean in serde dependency?
2. How does Rust's module system work compared to other languages?
3. What is the difference between `tokio::main` and regular `main`?
