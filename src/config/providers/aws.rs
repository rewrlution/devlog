use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    pub bucket: String,
    pub region: String,
}

impl AwsConfig {
    pub fn new(bucket: String, region: String) -> Self {
        Self {
            bucket,
            region,
        }
    }

    /// Validate AWS configuration
    pub fn validate(&self) -> color_eyre::Result<()> {
        if self.bucket.is_empty() {
            return Err(color_eyre::eyre::eyre!("AWS bucket name cannot be empty"));
        }
        
        if self.region.is_empty() {
            return Err(color_eyre::eyre::eyre!("AWS region cannot be empty"));
        }
        
        // Additional validation can be added here
        // For example, testing the connection to AWS
        
        Ok(())
    }
}