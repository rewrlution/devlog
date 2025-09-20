# Configuration Management Design

## Overview

The sync feature requires a configuration system to manage cloud provider credentials and settings. This design outlines how to parse and manage `config.toml` files securely and flexibly.

## Configuration File Location

The configuration file should be located at:

- Primary: `~/.devlog/config.toml`
- Fallback: `$PWD/.devlog/config.toml` (project-specific config)

## Configuration File Format

### Example `config.toml`

```toml
[sync]
provider = "azure"  # or "aws"

[sync.azure]
connection_string = "DefaultEndpointsProtocol=https;AccountName=myaccount;AccountKey=mykey;EndpointSuffix=core.windows.net"
container_name = "devlog-entries"

[sync.aws]
bucket_name = "my-devlog-bucket"
region = "us-west-2"
# AWS credentials can be provided via:
# 1. Environment variables (AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY)
# 2. AWS credentials file (~/.aws/credentials)
# 3. IAM roles (for EC2/Lambda)

[general]
# Future: other general settings
default_editor = "vim"
entries_path = ".devlog/entries"
```

## Configuration Structure

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DevlogConfig {
    pub sync: Option<SyncConfig>,
    pub general: Option<GeneralConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncConfig {
    pub provider: CloudProvider,
    pub azure: Option<AzureConfig>,
    pub aws: Option<AwsConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    Azure,
    Aws,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AzureConfig {
    pub connection_string: String,
    pub container_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AwsConfig {
    pub bucket_name: String,
    pub region: String,
    // AWS credentials are handled by the AWS SDK:
    // - Environment variables
    // - ~/.aws/credentials
    // - IAM roles
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    pub default_editor: Option<String>,
    pub entries_path: Option<String>,
}
```

## Configuration Manager

```rust
use std::path::{Path, PathBuf};
use color_eyre::Result;

pub struct ConfigManager {
    config_path: PathBuf,
    config: DevlogConfig,
}

impl ConfigManager {
    /// Load configuration from the standard locations
    pub fn load() -> Result<Self> {
        let config_path = Self::find_config_file()?;
        let config = Self::parse_config(&config_path)?;

        Ok(ConfigManager {
            config_path,
            config,
        })
    }

    /// Find the configuration file in standard locations
    fn find_config_file() -> Result<PathBuf> {
        // Try user config first
        if let Some(home_dir) = dirs::home_dir() {
            let user_config = home_dir.join(".devlog").join("config.toml");
            if user_config.exists() {
                return Ok(user_config);
            }
        }

        // Try project config
        let project_config = PathBuf::from(".devlog").join("config.toml");
        if project_config.exists() {
            return Ok(project_config);
        }

        Err(ConfigError::NotFound.into())
    }

    /// Parse configuration file
    fn parse_config(path: &Path) -> Result<DevlogConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: DevlogConfig = toml::from_str(&content)?;
        Self::validate_config(&config)?;
        Ok(config)
    }

    /// Validate configuration
    fn validate_config(config: &DevlogConfig) -> Result<()> {
        if let Some(sync_config) = &config.sync {
            match sync_config.provider {
                CloudProvider::Azure => {
                    if sync_config.azure.is_none() {
                        return Err(ConfigError::MissingAzureConfig.into());
                    }
                }
                CloudProvider::Aws => {
                    if sync_config.aws.is_none() {
                        return Err(ConfigError::MissingAwsConfig.into());
                    }
                }
            }
        }
        Ok(())
    }

    /// Create a default configuration file
    pub fn create_default_config(provider: CloudProvider) -> Result<()> {
        let config_dir = dirs::home_dir()
            .ok_or(ConfigError::NoHomeDirectory)?
            .join(".devlog");

        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let default_config = Self::default_config_for_provider(provider);

        let toml_content = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, toml_content)?;

        println!("Created default config at: {}", config_path.display());
        println!("Please edit the configuration file to add your credentials.");

        Ok(())
    }

    fn default_config_for_provider(provider: CloudProvider) -> DevlogConfig {
        match provider {
            CloudProvider::Azure => DevlogConfig {
                sync: Some(SyncConfig {
                    provider,
                    azure: Some(AzureConfig {
                        connection_string: "REPLACE_WITH_YOUR_CONNECTION_STRING".to_string(),
                        container_name: "devlog-entries".to_string(),
                    }),
                    aws: None,
                }),
                general: Some(GeneralConfig {
                    default_editor: Some("vim".to_string()),
                    entries_path: Some(".devlog/entries".to_string()),
                }),
            },
            CloudProvider::Aws => DevlogConfig {
                sync: Some(SyncConfig {
                    provider,
                    azure: None,
                    aws: Some(AwsConfig {
                        bucket_name: "REPLACE_WITH_YOUR_BUCKET_NAME".to_string(),
                        region: "us-west-2".to_string(),
                    }),
                }),
                general: Some(GeneralConfig {
                    default_editor: Some("vim".to_string()),
                    entries_path: Some(".devlog/entries".to_string()),
                }),
            },
        }
    }

    /// Get sync configuration
    pub fn sync_config(&self) -> Option<&SyncConfig> {
        self.config.sync.as_ref()
    }

    /// Get general configuration
    pub fn general_config(&self) -> Option<&GeneralConfig> {
        self.config.general.as_ref()
    }
}
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found")]
    NotFound,

    #[error("Missing Azure configuration")]
    MissingAzureConfig,

    #[error("Missing AWS configuration")]
    MissingAwsConfig,

    #[error("Invalid TOML format: {0}")]
    InvalidToml(#[from] toml::de::Error),

    #[error("No home directory found")]
    NoHomeDirectory,

    #[error("Invalid Azure connection string")]
    InvalidAzureConnectionString,

    #[error("Invalid AWS region: {0}")]
    InvalidAwsRegion(String),
}
```

## Environment Variable Support

For CI/CD and container deployments, support environment variable overrides:

```rust
impl ConfigManager {
    /// Load configuration with environment variable overrides
    pub fn load_with_env_overrides() -> Result<Self> {
        let mut config_manager = Self::load()?;
        config_manager.apply_env_overrides();
        Ok(config_manager)
    }

    fn apply_env_overrides(&mut self) {
        // Azure overrides
        if let Ok(azure_conn_str) = std::env::var("DEVLOG_AZURE_CONNECTION_STRING") {
            if let Some(sync_config) = &mut self.config.sync {
                if let Some(azure_config) = &mut sync_config.azure {
                    azure_config.connection_string = azure_conn_str;
                }
            }
        }

        // AWS overrides (AWS SDK handles AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY automatically)
        if let Ok(aws_bucket) = std::env::var("DEVLOG_AWS_BUCKET_NAME") {
            if let Some(sync_config) = &mut self.config.sync {
                if let Some(aws_config) = &mut sync_config.aws {
                    aws_config.bucket_name = aws_bucket;
                }
            }
        }

        if let Ok(aws_region) = std::env::var("DEVLOG_AWS_REGION") {
            if let Some(sync_config) = &mut self.config.sync {
                if let Some(aws_config) = &mut sync_config.aws {
                    aws_config.region = aws_region;
                }
            }
        }
    }
}
```

## CLI Integration

Add configuration-related commands to the CLI:

```rust
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Initialize configuration for sync
    Init {
        /// Cloud provider to configure
        #[arg(value_enum)]
        provider: CloudProvider,
    },
    /// Show current configuration
    Show,
    /// Validate configuration
    Validate,
}
```

## Security Best Practices

1. **Never commit config files with real credentials**
2. **Add `.devlog/config.toml` to `.gitignore`**
3. **Support environment variables for containerized deployments**
4. **Validate configuration before using it**
5. **Clear error messages for configuration issues**
6. **Use secure default permissions for config files (600)**
