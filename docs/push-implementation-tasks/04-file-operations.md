# Task 04: File Operations and Local Utilities

**Estimated Time**: 2-3 hours  
**Difficulty**: ⭐⭐ Beginner-Intermediate  
**Prerequisites**: Tasks 01, 02, and 03 completed

## Objective

Implement file scanning, hashing, and local utilities needed for detecting changes and managing files in the `.devlog` directory.

## What You'll Learn

- File system operations in Rust
- Directory traversal and filtering
- File hashing for change detection
- Pattern matching with glob patterns
- Working with file metadata

## Tasks

### 1. Define Local File Structures

Create a new file `src/local.rs` for local file operations:

```rust
//! Local file operations and utilities for DevLog

use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use anyhow::Result;
use sha2::{Sha256, Digest};
use std::io::Read;

/// Represents a local file with metadata
#[derive(Debug, Clone)]
pub struct LocalFile {
    pub path: PathBuf,
    pub relative_path: String,
    pub size: u64,
    pub hash: String,
    pub modified_time: DateTime<Utc>,
}

impl LocalFile {
    /// Create a LocalFile from a path, calculating hash and metadata
    pub fn from_path(path: &Path, base_path: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;

        if !metadata.is_file() {
            anyhow::bail!("Path is not a file: {:?}", path);
        }

        let relative_path = path.strip_prefix(base_path)
            .map_err(|_| anyhow::anyhow!("Path {:?} is not under base path {:?}", path, base_path))?
            .to_string_lossy()
            .replace('\\', "/");

        let hash = calculate_file_hash(path)?;

        let modified_time = metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let modified_time = DateTime::from_timestamp(modified_time as i64, 0)
            .unwrap_or_else(Utc::now);

        Ok(LocalFile {
            path: path.to_path_buf(),
            relative_path,
            size: metadata.len(),
            hash,
            modified_time,
        })
    }

    /// Get the remote key for this file (same as relative_path)
    pub fn remote_key(&self) -> &str {
        &self.relative_path
    }
}
```

### 2. Implement File Scanning

Add file discovery functionality:

```rust
/// File scanner for discovering files in the DevLog directory
pub struct FileScanner {
    base_path: PathBuf,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl FileScanner {
    /// Create a new scanner for the DevLog directory
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Unable to determine home directory"))?;

        let base_path = home_dir.join(".devlog");

        Ok(Self {
            base_path,
            include_patterns: vec![
                "events/*.jsonl".to_string(),
                "entries/*.md".to_string(),
                "config.toml".to_string(),
            ],
            exclude_patterns: vec![
                "cache/*".to_string(),
                "*.tmp".to_string(),
                "*.lock".to_string(),
            ],
        })
    }

    /// Create a scanner with custom base path (for testing)
    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self {
            base_path,
            include_patterns: vec![
                "events/*.jsonl".to_string(),
                "entries/*.md".to_string(),
                "config.toml".to_string(),
            ],
            exclude_patterns: vec![
                "cache/*".to_string(),
                "*.tmp".to_string(),
                "*.lock".to_string(),
            ],
        }
    }

    /// Scan for all matching files
    pub fn scan_files(&self) -> Result<Vec<LocalFile>> {
        let mut files = Vec::new();

        if !self.base_path.exists() {
            return Ok(files);
        }

        self.scan_directory(&self.base_path, &mut files)?;
        Ok(files)
    }

    /// Recursively scan a directory
    fn scan_directory(&self, dir: &Path, files: &mut Vec<LocalFile>) -> Result<()> {
        let entries = std::fs::read_dir(dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if self.should_include_file(&path) {
                    match LocalFile::from_path(&path, &self.base_path) {
                        Ok(local_file) => files.push(local_file),
                        Err(e) => {
                            eprintln!("Warning: Failed to process file {:?}: {}", path, e);
                        }
                    }
                }
            } else if path.is_dir() {
                self.scan_directory(&path, files)?;
            }
        }

        Ok(())
    }

    /// Check if a file should be included based on patterns
    fn should_include_file(&self, path: &Path) -> bool {
        let relative_path = match path.strip_prefix(&self.base_path) {
            Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
            Err(_) => return false,
        };

        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if glob_match(pattern, &relative_path) {
                return false;
            }
        }

        // Check include patterns
        for pattern in &self.include_patterns {
            if glob_match(pattern, &relative_path) {
                return true;
            }
        }

        false
    }

    /// Get the base path for scanning
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}
```

### 3. Add Hashing Utilities

Implement efficient file hashing:

```rust
/// Calculate SHA256 hash of a file
pub fn calculate_file_hash(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // 8KB buffer for efficient reading

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Calculate hash of a string (for testing)
pub fn calculate_string_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### 4. Implement Simple Glob Matching

Add basic glob pattern matching:

```rust
/// Simple glob pattern matching
/// Supports: * (any characters), ? (single character)
pub fn glob_match(pattern: &str, text: &str) -> bool {
    glob_match_recursive(pattern.as_bytes(), text.as_bytes())
}

fn glob_match_recursive(pattern: &[u8], text: &[u8]) -> bool {
    match (pattern.first(), text.first()) {
        (None, None) => true,
        (Some(b'*'), _) => {
            // Try matching with the rest of the pattern
            glob_match_recursive(&pattern[1..], text) ||
            // Or consume one character from text and try again
            (!text.is_empty() && glob_match_recursive(pattern, &text[1..]))
        }
        (Some(b'?'), Some(_)) => {
            // ? matches any single character
            glob_match_recursive(&pattern[1..], &text[1..])
        }
        (Some(p), Some(t)) if p == t => {
            // Exact character match
            glob_match_recursive(&pattern[1..], &text[1..])
        }
        _ => false,
    }
}
```

### 5. Add Change Detection

Implement utilities for detecting changed files:

```rust
/// Utilities for change detection
pub mod changes {
    use super::*;
    use crate::remote::{RemoteStorage, RemoteFileInfo};
    use std::collections::HashMap;

    /// Represents a file change
    #[derive(Debug, Clone)]
    pub enum FileChange {
        Added(LocalFile),
        Modified(LocalFile),
        Deleted(String), // remote key
    }

    /// Detect changes between local and remote files
    pub async fn detect_changes(
        local_files: &[LocalFile],
        remote_storage: &dyn RemoteStorage,
    ) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();

        // Get remote file list
        let remote_files = remote_storage.list_files("").await?;
        let remote_map: HashMap<String, RemoteFileInfo> = remote_files
            .into_iter()
            .map(|f| (f.key.clone(), f))
            .collect();

        // Create local file map
        let local_map: HashMap<String, &LocalFile> = local_files
            .iter()
            .map(|f| (f.remote_key().to_string(), f))
            .collect();

        // Check for added and modified files
        for local_file in local_files {
            let remote_key = local_file.remote_key();

            match remote_map.get(remote_key) {
                None => {
                    // File doesn't exist remotely - it's new
                    changes.push(FileChange::Added(local_file.clone()));
                }
                Some(remote_file) => {
                    // File exists remotely - check if it's changed
                    if let Some(remote_hash) = &remote_file.hash {
                        if &local_file.hash != remote_hash {
                            changes.push(FileChange::Modified(local_file.clone()));
                        }
                    } else {
                        // No hash available, consider it modified
                        changes.push(FileChange::Modified(local_file.clone()));
                    }
                }
            }
        }

        // Check for deleted files (exist remotely but not locally)
        for remote_key in remote_map.keys() {
            if !local_map.contains_key(remote_key) {
                changes.push(FileChange::Deleted(remote_key.clone()));
            }
        }

        Ok(changes)
    }

    /// Filter changes based on timestamp
    pub fn filter_changes_since(
        changes: Vec<FileChange>,
        since: DateTime<Utc>,
    ) -> Vec<FileChange> {
        changes
            .into_iter()
            .filter(|change| match change {
                FileChange::Added(file) | FileChange::Modified(file) => {
                    file.modified_time > since
                }
                FileChange::Deleted(_) => true, // Always include deletions
            })
            .collect()
    }
}
```

### 6. Update Module Declaration

Add the local module to your `src/main.rs` or `src/lib.rs`:

```rust
mod local;
```

### 7. Add Required Dependencies

Update your `Cargo.toml`:

```toml
dirs = "5.0"  # Should already be added from Task 02
```

## Validation Steps

### 1. Unit Tests

Create comprehensive tests in `src/local.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_local_file_creation() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let local_file = LocalFile::from_path(&test_file, temp_dir.path()).unwrap();

        assert_eq!(local_file.relative_path, "test.txt");
        assert_eq!(local_file.size, 13);
        assert!(!local_file.hash.is_empty());
    }

    #[test]
    fn test_file_scanner() {
        let temp_dir = tempdir().unwrap();

        // Create test directory structure
        fs::create_dir_all(temp_dir.path().join("events")).unwrap();
        fs::create_dir_all(temp_dir.path().join("entries")).unwrap();

        // Create test files
        fs::write(temp_dir.path().join("config.toml"), "test").unwrap();
        fs::write(temp_dir.path().join("events/test.jsonl"), "{}").unwrap();
        fs::write(temp_dir.path().join("entries/test.md"), "# Test").unwrap();
        fs::write(temp_dir.path().join("cache/temp.tmp"), "temp").unwrap(); // Should be excluded

        let scanner = FileScanner::with_base_path(temp_dir.path().to_path_buf());
        let files = scanner.scan_files().unwrap();

        assert_eq!(files.len(), 3); // config.toml, events/test.jsonl, entries/test.md

        let keys: Vec<&str> = files.iter().map(|f| f.remote_key()).collect();
        assert!(keys.contains(&"config.toml"));
        assert!(keys.contains(&"events/test.jsonl"));
        assert!(keys.contains(&"entries/test.md"));
    }

    #[test]
    fn test_glob_matching() {
        assert!(glob_match("*.txt", "file.txt"));
        assert!(glob_match("events/*.jsonl", "events/test.jsonl"));
        assert!(!glob_match("*.txt", "file.md"));
        assert!(glob_match("test?", "testa"));
        assert!(!glob_match("test?", "testab"));
    }

    #[test]
    fn test_file_hashing() {
        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let hash1 = calculate_file_hash(&test_file).unwrap();
        let hash2 = calculate_file_hash(&test_file).unwrap();

        assert_eq!(hash1, hash2); // Same file should have same hash
        assert_eq!(hash1.len(), 64); // SHA256 produces 64 hex characters
    }

    #[tokio::test]
    async fn test_change_detection() {
        use crate::remote::MockStorage;

        let temp_dir = tempdir().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").unwrap();

        let local_file = LocalFile::from_path(&test_file, temp_dir.path()).unwrap();
        let local_files = vec![local_file];

        let storage = MockStorage::new();
        let changes = changes::detect_changes(&local_files, &storage).await.unwrap();

        assert_eq!(changes.len(), 1);
        matches!(changes[0], changes::FileChange::Added(_));
    }
}
```

### 2. Integration Test

Create a simple integration test:

```bash
# Create a test script
cargo test local

# Test with actual files
mkdir -p /tmp/test-devlog/events
echo '{}' > /tmp/test-devlog/events/test.jsonl
echo 'provider = "azure"' > /tmp/test-devlog/config.toml
```

## Expected Outputs

After completing this task:

- ✅ `LocalFile` struct properly represents files with metadata
- ✅ `FileScanner` discovers relevant files in `.devlog` directory
- ✅ File hashing works correctly and consistently
- ✅ Glob pattern matching handles include/exclude patterns
- ✅ Change detection compares local and remote file states
- ✅ All tests pass without errors

## Troubleshooting

**Common Issues**:

1. **Path Separator Issues**: Use `replace('\\', "/")` for cross-platform compatibility
2. **Permission Errors**: Ensure test directories are writable
3. **File Handle Errors**: Make sure files are properly closed after reading
4. **Hash Consistency**: Verify that identical files produce identical hashes

**Testing Commands**:

```bash
# Run all local tests
cargo test local

# Test specific functions
cargo test test_file_scanner

# Check for memory leaks in file operations
cargo test --release
```

## Next Steps

Once this task is complete, proceed to **Task 05: CLI Command Structure** where we'll add the push command to the CLI interface.

## Rust Learning Notes

**Key Concepts Introduced**:

- **File I/O**: Reading files efficiently with buffers
- **Path Manipulation**: Cross-platform path handling
- **Pattern Matching**: Implementing glob patterns
- **Metadata**: Working with file system metadata
- **Collections**: Using HashMap for efficient lookups

**Questions to Research**:

1. Why do we use a buffer when reading files for hashing?
2. How does `strip_prefix` work and when can it fail?
3. What's the difference between `PathBuf` and `&Path`?
4. How does Rust handle file system operations across different platforms?
