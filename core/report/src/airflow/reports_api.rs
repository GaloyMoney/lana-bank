use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::config::AirflowConfig;
use crate::error::ReportError;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportGenerateResponse {
    pub run_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DagRunStatusResponse {
    pub run_id: String,
    pub state: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub execution_date: DateTime<Utc>,
    pub dag_id: String,
    pub task_instances: Vec<TaskInstanceInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskInstanceInfo {
    pub task_id: String,
    pub state: String,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub duration: Option<f64>,
    pub log_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportUri {
    pub uri: String,
    pub date: String,
    pub category: String,
    pub name: String,
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
        let url = format!("{}/api/v1/reports/health", self.base_url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let health: HealthResponse = response.json().await?;
            Ok(health)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::Sqlx(sqlx::Error::Protocol(format!(
                "Health check failed with status {status}: {text}"
            ))))
        }
    }

    #[tracing::instrument(name = "reports_api.get_report_dates", skip(self))]
    pub async fn get_report_dates(&self) -> Result<Vec<String>, ReportError> {
        let url = format!("{}/api/v1/reports/dates", self.base_url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let dates: Vec<String> = response.json().await?;
            Ok(dates)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::Sqlx(sqlx::Error::Protocol(format!(
                "Failed to get report dates with status {status}: {text}"
            ))))
        }
    }

    #[tracing::instrument(name = "reports_api.get_reports_by_date", skip(self))]
    pub async fn get_reports_by_date(&self, date: &str) -> Result<Vec<ReportUri>, ReportError> {
        let url = format!("{}/api/v1/reports/date/{}", self.base_url, date);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let report_uris: Vec<String> = response.json().await?;
            let reports = report_uris
                .into_iter()
                .map(|uri| self.parse_report_uri(uri, date))
                .collect();
            Ok(reports)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::Sqlx(sqlx::Error::Protocol(format!(
                "Failed to get reports for date {date} with status {status}: {text}"
            ))))
        }
    }

    #[tracing::instrument(name = "reports_api.generate_reports", skip(self))]
    pub async fn generate_reports(&self) -> Result<ReportGenerateResponse, ReportError> {
        let url = format!("{}/api/v1/reports/generate", self.base_url);

        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            let generate_response: ReportGenerateResponse = response.json().await?;
            Ok(generate_response)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::Sqlx(sqlx::Error::Protocol(format!(
                "Failed to generate reports with status {status}: {text}"
            ))))
        }
    }

    #[tracing::instrument(name = "reports_api.get_generation_status", skip(self))]
    pub async fn get_generation_status(
        &self,
        run_id: &str,
    ) -> Result<DagRunStatusResponse, ReportError> {
        let url = format!("{}/api/v1/reports/status/{}", self.base_url, run_id);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let status_response: DagRunStatusResponse = response.json().await?;
            Ok(status_response)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::Sqlx(sqlx::Error::Protocol(format!(
                "Failed to get generation status for run_id {run_id} with status {status}: {text}"
            ))))
        }
    }

    fn parse_report_uri(&self, uri: String, date: &str) -> ReportUri {
        let parts: Vec<&str> = uri.split('/').collect();
        let (category, name) = if parts.len() >= 4 {
            let category = parts[parts.len() - 2].to_string();
            let name = parts[parts.len() - 1].to_string();
            (category, name)
        } else {
            (
                "unknown".to_string(),
                uri.split('/').next_back().unwrap_or("unknown").to_string(),
            )
        };

        ReportUri {
            uri,
            date: date.to_string(),
            category,
            name,
        }
    }
}
