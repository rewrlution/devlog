# Task 06: Azure Storage Client Implementation

**Estimated Time**: 2-3 hours  
**Difficulty**: ⭐⭐ Beginner-Intermediate  
**Prerequisites**: Tasks 01-05 completed

## Objective

Implement the Azure Blob Storage client using the official Azure SDK for Rust. This provides a much cleaner and more robust implementation than raw HTTP requests.

## What You'll Learn

- Using official Azure SDK for Rust
- Authentication with Azure storage accounts
- Working with async Rust and blob operations
- Error handling with external libraries
- Parsing Azure storage URLs

## Tasks

### 1. Add Azure SDK Dependencies

Update your `Cargo.toml` to include the official Azure SDK:

```toml
# Azure SDK dependencies
azure_storage = "0.20"
azure_storage_blobs = "0.20"
azure_core = "0.20"
azure_identity = "0.20"

# Additional dependency for URL parsing
url = "2.4"

# Keep existing dependencies
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
async-trait = "0.1"
chrono = { version = "0.4", features = ["serde"] }
```

### 2. Implement Azure Storage Client

In `src/remote/azure.rs`, implement the Azure storage client:

```rust
//! Azure Blob Storage implementation using official Azure SDK

use crate::config::RemoteConfig;
use crate::remote::{RemoteStorage, RemoteFileInfo};
use anyhow::{Result, Context};
use async_trait::async_trait;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;
use chrono::{DateTime, Utc};
use std::path::Path;

/// Azure Blob Storage client
#[derive(Debug)]
pub struct AzureStorage {
    container_client: ContainerClient,
    container_name: String,
}

impl AzureStorage {
    /// Create a new Azure storage client from configuration
    pub fn new(config: &RemoteConfig) -> Result<Self> {
        if config.provider != "azure" {
            return Err(anyhow::anyhow!(
                "Expected Azure provider, got: {}",
                config.provider
            ));
        }

        let (account_name, container_name) = Self::parse_azure_url(&config.url)
            .context("Failed to parse Azure storage URL")?;

        // Get the account key from environment variable
        let account_key = std::env::var("AZURE_STORAGE_ACCOUNT_KEY")
            .context("AZURE_STORAGE_ACCOUNT_KEY environment variable not set")?;

        // Create storage credentials
        let storage_credentials = StorageCredentials::access_key(account_name.clone(), account_key);

        // Create the storage and container clients
        let storage_client = StorageAccountClient::new(account_name, storage_credentials);
        let container_client = storage_client.container_client(&container_name);

        Ok(Self {
            container_client,
            container_name,
        })
    }

    /// Parse Azure Blob Storage URL
    /// Expected format: https://account.blob.core.windows.net/container
    fn parse_azure_url(url: &str) -> Result<(String, String)> {
        let url = url.trim_end_matches('/');

        // Parse the URL to extract components
        let parsed = url::Url::parse(url)
            .context("Invalid Azure storage URL format")?;

        // Extract account name from hostname
        let host = parsed.host_str()
            .ok_or_else(|| anyhow::anyhow!("No hostname in Azure storage URL"))?;

        let account_name = host.split('.').next()
            .ok_or_else(|| anyhow::anyhow!("Cannot extract account name from hostname"))?
            .to_string();

        // Extract container name from path
        let container_name = parsed.path()
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();

        if container_name.is_empty() {
            return Err(anyhow::anyhow!("Container name not found in URL path"));
        }

        Ok((account_name, container_name))
    }
}

#[async_trait]
impl RemoteStorage for AzureStorage {
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<()> {
        let file_content = tokio::fs::read(local_path).await
            .with_context(|| format!("Failed to read local file: {:?}", local_path))?;

        self.container_client
            .blob_client(remote_key)
            .put_block_blob(file_content)
            .await
            .with_context(|| format!("Failed to upload file to blob: {}", remote_key))?;

        Ok(())
    }

    async fn download_file(&self, remote_key: &str, local_path: &Path) -> Result<()> {
        let blob_data = self.container_client
            .blob_client(remote_key)
            .get()
            .await
            .with_context(|| format!("Failed to download blob: {}", remote_key))?
            .data
            .collect()
            .await
            .context("Failed to collect blob data")?;

        // Create parent directories if needed
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create parent directories: {:?}", parent))?;
        }

        tokio::fs::write(local_path, blob_data).await
            .with_context(|| format!("Failed to write file: {:?}", local_path))?;

        Ok(())
    }

    async fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFileInfo>> {
        let mut list_builder = self.container_client.list_blobs();

        if !prefix.is_empty() {
            list_builder = list_builder.prefix(prefix);
        }

        let response = list_builder
            .await
            .context("Failed to list blobs")?;

        let files = response.blobs.blobs()
            .iter()
            .map(|blob| RemoteFileInfo {
                key: blob.name.clone(),
                size: blob.properties.content_length,
                hash: blob.properties.content_md5.clone(),
                last_modified: blob.properties.last_modified,
            })
            .collect();

        Ok(files)
    }

    async fn file_exists(&self, remote_key: &str) -> Result<bool> {
        match self.container_client
            .blob_client(remote_key)
            .get_properties()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                // Check if it's a "not found" error
                if err.to_string().contains("404") || err.to_string().contains("NotFound") {
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!("Error checking if file exists: {}", err))
                }
            }
        }
    }

    async fn get_file_info(&self, remote_key: &str) -> Result<Option<RemoteFileInfo>> {
        match self.container_client
            .blob_client(remote_key)
            .get_properties()
            .await
        {
            Ok(properties) => {
                Ok(Some(RemoteFileInfo {
                    key: remote_key.to_string(),
                    size: properties.blob.properties.content_length,
                    hash: properties.blob.properties.content_md5,
                    last_modified: properties.blob.properties.last_modified,
                }))
            }
            Err(err) => {
                // Check if it's a "not found" error
                if err.to_string().contains("404") || err.to_string().contains("NotFound") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Error getting file info: {}", err))
                }
            }
        }
    }

    async fn delete_file(&self, remote_key: &str) -> Result<()> {
        self.container_client
            .blob_client(remote_key)
            .delete()
            .await
            .with_context(|| format!("Failed to delete blob: {}", remote_key))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_azure_url() {
        // Valid URL
        let result = AzureStorage::parse_azure_url("https://myaccount.blob.core.windows.net/devlog");
        assert!(result.is_ok());
        let (account, container) = result.unwrap();
        assert_eq!(account, "myaccount");
        assert_eq!(container, "devlog");

        // URL with trailing slash
        let result = AzureStorage::parse_azure_url("https://myaccount.blob.core.windows.net/devlog/");
        assert!(result.is_ok());
        let (account, container) = result.unwrap();
        assert_eq!(account, "myaccount");
        assert_eq!(container, "devlog");

        // Invalid URL - no container
        let result = AzureStorage::parse_azure_url("https://myaccount.blob.core.windows.net/");
        assert!(result.is_err());

        // Invalid URL - malformed
        let result = AzureStorage::parse_azure_url("not-a-url");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_azure_storage_creation() {
        // This test requires environment variables, so we'll skip if not available
        if std::env::var("AZURE_STORAGE_ACCOUNT_KEY").is_err() {
            println!("Skipping test - AZURE_STORAGE_ACCOUNT_KEY not set");
            return;
        }

        let config = RemoteConfig {
            provider: "azure".to_string(),
            url: "https://testaccount.blob.core.windows.net/testcontainer".to_string(),
        };

        let result = AzureStorage::new(&config);
        assert!(result.is_ok());
    }
}
```

### 3. Update Storage Factory

Update the `StorageFactory` in `src/remote/mod.rs` to create Azure storage instances:

```rust
impl StorageFactory {
    /// Create a storage instance based on the provided configuration
    pub fn create_storage(config: &RemoteConfig) -> Result<Box<dyn RemoteStorage>> {
        match config.provider.as_str() {
            "azure" => {
                let azure_storage = crate::remote::azure::AzureStorage::new(config)?;
                Ok(Box::new(azure_storage))
            }
            _ => Err(anyhow::anyhow!(
                "Unsupported storage provider: {}. Currently supported: azure",
                config.provider
            )),
        }
    }
}
```

## Validation Steps

### 1. Environment Setup

Set up your Azure storage account for testing:

```bash
# Set your Azure storage account key
export AZURE_STORAGE_ACCOUNT_KEY="your_account_key_here"

# Create test configuration
mkdir -p ~/.devlog
cat > ~/.devlog/config.toml << EOF
[remote]
provider = "azure"
url = "https://youraccount.blob.core.windows.net/devlog"

[sync]
EOF
```

### 2. Compilation Test

```bash
# Build the project
cargo build

# Run unit tests
cargo test azure

# Test configuration validation
./target/debug/devlog push --dry-run
```

### 3. Integration Test (Optional)

If you have an actual Azure storage account set up:

```bash
# Test with real Azure storage
AZURE_STORAGE_ACCOUNT_KEY="your_key" cargo test --test integration
```

## Expected Outputs

After completing this task:

- ✅ Azure storage client implements all `RemoteStorage` trait methods
- ✅ Authentication with Azure storage account keys works
- ✅ File upload/download operations function correctly
- ✅ Error handling provides meaningful messages using `anyhow`
- ✅ Storage factory can create Azure storage instances
- ✅ All unit tests pass

## Troubleshooting

**Common Issues**:

1. **Authentication Errors**:

   ```bash
   # Check environment variable is set
   echo $AZURE_STORAGE_ACCOUNT_KEY
   ```

2. **URL Parsing Errors**:

   - Ensure URL format is: `https://account.blob.core.windows.net/container`
   - Check that the storage account name in URL matches your actual account

3. **Permission Errors**:

   - Verify your account key has the necessary permissions
   - Check that the container exists in your storage account

4. **Compilation Errors**:

   ```bash
   # Update dependencies
   cargo update

   # Check specific errors
   cargo check
   ```

**Testing Commands**:

```bash
# Check compilation
cargo check

# Run unit tests only
cargo test azure::tests

# Check with clippy for best practices
cargo clippy
```

## Next Steps

Once this task is complete, proceed to **Task 07: Sync Manager** where we'll implement the core synchronization logic that uses this Azure storage client.

## Rust Learning Notes

**Key Concepts Introduced**:

- **External Crates**: Using well-maintained third-party libraries
- **Error Context**: Using `anyhow::Context` for better error messages
- **URL Parsing**: Working with the `url` crate for parsing URLs
- **Environment Variables**: Reading configuration from environment
- **Async Programming**: Working with async/await in a real-world scenario

**Why Use Official SDKs?**:

1. **Less Code**: No manual HTTP requests or authentication
2. **Better Error Handling**: Proper typed errors and retry logic
3. **Security**: Authentication handled by Microsoft's experts
4. **Maintainability**: Updates and bug fixes from the Azure team
5. **Multiple Auth Methods**: Support for managed identity, SAS tokens, etc.

**Questions to Research**:

1. What are the benefits of using official SDKs vs. raw HTTP?
2. How does the `anyhow::Context` trait help with error handling?
3. What's the difference between `azure_storage` and `azure_storage_blobs` crates?
4. How does async error handling work in Rust?
