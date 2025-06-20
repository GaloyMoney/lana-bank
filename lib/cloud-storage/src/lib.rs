mod client;
pub mod config;
pub mod error;

pub use client::LocationInStorage;
use client::{GcpClient, LocalClient, StorageClient};
use config::StorageConfig;
use error::*;

#[derive(Clone)]
pub struct Storage {
    config: StorageConfig,
}

impl Storage {
    pub fn new(config: &StorageConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn client(&self) -> Result<Box<dyn StorageClient>, StorageError> {
        match &self.config {
            StorageConfig::Gcp(gcp_config) => {
                let client = GcpClient::init(gcp_config).await?;
                Ok(Box::new(client))
            }
            StorageConfig::Local(local_config) => {
                let client = LocalClient::new(local_config);
                Ok(Box::new(client))
            }
        }
    }

    pub fn bucket_name(&self) -> String {
        match &self.config {
            StorageConfig::Gcp(gcp_config) => gcp_config.bucket_name.clone(),
            // todo: check if this is required
            StorageConfig::Local(local_config) => local_config.root_folder.display().to_string(),
        }
    }
}
