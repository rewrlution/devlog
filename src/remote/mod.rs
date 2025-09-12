//! Remote storage abstractions and implementations
pub mod azure;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::path::Path;

/// Metadata about a remote file
#[derive(Debug, Clone)]
pub struct RemoteFileInfo {
    pub key: String,
    pub size: Option<u64>,
    pub hash: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
}

/// Generic interface for remote storage providers
#[async_trait]
pub trait RemoteStorage: Send + Sync {
    // `Send` and `Sync` are two fundamental traits in Rust that are crucial for
    // safe concurrency and memory management.
    // They are essential for the remote storage implementation because
    // - concurrent operations: multiple threads might need to perform upload/download simultaneously
    // - async runtime: async runtimes often move futures between threads
    // - thread safety: ensure storage client can be safely used in multi-threaded env
    // Most types automatically implement the `Send` and `Sync` if their components do

    /// Upload a local file to remote storage
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<()>;

    /// Download a file from remote storage to local path
    async fn download_file(&self, remote_key: &str, local_path: &Path) -> Result<()>;

    /// List all files with the given prefix
    async fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFileInfo>>;

    /// Check if a file exists in remote storage
    async fn file_exists(&self, remote_key: &str) -> Result<bool>;

    /// Get file metadata including hash if available
    async fn get_file_info(&self, remote_key: &str) -> Result<Option<RemoteFileInfo>>;

    /// Delete a file from remote storage
    async fn delete_file(&self, remote_key: &str) -> Result<()>;
}
