use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::config::AirflowConfig;
use crate::error::ReportError;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportGenerateResponse {
    pub run_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub extension: String,
    pub path_in_bucket: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub norm: String,
    pub files: Vec<File>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReportRunState {
    Queued,
    Running,
    Success,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ReportRunType {
    Scheduled,
    Manual,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Run {
    pub run_id: String,
    pub execution_date: DateTime<Utc>,
    pub state: ReportRunState,
    pub run_type: ReportRunType,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub reports: Vec<Report>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunWithoutReports {
    pub run_id: String,
    pub execution_date: DateTime<Utc>,
    pub state: ReportRunState,
    pub run_type: ReportRunType,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
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

    #[tracing::instrument(name = "reports_api.list_runs", skip(self))]
    pub async fn list_runs(
        &self,
        limit: Option<u32>,
        after: Option<String>,
    ) -> Result<Vec<RunWithoutReports>, ReportError> {
        let mut url = format!("{}/api/v1/reports", self.base_url);
        let mut query_params = Vec::new();

        if let Some(limit) = limit {
            query_params.push(format!("limit={limit}"));
        }
        if let Some(after) = after {
            query_params.push(format!("after={after}"));
        }

        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let runs: Vec<RunWithoutReports> = response.json().await?;
            Ok(runs)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(ReportError::ApiError(format!(
                "Failed to list runs with status {status}: {text}"
            )))
        }
    }

    #[tracing::instrument(name = "reports_api.get_run", skip(self))]
    pub async fn get_run(&self, run_id: &str) -> Result<Option<Run>, ReportError> {
        let url = format!("{}/api/v1/report/{}", self.base_url, run_id);

        let response = self.client.get(&url).send().await?;

        match response.status().as_u16() {
            200 => {
                let run: Run = response.json().await?;
                Ok(Some(run))
            }
            404 => Ok(None),
            status_code => {
                let text = response.text().await.unwrap_or_default();
                Err(ReportError::ApiError(format!(
                    "Failed to get run {run_id} with status {status_code}: {text}"
                )))
            }
        }
    }

    #[tracing::instrument(name = "reports_api.generate_report", skip(self))]
    pub async fn generate_report(&self) -> Result<ReportGenerateResponse, ReportError> {
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
}

impl From<ReportRunState> for crate::report_run::ReportRunState {
    fn from(state: ReportRunState) -> Self {
        match state {
            ReportRunState::Queued => crate::report_run::ReportRunState::Queued,
            ReportRunState::Running => crate::report_run::ReportRunState::Running,
            ReportRunState::Success => crate::report_run::ReportRunState::Success,
            ReportRunState::Failed => crate::report_run::ReportRunState::Failed,
        }
    }
}

impl From<crate::report_run::ReportRunState> for ReportRunState {
    fn from(state: crate::report_run::ReportRunState) -> Self {
        match state {
            crate::report_run::ReportRunState::Queued => ReportRunState::Queued,
            crate::report_run::ReportRunState::Running => ReportRunState::Running,
            crate::report_run::ReportRunState::Success => ReportRunState::Success,
            crate::report_run::ReportRunState::Failed => ReportRunState::Failed,
        }
    }
}

impl From<ReportRunType> for crate::report_run::ReportRunType {
    fn from(run_type: ReportRunType) -> Self {
        match run_type {
            ReportRunType::Scheduled => crate::report_run::ReportRunType::Scheduled,
            ReportRunType::Manual => crate::report_run::ReportRunType::Manual,
        }
    }
}

impl From<File> for crate::report::File {
    fn from(file: File) -> Self {
        crate::report::File {
            extension: file.extension,
            path_in_bucket: file.path_in_bucket,
        }
    }
}
