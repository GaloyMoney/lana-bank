use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use tracing_macros::record_error_severity;

use super::{config::DagsterConfig, error::DagsterError};

mod ts_float_seconds_option {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => serializer.serialize_some(&(dt.timestamp() as f64)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<f64> = Option::deserialize(deserializer)?;
        Ok(opt.map(|ts| {
            let secs = ts.trunc() as i64;
            let nsecs = ((ts.fract()) * 1_000_000_000.0) as u32;
            Utc.timestamp_opt(secs, nsecs).single().unwrap_or_default()
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportFile {
    #[serde(rename = "type")]
    pub extension: String,
    pub path_in_bucket: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub name: String,
    pub norm: String,
    pub files: Vec<ReportFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunStatus {
    Queued,
    NotStarted,
    Managed,
    Starting,
    Started,
    Success,
    Failure,
    Cancelling,
    Cancelled,
}

impl RunStatus {
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            RunStatus::Success | RunStatus::Failure | RunStatus::Cancelled
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunTag {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub run_id: String,
    pub status: RunStatus,
    #[serde(with = "ts_float_seconds_option")]
    pub start_time: Option<DateTime<Utc>>,
    pub tags: Vec<RunTag>,
}

impl RunResult {
    pub fn is_scheduled(&self) -> bool {
        self.tags
            .iter()
            .any(|tag| tag.key == "dagster/schedule_name")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Runs {
    pub count: i32,
    pub results: Vec<RunResult>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RunsOrError {
    Runs(Runs),
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileReportsRunsData {
    pub runs_or_error: RunsOrError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReportsRunsResponse {
    pub data: FileReportsRunsData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineSelector {
    pub pipeline_name: String,
    pub repository_location_name: String,
    pub repository_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionParams {
    pub selector: PipelineSelector,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum LaunchPipelineResult {
    LaunchRunSuccess {
        run: Option<LaunchRunDetails>,
    },
    #[serde(other)]
    Error,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchRunDetails {
    pub run_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPipelineData {
    pub launch_pipeline_execution: LaunchPipelineResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchPipelineResponse {
    pub data: LaunchPipelineData,
}

#[derive(Clone)]
pub struct GraphqlClient {
    http: Client,
    url: Url,
}

impl GraphqlClient {
    pub fn new(config: DagsterConfig) -> Self {
        Self {
            http: Client::new(),
            url: config.uri,
        }
    }

    #[record_error_severity]
    #[tracing::instrument(name = "dagster.graphql_client.file_reports_runs", skip(self))]
    pub async fn file_reports_runs(
        &self,
        limit: i32,
        cursor: Option<String>,
    ) -> Result<FileReportsRunsResponse, DagsterError> {
        let query = r#"
query FileReportsRuns($limit: Int!, $cursor: String) {
  runsOrError(
    limit: $limit
    cursor: $cursor
    filter: {pipelineName: "file_reports_generation"}
  ) {
    ... on Runs {
      count
      results {
        runId
        status
        startTime
        tags {
          key
          value
        }
      }
    }
  }
}
"#;

        let variables = if let Some(cursor) = cursor {
            json!({
                "limit": limit,
                "cursor": cursor
            })
        } else {
            json!({
                "limit": limit
            })
        };

        let request = json!({
            "query": query,
            "variables": variables
        });

        let response = self
            .http
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DagsterError::ApiError);
        }

        let response_data: FileReportsRunsResponse = response.json().await?;
        Ok(response_data)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "dagster.graphql_client.get_logs_for_run", skip(self))]
    pub async fn get_logs_for_run(&self, run_id: &str) -> Result<Vec<Report>, DagsterError> {
        let query = r#"
query GetLogsForRun($runId: ID!) {
  logsForRun(runId: $runId) {
    ... on EventConnection {
      events {
        __typename
        ... on MaterializationEvent {
          metadataEntries {
            __typename
            ... on JsonMetadataEntry {
              jsonString
            }
          }
        }
      }
    }
  }
}
"#;

        let variables = json!({
            "runId": run_id
        });

        let request = json!({
            "query": query,
            "variables": variables
        });

        let response = self
            .http
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DagsterError::ApiError);
        }

        let response_data: serde_json::Value = response.json().await?;

        let mut reports = Vec::new();
        let empty = vec![];

        let events = response_data["data"]["logsForRun"]["events"]
            .as_array()
            .unwrap_or(&empty);

        for event in events {
            if event["__typename"] == "MaterializationEvent" {
                let entries = event["metadataEntries"].as_array().unwrap_or(&empty);
                for entry in entries {
                    if let Some(json_string) = entry["jsonString"]
                        .as_str()
                        .filter(|_| entry["__typename"] == "JsonMetadataEntry")
                    {
                        let parsed: Report = serde_json::from_str(json_string)?;
                        reports.push(parsed);
                    }
                }
            }
        }

        Ok(reports)
    }

    #[record_error_severity]
    #[tracing::instrument(name = "dagster.graphql_client.trigger_file_report_run", skip(self))]
    pub async fn trigger_file_report_run(&self) -> Result<LaunchPipelineResponse, DagsterError> {
        let query = r#"
mutation LaunchPipeline($executionParams: ExecutionParams!) {
  launchPipelineExecution(executionParams: $executionParams) {
    __typename
    ... on LaunchRunSuccess {
      run {
        runId
      }
    }
  }
}
"#;

        let execution_params = ExecutionParams {
            selector: PipelineSelector {
                pipeline_name: "file_reports_generation".to_string(),
                repository_location_name: "Lana DW".to_string(),
                repository_name: "__repository__".to_string(),
            },
        };

        let variables = json!({
            "executionParams": execution_params
        });

        let request = json!({
            "query": query,
            "variables": variables
        });

        let response = self
            .http
            .post(self.url.clone())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DagsterError::ApiError);
        }

        let response_data: LaunchPipelineResponse = response.json().await?;
        Ok(response_data)
    }
}
