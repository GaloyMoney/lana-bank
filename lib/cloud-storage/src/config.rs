use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageProvider {
    Gcp,
    Local,
}

impl Default for StorageProvider {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct StorageConfig {
    #[serde(default)]
    pub provider: StorageProvider,
    #[serde(default)]
    pub root_folder: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bucket_name: Option<String>,
}

impl StorageConfig {
    pub fn new_dev_mode(name_prefix: String) -> StorageConfig {
        Self {
            provider: StorageProvider::Gcp,
            bucket_name: Some(format!("{}-lana-documents", name_prefix)),
            root_folder: name_prefix,
        }
    }
}
