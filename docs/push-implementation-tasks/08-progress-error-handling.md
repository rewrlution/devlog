# Task 08: Progress & Error Handling Enhancement

**Estimated Time**: 2-3 hours  
**Difficulty**: ⭐⭐ Beginner-Intermediate  
**Prerequisites**: Tasks 01-07 completed

## Objective

Enhance the user experience with better progress indication, retry mechanisms, and comprehensive error handling for network failures and edge cases.

## What You'll Learn

- Advanced progress reporting with progress bars
- Retry mechanisms with exponential backoff
- Comprehensive error categorization and recovery
- User-friendly error messages and suggestions
- Graceful handling of network interruptions

## Tasks

### 1. Enhanced Progress Reporter

Improve the progress reporting with visual progress bars and better formatting:

```rust
//! Enhanced progress reporting for DevLog sync operations

use crate::sync::{ProgressReporter, PushResult};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Enhanced console progress reporter with progress bars
pub struct EnhancedProgressReporter {
    state: Arc<Mutex<ProgressState>>,
}

#[derive(Debug)]
struct ProgressState {
    current_file: usize,
    total_files: usize,
    current_bytes: u64,
    total_bytes: u64,
    start_time: Option<Instant>,
    last_update: Option<Instant>,
}

impl EnhancedProgressReporter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ProgressState {
                current_file: 0,
                total_files: 0,
                current_bytes: 0,
                total_bytes: 0,
                start_time: None,
                last_update: None,
            })),
        }
    }

    fn draw_progress_bar(&self, percentage: f64, width: usize) -> String {
        let filled = ((percentage / 100.0) * width as f64) as usize;
        let empty = width - filled;

        format!(
            "[{}{}] {:.1}%",
            "█".repeat(filled),
            "░".repeat(empty),
            percentage
        )
    }

    fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    fn format_duration(&self, duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    fn estimate_remaining_time(&self, state: &ProgressState) -> Option<Duration> {
        if let Some(start_time) = state.start_time {
            if state.current_bytes > 0 && state.total_bytes > 0 {
                let elapsed = start_time.elapsed();
                let progress_ratio = state.current_bytes as f64 / state.total_bytes as f64;

                if progress_ratio > 0.0 {
                    let estimated_total = elapsed.as_secs_f64() / progress_ratio;
                    let remaining = estimated_total - elapsed.as_secs_f64();

                    if remaining > 0.0 {
                        return Some(Duration::from_secs_f64(remaining));
                    }
                }
            }
        }
        None
    }

    fn update_progress_line(&self) {
        let state = self.state.lock().unwrap();

        if state.total_files == 0 {
            return;
        }

        let file_percentage = if state.total_files > 0 {
            (state.current_file as f64 / state.total_files as f64) * 100.0
        } else {
            0.0
        };

        let byte_percentage = if state.total_bytes > 0 {
            (state.current_bytes as f64 / state.total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let progress_bar = self.draw_progress_bar(byte_percentage, 30);

        let mut status_line = format!(
            "\r{} {}/{} files {} ",
            progress_bar,
            state.current_file,
            state.total_files,
            self.format_bytes(state.total_bytes)
        );

        if let Some(eta) = self.estimate_remaining_time(&state) {
            status_line.push_str(&format!("ETA: {}", self.format_duration(eta)));
        }

        // Pad to clear previous line
        status_line.push_str(&" ".repeat(20));

        print!("{}", status_line);
        io::stdout().flush().unwrap();
    }
}

impl ProgressReporter for EnhancedProgressReporter {
    fn report_scan_start(&self, base_path: &Path) {
        println!("🔍 Scanning files in {:?}...", base_path);
    }

    fn report_scan_complete(&self, file_count: usize) {
        println!("📁 Found {} files to consider", file_count);
    }

    fn report_change_detection_start(&self) {
        print!("🔄 Detecting changes... ");
        io::stdout().flush().unwrap();
    }

    fn report_change_detection_complete(&self, changes: usize) {
        if changes == 0 {
            println!("✨ No changes detected");
        } else {
            println!("📝 {} file(s) need to be uploaded", changes);
        }
    }

    fn report_upload_start(&self, file_count: usize, total_bytes: u64) {
        let mut state = self.state.lock().unwrap();
        state.total_files = file_count;
        state.total_bytes = total_bytes;
        state.current_file = 0;
        state.current_bytes = 0;
        state.start_time = Some(Instant::now());

        println!("⬆️  Uploading {} file(s) ({})...", file_count, self.format_bytes(total_bytes));
        self.update_progress_line();
    }

    fn report_file_upload_start(&self, _file_path: &str, _size: u64) {
        // Update is handled in report_file_upload_complete for atomic updates
    }

    fn report_file_upload_complete(&self, file_path: &str) {
        let mut state = self.state.lock().unwrap();
        state.current_file += 1;

        // For simplicity, we'll estimate bytes uploaded based on file completion
        if state.total_files > 0 {
            state.current_bytes = (state.current_file as f64 / state.total_files as f64 * state.total_bytes as f64) as u64;
        }

        drop(state);
        self.update_progress_line();

        // Show individual file completion occasionally
        if file_path.len() < 50 {
            println!("\n  ✅ {}", file_path);
        } else {
            println!("\n  ✅ ...{}", &file_path[file_path.len() - 45..]);
        }

        self.update_progress_line();
    }

    fn report_file_upload_error(&self, file_path: &str, error: &str) {
        println!("\n  ❌ {}: {}", file_path, error);
        self.update_progress_line();
    }

    fn report_upload_complete(&self, result: &PushResult) {
        // Clear progress line
        print!("\r{}\r", " ".repeat(80));

        println!();
        if result.files_uploaded > 0 {
            println!("✅ Upload complete!");
            println!("   Files uploaded: {}", result.files_uploaded);
            if result.files_skipped > 0 {
                println!("   Files skipped: {}", result.files_skipped);
            }
            println!("   Total size: {}", self.format_bytes(result.total_bytes));
            println!("   Duration: {}", self.format_duration(result.duration));
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

### 2. Retry Mechanism with Exponential Backoff

Add retry logic to the sync manager for handling transient failures:

```rust
//! Retry mechanisms for robust sync operations

use std::time::Duration;
use tokio::time::sleep;
use anyhow::Result;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(error);

                if attempt < config.max_attempts {
                    eprintln!(
                        "⚠️  {} failed (attempt {}/{}): {}. Retrying in {}...",
                        operation_name,
                        attempt,
                        config.max_attempts,
                        last_error.as_ref().unwrap(),
                        format_duration(delay)
                    );

                    sleep(delay).await;

                    // Exponential backoff with jitter
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * config.backoff_multiplier) as u64
                        ),
                        config.max_delay,
                    );
                }
            }
        }
    }

    Err(last_error.unwrap())
}

fn format_duration(duration: Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{:.1}s", duration.as_secs_f64())
    } else {
        format!("{}ms", duration.as_millis())
    }
}

/// Check if an error is retryable
pub fn is_retryable_error(error: &crate::remote::StorageError) -> bool {
    match error {
        crate::remote::StorageError::NetworkError { .. } => true,
        crate::remote::StorageError::ProviderError { message } => {
            // Some Azure errors are retryable
            message.contains("timeout") ||
            message.contains("503") ||
            message.contains("502") ||
            message.contains("temporarily unavailable")
        }
        crate::remote::StorageError::AuthenticationFailed { .. } => false,
        crate::remote::StorageError::InvalidRemoteKey { .. } => false,
        _ => false,
    }
}
```

### 3. Enhanced Error Messages and User Guidance

Create a comprehensive error handling module:

```rust
//! Enhanced error handling and user guidance

use crate::remote::StorageError;
use crate::sync::SyncError;

/// Enhanced error formatter with user guidance
pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format a storage error with helpful suggestions
    pub fn format_storage_error(error: &StorageError) -> String {
        match error {
            StorageError::AuthenticationFailed { message } => {
                format!(
                    "❌ Authentication failed: {}\n\n\
                    💡 Troubleshooting steps:\n\
                    1. Check that AZURE_STORAGE_ACCOUNT_KEY environment variable is set\n\
                    2. Verify the account key is correct (check Azure portal)\n\
                    3. Ensure the storage account name in your URL matches your actual account\n\
                    4. Confirm the container exists and you have write permissions\n\n\
                    Example: export AZURE_STORAGE_ACCOUNT_KEY=\"your-account-key-here\"",
                    message
                )
            }
            StorageError::NetworkError { message } => {
                format!(
                    "❌ Network error: {}\n\n\
                    💡 Troubleshooting steps:\n\
                    1. Check your internet connection\n\
                    2. Verify the Azure storage URL is accessible\n\
                    3. Check if your firewall allows HTTPS connections\n\
                    4. Try again - this might be a temporary network issue",
                    message
                )
            }
            StorageError::FileNotFound { path } => {
                format!(
                    "❌ File not found: {}\n\n\
                    💡 This might happen if:\n\
                    1. The file was deleted from remote storage\n\
                    2. The file path changed\n\
                    3. There's a sync issue between local and remote state",
                    path
                )
            }
            StorageError::UploadFailed { path, message } => {
                format!(
                    "❌ Upload failed for {}: {}\n\n\
                    💡 Possible solutions:\n\
                    1. Check file permissions (ensure you can read the local file)\n\
                    2. Verify available storage space in your Azure account\n\
                    3. Check if the file is locked or in use by another process\n\
                    4. Try uploading a smaller file first to test connectivity",
                    path, message
                )
            }
            StorageError::DownloadFailed { path, message } => {
                format!(
                    "❌ Download failed for {}: {}\n\n\
                    💡 Possible solutions:\n\
                    1. Check local file permissions (ensure you can write to the destination)\n\
                    2. Verify available disk space\n\
                    3. Check if the destination directory exists\n\
                    4. Try downloading to a different location",
                    path, message
                )
            }
            StorageError::InvalidRemoteKey { key } => {
                format!(
                    "❌ Invalid remote key: {}\n\n\
                    💡 Remote keys should:\n\
                    1. Not contain '..' (parent directory references)\n\
                    2. Not start with '/' (absolute paths)\n\
                    3. Use forward slashes for directory separators\n\
                    4. Be valid file names in Azure Blob Storage",
                    key
                )
            }
            StorageError::ConfigurationError { message } => {
                format!(
                    "❌ Configuration error: {}\n\n\
                    💡 Check your ~/.devlog/config.toml file:\n\
                    1. Ensure the [remote] section exists\n\
                    2. Verify the provider is set to \"azure\"\n\
                    3. Check the URL format: https://account.blob.core.windows.net/container\n\
                    4. Make sure the file is valid TOML format",
                    message
                )
            }
            StorageError::ProviderError { message } => {
                format!(
                    "❌ Azure storage error: {}\n\n\
                    💡 This is an Azure-specific error. Check:\n\
                    1. Azure service status (status.azure.com)\n\
                    2. Your storage account status in Azure portal\n\
                    3. Container permissions and access policies\n\
                    4. Account quotas and billing status",
                    message
                )
            }
        }
    }

    /// Format a sync error with guidance
    pub fn format_sync_error(error: &SyncError) -> String {
        match error {
            SyncError::UploadFailed { path, message } => {
                format!(
                    "❌ Failed to upload {}: {}\n\n\
                    💡 You can:\n\
                    1. Try running the command again\n\
                    2. Use --force to retry all files\n\
                    3. Check the specific error message above for details",
                    path, message
                )
            }
            SyncError::ScanFailed { message } => {
                format!(
                    "❌ Failed to scan local files: {}\n\n\
                    💡 Check:\n\
                    1. Permissions on ~/.devlog directory\n\
                    2. Available disk space\n\
                    3. Whether ~/.devlog directory exists",
                    message
                )
            }
            SyncError::ChangeDetectionFailed { message } => {
                format!(
                    "❌ Failed to detect changes: {}\n\n\
                    💡 This might be due to:\n\
                    1. Network connectivity issues\n\
                    2. Remote storage authentication problems\n\
                    3. Try using --mode all to skip change detection",
                    message
                )
            }
            SyncError::ConfigurationError { message } => {
                format!(
                    "❌ Configuration error: {}\n\n\
                    💡 Check your ~/.devlog/config.toml file and ensure:\n\
                    1. It has proper TOML syntax\n\
                    2. Required fields are present\n\
                    3. Values are valid for your Azure storage account",
                    message
                )
            }
        }
    }

    /// Provide general troubleshooting tips
    pub fn general_troubleshooting_tips() -> &'static str {
        "\n🔧 General troubleshooting tips:\n\
        1. Run 'devlog push --dry-run' to test without making changes\n\
        2. Use --mode all to upload all files (skips change detection)\n\
        3. Check your internet connection and Azure account status\n\
        4. Verify your configuration with a simple test file first\n\
        5. Check the DevLog GitHub issues for known problems\n\n\
        📖 For more help, visit: https://github.com/your-repo/devlog/docs"
    }
}
```

### 4. Update Sync Manager with Retry Logic

Modify the sync manager to use retry mechanisms:

```rust
// Add this to your sync manager implementation

impl SyncManager {
    /// Upload a single file with retry logic
    async fn upload_file_with_retry(
        &self,
        file: &LocalFile,
        retry_config: &RetryConfig,
    ) -> Result<(), crate::remote::StorageError> {
        use crate::sync::retry::{retry_with_backoff, is_retryable_error};

        let operation = || async {
            self.remote_storage.upload_file(&file.path, &file.remote_key()).await
        };

        match retry_with_backoff(
            operation,
            retry_config,
            &format!("Upload {}", file.relative_path)
        ).await {
            Ok(result) => Ok(result),
            Err(error) => {
                // Convert anyhow error back to StorageError if needed
                Err(error)
            }
        }
    }

    /// Upload files with enhanced error handling and retry logic
    async fn upload_files_with_retry(
        &self,
        files_to_upload: Vec<LocalFile>,
        progress: Option<&dyn ProgressReporter>,
        mut result: PushResult,
    ) -> PushResult {
        let retry_config = RetryConfig::default();
        let total_bytes: u64 = files_to_upload.iter().map(|f| f.size).sum();

        if let Some(reporter) = progress {
            reporter.report_upload_start(files_to_upload.len(), total_bytes);
        }

        for file in files_to_upload {
            if let Some(reporter) = progress {
                reporter.report_file_upload_start(&file.relative_path, file.size);
            }

            match self.upload_file_with_retry(&file, &retry_config).await {
                Ok(()) => {
                    result.files_uploaded += 1;
                    result.total_bytes += file.size;

                    if let Some(reporter) = progress {
                        reporter.report_file_upload_complete(&file.relative_path);
                    }
                }
                Err(e) => {
                    let error_msg = crate::sync::errors::ErrorFormatter::format_storage_error(&e);
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

### 5. Update CLI Error Handling

Enhance the CLI to show better error messages:

```rust
/// Handle the push command with enhanced error handling
async fn handle_push_command(
    mode: PushMode,
    force: bool,
    dry_run: bool
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::sync::{SyncManager, PushOptions, EnhancedProgressReporter};
    use crate::sync::errors::ErrorFormatter;

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

    // Create sync manager with enhanced error handling
    let mut sync_manager = match SyncManager::new().await {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("{}", ErrorFormatter::format_configuration_error(&e));
            eprintln!("{}", ErrorFormatter::general_troubleshooting_tips());
            std::process::exit(1);
        }
    };

    // Create enhanced progress reporter
    let progress_reporter = EnhancedProgressReporter::new();

    // Perform the push with comprehensive error handling
    let result = match sync_manager.push(options, Some(&progress_reporter)).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("\n❌ Push operation failed: {}", e);
            eprintln!("{}", ErrorFormatter::general_troubleshooting_tips());
            std::process::exit(1);
        }
    };

    // Exit with error code if there were upload failures
    if !result.errors.is_empty() {
        eprintln!("\n❌ Some files failed to upload. You can:");
        eprintln!("   • Run the command again to retry failed uploads");
        eprintln!("   • Use --force to attempt uploading all files again");
        eprintln!("   • Check your network connection and try again");
        std::process::exit(1);
    }

    Ok(())
}
```

## Validation Steps

### 1. Test Progress Reporting

```bash
# Create multiple test files to see progress bars
for i in {1..20}; do
    echo "Test file $i content" > ~/.devlog/events/test$i.jsonl
done

# Test progress reporting
cargo build
./target/debug/devlog push --mode all
```

### 2. Test Retry Mechanism

```bash
# Test with invalid credentials to see retry behavior
unset AZURE_STORAGE_ACCOUNT_KEY
./target/debug/devlog push

# Test with correct credentials
export AZURE_STORAGE_ACCOUNT_KEY="your_key"
./target/debug/devlog push
```

### 3. Test Error Messages

```bash
# Test various error conditions
./target/debug/devlog push  # without config
./target/debug/devlog push  # with invalid config
./target/debug/devlog push  # with network issues
```

## Expected Outputs

After completing this task:

- ✅ Progress bars show upload progress visually
- ✅ Retry mechanism handles transient network failures
- ✅ Error messages provide clear guidance for resolution
- ✅ Upload operations are more robust and user-friendly
- ✅ Users get helpful suggestions when things go wrong

## Next Steps

Once this task is complete, proceed to **Task 09: Testing & Validation** where we'll create comprehensive tests for the entire push system.

## Rust Learning Notes

**Key Concepts Introduced**:

- **Progress Reporting**: Real-time UI updates in console applications
- **Retry Logic**: Exponential backoff for resilient operations
- **Error Formatting**: User-friendly error presentation
- **Mutex and Arc**: Thread-safe state management
- **Duration and Time**: Working with time-based calculations
