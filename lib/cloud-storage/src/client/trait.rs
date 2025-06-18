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
    async fn remove(&self, path_in_storage: &str) -> Result<(), StorageClientError>;
    async fn generate_download_link(
        &self,
        path_in_storage: &str,
    ) -> Result<String, StorageClientError>;
}
