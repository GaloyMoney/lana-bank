use serde::{Deserialize, Serialize};

use crate::service_account::ServiceAccountConfig;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ReportConfig {
    #[serde(default)]
    pub dataform_repo: String,
    #[serde(default)]
    pub dataform_output_dataset: String,
    #[serde(default)]
    pub dataform_release_config: String,

    #[serde(skip)]
    pub service_account: Option<ServiceAccountConfig>,
}

impl ReportConfig {
    pub fn new_dev_mode(
        name_prefix: String,
        service_account: ServiceAccountConfig,
    ) -> ReportConfig {
        Self {
            dataform_repo: format!("{}-repo", name_prefix),
            dataform_output_dataset: format!("dataform_{}", name_prefix),
            dataform_release_config: format!("{}-release", name_prefix),
            service_account: Some(service_account),
        }
    }

    pub(super) fn service_account(&self) -> &ServiceAccountConfig {
        self.service_account
            .as_ref()
            .expect("Service Account not set")
    }
}
