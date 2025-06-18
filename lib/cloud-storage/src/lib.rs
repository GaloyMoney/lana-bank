mod client;
pub mod config;
pub mod error;

use client::{GcpClient, LocalClient, StorageClient};
use config::StorageConfig;
use error::*;

#[derive(Debug, Clone)]
pub struct LocationInCloud<'a> {
    pub bucket: &'a str,
    pub path_in_bucket: &'a str,
}

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

    async fn client(&self) -> Result<Box<dyn StorageClient>, StorageError> {
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

    pub async fn upload(
        &self,
        file: Vec<u8>,
        path_in_bucket: &str,
        mime_type: &str,
    ) -> Result<(), StorageError> {
        self.client()
            .await?
            .upload(file, path_in_bucket, mime_type)
            .await?;
        Ok(())
    }

    pub async fn remove(&self, location: LocationInCloud<'_>) -> Result<(), StorageError> {
        self.client()
            .await?
            .remove(location.bucket, location.path_in_bucket)
            .await?;
        Ok(())
    }

    pub async fn generate_download_link(
        &self,
        location: impl Into<LocationInCloud<'_>>,
    ) -> Result<String, StorageError> {
        let location = location.into();

        let link = self
            .client()
            .await?
            .generate_download_link(location.bucket, location.path_in_bucket)
            .await?;

        Ok(link)
    }
}
