pub mod config;
mod error;

pub use error::StorageError;

use config::StorageConfig;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        delete::DeleteObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
    sign::SignedURLOptions,
};

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

    async fn client(&self) -> Result<Client, StorageError> {
        let config = ClientConfig::default().with_auth().await?;
        let client = Client::new(config);
        Ok(client)
    }

    pub fn bucket_name(&self) -> &str {
        &self.config.bucket_name
    }

    fn path_with_prefix(&self, path: &str) -> String {
        format!("{}/{}", self.config.root_folder, path)
    }

    pub async fn upload(
        &self,
        file: Vec<u8>,
        path_in_bucket: &str,
        mime_type: &str,
    ) -> Result<(), StorageError> {
        let client = self.client().await?;
        let bucket = self.bucket_name();
        let object_name = self.path_with_prefix(path_in_bucket);

        let mut media = Media::new(object_name);
        media.content_type = mime_type.to_owned().into();
        let upload_type = UploadType::Simple(media);

        let req = UploadObjectRequest {
            bucket: bucket.to_string(),
            ..Default::default()
        };
        client.upload_object(&req, file, &upload_type).await?;

        Ok(())
    }

    pub async fn remove(&self, location: LocationInCloud<'_>) -> Result<(), StorageError> {
        let client = self.client().await?;
        let bucket = location.bucket;
        let object_name = self.path_with_prefix(location.path_in_bucket);

        let req = DeleteObjectRequest {
            bucket: bucket.to_owned(),
            object: object_name,
            ..Default::default()
        };

        client.delete_object(&req).await?;
        Ok(())
    }

    pub async fn generate_download_link(
        &self,
        location: impl Into<LocationInCloud<'_>>,
    ) -> Result<String, StorageError> {
        let location = location.into();

        let client = self.client().await?;
        let bucket = location.bucket;
        let object_name = self.path_with_prefix(location.path_in_bucket);

        let opts = SignedURLOptions {
            expires: std::time::Duration::new(LINK_DURATION_IN_SECS, 0),
            ..Default::default()
        };

        let signed_url = client
            .signed_url(bucket, &object_name, None, None, opts)
            .await?;

        Ok(signed_url)
    }
}
