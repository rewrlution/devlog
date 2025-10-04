# Implementing XDG Base Directory Standard in Rust Applications

_A practical guide to organizing application data across platforms with real-world examples_

## Introduction

As developers, we've all encountered the frustration of applications cluttering our home directories with countless dotfiles and configuration folders. The XDG Base Directory Specification provides a elegant solution to this chaos, defining standardized locations for application data, configuration, and cache files on Unix-like systems.

In this post, we'll explore how to implement XDG Base Directory Standard in Rust applications, using lessons learned from building a journaling tool called `devlog`. We'll cover cross-platform considerations, migration strategies, and best practices for organizing your application's data.

## What is XDG Base Directory Standard?

The XDG Base Directory Specification defines where applications should store different types of files:

- **Configuration files** (`XDG_CONFIG_HOME`) - User preferences, settings
- **Application data** (`XDG_DATA_HOME`) - User-created content, databases
- **Cache files** (`XDG_CACHE_HOME`) - Temporary, regenerable performance data
- **State files** (`XDG_STATE_HOME`) - Logs, history, runtime state

### Default Directories by Platform

| Directory Type | Linux/Unix       | macOS                           | Windows          |
| -------------- | ---------------- | ------------------------------- | ---------------- |
| Config         | `~/.config`      | `~/Library/Application Support` | `%APPDATA%`      |
| Data           | `~/.local/share` | `~/Library/Application Support` | `%APPDATA%`      |
| Cache          | `~/.cache`       | `~/Library/Caches`              | `%LOCALAPPDATA%` |
| State          | `~/.local/state` | `~/Library/Application Support` | `%APPDATA%`      |

## Implementation in Rust

### Setting Up Dependencies

First, add the `dirs` crate to your `Cargo.toml`:

```toml
[dependencies]
dirs = "5.0"
```

The `dirs` crate automatically handles XDG environment variables on Linux and provides appropriate platform-specific defaults for macOS and Windows.

### Basic Directory Access

```rust
use dirs;
use std::path::PathBuf;

fn get_app_directories(app_name: &str) -> Result<AppDirs, Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join(app_name);

    let data_dir = dirs::data_dir()
        .ok_or("Could not determine data directory")?
        .join(app_name);

    let cache_dir = dirs::cache_dir()
        .ok_or("Could not determine cache directory")?
        .join(app_name);

    Ok(AppDirs {
        config: config_dir,
        data: data_dir,
        cache: cache_dir,
    })
}

struct AppDirs {
    config: PathBuf,
    data: PathBuf,
    cache: PathBuf,
}
```

## Real-World Example: Devlog Architecture

Let's examine how to structure a journaling application that manages different types of data:

### Directory Structure Design

```rust
use std::path::PathBuf;

struct DevlogPaths {
    // Configuration (user preferences)
    config_file: PathBuf,          // ~/.config/devlog/config.yaml
    editor_settings: PathBuf,      // ~/.config/devlog/editor.yaml

    // Data (persistent user content)
    entries_dir: PathBuf,          // ~/.local/share/devlog/entries/
    events_log: PathBuf,           // ~/.local/share/devlog/events.jsonl
    search_index: PathBuf,         // ~/.local/share/devlog/search.idx

    // Cache (regenerable performance data)
    metadata_cache: PathBuf,       // ~/.cache/devlog/metadata/
    tree_structure: PathBuf,       // ~/.cache/devlog/tree.cache
    rendered_content: PathBuf,     // ~/.cache/devlog/rendered/
}

impl DevlogPaths {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?
            .join("devlog");

        let data_dir = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("devlog");

        let cache_dir = dirs::cache_dir()
            .ok_or("Could not determine cache directory")?
            .join("devlog");

        Ok(Self {
            config_file: config_dir.join("config.yaml"),
            editor_settings: config_dir.join("editor.yaml"),

            entries_dir: data_dir.join("entries"),
            events_log: data_dir.join("events.jsonl"),
            search_index: data_dir.join("search.idx"),

            metadata_cache: cache_dir.join("metadata"),
            tree_structure: cache_dir.join("tree.cache"),
            rendered_content: cache_dir.join("rendered"),
        })
    }
}
```

### Deciding What Goes Where

**Configuration Directory** - Store user preferences and settings:

- Application settings (theme, default editor, sync preferences)
- User-defined templates and shortcuts
- Authentication tokens and API keys

**Data Directory** - Store valuable user-created content:

- Journal entries and notes
- Search indices and databases
- Event sourcing logs for data recovery
- Exported data and backups

**Cache Directory** - Store temporary performance optimizations:

- Pre-parsed metadata (tags, mentions, projects)
- Rendered markdown content
- UI state (tree expansion, recent searches)
- Computed analytics and statistics

## Migration Strategy: Backward Compatibility

When adopting XDG standards in existing applications, you'll want to support legacy installations:

```rust
use std::fs;
use std::path::{Path, PathBuf};

struct Storage {
    entries_path: PathBuf,
}

impl Storage {
    pub fn new(override_path: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let entries_path = match override_path {
            Some(path) => path.join("entries"),
            None => Self::determine_entries_path()?,
        };

        // Ensure directory exists
        fs::create_dir_all(&entries_path)?;

        Ok(Self { entries_path })
    }

    fn determine_entries_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Legacy path (pre-XDG)
        let legacy_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".devlog")
            .join("entries");

        // XDG-compliant path
        let xdg_path = dirs::data_dir()
            .ok_or("Could not determine data directory")?
            .join("devlog")
            .join("entries");

        // Use legacy if it exists and contains data, otherwise use XDG
        if legacy_path.exists() && Self::has_entries(&legacy_path)? {
            println!("Using legacy data directory: {}", legacy_path.display());
            println!("Consider migrating to: {}", xdg_path.display());
            Ok(legacy_path)
        } else {
            Ok(xdg_path)
        }
    }

    fn has_entries(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        if !path.exists() {
            return Ok(false);
        }

        let entries = fs::read_dir(path)?;
        Ok(entries.count() > 0)
    }
}
```

## Advanced: Configuration Management

Create a robust configuration system that respects XDG standards:

```rust
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Default)]
struct DevlogConfig {
    editor: String,
    sync_enabled: bool,
    default_template: Option<String>,
    ui_theme: String,
}

impl DevlogConfig {
    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = dirs::config_dir()
            .ok_or("Could not determine config directory")?
            .join("devlog")
            .join("config.yaml");

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            let config: DevlogConfig = serde_yaml::from_str(&contents)?;
            Ok(config)
        } else {
            // Create default config
            let default_config = DevlogConfig::default();
            default_config.save()?;
            Ok(default_config)
        }
    }

    fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?
            .join("devlog");

        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.yaml");
        let contents = serde_yaml::to_string(self)?;
        fs::write(&config_path, contents)?;

        Ok(())
    }
}
```

## Environment Variable Handling

The `dirs` crate automatically respects XDG environment variables when they're set:

```bash
# User can override defaults
export XDG_CONFIG_HOME="$HOME/.config"
export XDG_DATA_HOME="$HOME/.local/share"
export XDG_CACHE_HOME="$HOME/.cache"

# Your Rust app will automatically use these paths
./devlog list
```

## Cross-Platform Considerations

### Testing Across Platforms

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_creation() {
        let paths = DevlogPaths::new().expect("Failed to create paths");

        // Verify paths are platform-appropriate
        #[cfg(target_os = "linux")]
        {
            assert!(paths.config_file.to_string_lossy().contains(".config"));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(paths.config_file.to_string_lossy().contains("Library/Application Support"));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(paths.config_file.to_string_lossy().contains("AppData"));
        }
    }
}
```

### Platform-Specific Behaviors

Be aware that different platforms have different expectations:

- **Linux**: Users expect XDG compliance and may set custom XDG paths
- **macOS**: Users expect Applications Support structure; XDG variables rarely used
- **Windows**: Users expect AppData structure; completely different paradigm

## Best Practices

### 1. Graceful Fallbacks

Always provide fallbacks when directory detection fails:

```rust
let config_dir = dirs::config_dir()
    .unwrap_or_else(|| {
        eprintln!("Warning: Could not determine config directory, using fallback");
        dirs::home_dir().unwrap_or_default().join(".config")
    })
    .join("myapp");
```

### 2. Clear Error Messages

Help users understand directory issues:

```rust
fn create_app_directories() -> Result<(), Box<dyn std::error::Error>> {
    let paths = DevlogPaths::new()?;

    for dir in [&paths.config_file.parent().unwrap(), &paths.entries_dir, &paths.metadata_cache] {
        fs::create_dir_all(dir).map_err(|e| {
            format!("Failed to create directory {}: {}", dir.display(), e)
        })?;
    }

    Ok(())
}
```

### 3. Cache Invalidation

Make cache directories easy to clear:

```rust
impl DevlogPaths {
    pub fn clear_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.metadata_cache.exists() {
            fs::remove_dir_all(&self.metadata_cache)?;
        }
        if self.tree_structure.exists() {
            fs::remove_file(&self.tree_structure)?;
        }
        Ok(())
    }
}
```

### 4. Documentation

Document your directory structure for users:

```rust
/// Prints the current directory configuration
pub fn show_paths() {
    let paths = DevlogPaths::new().expect("Failed to get paths");

    println!("Devlog Directory Configuration:");
    println!("  Config: {}", paths.config_file.parent().unwrap().display());
    println!("  Data:   {}", paths.entries_dir.parent().unwrap().display());
    println!("  Cache:  {}", paths.metadata_cache.parent().unwrap().display());
}
```

## Conclusion

Implementing XDG Base Directory Standard in Rust applications creates a cleaner, more professional user experience while respecting platform conventions. The `dirs` crate makes cross-platform implementation straightforward, handling the complexity of different operating system expectations.

Key takeaways:

- Use **config** directories for user settings and preferences
- Use **data** directories for valuable user-created content
- Use **cache** directories for regenerable performance optimizations
- Always provide migration paths for existing users
- Test across platforms to ensure appropriate behavior
- Document your directory structure for transparency

By following these patterns, your Rust applications will integrate seamlessly into users' systems while maintaining the flexibility to grow and evolve over time.

## Further Reading

- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [`dirs` crate documentation](https://docs.rs/dirs/)
- [Platform-specific conventions](https://docs.rs/dirs/latest/dirs/#examples)

---

_This post was written while developing `devlog`, an open-source journaling tool built in Rust. The complete implementation can be found on GitHub._
