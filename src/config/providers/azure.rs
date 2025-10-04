use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub connection_string: String,
    pub container_name: String,
}

impl AzureConfig {
    pub fn new(connection_string: String, container_name: String) -> Self {
        Self {
            connection_string,
            container_name,
        }
    }

    /// Validate Azure configuration
    pub fn validate(&self) -> color_eyre::Result<()> {
        if self.connection_string.is_empty() {
            return Err(color_eyre::eyre::eyre!("Azure connection string cannot be empty"));
        }
        
        if self.container_name.is_empty() {
            return Err(color_eyre::eyre::eyre!("Azure container name cannot be empty"));
        }
        
        // Additional validation can be added here
        // For example, testing the connection to Azure
        
        Ok(())
    }
}