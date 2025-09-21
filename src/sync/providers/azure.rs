use std::path::Path;
use async_trait::async_trait;
use color_eyre::{Result, eyre::eyre};

use crate::sync::{CloudStorage, CloudFile};

/// Azure Blob Storage provider
/// 
/// MVP: Placeholder implementation - will use REST API calls
/// Future: Use official Azure SDK when stable
#[allow(dead_code)]
pub struct AzureProvider {
    connection_string: String,
    container_name: String,
}

impl AzureProvider {
    /// Create a new Azure provider with connection string
    pub fn new(connection_string: &str, container_name: &str) -> Result<Self> {
        if connection_string.is_empty() || container_name.is_empty() {
            return Err(eyre!("Azure connection string and container name cannot be empty"));
        }
        
        Ok(Self {
            connection_string: connection_string.to_string(),
            container_name: container_name.to_string(),
        })
    }
}

#[async_trait]
impl CloudStorage for AzureProvider {
    async fn upload(&self, _local_path: &Path, remote_name: &str) -> Result<()> {
        // MVP: Not implemented yet - use local provider for testing
        return Err(eyre!(
            "Azure upload not yet implemented. File: {} would be uploaded to container: {}", 
            remote_name, 
            self.container_name
        ));
    }
    
    async fn download(&self, remote_name: &str, _local_path: &Path) -> Result<()> {
        // MVP: Not implemented yet - use local provider for testing
        return Err(eyre!(
            "Azure download not yet implemented. File: {} would be downloaded from container: {}", 
            remote_name, 
            self.container_name
        ));
    }
    
    async fn list_files(&self) -> Result<Vec<CloudFile>> {
        // MVP: Not implemented yet - use local provider for testing
        return Err(eyre!(
            "Azure list_files not yet implemented. Would list files from container: {}", 
            self.container_name
        ));
    }
}
