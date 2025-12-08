use async_trait::async_trait;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        delete::DeleteObjectRequest,
        upload::{Media, UploadObjectRequest, UploadType},
    },
    sign::SignedURLOptions,
};
use serde::{Deserialize, Serialize};

use super::{StorageClientError, r#trait::StorageClient};

const LINK_DURATION_IN_SECS: u64 = 60 * 5;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GcpConfig {
    #[serde(default)]
    pub bucket_name: String,
}

#[derive(Clone)]
pub struct GcpClient {
    client: Client,
    config: GcpConfig,
}

impl GcpClient {
    pub async fn init(config: &GcpConfig) -> Result<Self, StorageClientError> {
        let client_config = ClientConfig::default().with_auth().await?;
        Ok(GcpClient {
            client: Client::new(client_config),
            config: config.clone(),
        })
    }

    pub fn bucket_name(&self) -> &str {
        &self.config.bucket_name
    }
}

#[async_trait]
impl StorageClient for GcpClient {
    async fn upload(
        &self,
        file: Vec<u8>,
        path_in_bucket: &str,
        mime_type: &str,
    ) -> Result<(), StorageClientError> {
        let bucket = self.bucket_name();

        let mut media = Media::new(path_in_bucket.to_string());
        media.content_type = mime_type.to_owned().into();
        let upload_type = UploadType::Simple(media);

        let req = UploadObjectRequest {
            bucket: bucket.to_owned(),
            ..Default::default()
        };
        self.client.upload_object(&req, file, &upload_type).await?;
        Ok(())
    }

    async fn remove<'a>(
        &self,
        location_in_storage: super::r#trait::LocationInStorage<'a>,
    ) -> Result<(), StorageClientError> {
        let req = DeleteObjectRequest {
            bucket: self.bucket_name().to_owned(),
            object: location_in_storage.path.to_string(),
            ..Default::default()
        };
        self.client.delete_object(&req).await?;
        Ok(())
    }

    async fn generate_download_link<'a>(
        &self,
        location_in_storage: super::r#trait::LocationInStorage<'a>,
    ) -> Result<String, StorageClientError> {
        let opts = SignedURLOptions {
            expires: std::time::Duration::new(LINK_DURATION_IN_SECS, 0),
            ..Default::default()
        };

        let signed_url = self
            .client
            .signed_url(
                self.bucket_name(),
                location_in_storage.path,
                None,
                None,
                opts,
            )
            .await?;

        Ok(signed_url)
    }
}
