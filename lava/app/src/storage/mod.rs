pub mod config;
mod error;

pub use error::StorageError;

use cloud_storage::{ListRequest, Object};
use config::StorageConfig;
use futures::TryStreamExt;

const LINK_DURATION_IN_SECS: u32 = 60 * 5;

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
        Object::create(
            &self.config.bucket_name,
            file,
            &self.path_with_prefix(path_in_bucket),
            mime_type,
        )
        .await?;

        Ok(())
    }

    pub async fn remove(&self, location: LocationInCloud<'_>) -> Result<(), StorageError> {
        Object::delete(
            &self.config.bucket_name,
            &self.path_with_prefix(location.path_in_bucket),
        )
        .await?;

        Ok(())
    }

    pub async fn generate_download_link(
        &self,
        location: impl Into<LocationInCloud<'_>>,
    ) -> Result<String, StorageError> {
        let location = location.into();
        Ok(Object::read(
            location.bucket,
            &self.path_with_prefix(location.path_in_bucket),
        )
        .await?
        .download_url(LINK_DURATION_IN_SECS)?)
    }

    pub async fn _list(&self, filter_prefix: String) -> anyhow::Result<Vec<String>> {
        let full_prefix = self.path_with_prefix(&filter_prefix);
        let mut filenames = Vec::new();
        let stream = Object::list(
            &self.config.bucket_name,
            ListRequest {
                prefix: Some(full_prefix.clone()),
                ..Default::default()
            },
        )
        .await?;

        let mut stream = Box::pin(stream.into_stream());

        while let Some(result) = stream.try_next().await? {
            for item in result.items {
                if let Some(stripped) = item.name.strip_prefix(&self.path_with_prefix("")) {
                    filenames.push(stripped.trim_start_matches('/').to_string());
                }
            }
        }

        Ok(filenames)
    }
}
