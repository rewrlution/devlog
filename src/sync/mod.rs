use std::path::Path;
use chrono::{DateTime, Utc};
use color_eyre::Result;
use async_trait::async_trait;

pub mod config;
pub mod engine;
pub mod providers;

/// Represents a file in cloud storage
#[derive(Debug, Clone)]
pub struct CloudFile {
    pub name: String,
    pub last_modified: DateTime<Utc>,
    pub size: u64,
}

/// Trait for cloud storage providers
/// 
/// MVP: We'll implement a simple local file system version first
/// Future: Add Azure, AWS implementations
#[async_trait]
pub trait CloudStorage: Send + Sync {
    async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<()>;
    async fn download(&self, remote_name: &str, local_path: &Path) -> Result<()>;
    async fn list_files(&self) -> Result<Vec<CloudFile>>;
}

/// Result of a sync operation
#[derive(Debug, Default)]
pub struct SyncResult {
    pub uploaded: Vec<String>,
    pub downloaded: Vec<String>,
    pub skipped: Vec<String>,
}

impl SyncResult {
    pub fn print_summary(&self) {
        if !self.uploaded.is_empty() {
            println!("Uploaded: {}", self.uploaded.join(", "));
        }
        if !self.downloaded.is_empty() {
            println!("Downloaded: {}", self.downloaded.join(", "));
        }
        if !self.skipped.is_empty() {
            println!("Skipped: {} files (already in sync)", self.skipped.len());
        }
    }
}
