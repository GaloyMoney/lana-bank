use async_trait::async_trait;
use google_cloud_auth::signer::Signer;
use google_cloud_storage::{
    builder::storage::SignedUrlBuilder,
    client::{Storage, StorageControl},
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
    storage: Storage,
    control: StorageControl,
    signer: Signer,
    config: GcpConfig,
}

impl GcpClient {
    pub async fn init(config: &GcpConfig) -> Result<Self, StorageClientError> {
        let storage = Storage::builder()
            .build()
            .await
            .map_err(|e| StorageClientError::Init(Box::new(e)))?;
        let control = StorageControl::builder()
            .build()
            .await
            .map_err(|e| StorageClientError::Init(Box::new(e)))?;
        let signer = google_cloud_auth::credentials::Builder::default()
            .build_signer()
            .map_err(|e| StorageClientError::Init(Box::new(e)))?;
        Ok(GcpClient {
            storage,
            control,
            signer,
            config: config.clone(),
        })
    }

    fn bucket_path(&self) -> String {
        format!("projects/_/buckets/{}", self.config.bucket_name)
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
        let payload = bytes::Bytes::from(file);
        self.storage
            .write_object(self.bucket_path(), path_in_bucket, payload)
            .set_content_type(mime_type)
            .send_unbuffered()
            .await?;
        Ok(())
    }

    async fn remove<'a>(
        &self,
        location_in_storage: super::r#trait::LocationInStorage<'a>,
    ) -> Result<(), StorageClientError> {
        self.control
            .delete_object()
            .set_bucket(self.bucket_path())
            .set_object(location_in_storage.path)
            .send()
            .await?;
        Ok(())
    }

    async fn generate_download_link<'a>(
        &self,
        location_in_storage: super::r#trait::LocationInStorage<'a>,
    ) -> Result<String, StorageClientError> {
        let signed_url = SignedUrlBuilder::for_object(self.bucket_path(), location_in_storage.path)
            .with_expiration(std::time::Duration::new(LINK_DURATION_IN_SECS, 0))
            .sign_with(&self.signer)
            .await?;
        Ok(signed_url)
    }
}
