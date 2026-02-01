use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::{StorageClient, error::StorageClientError};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LocalConfig {
    #[serde(default)]
    pub root_folder: PathBuf,
    #[serde(default)]
    pub admin_panel_url: Option<String>,
}

pub struct LocalClient {
    root_folder: PathBuf,
    admin_panel_url: Option<String>,
}

impl LocalClient {
    pub fn new(config: &LocalConfig) -> Self {
        LocalClient {
            root_folder: config.root_folder.clone(),
            admin_panel_url: config.admin_panel_url.clone(),
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
        // Resolve to absolute path so file:// URL works when opened from the browser (e.g. admin panel).
        // With a relative root_folder (e.g. '' in dev), full_path would otherwise be relative.
        let absolute = tokio::fs::canonicalize(&full_path).await?;

        // If admin_panel_url is configured, use the API route to serve files
        // This is primarily for local development where file:// URLs may not work well
        if let Some(ref base_url) = self.admin_panel_url {
            use base64::Engine as _;
            let absolute_str = absolute.to_string_lossy();
            let encoded_path =
                base64::engine::general_purpose::STANDARD.encode(absolute_str.as_bytes());
            Ok(format!(
                "{}/api/local-files/{}",
                base_url.trim_end_matches('/'),
                urlencoding::encode(&encoded_path)
            ))
        } else {
            // Fall back to file:// URL for backwards compatibility
            // This will not open the file and will console error saying "Not allowed to load local resource"
            let url_path = absolute.to_string_lossy().replace('\\', "/");
            Ok(format!("file:///{}", url_path.trim_start_matches('/')))
        }
    }
}
