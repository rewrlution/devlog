use async_trait::async_trait;
use azure_storage::StorageCredentials;
use azure_storage_blobs::prelude::*;
use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use std::path::Path;

use crate::sync::{CloudFile, CloudStorage};

/// Azure Blob Storage provider
pub struct AzureProvider {
    blob_service: BlobServiceClient,
    container_name: String,
}

impl AzureProvider {
    pub fn new(connection_string: &str, container_name: &str) -> Result<Self> {
        // Parse connection string manually
        let mut account_name = String::new();
        let mut account_key = String::new();

        for part in connection_string.split(';') {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "AccountName" => account_name = value.to_string(),
                    "AccountKey" => account_key = value.to_string(),
                    _ => {} // Ignore other parts like DefaultEndpointsProtocol
                }
            }
        }

        if account_name.is_empty() || account_key.is_empty() {
            return Err(eyre!(
                "Invalid Azure connection string: missing AccountName or AccountKey"
            ));
        }

        // Create credentials using the extracted values
        let storage_credentials = StorageCredentials::access_key(account_name.clone(), account_key);
        let blob_service = BlobServiceClient::new(account_name, storage_credentials);

        Ok(AzureProvider {
            blob_service,
            container_name: container_name.to_string(),
        })
    }

    async fn ensure_container_exists(&self) -> Result<()> {
        let container_client = self.blob_service.container_client(&self.container_name);

        // First, try to check if the container exists by trying to get its properties
        match container_client.get_properties().await {
            Ok(_) => {
                // Container exists, nothing to do
                return Ok(());
            }
            Err(_) => {
                // Container doesn't exist, try to create it
            }
        }

        // Try to create the container
        match container_client.create().await {
            Ok(_) => {
                println!("Created Azure container: {}", self.container_name);
                Ok(())
            }
            Err(e) => {
                let error_string = e.to_string();
                // Container might already exist (race condition), which is fine
                if error_string.contains("ContainerAlreadyExists")
                    || error_string.contains("The specified container already exists")
                    || error_string.contains("409")
                {
                    Ok(())
                } else {
                    Err(eyre!("Failed to ensure container exists: {}", e))
                }
            }
        }
    }
}

#[async_trait]
impl CloudStorage for AzureProvider {
    async fn upload(&self, local_path: &Path, remote_name: &str) -> Result<()> {
        self.ensure_container_exists().await?;

        let content = tokio::fs::read(local_path).await?;

        let blob_client = self
            .blob_service
            .container_client(&self.container_name)
            .blob_client(remote_name);

        blob_client
            .put_block_blob(content)
            .content_type("text/markdown")
            .await
            .map_err(|e| eyre!("Failed to upload '{}' to Azure: {}", remote_name, e))?;

        println!("  → Uploaded to Azure: {}", remote_name);
        Ok(())
    }

    async fn download(&self, remote_name: &str, local_path: &Path) -> Result<()> {
        let blob_client = self
            .blob_service
            .container_client(&self.container_name)
            .blob_client(remote_name);

        let response = blob_client
            .get_content()
            .await
            .map_err(|e| eyre!("Failed to download '{}' from Azure: {}", remote_name, e))?;

        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(local_path, &response).await?;

        println!("  ← Downloaded from Azure: {}", remote_name);
        Ok(())
    }

    async fn list_files(&self) -> Result<Vec<CloudFile>> {
        self.ensure_container_exists().await?;

        let container_client = self.blob_service.container_client(&self.container_name);

        let mut files = Vec::new();

        // List blobs in the container
        let mut stream = container_client.list_blobs().into_stream();

        use futures::StreamExt;
        while let Some(response) = stream.next().await {
            let response = response.map_err(|e| eyre!("Failed to list Azure blobs: {}", e))?;

            for blob in response.blobs.blobs() {
                if blob.name.ends_with(".md") {
                    // Handle datetime conversion properly
                    let last_modified: DateTime<Utc> = {
                        // Convert time::OffsetDateTime to chrono::DateTime<Utc>
                        let timestamp = blob.properties.last_modified.unix_timestamp();
                        DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now)
                    };

                    files.push(CloudFile {
                        name: blob.name.clone(),
                        last_modified,
                    });
                }
            }
        }

        Ok(files)
    }
}
