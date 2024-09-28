pub mod config;
mod error;

pub use error::StorageError;

use cloud_storage::{ListRequest, Object};
use config::StorageConfig;
use futures::TryStreamExt;

const LINK_DURATION_IN_SECS: u32 = 60 * 5;

#[derive(Debug, Clone)]
pub struct ReportLocationInCloudWithMeta {
    pub report_name: String,
    pub bucket: String,
    pub path_in_bucket: String,
}

#[derive(Debug, Clone)]
pub struct ReportLocationInCloud {
    pub bucket: String,
    pub path_in_bucket: String,
}

impl From<ReportLocationInCloudWithMeta> for ReportLocationInCloud {
    fn from(meta: ReportLocationInCloudWithMeta) -> Self {
        ReportLocationInCloud {
            bucket: meta.bucket,
            path_in_bucket: meta.path_in_bucket,
        }
    }
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

    pub fn bucket_name(&self) -> String {
        self.config.bucket_name.clone()
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

    pub async fn remove(&self, path_in_bucket: &str) -> Result<(), StorageError> {
        Object::delete(
            &self.config.bucket_name,
            &self.path_with_prefix(path_in_bucket),
        )
        .await?;

        Ok(())
    }

    pub async fn generate_download_link<T>(&self, location: T) -> Result<String, StorageError>
    where
        T: Into<ReportLocationInCloud>,
    {
        let location: ReportLocationInCloud = location.into();

        Ok(Object::read(
            &location.bucket,
            &self.path_with_prefix(&location.path_in_bucket),
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
