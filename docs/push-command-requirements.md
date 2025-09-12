# Push Command Functional Requirements

## Overview

The `push` command enables uploading local DevLog data to remote cloud storage, providing backup capabilities and enabling future synchronization features.
This document outlines the MVP implementation that is simple enough while maintaining extensibility for future enhancements.

## 1. Core Architecture Design

### Cloud-Agnostic Storage Interface

To support multiple cloud providers and enable open-source contributions, we implement a trait-based architecture:

```rust
/// Generic interface for remote storage providers
pub trait RemoteStorage {
    fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn download_file(&self, remote_key: &str, local_path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn list_files(&self, prefix: &str) -> Result<Vec<String>, Box<dyn std::error::Error>>;
    fn file_exists(&self, remote_key: &str) -> Result<bool, Box<dyn std::error::Error>>;
    fn get_file_hash(&self, remote_key: &str) -> Result<Option<String>, Box<dyn std::error::Error>>;
}
```

### Provider Implementations

**MVP Focus**: Azure Blob Storage
**Future Roadmap**: AWS S3, Google Cloud Storage, custom implementations

```rust
// src/remote/azure.rs
pub struct AzureStorage {
    container_url: String,
    account_key: Option<String>,
}

impl RemoteStorage for AzureStorage {
    // Azure-specific implementation using REST API
}
```

## 2. Configuration System

### Local Configuration File: `~/.devlog/config.toml`

Simple TOML-based configuration that can be extended in the future:

```toml
[remote]
provider = "azure"  # Extensible: "aws", "gcp", "custom"
url = "https://myaccount.blob.core.windows.net/devlog"

[sync]
last_push_timestamp = "2025-09-11T10:30:00Z"

# Future extensions:
# [preferences]
# preferred_language = "en"
# sync_interval = "daily"
#
# [auth]
# method = "key"  # or "oauth", "service_principal"
```

### Configuration Management

```rust
// src/config.rs
#[derive(Debug, Deserialize, Serialize)]
pub struct DevLogConfig {
    pub remote: RemoteConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemoteConfig {
    pub provider: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SyncConfig {
    pub last_push_timestamp: Option<DateTime<Utc>>,
}
```

## 3. Push Command Interface

### Command Structure

```rust
#[derive(Debug, Clone)]
pub enum PushMode {
    Incremental,
    All,
}

impl std::str::FromStr for PushMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "incremental" => Ok(PushMode::Incremental),
            "all" => Ok(PushMode::All),
            _ => Err(format!("Invalid push mode: '{}'. Valid options: 'incremental', 'all'", s)),
        }
    }
}

/// Push local changes to remote storage
Push {
    /// Push mode: 'incremental' (default) or 'all'
    #[arg(long, default_value = "incremental")]
    mode: PushMode,
},
```

### Usage Examples

```bash
# Default: incremental push (only new/modified files)
devlog push

# Explicit incremental mode
devlog push --mode incremental

# Upload all files
devlog push --mode all
```

## 4. Incremental Sync Strategy

### Change Detection Logic

The push command implements intelligent incremental synchronization:

1. **File System Scan**: Identify all files in `~/.devlog/`
2. **Hash Comparison**: Calculate SHA-256 hashes for local files
3. **Remote Metadata Check**: Compare with remote file hashes
4. **Change Set Determination**: Upload only modified files
5. **Metadata Update**: Record successful uploads with timestamps

### File Types Synchronized

- **Entry Events**: `~/.devlog/events/*.jsonl` (event sourcing data)
- **Entry Markdown**: `~/.devlog/entries/*.md` (generated markdown files)
- **Configuration**: `~/.devlog/config.toml` (user configuration)
- **Future**: Analytics data, search indices, etc.

### Sync Manager Implementation

```rust
// src/sync.rs
pub struct SyncManager {
    local_storage: Box<dyn EntryStorage>,
    remote_storage: Box<dyn RemoteStorage>,
    config: DevLogConfig,
}

impl SyncManager {
    pub fn push(&self, options: PushOptions) -> Result<PushResult, Box<dyn std::error::Error>> {
        let local_files = self.scan_local_files()?;

        let changes = match options.mode {
            PushMode::All => local_files,
            PushMode::Incremental => self.detect_changes(&local_files)?,
        };

        let result = self.upload_changes(&changes)?;
        self.update_sync_metadata()?;

        Ok(result)
    }

    fn detect_changes(&self, local_files: &[LocalFile]) -> Result<Vec<LocalFile>, Box<dyn std::error::Error>> {
        // Compare local file hashes with remote metadata
        // Return only files that have changed since last push
    }
}
```

## 5. Error Handling & User Experience

### Progress Indication

```rust
pub struct ProgressReporter {
    total_files: usize,
    uploaded_files: usize,
    total_bytes: u64,
    uploaded_bytes: u64,
}

impl ProgressReporter {
    pub fn update(&mut self, file_name: &str, bytes_uploaded: u64) {
        // Update progress and display to user
        println!("Uploading {} [{}/{}] - {}% complete",
                 file_name,
                 self.uploaded_files + 1,
                 self.total_files,
                 self.progress_percentage());
    }
}
```

### Error Recovery

- **Network failures**: Retry with exponential backoff
- **Authentication errors**: Clear error messages with resolution steps
- **Configuration errors**: Validation with helpful suggestions
- **Partial failures**: Continue with remaining files, report failed uploads

### User Feedback

```bash
# Successful push
✓ Pushed 3 files to Azure Storage (2.1 KB)
  - events/20250911.jsonl
  - entries/20250911.md
  - entries/20250910.md

# Error example
✗ Push failed: Authentication error
  Check your Azure storage account configuration in ~/.devlog/config.toml
  Expected format: https://account.blob.core.windows.net/container
```

## 6. MVP Implementation Phases

### Phase 1: Infrastructure (Week 1-2)

- [ ] Configuration system (`config.rs`)
- [ ] Remote storage trait (`remote/mod.rs`)
- [ ] Basic CLI command structure
- [ ] File scanning and hashing utilities

### Phase 2: Azure Implementation (Week 2-3)

- [ ] Azure Blob Storage client (`remote/azure.rs`)
- [ ] Authentication handling
- [ ] Basic upload/download operations
- [ ] Error handling for Azure-specific issues

### Phase 3: Sync Logic (Week 3-4)

- [ ] Incremental change detection
- [ ] Upload orchestration
- [ ] Progress reporting
- [ ] Metadata management

### Phase 4: Polish & Testing (Week 4)

- [ ] Comprehensive error handling
- [ ] User experience improvements
- [ ] Integration tests
- [ ] Documentation updates

## 7. Success Criteria

### MVP Definition of Done

1. **Configuration**: User can set up Azure storage URL in `~/.devlog/config.toml`
2. **Basic Push**: `devlog push` uploads changed files since last push (incremental mode)
3. **All Files Push**: `devlog push --mode all` uploads all files regardless of changes
4. **Progress**: Clear progress indication during upload process
5. **Error Handling**: Meaningful error messages for common failure scenarios
6. **Backwards Compatibility**: Existing local functionality remains unchanged

### Performance Targets

- **Small Changes**: < 5 seconds for 1-3 modified files
- **Initial Upload**: < 30 seconds for complete `.devlog` folder (typical size)
- **Large Files**: Progress indication for files > 1MB

### Reliability Requirements

- **Network Resilience**: Handle temporary network issues gracefully
- **Data Integrity**: Verify upload success with checksums
- **Atomic Operations**: Prevent partial state corruption on failures

## 8. Future Extensibility

### Plugin Architecture for Providers

```rust
// Future: Dynamic provider loading
pub trait RemoteStorageProvider {
    fn name(&self) -> &'static str;
    fn create_storage(&self, config: &RemoteConfig) -> Result<Box<dyn RemoteStorage>, Error>;
}

// Enable third-party provider implementations
pub fn register_provider(provider: Box<dyn RemoteStorageProvider>) {
    // Plugin registration system
}
```

### Configuration Extensions

```toml
# Future configuration capabilities
[remote.azure]
account = "myaccount"
container = "devlog"
auth_method = "service_principal"

[remote.aws]
bucket = "my-devlog-bucket"
region = "us-west-2"
profile = "devlog"

[sync]
auto_push = true
conflict_resolution = "local_wins"  # or "remote_wins", "prompt"

[encryption]
enabled = true
key_source = "env"  # or "keychain", "file"
```

### Hooks for Future Features

```rust
// Event hooks for extensibility
pub trait SyncHook {
    fn pre_push(&self, files: &[LocalFile]) -> Result<(), Error>;
    fn post_push(&self, result: &PushResult) -> Result<(), Error>;
    fn on_error(&self, error: &Error) -> Result<(), Error>;
}
```

## 9. Technical Dependencies

### New Cargo Dependencies

```toml
# Configuration and serialization
toml = "0.8"                    # TOML configuration parsing
serde = { version = "1.0", features = ["derive"] }

# Cryptography and hashing
sha2 = "0.10"                   # File hashing for change detection

# HTTP and networking
reqwest = { version = "0.11", features = ["json"] }  # Azure REST API client
tokio = { version = "1.0", features = ["full"] }     # Async runtime

# Error handling and utilities
anyhow = "1.0"                  # Enhanced error handling
thiserror = "1.0"               # Custom error types

# Testing and development
tempfile = "3.0"                # Temporary files for testing
```

### Azure-Specific Dependencies

```toml
# For more robust Azure integration (future)
azure_storage = "0.16"          # Official Azure SDK (if available)
azure_core = "0.16"             # Core Azure types and authentication
```

## 10. Security Considerations

### Authentication Methods (MVP)

- **Connection String**: Simple for MVP, stored in config file
- **Future**: Service Principal, Managed Identity, SAS tokens

### Data Protection

- **In Transit**: HTTPS for all remote operations
- **At Rest**: Rely on cloud provider encryption
- **Future**: Client-side encryption before upload

### Access Control

- **MVP**: User-level access through Azure storage account
- **Future**: Team collaboration with role-based access
