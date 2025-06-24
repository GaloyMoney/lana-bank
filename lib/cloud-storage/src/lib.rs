mod client;
pub mod config;
pub mod error;

use std::sync::Arc;
use tokio::sync::OnceCell;

pub use client::LocationInStorage;
use client::{GcpClient, LocalClient, StorageClient};
use config::StorageConfig;
use error::*;
#[derive(Clone)]
pub struct Storage {
    config: StorageConfig,
    cell: OnceCell<Arc<dyn StorageClient>>,
}

impl Storage {
    pub fn new(config: &StorageConfig) -> Self {
        Self {
            config: config.clone(),
            cell: OnceCell::new(),
        }
    }

    async fn get_client(&self) -> Result<&Arc<dyn StorageClient>, StorageError> {
        self.cell
            .get_or_try_init(|| async {
                let client: Arc<dyn StorageClient> = match self.config {
                    StorageConfig::Gcp(ref gcp_config) => {
                        let gcp = GcpClient::init(gcp_config).await?;
                        Arc::new(gcp)
                    }
                    StorageConfig::Local(ref local_config) => {
                        let local = LocalClient::new(local_config);
                        Arc::new(local)
                    }
                };
                Ok(client)
            })
            .await
    }

    pub fn bucket_name(&self) -> String {
        match &self.config {
            StorageConfig::Gcp(gcp_config) => gcp_config.bucket_name.clone(),
            StorageConfig::Local(local_config) => local_config.root_folder.display().to_string(),
        }
    }

    pub async fn upload(
        &self,
        file: Vec<u8>,
        path: &str,
        mime_type: &str,
    ) -> Result<(), StorageError> {
        let client = self.get_client().await?;
        client.upload(file, path, mime_type).await?;
        Ok(())
    }

    pub async fn remove<'a>(
        &self,
        location_in_storage: LocationInStorage<'a>,
    ) -> Result<(), StorageError> {
        let client = self.get_client().await?;
        client.remove(location_in_storage).await?;
        Ok(())
    }

    pub async fn generate_download_link<'a>(
        &self,
        location_in_storage: LocationInStorage<'a>,
    ) -> Result<String, StorageError> {
        let client = self.get_client().await?;
        let link = client.generate_download_link(location_in_storage).await?;
        Ok(link)
    }

    pub fn identifier(&self) -> String {
        self.config.identifier()
    }
}
