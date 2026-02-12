use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{StorageClient, error::StorageClientError};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LocalConfig {
    #[serde(default)]
    pub root_folder: PathBuf,
}

pub(crate) struct LocalClient {
    root_folder: PathBuf,
}

impl LocalClient {
    pub(crate) fn new(config: &LocalConfig) -> Self {
        LocalClient {
            root_folder: config.root_folder.clone(),
        }
    }

    fn resolve<P: AsRef<Path>>(&self, relative: P) -> PathBuf {
        self.root_folder.join(relative)
    }
}

#[async_trait::async_trait]
impl StorageClient for LocalClient {
    async fn upload(
        &self,
        file: Vec<u8>,
        path_in_bucket: &str,
        _mime_type: &str,
    ) -> Result<(), StorageClientError> {
        let full_path = self.resolve(path_in_bucket);

        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(full_path, file).await?;
        Ok(())
    }

    async fn remove<'a>(
        &self,
        location_in_storage: super::LocationInStorage<'a>,
    ) -> Result<(), StorageClientError> {
        let full_path = self.resolve(location_in_storage.path);
        tokio::fs::remove_file(full_path).await?;
        Ok(())
    }

    async fn generate_download_link<'a>(
        &self,
        location_in_storage: super::LocationInStorage<'a>,
    ) -> Result<String, StorageClientError> {
        let full_path = self.resolve(location_in_storage.path);
        Ok(format!("file://{}", full_path.to_string_lossy()))
    }
}
