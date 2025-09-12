# Task 10: Documentation & Polish

**Estimated Time**: 2-3 hours  
**Difficulty**: ⭐⭐ Beginner-Intermediate  
**Prerequisites**: Tasks 01-09 completed

## Objective

Complete the implementation with comprehensive documentation, user guides, and final polish to ensure a great user experience and maintainable codebase.

## What You'll Learn

- Writing comprehensive documentation for Rust projects
- Creating user guides and tutorials
- Code documentation best practices
- Error message improvements
- Performance optimizations
- Release preparation

## Tasks

### 1. API Documentation

Add comprehensive rustdoc comments to all public APIs:

````rust
//! # DevLog Push Command
//!
//! This module implements the push functionality for DevLog, enabling users to
//! upload their local journal entries to remote cloud storage (currently Azure Blob Storage).
//!
//! ## Features
//!
//! - **Incremental Sync**: Only uploads changed files by default
//! - **Full Upload**: Option to upload all files regardless of changes
//! - **Progress Reporting**: Real-time progress indication with progress bars
//! - **Retry Logic**: Automatic retry with exponential backoff for transient failures
//! - **Dry Run Mode**: Preview what would be uploaded without making changes
//!
//! ## Usage
//!
//! ```bash
//! # Upload only changed files (default)
//! devlog push
//!
//! # Upload all files
//! devlog push --mode all
//!
//! # Preview what would be uploaded
//! devlog push --dry-run
//!
//! # Force upload even if no changes detected
//! devlog push --force
//! ```
//!
//! ## Configuration
//!
//! Before using the push command, configure your remote storage in `~/.devlog/config.toml`:
//!
//! ```toml
//! [remote]
//! provider = "azure"
//! url = "https://youraccount.blob.core.windows.net/yourcontainer"
//!
//! [sync]
//! # This section is managed automatically
//! ```
//!
//! Set your Azure storage account key as an environment variable:
//!
//! ```bash
//! export AZURE_STORAGE_ACCOUNT_KEY="your-account-key-here"
//! ```

use crate::config::DevLogConfig;
use crate::local::{FileScanner, LocalFile};
use crate::remote::{RemoteStorage, StorageFactory};
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Options for configuring push operations
///
/// This struct controls how the push operation behaves, including
/// what files to upload and whether to actually perform the upload.
///
/// # Examples
///
/// ```rust
/// use devlog::sync::{PushOptions, PushMode};
///
/// // Default incremental push
/// let options = PushOptions {
///     mode: PushMode::Incremental,
///     force: false,
///     dry_run: false,
/// };
///
/// // Dry run to preview changes
/// let preview_options = PushOptions {
///     mode: PushMode::Incremental,
///     force: false,
///     dry_run: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PushOptions {
    /// Upload mode: incremental (only changed files) or all files
    pub mode: PushMode,
    /// Force upload even if files haven't changed
    pub force: bool,
    /// Preview mode - show what would be uploaded without uploading
    pub dry_run: bool,
}

/// Determines which files to include in the push operation
///
/// The push mode affects which files are considered for upload:
///
/// - `Incremental`: Only files modified since the last push
/// - `All`: All files in the DevLog directory (subject to include/exclude patterns)
///
/// In both modes, files are still checked against remote storage to avoid
/// unnecessary uploads, unless the `force` option is used.
#[derive(Debug, Clone, PartialEq)]
pub enum PushMode {
    /// Upload only files that have changed since the last push timestamp
    ///
    /// This is the default mode and most efficient for regular use.
    /// Files are filtered by modification time and then checked against
    /// remote storage to ensure only truly changed files are uploaded.
    Incremental,

    /// Consider all files for upload, regardless of last push timestamp
    ///
    /// This mode considers all files in the DevLog directory but still
    /// performs change detection unless `force` is also specified.
    /// Useful for initial setup or recovering from sync issues.
    All,
}

/// Reports progress during push operations
///
/// This trait allows different implementations of progress reporting,
/// from simple console output to GUI progress bars or logging systems.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` as they may be called from
/// async contexts and shared between tasks.
///
/// # Examples
///
/// ```rust
/// use devlog::sync::ProgressReporter;
/// use std::path::Path;
///
/// struct SimpleReporter;
///
/// impl ProgressReporter for SimpleReporter {
///     fn report_scan_start(&self, base_path: &Path) {
///         println!("Scanning {:?}...", base_path);
///     }
///
///     fn report_scan_complete(&self, file_count: usize) {
///         println!("Found {} files", file_count);
///     }
///
///     // ... implement other methods
/// }
/// ```
#[async_trait]
pub trait ProgressReporter: Send + Sync {
    /// Called when file scanning begins
    ///
    /// # Parameters
    /// * `base_path` - The directory being scanned (typically ~/.devlog)
    fn report_scan_start(&self, base_path: &Path);

    /// Called when file scanning completes
    ///
    /// # Parameters
    /// * `file_count` - Total number of files discovered
    fn report_scan_complete(&self, file_count: usize);

    /// Called when change detection begins
    ///
    /// This involves comparing local files with remote storage
    /// to determine which files need to be uploaded.
    fn report_change_detection_start(&self);

    /// Called when change detection completes
    ///
    /// # Parameters
    /// * `changes` - Number of files that need to be uploaded
    fn report_change_detection_complete(&self, changes: usize);

    /// Called when upload operations begin
    ///
    /// # Parameters
    /// * `file_count` - Number of files to upload
    /// * `total_bytes` - Total size of all files to upload
    fn report_upload_start(&self, file_count: usize, total_bytes: u64);

    /// Called when an individual file upload begins
    ///
    /// # Parameters
    /// * `file_path` - Relative path of the file being uploaded
    /// * `size` - Size of the file in bytes
    fn report_file_upload_start(&self, file_path: &str, size: u64);

    /// Called when an individual file upload completes successfully
    ///
    /// # Parameters
    /// * `file_path` - Relative path of the uploaded file
    fn report_file_upload_complete(&self, file_path: &str);

    /// Called when an individual file upload fails
    ///
    /// # Parameters
    /// * `file_path` - Relative path of the file that failed
    /// * `error` - Error message describing the failure
    fn report_file_upload_error(&self, file_path: &str, error: &str);

    /// Called when all upload operations complete
    ///
    /// # Parameters
    /// * `result` - Summary of the push operation results
    fn report_upload_complete(&self, result: &PushResult);
}

/// Summary of a completed push operation
///
/// This struct contains metrics and status information about
/// a push operation, including success/failure counts and timing.
///
/// # Examples
///
/// ```rust
/// use devlog::sync::PushResult;
/// use std::time::Duration;
///
/// let result = PushResult {
///     files_uploaded: 5,
///     files_skipped: 2,
///     total_bytes: 1024,
///     duration: Duration::from_secs(10),
///     errors: vec![],
/// };
///
/// if result.errors.is_empty() {
///     println!("Successfully uploaded {} files", result.files_uploaded);
/// }
/// ```
#[derive(Debug)]
pub struct PushResult {
    /// Number of files successfully uploaded
    pub files_uploaded: usize,
    /// Number of files skipped (no changes detected)
    pub files_skipped: usize,
    /// Total bytes uploaded
    pub total_bytes: u64,
    /// Total time taken for the operation
    pub duration: std::time::Duration,
    /// List of errors encountered during the operation
    pub errors: Vec<SyncError>,
}

impl PushResult {
    /// Returns true if the push operation was completely successful
    ///
    /// A push is considered successful if no errors occurred,
    /// regardless of whether any files were actually uploaded.
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns the total number of files processed
    pub fn total_files(&self) -> usize {
        self.files_uploaded + self.files_skipped
    }

    /// Returns the upload rate in bytes per second
    ///
    /// Returns 0 if the operation took less than 1 millisecond.
    pub fn upload_rate(&self) -> f64 {
        if self.duration.as_millis() > 0 {
            self.total_bytes as f64 / self.duration.as_secs_f64()
        } else {
            0.0
        }
    }
}

/// The main synchronization manager
///
/// `SyncManager` coordinates all aspects of the push operation,
/// from scanning local files to uploading them to remote storage.
/// It handles configuration, progress reporting, and error recovery.
///
/// # Examples
///
/// ```rust,no_run
/// use devlog::sync::{SyncManager, PushOptions, PushMode, ConsoleProgressReporter};
///
/// # tokio_test::block_on(async {
/// let mut sync_manager = SyncManager::new().await?;
///
/// let options = PushOptions {
///     mode: PushMode::Incremental,
///     force: false,
///     dry_run: false,
/// };
///
/// let progress = ConsoleProgressReporter::new();
/// let result = sync_manager.push(options, Some(&progress)).await?;
///
/// if result.is_success() {
///     println!("Push completed successfully!");
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// # });
/// ```
pub struct SyncManager {
    config: DevLogConfig,
    remote_storage: Box<dyn RemoteStorage>,
    file_scanner: FileScanner,
}

impl SyncManager {
    /// Create a new sync manager with default configuration
    ///
    /// This loads configuration from `~/.devlog/config.toml` and
    /// creates the appropriate remote storage client.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration file cannot be loaded or is invalid
    /// - Remote storage client cannot be created
    /// - Required environment variables are missing
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use devlog::sync::SyncManager;
    ///
    /// # tokio_test::block_on(async {
    /// let sync_manager = SyncManager::new().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn new() -> Result<Self> {
        // Implementation details...
    }

    /// Create a sync manager with custom configuration
    ///
    /// This is primarily useful for testing or when you need
    /// to override the default configuration loading behavior.
    ///
    /// # Parameters
    /// * `config` - The DevLog configuration to use
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or
    /// the remote storage client cannot be created.
    pub async fn with_config(config: DevLogConfig) -> Result<Self> {
        // Implementation details...
    }

    /// Perform a push operation
    ///
    /// This is the main entry point for uploading files to remote storage.
    /// The operation can be customized using `PushOptions` and progress
    /// can be reported via the optional `ProgressReporter`.
    ///
    /// # Parameters
    /// * `options` - Configuration for the push operation
    /// * `progress` - Optional progress reporter for user feedback
    ///
    /// # Returns
    ///
    /// A `PushResult` containing metrics and status information.
    /// The operation is considered successful if `result.is_success()` returns true.
    ///
    /// # Errors
    ///
    /// Returns an error only for catastrophic failures that prevent
    /// the operation from proceeding (e.g., configuration errors).
    /// Individual file upload failures are reported in `PushResult.errors`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use devlog::sync::{SyncManager, PushOptions, PushMode};
    ///
    /// # tokio_test::block_on(async {
    /// let mut sync_manager = SyncManager::new().await?;
    ///
    /// let options = PushOptions {
    ///     mode: PushMode::Incremental,
    ///     force: false,
    ///     dry_run: true,  // Preview only
    /// };
    ///
    /// let result = sync_manager.push(options, None).await?;
    /// println!("Would upload {} files", result.files_uploaded + result.files_skipped);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn push(
        &mut self,
        options: PushOptions,
        progress: Option<&dyn ProgressReporter>,
    ) -> Result<PushResult> {
        // Implementation details...
    }
}
````

### 2. User Guide Documentation

Create `docs/user-guide.md`:

````markdown
# DevLog Push Command User Guide

## Overview

The `devlog push` command uploads your local journal entries to cloud storage, providing backup and synchronization capabilities. This guide covers setup, usage, and troubleshooting.

## Quick Start

### 1. Set Up Azure Storage

1. Create an Azure Storage Account in the [Azure Portal](https://portal.azure.com)
2. Create a blob container (e.g., "devlog")
3. Get your account access key from the Azure Portal

### 2. Configure DevLog

Create or edit `~/.devlog/config.toml`:

```toml
[remote]
provider = "azure"
url = "https://youraccount.blob.core.windows.net/yourcontainer"

[sync]
# This section is managed automatically
```
````

### 3. Set Environment Variable

```bash
# Linux/macOS
export AZURE_STORAGE_ACCOUNT_KEY="your-account-key-here"

# Windows
set AZURE_STORAGE_ACCOUNT_KEY=your-account-key-here
```

### 4. First Push

```bash
# Preview what would be uploaded
devlog push --dry-run

# Upload all files
devlog push --mode all
```

## Usage Examples

### Basic Operations

```bash
# Upload only changed files (default)
devlog push

# Upload all files
devlog push --mode all

# Preview changes without uploading
devlog push --dry-run

# Force upload even if no changes detected
devlog push --force
```

### Advanced Usage

```bash
# Combine options
devlog push --mode all --dry-run --force

# Upload with verbose output (if implemented)
devlog push --verbose
```

## Configuration Reference

### Remote Storage Settings

| Setting    | Description            | Example                                             |
| ---------- | ---------------------- | --------------------------------------------------- |
| `provider` | Cloud storage provider | `"azure"`                                           |
| `url`      | Storage container URL  | `"https://account.blob.core.windows.net/container"` |

### Sync Settings

| Setting               | Description                            | Managed By |
| --------------------- | -------------------------------------- | ---------- |
| `last_push_timestamp` | When the last successful push occurred | DevLog     |

### Environment Variables

| Variable                    | Description                      | Required |
| --------------------------- | -------------------------------- | -------- |
| `AZURE_STORAGE_ACCOUNT_KEY` | Azure storage account access key | Yes      |

## File Synchronization

### What Gets Uploaded

DevLog uploads these file types:

- **Event files**: `~/.devlog/events/*.jsonl`
- **Entry files**: `~/.devlog/entries/*.md`
- **Configuration**: `~/.devlog/config.toml`

### What Gets Excluded

These files are never uploaded:

- Temporary files (`*.tmp`, `*.lock`)
- Cache directory (`cache/*`)
- Hidden files (`.DS_Store`, etc.)

### Sync Modes

#### Incremental Mode (Default)

```bash
devlog push
```

- Only uploads files modified since the last push
- Most efficient for regular use
- Automatically set as default

#### All Mode

```bash
devlog push --mode all
```

- Considers all files for upload
- Still performs change detection unless `--force` is used
- Useful for initial setup or sync recovery

#### Force Mode

```bash
devlog push --force
```

- Uploads files even if they haven't changed
- Can be combined with either sync mode
- Useful for troubleshooting or ensuring complete sync

## Troubleshooting

### Common Issues

#### Authentication Errors

**Symptoms**: "Authentication failed" messages

**Solutions**:

1. Verify `AZURE_STORAGE_ACCOUNT_KEY` is set correctly
2. Check the account key in Azure Portal
3. Ensure the storage account name in URL matches

#### Network Errors

**Symptoms**: "Network error" or timeout messages

**Solutions**:

1. Check internet connectivity
2. Verify Azure storage URL is accessible
3. Check firewall settings for HTTPS traffic
4. Try again (temporary network issues)

#### Configuration Errors

**Symptoms**: "Configuration error" messages

**Solutions**:

1. Check `~/.devlog/config.toml` syntax
2. Verify required fields are present
3. Ensure URL format is correct

#### File Permission Errors

**Symptoms**: "Failed to read local file" messages

**Solutions**:

1. Check file permissions in `~/.devlog`
2. Ensure DevLog can read the files
3. Verify disk space availability

### Getting Help

1. **Dry Run**: Use `--dry-run` to diagnose issues
2. **Force Mode**: Try `--force` if sync seems stuck
3. **Check Status**: Verify Azure service status
4. **Documentation**: Review this guide and error messages
5. **Issues**: Report bugs on [GitHub](https://github.com/your-repo/devlog/issues)

### Recovery Procedures

#### Reset Sync State

If sync becomes corrupted:

```bash
# Edit ~/.devlog/config.toml and remove the [sync] section
# Then run a full push
devlog push --mode all --force
```

#### Verify Remote Files

Check what's uploaded to your Azure storage:

1. Use Azure Storage Explorer
2. Check the Azure Portal
3. Use Azure CLI tools

## Performance Tips

### Optimize Upload Speed

1. **Use Incremental Mode**: Only upload what's changed
2. **Stable Network**: Use wired connection for large uploads
3. **Batch Operations**: Let DevLog handle multiple files efficiently

### Monitor Usage

- Check Azure storage costs in the portal
- Monitor storage quota usage
- Review upload logs for patterns

## Security Considerations

### Access Keys

- Store account keys securely
- Rotate keys periodically
- Use environment variables, not config files

### Network Security

- DevLog uses HTTPS for all transfers
- Data is encrypted in transit
- Follow Azure security best practices

### Data Privacy

- Review what data gets uploaded
- Understand Azure data residency policies
- Consider data retention requirements

## Advanced Topics

### Multiple Devices

To sync between multiple devices:

1. Use the same Azure storage configuration
2. Run `devlog push` on each device
3. Implement pull functionality (future feature)

### Automation

To automate pushes:

```bash
# Add to cron (Linux/macOS)
0 */6 * * * /path/to/devlog push

# Add to Task Scheduler (Windows)
# Create a scheduled task to run devlog push
```

### Custom Providers

Future versions will support:

- AWS S3
- Google Cloud Storage
- Custom storage backends

## Appendix

### Azure Storage Setup Details

1. **Create Storage Account**:

   - Choose "StorageV2" account type
   - Select appropriate replication option
   - Note the account name

2. **Create Container**:

   - Container name must be lowercase
   - Set access level to "Private"
   - Note the container name

3. **Get Access Key**:
   - Go to "Access keys" in the Azure Portal
   - Copy either key1 or key2
   - Keep this secure

### URL Format Examples

```
# Correct format
https://myaccount.blob.core.windows.net/devlog

# Common mistakes
http://myaccount.blob.core.windows.net/devlog  # Not HTTPS
https://myaccount.blob.core.windows.net/       # Missing container
https://myaccount.blob.core.windows.net/devlog/  # Trailing slash (removed automatically)
```

### Environment Variable Setup

#### Linux/macOS (Persistent)

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export AZURE_STORAGE_ACCOUNT_KEY="your-key-here"
```

#### Windows (Persistent)

```cmd
setx AZURE_STORAGE_ACCOUNT_KEY "your-key-here"
```

#### Docker/Container

```bash
docker run -e AZURE_STORAGE_ACCOUNT_KEY="your-key" your-devlog-image
```

````

### 3. Code Examples and Tutorials

Create `docs/examples.md`:

```markdown
# DevLog Push Command Examples

This document provides practical examples of using the `devlog push` command in various scenarios.

## Basic Usage Scenarios

### First-Time Setup

```bash
# 1. Set up configuration
mkdir -p ~/.devlog
cat > ~/.devlog/config.toml << EOF
[remote]
provider = "azure"
url = "https://myaccount.blob.core.windows.net/devlog"

[sync]
EOF

# 2. Set environment variable
export AZURE_STORAGE_ACCOUNT_KEY="your-account-key"

# 3. Create some test content
echo '{"event": "setup", "timestamp": "'$(date -Iseconds)'"}' > ~/.devlog/events/$(date +%Y%m%d).jsonl

# 4. Test with dry run
devlog push --dry-run

# 5. Perform first upload
devlog push --mode all
````

### Daily Usage

```bash
# After creating new entries, upload changes
devlog new -m "Completed project milestone"
devlog push  # Only uploads new/changed files
```

### Weekly Backup

```bash
# Ensure everything is backed up
devlog push --mode all --force
```

## Advanced Scenarios

### Recovering from Sync Issues

```bash
# If sync state seems corrupted, reset and re-upload everything
# First, backup your current config
cp ~/.devlog/config.toml ~/.devlog/config.toml.backup

# Edit config to remove [sync] section
sed -i '/\[sync\]/,$d' ~/.devlog/config.toml

# Force upload all files
devlog push --mode all --force
```

### Testing Configuration

```bash
# Test without making changes
devlog push --dry-run

# Test with different modes
devlog push --mode incremental --dry-run
devlog push --mode all --dry-run
devlog push --force --dry-run
```

### Automation Scripts

#### Simple Backup Script

```bash
#!/bin/bash
# backup-devlog.sh

set -e

# Check if DevLog is available
if ! command -v devlog &> /dev/null; then
    echo "Error: devlog command not found"
    exit 1
fi

# Check if Azure key is set
if [ -z "$AZURE_STORAGE_ACCOUNT_KEY" ]; then
    echo "Error: AZURE_STORAGE_ACCOUNT_KEY not set"
    exit 1
fi

# Perform backup
echo "Starting DevLog backup..."
if devlog push; then
    echo "Backup completed successfully"
else
    echo "Backup failed"
    exit 1
fi
```

#### Scheduled Backup with Cron

```bash
# Add to crontab (run every 6 hours)
0 */6 * * * /home/user/scripts/backup-devlog.sh >> /var/log/devlog-backup.log 2>&1
```

#### Windows Scheduled Task

```powershell
# PowerShell script for Windows
$env:AZURE_STORAGE_ACCOUNT_KEY = "your-key-here"

try {
    devlog push
    Write-Host "DevLog backup completed successfully"
} catch {
    Write-Error "DevLog backup failed: $_"
    exit 1
}
```

## Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/backup-devlog.yml
name: Backup DevLog
on:
  schedule:
    - cron: "0 12 * * *" # Daily at noon
  workflow_dispatch:

jobs:
  backup:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install DevLog
        run: cargo install devlog
      - name: Setup DevLog Config
        run: |
          mkdir -p ~/.devlog
          echo '[remote]' > ~/.devlog/config.toml
          echo 'provider = "azure"' >> ~/.devlog/config.toml
          echo 'url = "${{ secrets.AZURE_STORAGE_URL }}"' >> ~/.devlog/config.toml
        env:
          AZURE_STORAGE_ACCOUNT_KEY: ${{ secrets.AZURE_STORAGE_KEY }}
      - name: Backup DevLog
        run: devlog push --mode all
```

### Docker Integration

```dockerfile
# Dockerfile for automated DevLog backup
FROM rust:1.70

# Install DevLog
RUN cargo install devlog

# Copy configuration
COPY config.toml /root/.devlog/config.toml

# Copy entry point script
COPY backup.sh /usr/local/bin/backup.sh
RUN chmod +x /usr/local/bin/backup.sh

# Set up cron job
RUN echo "0 */6 * * * /usr/local/bin/backup.sh" | crontab -

CMD ["cron", "-f"]
```

## Error Handling Examples

### Retry Logic

```bash
#!/bin/bash
# retry-push.sh - Retry push with exponential backoff

MAX_ATTEMPTS=3
DELAY=1

for i in $(seq 1 $MAX_ATTEMPTS); do
    echo "Attempt $i/$MAX_ATTEMPTS..."

    if devlog push; then
        echo "Push successful!"
        exit 0
    else
        if [ $i -lt $MAX_ATTEMPTS ]; then
            echo "Push failed, retrying in ${DELAY}s..."
            sleep $DELAY
            DELAY=$((DELAY * 2))
        fi
    fi
done

echo "Push failed after $MAX_ATTEMPTS attempts"
exit 1
```

### Health Check Script

```bash
#!/bin/bash
# health-check.sh - Verify DevLog setup

echo "DevLog Health Check"
echo "==================="

# Check DevLog installation
if command -v devlog &> /dev/null; then
    echo "✓ DevLog is installed"
else
    echo "✗ DevLog is not installed"
    exit 1
fi

# Check configuration
if [ -f ~/.devlog/config.toml ]; then
    echo "✓ Configuration file exists"
else
    echo "✗ Configuration file missing"
    exit 1
fi

# Check environment variable
if [ -n "$AZURE_STORAGE_ACCOUNT_KEY" ]; then
    echo "✓ Azure storage key is set"
else
    echo "✗ Azure storage key is not set"
    exit 1
fi

# Test with dry run
if devlog push --dry-run &> /dev/null; then
    echo "✓ Dry run successful"
else
    echo "✗ Dry run failed"
    exit 1
fi

echo "✓ All checks passed!"
```

## Performance Testing

### Benchmark Script

```bash
#!/bin/bash
# benchmark-push.sh - Test push performance with different file counts

create_test_files() {
    local count=$1
    local base_dir=~/.devlog-test

    rm -rf "$base_dir"
    mkdir -p "$base_dir/events"

    for i in $(seq 1 $count); do
        echo "{\"event\": \"test_$i\", \"timestamp\": \"$(date -Iseconds)\"}" > "$base_dir/events/test_$i.jsonl"
    done

    # Copy config
    cp ~/.devlog/config.toml "$base_dir/"
}

benchmark_push() {
    local file_count=$1
    echo "Benchmarking with $file_count files..."

    create_test_files $file_count

    # Time the push operation
    start_time=$(date +%s.%N)

    # Use a test configuration that points to the test directory
    DEVLOG_CONFIG_DIR=~/.devlog-test devlog push --mode all

    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)

    echo "Time for $file_count files: ${duration}s"

    # Cleanup
    rm -rf ~/.devlog-test
}

# Run benchmarks
for count in 10 50 100 500 1000; do
    benchmark_push $count
    sleep 2
done
```

## Best Practices

### Configuration Management

```bash
# Use version control for configuration templates
git init ~/.devlog-config
cd ~/.devlog-config

# Create template
cat > config.template.toml << EOF
[remote]
provider = "azure"
url = "REPLACE_WITH_YOUR_URL"

[sync]
# Managed automatically
EOF

# Create environment-specific configs
sed 's/REPLACE_WITH_YOUR_URL/https:\/\/prod.blob.core.windows.net\/devlog/' config.template.toml > prod.config.toml
sed 's/REPLACE_WITH_YOUR_URL/https:\/\/test.blob.core.windows.net\/devlog/' config.template.toml > test.config.toml

git add . && git commit -m "Add DevLog config templates"
```

### Multiple Environment Setup

```bash
# Production environment
export DEVLOG_ENV=prod
export AZURE_STORAGE_ACCOUNT_KEY="$PROD_AZURE_KEY"
cp ~/.devlog-config/prod.config.toml ~/.devlog/config.toml
devlog push

# Test environment
export DEVLOG_ENV=test
export AZURE_STORAGE_ACCOUNT_KEY="$TEST_AZURE_KEY"
cp ~/.devlog-config/test.config.toml ~/.devlog/config.toml
devlog push --dry-run  # Always dry-run in test first
```

### Monitoring and Alerting

```bash
# monitoring.sh - Check if push is working and alert if not
#!/bin/bash

WEBHOOK_URL="https://hooks.slack.com/your-webhook-url"

if ! devlog push --dry-run &> /dev/null; then
    # Send alert
    curl -X POST -H 'Content-type: application/json' \
        --data '{"text":"DevLog push health check failed!"}' \
        "$WEBHOOK_URL"

    exit 1
fi

echo "DevLog push health check passed"
```

````

### 4. Performance Optimizations

Add final performance improvements:

```rust
//! Performance optimizations for DevLog push operations

use std::sync::Arc;
use tokio::sync::Semaphore;

/// Configuration for performance tuning
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum number of concurrent uploads
    pub max_concurrent_uploads: usize,
    /// Buffer size for file reading (in bytes)
    pub file_buffer_size: usize,
    /// Enable parallel file hashing
    pub parallel_hashing: bool,
    /// Batch size for remote operations
    pub batch_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_uploads: 4,
            file_buffer_size: 8192,
            parallel_hashing: true,
            batch_size: 10,
        }
    }
}

impl SyncManager {
    /// Upload files with optimized concurrency
    async fn upload_files_optimized(
        &self,
        files_to_upload: Vec<LocalFile>,
        progress: Option<&dyn ProgressReporter>,
        config: &PerformanceConfig,
    ) -> PushResult {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_uploads));
        let mut tasks = Vec::new();

        for file in files_to_upload {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let storage = self.remote_storage.as_ref();
            let file_clone = file.clone();

            let task = tokio::spawn(async move {
                let _permit = permit; // Hold permit until task completes
                storage.upload_file(&file_clone.path, &file_clone.remote_key()).await
            });

            tasks.push((file, task));
        }

        // Process results as they complete
        let mut result = PushResult::default();
        for (file, task) in tasks {
            match task.await {
                Ok(Ok(())) => {
                    result.files_uploaded += 1;
                    result.total_bytes += file.size;

                    if let Some(reporter) = progress {
                        reporter.report_file_upload_complete(&file.relative_path);
                    }
                }
                Ok(Err(e)) | Err(e) => {
                    result.errors.push(SyncError::UploadFailed {
                        path: file.relative_path.clone(),
                        message: e.to_string(),
                    });

                    if let Some(reporter) = progress {
                        reporter.report_file_upload_error(&file.relative_path, &e.to_string());
                    }
                }
            }
        }

        result
    }
}
````

### 5. Final Error Message Polish

Enhance error messages with more context:

```rust
//! Enhanced error messages with actionable guidance

impl ErrorFormatter {
    /// Format error with environment-specific guidance
    pub fn format_error_with_context(
        error: &dyn std::error::Error,
        operation: &str,
        context: &ErrorContext,
    ) -> String {
        let base_message = format!("❌ Failed to {}: {}", operation, error);
        let guidance = Self::get_contextual_guidance(error, context);

        format!("{}\n\n{}", base_message, guidance)
    }

    fn get_contextual_guidance(
        error: &dyn std::error::Error,
        context: &ErrorContext,
    ) -> String {
        // Provide guidance based on user's environment and error type
        match context.platform {
            Platform::Windows => Self::windows_specific_guidance(error),
            Platform::MacOS => Self::macos_specific_guidance(error),
            Platform::Linux => Self::linux_specific_guidance(error),
        }
    }
}

#[derive(Debug)]
pub struct ErrorContext {
    pub platform: Platform,
    pub is_first_time_user: bool,
    pub has_config_file: bool,
    pub azure_key_set: bool,
}

#[derive(Debug)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}
```

## Validation Steps

### 1. Documentation Quality Check

```bash
# Generate and review documentation
cargo doc --open

# Check for broken links
cargo doc --no-deps

# Verify examples compile
cargo test --doc
```

### 2. User Experience Testing

```bash
# Test the complete new user flow
rm -rf ~/.devlog
unset AZURE_STORAGE_ACCOUNT_KEY

# Follow the user guide step by step
# Verify error messages are helpful
# Check that examples work as described
```

### 3. Performance Validation

```bash
# Test with various file counts
for count in 1 10 100 1000; do
    echo "Testing with $count files"
    # Create test files and measure push time
done
```

## Expected Outputs

After completing this task:

- ✅ Comprehensive API documentation with examples
- ✅ User guide covering all scenarios
- ✅ Code examples and tutorials
- ✅ Performance optimizations implemented
- ✅ Error messages are helpful and actionable
- ✅ Ready for production use

## Final Checklist

- [ ] All public APIs have rustdoc comments
- [ ] User guide is complete and tested
- [ ] Examples cover common use cases
- [ ] Error messages provide clear guidance
- [ ] Performance is acceptable for target use cases
- [ ] Code is well-organized and maintainable
- [ ] Tests pass consistently
- [ ] Documentation is up to date

## Rust Learning Notes

**Key Concepts Introduced**:

- **Documentation**: Writing effective rustdoc comments
- **API Design**: Creating user-friendly public interfaces
- **Performance Tuning**: Optimizing async operations
- **Error UX**: Designing helpful error messages
- **Code Organization**: Structuring larger Rust projects

Congratulations! You've successfully implemented a complete push command feature for DevLog with proper documentation, testing, and user experience considerations.
