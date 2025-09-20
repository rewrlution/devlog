# Sync Architecture Design

## Overview

The sync feature enables devlog entries to be synchronized with cloud storage providers. The design follows a provider-agnostic architecture with concrete implementations for Azure Blob Storage and AWS S3.

## Architecture Principles

1. **Simple Interface**: Clean abstractions that hide cloud provider complexity
2. **Provider Agnostic**: Support multiple cloud providers through a common interface
3. **Conflict Resolution**: Last-modified-time wins for conflicting files
4. **SDK-Based**: Use official cloud provider SDKs, no custom HTTP implementations

## Core Components

### 1. Cloud Storage Trait

```rust
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct CloudFile {
    pub name: String,
    pub content: Vec<u8>,
    pub last_modified: DateTime<Utc>,
    pub size: u64,
}

pub trait CloudStorage: Send + Sync {
    /// Upload a file to cloud storage
    async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<(), SyncError>;

    /// Download a file from cloud storage
    async fn download(&self, remote_name: &str, local_path: &Path) -> Result<(), SyncError>;

    /// List all files in the storage container/bucket
    async fn list_files(&self) -> Result<Vec<CloudFile>, SyncError>;

    /// Get metadata for a specific file
    async fn get_metadata(&self, remote_name: &str) -> Result<CloudFile, SyncError>;

    /// Delete a file from cloud storage
    async fn delete(&self, remote_name: &str) -> Result<(), SyncError>;
}
```

### 2. Provider Implementations

#### Azure Blob Storage

- Use `azure_storage_blobs` crate
- Authenticate via connection string from config
- Container name: `devlog-entries`

#### AWS S3

- Use `aws-sdk-s3` crate
- Authenticate via AWS credentials (env vars or config)
- Bucket name: configurable in `config.toml`

### 3. Configuration Structure

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct SyncConfig {
    pub provider: CloudProvider,
    pub azure: Option<AzureConfig>,
    pub aws: Option<AwsConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum CloudProvider {
    Azure,
    Aws,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AzureConfig {
    pub connection_string: String,
    pub container_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AwsConfig {
    pub bucket_name: String,
    pub region: String,
}
```

### 4. Sync Engine

```rust
pub struct SyncEngine {
    storage: Box<dyn CloudStorage>,
    local_entries_path: PathBuf,
}

impl SyncEngine {
    /// Push local changes to cloud
    pub async fn push(&self) -> Result<SyncResult, SyncError>;

    /// Pull remote changes to local
    pub async fn pull(&self) -> Result<SyncResult, SyncError>;

    /// Sync both directions with conflict resolution
    pub async fn sync(&self) -> Result<SyncResult, SyncError>;
}
```

## File Structure

```
src/
├── sync/
│   ├── mod.rs              # Public sync API
│   ├── engine.rs           # SyncEngine implementation
│   ├── providers/
│   │   ├── mod.rs          # CloudStorage trait
│   │   ├── azure.rs        # Azure Blob Storage implementation
│   │   └── aws.rs          # AWS S3 implementation
│   ├── config.rs           # Configuration parsing
│   └── error.rs            # Sync-specific error types
└── commands/
    └── sync.rs             # CLI command for sync operations
```

## Dependencies to Add

```toml
# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Configuration parsing
toml = "0.8"

# Azure Blob Storage
azure_storage = "0.20"
azure_storage_blobs = "0.20"

# AWS S3
aws-config = "1.0"
aws-sdk-s3 = "1.0"

# Additional utilities
uuid = { version = "1.0", features = ["v4"] }
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Cloud storage error: {0}")]
    CloudStorage(String),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Network error: {0}")]
    Network(String),
}
```

## Security Considerations

1. **Credentials Storage**: Never commit credentials to version control
2. **Environment Variables**: Support env var overrides for CI/CD
3. **Validation**: Validate all configuration before attempting sync
4. **Error Messages**: Don't expose sensitive information in error messages
