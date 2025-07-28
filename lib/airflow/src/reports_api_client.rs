use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{config::AirflowConfig, error::AirflowError};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportGenerateResponse {
    pub run_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportFile {
    pub extension: String,
    pub path_in_bucket: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub id: String,
    pub name: String,
    pub norm: String,
    pub files: Vec<ReportFile>,
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
pub struct RunData {
    pub run_id: String,
    pub execution_date: DateTime<Utc>,
    pub state: ReportRunState,
    pub run_type: ReportRunType,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub reports: Option<Vec<Report>>,
}

#[derive(Clone)]
pub struct ReportsApiClient {
    http: Client,
    base: Url,
}

impl ReportsApiClient {
    pub fn new(config: AirflowConfig) -> Self {
        Self {
            http: Client::new(),
            base: config.uri,
        }
    }

    fn reports_url(&self) -> Result<Url, AirflowError> {
        Ok(self.base.join("api/v1/reports/")?)
    }

    #[tracing::instrument(name = "airflow.reports_api_client.list_runs", skip(self))]
    pub async fn list_runs(
        &self,
        limit: Option<u32>,
        after: Option<String>,
    ) -> Result<Vec<RunData>, AirflowError> {
        let mut url = self.reports_url()?;

        {
            let mut qp = url.query_pairs_mut();
            if let Some(limit) = limit {
                qp.append_pair("limit", &limit.to_string());
            }
            if let Some(after) = after {
                qp.append_pair("after", &after);
            }
        }

        let response = self.http.get(url).send().await?;

        if !response.status().is_success() {
            return Err(AirflowError::ApiError);
        }

        let runs: Vec<RunData> = response.json().await?;
        Ok(runs)
    }

    #[tracing::instrument(name = "airflow.reports_api_client.get_run", skip(self))]
    pub async fn get_run(&self, run_id: &str) -> Result<Option<RunData>, AirflowError> {
        let url = self.reports_url()?.join(run_id)?;
        let response = self.http.get(url).send().await?;

        if !response.status().is_success() {
            return Err(AirflowError::ApiError);
        }

        let run: RunData = response.json().await?;
        Ok(Some(run))
    }

    #[tracing::instrument(name = "reports_api.generate_report", skip(self))]
    pub async fn generate_report(&self) -> Result<ReportGenerateResponse, AirflowError> {
        let url = self.reports_url()?.join("generate")?;
        let response = self.http.post(url).send().await?;

        if !response.status().is_success() {
            return Err(AirflowError::ApiError);
        }

        let generate_response: ReportGenerateResponse = response.json().await?;
        Ok(generate_response)
    }
}
