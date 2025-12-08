use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use tracing_macros::record_error_severity;

use super::{config::DagsterConfig, error::DagsterError};

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
pub struct RunResult {
    #[serde(rename = "runId")]
    pub run_id: String,
    pub status: RunStatus,
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
pub struct FileReportsRunsData {
    #[serde(rename = "runsOrError")]
    pub runs_or_error: RunsOrError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileReportsRunsResponse {
    pub data: FileReportsRunsData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonMetadataEntry {
    #[serde(rename = "jsonString")]
    pub json_string: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum Event {
    MaterializationEvent {
        #[serde(rename = "metadataEntries")]
        metadata_entries: Vec<JsonMetadataEntry>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventConnection {
    pub events: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LogsForRunResult {
    EventConnection(EventConnection),
    Error { message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLogsForRunData {
    #[serde(rename = "logsForRun")]
    pub logs_for_run: LogsForRunResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLogsForRunResponse {
    pub data: GetLogsForRunData,
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

        let response_data: GetLogsForRunResponse = response.json().await?;

        let mut reports = Vec::new();

        if let LogsForRunResult::EventConnection(event_conn) = response_data.data.logs_for_run {
            for event in event_conn.events {
                if let Event::MaterializationEvent { metadata_entries } = event {
                    for entry in metadata_entries {
                        let parsed: Vec<Report> = serde_json::from_str(&entry.json_string)?;
                        reports.extend(parsed);
                    }
                }
            }
        }

        Ok(reports)
    }
}
