# Task 09: Testing & Validation

**Estimated Time**: 2-3 hours  
**Difficulty**: ⭐⭐⭐ Intermediate  
**Prerequisites**: Tasks 01-08 completed

## Objective

Create comprehensive tests for the entire push system, including unit tests, integration tests, and end-to-end validation to ensure reliability and correctness.

## What You'll Learn

- Comprehensive testing strategies for async Rust code
- Mocking external dependencies (Azure storage)
- Integration testing with temporary files and directories
- Property-based testing for edge cases
- Performance and stress testing

## Tasks

### 1. Unit Test Framework Setup

Create a comprehensive test structure in each module:

```rust
// Add to src/lib.rs (create if it doesn't exist)
//! DevLog library for testing

pub mod config;
pub mod remote;
pub mod local;
pub mod sync;
pub mod cli;

// Re-export commonly used items for tests
pub use config::DevLogConfig;
pub use remote::{RemoteStorage, StorageFactory};
pub use local::{FileScanner, LocalFile};
pub use sync::{SyncManager, PushOptions, PushMode};
```

### 2. Configuration System Tests

Enhance tests in `src/config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};
    use std::fs;

    #[test]
    fn test_config_serialization() {
        let config = DevLogConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: DevLogConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.remote.provider, parsed.remote.provider);
        assert_eq!(config.remote.url, parsed.remote.url);
    }

    #[test]
    fn test_config_validation_cases() {
        let mut config = DevLogConfig::default();

        // Test empty URL
        assert!(config.validate().is_err());

        // Test non-HTTPS URL
        config.remote.url = "http://example.com".to_string();
        assert!(config.validate().is_err());

        // Test unsupported provider
        config.remote.provider = "unsupported".to_string();
        config.remote.url = "https://account.blob.core.windows.net/container".to_string();
        assert!(config.validate().is_err());

        // Test valid configuration
        config.remote.provider = "azure".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create a test config
        let config = DevLogConfig {
            remote: RemoteConfig {
                provider: "azure".to_string(),
                url: "https://test.blob.core.windows.net/devlog".to_string(),
            },
            sync: SyncConfig {
                last_push_timestamp: Some(Utc::now()),
            },
        };

        // Test saving
        let config_content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &config_content).unwrap();

        // Test loading
        let loaded_content = fs::read_to_string(&config_path).unwrap();
        let loaded_config: DevLogConfig = toml::from_str(&loaded_content).unwrap();

        assert_eq!(config.remote.provider, loaded_config.remote.provider);
        assert_eq!(config.remote.url, loaded_config.remote.url);
    }

    #[test]
    fn test_config_timestamp_update() {
        let mut config = DevLogConfig {
            remote: RemoteConfig {
                provider: "azure".to_string(),
                url: "https://test.blob.core.windows.net/devlog".to_string(),
            },
            sync: SyncConfig {
                last_push_timestamp: None,
            },
        };

        assert!(config.sync.last_push_timestamp.is_none());

        // This would normally save to file, but we'll just test the logic
        config.sync.last_push_timestamp = Some(Utc::now());
        assert!(config.sync.last_push_timestamp.is_some());
    }

    #[test]
    fn test_invalid_toml_handling() {
        let invalid_toml = r#"
            [remote
            provider = "azure"
            url = "https://test.blob.core.windows.net/devlog"
        "#;

        let result: Result<DevLogConfig, _> = toml::from_str(invalid_toml);
        assert!(result.is_err());
    }
}
```

### 3. Local File Operations Tests

Comprehensive tests for `src/local.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use chrono::Utc;

    fn create_test_file_structure(base_path: &Path) -> Vec<PathBuf> {
        let mut created_files = Vec::new();

        // Create events directory and files
        let events_dir = base_path.join("events");
        fs::create_dir_all(&events_dir).unwrap();

        let event_file = events_dir.join("20250911.jsonl");
        fs::write(&event_file, r#"{"event": "test", "timestamp": "2025-09-11T10:30:00Z"}"#).unwrap();
        created_files.push(event_file);

        // Create entries directory and files
        let entries_dir = base_path.join("entries");
        fs::create_dir_all(&entries_dir).unwrap();

        let entry_file = entries_dir.join("20250911.md");
        fs::write(&entry_file, "# Test Entry\n\nThis is a test entry.").unwrap();
        created_files.push(entry_file);

        // Create config file
        let config_file = base_path.join("config.toml");
        fs::write(&config_file, r#"[remote]
provider = "azure"
url = "https://test.blob.core.windows.net/devlog"

[sync]
"#).unwrap();
        created_files.push(config_file);

        // Create files that should be excluded
        let cache_dir = base_path.join("cache");
        fs::create_dir_all(&cache_dir).unwrap();
        fs::write(cache_dir.join("temp.tmp"), "temporary").unwrap();

        created_files
    }

    #[test]
    fn test_local_file_creation() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let local_file = LocalFile::from_path(&test_file, temp_dir.path()).unwrap();

        assert_eq!(local_file.relative_path, "test.txt");
        assert_eq!(local_file.size, 13);
        assert!(!local_file.hash.is_empty());
        assert_eq!(local_file.hash.len(), 64); // SHA256 produces 64 hex chars
        assert!(local_file.modified_time <= Utc::now());
    }

    #[test]
    fn test_file_scanner_inclusion_exclusion() {
        let temp_dir = tempdir().unwrap();
        create_test_file_structure(temp_dir.path());

        let scanner = FileScanner::with_base_path(temp_dir.path().to_path_buf());
        let files = scanner.scan_files().unwrap();

        // Should include: events/*.jsonl, entries/*.md, config.toml
        // Should exclude: cache/*.tmp
        assert_eq!(files.len(), 3);

        let keys: Vec<&str> = files.iter().map(|f| f.remote_key()).collect();
        assert!(keys.contains(&"events/20250911.jsonl"));
        assert!(keys.contains(&"entries/20250911.md"));
        assert!(keys.contains(&"config.toml"));

        // Ensure excluded files are not present
        assert!(!keys.iter().any(|k| k.contains("cache")));
        assert!(!keys.iter().any(|k| k.contains(".tmp")));
    }

    #[test]
    fn test_file_hashing_consistency() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = "This is test content for hashing";
        fs::write(&test_file, content).unwrap();

        let hash1 = calculate_file_hash(&test_file).unwrap();
        let hash2 = calculate_file_hash(&test_file).unwrap();
        let hash3 = calculate_string_hash(content);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn test_glob_matching_patterns() {
        assert!(glob_match("*.txt", "file.txt"));
        assert!(glob_match("*.txt", "long_file_name.txt"));
        assert!(!glob_match("*.txt", "file.md"));

        assert!(glob_match("events/*.jsonl", "events/20250911.jsonl"));
        assert!(glob_match("events/*.jsonl", "events/test.jsonl"));
        assert!(!glob_match("events/*.jsonl", "events/test.md"));
        assert!(!glob_match("events/*.jsonl", "entries/test.jsonl"));

        assert!(glob_match("test?", "testa"));
        assert!(glob_match("test?", "test1"));
        assert!(!glob_match("test?", "test"));
        assert!(!glob_match("test?", "testab"));

        assert!(glob_match("**/*.md", "entries/test.md"));
        assert!(glob_match("**/*.md", "deep/nested/file.md"));
    }

    #[tokio::test]
    async fn test_change_detection() {
        use crate::remote::MockStorage;

        let temp_dir = tempdir().unwrap();
        create_test_file_structure(temp_dir.path());

        let scanner = FileScanner::with_base_path(temp_dir.path().to_path_buf());
        let local_files = scanner.scan_files().unwrap();

        let storage = MockStorage::new();

        // Test with empty remote storage (all files should be "added")
        let changes = changes::detect_changes(&local_files, &storage).await.unwrap();
        assert_eq!(changes.len(), 3);
        assert!(changes.iter().all(|c| matches!(c, changes::FileChange::Added(_))));

        // Upload one file to mock storage
        if let Some(file) = local_files.first() {
            storage.upload_file(&file.path, &file.remote_key()).await.unwrap();
        }

        // Test again - should have fewer changes
        let changes = changes::detect_changes(&local_files, &storage).await.unwrap();
        assert_eq!(changes.len(), 2);
    }

    #[test]
    fn test_path_conversion_utilities() {
        use std::path::PathBuf;

        // Test local path to remote key conversion
        let base_path = PathBuf::from("/home/user/.devlog");
        let local_path = PathBuf::from("/home/user/.devlog/events/20250911.jsonl");

        let remote_key = crate::remote::utils::local_path_to_remote_key(&local_path, &base_path).unwrap();
        assert_eq!(remote_key, "events/20250911.jsonl");

        // Test remote key to local path conversion
        let result_path = crate::remote::utils::remote_key_to_local_path("events/20250911.jsonl", &base_path);
        assert_eq!(result_path, local_path);

        // Test with Windows paths
        let windows_base = PathBuf::from("C:\\Users\\user\\.devlog");
        let windows_local = PathBuf::from("C:\\Users\\user\\.devlog\\events\\20250911.jsonl");

        let remote_key = crate::remote::utils::local_path_to_remote_key(&windows_local, &windows_base).unwrap();
        assert_eq!(remote_key, "events/20250911.jsonl");
    }

    #[test]
    fn test_file_scanner_empty_directory() {
        let temp_dir = tempdir().unwrap();

        let scanner = FileScanner::with_base_path(temp_dir.path().to_path_buf());
        let files = scanner.scan_files().unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_file_scanner_large_directory() {
        let temp_dir = tempdir().unwrap();

        // Create many files to test performance
        let events_dir = temp_dir.path().join("events");
        fs::create_dir_all(&events_dir).unwrap();

        for i in 0..1000 {
            let file_path = events_dir.join(format!("event_{}.jsonl", i));
            fs::write(&file_path, format!(r#"{{"event": "test_{}", "id": {}}}"#, i, i)).unwrap();
        }

        let scanner = FileScanner::with_base_path(temp_dir.path().to_path_buf());
        let start = std::time::Instant::now();
        let files = scanner.scan_files().unwrap();
        let duration = start.elapsed();

        assert_eq!(files.len(), 1000);
        assert!(duration.as_millis() < 1000, "Scanning 1000 files took too long: {:?}", duration);
    }
}
```

### 4. Remote Storage Mock Tests

Enhanced mock storage for comprehensive testing:

```rust
// Add to src/remote/mod.rs

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use anyhow::Result;

    /// Enhanced mock storage with failure simulation
    #[derive(Debug, Clone)]
    pub struct EnhancedMockStorage {
        files: Arc<Mutex<HashMap<String, MockFile>>>,
        config: Arc<Mutex<MockConfig>>,
    }

    #[derive(Debug, Clone)]
    struct MockFile {
        content: Vec<u8>,
        hash: String,
        last_modified: chrono::DateTime<chrono::Utc>,
    }

    #[derive(Debug)]
    struct MockConfig {
        should_fail_auth: bool,
        should_fail_network: bool,
        upload_delay_ms: u64,
        fail_after_n_operations: Option<usize>,
        operation_count: usize,
    }

    impl EnhancedMockStorage {
        pub fn new() -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::new())),
                config: Arc::new(Mutex::new(MockConfig {
                    should_fail_auth: false,
                    should_fail_network: false,
                    upload_delay_ms: 0,
                    fail_after_n_operations: None,
                    operation_count: 0,
                })),
            }
        }

        /// Configure the mock to simulate authentication failures
        pub fn with_auth_failure(self) -> Self {
            self.config.lock().unwrap().should_fail_auth = true;
            self
        }

        /// Configure the mock to simulate network failures
        pub fn with_network_failure(self) -> Self {
            self.config.lock().unwrap().should_fail_network = true;
            self
        }

        /// Configure the mock to add delay to operations
        pub fn with_delay(self, delay_ms: u64) -> Self {
            self.config.lock().unwrap().upload_delay_ms = delay_ms;
            self
        }

        /// Configure the mock to fail after N operations
        pub fn with_failure_after(self, n: usize) -> Self {
            self.config.lock().unwrap().fail_after_n_operations = Some(n);
            self
        }

        async fn simulate_delay_and_failures(&self) -> Result<()> {
            let mut config = self.config.lock().unwrap();
            config.operation_count += 1;

            if config.should_fail_auth {
                return Err(StorageError::AuthenticationFailed {
                    message: "Mock authentication failure".to_string(),
                }.into());
            }

            if config.should_fail_network {
                return Err(StorageError::NetworkError {
                    message: "Mock network failure".to_string(),
                }.into());
            }

            if let Some(fail_after) = config.fail_after_n_operations {
                if config.operation_count > fail_after {
                    return Err(StorageError::NetworkError {
                        message: "Mock failure after N operations".to_string(),
                    }.into());
                }
            }

            if config.upload_delay_ms > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(config.upload_delay_ms)).await;
            }

            Ok(())
        }

        /// Get the number of operations performed
        pub fn operation_count(&self) -> usize {
            self.config.lock().unwrap().operation_count
        }

        /// Reset the mock state
        pub fn reset(&self) {
            self.files.lock().unwrap().clear();
            let mut config = self.config.lock().unwrap();
            config.operation_count = 0;
            config.should_fail_auth = false;
            config.should_fail_network = false;
            config.upload_delay_ms = 0;
            config.fail_after_n_operations = None;
        }
    }

    #[async_trait]
    impl RemoteStorage for EnhancedMockStorage {
        async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<()> {
            self.simulate_delay_and_failures().await?;

            let content = tokio::fs::read(local_path).await?;
            let hash = format!("{:x}", sha2::Sha256::digest(&content));

            let mock_file = MockFile {
                content,
                hash,
                last_modified: chrono::Utc::now(),
            };

            self.files.lock().unwrap().insert(remote_key.to_string(), mock_file);
            Ok(())
        }

        async fn download_file(&self, remote_key: &str, local_path: &Path) -> Result<()> {
            self.simulate_delay_and_failures().await?;

            let files = self.files.lock().unwrap();
            let mock_file = files.get(remote_key)
                .ok_or_else(|| StorageError::FileNotFound { path: remote_key.to_string() })?;

            if let Some(parent) = local_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(local_path, &mock_file.content).await?;
            Ok(())
        }

        async fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFileInfo>> {
            self.simulate_delay_and_failures().await?;

            let files = self.files.lock().unwrap();
            let mut result = Vec::new();

            for (key, mock_file) in files.iter() {
                if key.starts_with(prefix) {
                    result.push(RemoteFileInfo {
                        key: key.clone(),
                        size: Some(mock_file.content.len() as u64),
                        hash: Some(mock_file.hash.clone()),
                        last_modified: Some(mock_file.last_modified),
                    });
                }
            }

            Ok(result)
        }

        async fn file_exists(&self, remote_key: &str) -> Result<bool> {
            self.simulate_delay_and_failures().await?;
            Ok(self.files.lock().unwrap().contains_key(remote_key))
        }

        async fn get_file_info(&self, remote_key: &str) -> Result<Option<RemoteFileInfo>> {
            self.simulate_delay_and_failures().await?;

            let files = self.files.lock().unwrap();
            if let Some(mock_file) = files.get(remote_key) {
                Ok(Some(RemoteFileInfo {
                    key: remote_key.to_string(),
                    size: Some(mock_file.content.len() as u64),
                    hash: Some(mock_file.hash.clone()),
                    last_modified: Some(mock_file.last_modified),
                }))
            } else {
                Ok(None)
            }
        }

        async fn delete_file(&self, remote_key: &str) -> Result<()> {
            self.simulate_delay_and_failures().await?;
            self.files.lock().unwrap().remove(remote_key);
            Ok(())
        }
    }
}
```

### 5. Integration Tests

Create `tests/integration_tests.rs`:

```rust
//! Integration tests for DevLog push functionality

use devlog::{DevLogConfig, SyncManager, PushOptions, PushMode};
use devlog::config::{RemoteConfig, SyncConfig};
use devlog::remote::test_utils::EnhancedMockStorage;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_full_push_workflow() {
    let temp_dir = tempdir().unwrap();

    // Create test file structure
    let events_dir = temp_dir.path().join("events");
    fs::create_dir_all(&events_dir).unwrap();
    fs::write(events_dir.join("20250911.jsonl"), r#"{"event": "test"}"#).unwrap();

    let entries_dir = temp_dir.path().join("entries");
    fs::create_dir_all(&entries_dir).unwrap();
    fs::write(entries_dir.join("20250911.md"), "# Test Entry").unwrap();

    // Create configuration
    let config = DevLogConfig {
        remote: RemoteConfig {
            provider: "azure".to_string(),
            url: "https://test.blob.core.windows.net/devlog".to_string(),
        },
        sync: SyncConfig {
            last_push_timestamp: None,
        },
    };

    // Test with mock storage
    let mock_storage = EnhancedMockStorage::new();

    // This would require dependency injection in the actual implementation
    // For now, we'll test the components separately
}

#[tokio::test]
async fn test_incremental_push_logic() {
    // Test that incremental push only uploads changed files
}

#[tokio::test]
async fn test_retry_mechanism() {
    // Test that failed uploads are retried appropriately
}

#[tokio::test]
async fn test_error_recovery() {
    // Test that partial failures don't corrupt state
}

#[test]
fn test_cli_argument_parsing() {
    // Test that CLI arguments are parsed correctly
}
```

### 6. Property-Based Tests

Add property-based testing with `proptest`:

```toml
# Add to Cargo.toml [dev-dependencies]
proptest = "1.0"
```

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_file_hashing_properties(content in ".*") {
            // Property: same content should always produce same hash
            let hash1 = calculate_string_hash(&content);
            let hash2 = calculate_string_hash(&content);
            prop_assert_eq!(hash1, hash2);

            // Property: hash should be 64 characters (SHA256)
            prop_assert_eq!(hash1.len(), 64);

            // Property: hash should be valid hex
            prop_assert!(hash1.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn test_glob_pattern_properties(
            pattern in "[a-zA-Z0-9*?/.-]{1,50}",
            text in "[a-zA-Z0-9/.-]{1,50}"
        ) {
            // Property: glob matching should not panic
            let _ = glob_match(&pattern, &text);
        }

        #[test]
        fn test_remote_key_validation_properties(key in ".*") {
            // Property: validation should not panic
            let result = crate::remote::utils::validate_remote_key(&key);

            // Property: empty key should always be invalid
            if key.is_empty() {
                prop_assert!(result.is_err());
            }

            // Property: keys with .. should be invalid
            if key.contains("..") {
                prop_assert!(result.is_err());
            }

            // Property: keys starting with / should be invalid
            if key.starts_with('/') {
                prop_assert!(result.is_err());
            }
        }
    }
}
```

### 7. Performance and Stress Tests

Create performance benchmarks:

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_file_scanning_performance() {
        let temp_dir = tempdir().unwrap();

        // Create 1000 files
        let events_dir = temp_dir.path().join("events");
        fs::create_dir_all(&events_dir).unwrap();

        for i in 0..1000 {
            fs::write(
                events_dir.join(format!("event_{}.jsonl", i)),
                format!(r#"{{"id": {}, "data": "test data for file {}" }}"#, i, i)
            ).unwrap();
        }

        let scanner = FileScanner::with_base_path(temp_dir.path().to_path_buf());

        let start = Instant::now();
        let files = scanner.scan_files().unwrap();
        let duration = start.elapsed();

        assert_eq!(files.len(), 1000);
        assert!(duration.as_millis() < 5000, "Scanning took too long: {:?}", duration);

        println!("Scanned {} files in {:?}", files.len(), duration);
    }

    #[test]
    fn test_file_hashing_performance() {
        let temp_dir = tempdir().unwrap();

        // Create a large file (1MB)
        let large_content = "x".repeat(1024 * 1024);
        let large_file = temp_dir.path().join("large_file.txt");
        fs::write(&large_file, &large_content).unwrap();

        let start = Instant::now();
        let hash = calculate_file_hash(&large_file).unwrap();
        let duration = start.elapsed();

        assert_eq!(hash.len(), 64);
        assert!(duration.as_millis() < 1000, "Hashing 1MB took too long: {:?}", duration);

        println!("Hashed 1MB file in {:?}", duration);
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        use tokio::task;
        use std::sync::Arc;

        let mock_storage = Arc::new(EnhancedMockStorage::new());
        let temp_dir = tempdir().unwrap();

        // Create multiple test files
        for i in 0..10 {
            fs::write(
                temp_dir.path().join(format!("file_{}.txt", i)),
                format!("Content for file {}", i)
            ).unwrap();
        }

        // Upload files concurrently
        let mut handles = Vec::new();
        for i in 0..10 {
            let storage = Arc::clone(&mock_storage);
            let file_path = temp_dir.path().join(format!("file_{}.txt", i));
            let remote_key = format!("file_{}.txt", i);

            let handle = task::spawn(async move {
                storage.upload_file(&file_path, &remote_key).await
            });
            handles.push(handle);
        }

        // Wait for all uploads to complete
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // Verify all files were uploaded
        let files = mock_storage.list_files("").await.unwrap();
        assert_eq!(files.len(), 10);
    }
}
```

## Validation Steps

### 1. Run All Tests

```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture

# Run specific test modules
cargo test config
cargo test local
cargo test remote
```

### 2. Performance Benchmarks

```bash
# Run performance tests
cargo test performance_tests -- --nocapture

# Run with release optimizations
cargo test --release performance_tests
```

### 3. Property-Based Testing

```bash
# Run property tests with more cases
PROPTEST_CASES=10000 cargo test property_tests
```

### 4. Coverage Analysis

```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Expected Outputs

After completing this task:

- ✅ Comprehensive unit tests cover all modules
- ✅ Integration tests validate end-to-end workflows
- ✅ Property-based tests catch edge cases
- ✅ Performance tests ensure acceptable speed
- ✅ All tests pass consistently
- ✅ Code coverage is above 80%

### Sample Test Output

```
running 45 tests
test config::tests::test_config_serialization ... ok
test config::tests::test_config_validation_cases ... ok
test local::tests::test_file_scanner_inclusion_exclusion ... ok
test local::tests::test_file_hashing_consistency ... ok
test remote::tests::test_mock_storage ... ok
test sync::tests::test_push_dry_run ... ok
test performance_tests::test_file_scanning_performance ... ok

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Next Steps

Once this task is complete, proceed to **Task 10: Documentation & Polish** for final documentation and user experience improvements.

## Rust Learning Notes

**Key Concepts Introduced**:

- **Test Organization**: Structuring tests in Rust projects
- **Property-Based Testing**: Using proptest for edge case discovery
- **Async Testing**: Testing async functions with tokio::test
- **Mock Objects**: Creating test doubles for external dependencies
- **Performance Testing**: Measuring and validating performance characteristics
