# Task 07: Sync Manager Implementation

**Estimated Time**: 3-4 hours  
**Difficulty**: ⭐⭐⭐ Intermediate-Advanced  
**Prerequisites**: Tasks 01-06 completed

## Objective

Implement the core synchronization logic that orchestrates file scanning, change detection, and upload operations using the Azure storage client and local file utilities.

## What You'll Learn

- Orchestrating complex async operations
- Change detection algorithms
- Progress tracking and reporting
- Error recovery and partial failure handling
- State management for sync operations

## Tasks

### 1. Define Sync Data Structures

In `src/sync.rs`, define the core data structures:

```rust
//! Synchronization manager and logic for DevLog

use crate::config::DevLogConfig;
use crate::local::{FileScanner, LocalFile, changes::{FileChange, detect_changes}};
use crate::remote::{RemoteStorage, StorageFactory};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::path::Path;

/// Options for push operations
#[derive(Debug, Clone)]
pub struct PushOptions {
    pub mode: PushMode,
    pub force: bool,
    pub dry_run: bool,
}

/// Push mode enum (re-exported from CLI)
#[derive(Debug, Clone)]
pub enum PushMode {
    Incremental,
    All,
}

/// Result of a push operation
#[derive(Debug)]
pub struct PushResult {
    pub files_uploaded: usize,
    pub files_skipped: usize,
    pub total_bytes: u64,
    pub duration: std::time::Duration,
    pub errors: Vec<SyncError>,
}

/// Sync-specific errors
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("File upload failed: {path} - {message}")]
    UploadFailed { path: String, message: String },

    #[error("File scan failed: {message}")]
    ScanFailed { message: String },

    #[error("Change detection failed: {message}")]
    ChangeDetectionFailed { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

/// Progress callback for reporting upload progress
pub trait ProgressReporter: Send + Sync {
    fn report_scan_start(&self, base_path: &Path);
    fn report_scan_complete(&self, file_count: usize);
    fn report_change_detection_start(&self);
    fn report_change_detection_complete(&self, changes: usize);
    fn report_upload_start(&self, file_count: usize, total_bytes: u64);
    fn report_file_upload_start(&self, file_path: &str, size: u64);
    fn report_file_upload_complete(&self, file_path: &str);
    fn report_file_upload_error(&self, file_path: &str, error: &str);
    fn report_upload_complete(&self, result: &PushResult);
}

/// Console progress reporter
pub struct ConsoleProgressReporter {
    start_time: std::sync::Mutex<Option<std::time::Instant>>,
}

impl ConsoleProgressReporter {
    pub fn new() -> Self {
        Self {
            start_time: std::sync::Mutex::new(None),
        }
    }
}

impl ProgressReporter for ConsoleProgressReporter {
    fn report_scan_start(&self, base_path: &Path) {
        println!("🔍 Scanning files in {:?}...", base_path);
    }

    fn report_scan_complete(&self, file_count: usize) {
        println!("📁 Found {} files to consider", file_count);
    }

    fn report_change_detection_start(&self) {
        println!("🔄 Detecting changes...");
    }

    fn report_change_detection_complete(&self, changes: usize) {
        if changes == 0 {
            println!("✨ No changes detected");
        } else {
            println!("📝 {} file(s) need to be uploaded", changes);
        }
    }

    fn report_upload_start(&self, file_count: usize, total_bytes: u64) {
        *self.start_time.lock().unwrap() = Some(std::time::Instant::now());
        println!("⬆️  Uploading {} file(s) ({} bytes)...", file_count, total_bytes);
    }

    fn report_file_upload_start(&self, file_path: &str, size: u64) {
        println!("  📤 {} ({} bytes)", file_path, size);
    }

    fn report_file_upload_complete(&self, file_path: &str) {
        println!("  ✅ {}", file_path);
    }

    fn report_file_upload_error(&self, file_path: &str, error: &str) {
        println!("  ❌ {}: {}", file_path, error);
    }

    fn report_upload_complete(&self, result: &PushResult) {
        let duration = self.start_time.lock().unwrap()
            .map(|start| start.elapsed())
            .unwrap_or(result.duration);

        println!();
        if result.files_uploaded > 0 {
            println!("✅ Upload complete!");
            println!("   Files uploaded: {}", result.files_uploaded);
            if result.files_skipped > 0 {
                println!("   Files skipped: {}", result.files_skipped);
            }
            println!("   Total size: {} bytes", result.total_bytes);
            println!("   Duration: {:.2}s", duration.as_secs_f64());
        } else {
            println!("✨ No files needed uploading");
        }

        if !result.errors.is_empty() {
            println!("⚠️  {} error(s) occurred:", result.errors.len());
            for error in &result.errors {
                println!("   • {}", error);
            }
        }
    }
}
```

### 2. Implement the Core Sync Manager

```rust
/// The main synchronization manager
pub struct SyncManager {
    config: DevLogConfig,
    remote_storage: Box<dyn RemoteStorage>,
    file_scanner: FileScanner,
}

impl SyncManager {
    /// Create a new sync manager
    pub async fn new() -> Result<Self> {
        let config = DevLogConfig::load()?;
        config.validate()?;

        let remote_storage = StorageFactory::create_storage(&config.remote)?;
        let file_scanner = FileScanner::new()?;

        Ok(Self {
            config,
            remote_storage,
            file_scanner,
        })
    }

    /// Create sync manager with custom configuration (for testing)
    pub async fn with_config(config: DevLogConfig) -> Result<Self> {
        config.validate()?;

        let remote_storage = StorageFactory::create_storage(&config.remote)?;
        let file_scanner = FileScanner::new()?;

        Ok(Self {
            config,
            remote_storage,
            file_scanner,
        })
    }

    /// Perform a push operation
    pub async fn push(
        &mut self,
        options: PushOptions,
        progress: Option<&dyn ProgressReporter>,
    ) -> Result<PushResult> {
        let start_time = std::time::Instant::now();
        let mut result = PushResult {
            files_uploaded: 0,
            files_skipped: 0,
            total_bytes: 0,
            duration: std::time::Duration::default(),
            errors: Vec::new(),
        };

        // Step 1: Scan local files
        if let Some(reporter) = progress {
            reporter.report_scan_start(self.file_scanner.base_path());
        }

        let local_files = match self.file_scanner.scan_files() {
            Ok(files) => files,
            Err(e) => {
                result.errors.push(SyncError::ScanFailed {
                    message: e.to_string(),
                });
                result.duration = start_time.elapsed();
                return Ok(result);
            }
        };

        if let Some(reporter) = progress {
            reporter.report_scan_complete(local_files.len());
        }

        // Step 2: Determine which files to upload
        let files_to_upload = match options.mode {
            PushMode::All => {
                if options.force {
                    local_files
                } else {
                    // Even in "all" mode, skip files that haven't changed unless forced
                    self.filter_changed_files(local_files, progress).await?
                }
            }
            PushMode::Incremental => {
                self.filter_incremental_files(local_files, progress).await?
            }
        };

        if let Some(reporter) = progress {
            reporter.report_change_detection_complete(files_to_upload.len());
        }

        // Step 3: Upload files
        if !files_to_upload.is_empty() && !options.dry_run {
            result = self.upload_files(files_to_upload, progress, result).await;

            // Step 4: Update sync metadata on success
            if result.errors.is_empty() {
                if let Err(e) = self.config.update_last_push_timestamp() {
                    result.errors.push(SyncError::ConfigurationError {
                        message: format!("Failed to update sync timestamp: {}", e),
                    });
                }
            }
        } else if options.dry_run {
            result.files_skipped = files_to_upload.len();
            result.total_bytes = files_to_upload.iter().map(|f| f.size).sum();

            if let Some(reporter) = progress {
                println!("🔍 Dry run - would upload {} file(s):", files_to_upload.len());
                for file in &files_to_upload {
                    println!("  📤 {} ({} bytes)", file.relative_path, file.size);
                }
            }
        }

        result.duration = start_time.elapsed();

        if let Some(reporter) = progress {
            reporter.report_upload_complete(&result);
        }

        Ok(result)
    }

    /// Filter files that have changed (for "all" mode without force)
    async fn filter_changed_files(
        &self,
        local_files: Vec<LocalFile>,
        progress: Option<&dyn ProgressReporter>,
    ) -> Result<Vec<LocalFile>> {
        if let Some(reporter) = progress {
            reporter.report_change_detection_start();
        }

        let changes = detect_changes(&local_files, self.remote_storage.as_ref()).await
            .map_err(|e| SyncError::ChangeDetectionFailed {
                message: e.to_string(),
            })?;

        let files_to_upload: Vec<LocalFile> = changes
            .into_iter()
            .filter_map(|change| match change {
                FileChange::Added(file) | FileChange::Modified(file) => Some(file),
                FileChange::Deleted(_) => None, // We don't handle deletions in push
            })
            .collect();

        Ok(files_to_upload)
    }

    /// Filter files for incremental upload (since last push timestamp)
    async fn filter_incremental_files(
        &self,
        local_files: Vec<LocalFile>,
        progress: Option<&dyn ProgressReporter>,
    ) -> Result<Vec<LocalFile>> {
        if let Some(reporter) = progress {
            reporter.report_change_detection_start();
        }

        if let Some(last_push) = self.config.sync.last_push_timestamp {
            // Filter files modified since last push
            let recent_files: Vec<LocalFile> = local_files
                .into_iter()
                .filter(|file| file.modified_time > last_push)
                .collect();

            // Still check against remote to avoid re-uploading unchanged files
            self.filter_changed_files(recent_files, None).await
        } else {
            // First push - upload all files
            self.filter_changed_files(local_files, None).await
        }
    }

    /// Upload a list of files
    async fn upload_files(
        &self,
        files_to_upload: Vec<LocalFile>,
        progress: Option<&dyn ProgressReporter>,
        mut result: PushResult,
    ) -> PushResult {
        let total_bytes: u64 = files_to_upload.iter().map(|f| f.size).sum();

        if let Some(reporter) = progress {
            reporter.report_upload_start(files_to_upload.len(), total_bytes);
        }

        for file in files_to_upload {
            if let Some(reporter) = progress {
                reporter.report_file_upload_start(&file.relative_path, file.size);
            }

            match self.remote_storage.upload_file(&file.path, &file.remote_key()).await {
                Ok(()) => {
                    result.files_uploaded += 1;
                    result.total_bytes += file.size;

                    if let Some(reporter) = progress {
                        reporter.report_file_upload_complete(&file.relative_path);
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    result.errors.push(SyncError::UploadFailed {
                        path: file.relative_path.clone(),
                        message: error_msg.clone(),
                    });

                    if let Some(reporter) = progress {
                        reporter.report_file_upload_error(&file.relative_path, &error_msg);
                    }
                }
            }
        }

        result
    }
}
```

### 3. Update CLI Integration

Update the push command handler in `src/cli.rs` to use the sync manager:

```rust
/// Handle the push command
async fn handle_push_command(
    mode: PushMode,
    force: bool,
    dry_run: bool
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::sync::{SyncManager, PushOptions, ConsoleProgressReporter};

    let options = PushOptions {
        mode: match mode {
            crate::cli::PushMode::Incremental => crate::sync::PushMode::Incremental,
            crate::cli::PushMode::All => crate::sync::PushMode::All,
        },
        force,
        dry_run,
    };

    println!("🔄 DevLog Push");
    if dry_run {
        println!("🔍 Dry run mode - no files will actually be uploaded");
    }
    if force {
        println!("💪 Force mode - uploading all files regardless of changes");
    }
    println!();

    // Create sync manager
    let mut sync_manager = SyncManager::new().await?;

    // Create progress reporter
    let progress_reporter = ConsoleProgressReporter::new();

    // Perform the push
    let result = sync_manager.push(options, Some(&progress_reporter)).await?;

    // Exit with error code if there were upload failures
    if !result.errors.is_empty() {
        std::process::exit(1);
    }

    Ok(())
}
```

### 4. Update main.rs for Async CLI

Update the CLI run method to be async:

```rust
impl Cli {
    /// Run the CLI application
    pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::parse();

        // TODO: read user defined storage path
        // For now, we use the default `base_dir`, which is `~/.devlog`
        let storage = LocalEntryStorage::new(None)?;

        match cli.command {
            Commands::New { message, id } => {
                Self::handle_new_command(message, id, &storage)?;
            }
            Commands::Edit { id } => {
                Self::handle_edit_command(id, &storage)?;
            }
            Commands::Show { id, formatted } => {
                Self::handle_show_command(id, formatted, &storage)?;
            }
            Commands::List => {
                Self::handle_list_command(&storage)?;
            }
            Commands::Push { mode, force, dry_run } => {
                Self::handle_push_command(mode, force, dry_run).await?;
            }
        }

        Ok(())
    }

    // ... existing methods remain the same ...
}
```

And update `src/main.rs`:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::Cli::run().await
}
```

### 5. Add Module Dependencies

Update `src/sync.rs` imports and make sure to include the local module:

```rust
use crate::config::DevLogConfig;
use crate::local::{FileScanner, LocalFile};
use crate::local::changes::{FileChange, detect_changes};
use crate::remote::{RemoteStorage, StorageFactory};
```

## Validation Steps

### 1. Unit Tests

Create comprehensive tests in `src/sync.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::remote::MockStorage;
    use crate::config::{RemoteConfig, SyncConfig};
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_sync_manager_creation() {
        // This will fail without proper config, which is expected for testing
        match SyncManager::new().await {
            Ok(_) => println!("Sync manager created"),
            Err(e) => println!("Expected error without config: {}", e),
        }
    }

    #[tokio::test]
    async fn test_push_dry_run() {
        let temp_dir = tempdir().unwrap();

        // Create test files
        fs::create_dir_all(temp_dir.path().join("events")).unwrap();
        fs::write(temp_dir.path().join("events/test.jsonl"), "{}").unwrap();

        let config = DevLogConfig {
            remote: RemoteConfig {
                provider: "azure".to_string(),
                url: "https://test.blob.core.windows.net/test".to_string(),
            },
            sync: SyncConfig {
                last_push_timestamp: None,
            },
        };

        // This test would need a mock implementation to work fully
        // For now, we'll just test the structure
    }

    #[test]
    fn test_progress_reporter() {
        let reporter = ConsoleProgressReporter::new();
        reporter.report_scan_start(Path::new("/test"));
        reporter.report_scan_complete(5);
        reporter.report_change_detection_start();
        reporter.report_change_detection_complete(3);

        // These should not panic
    }
}
```

### 2. Integration Test

Test the full flow with a real configuration:

```bash
# Set up environment
export AZURE_STORAGE_ACCOUNT_KEY="your_key"

# Create some test files
mkdir -p ~/.devlog/events
echo '{"event": "test"}' > ~/.devlog/events/$(date +%Y%m%d).jsonl

# Test dry run
cargo build
./target/debug/devlog push --dry-run

# Test actual push
./target/debug/devlog push --mode all
```

### 3. Performance Test

Create a test with multiple files to verify performance:

```bash
# Create multiple test files
for i in {1..10}; do
    echo '{"event": "test '$i'"}' > ~/.devlog/events/test$i.jsonl
done

# Time the push operation
time ./target/debug/devlog push --mode all
```

## Expected Outputs

After completing this task:

- ✅ Sync manager coordinates all aspects of the push operation
- ✅ Progress reporting provides clear feedback to users
- ✅ Change detection works for both incremental and all modes
- ✅ File uploads handle errors gracefully and continue with remaining files
- ✅ Sync metadata is properly updated after successful uploads
- ✅ Dry run mode shows what would be uploaded without making changes

### Sample Output

When running `devlog push`, you should see:

```
🔄 DevLog Push

🔍 Scanning files in "/Users/username/.devlog"...
📁 Found 3 files to consider
🔄 Detecting changes...
📝 2 file(s) need to be uploaded
⬆️  Uploading 2 file(s) (156 bytes)...
  📤 events/20250911.jsonl (89 bytes)
  ✅ events/20250911.jsonl
  📤 config.toml (67 bytes)
  ✅ config.toml

✅ Upload complete!
   Files uploaded: 2
   Total size: 156 bytes
   Duration: 1.23s
```

## Troubleshooting

**Common Issues**:

1. **Async Compilation Errors**: Make sure all functions that call async operations are marked as async
2. **Trait Object Errors**: Ensure progress reporter trait is object-safe
3. **Borrow Checker Issues**: Use proper references and lifetimes for the sync manager
4. **Module Import Errors**: Verify all modules are properly declared and imported

**Testing Commands**:

```bash
# Check async compilation
cargo check

# Run sync-specific tests
cargo test sync

# Test full integration
cargo build && ./target/debug/devlog push --dry-run
```

## Next Steps

Once this task is complete, proceed to **Task 08: Progress & Error Handling** where we'll enhance the user experience with better progress indication and error recovery.

## Rust Learning Notes

**Key Concepts Introduced**:

- **Async Orchestration**: Coordinating multiple async operations
- **Trait Objects**: Using `Box<dyn Trait>` for dynamic dispatch
- **Error Accumulation**: Collecting errors while continuing operation
- **Progress Callbacks**: Implementing observer pattern with traits
- **State Management**: Managing configuration and sync state

**Questions to Research**:

1. How does async orchestration work in Rust?
2. What's the difference between `&dyn Trait` and `Box<dyn Trait>`?
3. How do you handle partial failures in async operations?
4. What are the best practices for progress reporting in CLI applications?
