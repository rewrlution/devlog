# Task 06: Azure Storage Client Implementation

**Estimated Time**: 3-4 hours  
**Difficulty**: ⭐⭐⭐ Intermediate  
**Prerequisites**: Tasks 01-05 completed

## Objective

Implement the Azure Blob Storage client that integrates with the `RemoteStorage` trait, providing actual cloud storage functionality.

## What You'll Learn

- Azure Blob Storage REST API integration
- HTTP client usage with reqwest
- Authentication with Azure storage accounts
- Azure-specific error handling
- Working with async Rust and HTTP requests

## Tasks

### 1. Understand Azure Blob Storage REST API

Azure Blob Storage provides a REST API for file operations. Key concepts:

- **Storage Account**: Your Azure storage account (e.g., `myaccount.blob.core.windows.net`)
- **Container**: A logical grouping of blobs (like a folder)
- **Blob**: An individual file stored in the container
- **Authentication**: Account key, SAS token, or Azure AD

### 2. Implement Azure Storage Client

In `src/remote/azure.rs`, implement the Azure storage client:

```rust
//! Azure Blob Storage implementation

use crate::config::RemoteConfig;
use crate::remote::{RemoteStorage, RemoteFileInfo, StorageError};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, Method, StatusCode};
use std::path::Path;
use std::collections::HashMap;

/// Azure Blob Storage client
#[derive(Debug)]
pub struct AzureStorage {
    client: Client,
    account_name: String,
    container_name: String,
    account_key: String,
    base_url: String,
}

impl AzureStorage {
    /// Create a new Azure storage client from configuration
    pub fn new(config: &RemoteConfig) -> Result<Self> {
        if config.provider != "azure" {
            return Err(anyhow::anyhow!("Invalid provider for Azure storage: {}", config.provider));
        }

        let (account_name, container_name) = Self::parse_azure_url(&config.url)?;

        // For MVP, we'll use account key from environment variable
        // In production, you might want to support multiple auth methods
        let account_key = std::env::var("AZURE_STORAGE_ACCOUNT_KEY")
            .map_err(|_| anyhow::anyhow!(
                "AZURE_STORAGE_ACCOUNT_KEY environment variable not set. \
                Please set this to your Azure storage account key."
            ))?;

        let base_url = format!("https://{}.blob.core.windows.net/{}", account_name, container_name);

        Ok(Self {
            client: Client::new(),
            account_name,
            container_name,
            account_key,
            base_url,
        })
    }

    /// Parse Azure Blob Storage URL
    /// Expected format: https://account.blob.core.windows.net/container
    fn parse_azure_url(url: &str) -> Result<(String, String)> {
        let url = url.trim_end_matches('/');

        if !url.starts_with("https://") {
            return Err(anyhow::anyhow!("Azure storage URL must use HTTPS"));
        }

        if !url.contains(".blob.core.windows.net") {
            return Err(anyhow::anyhow!("Invalid Azure blob storage URL format"));
        }

        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() < 4 {
            return Err(anyhow::anyhow!(
                "Azure storage URL must include container name: https://account.blob.core.windows.net/container"
            ));
        }

        let domain = parts[2]; // account.blob.core.windows.net
        let container = parts[3];

        let account_name = domain.split('.').next()
            .ok_or_else(|| anyhow::anyhow!("Could not extract account name from URL"))?
            .to_string();

        Ok((account_name, container.to_string()))
    }

    /// Generate Azure Storage authentication header
    fn generate_auth_header(&self, method: &str, url: &str, content_length: u64) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        use base64::{Engine as _, engine::general_purpose};

        let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        // Simplified auth for MVP - in production you'd want full canonical string
        let string_to_sign = format!(
            "{}\n\n\n{}\n\n\n\n\n\n\n\n\nx-ms-date:{}\nx-ms-version:2020-04-08",
            method.to_uppercase(),
            content_length,
            date
        );

        let account_key_bytes = general_purpose::STANDARD.decode(&self.account_key)
            .expect("Invalid base64 in account key");

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(&account_key_bytes)
            .expect("HMAC can take key of any size");
        mac.update(string_to_sign.as_bytes());

        let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        format!("SharedKey {}:{}", self.account_name, signature)
    }

    /// Build blob URL for a given key
    fn blob_url(&self, blob_key: &str) -> String {
        format!("{}/{}", self.base_url, blob_key)
    }
}

#[async_trait]
impl RemoteStorage for AzureStorage {
    async fn upload_file(&self, local_path: &Path, remote_key: &str) -> Result<()> {
        let file_content = tokio::fs::read(local_path).await
            .map_err(|e| StorageError::UploadFailed {
                path: local_path.to_string_lossy().to_string(),
                message: format!("Failed to read local file: {}", e),
            })?;

        let url = self.blob_url(remote_key);
        let content_length = file_content.len() as u64;
        let auth_header = self.generate_auth_header("PUT", &url, content_length);

        let response = self.client
            .put(&url)
            .header("Authorization", auth_header)
            .header("x-ms-date", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header("x-ms-version", "2020-04-08")
            .header("x-ms-blob-type", "BlockBlob")
            .header("Content-Length", content_length.to_string())
            .body(file_content)
            .send()
            .await
            .map_err(|e| StorageError::NetworkError {
                message: format!("Failed to send request: {}", e),
            })?;

        match response.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::UNAUTHORIZED => Err(StorageError::AuthenticationFailed {
                message: "Invalid account key or permissions".to_string(),
            }.into()),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(StorageError::UploadFailed {
                    path: remote_key.to_string(),
                    message: format!("HTTP {}: {}", status, error_text),
                }.into())
            }
        }
    }

    async fn download_file(&self, remote_key: &str, local_path: &Path) -> Result<()> {
        let url = self.blob_url(remote_key);
        let auth_header = self.generate_auth_header("GET", &url, 0);

        let response = self.client
            .get(&url)
            .header("Authorization", auth_header)
            .header("x-ms-date", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header("x-ms-version", "2020-04-08")
            .send()
            .await
            .map_err(|e| StorageError::NetworkError {
                message: format!("Failed to send request: {}", e),
            })?;

        match response.status() {
            StatusCode::OK => {
                let content = response.bytes().await
                    .map_err(|e| StorageError::DownloadFailed {
                        path: remote_key.to_string(),
                        message: format!("Failed to read response body: {}", e),
                    })?;

                if let Some(parent) = local_path.parent() {
                    tokio::fs::create_dir_all(parent).await
                        .map_err(|e| StorageError::DownloadFailed {
                            path: local_path.to_string_lossy().to_string(),
                            message: format!("Failed to create parent directory: {}", e),
                        })?;
                }

                tokio::fs::write(local_path, content).await
                    .map_err(|e| StorageError::DownloadFailed {
                        path: local_path.to_string_lossy().to_string(),
                        message: format!("Failed to write file: {}", e),
                    })?;

                Ok(())
            }
            StatusCode::NOT_FOUND => Err(StorageError::FileNotFound {
                path: remote_key.to_string(),
            }.into()),
            StatusCode::UNAUTHORIZED => Err(StorageError::AuthenticationFailed {
                message: "Invalid account key or permissions".to_string(),
            }.into()),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(StorageError::DownloadFailed {
                    path: remote_key.to_string(),
                    message: format!("HTTP {}: {}", status, error_text),
                }.into())
            }
        }
    }

    async fn list_files(&self, prefix: &str) -> Result<Vec<RemoteFileInfo>> {
        let mut url = format!("{}", self.base_url.replace(&self.container_name, ""));
        url.push_str(&format!("{}?restype=container&comp=list", self.container_name));

        if !prefix.is_empty() {
            url.push_str(&format!("&prefix={}", prefix));
        }

        let auth_header = self.generate_auth_header("GET", &url, 0);

        let response = self.client
            .get(&url)
            .header("Authorization", auth_header)
            .header("x-ms-date", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header("x-ms-version", "2020-04-08")
            .send()
            .await
            .map_err(|e| StorageError::NetworkError {
                message: format!("Failed to send request: {}", e),
            })?;

        match response.status() {
            StatusCode::OK => {
                let xml_content = response.text().await
                    .map_err(|e| StorageError::NetworkError {
                        message: format!("Failed to read response: {}", e),
                    })?;

                // Simple XML parsing for MVP - in production use a proper XML parser
                let files = Self::parse_blob_list_xml(&xml_content)?;
                Ok(files)
            }
            StatusCode::UNAUTHORIZED => Err(StorageError::AuthenticationFailed {
                message: "Invalid account key or permissions".to_string(),
            }.into()),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(StorageError::ProviderError {
                    message: format!("Failed to list files - HTTP {}: {}", status, error_text),
                }.into())
            }
        }
    }

    async fn file_exists(&self, remote_key: &str) -> Result<bool> {
        let url = self.blob_url(remote_key);
        let auth_header = self.generate_auth_header("HEAD", &url, 0);

        let response = self.client
            .head(&url)
            .header("Authorization", auth_header)
            .header("x-ms-date", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header("x-ms-version", "2020-04-08")
            .send()
            .await
            .map_err(|e| StorageError::NetworkError {
                message: format!("Failed to send request: {}", e),
            })?;

        match response.status() {
            StatusCode::OK => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            StatusCode::UNAUTHORIZED => Err(StorageError::AuthenticationFailed {
                message: "Invalid account key or permissions".to_string(),
            }.into()),
            status => {
                Err(StorageError::ProviderError {
                    message: format!("Unexpected response checking file existence - HTTP {}", status),
                }.into())
            }
        }
    }

    async fn get_file_info(&self, remote_key: &str) -> Result<Option<RemoteFileInfo>> {
        let url = self.blob_url(remote_key);
        let auth_header = self.generate_auth_header("HEAD", &url, 0);

        let response = self.client
            .head(&url)
            .header("Authorization", auth_header)
            .header("x-ms-date", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header("x-ms-version", "2020-04-08")
            .send()
            .await
            .map_err(|e| StorageError::NetworkError {
                message: format!("Failed to send request: {}", e),
            })?;

        match response.status() {
            StatusCode::OK => {
                let size = response.headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse().ok());

                let hash = response.headers()
                    .get("content-md5")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                let last_modified = response.headers()
                    .get("last-modified")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| DateTime::parse_from_rfc2822(s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                Ok(Some(RemoteFileInfo {
                    key: remote_key.to_string(),
                    size,
                    hash,
                    last_modified,
                }))
            }
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED => Err(StorageError::AuthenticationFailed {
                message: "Invalid account key or permissions".to_string(),
            }.into()),
            status => {
                Err(StorageError::ProviderError {
                    message: format!("Unexpected response getting file info - HTTP {}", status),
                }.into())
            }
        }
    }

    async fn delete_file(&self, remote_key: &str) -> Result<()> {
        let url = self.blob_url(remote_key);
        let auth_header = self.generate_auth_header("DELETE", &url, 0);

        let response = self.client
            .delete(&url)
            .header("Authorization", auth_header)
            .header("x-ms-date", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .header("x-ms-version", "2020-04-08")
            .send()
            .await
            .map_err(|e| StorageError::NetworkError {
                message: format!("Failed to send request: {}", e),
            })?;

        match response.status() {
            StatusCode::ACCEPTED => Ok(()),
            StatusCode::NOT_FOUND => Err(StorageError::FileNotFound {
                path: remote_key.to_string(),
            }.into()),
            StatusCode::UNAUTHORIZED => Err(StorageError::AuthenticationFailed {
                message: "Invalid account key or permissions".to_string(),
            }.into()),
            status => {
                let error_text = response.text().await.unwrap_or_default();
                Err(StorageError::ProviderError {
                    message: format!("Failed to delete file - HTTP {}: {}", status, error_text),
                }.into())
            }
        }
    }
}

impl AzureStorage {
    /// Simple XML parser for blob list response (MVP implementation)
    fn parse_blob_list_xml(xml: &str) -> Result<Vec<RemoteFileInfo>> {
        let mut files = Vec::new();

        // Very basic XML parsing - in production, use a proper XML parser like quick-xml
        for line in xml.lines() {
            if line.trim().starts_with("<Name>") && line.trim().ends_with("</Name>") {
                let name = line.trim()
                    .strip_prefix("<Name>").unwrap()
                    .strip_suffix("</Name>").unwrap()
                    .to_string();

                files.push(RemoteFileInfo {
                    key: name,
                    size: None,
                    hash: None,
                    last_modified: None,
                });
            }
        }

        Ok(files)
    }
}
```

### 3. Add Required Dependencies

Update your `Cargo.toml` to include the necessary dependencies:

```toml
# Add these to your existing dependencies
base64 = "0.21"       # For base64 encoding/decoding
hmac = "0.12"         # For HMAC authentication
sha2 = "0.10"         # For SHA256 hashing (should already be included)
tokio = { version = "1.0", features = ["full", "fs"] }  # Add "fs" feature
```

### 4. Update Storage Factory

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

### 2. Unit Tests

Create tests in `src/remote/azure.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RemoteConfig;

    #[test]
    fn test_parse_azure_url() {
        let (account, container) = AzureStorage::parse_azure_url(
            "https://myaccount.blob.core.windows.net/devlog"
        ).unwrap();

        assert_eq!(account, "myaccount");
        assert_eq!(container, "devlog");
    }

    #[test]
    fn test_invalid_azure_urls() {
        assert!(AzureStorage::parse_azure_url("http://example.com").is_err());
        assert!(AzureStorage::parse_azure_url("https://example.com").is_err());
        assert!(AzureStorage::parse_azure_url("https://account.blob.core.windows.net").is_err());
    }

    #[tokio::test]
    async fn test_azure_storage_creation() {
        let config = RemoteConfig {
            provider: "azure".to_string(),
            url: "https://testaccount.blob.core.windows.net/devlog".to_string(),
        };

        // This will fail without environment variable, which is expected
        match AzureStorage::new(&config) {
            Ok(_) => println!("Azure storage created successfully"),
            Err(e) => println!("Expected error without environment variable: {}", e),
        }
    }
}
```

### 3. Integration Test

Create a simple integration test (requires actual Azure storage):

```bash
# Build the project
cargo build

# Test configuration validation
./target/debug/devlog push --dry-run
```

## Expected Outputs

After completing this task:

- ✅ Azure storage client implements all `RemoteStorage` trait methods
- ✅ Authentication with Azure storage account keys works
- ✅ File upload/download operations function correctly
- ✅ Error handling provides meaningful Azure-specific error messages
- ✅ Storage factory can create Azure storage instances
- ✅ All tests pass (unit tests should pass, integration tests require Azure setup)

## Troubleshooting

**Common Issues**:

1. **Authentication Errors**:

   - Verify `AZURE_STORAGE_ACCOUNT_KEY` is set correctly
   - Check that the storage account name in URL matches your actual account

2. **Permission Errors**:

   - Ensure the container exists in your Azure storage account
   - Verify your account key has the necessary permissions

3. **Network Errors**:

   - Check internet connectivity
   - Verify Azure storage account URL is correct

4. **Compilation Errors**:
   - Make sure all dependencies are added to Cargo.toml
   - Check that async/await syntax is used correctly

**Testing Commands**:

```bash
# Check compilation
cargo check

# Run unit tests
cargo test azure

# Test with actual Azure storage (requires setup)
AZURE_STORAGE_ACCOUNT_KEY="your_key" cargo test --test integration
```

## Next Steps

Once this task is complete, proceed to **Task 07: Sync Manager** where we'll implement the core synchronization logic that uses this Azure storage client.

## Rust Learning Notes

**Key Concepts Introduced**:

- **HTTP Clients**: Using `reqwest` for HTTP requests
- **Async Programming**: Working with async/await and `tokio::fs`
- **Authentication**: Implementing HMAC-SHA256 for Azure authentication
- **Error Mapping**: Converting different error types into custom errors
- **Base64 Encoding**: Using base64 for authentication signatures

**Questions to Research**:

1. How does Azure Blob Storage authentication work?
2. What's the difference between `tokio::fs` and `std::fs`?
3. How does HMAC-SHA256 authentication ensure security?
4. What are the best practices for handling HTTP client errors in Rust?
