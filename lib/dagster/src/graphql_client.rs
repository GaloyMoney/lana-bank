use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use super::{config::DagsterConfig, error::DagsterError};

#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GraphQLError {
    message: String,
}

#[derive(Clone)]
pub struct DagsterGraphQLClient {
    http: Client,
    endpoint: Url,
}

impl DagsterGraphQLClient {
    pub fn new(config: DagsterConfig) -> Self {
        let endpoint = config
            .uri
            .join("graphql")
            .expect("Failed to construct graphql endpoint");
        Self {
            http: Client::new(),
            endpoint,
        }
    }

    #[tracing::instrument(name = "dagster.graphql_client.check_for_new_reports", skip(self))]
    pub async fn check_for_new_reports(
        &self,
        cursor: Option<String>,
    ) -> Result<FileReportsRunsResponse, DagsterError> {
        let query = r#"
            query FileReportsRuns($limit: Int!, $cursor: String) {
                runsOrError(
                    limit: $limit
                    cursor: $cursor
                    filter: { pipelineName: "file_reports_generation" }
                ) {
                    __typename
                    ... on Runs {
                        count
                        results {
                            id
                            runId
                            status
                            assets {
                                id
                                key {
                                    path
                                    __typename
                                }
                                assetEventHistory(
                                    eventTypeSelectors: [MATERIALIZATION]
                                    limit: 10
                                ) {
                                    cursor
                                    results {
                                        __typename
                                        ... on MaterializationEvent {
                                            stepKey
                                            solidHandleID
                                            runId
                                            timestamp
                                            label
                                            description
                                            metadataEntries {
                                                label
                                                description
                                                __typename
                                                ... on JsonMetadataEntry {
                                                    jsonString
                                                }
                                                ... on PathMetadataEntry {
                                                    path
                                                }
                                                ... on UrlMetadataEntry {
                                                    url
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "limit": 100,
            "cursor": cursor,
        });

        let request = GraphQLRequest {
            query: query.to_string(),
            variables: Some(variables),
        };

        let response = self
            .http
            .post(self.endpoint.clone())
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(DagsterError::ApiError);
        }

        let gql_response: GraphQLResponse<FileReportsRunsData> = response.json().await?;

        if let Some(errors) = gql_response.errors
            && !errors.is_empty()
        {
            tracing::error!("GraphQL errors: {:?}", errors);
            return Err(DagsterError::ApiError);
        }

        let data = gql_response.data.ok_or(DagsterError::ApiError)?;

        match data.runs_or_error {
            RunsOrError::Runs { count, results } => Ok(FileReportsRunsResponse {
                count,
                runs: results,
            }),
            RunsOrError::PythonError { message } => {
                tracing::error!("Dagster PythonError: {}", message);
                Err(DagsterError::ApiError)
            }
            RunsOrError::InvalidPipelineRunsFilterError { message } => {
                tracing::error!("Dagster InvalidPipelineRunsFilterError: {}", message);
                Err(DagsterError::ApiError)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FileReportsRunsData {
    runs_or_error: RunsOrError,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "__typename")]
enum RunsOrError {
    Runs {
        count: i32,
        results: Vec<DagsterRun>,
    },
    PythonError {
        message: String,
    },
    InvalidPipelineRunsFilterError {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DagsterRun {
    pub id: String,
    pub run_id: String,
    pub status: DagsterRunStatus,
    pub assets: Vec<DagsterAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DagsterAsset {
    pub id: String,
    pub key: AssetKey,
    pub asset_event_history: AssetEventHistory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetKey {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEventHistory {
    pub cursor: Option<String>,
    pub results: Vec<AssetEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum AssetEvent {
    MaterializationEvent {
        #[serde(rename = "stepKey")]
        step_key: Option<String>,
        #[serde(rename = "solidHandleID")]
        solid_handle_id: Option<String>,
        #[serde(rename = "runId")]
        run_id: String,
        timestamp: String,
        label: Option<String>,
        description: Option<String>,
        #[serde(rename = "metadataEntries")]
        metadata_entries: Vec<MetadataEntry>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__typename")]
pub enum MetadataEntry {
    JsonMetadataEntry {
        label: String,
        description: Option<String>,
        #[serde(rename = "jsonString")]
        json_string: String,
    },
    PathMetadataEntry {
        label: String,
        description: Option<String>,
        path: String,
    },
    UrlMetadataEntry {
        label: String,
        description: Option<String>,
        url: String,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DagsterRunStatus {
    /// Runs waiting to be launched by the Dagster Daemon.
    Queued,
    /// Runs that have been created, but not yet submitted for launch.
    NotStarted,
    /// Runs that are managed outside of the Dagster control plane.
    Managed,
    /// Runs that have been launched, but execution has not yet started.
    Starting,
    /// Runs that have been launched and execution has started.
    Started,
    /// Runs that have successfully completed.
    Success,
    /// Runs that have failed to complete.
    Failure,
    /// Runs that are in-progress and pending to be canceled.
    Canceling,
    /// Runs that have been canceled before completion.
    Canceled,
}

#[derive(Debug)]
pub struct FileReportsRunsResponse {
    pub count: i32,
    pub runs: Vec<DagsterRun>,
}

/// A report file with its path and type from Dagster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagsterReportFile {
    pub path_in_bucket: String,
    #[serde(rename = "type")]
    pub file_type: String,
}

/// A parsed report from the Dagster metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagsterParsedReport {
    pub name: String,
    pub norm: String,
    pub files: Vec<DagsterReportFile>,
}

impl DagsterRun {
    /// Extract all reports from the run's asset materialization events
    pub fn extract_reports(&self) -> Vec<DagsterParsedReport> {
        let mut reports = Vec::new();

        for asset in &self.assets {
            for event in &asset.asset_event_history.results {
                if let AssetEvent::MaterializationEvent {
                    metadata_entries, ..
                } = event
                {
                    for entry in metadata_entries {
                        if let MetadataEntry::JsonMetadataEntry {
                            label, json_string, ..
                        } = entry
                            && label == "reports"
                        {
                            if let Ok(parsed) =
                                serde_json::from_str::<Vec<DagsterParsedReport>>(json_string)
                            {
                                reports.extend(parsed);
                            } else {
                                tracing::warn!("Failed to parse reports JSON: {}", json_string);
                            }
                        }
                    }
                }
            }
        }

        reports
    }
}
