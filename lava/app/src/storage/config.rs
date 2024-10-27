use serde::{Deserialize, Serialize};

use crate::service_account::ServiceAccountConfig;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct StorageConfig {
    #[serde(default)]
    pub root_folder: String,
    #[serde(default)]
    pub bucket_name: String,

    #[serde(skip)]
    pub service_account: Option<ServiceAccountConfig>,
}

impl StorageConfig {
    /// This function only needs to be run in dev mode
    ///
    /// in production, the value is set directly through lava.yml
    pub fn new_dev_mode(
        name_prefix: String,
        service_account: ServiceAccountConfig,
    ) -> StorageConfig {
        Self {
            bucket_name: format!("{}-volcano-documents", name_prefix),
            root_folder: name_prefix,
            service_account: Some(service_account),
        }
    }
}
