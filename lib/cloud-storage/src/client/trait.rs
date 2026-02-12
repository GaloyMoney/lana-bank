use async_trait::async_trait;

use super::StorageClientError;

#[derive(Debug, Clone)]
pub struct LocationInStorage<'a> {
    pub path: &'a str,
}

#[async_trait]
pub(crate) trait StorageClient: Send + Sync {
    async fn upload(
        &self,
        file: Vec<u8>,
        path: &str,
        mime_type: &str,
    ) -> Result<(), StorageClientError>;

    async fn remove<'a>(
        &self,
        location_in_storage: LocationInStorage<'a>,
    ) -> Result<(), StorageClientError>;

    async fn generate_download_link<'a>(
        &self,
        location_in_storage: LocationInStorage<'a>,
    ) -> Result<String, StorageClientError>;
}
