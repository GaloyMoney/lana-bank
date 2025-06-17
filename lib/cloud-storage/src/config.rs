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
        StorageConfig::Gcp(GcpConfig::default())
    }
}

impl StorageConfig {
    pub fn new_gcp_dev_mode(name_prefix: String) -> Self {
        StorageConfig::Gcp(GcpConfig::new_dev_mode(name_prefix))
    }

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
}
