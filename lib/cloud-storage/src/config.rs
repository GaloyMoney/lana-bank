use serde::{Deserialize, Serialize};

use super::client::{GcpConfig, LocalConfig};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "provider", rename_all = "lowercase")]
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
    pub fn new_gcp(bucket_name: String, root_folder: String) -> Self {
        StorageConfig::Gcp(GcpConfig {
            bucket_name,
            root_folder,
        })
    }

    pub fn new_local(root_folder: String) -> Self {
        StorageConfig::Local(LocalConfig {
            root_folder: root_folder.into(),
        })
    }

    pub fn identifier(&self) -> String {
        match self {
            StorageConfig::Gcp(gcp_config) => {
                format!("gcp:{}:{}", gcp_config.bucket_name, gcp_config.root_folder)
            }
            StorageConfig::Local(local_config) => {
                format!("local:{}", local_config.root_folder.display())
            }
        }
    }
}
