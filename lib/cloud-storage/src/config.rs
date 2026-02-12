use serde::{Deserialize, Serialize};

use super::client::{GcpConfig, LocalConfig};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum StorageConfig {
    Gcp(GcpConfig),
    Local(LocalConfig),
}

impl Default for StorageConfig {
    fn default() -> Self {
        StorageConfig::Local(LocalConfig::default())
    }
}

impl StorageConfig {
    pub fn new_gcp(bucket_name: String) -> Self {
        StorageConfig::Gcp(GcpConfig { bucket_name })
    }

    pub fn new_local(root_folder: String, server_url: String, signing_secret: String) -> Self {
        StorageConfig::Local(LocalConfig {
            root_folder: root_folder.into(),
            server_url,
            signing_secret,
        })
    }

    pub fn identifier(&self) -> String {
        match self {
            StorageConfig::Gcp(gcp_config) => {
                format!("gcp:{}", gcp_config.bucket_name)
            }
            StorageConfig::Local(local_config) => {
                format!("local:{}", local_config.root_folder.display())
            }
        }
    }
}
