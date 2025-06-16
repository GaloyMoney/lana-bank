pub mod config;
pub mod error;

use config::StorageConfig;
use config::StorageProvider;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        delete::DeleteObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
    sign::SignedURLOptions,
};

use error::*;

use std::path::Path;

const LINK_DURATION_IN_SECS: u64 = 60 * 5;

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

    pub fn bucket_name(&self) -> Option<&str> {
        self.config.bucket_name.as_deref()
    }

    fn path_with_prefix(&self, path: &str) -> String {
        format!("{}/{}", self.config.root_folder, path)
    }

    async fn client(&self) -> Result<Client, StorageError> {
        let client_config = ClientConfig::default().with_auth().await?;
        Ok(Client::new(client_config))
    }

    pub async fn upload(
        &self,
        file: Vec<u8>,
        path_in_bucket: &str,
        mime_type: &str,
    ) -> Result<(), StorageError> {
        match self.config.provider {
            StorageProvider::Gcp => {
                let bucket = self
                    .bucket_name()
                    .ok_or(StorageError::MissingBucket)?;
                let object_name = self.path_with_prefix(path_in_bucket);

                let mut media = Media::new(object_name);
                media.content_type = mime_type.to_owned().into();
                let upload_type = UploadType::Simple(media);

                let req = UploadObjectRequest {
                    bucket: bucket.to_string(),
                    ..Default::default()
                };
                self.client()
                    .await?
                    .upload_object(&req, file, &upload_type)
                    .await?;

                Ok(())
            }
            StorageProvider::Local => {
                let path = Path::new(&self.config.root_folder).join(path_in_bucket);
                if let Some(parent) = path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(path, file).await?;
                Ok(())
            }
        }
    }

    pub async fn remove(&self, location: LocationInCloud<'_>) -> Result<(), StorageError> {
        match self.config.provider {
            StorageProvider::Gcp => {
                let bucket = location.bucket;
                let object_name = self.path_with_prefix(location.path_in_bucket);

                let req = DeleteObjectRequest {
                    bucket: bucket.to_owned(),
                    object: object_name,
                    ..Default::default()
                };

                self.client().await?.delete_object(&req).await?;
                Ok(())
            }
            StorageProvider::Local => {
                let path = Path::new(&self.config.root_folder).join(location.path_in_bucket);
                if tokio::fs::remove_file(path).await.is_err() {
                    // ignore errors if file doesn't exist
                }
                Ok(())
            }
        }
    }

    pub async fn generate_download_link(
        &self,
        location: impl Into<LocationInCloud<'_>>,
    ) -> Result<String, StorageError> {
        let location = location.into();
        match self.config.provider {
            StorageProvider::Gcp => {
                let bucket = location.bucket;
                let object_name = self.path_with_prefix(location.path_in_bucket);

                let opts = SignedURLOptions {
                    expires: std::time::Duration::new(LINK_DURATION_IN_SECS, 0),
                    ..Default::default()
                };

                let signed_url = self
                    .client()
                    .await?
                    .signed_url(bucket, &object_name, None, None, opts)
                    .await?;

                Ok(signed_url)
            }
            StorageProvider::Local => {
                let path = Path::new(&self.config.root_folder).join(location.path_in_bucket);
                Ok(format!("file://{}", path.to_string_lossy()))
            }
        }
    }

}
