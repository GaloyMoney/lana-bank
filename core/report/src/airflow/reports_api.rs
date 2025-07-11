use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::config::AirflowConfig;
use crate::error::ReportError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportGenerateResponse {
    pub run_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunType {
    Scheduled,
    ApiTriggered,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LastRun {
    pub run_type: RunType,
    pub run_started_at: Option<DateTime<Utc>>,
    pub status: String,
    pub logs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DagRunStatusResponse {
    pub running: bool,
    pub run_type: Option<RunType>,
    pub run_started_at: Option<DateTime<Utc>>,
    pub logs: Option<String>,
    pub last_run: Option<LastRun>,
    pub error: Option<String>,
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

#[derive(Clone)]
pub struct ReportsApiClient {
    client: Client,
    base_url: String,
}

impl ReportsApiClient {
    pub fn new(config: AirflowConfig) -> Self {
        Self {
            client: Client::new(),
            base_url: config.uri,
        }
    }

    #[tracing::instrument(name = "reports_api.get_report_dates", skip(self))]
    pub async fn get_report_dates(&self) -> Result<Vec<NaiveDate>, ReportError> {
        let url = format!("{}/api/v1/reports/dates", self.base_url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let date_strings: Vec<String> = response.json().await?;
            let dates = date_strings
                .into_iter()
                .map(|date_str| {
                    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                        .map_err(|e| ReportError::ApiError(format!(
                            "Failed to parse date '{}': {}", date_str, e
                        )))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(dates)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::ApiError(format!(
                "Failed to get report dates with status {status}: {text}"
            )))
        }
    }

    #[tracing::instrument(name = "reports_api.get_reports_by_date", skip(self))]
    pub async fn get_reports_by_date(&self, date: &str) -> Result<Vec<String>, ReportError> {
        let url = format!("{}/api/v1/reports/date/{}", self.base_url, date);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let report_uris: Vec<String> = response.json().await?;
            Ok(report_uris)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::ApiError(format!(
                "Failed to get reports for date {date} with status {status}: {text}"
            )))
        }
    }

    #[tracing::instrument(name = "reports_api.generate_todays_report", skip(self))]
    pub async fn generate_todays_report(&self) -> Result<ReportGenerateResponse, ReportError> {
        let url = format!("{}/api/v1/reports/generate", self.base_url);

        let response = self.client.post(&url).send().await?;

        if response.status().is_success() {
            let generate_response: ReportGenerateResponse = response.json().await?;
            Ok(generate_response)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::ApiError(format!(
                "Failed to generate reports with status {status}: {text}"
            )))
        }
    }

    #[tracing::instrument(name = "reports_api.get_generation_status", skip(self))]
    pub async fn get_generation_status(&self) -> Result<DagRunStatusResponse, ReportError> {
        let url = format!("{}/api/v1/reports/status", self.base_url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let status_response: DagRunStatusResponse = response.json().await?;
            Ok(status_response)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::ApiError(format!(
                "Failed to get generation status with status {status}: {text}"
            )))
        }
    }
}
