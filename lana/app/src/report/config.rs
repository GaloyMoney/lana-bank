use serde::{Deserialize, Serialize};

use crate::service_account::ServiceAccountConfig;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct ReportConfig {
    #[serde(default)]
    pub dbt_output_dataset: String,
    #[serde(default)]
    pub dev_disable_auto_create: bool,

    #[serde(skip)]
    pub service_account: Option<ServiceAccountConfig>,
}

impl ReportConfig {
    pub fn new_dev_mode(
        name_prefix: String,
        service_account: ServiceAccountConfig,
        dev_disable_auto_create: bool,
    ) -> ReportConfig {
        Self {
            dbt_output_dataset: format!("dbt_{name_prefix}"),
            dev_disable_auto_create,
            service_account: Some(service_account),
        }
    }

    pub(super) fn service_account(&self) -> &ServiceAccountConfig {
        self.service_account
            .as_ref()
            .expect("Service Account not set")
    }
}
