use color_eyre::eyre::Result;
use std::path::PathBuf;

/// Platform-specific directory types for XDG compliance
#[derive(Debug, Clone, Copy)]
pub enum XdgDirectoryType {
    Config,
    Data,
    Cache,
    State,
}

/// Get platform-specific fallback directory for the given XDG directory type
fn get_platform_fallback_dir(dir_type: XdgDirectoryType) -> Option<PathBuf> {
    match (dir_type, get_current_platform()) {
        // Linux/FreeBSD: Standard XDG paths
        (XdgDirectoryType::Config, Platform::Unix) => get_unix_config_dir(),
        (XdgDirectoryType::Data, Platform::Unix) => get_unix_local_share_dir(),
        (XdgDirectoryType::Cache, Platform::Unix) => get_unix_cache_dir(),
        (XdgDirectoryType::State, Platform::Unix) => get_unix_state_dir(),

        // macOS: Library-based paths
        (XdgDirectoryType::Config, Platform::MacOS) => get_macos_app_support_dir(),
        (XdgDirectoryType::Data, Platform::MacOS) => get_macos_app_support_dir(),
        (XdgDirectoryType::Cache, Platform::MacOS) => get_macos_cache_dir(),
        (XdgDirectoryType::State, Platform::MacOS) => get_macos_app_support_dir(),

        // Windows: AppData paths
        (XdgDirectoryType::Config, Platform::Windows) => get_windows_appdata(),
        (XdgDirectoryType::Data, Platform::Windows) => get_windows_appdata(),
        (XdgDirectoryType::Cache, Platform::Windows) => get_windows_local_appdata(),
        (XdgDirectoryType::State, Platform::Windows) => get_windows_appdata(),
    }
}

/// Platform enum for cleaner conditional logic
#[derive(Debug, Clone, Copy, PartialEq)]
enum Platform {
    Unix,    // Linux, FreeBSD, etc.
    MacOS,   // macOS
    Windows, // Windows
}

/// Detect the current platform
fn get_current_platform() -> Platform {
    if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else {
        // Default to Unix for Linux, FreeBSD, and other Unix-like systems
        Platform::Unix
    }
}

/// Get Windows APPDATA directory (Roaming)
fn get_windows_appdata() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join("AppData").join("Roaming")))
}

/// Get Windows Local AppData directory
fn get_windows_local_appdata() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join("AppData").join("Local")))
}

/// Get Unix-style config directory (~/.config)
fn get_unix_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config"))
}

/// Get Unix-style local share directory (~/.local/share)
fn get_unix_local_share_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".local").join("share"))
}

/// Get Unix-style cache directory (~/.cache)
fn get_unix_cache_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".cache"))
}

/// Get Unix-style state directory (~/.local/state)
fn get_unix_state_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".local").join("state"))
}

/// Get macOS Library/Application Support directory
fn get_macos_app_support_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join("Library").join("Application Support"))
}

/// Get macOS Library/Caches directory
fn get_macos_cache_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join("Library").join("Caches"))
}

/// Generic function to get XDG-compliant directory with platform-specific fallbacks
pub fn get_xdg_directory(
    dir_type: XdgDirectoryType,
    app_name: &str,
    dirs_fn: impl FnOnce() -> Option<PathBuf>,
) -> Result<PathBuf> {
    let base_dir = dirs_fn()
        .or_else(|| get_platform_fallback_dir(dir_type))
        .map(|dir| dir.join(app_name))
        .ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "Could not determine {} directory",
                format_directory_type(dir_type)
            )
        })?;

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&base_dir).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Failed to create {} directory: {} - {}",
            format_directory_type(dir_type),
            base_dir.display(),
            e
        )
    })?;

    Ok(base_dir)
}

/// Helper function to format directory type for error messages
fn format_directory_type(dir_type: XdgDirectoryType) -> &'static str {
    match dir_type {
        XdgDirectoryType::Config => "config",
        XdgDirectoryType::Data => "data",
        XdgDirectoryType::Cache => "cache",
        XdgDirectoryType::State => "state",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = get_current_platform();

        // Verify platform detection works
        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);

        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);

        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        assert_eq!(platform, Platform::Unix);
    }

    #[test]
    fn test_platform_fallback_directories() {
        // Test that fallback directories are returned for each platform/type combination
        let dir_types = [
            XdgDirectoryType::Config,
            XdgDirectoryType::Data,
            XdgDirectoryType::Cache,
            XdgDirectoryType::State,
        ];

        for dir_type in dir_types {
            let fallback = get_platform_fallback_dir(dir_type);
            // Should return Some path on all platforms
            assert!(
                fallback.is_some(),
                "Fallback should exist for {:?}",
                dir_type
            );
        }
    }

    #[test]
    fn test_xdg_directory_creation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_app_name = "test_app";

        // Test that the function creates directories properly
        let result = get_xdg_directory(XdgDirectoryType::Config, test_app_name, || {
            Some(temp_dir.path().to_path_buf())
        });

        assert!(result.is_ok());
        let config_dir = result.unwrap();
        assert!(config_dir.exists());
        assert!(config_dir.ends_with(test_app_name));
    }
}
