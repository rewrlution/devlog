# Sync Operations Design

## Overview

This document details the implementation of sync operations including push, pull, and bidirectional sync with conflict resolution. The design emphasizes simplicity and reliability using the "last modified wins" strategy.

## Core Operations

### 1. Push Operation

Uploads local entries that are newer than their cloud counterparts or don't exist in the cloud.

```rust
impl SyncEngine {
    pub async fn push(&self) -> Result<SyncResult, SyncError> {
        let mut result = SyncResult::new();

        // 1. Get all local files
        let local_files = self.get_local_files()?;

        // 2. Get cloud file metadata
        let cloud_files = self.storage.list_files().await?;
        let cloud_map: HashMap<String, CloudFile> = cloud_files
            .into_iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        // 3. Process each local file
        for local_file in local_files {
            let filename = local_file.file_name()
                .and_then(|n| n.to_str())
                .ok_or(SyncError::InvalidFilename)?;

            let should_upload = match cloud_map.get(filename) {
                None => {
                    // File doesn't exist in cloud, upload it
                    result.uploaded.push(filename.to_string());
                    true
                }
                Some(cloud_file) => {
                    // Compare modification times
                    let local_mtime = self.get_file_mtime(&local_file)?;
                    if local_mtime > cloud_file.last_modified {
                        result.uploaded.push(filename.to_string());
                        true
                    } else {
                        result.skipped.push(filename.to_string());
                        false
                    }
                }
            };

            if should_upload {
                self.storage.upload(&local_file, filename).await?;
            }
        }

        Ok(result)
    }
}
```

### 2. Pull Operation

Downloads cloud entries that are newer than their local counterparts or don't exist locally.

```rust
impl SyncEngine {
    pub async fn pull(&self) -> Result<SyncResult, SyncError> {
        let mut result = SyncResult::new();

        // 1. Get all cloud files
        let cloud_files = self.storage.list_files().await?;

        // 2. Get local file metadata
        let local_files = self.get_local_files()?;
        let local_map: HashMap<String, (PathBuf, DateTime<Utc>)> = local_files
            .into_iter()
            .map(|path| {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let mtime = self.get_file_mtime(&path).unwrap_or(DateTime::UNIX_EPOCH);
                (filename, (path, mtime))
            })
            .collect();

        // 3. Process each cloud file
        for cloud_file in cloud_files {
            let local_path = self.local_entries_path.join(&cloud_file.name);

            let should_download = match local_map.get(&cloud_file.name) {
                None => {
                    // File doesn't exist locally, download it
                    result.downloaded.push(cloud_file.name.clone());
                    true
                }
                Some((_, local_mtime)) => {
                    // Compare modification times
                    if cloud_file.last_modified > *local_mtime {
                        result.downloaded.push(cloud_file.name.clone());
                        true
                    } else {
                        result.skipped.push(cloud_file.name.clone());
                        false
                    }
                }
            };

            if should_download {
                self.storage.download(&cloud_file.name, &local_path).await?;
                // Update local file modification time to match cloud
                self.set_file_mtime(&local_path, cloud_file.last_modified)?;
            }
        }

        Ok(result)
    }
}
```

### 3. Bidirectional Sync

Combines push and pull operations with conflict resolution.

```rust
impl SyncEngine {
    pub async fn sync(&self) -> Result<SyncResult, SyncError> {
        let mut result = SyncResult::new();

        // 1. Get both local and cloud files
        let local_files = self.get_local_files()?;
        let cloud_files = self.storage.list_files().await?;

        // 2. Build maps for efficient lookup
        let local_map: HashMap<String, (PathBuf, DateTime<Utc>)> = local_files
            .into_iter()
            .map(|path| {
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let mtime = self.get_file_mtime(&path).unwrap_or(DateTime::UNIX_EPOCH);
                (filename, (path, mtime))
            })
            .collect();

        let cloud_map: HashMap<String, CloudFile> = cloud_files
            .into_iter()
            .map(|f| (f.name.clone(), f))
            .collect();

        // 3. Get all unique filenames
        let all_files: HashSet<String> = local_map.keys()
            .chain(cloud_map.keys())
            .cloned()
            .collect();

        // 4. Process each file
        for filename in all_files {
            match (local_map.get(&filename), cloud_map.get(&filename)) {
                (Some((local_path, local_mtime)), Some(cloud_file)) => {
                    // File exists in both locations - resolve conflict
                    if *local_mtime > cloud_file.last_modified {
                        // Local is newer, upload
                        self.storage.upload(local_path, &filename).await?;
                        result.uploaded.push(filename);
                    } else if cloud_file.last_modified > *local_mtime {
                        // Cloud is newer, download
                        self.storage.download(&filename, local_path).await?;
                        self.set_file_mtime(local_path, cloud_file.last_modified)?;
                        result.downloaded.push(filename);
                    } else {
                        // Same modification time, skip
                        result.skipped.push(filename);
                    }
                }
                (Some((local_path, _)), None) => {
                    // File only exists locally, upload
                    self.storage.upload(local_path, &filename).await?;
                    result.uploaded.push(filename);
                }
                (None, Some(cloud_file)) => {
                    // File only exists in cloud, download
                    let local_path = self.local_entries_path.join(&filename);
                    self.storage.download(&filename, &local_path).await?;
                    self.set_file_mtime(&local_path, cloud_file.last_modified)?;
                    result.downloaded.push(filename);
                }
                (None, None) => {
                    // This shouldn't happen, but handle gracefully
                    continue;
                }
            }
        }

        Ok(result)
    }
}
```

## Conflict Resolution Strategy

### Last Modified Wins

The conflict resolution strategy is simple and deterministic:

1. **Compare modification times** of local and cloud files
2. **Newer file wins** - overwrites the older version
3. **Preserve modification time** when downloading to maintain consistency
4. **No manual merge** - completely automatic resolution

### Edge Cases

```rust
impl SyncEngine {
    /// Handle edge cases in sync operations
    fn handle_edge_cases(&self) -> Result<(), SyncError> {
        // 1. Invalid filenames (non-UTF8)
        // 2. Very large files (>100MB)
        // 3. Network timeouts
        // 4. Permission errors
        // 5. Disk space issues
        Ok(())
    }

    /// Validate file before sync operation
    fn validate_file(&self, path: &Path) -> Result<(), SyncError> {
        // Check file size (limit to 100MB)
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > 100 * 1024 * 1024 {
            return Err(SyncError::FileTooLarge(path.to_path_buf()));
        }

        // Check filename validity
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if !filename.ends_with(".md") {
                return Err(SyncError::InvalidFileType(filename.to_string()));
            }
        } else {
            return Err(SyncError::InvalidFilename);
        }

        Ok(())
    }
}
```

## Result Reporting

```rust
#[derive(Debug, Default)]
pub struct SyncResult {
    pub uploaded: Vec<String>,
    pub downloaded: Vec<String>,
    pub skipped: Vec<String>,
    pub conflicts_resolved: Vec<ConflictResolution>,
    pub errors: Vec<SyncError>,
}

#[derive(Debug)]
pub struct ConflictResolution {
    pub filename: String,
    pub winner: ConflictWinner,
    pub local_mtime: DateTime<Utc>,
    pub cloud_mtime: DateTime<Utc>,
}

#[derive(Debug)]
pub enum ConflictWinner {
    Local,
    Cloud,
}

impl SyncResult {
    pub fn print_summary(&self) {
        println!("Sync Summary:");

        if !self.uploaded.is_empty() {
            println!("  Uploaded {} files: {}",
                self.uploaded.len(),
                self.uploaded.join(", ")
            );
        }

        if !self.downloaded.is_empty() {
            println!("  Downloaded {} files: {}",
                self.downloaded.len(),
                self.downloaded.join(", ")
            );
        }

        if !self.skipped.is_empty() {
            println!("  Skipped {} files (already in sync)", self.skipped.len());
        }

        if !self.conflicts_resolved.is_empty() {
            println!("  Resolved {} conflicts:", self.conflicts_resolved.len());
            for conflict in &self.conflicts_resolved {
                println!("    {}: {:?} won", conflict.filename, conflict.winner);
            }
        }

        if !self.errors.is_empty() {
            println!("  Errors: {}", self.errors.len());
            for error in &self.errors {
                eprintln!("    Error: {}", error);
            }
        }
    }
}
```

## Utility Functions

```rust
impl SyncEngine {
    /// Get all markdown files in the local entries directory
    fn get_local_files(&self) -> Result<Vec<PathBuf>, SyncError> {
        let mut files = Vec::new();

        if !self.local_entries_path.exists() {
            return Ok(files);
        }

        for entry in walkdir::WalkDir::new(&self.local_entries_path) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                self.validate_file(path)?;
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// Get file modification time
    fn get_file_mtime(&self, path: &Path) -> Result<DateTime<Utc>, SyncError> {
        let metadata = std::fs::metadata(path)?;
        let mtime = metadata.modified()?;
        Ok(DateTime::from(mtime))
    }

    /// Set file modification time
    fn set_file_mtime(&self, path: &Path, mtime: DateTime<Utc>) -> Result<(), SyncError> {
        let system_time: SystemTime = mtime.into();
        let file = std::fs::File::open(path)?;
        file.set_modified(system_time)?;
        Ok(())
    }
}
```

## Performance Considerations

1. **Batch Operations**: Process multiple files concurrently when possible
2. **Incremental Sync**: Only check modification times, don't read file contents unless necessary
3. **Metadata Caching**: Cache cloud file metadata to reduce API calls
4. **Connection Pooling**: Reuse HTTP connections for cloud operations
5. **Progress Reporting**: Show progress for large sync operations

## Error Recovery

```rust
impl SyncEngine {
    /// Retry failed operations with exponential backoff
    async fn retry_operation<F, T>(&self, operation: F, max_retries: u32) -> Result<T, SyncError>
    where
        F: Fn() -> BoxFuture<'_, Result<T, SyncError>>,
    {
        let mut retries = 0;
        let mut delay = Duration::from_millis(100);

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if retries >= max_retries || !self.is_retryable_error(&error) {
                        return Err(error);
                    }

                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                    retries += 1;
                }
            }
        }
    }

    fn is_retryable_error(&self, error: &SyncError) -> bool {
        matches!(error,
            SyncError::Network(_) |
            SyncError::CloudStorage(_)
        )
    }
}
```
