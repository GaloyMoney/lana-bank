#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;
pub mod reports_api_client;

pub use config::AirflowConfig;
pub use error::AirflowError;
use reports_api_client::ReportsApiClient;

#[derive(Clone)]
pub struct Airflow {
    reports: ReportsApiClient,
}

impl Airflow {
    pub fn new(config: AirflowConfig) -> Self {
        Self {
            reports: ReportsApiClient::new(config),
        }
    }

    pub fn reports(&self) -> &ReportsApiClient {
        &self.reports
    }
}
