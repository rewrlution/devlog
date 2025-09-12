# Task 03: Remote Storage Trait Definition

**Estimated Time**: 1-2 hours  
**Difficulty**: ⭐⭐ Beginner-Intermediate  
**Prerequisites**: Task 01 and Task 02 completed

## Objective

Define a cloud-agnostic `RemoteStorage` trait that provides a common interface for different cloud storage providers. This trait will be the foundation for the Azure implementation and future cloud providers.

## What You'll Learn

- Rust trait definitions and implementations
- Async traits and Box<dyn Error>
- Path handling and file operations
- Error type design
- Generic programming concepts

## Tasks

### 1. Define the RemoteStorage Trait

In `src/remote/mod.rs`, implement the core trait:

```rust
use std::path::Path;
use async_trait::async_trait;
use anyhow::Result;

/// Metadata about a remote file
#[derive(Debug, Clone)]
pub struct RemoteFileInfo {
    pub key: String,
    pub size: Option<u64>,
    pub hash: Option<String>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

/// Generic interface for remote storage providers
#[async_trait]
pub trait RemoteStorage: Send + Sync {
    /// Upload a local file to remote storage
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<()>;

    /// Download a file from remote storage to local path
    async fn download_file(&self, remote_key: &str, local_path: &Path) -> Result<()>;

    /// List files with the given prefix
    async fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFileInfo>>;

    /// Check if a file exists in remote storage
    async fn file_exists(&self, remote_key: &str) -> Result<bool>;

    /// Get file metadata including hash if available
    async fn get_file_info(&self, remote_key: &str) -> Result<Option<RemoteFileInfo>>;

    /// Delete a file from remote storage
    async fn delete_file(&self, remote_key: &str) -> Result<()>;
}
```

### 2. Add Required Dependencies

Update your `Cargo.toml` to include async-trait:

```toml
async-trait = "0.1"  # Enables async functions in traits
```

### 3. Define Storage Errors

Create custom error types for better error handling:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Upload failed for {path}: {message}")]
    UploadFailed { path: String, message: String },

    #[error("Download failed for {path}: {message}")]
    DownloadFailed { path: String, message: String },

    #[error("Invalid remote key: {key}")]
    InvalidRemoteKey { key: String },

    #[error("Storage provider error: {message}")]
    ProviderError { message: String },
}

impl StorageError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, StorageError::FileNotFound { .. })
    }

    pub fn is_auth_error(&self) -> bool {
        matches!(self, StorageError::AuthenticationFailed { .. })
    }
}
```

### 4. Create Storage Factory

Add a factory pattern for creating storage instances:

```rust
use crate::config::RemoteConfig;

/// Factory for creating RemoteStorage instances based on configuration
pub struct StorageFactory;

impl StorageFactory {
    /// Create a storage instance based on the provided configuration
    pub fn create_storage(config: &RemoteConfig) -> Result<Box<dyn RemoteStorage>> {
        match config.provider.as_str() {
            "azure" => {
                let azure_storage = crate::remote::azure::AzureStorage::new(config)?;
                Ok(Box::new(azure_storage))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported storage provider: {}. Currently supported: azure",
                config.provider
            )),
        }
    }
}
```

### 5. Add Utility Functions

Create helper functions for common operations:

```rust
/// Utility functions for remote storage operations
pub mod utils {
    use super::*;
    use std::path::Path;
    use sha2::{Sha256, Digest};
    use std::io::Read;

    /// Calculate SHA256 hash of a local file
    pub fn calculate_file_hash(path: &Path) -> Result<String> {
        let mut file = std::fs::File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Convert local file path to remote key
    /// Example: ~/.devlog/events/20250911.jsonl -> events/20250911.jsonl
    pub fn local_path_to_remote_key(local_path: &Path, base_path: &Path) -> Result<String> {
        let relative_path = local_path.strip_prefix(base_path)
            .map_err(|_| anyhow::anyhow!("Path {:?} is not under base path {:?}", local_path, base_path))?;

        let key = relative_path.to_string_lossy().replace('\\', "/");
        Ok(key)
    }

    /// Convert remote key to local file path
    /// Example: events/20250911.jsonl -> ~/.devlog/events/20250911.jsonl
    pub fn remote_key_to_local_path(remote_key: &str, base_path: &Path) -> PathBuf {
        base_path.join(remote_key.replace('/', std::path::MAIN_SEPARATOR_STR))
    }

    /// Validate remote key format
    pub fn validate_remote_key(key: &str) -> Result<(), StorageError> {
        if key.is_empty() {
            return Err(StorageError::InvalidRemoteKey {
                key: key.to_string(),
            });
        }

        if key.contains("..") || key.starts_with('/') {
            return Err(StorageError::InvalidRemoteKey {
                key: key.to_string(),
            });
        }

        Ok(())
    }
}
```

### 6. Update Module Structure

Make sure your `src/remote/mod.rs` exports everything properly:

```rust
//! Remote storage abstractions and implementations

pub mod azure;
pub mod utils;

pub use self::{
    RemoteStorage, RemoteFileInfo, StorageError, StorageFactory
};

use std::path::{Path, PathBuf};
use async_trait::async_trait;
use anyhow::Result;
use thiserror::Error;

// ... (all the code from above tasks)
```

### 7. Create Mock Implementation for Testing

Add a mock implementation for testing purposes:

```rust
/// Mock storage implementation for testing
#[derive(Debug)]
pub struct MockStorage {
    files: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            files: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl RemoteStorage for MockStorage {
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<()> {
        let content = std::fs::read(local_path)?;
        let mut files = self.files.lock().unwrap();
        files.insert(remote_key.to_string(), content);
        Ok(())
    }

    async fn download_file(&self, remote_key: &str, local_path: &Path) -> Result<()> {
        let files = self.files.lock().unwrap();
        let content = files.get(remote_key)
            .ok_or_else(|| StorageError::FileNotFound { path: remote_key.to_string() })?;

        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(local_path, content)?;
        Ok(())
    }

    async fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFileInfo>> {
        let files = self.files.lock().unwrap();
        let mut result = Vec::new();

        for key in files.keys() {
            if key.starts_with(prefix) {
                result.push(RemoteFileInfo {
                    key: key.clone(),
                    size: Some(files[key].len() as u64),
                    hash: None,
                    last_modified: Some(chrono::Utc::now()),
                });
            }
        }

        Ok(result)
    }

    async fn file_exists(&self, remote_key: &str) -> Result<bool> {
        let files = self.files.lock().unwrap();
        Ok(files.contains_key(remote_key))
    }

    async fn get_file_info(&self, remote_key: &str) -> Result<Option<RemoteFileInfo>> {
        let files = self.files.lock().unwrap();
        if let Some(content) = files.get(remote_key) {
            Ok(Some(RemoteFileInfo {
                key: remote_key.to_string(),
                size: Some(content.len() as u64),
                hash: None,
                last_modified: Some(chrono::Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete_file(&self, remote_key: &str) -> Result<()> {
        let mut files = self.files.lock().unwrap();
        files.remove(remote_key);
        Ok(())
    }
}
```

## Validation Steps

### 1. Unit Tests

Create tests in `src/remote/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_mock_storage() {
        let storage = MockStorage::new();
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");

        // Create test file
        std::fs::write(&test_file, b"Hello, World!").unwrap();

        // Upload file
        storage.upload_file(&test_file, "test.txt").await.unwrap();

        // Check if file exists
        assert!(storage.file_exists("test.txt").await.unwrap());

        // Get file info
        let info = storage.get_file_info("test.txt").await.unwrap();
        assert!(info.is_some());
        assert_eq!(info.unwrap().size, Some(13));

        // List files
        let files = storage.list_files("").await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].key, "test.txt");
    }

    #[test]
    fn test_path_utilities() {
        use std::path::PathBuf;

        // Test local path to remote key conversion
        let base_path = PathBuf::from("/home/user/.devlog");
        let local_path = PathBuf::from("/home/user/.devlog/events/20250911.jsonl");

        let remote_key = utils::local_path_to_remote_key(&local_path, &base_path).unwrap();
        assert_eq!(remote_key, "events/20250911.jsonl");

        // Test remote key to local path conversion
        let result_path = utils::remote_key_to_local_path("events/20250911.jsonl", &base_path);
        assert_eq!(result_path, local_path);
    }

    #[test]
    fn test_remote_key_validation() {
        use utils::validate_remote_key;

        // Valid keys
        assert!(validate_remote_key("events/file.jsonl").is_ok());
        assert!(validate_remote_key("config.toml").is_ok());

        // Invalid keys
        assert!(validate_remote_key("").is_err());
        assert!(validate_remote_key("../evil").is_err());
        assert!(validate_remote_key("/absolute/path").is_err());
    }
}
```

### 2. Compilation Test

```bash
# Build the project
cargo build

# Run tests
cargo test remote

# Check for any warnings
cargo clippy
```

## Expected Outputs

After completing this task:

- ✅ `RemoteStorage` trait is defined with all required methods
- ✅ Custom error types are implemented
- ✅ Storage factory pattern is working
- ✅ Utility functions for path and hash operations are implemented
- ✅ Mock storage implementation passes all tests
- ✅ All tests pass without errors

## Troubleshooting

**Common Issues**:

1. **Async Trait Errors**: Make sure `async-trait` is properly imported
2. **Lifetime Issues**: Use `&str` and `&Path` in trait methods for flexibility
3. **Send + Sync Bounds**: Required for traits used across threads
4. **Path Separator Issues**: Use proper cross-platform path handling

**Testing Commands**:

```bash
# Check async trait compilation
cargo check

# Run specific tests
cargo test remote::tests

# Check trait bounds
cargo test --features full
```

## Next Steps

Once this task is complete, proceed to **Task 04: File Operations** where we'll implement file scanning, hashing, and local utilities needed for synchronization.

## Rust Learning Notes

**Key Concepts Introduced**:

- **Traits**: Defining shared behavior across types
- **Async Traits**: Using `async-trait` for async methods in traits
- **Error Types**: Custom error enums with `thiserror`
- **Generics**: `Box<dyn Trait>` for trait objects
- **Factory Pattern**: Creating instances based on configuration

**Questions to Research**:

1. Why do we need `async-trait` for async methods in traits?
2. What does `Send + Sync` mean and why is it required?
3. How does `Box<dyn Trait>` work in Rust?
4. What's the difference between trait objects and generic parameters?
