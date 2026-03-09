use chrono::NaiveDate;
use dagster::{Dagster, graphql_client};

use crate::{
    AS_OF_DATE_TAG_KEY, MANUAL_SINGLE_REPORT_TAG_KEY, REPORT_DEFINITION_ID_TAG_KEY,
    REPORT_NAME_TAG_KEY, REPORT_NORM_TAG_KEY, ReportDefinition,
};

/// Domain-specific adapter over the Dagster GraphQL client.
///
/// Translates core-report concepts into Dagster-specific GraphQL types so that
/// job runners do not depend on the raw `graphql_client` structs.
#[derive(Clone)]
pub struct DagsterReportAdapter {
    dagster: Dagster,
}

impl DagsterReportAdapter {
    pub fn new(dagster: Dagster) -> Self {
        Self { dagster }
    }

    /// Launch a Dagster run for a single report definition.
    ///
    /// Returns the Dagster `run_id` on success.
    pub async fn launch_report_run(
        &self,
        report_definition: &ReportDefinition,
        as_of_date: Option<NaiveDate>,
    ) -> Result<String, dagster::error::DagsterError> {
        let input = graphql_client::LaunchFileReportRunInput {
            asset_selection: report_definition
                .asset_selection_paths()
                .into_iter()
                .map(graphql_client::AssetKeyInput::from_path)
                .collect(),
            as_of_date,
            tags: Self::build_run_tags(report_definition, as_of_date),
        };

        let response = self.dagster.graphql().launch_file_report_run(input).await?;

        match response.data.launch_run {
            graphql_client::LaunchRunResult::LaunchRunSuccess { run: Some(details) } => {
                Ok(details.run_id)
            }
            graphql_client::LaunchRunResult::LaunchRunSuccess { run: None } => {
                Err(dagster::error::DagsterError::ApiError)
            }
            graphql_client::LaunchRunResult::Error => Err(dagster::error::DagsterError::ApiError),
        }
    }

    /// Fetch a single Dagster run by its ID.
    pub async fn fetch_run(
        &self,
        run_id: &str,
    ) -> Result<Option<graphql_client::RunResult>, dagster::error::DagsterError> {
        self.dagster.graphql().file_report_run(run_id).await
    }

    /// Fetch the most recent Dagster report runs.
    pub async fn fetch_recent_runs(
        &self,
        limit: i32,
    ) -> Result<graphql_client::FileReportsRunsResponse, dagster::error::DagsterError> {
        self.dagster.graphql().file_reports_runs(limit, None).await
    }

    /// Fetch materialization logs for a run and parse them into reports.
    pub async fn fetch_reports_for_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<graphql_client::Report>, dagster::error::DagsterError> {
        self.dagster.graphql().get_logs_for_run(run_id).await
    }

    fn build_run_tags(
        report_definition: &ReportDefinition,
        as_of_date: Option<NaiveDate>,
    ) -> Vec<graphql_client::ExecutionTag> {
        let mut tags = vec![
            graphql_client::ExecutionTag {
                key: MANUAL_SINGLE_REPORT_TAG_KEY.to_string(),
                value: "true".to_string(),
            },
            graphql_client::ExecutionTag {
                key: REPORT_DEFINITION_ID_TAG_KEY.to_string(),
                value: report_definition.report_definition_id().to_string(),
            },
            graphql_client::ExecutionTag {
                key: REPORT_NORM_TAG_KEY.to_string(),
                value: report_definition.norm.clone(),
            },
            graphql_client::ExecutionTag {
                key: REPORT_NAME_TAG_KEY.to_string(),
                value: report_definition.friendly_name.clone(),
            },
        ];

        if let Some(as_of_date) = as_of_date {
            tags.push(graphql_client::ExecutionTag {
                key: AS_OF_DATE_TAG_KEY.to_string(),
                value: as_of_date.to_string(),
            });
        }

        tags
    }
}
