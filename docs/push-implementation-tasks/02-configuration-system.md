# Task 02: Configuration System Implementation

**Estimated Time**: 2-3 hours  
**Difficulty**: ⭐⭐ Beginner-Intermediate  
**Prerequisites**: Task 01 completed

## Objective

Implement a TOML-based configuration system that stores remote storage settings and sync metadata in `~/.devlog/config.toml`.

## What You'll Learn

- Rust struct definitions with serde for serialization
- File I/O operations in Rust
- Error handling patterns
- Working with the `toml` crate
- Path manipulation for cross-platform compatibility

## Tasks

### 1. Define Configuration Structures

In `src/config.rs`, implement the configuration data structures:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DevLogConfig {
    pub remote: RemoteConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoteConfig {
    pub provider: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncConfig {
    pub last_push_timestamp: Option<DateTime<Utc>>,
}

impl Default for DevLogConfig {
    fn default() -> Self {
        Self {
            remote: RemoteConfig {
                provider: "azure".to_string(),
                url: String::new(),
            },
            sync: SyncConfig {
                last_push_timestamp: None,
            },
        }
    }
}
```

### 2. Implement Configuration File Operations

Add configuration management functions:

```rust
impl DevLogConfig {
    /// Get the default config file path: ~/.devlog/config.toml
    pub fn config_file_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .context("Unable to determine home directory")?;

        let devlog_dir = home_dir.join(".devlog");
        let config_path = devlog_dir.join("config.toml");

        Ok(config_path)
    }

    /// Load configuration from file, or create default if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            // Create default config and save it
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: DevLogConfig = toml::from_str(&config_content)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Ensure the .devlog directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {:?}", parent))?;
        }

        let config_content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;

        std::fs::write(&config_path, config_content)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }

    /// Update the last push timestamp and save
    pub fn update_last_push_timestamp(&mut self) -> Result<()> {
        self.sync.last_push_timestamp = Some(Utc::now());
        self.save()
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        if self.remote.url.is_empty() {
            anyhow::bail!("Remote URL is not configured. Please set the URL in ~/.devlog/config.toml");
        }

        if !self.remote.url.starts_with("https://") {
            anyhow::bail!("Remote URL must use HTTPS protocol");
        }

        if self.remote.provider != "azure" {
            anyhow::bail!("Only 'azure' provider is currently supported");
        }

        Ok(())
    }
}
```

### 3. Add Required Dependency

Update your `Cargo.toml` to include the `dirs` crate for cross-platform home directory detection:

```toml
dirs = "5.0"  # Cross-platform directory utilities
```

### 4. Create Configuration Utilities

Add helper functions for common configuration operations:

```rust
/// Configuration utilities and helpers
pub mod utils {
    use super::*;

    /// Initialize configuration with user prompts
    pub fn setup_config_interactive() -> Result<DevLogConfig> {
        println!("Setting up DevLog remote storage configuration...");

        print!("Enter your Azure Storage URL (e.g., https://account.blob.core.windows.net/container): ");
        std::io::stdout().flush().unwrap();

        let mut url = String::new();
        std::io::stdin().read_line(&mut url)
            .context("Failed to read user input")?;

        let url = url.trim().to_string();

        let config = DevLogConfig {
            remote: RemoteConfig {
                provider: "azure".to_string(),
                url,
            },
            sync: SyncConfig {
                last_push_timestamp: None,
            },
        };

        config.validate()?;
        config.save()?;

        println!("Configuration saved to {:?}", DevLogConfig::config_file_path()?);
        Ok(config)
    }

    /// Check if configuration exists and is valid
    pub fn check_config() -> Result<bool> {
        match DevLogConfig::load() {
            Ok(config) => {
                config.validate()?;
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }
}

// Don't forget to add this import at the top
use std::io::Write;
```

### 5. Create Example Configuration File

Create a template configuration file for documentation:

**Create file: `docs/config-example.toml`**:

```toml
# DevLog Configuration File
# Location: ~/.devlog/config.toml

[remote]
# Cloud storage provider (currently only "azure" is supported)
provider = "azure"

# Azure Blob Storage URL
# Format: https://<account>.blob.core.windows.net/<container>
# Example: https://myaccount.blob.core.windows.net/devlog
url = "https://your-storage-account.blob.core.windows.net/your-container"

[sync]
# Timestamp of the last successful push (managed automatically)
# Format: RFC 3339 timestamp
# Example: "2025-09-11T10:30:00Z"
last_push_timestamp = "2025-09-11T10:30:00Z"
```

## Validation Steps

### 1. Unit Tests

Create basic tests in `src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_serialization() {
        let config = DevLogConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: DevLogConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.remote.provider, parsed.remote.provider);
        assert_eq!(config.remote.url, parsed.remote.url);
    }

    #[test]
    fn test_config_validation() {
        let mut config = DevLogConfig::default();

        // Should fail - empty URL
        assert!(config.validate().is_err());

        // Should fail - non-HTTPS URL
        config.remote.url = "http://example.com".to_string();
        assert!(config.validate().is_err());

        // Should succeed
        config.remote.url = "https://account.blob.core.windows.net/container".to_string();
        assert!(config.validate().is_ok());
    }
}
```

### 2. Manual Testing

Test the configuration system:

```bash
# Build the project
cargo build

# Run tests
cargo test config

# Create a simple test program to verify config loading
```

## Expected Outputs

After completing this task:

- ✅ Configuration structures are defined with proper serde annotations
- ✅ Config file can be loaded from/saved to `~/.devlog/config.toml`
- ✅ Default configuration is created when file doesn't exist
- ✅ Configuration validation works correctly
- ✅ All tests pass

## Troubleshooting

**Common Issues**:

1. **Permission Errors**: Ensure you have write access to home directory
2. **Path Issues**: Check that the `dirs` crate is working on your system
3. **Serialization Errors**: Verify your structs have proper serde derives
4. **File Not Found**: Make sure the parent directory creation logic works

**Testing Commands**:

```bash
# Check compilation
cargo check

# Run specific tests
cargo test config

# Check for warnings
cargo clippy
```

## Next Steps

Once this task is complete, proceed to **Task 03: Remote Storage Trait** where we'll define the cloud-agnostic storage interface.

## Rust Learning Notes

**Key Concepts Introduced**:

- **Serde**: Serialization/deserialization framework
- **Error Handling**: Using `anyhow` and `Result<T, E>`
- **Traits**: `Default`, `Debug`, `Clone`
- **File I/O**: Reading and writing files
- **Path Handling**: Cross-platform path manipulation

**Questions to Research**:

1. What does `#[derive(Debug, Deserialize, Serialize, Clone)]` do?
2. How does `anyhow::Context` improve error messages?
3. What's the difference between `String` and `&str` in Rust?
4. How does Rust's ownership system affect file operations?
