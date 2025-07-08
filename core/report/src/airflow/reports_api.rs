use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::config::AirflowConfig;
use crate::error::ReportError;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Clone)]
pub struct ReportsApiClient {
    client: Client,
    base_url: String,
}

impl ReportsApiClient {
    pub fn new(config: AirflowConfig) -> Self {
        let base_url = format!("http://{}:{}", config.host, config.port);

        Self {
            client: Client::new(),
            base_url,
        }
    }

    #[tracing::instrument(name = "reports_api.health_check", skip(self))]
    pub async fn health_check(&self) -> Result<HealthResponse, ReportError> {
        let url = format!("{}/api/v1/health", self.base_url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let health: HealthResponse = response.json().await?;
            Ok(health)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::Sqlx(sqlx::Error::Protocol(format!(
                "Health check failed with status {}: {}", status, text
            ))))
        }
    }
}
