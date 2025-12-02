#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![cfg_attr(feature = "fail-on-warnings", deny(clippy::all))]

mod config;
mod error;
pub mod graphql_client;

pub use config::DagsterConfig;
pub use error::DagsterError;
pub use graphql_client::{
    AssetEvent, AssetEventHistory, AssetKey, DagsterAsset, DagsterGraphQLClient,
    DagsterParsedReport, DagsterReportFile, DagsterRun, DagsterRunStatus, FileReportsRunsResponse,
    MetadataEntry,
};

#[derive(Clone)]
pub struct Dagster {
    client: DagsterGraphQLClient,
}

impl Dagster {
    pub fn new(config: DagsterConfig) -> Self {
        Self {
            client: DagsterGraphQLClient::new(config),
        }
    }

    pub fn client(&self) -> &DagsterGraphQLClient {
        &self.client
    }
}
