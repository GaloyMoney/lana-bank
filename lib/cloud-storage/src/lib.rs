pub mod config;
pub mod error;

use config::StorageConfig;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        delete::DeleteObjectRequest,
        list::ListObjectsRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
    sign::SignedURLOptions,
};

use error::*;

const LINK_DURATION_IN_SECS: u64 = 60 * 5;

#[derive(Debug, Clone)]
pub struct LocationInStorage<'a> {
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

    pub fn bucket_name(&self) -> &str {
        &self.config.bucket_name
    }

    pub fn storage_identifier(&self) -> String {
        format!("gcp:{}", self.config.bucket_name)
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
        let bucket = self.bucket_name();
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

    pub async fn remove(&self, location: LocationInStorage<'_>) -> Result<(), StorageError> {
        let bucket = self.config.bucket_name.clone();
        let object_name = self.path_with_prefix(location.path_in_bucket);

        let req = DeleteObjectRequest {
            bucket,
            object: object_name,
            ..Default::default()
        };

        self.client().await?.delete_object(&req).await?;
        Ok(())
    }

    pub async fn generate_download_link(
        &self,
        location: impl Into<LocationInStorage<'_>>,
    ) -> Result<String, StorageError> {
        let location = location.into();

        let object_name = self.path_with_prefix(location.path_in_bucket);

        let opts = SignedURLOptions {
            expires: std::time::Duration::new(LINK_DURATION_IN_SECS, 0),
            ..Default::default()
        };

        let signed_url = self
            .client()
            .await?
            .signed_url(&self.config.bucket_name, &object_name, None, None, opts)
            .await?;

        Ok(signed_url)
    }

    pub async fn _list(&self, filter_prefix: String) -> anyhow::Result<Vec<String>> {
        let full_prefix = self.path_with_prefix(&filter_prefix);
        let bucket = self.bucket_name();

        let req = ListObjectsRequest {
            bucket: bucket.to_owned(),
            prefix: Some(full_prefix),
            ..Default::default()
        };

        let result =
            self.client().await?.list_objects(&req).await.map_err(|e| {
                anyhow::anyhow!("Error listing objects from bucket {}: {e}", bucket)
            })?;

        let mut filenames = Vec::new();
        if let Some(items) = result.items {
            for item in items {
                // `item.name` is the full path/key in the bucket
                filenames.push(item.name);
            }
        }

        Ok(filenames)
    }
}
