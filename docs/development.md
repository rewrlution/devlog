# Development

## Development

```sh
# Test
cargo test

# Run with cargo
cargo run -- -h

# Create a new entry
cargo run -- new -m "Tody I worked with @alice on::devlog using +rust"

# Open editor for new entry
cargo run -- new

# Edit an existing entry
cargo run -- edit 20250906
```

## Build and Install Locally

```sh
# Build optimized release version
cargo build --release

# The binary will be at target/release/devlog
./target/release/devlog --help

# Or install it to your cargo bin directory (~/.cargo/bin)
cargo install --path .

cargo build --release

# Binary will be at target/release/devlog
./target/release/devlog --help

# Add to your `~/.zshrc`:
alias devlog='cargo run --manifest-path /{your_path_to_the_repo}/devlog/Cargo.toml --'

# Now you can run it directly (if ~/.cargo/bin is in your PATH)
devlog --help
devlog new -m "My first entry"
```

## Format

Format the code: `cargo fmt`.

Format on save:

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```
