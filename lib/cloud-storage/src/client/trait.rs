use async_trait::async_trait;

use super::StorageClientError;

#[async_trait]
pub trait StorageClient: Send + 'static {
    async fn upload(
        &self,
        file: Vec<u8>,
        path: &str,
        mime_type: &str,
    ) -> Result<(), StorageClientError>;
    async fn remove(&self, bucket: &str, path_in_bucket: &str) -> Result<(), StorageClientError>;
    async fn generate_download_link(
        &self,
        bucket: &str,
        path_in_bucket: &str,
    ) -> Result<String, StorageClientError>;
}
