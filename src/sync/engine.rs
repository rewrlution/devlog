use async_trait::async_trait;
use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::sync::{CloudFile, CloudStorage, SyncResult};

/// MVP: Simple local file system "cloud" provider
/// This simulates cloud storage by copying files to another local directory
pub struct LocalProvider {
    sync_dir: PathBuf,
}

impl LocalProvider {
    pub fn new(sync_dir: impl Into<PathBuf>) -> Result<Self> {
        let sync_dir = sync_dir.into();
        std::fs::create_dir_all(&sync_dir)?;
        Ok(Self { sync_dir })
    }

    fn get_file_mtime(path: &Path) -> Result<DateTime<Utc>> {
        let metadata = std::fs::metadata(path)?;
        let mtime = metadata.modified()?;
        Ok(DateTime::from(mtime))
    }
}

#[async_trait]
impl CloudStorage for LocalProvider {
    async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<()> {
        let remote_path = self.sync_dir.join(remote_name);
        if let Some(parent) = remote_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        tokio::fs::copy(local_path, &remote_path).await?;
        println!("  → Uploaded: {}", remote_name);
        Ok(())
    }

    async fn download(&self, remote_name: &str, local_path: &Path) -> Result<()> {
        let remote_path = self.sync_dir.join(remote_name);
        if !remote_path.exists() {
            return Err(eyre!("Remote file not found: {}", remote_name));
        }

        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        tokio::fs::copy(&remote_path, local_path).await?;
        println!("  ← Downloaded: {}", remote_name);
        Ok(())
    }

    async fn list_files(&self) -> Result<Vec<CloudFile>> {
        let mut files = Vec::new();

        if !self.sync_dir.exists() {
            return Ok(files);
        }

        for entry in WalkDir::new(&self.sync_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                let name = path
                    .strip_prefix(&self.sync_dir)?
                    .to_string_lossy()
                    .to_string();

                let last_modified = Self::get_file_mtime(path)?;

                files.push(CloudFile {
                    name,
                    last_modified,
                });
            }
        }

        Ok(files)
    }
}

/// Sync engine for managing sync operations
pub struct SyncEngine {
    provider: Box<dyn CloudStorage>,
    entries_dir: PathBuf,
}

impl SyncEngine {
    pub fn new(provider: Box<dyn CloudStorage>, entries_dir: impl Into<PathBuf>) -> Self {
        Self {
            provider,
            entries_dir: entries_dir.into(),
        }
    }

    /// Get all local markdown files
    fn get_local_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !self.entries_dir.exists() {
            return Ok(files);
        }

        for entry in WalkDir::new(&self.entries_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    fn get_file_mtime(path: &Path) -> Result<DateTime<Utc>> {
        let metadata = std::fs::metadata(path)?;
        let mtime = metadata.modified()?;
        Ok(DateTime::from(mtime))
    }

    /// Push local changes to cloud
    pub async fn push(&self) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        println!("📤 Pushing local changes...");

        let local_files = self.get_local_files()?;
        let cloud_files = self.provider.list_files().await?;

        // Build cloud files map for quick lookup
        let cloud_map: HashMap<String, CloudFile> = cloud_files
            .into_iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        for local_file in local_files {
            let relative_path = local_file.strip_prefix(&self.entries_dir)?;
            let filename = relative_path.to_string_lossy().to_string();

            let should_upload = match cloud_map.get(&filename) {
                None => {
                    // File doesn't exist in cloud
                    true
                }
                Some(cloud_file) => {
                    // Compare modification times
                    let local_mtime = Self::get_file_mtime(&local_file)?;
                    local_mtime > cloud_file.last_modified
                }
            };

            if should_upload {
                self.provider.upload(&local_file, &filename).await?;
                result.uploaded.push(filename);
            } else {
                result.skipped.push(filename);
            }
        }

        Ok(result)
    }

    /// Pull remote changes to local
    pub async fn pull(&self) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        println!("📥 Pulling remote changes...");

        let cloud_files = self.provider.list_files().await?;
        let local_files = self.get_local_files()?;

        // Build local files map
        let local_map: HashMap<String, DateTime<Utc>> = local_files
            .into_iter()
            .map(|path| {
                let relative_path = path.strip_prefix(&self.entries_dir).unwrap();
                let filename = relative_path.to_string_lossy().to_string();
                let mtime = Self::get_file_mtime(&path).unwrap_or(DateTime::UNIX_EPOCH);
                (filename, mtime)
            })
            .collect();

        for cloud_file in cloud_files {
            let local_path = self.entries_dir.join(&cloud_file.name);

            let should_download = match local_map.get(&cloud_file.name) {
                None => {
                    // File doesn't exist locally
                    true
                }
                Some(local_mtime) => {
                    // Compare modification times
                    cloud_file.last_modified > *local_mtime
                }
            };

            if should_download {
                self.provider
                    .download(&cloud_file.name, &local_path)
                    .await?;
                result.downloaded.push(cloud_file.name);
            } else {
                result.skipped.push(cloud_file.name);
            }
        }

        Ok(result)
    }

    /// Bidirectional sync
    pub async fn sync(&self) -> Result<SyncResult> {
        println!("🔄 Starting bidirectional sync...");

        let push_result = self.push().await?;
        let pull_result = self.pull().await?;

        Ok(SyncResult {
            uploaded: push_result.uploaded,
            downloaded: pull_result.downloaded,
            skipped: [push_result.skipped, pull_result.skipped].concat(),
        })
    }
}
