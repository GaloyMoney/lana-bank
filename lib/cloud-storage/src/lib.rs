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
    client: std::sync::Arc<dyn StorageClient>,
}

impl Storage {
    pub async fn init(config: &StorageConfig) -> Result<Self, StorageError> {
        let client = match config {
            StorageConfig::Gcp(gcp_config) => {
                let client = GcpClient::init(gcp_config).await?;
                std::sync::Arc::new(client) as std::sync::Arc<dyn StorageClient>
            }
            StorageConfig::Local(local_config) => {
                let client = LocalClient::new(local_config);
                std::sync::Arc::new(client) as std::sync::Arc<dyn StorageClient>
            }
        };

        Ok(Self {
            config: config.clone(),
            client,
        })
    }

    pub fn bucket_name(&self) -> String {
        match &self.config {
            StorageConfig::Gcp(gcp_config) => gcp_config.bucket_name.clone(),
            StorageConfig::Local(local_config) => local_config.root_folder.display().to_string(),
        }
    }
}

impl std::ops::Deref for Storage {
    type Target = dyn StorageClient;

    fn deref(&self) -> &Self::Target {
        self.client.as_ref()
    }
}
